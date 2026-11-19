---
plan_id: PLAN-980
status: archived
feature_name: AutoUI 的 ACE 基础接入
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-19
plan_revision: 20
current_step: 18
total_steps: 18
depends_on: [PLAN-960, PLAN-963, PLAN-967, PLAN-968, PLAN-969, PLAN-972, PLAN-973, PLAN-976, PLAN-978, PLAN-979]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-autoui.md]
touched_goals: [GOAL-002]
---

# [PLAN-980] AutoUI 的 ACE 基础接入

## 0. 变更摘要

### 2026-11-19 · AutoMan 消费目标

M2 AutoMan UI 三命令的两端生产验收由 982 联同 1072（Vue/A2TS）与 1074（A2R/Iced）负责。A2Ark/A2Jet 按既有 Demo 保留来源，产品支持在 v0.7 或 v0.8 重新定方案。Tauri 按 1073 的独立产品合同验收。

### 2026-11-19 · 基础范围与正式后继

保留 T-HELLO、T-01、B-DIALECT/STATE/NATIVE 的已验基础范围与正式接口。T-HELLO-D 核实际已合入源和验收证据后记账。980 保留当前 r21 的 package SFC 消费修复，作为 981 合入的明确接口交接。

本节是原分波、跨计划和验收条款的当前 owner 映射。§5–7 保留全部原技术合同、分母、独立 oracle、负控与资源约束，按本表接手。已有收据仅按其实际来源及边界复用。基础归档取保留范围的正式验收；M2 产品继续按三波与各模式验收。

| 原工作 | 正式接手方 | 原验收与证据 |
|---|---|---|
| B-AV-I/V/R；DIALECT/STATE/NATIVE 的 B 产品 AV | [1071](1071-autoui-w1-av.md) | 完整 U01–U06/Widgets、V-UI-W1、V-UI-SOURCE/FULL 的 AV 格、V-EVT-AV-CONSUME、真窗/输入/恢复/资源 |
| B-WEB-I/V/R、PHASE-LAMBDA-IF | [1072](1072-autoui-vue-web.md) | V-B-WEB、V-EVT-WEB-CONSUME、V-TS-CONSUMER-UI Vue 格、T-LIF-1..5/AC-LIF-01..07 与原分母 |
| B-TAURI-I/V/R | [1073](1073-autoui-tauri.md) | V-B-TAURI、V-TS-CONSUMER-UI Tauri 格、invoke/事件/权限/关闭重开 |
| B-RUST-I/V/R | [1074](1074-autoui-w3-a2r.md) | V-B-RUST、V-UI-W3、V-UI-FULL A2R 格；AXR0 按原有界证据复用 |
| AC-NODE-DESKTOP、AC-AC-STAGED 的 B 范围 | [1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md) | V-AC-WAVE2、真实 AC 业务/同版宿主、错误/资源/恢复 |
| B-PARITY-I/V/R、AC-07、AC-NODE-COMPAT 和 AC-UI-WAVES 剩余全模式范围 | [985](985-ace-m2-certification.md) | 各生产方逐格实现和核验。985/B 按 V-UI-SOURCE/FULL、V-UI-WAVE-RECEIPT、V-UI-NODE-COMPAT 复核原组件/主题/生命周期/错误/资源和删格负控 |

原 A 时段事实、字段及交接接口由基础计划提供。后继负责对应实际产品的实施、验证、独立复审及交付。未关闭行随 owner 迁移，既有用例、约束和旧证据来源继续可追。

2026-11-19 r19 历史修订：新增 **PHASE-LAMBDA-IF（lambda-if 刀）**——§9 r18 边界②「005 lambda-if：文法臂已开（view-stmt 共用）但 lambda 投影仍 assigns-only+foreign——值函数铸造需结构化（下一小刀）」的用户指令收口阶段（2026-11-19 裁定：不另立新计划，直接入本包 phase；曾取号 PLAN-1071 同日撤销，`.next-id` 恢复 1071、无新计划文件落库）。设计/验证/验收/步骤见 §5/§6/§7/§8 新阶段块（非任务行，不改 32 任务/12 完成口径）。**零文法改动**（grammar-id 零漂移）、复用 on-if 五环 c/d/e 共享层；主战场=lambda 投影接线（`project.rs:5774-5792` 接 `project_view_stmt_children`+`Lambda` 增 `stmts`）+值函数铸造 If 递归降版+两个验证面深走查；lane/UI 侧预期零/近零产品代码改动（机理链假设，T-LIF-4 bounded probe 实证）。本轮仅规划落档（零代码/零建树/零运行/零合入）；原 AC/V 与规范主体不变。

2026-11-18 r18当前：PHASE-④-EXT **七项裁定落档**（D-④-1=A 先行、D-④-2=开放符号表导入+981 会签、D-④-3=首切片保留宽数值拒绝、D-④-4=foreign AV 后置、D-④-5=TS 拒粒度维持、D-④-6=host 表不建、D-④-7=小范围验包绑定/保留作者语法）；**981 会签修正实施路径**（`81b0a03b1`：不开 age-component 面，多模块闭包走 `age-package`，age-component 单文件 `use` 改具名拒，硬条件=惰性导入，grammar 零改动），**用户复核接受并选 R1′**（先闭包→派生真实例→AV/SFC；表面补齐并行）；**④-A 首片 #1/#2a/#2b/#2c 全交**（点分 `use` 投影+引用驱动导入 lang `0db44dfdf`；AV 观察 `[4,2]`→`[20,10]`；包感知桌面桥+真窗 ui `145c4f13`；包片段 lane+同源 SFC script 段 ui `404dc442`）；**P3 `style` 会签材料已备**（126 行实测；style≡str const/fn；两值 grammar-id 影响；待 981 回签后落文法）。**U01 #14 已收口落主**（lang `95b0ccb49`/ui `91d549ea`）。32 任务/12 完成口径、原 AC/V 与规范主体不变。

2026-11-18 r17当前：新增 **PHASE-④-EXT** 设计阶段（§5/§6/§8/§10），产出 use-import/外部调用进受检体的**外部名域合同草案 v0** 供用户/976 后继拍板；32 任务/12 完成口径、B-AV/B-WEB 状态、原 AC/V 与规范主体均不变，本轮零代码/零建树/零运行/零合入。并具名订正 2026-11-18 分母冻结行的宽数值口径（003 `double`×2、011 `float` 去注释 7 处且全在 helper、031i `float`×3 字段漏列）与 35 例导入 census（34/35 含 use、17 例真实引用导入名）。

2026-11-16 r15当前：EVENT-SPLIT-20261116明确A/B逐切片等待，三消息缺口归981原LANGUAGE/SESSION、980原AV/WEB消费者及985原认证任务；复用980/NATIVE与#6i，A关最小真实事件/保真/安全拒绝，B关完整payload/绑定/WEB及三端产品。旧980/981/985整包互等不再作为前置；原任务/勾选/完成数和全部验收保留，仅规划。

2026-11-16 r13当前：M2-A由B-NATIVE-I/V/R交AV＋Rust FFI驱动Iced的可复用Node桌面入口；已交HELLO仅为有界真窗，A2R/AC分别取真实后端证据。M2-B AC桌面由既有1068/T-04承接，交981/FUSION/AUDIT和985/B；保留任务ID/历史收据，新范围NATIVE I/V/R重新打开（15/29→12/29）。本轮仅规划。

2026-11-16 r12最新阶段：AV/AC/A2R最终完整一致；A期AC基础以外差额可逐行交M2-B，1067/1068承接原生异步与三端补齐，A2C完整对齐增强。原任务/勾选/实际进度不变；新阶段条款不能冒实现或复审已过。

2026-11-16 r11（当前，用户范围裁定）：**JET/ARK 两后端归 M3（v0.7），从 v0.6 移除**——B-JET-I/V/R 与 B-ARK-I/V/R 六任务行撤出执行清单（原条目及 AC/V 留 §9 历史），total 35→29、进度口径 12/29；AC-B-JET/AC-B-ARK/V-B-JET/V-B-ARK 随批归 M3；PARITY 的 v0.6 联合覆盖改为现役模式（NATIVE/WEB/RUST/TAURI），JET/ARK 的模式覆盖由 M3 联合验收补。**WEB（Vue+TS）保留 v0.6**（前置仍为 L976）。本裁定同步 M2 路线与 m2-autoui 规范；此前 §9 round 17 的 JET 环境阻塞记录由本裁定取代（归 M3 后 v0.6 不再含该腿）。本包已交付各批（DIALECT/STATE/HELLO/RUST-AXR0）不受影响。

2026-11-16 r11（原 Godot/TS 范围裁定）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包WEB/TAURI/ARK等生产TS/JS能力仍required，真实消费976新受检provider；不保旧TypeScriptTrans fallback，不以Hello或少量Demo代完整现用UI合法表面。DIALECT已交证据与3/35保持，Godot不成为本包或979/981交接前置。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

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

### 本轮 lambda-if 阶段修订的授权与取材（2026-11-19）

用户指令（2026-11-19）：§9 r18 边界②「值函数铸造结构化=下一小刀」直接入 PLAN-980 新 phase，不另立新计划（已取号 PLAN-1071 同日撤销，`.next-id` 恢复 1071）。本轮=规划落档（§5/§6/§7/§8 阶段块+frontmatter r19），零代码/零建树/零运行/零合入；实施授权=本阶段块，执行沿唯一树 lang-980（worktree `D:/autostack/.wt/lang-980/{auto-lang,auto-ui}` @ plan-980-dev），材料入既有 `plan-artifacts/plan-980/` scope。

取材全部 master 实测（lang `15cae2ded` / auto-ui master，2026-11-19）：文法 `experimental/ace/grammars/auto-source.at:578-592,1063-1068`（lambda 体=`list-nl age-view-stmt`，if 臂已共用——「文法臂已开」实证）；投影缺口 `cst/project.rs:5774-5792`（if 形落 foreign）；结构化投影器 `project.rs:7141-7199`（`project_view_if`/`project_view_stmt_children`，handler 面先例 `:6151-6199`）；铸造 `age_component.rs:1362-1409,2150-2260`（`collect_lambdas`/`lambda_value_fn` assigns-only）；深走查先例 `age_component.rs:1892-1916`（`walk_stmt_exprs`）；验证面消费者 `age_component.rs:1935-1965`（`validate_channel_value_use`）+ `cst/frontdoor.rs:1199`（`collect_item_exprs` Lambda 臂）；降版/发射 `a2x-ts lower/mod.rs:412`（`hir::Stmt::If` 臂）+ `fragment.rs:172`（`emit_stmt_fragment`）；lane `auto-ui crates/auto-ui-runtime/src/ui/ace_ts_provider.rs:492-660`（`stmt_fragments_in` 根块 If 整语句发射=on-if 刀 e 环已交、`lambda_fn_covering` 区间相交）+ `ace_vue_script.rs:688-780`（`lambda_body_regions`/`pair_lambda_regions`——括号深度配对天然容纳嵌套 if 大括号、mint 计数序失配全体缺席）。规范依据 `docs/specs/ace-migration/m2-autoui.md` §13.12（#6k 值函数道）。lane「零产品代码改动」为机理链**假设**，T-LIF-4 bounded probe 实证后方可当结论。

## 5. 详细设计

> 原完整合同保留在本节。现行实施与验收归属见 §0 的移交表，基础收口任务见 §8。

### AGE三路线接面补充（2026-11-17，A分母不扩）

原MIN-A/事件A/B与全部既有32任务/进度不变。统一[AGE三路线strategy](../design/strategy/age-m2-three-route-strategy.md)。UI-W1-AV、UI-W2-AC、UI-W3-A2R只签原UI示例/Widgets全集，**不等待新增UI099–102或AutoForge**；新增引擎cap由981/1068独立域格承接，不倒置成B-AV/PARITY各波交付前置。

B-AV-I/V/R可在完整W1前将已验文本、按钮、文本输入、列表/选择与场景预览宿主子面独立R/fold，交精确组件/props/事件payload/绑定/权限/资源与源码/ABI身份；供Forge001使用，不能把MIN-A Counter当完整编辑组件或把早期子面当UI-W1。事件缺口仍归981 EVENT-B，普通widget兼容仍同Node/Schema，不另开旧通路。

B-RUST与981 RUST/FUSION协作第三波后新增cap-A2R（MAP/ASSET/PBR/CLIP），保持同业务源/资产/独立oracle，真实rustc/运行；原UI-W3完整基线先按原分母交，不反等全部cap。额外cap来源由981交AG031逐行收，本包不重造renderer/FFI/打包器。原WEB/TAURI/TS及其验收义务保持，Godot不复活。新cap/Forge不加985/A，原全包final义务不减。

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

### 本包逐批解锁，不是三个条件的AND总门

| 原批次/动作 | 最小恢复条件 | 明确不等待 |
|---|---|---|
| 已交NATIVE A的来源消费/身份核对 | 981 T-01读取上述正式源、测试断言及ABI；缺文件/版本由980原NATIVE交接owner补记具体缺件 | 980全包、WEB/TAURI、985/A签发、1068、981整包 |
| B-AV-I/V/R（W1） | M2A-08；每组实际所需981 EVENT-B/领域接口正式可用 | UI-W2-AC、W3、完整WEB/TAURI或OS |
| B-WEB-I/V/R完整受检闭合 | M2A-08及所用IF-981-MSG/PAYLOAD/PARAMS-B、976正式provider；已落主切片逐项复用 | 980 B-AV全波、1068整包、981整包、983全产品 |
| B-RUST-I/V/R新增图形W3 | UI-W2-AC及所用981 RUST/领域正式输入；原AXR0只复用已验范围 | OS/985 B最终、1068全包最终三端汇总 |
| B-TAURI | WEB所用正式功能与原invoke/权限接口 | 与其无关的整包/后端收口；最终合同仍全验 |
| B-PARITY及T-05/06/07 | 分波先接各自真实收据；整包终验才汇合全部现役模式/原完整义务 | 不以PARITY整项/全包完成反卡任何前一波 |

980在B可按实际功能子集消费已交事件接口；未支持行必须pending/具名拒绝，不能把旧编译通路或lane缺席算ACE生产通过。旧round39/40等待策略的事实与用户当时选择留档，本次新合同改为逐批满足即具备规划恢复条件，不自动恢复其目标。

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
| 980/JET | 原 Demo 留档，产品适配至 v0.7 或 v0.8 再定 | v0.6 无产品验收 | auto-ui |
| 980/ARK | 原 Demo 留档，产品适配至 v0.7 或 v0.8 再定 | v0.6 无产品验收 | auto-ui |
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

### EVENT-SPLIT拟议规范增量（交付时由原merge任务沉淀）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-EVENT-PHASE | modify（拟议） | `docs/specs/ace-migration/m2-autoui.md` | 交付只沉淀实际完成的A空载荷/声明保真/拒绝、B typed payload/参数/来源与各consumer；明确逐批解锁，r23分母与完整义务保持 | 三缺口具体owner与避免整包循环 | AC-EVENT-PHASE |


### PHASE-④-EXT：外部名域合同（use-import/外部调用进受检体）草案 v0（2026-11-18，待拍板）

**阶段性质与授权。** 用户 2026-11-18 裁定：④ 域合同先做，并在本计划内新增 phase（不新立项、不新计划号、不改原任务）。本 phase 是**设计/决策准备阶段**（owner_stage: design）：产出合同草案 v0 与决策请求，供用户/976 后继拍板。不实施、不改语法/实现/规范主体、不建树/不运行产品、不勾选任何原任务；32 任务/12 完成口径与 B-AV/B-WEB 状态不变。本 phase 不占 phase fold 号（#14 仍属 U01 首切片）。

**证据基座（静态读取 + 本次 census 实测；未运行产品/门禁）。**

① **作者面 census（35 例分母，本次实测；去注释/字符串）**：34/35 含 `use` 行（仅 032 无）。形态：点分成员 `use <m>.<n>: A, B` 34 例、符号表 `use <mod>: a, b` 16 例、包声明 `use { package: … }` 5 例、命名空间 `use auto.*` 2 例、`use back.api: …` 3 例（025/027/031i；017 的该行为注释，且注释明言 VM 轨不可解析）、`use.rs`/`use.c` **0 例**。导入符号共 137 个；**17 例在 `use` 行之外真实引用导入名**（007/011/013/015/016/017/019/020/023/025/026/027/029/030/031i/031p/041），**另 17 例导入完全惰性**（只 themes 名，代码用 `"Auto"` 字面量）。→ 墙不在"有 use 行"，而在**被引用的外部名**与**无 use 行的内建命名空间**（003 `math.round`、011 `math.*`/`auto.clipboard`、027 `back.api`/`auto.*`）。census 方法：逐文件读 `src/front/app.at`，剥 `//` 注释与字符串后统计 `use` 行形态/导入符号/符号引用/`double|float|f64` 词；明细落 `plan-artifacts/plan-980/imports-census-20261118.txt`（merge 时随 scope 删除，数字已折入本节）。

② **语法面（grammar profile 门，实读 `grammars/auto-source.at`）**：`age-use-module-item`（`use m.n: syms`）对 age-component/age-package 开——`use back.api: …` 属此形，**可解析**（且今日零事实）；`age-use-sym-item`（`use mod: syms`）**仅 age-package**；点分无符号 `use auto.fs` 在 age-component 无规则（仅 packages 面 `use-item-p`）；包声明 `use { package: … }` **在 ACE 任何 profile 都无规则**（5 例真实源连解析都不成立——不是绑定问题）；`use.rs`/`use.c` 有规则并投影为 `Item::UseRs`（967 F0 谱系），package 模块面反向具名拒（`source.use-rs.surface`，"belongs to the single-module surface"）。

③ **投影/绑定面**：组件前门 `project_component_nodes` 不读 `age-use-module-item` → **`use` 行=解析后零事实**（002 冻结源带该行仍是既有永久用例 `plan980_dialect_full` 的输入并能编译=实证；被引用名未绑定，按推断落 Bind 处 `source.undefined`，待 ④-A 实测确认）；`hidden_name_conflict` 对 `UseRs` 返回 None（不参与保留名检测）。age-package 侧有真实导入绑定（960/981：imports/exports、按引用查符号、跨模块调用）。

④ **受检体面**：handler 只收平铺赋值 + 头参数通道调用；`body_foreign`（let/if/非头参数调用）→ 检查函数不铸、lane 缺席、ops 零效果拒（§13.16）。字段值域=int/i32/bool/str；float/f32/其它 ident 在派发/字段处 `source.age.component-unsupported`（`age_component.rs` 值域无浮点车道）。

⑤ **车道/后端面**：TS fragment 对**任何** foreign 声明整 view 拒 `a2ts.foreign-unsupported`（`fragment.rs::validate_view_admission`，粒度=view 级非 per-call）；976 包级 lane（`compile_package_product`，模块/use/init 扁平化+跨模块调用）已有跨模块真实发射/执行实证（m2-typescript §消费接口 1，`tsready_976`）；SFC 生成道的 fragment 程序上下文只有 scalar enum/struct/unit variant 集，**无被调 fn 名映射**。AV 执行 Auto 代码；967 冻结可调用域=view+2×i32 七函数；组件场景派发无宿主进程。A2R 同 `a2r.foreign-unsupported`（F0 例外）。

**事实订正（覆盖 §9 2026-11-18 分母冻结行三处口径；原行保留不删）。** ①003 的宽数值是 **`double`×2 个 model 字段**（非 "f64"），其 `math.round` 是命名空间调用、`use` 行本身惰性；②011 的 `float` 去注释后 **7 处，全在同文件 helper（`fmt_num(v float)` 等）与局部**，**model 字段全 int/str/bool**——"f64×14" 与实测不符，011 真墙=`use prog_util: syms`（形态未开）+8 个被引用符号+`math.*`/`auto.clipboard`；③**031i 前表漏列**：`zoom/offset_x/offset_y` 三个 `float` **model 字段**。→ 宽数值 model 字段实测仅 **003（double×2）、031i（float×3）** 两例，**不是 35 例主导墙**；优先级应为 **名面 ≫ 宽数值**。

**目标域设计（名面 N × 轴 A–F）。**

| 名面 | 内容 | 声明源（权威） | 今日状态 |
|---|---|---|---|
| N1 组件本地 | model 字段/msg/handler 参数/头参数通道 | 组件前门 | 已交（#6i/#6p） |
| N2 Auto 模块名 | 同/兄弟模块 `pub fn`/`const`/`style`/store/type | age-package 闭包绑定（960/981） | age-component 未接；符号表形态未开 |
| N3 foreign 声明名 | `use.rs`/`use.c` provider 函数 | 938/939 SourceDecl（manifest 非权威）+宿主身份 | 单模块面已投影；组件面未进车道；TS 整 view 拒 |
| N4 内建/宿主名 | `math.*`/`auto.*`/`back.api.*`（可无 use 行） | **新**：后端宿主适配表 | 无合同；只能经旧道 |

- **A 名面**：非 N1 名进入受检体前必须有声明源；无声明=具名拒 `source.import.unresolved`。`use` 行零事实必须闭合（绑或不绑都要具名），不得继续静默；`body_foreign` 保留兜底但**不再作为设计承载**。
- **B 类型**：承载=N1 值域 + 961 标量/记录闭包；宽数值不入本合同（D-④-3）；list/map/泛型/变体=域外具名拒。
- **C 效应**：外部调用=效应；必须登记（967/939 资源/凭证、968 异步、972–975 stdlib 端口）；未登记/未授权=不执行；handler 内 `~T` 先拒（调度归 968/981）。
- **D 车道**：按承载矩阵；**不部分承载**——缺任一被调 fn 的发射/生效=整 handler 缺席（#6p 红线推广）。
- **E 身份**：闭包 hash+host 表版本+foreign digest 进 artifact/manifest/证据；漂移=缓存与旧证失效（985 拒过期）。
- **F 失败**：`source.import.{unsupported|unresolved|backend|effect|type}`，span 到名字、零效果；`a2ts.foreign-unsupported` 粒度（view 级整拒 vs per-call）登记 D-④-5。

**承载矩阵（"承载"=可执行/可发射，非"能解析"）。**

| 名面 | AV（组件/窗口） | TS fragment（SFC） | TS 包级 lane | A2R |
|---|---|---|---|---|
| N2 Auto 模块名 | 需组件面闭包（多 unit 进同一 AV bundle）；纯 Auto 调用无新 ABI（推论） | 需生成器道把被调 fn 发射为 SFC 模块级函数并做名字映射（fragment 上下文无 fn 名） | 已支持（976 包 lane 扁平化+跨模块调用，`tsready_976` 实证） | 需同源闭包发射（W3） |
| N3 foreign | 需 967/968 具体调用入口（今日仅 view+2×i32 七函数）；场景派发无宿主 | 保持 `a2ts.foreign-unsupported`（整 view 拒） | 同拒 | `a2r.foreign-unsupported`（F0 例外） |
| N4 内建/宿主 | 需 host 表→VM/FFI 映射+授权 | 需 host 表→SFC import | 需 host 表 | 需 host 表 |

→ **N2 是唯一不新增信任面的名面**（三后端都是"把 Auto 代码带进来"）；N3/N4 都触碰授权/信任面。

**三方案（v0 历史记录；A 的**入口选型**已被 981 会签 + 用户 2026-11-18 裁定取代——整体走 `age-package`，不再开 age-component；B/C 仍为后续选项）。**

- **A（④-A 最小闭包）**：只接 N2——**作者继续用原 `use` 拼写**，多模块闭包整体按 `age-package` 面编译（**不**开 age-component 面、不做混合面闭包），以 960/981 包闭包绑定/verify 为唯一权威；导入只取 `pub fn`/`const`（`style` 作 const 形）。AV 把闭包 unit 一并编译；TS 生成器道发射被调 fn；A2R 同源。N3/N4 保持具名拒。**解锁**：17 例被引用导入的**模块子面**（stylekit/prog_util/store 引用族）；**不等于这些例整例可编译**——其余缺口（design/style/routes/包声明/宽数值）仍按原归属另行闭合。**成本**：包面闭包导入接线+惰性绑定+三后端闭包发射（AV 最轻、TS 生成器道最重）；**无新 ABI/授权面**。
- **B（④-B foreign 优先）**：接 N3——`use.rs`/`use.c` 进组件受检体，签名走 938/939+verify 既有 foreign 检查；AV 需新 ABI 格+授权；TS/A2R 保持拒。**35 例实测 0 例含 `use.rs`**，直接收益≈0；真正消费者是网络/DB/文件产品线（983 谱系，归其 owner）。**成本最高**。
- **C（④-C host 表）**：接 N4——中立"宿主名表"（名字→签名/效应/后端集合/授权行）。**解锁**：003 `math.round`、011 `math.*`/`auto.clipboard`、027/031p `auto.*`、027/031i `back.api`（约 5 例）。**成本中**；**风险**=新信任面（名字可逃逸受检），须 hash/审查/能力行，不得成为绕过 967 的后门。
- **推荐：A 先行**（与 U01 余量/生成器道并行），B/C 分立决策；D-④-3 独立给 981。

**合同条款草案 v0（历史；已被 §5 v2 的 S0–S8 取代，保留供追溯）。** C1 名面准入（有声明源才进受检体，无声明具名拒）；C2 形态准入（未开形态=具名 `source.import.unsupported`，不得落通用首 token 错、不得静默丢）；C3 类型（按声明源真实签名；域外具名拒）；C4 效应（资源/凭证/异步/授权未登记=拒，零效果）；C5 承载（矩阵逐格实测；不部分承载）；C6 身份（闭包 hash/host 表版本/foreign digest 进身份，漂移失效）；C7 拒绝粒度（`a2ts.foreign-unsupported` 是否收窄，D-④-5）；C8 禁用形态（不得"能解析"当"已绑定"；不得旧 TS 委托/旧 VM 承载；host 表不得成为未受检逃逸口）。

**判别力（拟建永久反例，本 phase 不建）**：未声明名调用（拒+零效果）；声明源缺符号（拒）；未开形态（具名拒非首 token 错）；跨后端不承载（整 handler 缺席+兄弟保全）；闭包缺 fn 发射（不部分）；身份漂移（旧证失效）；host 表越权（拒）。

**阶段退出条件。** ①本节+§6 V-DOMAIN-④+§10 决策表落档；②承载矩阵每格标注"已验/推论"（本版：N2-TS 包级=已验；N2-AV/N2-TS-fragment=推论）；③D-④-1..7 有裁定；④裁定后切成 B-AV/B-WEB 具名切片（原任务行/AC 不变，仍走 §9 收据+phase fold）。

**非目标。** 不改语法/实现/规范主体；不勾任务、不改 12/32；不降低 AC；不新立项；本阶段不跑产品门禁（census 为静态读取）。

**裁定落档与执行顺序（2026-11-18，用户）。** 七项全部裁定（原表见 §10；裁定值：A 先行 / 开放符号表导入+981 会签 / 宽数值首切片保留拒 / foreign AV 后置 / TS 拒粒度维持 / host 表不建 / 小范围验包绑定且保留作者语法）。执行顺序：①先收口 U01 #14（独立复审+按适用范围核验真窗输入+记录未实跑项→合入；合入后把可消费 UI/事件源、命令、oracle 按版本交 981）；②启动 ④-A 首切片（真实示例：原 `use`→共享绑定→跨模块 helper 调用→AV 窗口可观察结果；同源验 SFC 被调函数发射与名字映射，尽早证伪/证实承载矩阵两格"推论"）；③UI058 T-02 与 ④-A 并行（981 已交 `IF-981-GFX-AV-SEAM` 冻结 f32 注解/运算、`spawn` 拼写、getter 轴参数与预算映射，再由 UI058 T-03 接真实宿主；980 WEB 收口不排其前）。

**④-A 首切片合同 v2（981 会签版，2026-11-18；取代 v1——v1 的"开 age-component 面"被 981 否决）。**

- **S0 会签结论（981 r18，lang master `81b0a03b1`；981 §9 会签节）**：④-A 方向与"包绑定器唯一权威"**通过**；**不开 age-component 面**——多模块闭包整体走 `age-package` 面（两形已在、多模块已有永久用例）；`age-component` 保持单文件，其 `use` 行改具名拒（S7）。依据（静态）：绑定器对每 unit 断言单一 face 的 grammar-id（`binding.rs` L657-669）⇒ 混合面闭包今天不可表达；且 age-component 无 `pub fn`/`pub const` 规则、`themes` 模块的 `pub theme` ACE 不可解析。
- **S1 入口/面**：整个 widget 工程按 `age-package` 面编译（`compile_age_package`，含 `LegacyWidgetAlias`/scenario/模块图）；不新增第二绑定器、不源拼接、不做"入口组件面+依赖包面"混合闭包。
- **S2 作者形→绑定**：`use mod: a, b` ≡ `path=[mod, sym]`；`use pkg.mod: a, b` ≡ `[pkg, mod, sym]`（同命名空间/同可见性/同错误）；`use a.b`（2 段无符号表）是 item 导入，**不得**当"模块+符号"。
- **S3 惰性（引用驱动）绑定——硬条件**：未被子引用的导入不解析、不建模块边、不要求目标模块在场；被引用必须解析，失败具名拒。依据：34/35 例含 `use`、17 例纯惰性，而 `themes` 模块 ACE 不可解析——eager 绑定会让 34 例全红；该差异**以 face 为界，不改 plain 包面**。
- **S4 符号种类**：值面=`pub fn`/`pub const`（`style` 作 const 形）；类型导入只作注解；`node` 导入不进值面（进值位置须具名拒，不静默）；模块 store 恒拒（`source.module-private`）。
- **S5 失败面**：沿用包绑定码表（`source.module-missing`/`-private`/`-reexport-private`/`-symbol`/`-path`/`-cycle`/`-stale`）；新码必须具名且 span 到名字（不许 @0）。
- **S6 身份**：`ace.age.component/v1`/`ace.age.package/v1`/`ui.widget-decl/v1` **不变**；导入只进闭包身份（模块边/`input_hash`——改 helper 源必须变）。若实现最终仍改文法，981 在**同一提交**重基线 `plan981_grammar_scope.rs` 且 13 邻居逐字不变。
- **S7 fail-closed 补口（可独立小修，不必并入 ④-A）**：组件单文件入口对 `use` 行"解析后零事实"→改具名拒 `source.import.unsupported`（"单文件面不解析导入，请走 AGE 包入口"）。
- **S8 门禁与永久 case**：lang `cargo t ace-source ace-av`；UI `ace_node_desktop`（含真窗）+ `ace_ts_provider` + vue 门 77/82（chart×5 具名预存）；永久 case=跨模块 helper 值、缺模块/缺符号/私有符号三负控、**惰性未引用不要求模块在场**、闭包身份漂移（改 helper 源 `input_hash` 变）、SFC 发射映射文本断言、AV 真窗可观察结果。**用户 2026-11-18 验收判据（最小三件）**：①**未被引用的 `themes` 导入不拉入模块**（模块可缺席）；②**真正被引用的 helper 进入闭包并影响运行结果**（值随 helper 源变化）；③**缺模块/缺符号/私有符号具名拒绝**。引用驱动**只限 ④-A 的符号导入**；普通包面既有初始化语义（`init`/`use` 的 eager 部分）保持不变。
- **首个真实示例（以可解析性实测选例）**：被引 helper 须在 int/str/bool/record 域且闭包内**被引用**模块均可解析。候选重评：011（`prog_util` 纯 fn；但 app.at 自身含 f64 helper 与 `math.*`/`auto.clipboard`）、016（`calendar_util` 纯 fn；`stylekit` 被引用且其模块形状待核）；**不得**以合成 fixture 冒充真实示例，也不得把示例整例通过当切片完成。
- **非目标（v2 明确）**：`use { package: … }`（5 例，任何面无规则）、`use auto.*`（命名空间形无规则）、N3 foreign、N4 host 表、宽数值——保持具名拒，归 ④-B/④-C/976 后继/981。

**交付状态（2026-11-18，R1′）**：**S1 点分 `use` 投影 + S3 引用驱动（惰性）导入已交**（lang `0db44dfdf`）。**S4 AV 执行/观察面已交**（`observe_age_package`）。**S4 真窗已交**（`PackageStateEngine` + `load_package` + 窗口 exe `--package`）。**S5 同源 SFC script 段 + 浏览器执行档已交**（`PackageFragmentLane` 发射；playwright/chromium 中 base 点击 2→4、edited 10→20——同一份发射文本）。**完整 SFC template 未交（阻塞已实测）**：既有方言解析器（`UiDialect`）**不认 AGE `node`/匿名 msg 源**（实测 20 个错误），需"view 树→Vue 模板"生成器道或方言 AGE 支持——下一片。**S0.5a 表面补齐**按 R1′ 与 981 会签并行分切片。

**S0.5 真实示例可解析性实测（2026-11-18，临时探针，已删；证据 `plan-artifacts/plan-980/closure-surface-probe-20261118.txt`）。**方法：`compile_age_package`（age-package 面，`LegacyWidgetAlias::Accept`）喂真实源+同包 module 全量；合成最小源隔离 widget 体能力。结果：

| 实测 | 结论 |
|---|---|
| `001-helloworld` | **OK**（唯一可直接编译的真实例；无跨模块 helper） |
| `002-counter` | ERR `source.lex` 反引号模板文本（`text \`Counter: ${.count}\``）——age-package 词法面缺反引号文本 |
| `010/011` | ERR `source.syntax expected fn, found style`——**顶层 `style` 声明**在 age-package 面无规则 |
| `013/019/020/021/023/025/038` | ERR 回滚位 `expected fn, found widget`；合成隔离证明=**`msg { X(T) }` payload 变体**不在包面 widget 体（`msg{A,B}` 单元可、`msg{A(str)}` 不可） |
| `026-database` | ERR `source.lex` `$`（模板/`$` 文本面） |

规模（35 例，去注释）：**顶层 `style` 20/35（126 行）**、**payload msg 17/35**、`use { package }` 5/35、反引号 2/35。合成矩阵：`model+view`、`msg` 单元、`design`、`use m.n: A,B + msg` 在包面 **OK**；`msg` payload、反引号文本 **FAIL**。

**结论与重划（待用户/981 裁定，见 §10）**：981 会签 A1"整个 widget 工程按 age-package 面编译"在真实源上**尚不可行**——包面缺 age-component **已交**的三类 widget 体表面（顶层 `style`、payload msg、反引号文本；`use { package }` 已由 D-④-7 单列）。④-A 本旨（导入/闭包/跨模块 helper）在这些表面补齐前**无法在真实示例上被触达**。建议 ④-A 首切片顺序改为：**S0.5a 表面补齐**（style 声明 + 包面 payload msg/反引号，按用户裁定归属：980 域内收敛 vs 981 语言面）→ **S0.5b 惰性绑定**（A3）→ **S1 真实例闭包 + helper + AV/TS 承载**。不得以 001（无 helper）或合成 fixture 冒充 ④-A 打通。

**S0.6 形状准入补测（2026-11-18 第二轮探针，同一临时 harness，已删净）。** 包面对**真实例**形状的准入：`ace-node-counter`（node+匿名 msg 单元+`on { .X -> … }` 块）**OK**、`ace-hello-minimal` **OK**、`001-helloworld` **OK**、`legacy-on-block` 合成 **OK**、`canonical-msg-named`（`node`+`msg Msg { A }`+`on A {…}`）**OK**。⇒ 形状墙不是 `widget`/`node` 或 `on` 块形，而是**匿名 `msg { X(T) }` payload**（17/35）、**顶层 `style`**（20/35）、**反引号文本**（2/35）。故"真实示例"可选面=**已可解析的 `ace-node-counter`/`ace-hello-minimal`/001**；这三者当前**无跨模块 helper**，④-A 必须在其上接入真实 helper 模块（作者拼写 `use helper: sym`）才能验"引用驱动 + helper 影响结果"——该接入会新增 import 行，属**切片测试源**（须注明"由真实例派生"，不得宣称原例已支持）。

**④-A 代码顺序（用户 2026-11-18 指令落档）。** ①**点分 `use` 投影 + 惰性绑定必须同批落**：今天 `age-use-module-item` 在包面无投影分支（零事实），单加投影会让 34/35 例变 eager 导入而全红；单加惰性绑定则无输入。②在已可解析的真实例（`ace-node-counter` 派生源）上接入真实 helper 模块，验三条最小判据（未引用 `themes` 不拉模块／被引用 helper 进闭包并改结果／缺模块·缺符号·私有符号具名拒）。③AV 真窗可观察结果。④同源 SFC 调用发射与名字映射。**未完前置**：S0.5a 表面补齐（style/payload msg/反引号）仍为"整例真实源"的前置，归属待 §10 R1/R2/R3 裁定；本顺序只保"派生真实例"可开工。

### P3 提案：`style` 声明面（981 会签材料，2026-11-18；**本轮不动文法**）

**一、现场实测（35 例 + stylekit 包）**
- 35 例中 **20 例**含顶层 `style`，共 **126 行**：125 行 `style NAME = "字面量"`、1 行 ident 复制（007 `style hint_sm = caption_text`）、1 行带参调用（007 `style stat_value = section_title(size: "text-sm")`）；**0 个参数化 style 声明、0 个 `pub style`**。
- 依赖包 `examples/ui/stylekit/src/front/styles.at`：**7 条 `pub style`**——4 条无参字面量（card_base/hint_text/caption_text/icon_base/input_field 中的无参项）、**3 条参数化**（`pill(bg: str = …, fg: str = …, pad: str = …) = "{pad} {bg} …"` 形，body 含 `{param}` 插值、带默认值），另有 1 条非 pub `style internal_only`。
- 今天 ACE 文法无 `style` 规则：包面在首条 style 行报 `source.syntax expected fn, found style`（20/20 同因）。
- **移除 style 行后的下一阻断（实测两轮）**：包面 18/20 仍报 `expected fn, found widget`（**回滚位**，真实原因被掩盖）、2/20 报 `source.lex unknown character $`；组件面（诊断更精确）017 例：003/004 **可直接解析**；7 例卡 `use mod: syms`（组件面未开，但 ④-A 的包面已开）；5 例卡 model 字段形状（`unexpected token in a model field`）；7 例卡 view/model/node 槽位（含 `use { package }` 块与包裹元素）；2 例卡 `$`。
  ⇒ **style 是"第一个"解析阻断，但不是充分条件**；下一刀必须按"每次放行后的下一阻断"重排（实测驱动，不按文件名/组号）。

**二、提案（第一刀 P3.1：`style` 面）**
- **语义（不引入新 HIR 域，`str` 常量/函数语法糖）**：
  - `style N = "s"` → `Item::Const{name:N, ty:str, value:"s"}`；
  - `style N = other`（ident 复制）→ `const N str = other`（绑定期按常量读；缺名沿用现有具名拒）；
  - `style N(p: str = d, …) = "{p} …"` → `Item::Fn`（参数按声明序、类型 str；body=`{p}`→ACE f-string `$p` 的 str 表达式）；**默认值本刀不支持**（含默认值→具名拒 `source.age.style-defaults`）；
  - `pub style …` → 公共面 const/fn（复用 `public-item` 的可见性/重导出）；
  - **stylekit 默认参数=本刀按具名拒绝验收**：stylekit 的 3 条参数化 recipe 带默认值（`pill(bg: str = …, …)`），本刀不接默认值——含默认值的声明以 `source.age.style-defaults` **具名拒**；验收口径=该拒绝（不是静默忽略、也不假装已支持）。因此 stylekit 在本刀**不解锁整例**，007/016/017/029/031p 的 stylekit 面留后续刀；
  - **调用点折叠**：`style X = f(arg: "lit")` 与 view 内 `style: f(arg: "lit")` 当且仅当实参全为字面量时在投影期折叠为 str 常量；非字面量实参→具名拒 `source.age.style-call`（不猜求值序）。命名实参按声明序对位（未知键/缺参具名拒）。
- **文法**：新增 `style-item` 与 `public-body` 的 `style` 臂；profile 门 = `[age-component, age-package]`；`style` 保持普通标识符拼写（**不进 token 表**），仅在声明头/公共体按 spelling 识别（同 `age-node-head` 的 `name(spelling:)` 机制）；**普通标识符位置的 `style` 保持合法**（永久负控）。
- **投影/AST**：新规则 → 上述 `Item::Const`/`Item::Fn`；不新增 AST/HIR 节点；名字进 const/fn 命名空间，重名走现有重复声明拒。
- **身份与后端**：const/fn 的 mangle、可见性、AV/TS/A2R 发射**全部复用现路径**——本刀无后端改动。

**三、文法身份影响（981 会签要点）**
- 现冻结值（**spawn/f32 落主后的基线**，2026-11-18 复核）：`age-component` = `19a1376eac5f32404b0a653e84d58206c74a954b1546be5aec4485a3fb9bbae5`；`age-package` = `0290d0ab2441b23399f6505512da4e3c9a9e907890d9584ba8a45ff2694136c8`（前值 f35cf593…/6c1bb477… 随 #6o 与 981 spawn 臂移动）。
- 本刀**只移动 ge-package 一个值**（按 981 回签 (a) 收窄；ge-component 保持 19a1376e…）；其余 **13 profile 必须逐字节不变**（会签后由 980 提交 diff+逐 profile 转储，981 在**同一提交**重基线 `plan981_grammar_scope.rs` 并注明前值/原因）。
- `themes` 的 `pub theme` 仍是**独立未支持面**：style 不放行 theme，不得把两者混为一谈。

**四、验证计划（会签后执行）**
- 解析级：20 例 + stylekit 的 style 行全部按上述映射通过；永久用例=字面量 / ident 复制 / 参数化+`{p}` 插值 / `pub style` / 字面量实参折叠 / 非字面量实参具名拒 / 默认值具名拒 / 重名拒 / `style` 作普通变量名仍合法。
- 顺序级：放行后对 20 例**重测下一阻断**（包面 + 组件面双读），产出"下一刀"排序（预计 payload msg / `use { package }` / model 字段形状 / `$`）；包面**回滚位诊断**（18/20 同报 `widget`）列为本刀附带修项候选（只改错误选择、不改语义；若触及 981 冻结的错误用例，先报后改）。
- 门禁：`cargo t ace-source ace-av`；只签 style 面，**不宣称整例通过**。

**五、请求 981 会签（回签后 980 才动文法）**
1. 认可 `style ≡ str const/fn` 映射与 profile 门 `[age-component, age-package]`；
2. 认可 grammar ID 同批重基线（**收窄后仅 age-package 一值**；981 同提交更新冻结期望，注明前值）——已按回签 (b) 做法 A 采纳；
3. 确认 const/fn 命名空间与现有身份/可见性契约无冲突；
4. 回签后落文法 + 投影 + 永久用例；本材料**未改任何文法/代码**。

**六、981 回签（2026-11-18，`edfa4220c`）与 980 回执（本版按修订收窄）**
- **(a) 收窄通过**：profile 门**只 `[age-package]`**（`age-component` 不开）。依据=组件面无语义面 const/public 规则、resolve 的 const 由 packages 绑定器消费、`ModuleGlobalsPlan` 仅 AGE 包前门钉入——开组件面=能解析跑不了。**只移动 age-package 一个 grammar ID**；`age-component` 保持 `19a1376e…`。
- **(b) 做法 A 采纳**：980 在文法提交内同步更新 `plan981_grammar_scope.rs`（注明前值，沿 #6m/#6o），13 邻居逐 profile 转储随提交；981 在该提交复跑 `p981_grammar_scope` 并核 diff。
- **(c)1 采纳**：重名走现有 `source.module-duplicate-declaration`（同一绑定图），不另造码、不只报 mangle。
- **(c)2 980 回执=`Str`（owned）**：参数化 style 生成 `Item::Fn` 返回 `Str`；零参 style=const `Str`。理由：body 是 f-string/拼接（owned text），与文本叶 `.clone()` 约定同源，避免 view 生命周期；A2R/TS 按 owned 串发射。
- **(c)3 980 回执=投影期字面量替换**：P3.1 中 style 名字（`style: NAME`、`style N = other`）在投影期替换为其字面量；带参调用一律 `style-call` 具名拒（**不做**折叠——语料唯一带参调用的目标声明带默认值本刀本就拒；跨模块常量求值收益 0、缓存身份/求值序风险不为 0）。**明示撤回**：用法不再声称复用 mangle/可见性；声明仍落 const/fn（身份/重名/导出/`pub` 显示面）。
- **(d) 采纳**：`style-defaults` 指向**默认值 token**；`style-call` 区分 unknown-key/missing-arg/non-literal（同码不同臂、原 span）；未知形态不得回落 `source.syntax`。

**七、落刀工作单（980 自用；回执后执行）**
1. 文法（`grammars/auto-source.at`，仅 age-package）：`name(style-head){spelling:"style",token:"ident",role:"style"}`；`style-item`（name + 可选 `style-params`/`style-param`（`: ty` 可选、`= default` 可解析）+ `=` + `pratt wide-ops` value）；项位 alt 与 `public-body` 各加 `style-item` 臂（public 体首 token `style` 与 fn/type/const/use 互异）。
2. 投影（`cst/frontdoor.rs` 包面）：zero-param → `Item::Const{ty:Str}`；param → `Item::Fn{ret:Str}`；默认值存在→`source.age.style-defaults`（span=默认值 token）；登记 `declarations`/`exports`（`pub style`）；重名走共享绑定器。用法替换：组件投影的 `style:` 条目与 `style N = other` 的初值在投影期解析为字面量（同模块；跨模块命中即 `style-call`）。
3. 永久用例：字面量 / `pub style` / ident 复制 / 参数化（无默认）/ 默认值拒 / 带参调用拒（unknown-key·missing-arg·non-literal 三臂）/ 重名拒 / `style` 作普通标识符仍合法 / 13-profile 逐字节不变 + age-package 新值。
4. 身份：更新 `plan981_grammar_scope.rs` 的 age-package 值与注释（前值 `0290d0ab…` → 新值），age-component 行不动；随提交附 13 邻居转储。
5. 门禁：`cargo t ace-source ace-av`；981 复跑冻结测试。

### PHASE-LAMBDA-IF：view 内联 lambda 值函数铸造结构化（2026-11-19 用户指令刀；r18 边界②收口）

**定位。** view 内联 lambda `() => { … }` 的值函数铸造（`__age_vl_<seq>`，#6k 交付）现只收平铺赋值；体含 `if`（`age-view-if-stmt` 臂，on-if 刀已开、lambda 体共用 `age-view-stmt` 文法）时投影层落 `foreign=true` → 值函数不铸 → lane 缺席（旧道承载）。本刀=该面 **assigns-only → 结构化语句面**（If/Assign 序保持）；落成翻转 005 语料 `oninput: () => { if … }` 族 lane 全派生。

**五环复用矩阵（现状全部 master 实测，见 §4 本轮取材）。**

| 环 | 现状 | 本刀 |
|---|---|---|
| a 文法 | lambda 体=`list-nl age-view-stmt`（`auto-source.at:1063-1068`），if 臂已共用（:591） | **零改动**；grammar-id 冻结断言 |
| b′ 投影 | lambda 只解 `field` 臂，if 形落 foreign（`project.rs:5774-5792`）；结构化投影器已在（:7141-7199） | 接 `project_view_stmt_children`（channel_names 传空——call 形统一置 foreign 与 #6p 合同一致）；`ComponentEntryValue::Lambda` 增 `stmts: Vec<ComponentStmt>`（镜像 `ComponentHandler.stmts` 先例），`assigns` 保深平铺兼容（`flatten_stmt_assigns`） |
| 铸造 | `collect_lambdas`/`lambda_value_fn` assigns-only（`age_component.rs:1362-1409,2155-2260`） | 收 stmts 面；`lambda_value_fn` 增 `If` 递归降版 `Stmt::If`（镜像 `hook_stmts_lower` If 臂分支快照语义；lambda 无形参/无 let ⇒ 无 scope 需求；`Assign` 保持既有 `rename_expr` 路径**不改发射文本**）；DFS 计数序/字段 init let/return 首字段/`__age_vl_` 保留名均不动 |
| c/d 降版/发射 | a2x-ts If 臂（`lower/mod.rs:412`）+ `emit_stmt_fragment`（`fragment.rs:172`） | 复用，零改动 |
| e lane/UI | `stmt_fragments_in` 根块 If 整语句发射（on-if e 环已交）、`lambda_fn_covering` 区间相交、`pair_lambda_regions` 括号配对 | 预期**零/近零产品代码改动**（假设）；T-LIF-4 bounded probe 实证三点：①配对成功（mint 计数序不变）②`lambda_fn_covering` 命中（If cond/RHS 与区间相交）③If 整语句发射+宿主映射/临时名改号不破 |

**验证面深走查。** `validate_channel_value_use`（`age_component.rs:1935-1965`）与 frontdoor `collect_item_exprs` Lambda 臂（`frontdoor.rs:1199`）从平铺 assigns 改 stmts 深走查（If cond + 分支赋值 RHS——通道值位置拒/前门表达式收集覆盖分支内）；共享助手复用/镜像 `walk_stmt_exprs`（:1892）。`field_manifest.rs:382` 只计数、ops=lambda 不进 ops（既定合同）——均不受影响。

**合同不变项（非目标显名）。** 调用形 lambda 仍 foreign 不铸（#6p）；view-stmt `let` 仍拒（G2 负控合同）；单行多语句形=`list-nl` 既定边界；lambda 赋值 owned-transfer wrap 语义保持现状（与 handler 面 `wrap_owned_transfer` 差异记为具名边界，T-LIF-5 用例如实钉 str 字段行为，异常按证据定修/记账）；条件渲染语义（m2-age §3 撤门条件）owner 另属；既有 assigns-only lambda 发射文本逐字节不变（兼容面）。

**规范增量（沿 §5 规范增量机制，交付后沉淀）。**

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LAMBDA-IF | modify | `m2-autoui.md` §13.12（沿原 owner） | before：值函数道 assigns-only（非赋值体=foreign 不铸）；after：结构化（Assign/If 序保持）铸造+If 整语句进 lane 发射；调用形仍 foreign | 005 lambda-if 族翻转；r18 边界②收口 | AC-LIF-01..06 |

## 6. 测试设计

> 原完整合同保留在本节。现行实施与验收归属见 §0 的移交表，基础收口任务见 §8。
### V-AGE-CAP-HANDOFF（新增分阶段控制，原V全部保留）

按§5原任务冻结实际cap/owner/源/平台与各端oracle：原UI波基线齐且新cap pending可交对应UI-W1/2/3，但不能签cap/AGE全齐；cap合法来源可逐端交，删cap/错源/把AV冒AC或仅emit冒A2R必须红，恢复真实产物可通过。用于新增cap的typed接口须先有独立合法/错误/owner测试，消费方真实GPU/交互/资源/包另验；缺模型/Forge产品不影响基线。AC-AGE-CAP-HANDOFF由本表原owner及最终review核。

### EVENT-SPLIT消费验证（拟议，原B-AV/B-WEB建立并运行）

| V / owner任务 | 输入→consumer与独立oracle | 条件/边界 |
|---|---|---|
| V-EVT-UI-A-SOURCE / 原NATIVE已交来源核对，981 T-01与985 T-A03接收 | 绑定三类MIN-A源、`ace_node_desktop`独立预期和真实FFI/事件/ABI；缺具体文件/hash/oracle指出缺件，不因executing拒收。 | 不重开已交NATIVE；上游改面仅重验受影响接口，旧PASS不扩大范围。 |
| V-EVT-WEB-CONSUME / B-WEB-I/V/R | 现p980_web_unified_entry_census_v2的param-handler保原常量体，但另取QueryInput实际读参、多载荷emit、bool/float及真实浏览器事件；同作者业务源走新Checked provider→SFC→执行，对原独立结果/DOM。 | 分列parse/typed/fragment/splice/browser/资源，spliced=1不等价动态载荷支持；无参数环境、假数值、旧provider回退需红。census全部原语料/条件/预存红与修复归属保留。 |
| V-EVT-AV-CONSUME / B-AV-I/V/R | 消费981 EVENT-B与SESSION typed事件，payload真改变值、双实例隔离、关闭/late/撤权/坏更新恢复；覆盖原UI/Widgets所有相关组。 | 只AV当前波，不每组编译A2R；最终完整AV仍需全部原分母。 |
| V-EVT-GATE-UI / B-PARITY-I/V/R及985接收 | 逐批实证：有M2A-08但无UI-W2-AC可进B-AV/所需WEB；无M2A-08不能用B结果签A；仅UI-W2-AC不冒完整W3。 | 这是原阶段/收据准入的负控，不新建调度器/runner；不能用“外部三件都未齐”拦已就绪批。 |


### V-DOMAIN-④（拟议，未执行；PHASE-④-EXT）

| Case | AC/承接 | 入口 | 独立判据 |
|---|---|---|---|
| V-DOMAIN-④ | 原 B-AV-I、B-WEB-I 的前置（**不新增 AC**，不改原任务分母） | 35 例 census 可复现（use 行/形态/137 符号/17 引用例/宽数值 003 double×2 与 031i float×3）；承载矩阵逐格证据等级；D-④-1..7 裁定在案；任务/进度/勾选零漂移；零语法/实现改动 | 静态复核+用户裁定记录；裁定后切片仍按原 V-B-AV/V-B-WEB 运行，不另立判据，不以静态 census 冒充产品 PASS |

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

### V-LAMBDA-IF（lambda-if 刀；拟实施，未执行）

**不变量**：①序保持（if 前后赋值源序不重排）；②fail-closed 不部分体（调用形 lambda 不铸）；③grammar-id 零漂移（零文法改动）；④既有 assigns-only lambda 发射文本不变（兼容面）。正例形：bare if / if-else / else-if 链 / 嵌套 / if+赋值混合序 / view-if Cond 子树内 lambda（DFS 序）。负例形：调用形 lambda（仍缺席）、未声明字段（具名拒）、cond 未绑定名（verify 红）、`let`（文法拒）。

| V | 入口 | oracle |
|---|---|---|
| lang core 永久 `plan980_lambda_if.rs`（注册随 `plan980_view_if` 先例） | 组件面入口 | 铸造名 `__age_vl_<n>` 逐条、HIR 含 `Stmt::If`（cond/分支）、序断言、负控三形、深嵌/else 链 |
| av `plan980_web_6p_channel_call.rs` 既有 foreign-lambda 负控 | 不变 | 仍绿（调用形仍拒——用例不破的判别点） |
| UI `p980_lambda_if_lane`（ace_ts_provider） | 产品出口 SFC 生成 | lane 派生非缺席、拼接体含 `if (…) {…} else {…}` 文本（a2x 形）、宿主映射过语句文本、缺席清单不含目标；调用形缺席保持 |
| UI 浏览器排窗档（`AUTO_UI_ACE_WEB_E2E=1`，镜像 `p980_web_lambda_browser_mount`） | 真挂载 | if 分支行为可判（条件翻转两分支 DOM 效果） |
| 005 真源 lane 对账 | 既有语料 rig | lambda-if 族缺席翻转；**整例残墙如实归名（不承诺）**；census 具名修订行 |

门禁：lang `cargo t ace-source`（grammar-id 冻结+core case）+ 触面组按 `cargo t --list` 实名选定；UI `cargo test -p auto-ui-runtime --test ace_ts_provider`（+ `plan980_package_sfc`/`p980_web_lambda_origin` 回归）。探针/语料日志入 `plan-artifacts/plan-980/`（沿既有 scope），merge 折叠摘要。

## 7. 验收标准

> 原完整合同保留在本节。现行实施与验收归属见 §0 的移交表，基础收口任务见 §8。

- [ ] **AC-AGE-CAP-HANDOFF / work**：V-AGE-CAP-HANDOFF及§5本包实际producer/consumer逐cap义务闭合；UI三波与新图形cap来源分开、无循环、原验收不减，新cap不成为985/A新增前置。阶段R/fold不冒整包final。

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

### EVENT-SPLIT验收补充（owner_stage沿原work/review/merge，待实施）

| AC | 原承接任务 | 可观察关闭条件 |
|---|---|---|
| AC-EVENT-PHASE | 981 T-01、LANGUAGE/SESSION/PROOF各I/V/R[A/B]；本包B-AV/B-WEB及PARITY；985 T-A03/A04/A05与B原任务 | §5三缺口逐阶段/真实source/oracle/owner/现有任务可追；A真实正例/保真/安全拒绝齐，B支持/执行/资源/错误与完整原用例全部闭；语法/splice/历史或worker总绿不越级。对应本节V-EVT-*，独立review绑定revision与正式来源；当前规划不得勾PASS。 |



### PHASE-LAMBDA-IF 验收（owner_stage 按行；待实施，当前不得勾 PASS）

| ID | owner_stage | 可观察标准 | 验证 |
|---|---|---|---|
| AC-LIF-01 | work | if 形 lambda 投影结构化 stmts（bare/else 链/嵌套/混合序）+assigns 深平铺兼容面保持 | `plan980_lambda_if` + 既有 core case 绿 |
| AC-LIF-02 | work | `__age_vl_<seq>` 铸造含 If（HIR `Stmt::If`→a2x-ts 全链）；lane 整语句发射 if/else 宿主文本 | `p980_lambda_if_lane` + 浏览器排窗档 |
| AC-LIF-03 | work | 负控合同不变：调用形 lambda 仍不铸（具名缺席）、未声明字段具名拒、let 文法拒、通道值位置拒覆盖分支内 | 负控用例 + `plan980_web_6p_channel_call` 全绿 |
| AC-LIF-04 | work | grammar-id 零漂移（零文法文件改动，diff 自证+冻结面绿） | `cargo t ace-source` |
| AC-LIF-05 | review | 既有永久用例零回归（`plan980_view_if`/`plan980_on_if`、`p980_web_lambda_origin` 002 e2e、981 冻结、ace_ts_provider 全量）；既有 lambda 发射文本不变 | 冻结门重放 |
| AC-LIF-06 | review | 005 语料 lambda-if 族 lane 翻转实测记账（absent 变化+census 修订行）；整例残墙如实归名 | 语料 rig 对账 |
| AC-LIF-07 | merge | SD-LAMBDA-IF 落 `m2-autoui.md` §13.12；收据入 §9；plan-artifacts 清理+收尾路径对账 | merge 流程 |

## 8. 执行步骤

保留 T-HELLO、T-01、B-DIALECT/STATE/NATIVE 的已验基础范围与正式接口。T-HELLO-D 核实际已合入源和验收证据后记账。980 保留当前 r21 的 package SFC 消费修复，作为 981 合入的明确接口交接。

- [x] **T-HELLO-I｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：967/F0-D、979/H0-D、969/E0-D、978/B0-D齐后，按§5冻结AGE node首源/Schema激活及旧widget兼容配置、CLI/主线程/文本所有权与资源，建ace-hello-minimal及ace_ui_hello永久case；只实现同一受检通路，不等T-01全部UI分母/STATE async/PARITY。

- [x] **T-HELLO-V｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：T-HELLO-I后跑全部V-HELLO-*与真实受影响UI命名case/必要语言职责并集，AV首窗全链记录独立oracle/窗口/FFI dispatch/退出；AXR0由原RUST并行验证，未执行腿保持pending，不阻AV部分正式交接。

- [x] **T-HELLO-R｜M2-A｜owner_stage: review｜AC-HELLO-REVIEW**：T-HELLO-V的AV全部V-HELLO-*齐后停写同树独立核最小普通源到真实Iced、AV FFI与资源/反控；finding返I，不把AV Hello pass冒A2R或完整UI。

- [x] **T-HELLO-D｜M2-A｜owner_stage: merge｜AC-HELLO-LAND**：T-HELLO-R pass后按各owner phase fold正式合入首Hello代码/已验规范与source/ABI/包来源，保唯一树/resync继续原九批与全包review；不改默认、不归档980或宣称完整RG-A03/M2A-08已过。
  [✅ 补记完成 2026-11-19（用户验收裁定，无需重实现）：落主收据=§9 :1286（2026-11-16 stage: merge，phase fold #4）——lang master `1991021a5`（代码 dd27cdf0c→50d0dcce4+规范 098d0f4a3，两次 rebase range-diff 全等）+ auto-ui master `e529dde6`（0 behind 直落）、wt-guard clean+ff-only、canonical m2-autoui.md 新 §7 首窗链；落地后复核门在收据（ace-facade PASS/ace-abi 预存红具名/UI ace_ui_hello 7/7）。双 SHA 祖先链用户核验+本会话 `merge-base --is-ancestor` 复核双 YES。]

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

- [x] **T-BASE-HANDOFF｜基础收口｜owner_stage: work**。闭合 981 F-R21-2。在 `auto-ui/crates/auto-ui-runtime/src/ui/ace_package_sfc.rs` 的重放与相关消费点，核受检 Let/If 门、实际分支与原失败恢复。随 981 F-R21-3 对公共写面合入并集和正式来源，逐项对证后继移交。
  [✅ 已完成 2026-11-19（三方对账，本会话 `merge-base --is-ancestor` 复核全 YES）] **981**：F-R21-2 消费门落主+双树验收（§9 F-R21-2 收口/双树验收收据）→ fold 并集落主 `33895d91c`（收据 `c84f25cf4`，规范 m2-age.md §4）→ 980→981 交接落账 `7fc15205a` → **T-06 终局归档 `a475cc8c2`**（981 整包闭）。**AutoUI**：Let/If 消费门 `7b8456e5`（`source.age.sfc-handler-stmt` 具名拒+永久负例+红相）+ Let 臂永久例 `d2e7d7a4`（fold 后主线可达，8/8）；实际分支/失败恢复=双树 `plan980_package_sfc` 7/7→8/8 + ace_ts_provider 串行 27/27（§9 收据）。**985**：接收记录=985 §「2026-11-19 · 980/981 后继接口」（T-02/03、T-08[B]、T-04 负责从 980 PARITY 和 981 AUDIT 的联合接收与认证；波次分接 1071/1068/1074/1072/1073/1075-1077——随 r20 冻结提交入库）。后继移交对证=§0 移交表+r20 冻结合同。

- [x] **T-05｜基础收口｜owner_stage: review**。同树冻结，独立逐项核保留的能力、原适用反例、联合交接、后继义务完整性与来源。修复发现后重新验证。
  [✅ 已完成 2026-11-19（同会话终审——限制：非独立上下文；结论由工件重构：冻结门实跑+SHA 祖先链机械核+收据交叉对账，非采信执行摘要）] **冻结门（r20 提交 `3ba8d4259` 后 master 实跑）**：lang `cargo t ace-source` **PASS**（P3 `e8e4c16da` 重基线后冻结面绿）；`cargo t ace-av` **1426/1428**——唯二红=host_console 预存对（stash 基线同红实证[lambda-if 刀轮]+981 fold 收据同口径+本轮隔离复现同对，不判本包）；UI：ace_ts_provider 串行 **27/27**（3.17s）、`plan980_package_sfc` **8/8**（0.11s）、`ace_ui_hello` **7/7**（1.76s；坏源负例首跑=具名预备失败「exe not built」[纪律正确]，prep 段 `--example ace_hello_window --features ace-bridge` 3m15s 后全绿）。**逐项核**：基础 14 任务代表 SHA 祖先链全 YES（HELLO `1991021a5`/`e529dde6`、DIALECT `46f4ac0ec`、STATE、NATIVE `7f9d03bf6`、on-if `9468fdcf6`、lambda-if `347540437`、消费门 `7b8456e5`/`d2e7d7a4`、P3 `e8e4c16da`）；交付面零调试残留；后继计划 1071-1077+985/983-987 接收记录随 r20 入库。**findings=零**（无需修复重验）。

- [x] **T-06｜基础收口｜owner_stage: merge**。按 owner 次序合入，沉淀基础范围的正式规范与后继链接，核真实消费来源并归档。
  [✅ 已完成 2026-11-19] 全部基础交付已按 owner 次序落各 master（T-05 收据 SHA 链全 YES）；规范收口=`m2-autoui.md` Status 头更新为「基础范围已收口」（已交付清单+后继归属 1071-1077/1068/985 链接）；消费来源核对=981 归档 `a475cc8c2`+985/983-987 接收记录（r20 `3ba8d4259` 入库）；本文件 git mv 入 `docs/plans/archive/`+status: archived（终态）。

- [x] **T-07｜基础收口｜owner_stage: merge**。依项目规约完成证据摘要、输出对账、登记和 guard 后清理。
  [✅ 已完成 2026-11-19] 证据摘要已折 §9（各刀收据：lambda-if/F-R21-2/双树验收/T-BASE-HANDOFF/T-05/T-HELLO-D 补记）；**plan-artifacts/plan-980 整目录删除**（~150 项探针/日志/补丁——收据内嵌后按 2026-10-31 裁定全删；无具名持有）；**lang-980 组四树 wt-guard 全 clean 后移除**（auto-lang 树因 ignored 构建产物先注销后残留 3.8M 源副本，guard 复扫 clean 后删——非穿透；组目录消）；两仓 `plan-980-dev` 分支删（lang `dce7492dd`/ui `d2e7d7a4`）；临时验证组 verify-981union 早前已清。收尾路径对账：本计划全部输出目录=两 worktree 组+plan-artifacts+临时组（均已消），无其他盘符/外置 target。

## 9. 复审记录

2026-11-19 stage: revision。按用户裁定拆分。原任务、AC/V、负控与产品义务按 §0 移交。基础计划保持 executing，按本节上方 §8 实际勾选为 13/18。后继为 drafting。本轮未生成产品验收结论。

### 拆分前 §8 任务合同

快照 78b375bf4。原转出任务共 15 行，仍未关闭。执行 owner 按 §0。以下为冻结文字，保留现场来源和先前证据。

当前共32任务，已完成13项（B-NATIVE-I/V/R r13 批已复核勾选；T-HELLO-D 2026-11-19 补记——落主收据 §9 :1286，双仓 SHA 祖先链复核 YES）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

#### 阶段 PHASE-④-EXT（2026-11-18 新增；非任务行，不改 32 任务/12 完成口径）

- 依据与内容：§5 PHASE-④-EXT（外部名域合同：名面 N1–N4、六轴、承载矩阵、三方案、条款 C1–C8）+ §5 裁定落档/执行顺序 + ④-A 首切片合同 v1；owner_stage: design→executing（裁定后）。
- 现状（2026-11-18）：**七项裁定已落**；①U01 #14 **已收口落主**（lang `95b0ccb49`/ui `91d549ea`，phase fold #14，见 §9）；②④-A：981 已会签（合同 v2，`81b0a03b1`），用户选 **R1′**（先闭包、派生真实例、表面补齐并行）；**首片 #1 已交**（点分 `use` 投影 + 引用驱动导入，lang `0db44dfdf`，四判据绿）——**下一步=AV 真窗 + 同源 SFC**；③UI058 T-02/981 联冻与包面表面补齐为并行线。
- 退出：④-A 首切片收据（含 AV 真窗与 SFC 映射两格由"推论"变"已验"）→ 逐示例扩展；原任务/AC 不变，切片仍走 §9 收据+phase fold。

#### 阶段 PHASE-LAMBDA-IF（2026-11-19 新增；非任务行，不改 32 任务/12 完成口径）

- 依据与内容：§5 PHASE-LAMBDA-IF（r18 边界②收口：lambda 投影结构化 + 值函数铸造 If 递归降版 + 验证面深走查；五环 a/c/d 零改动、e 复用实证）；owner_stage: work→review→merge 按切片。
- 步骤（切片序，收据入 §9，phase fold 沿例）：**T-LIF-1** 投影结构化（`Lambda` 增 `stmts`+接线 `project_view_stmt_children`+深平铺）→ **T-LIF-2** 铸造结构化（`collect_lambdas`/`lambda_value_fn` If 递归降版）→ **T-LIF-3** 验证面深走查（`validate_channel_value_use`/frontdoor `collect_item_exprs`）→ **T-LIF-4** UI bounded probe（零改动假设实证：配对/区间发现/整语句发射三点；小缺口刀内修、跨域缺口具名拒不冒闭）→ **T-LIF-5** 永久用例（lang `plan980_lambda_if` + UI `p980_lambda_if_lane` + 浏览器排窗档）→ **T-LIF-6** 005 语料对账 + census 具名修订 → **T-LIF-7** 独立复审/冻结门/merge（SD-LAMBDA-IF 沉淀、收尾对账）。
- 退出：AC-LIF-01..07 齐（005 lambda-if 族翻转或残墙具名归档）；原任务/AC 不变；实施沿唯一树 lang-980（plan-980-dev），材料入 `plan-artifacts/plan-980/`。

#### A最小与B各波唯一任务清单（阶段按行标签）

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- 原状态：已勾 · **T-HELLO-I｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：967/F0-D、979/H0-D、969/E0-D、978/B0-D齐后，按§5冻结AGE node首源/Schema激活及旧widget兼容配置、CLI/主线程/文本所有权与资源，建ace-hello-minimal及ace_ui_hello永久case；只实现同一受检通路，不等T-01全部UI分母/STATE async/PARITY。

- 原状态：已勾 · **T-HELLO-V｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：T-HELLO-I后跑全部V-HELLO-*与真实受影响UI命名case/必要语言职责并集，AV首窗全链记录独立oracle/窗口/FFI dispatch/退出；AXR0由原RUST并行验证，未执行腿保持pending，不阻AV部分正式交接。

- 原状态：已勾 · **T-HELLO-R｜M2-A｜owner_stage: review｜AC-HELLO-REVIEW**：T-HELLO-V的AV全部V-HELLO-*齐后停写同树独立核最小普通源到真实Iced、AV FFI与资源/反控；finding返I，不把AV Hello pass冒A2R或完整UI。

- 原状态：待验 · **T-HELLO-D｜M2-A｜owner_stage: merge｜AC-HELLO-LAND**：T-HELLO-R pass后按各owner phase fold正式合入首Hello代码/已验规范与source/ABI/包来源，保唯一树/resync继续原九批与全包review；不改默认、不归档980或宣称完整RG-A03/M2A-08已过。



- 原状态：已勾 · **T-01｜M2-A｜owner_stage: work｜AC-01/07/AC-PHASE**：核959行及§5完整传统UI的grammar/AST/Spec/组件/模式分母、来源/接口与唯一树/写窗，冻结纯UI AV/A2R及各批命令/oracle/资源，不能只列图形小例。

- 原状态：已勾 · **B-DIALECT-I｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-02/03/06/07/AC-NODE-COMPAT/AC-HANDOFF**：按WidgetDecl全部字段逐项投影/明确模式适用性，schema→typed领域IR→ACE行为；包含named_views/bind/routes/setup/watch/expose/actions/timer/theme及实际AST/Spec其余字段。先落实§5 IF-980-FIELDS的可执行字段清单/通用适配和永久正反例，R复核/正式交付子面后继续完整DIALECT；消费已交979基础、不等979归档。按§6冻结命令，原传统UI/V-UI-SOURCE/FULL义务不减。

- 原状态：已勾 · **B-DIALECT-V｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-07/AC-NODE-COMPAT/AC-HANDOFF**：运行V-B-DIALECT完整职责/真实consumer及IF-980-FIELDS子面，独立expected/源/工具/资源/子终态齐。核完整字段/构造覆盖、删实际映射与删清单行反控；交接子面不提前勾本任务或整个DIALECT。

- 原状态：已勾 · **B-DIALECT-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源；可先按§5复核IF-980-FIELDS子面并正式phase fold交979，完整本批仍须核全部原义务才勾选；缺陷回I，不冒整包交付。

- 原状态：已勾 · **B-STATE-I｜M2-A｜owner_stage: work｜AC-B-STATE/AC-02/03/06/07**：用ACE执行服务替旧handler bytecode生成/调用，接组件props/emit/computed与App store、挂起/唤醒、卸载和late generation。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- 原状态：已勾 · **B-STATE-V｜M2-A｜owner_stage: work｜AC-B-STATE**：运行V-B-STATE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- 原状态：已勾 · **B-STATE-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- 原状态：已勾 · **B-NATIVE-I｜M2-A最小｜owner_stage: work｜AC-B-NATIVE/AC-02/03/06/07**：实现MIN-A-UI普通Node入口/实际UI-FFI/typed事件与owner；新用例V-UI-A-MIN；完整001/002与所有Widgets、媒体/terminal迁B-AV。 [✅ r13批 round 24/25：ui980 provider+桥+通用入口，收据 §9]

- 原状态：已勾 · **B-NATIVE-V｜M2-A最小｜owner_stage: work｜AC-B-NATIVE/AC-07**：只关闭V-UI-A-MIN及所用接口/错误/资源原义务，真实窗/事件/恢复，不以字节探针代UI；完整V-UI-FULL交B波。 [✅ r13批 round 24：CPU 7/7+排窗 e2e 1/1+受影响面全绿]

- 原状态：已勾 · **B-NATIVE-R｜M2-A最小｜owner_stage: review｜AC-04**：独立核MIN-A-UI/正式source/ABI/FFI效果与B承接，phase fold交IF-980-NODE-DESKTOP-A；不要求完整Widgets/图形A2R，也不签完整NATIVE产品覆盖。 [✅ round 25 批次终核 pass，authored 反例 e203caa8]

- 原状态：待验 · **B-AV-I｜M2-B-W1｜owner_stage: work｜AC-UI-WAVES/AC-07**：M2A-08后冻结U01..06原源/Widgets分母和旧新oracle映射，在既有DIALECT/STATE/NATIVE owner扩完整AV组件/场景/输入/错误/资源；只做AV，多个简单例按能力成组，不另建引擎/runner。 EVENT-SPLIT：只等M2A-08及本组实际IF-981-EVENT-*-B接口，不等W2/WEB全包；无参已交面复用。

- 原状态：待验 · **B-AV-V｜M2-B-W1｜owner_stage: work｜AC-UI-WAVES/AC-07**：运行V-UI-W1与该组受影响完整职责/真实窗口；原001/002和全部示例/Widgets余行闭合，不运行图形A2R；收981所需AV域格，不能等AC/OS全量再交。 EVENT-SPLIT：实跑V-EVT-AV-CONSUME；完整payload/绑定用实际值，不用常量体代参。

- 原状态：待验 · **B-AV-R｜M2-B-W1｜owner_stage: review｜AC-UI-WAVES/AC-04**：同树停写独立核完整AV分母/原oracle/失败控制/来源，phase fold交UI-W1-AV及宿主/域ABI供1068 W2；不提前签三端/整包PARITY或归档。 EVENT-SPLIT：核AC-EVENT-PHASE/真实EVENT-B来源后逐组交AV，不等全域终验。

- 原状态：待验 · **B-WEB-I｜M2-B独立现役模式｜owner_stage: work｜AC-B-WEB/AC-02/03/06**：Vue生成使用新TS/JS provider，保watch/setup/expose/ext_imports/style及绑定；服务端使用已交宿主接口或隔离真实fixture，不等整个983才能证UI表达式。 按§5真实改面补永久fixture并固定§6命令。 EVENT-SPLIT：复用#6i，具体消费981 B-LANGUAGE-I/V/R的MSG/PAYLOAD/PARAMS-B与B-SESSION所需typed接口；本批完整实现门=M2A-08及所用接口，不等W1/W2整波。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。

- 原状态：待验 · **B-WEB-V｜M2-B独立现役模式｜owner_stage: work｜AC-B-WEB**：运行V-B-WEB完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 EVENT-SPLIT：V-EVT-WEB-CONSUME按真实旧语料＋实际读参区分例对独立oracle，缺席/旧道不计ACE覆盖。

- 原状态：待验 · **B-WEB-R｜M2-B独立现役模式｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 EVENT-SPLIT：核AC-EVENT-PHASE及三件语义范围，不能以#6i语法/拼接计数关闭完整WEB。

- 原状态：待验 · **B-RUST-I｜M2-B-W3｜owner_stage: work｜AC-B-RUST/AC-02/03/06/07**：先按§5 AXR0切片运行同源单text V-UI-AXR0并独立复核/正式交接；随后多文件store/组件与sidecar经新A2R生成，保持原Rust UI行为和增量产物；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。 EVENT-SPLIT：W3只取UI-W2-AC及所用正式接口，不要求980所有等待条件先齐。

- 原状态：待验 · **B-RUST-V｜M2-B-W3｜owner_stage: work｜AC-B-RUST/AC-07**：运行V-B-RUST完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。

- 原状态：待验 · **B-RUST-R｜M2-B-W3｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。

- 原状态：待验 · **B-TAURI-I｜M2-B独立现役模式｜owner_stage: work｜AC-B-TAURI/AC-02/03/06**：接新TS/JS与Rust backend生成，保持前后端invoke/事件/权限边界；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。

- 原状态：待验 · **B-TAURI-V｜M2-B独立现役模式｜owner_stage: work｜AC-B-TAURI**：运行V-B-TAURI完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- 原状态：待验 · **B-TAURI-R｜M2-B独立现役模式｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

<!-- r11（2026-11-16 用户裁定）：B-JET-I/V/R 与 B-ARK-I/V/R 六任务行撤出 v0.6 执行清单，归 M3（v0.7）。原文与 AC/V 留档：B-JET-I=保持现有Jet生成模式，以受检UI领域IR供目标adapter；不新增原未承诺能力；旧AST emitter无回退（AC-B-JET/V-B-JET：原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行）。B-ARK 同型（AC-B-ARK/V-B-ARK）。M3 再启时按原义务实施。 -->

- 原状态：待验 · **B-PARITY-I｜M2-B分波接收/最终汇合｜owner_stage: work｜AC-B-PARITY/AC-02/03/06/07/AC-NODE-COMPAT**：从001按文字布局→表单列表滚动→菜单Blueprint→媒体编辑分组；核完整语法/字段/组件/变体/模式双向表，补未被Demo覆盖的语法/错态及删除必需行负控，建V-UI-SOURCE/FULL并汇总纯传统UI AV/A2R实际结果。按§5接fixture/固定命令，不等981完整Node终验。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。 EVENT-SPLIT：按V-EVT-GATE-UI逐批接收，最终才汇合全部现役模式。

- 原状态：待验 · **B-PARITY-V｜M2-B分波接收/最终汇合｜owner_stage: work｜AC-B-PARITY/AC-07/AC-NODE-COMPAT**：运行V-B-PARITY完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。

- 原状态：待验 · **B-PARITY-R｜M2-B分波接收/最终汇合｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。

- 原状态：待验 · **T-05｜M2-B最终｜owner_stage: review｜AC-04**：全包独立final核全部批收据/组合/遗漏/延期/workaround、warning/fmt/debug和Spec；缺必需结果不得pass。

- 原状态：待验 · **T-06｜M2-B最终｜owner_stage: merge｜AC-05**：按owner次序合入/落规范，核真实来源/consumer同版，全部齐后归档。

- 原状态：待验 · **T-07｜M2-B最终｜owner_stage: merge｜AC-05**：摘要折Plan、输出逐目录对账/清scope/登记，guard clean后清树。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

#### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

### 原阶段收据

### 2026-11-18 stage: handoff（**PLAN-981 → PLAN-980：W1 分组 AV 证据交接，供 `B-AV-R` 汇入 `UI-W1-AV`**）| 提供方 981 r18 | 绑定 `3047b3481` | outcome: **可供接收（3 组）；1 组不属 981**

981 按组件组复跑了 W1 的 AV 适用格（`V-AGE-WAVE` 的 AV 子面），逐组证据与责任边界如下——**本交接只提供分组证据，不代 980 签发 `UI-W1-AV`**：

| 组 | 981 证据（绑定 `3047b3481`） | 建议 |
|---|---|---|
| **G-INSTANCE 双实例 2:1** | `plan981_event_b_av`（两实例 `(i0,+3)(i1,+5)(i0,+4)`→[7,5]）＋`plan981_cross_module_nodes`（**两真实模块** `ForNode{0}×2+{1}×1`→2:1；模块序调换随动；越界报包内组件数） | 可并入 B-AV-R |
| **G-LIFECYCLE 关开** | `plan981_gfx_host`（`stop()`→`age-gfx.session-stopped`、provider 丢弃、新会话重注册）＋`plan981_gfx_hooks`（新 callback epoch 归零、跨 Tick 账本重置） | 可并入 B-AV-R |
| **G-VIEW/FRAME/INPUT** | `plan980_*`/`plan979_*` AV 家族随本批复跑（合计 75/75 PASS，0.16s 运行段） | 可并入 B-AV-R |
| **G-BADEDIT 坏编辑保旧** | 源侧：core `plan981_age_package_modules::recovers_on_the_same_entry`、`plan981_age_ref::recovers_after_the_import_is_added`、`plan981_event_b`（具名拒→修复同入口绿）；**会话语义（失败不动既有 ops）归 auto-ui R64-1** | 源侧可并入；会话侧请以 auto-ui 正式来源计 |
| **G-RESOURCE 资源归零** | **会话腿已由 981 提供**（`plan981_resource_ledger` 2 例）：显式释放 ⇒ `leaked_regions_cleaned==0`/`live_mapped_at_exit==0`/arena 0；**判别性负控**（只分配不释放）⇒ `1`/`>=16` 且运行仍成功。**车道边界**：平台保留名在 AGE 面不可达（仅 HostConsole/TextRecord/Aggregates），故 AGE 节点自身的资源面（宿主行/精灵回收）仍归 UI058/1068 | 会话腿格可并入；AGE 节点资源面不在此组 |

**同时告知（与 W1 相关的既有交付）**：①跨模块实例身份已修（三处统一为**包级组件声明序**：绑定器级场景核对、`node_base` 线程化发射、AV `symbol_of_in` 解析）——UI058 可把 `Target`/`Player` 拆回两个真实模块；②UI058 缺口⑤的答复（typed 队列可直接复用、作者内自派发按 host authority 不新增）见 058 §T-05 答复条（auto-ui `8438e0bf`）。

**未由本交接覆盖**：`UI-W1-AV` 汇总签发（980 `B-AV-R`）、G-RESOURCE 组、以及 W2/W3 波次；981 整包仍 `executing`。

### 2026-11-17 三路线修订的规划预检（不是实施验收）

本轮仅文档：统一strategy/架构、四个引擎AV新合同与四个新版AutoForge合同；保本包原任务ID/进度/全部旧AC/V/SD，新增阶段义务见§5–7。静态核对8新Plan共74任务、9既有Plan共140原任务ID、阶段DAG 44节点/90依赖无环、新增本地链接均可达，16份取材源码/规范/已归档合同身份未改变；四仓文档diff格式检查通过。981并发实施会话的T-01勾选保留，不计本轮交付。下方历史记录原样保留。

只说明规划合同可审阅；未运行Cargo/GUI/模型/构建/产品测试，未建实施树、未resume暂停工作、未合入/归档/签章。所需正式接口/最低负载/真实oracle仍须未来work→独立review→merge实际验证；原已存在历史失效链接不作为当前入口，新增链接已单独核对。

> EVENT-SPLIT-20261116历史解释：本节旧blocked/“等待980正式源”、round39/40及旧全量A门只证明当时核对/选择；当前依§5和§8逐切片合同，不用executing/整包未勾代接口缺失。旧产品结果不因本次修订自动扩大为参数/载荷或新A验收PASS。

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

2026-11-16 stage: goal（用户裁定轮——等待外部交付，goal round 39）| plan_id: PLAN-980 | plan_revision: r14 | outcome: **blocked（用户裁定=等待，条件具名）**| 状态：executing 12/32，无阻塞可续项。

**用户裁定**：①等待外部交付（原裁定不变）。980 保持 executing，等 PLAN-985（M2A-08）、PLAN-1068（UI-W2-AC）、语言侧前置×3 交付后再续 B-AV/B-RUST/PARITY/终验。

**本会话最终交付清单（round 24–39，15 个 commit）**：
- NATIVE r13 批全链：ui980 provider FFI 实效果+typed 事件回 AV+MIN-A-UI 通用入口+AUTO_UI_ENGINE=ace 重定向（fold #7：ui 403c936d/lang 7f431bd5f）
- WEB 批十三个切片（#1–#6h）：fragment 消费/普查冻结表/组件面入口/O-C3 精确定位/真实002源端到端/方言 OnHandler span 捕获/路线B核心/SFC拼接/一体化入口+state归一化/参数化域缺口具名/全量双轨仪表/浏览器挂载档
- 规范沉淀 §12/§13.1–13.11；ace_ts_provider 14/14；ace_node_desktop 8/8+排窗 e2e

**等待的外部条件（unblock）**：
1. PLAN-985 交付 M2A-08（当前 executing 4/14）
2. PLAN-1068 交付 UI-W2-AC（当前 drafting 0/10）
3. 语言侧前置×3：ScenarioOp payload 扩展、具名 msg 语法、参数绑定面（979/981 谱系后继）

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 39 | from: user | to: blocked | status: executing（12/32）| outcome: blocked（等待外部，条件具名）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui 15b9c4c6 lang 6512eb531 | next: 外部交付后 resume work（B-AV/B-RUST/PARITY/终验）

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 40 | from: user | to: blocked | status: executing（12/32）| outcome: blocked（用户裁定=等待外部交付，条件具名；本会话无阻塞可续项）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui 15b9c4c6 lang 6512eb531 | next: 外部交付后 resume work（985 M2A-08 → B-AV；1068 UI-W2-AC → B-RUST W3；语言侧前置×3 → WEB 真语料 spliced 切换 → TAURI → PARITY → T-05/06/07 → 归档）

2026-11-16 stage:new/revision | PLAN-980 r15 | outcome:pass（规划合同交接，非代码/产品/独立A验收PASS） | EVENT-SPLIT三缺口→原LANGUAGE/SESSION/AV/WEB/认证任务，A/B交付和逐批恢复边界明确；复用#6i与正式MIN-A来源，保全部原任务ID/勾选/状态/完成数/依赖/验收。新增AC-EVENT-PHASE与V-EVT正反控制均未实施。next:未来原owner按§5/§8在各自已授权范围执行，当前只规划，不建树/改代码/测试资产/运行/派agent/提交合入/清理/恢复目标。

2026-11-16 stage: work（WEB 批切片 #6i：组件面语法域扩——具名 msg+带参 handler，goal round 41）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（语法扩展功能面成立；**B-WEB-I/V/R 保持未勾**——跨计划 981 冻结 hash 须其 owner 核准后方可 ace-source 全绿）| worktree: lang-980 @ plan-980-dev | commit: lang **b043117b8**（grammar）+ 分支持有 debts（981 冻结条目）、ui **876c99a3**（锚定修复+census 修订）。

**交付。** ①`auto-source.at` 语法扩展：`age-msg-member` 可选具名段（`msg Msg { Add(str) }`）+ `age-on-widget-handler` 可选参数段（`.Add(t) -> { … }`）——真语料主导形态的域缺口；②`handler_body_region` 参数段跳过 + SFC 拼接 `): void {` 锚（带参函数签名）；③census v2 修订：**param-handler=1 spliced**（语法扩展生效——原 absent 现可拼接）。

**验证（冻结提交实测）。** ace_ts_provider 14/14（param-handler spliced=1）；ace-source 801/803（2 红=`p981_grammar_id_is_frozen` + `p981_grammar_ids_match`——**语法扩展改变 age-component/age-package 两 profile grammar-id → 981 冻结 hash 漂移**，测试注释明示"值随批次推进更新"但修改他会话冻结测试须其 owner 核准——D-980-WEB-GRAMMAR-981-FREEZE 债务在分支）。ui ff 落主 `876c99a3`；lang grammar 落主 `b043117b8`。

**边界（诚实）。** 语法扩展证明功能面成立但 981 冻结 hash 未更新（跨计划）→ ace-source 组 2 红在 981 owner 核准前不可全绿。真语料全量双轨聚合仍为 spliced=0/absent=23（复杂结构在面编译级仍失败——computed/watch/ext_imports 等，语法扩展只是第一步）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 41 | from: work | to: blocked | status: executing（12/32）| outcome: blocked（981 冻结 hash 跨计划核准 + M2A-08/UI-W2-AC 波次门）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui 876c99a3 lang b043117b8 | next: 981 owner 核准冻结 hash 更新；外部交付后 resume（B-AV/B-RUST/PARITY/终验）

本轮文档静态预检（非产品PASS）：8份规划/路线/债务文档；980/981/985原80个任务ID、勾选、32/34/14总数与12/3/4完成、状态及依赖并集保留，原AC/V/SD标识未删除。新增表格/引用/任务和阶段映射、Markdown围栏（含985原四反引号嵌套例）与空白通过；最终阶段图32节点37边无环。12份读取的代码/用例/规范内容hash未变，980并发round41收据完整保留；所报两红只完成归属规划、未修复/复测。新增V-EVT均未创建/运行，实际矩阵/SDK/目标状态及canonical Specs未改。next为原owner后续按现合同执行；当前只规划，不签代码/独立产品review或A出口。

2026-11-16 stage: new/revision | PLAN-980 r15（用户范围裁定 v2）| 用户裁定**新等待顺序**：① 980/A已交源✅→② 981/A及其他A前提→③ 985/A→④ 981事件B接口→⑤ 980完整AV/WEB消费→⑥ AC波→⑦ A2R波。规则：各批分别解锁、不等三外部齐备；981/A不等980整包；985不能提前签发。原 r13/r14 的 M2A-08 门由⑤取代、UI-W2-AC 门由⑦取代；WEB 真语料切换不等 985/A（981/A 交付 face 扩展即可消费）。WAVES-v2 表已入 §5（§10 前）。980/A 全部已交付（12/32），无剩余 A 期工作。本轮仅修订合同。

stage: goal | plan_id: PLAN-980 | plan_revision: r15 | round: 42 | from: user | to: blocked | status: executing（12/32）| outcome: waiting（新顺序下 980 下一个可开工位=⑤，需 ②③④ 先行交付）| worktree: D:/autostack/.wt/lang-980（保留）| next: 981/A 交付 face 扩展→WEB 真语料可提前消费；否则等 ⑤

### WAVES-20261116-v2：新等待顺序裁定（用户，2026-11-16 晚段）

**新顺序取代 r13/r14 波次门**（各批分别解锁，不等三外部齐备）：

```
① 980/A 已交源 ✅ → ② 981/A 及其他A前提 → ③ 985/A → ④ 981事件B接口
→ ⑤ 980 完整AV/WEB 消费 → ⑥ AC波（1068）→ ⑦ A2R波
```

规则：980 各批**分别解锁**；981/A 不等 980 整包；985 不能提前签发解锁自身前置。


**980 各批新解锁条件**：
| 批次 | 新解锁条件 | 备注 |
|---|---|---|
| B-WEB 真语料 spliced 切换 | 981/A 交付语言侧 face 扩展（ScenarioOp payload / 参数绑定——§13.9/§13.10 前置）；**不等** 985/A 或事件B接口 | 981/A 交付即可消费 |
| B-AV（完整示例/Widgets AV 面） | **⑤ 全闭（2026-11-18 对账）**：②981/A **已验收**（MIN-A-AGE 整体交接 `e8d637bb9` review-71 pass，985 r28 复跑 PROOF A 4/4＋p981 205/205）→③M2A-08（`95c7fbad8`）→④EVENT-B 三件（`f4158e499`/`7b127afc8`/`ba92217b`） | **B-AV 已开工**（2026-11-18 用户确认；U01..06 分母冻结已交，见 §9 收据）。UI-W1-AV 签发前置=真实组件+窗口完成 B-AV-I/V/R＋接齐所需 981 B 阶段成果 |
| B-RUST W3 完整 A2R | **⑦** A2R 波（AC波后） | 原"UI-W2-AC"门由⑦取代 |
| B-TAURI | WEB 完成（真语料切换义） | 不变 |
| B-PARITY / T-05/06/07 | 各随前序 | 不变 |

**980/A 现状**：A 期全部已交付（12/32）——无剩余 A 期工作需解锁。~~下一可开工位=⑤~~ **⑤ 已解锁，B-AV 进行中**（分母冻结 2026-11-18；结构发现：全分母压在 ④外部名面域合同——与 WEB jade-garden 同墙，④合同草案=两批共同关键路径）。

### 2026-11-17 WEB 批 981 B-W1 消费（goal round 43+，切片 #6j/#6k）

2026-11-17 stage: work（WEB 批切片 #6j/#6k：981 B-W1 三接口消费——TS provider 参数环境 + V-EVT-B-ORIGIN）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——真语料全量 spliced 切换/生成器道为批剩余）| worktree: lang-980 @ plan-980-dev | code_commit: ui **4b1304fc**（#6j 参数环境）+ **后续#6k**、lang **待提交**（age_component `__age_vl_` 铸造+保留名+规范 §13.12）。

**解锁核验（磁盘级）。** 981 B-W1 事件批已落主：lang `f4158e499`+`7b127afc8`、ui `ba92217b`（SESSION typed 队列 fold）——IF-981-EVENT-PAYLOAD-B/MSG-B/PARAMS-B 三接口明确「for 980 B-AV/B-WEB」；grammar 冻结身份已按 #6i 重基线（981 owner，grammar 零改动）→ **r41 两红已闭**（ace-source 组本轮 0 红实证）。r15 WAVES-v2 的 WEB 解锁条件（981/A face 扩展交付）满足且更强（②④ 均交）。分支停靠债务提交 58bf1e3d7（D-980-WEB-GRAMMAR-981-FREEZE，含历史 stash 冲突标记）在 rebase 中**丢弃**——其三项内容均已被 master 更优形态覆盖（KNOWN-DEBT D-980-981-EVENT-PHASE 行 62-68 归 981 已交付；vue 10 预存红/O-C4 已在计划 §9 round 31/32 收据持久化）。

**交付 #6j（参数环境）。** param handler 发射源=981 声明级检查函数 `__age_hv_<n>_<h>_<pos>`（真形参，无代表值/无假数值）：ops 只含 unit handler（`Send` 对载荷变体=981 arity 域错误）；`check_fn_covering`（span 相交发现）+ `map_emitted_with`（`___age_hv_` 按位→SFC 形参名）。str 载荷 owned 转移=`a2x_str_clone`（单次求值拷贝）；多载荷 `(int,int)`=`a2x_add(dx, dy)` 双参实读；bool 直读；float 等不可承载=具名缺席（fail-closed）；unit spec 求导载荷变体=编译期具名拒（不再拼接常量体）。census v2 具名修订#2：param-handler 0→1 + param-read/float-payload 行。永久 case `p980_web_param_environment`（typed/fragment/splice/node+双负控）+ 浏览器档 `p980_web_param_browser_mount`（真实 input 事件实读参+多载荷/bool 点击；test_lane_browser_params.mjs）。

**交付 #6k（V-EVT-B-ORIGIN）。** 语言侧：组件面为 view 树（含命名 view）每条 `Lambda(assigns)` 铸 `__age_vl_<seq>` 值函数（DFS 序、赋值 RHS **原源 span**、无参环境、`__age_vl_` 入保留名表）。UI 侧：`lambda_body_regions`（脱糖源 `=> {` 区间，`desugared_lane_source` 公共坐标出口）↔ 带体 `__evt_*` 铸造 handler（mint 计数序）配对——计数失配全体缺席（fail-closed）；SFC 合成 `__evt_*` 体经 lane 拼接，负控=拼接体不得是旧 TS fallback 形（`n.value += 1` vs lane `a2x_add`）。002 真源端到端 **9/9**（6 on-block+3 view lambda 全 lane 派生）。census v2 具名修订#3：inline-lambda-only 0+absent→1。永久 case `p980_web_lambda_origin` + 浏览器档 `p980_web_lambda_browser_mount`（真实点击 5→6；test_lane_browser_lambda.mjs）。

**验证（冻结提交实测）。** UI ace_ts_provider **18/18**（CPU 档 2.6s；含 3 浏览器档排窗 AUTO_UI_ACE_WEB_E2E=1 实跑 17.1s——param str 输入事件/多载荷/bool/lambda 点击四腿全绿）；新 case 单条：param_environment 0.24s、lambda_origin（含 002 e2e+node）≈1s、浏览器档 ≤3.2s。lang `cargo t ace-source ace-av`：**ace-source 0 红**（含 981 冻结/全部 p979/p980/p981 core case）；ace-av 7 红=cl.exe spawn Deadline/ExecutionWall/wall-budget 族（bstr/console_1041/json_975/kv_zip_1064/packages_960/stdlib_1043/975——全为 C 工具链/a2r 外部腿，当时 3 个并发 cargo 在场；组件面 p979/p980/p981 case 全绿）——隔离串行重放（ace workspace 根、--test-threads=1）**7/7 全绿**（11.04s）——环境归因成立（台账记，不判 980）。

**边界（诚实）。** 全量普查聚合 (83 源 ok=72/err=11/spliced=0/absent=23) **未变**——真语料阻塞形非 param/lambda（主导=无 on 块结构源/多语句体/store/computed 族，#6d/#6g 具名边界保持）。单赋值 handler 家族为已验面；真语料全量 spliced 切换（vue/script.rs 生成道+watch/setup/expose/ext_imports）与 vue_capabilities 10 预存红（master 预存归 triage）为 B-WEB 剩余义务。

next_responsible_role: review（#6j/#6k 批级复核：AC-B-WEB 映射/反例重放/边界核对）→ merge（phase fold #8）。

2026-11-17 stage: review（#6j/#6k 批级复核）| review_scope: phase | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——authored 反例、命令重放、机械审计，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r16 | outcome: **pass（批增量；B-WEB-I/V/R 保持未勾）** | reviewed_commit: ui 4b1304fc+0f3a2ffa+复核修复 **f256276f** / lang **0b34c9776** | base: ui ba92217b / lang 01e44bb1a。

**复核内容与结果。** ①authored 反例（F-1，真缺陷）：`.Set(n) -> { .n = n }`（形参遮蔽同拼写字段）——node 实证 lane 体 `n.value = n;` 在 SFC 函数形参数字上写属性=TypeError（PARAMS-B「`.field` 恒指实例」在 TS 宿主不可表达）→ **修复=碰撞族具名缺席**（fail-closed 旧道承载，f256276f）+ 永久反例 `p980_web_param_shadowing_is_param_not_field`。②V-EVT-WEB-CONSUME 映射：实读参/多载荷/bool/浏览器事件/双负控（float 缺席、unit-spec arity 拒）逐项在永久 case；spliced 判别力由 census v2 param-read 行+体文本断言承担。③V-EVT-B-ORIGIN 映射：origin 链（`__age_vl_` span 相交发现→发射→SFC `__evt_*` 拼接）+旧 TS fallback 负控+002 端到端 9/9+浏览器真实点击 5→6。④门禁重放：UI ace_ts_provider **19/19**（CPU 8.4s；排窗 21.2s 含 3 浏览器档）；lang ace-source **0 红**（联合 run 实测）；ace-av 7 红=C 工具链竞争族（隔离串行重放 7/7 绿=环境归因，台账记）。⑤机械审计：双树 clean 于复审提交；探针全删（grep zz_probe=0）；fmt clean；零台账/legacy 写入；保留名表加法无精确断言测试冲突。⑥规范 §13.12 与交付现实一致（机制/边界/census 修订三处具名）。

**新增 case 行为/耗时表。** param_environment 0.24s；lambda_origin ≈1s（002 e2e+node）；param_browser 3.2s/lambda_browser ≈1s（排窗）；shadow 反例 <0.1s——全部 ≤10s。

**findings**: F-1（已修复闭环）；无阻断缺陷。边界核对：真语料聚合 (72,11,0,23) 未变如实记录；单赋值家族=已验面；全量 spliced 切换/生成器道=批剩余（B-WEB-I/V/R 未勾，正确）。

next: merge（phase fold #8：ui 4b1304fc+0f3a2ffa+f256276f 链 + lang 0b34c9776 rebase 落主 + 规范 §13.12）。

2026-11-17 stage: merge（phase fold #8，WEB #6j/#6k 批增量正式落主；整包 executing 12/32 保持，唯一树保留续 WEB 剩余/TAURI/PARITY，不归档/不清树）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（批增量交付）| delivery_commit: auto-ui master **f256276f**（4b1304fc→0f3a2ffa→f256276f 复核修复，零 rebase 直 ff）、auto-lang master **e6690691a**（0b34c9776 rebase onto 前进后 master——982 docs 落主所致；`git range-diff` **1/1 `=`** 全等=安全改写证明，无冲突）| canonical: docs/specs/ace-migration/m2-autoui.md §13.12（master sha256/16=**780ea992efd6f5a4**）| 落主方式：wt-guard 双树 clean + `git merge --ff-only`（无 merge commit）。**落地后核对**：双仓 tip=delivery commit ✓；ui 落主内容与复审态逐字节同一（零 rebase）、lang range-diff 全等——19/19（含 3 浏览器档）与 ace-source 0 红证据按**输入同一性复用**（裁定口径），master 过滤重放未重复执行。主检出 ui 存他会话 WIP（Cargo.lock/docs/design/**）原样保留未纳入本 fold。

next_responsible_role: work（WEB 批剩余=真语料全量 spliced 切换（vue/script.rs 生成道：watch/setup/expose/ext_imports/style）——多会话量级；B-WEB-I/V/R 勾选前置。TAURI 随 WEB；波次门 B-AV←⑤、B-RUST←⑦ 不变）。

2026-11-18 stage: work（WEB 批切片 #6l：V-B-WEB 处置——语料诊断×修复×taxonomy）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——七族缺席 taxonomy 归属处置为勾选前置）| worktree: lang-980 @ plan-980-dev | code_commit: ui **05bdf5b5**、lang **待复核提交**（text-leaf 放宽+规范 §13.13）。

**逐源诊断（83 源，census 同提取）。** 阻塞全景（具名）：text-leaf 门（states-only 误受约束）、方言 `state{}` 无臂（误落视图元素臂→伪 duplicate view=vue 门 parse 族 3 预存红根因，898-N5 时代已知债）、`$event`/元素括号属性行/引号键=年龄面语法缺口、float 域、双 widget 源、控制流/let/调用体、空体、store/HTML 伪源。

**交付。** ①lang text-leaf 门只约束文本观测车道；②方言 state 别名=model（model+state 并存仍拒）；③lane `stmt_fragments_in` 多赋值逐语句顺序发射（临时名按片段改号；976 片段接口缺口=宿主补 `a2x_ret_owned_skip` 声明）；④`widget_emit_set` 名册精化（prop 派生 emit 仅实际调用才入——444 直通红修复）；⑤vmodel 断言刷新至 Plan 008 单参取文本后来设计；⑥p980_fields 预存 span 断裂修复（dialect lib 105/105 恢复）。

**验证（冻结提交实测）。** ace_ts_provider **20/20**（CPU 6.5s 含 node 腿；排窗浏览器档含 param 档 helper 补齐全绿）；vue 门 **77/82**（chart×5=PLAN-643 分歧具名预存，归属 chart/官方包线）；dialect lib 105/105；census 具名修订 #4：(72,11,0,23)→**(75,8,1,27)**+七族 taxonomy。新 case `p980_web_textless_state_alias_multi_assign`（text-leaf+state 别名+多赋值 42/0/42 node 判别）。lang `cargo t ace-source`=**PASS**（run-ch578xkg，冻结提交实测）。

**边界（诚实）。** 语料全量 spliced 剩余=taxonomy 七族：①`$event`/括号属性/引号键语法缺口（跨 981 冻结 grammar 的语言面收敛决策）、②float 域（981 不支撑）、③双 widget（IF-981-MODULE）、④控制流/let/调用体模型（语言面扩展决策）、⑤空体（no-op）、⑥引号 handler（同①）、⑦store/HTML 伪源（提取伪影）。**B-WEB-I/V/R 勾选前置=①③④的语言面归属裁定**（用户/981 域），②⑤⑦语义无可拼接。

next_responsible_role: review（#6l 批级复核）→ merge（phase fold #9）。

2026-11-18 stage: review（#6l 批级复核）| review_scope: phase | reviewer: 同会话独立复核（限制：非独立上下文；工件重构——冻结树核对/门禁重放/机械审计）| outcome: **pass（批增量）** | reviewed_commit: ui 05bdf5b5（rebase→**7e329df9**） / lang 45ecbae87（rebase→**3c2b2a867**）。

**复核内容与结果。** ①冻结核对：双树 clean 于复审提交；探针零残留（grep zz_=0）；无调试输出。②门禁重放：ace_ts_provider **20/20**（含排窗浏览器档——str 输入事件/多载荷/bool/lambda 四腿）；vue 门 **77/82**（chart×5 具名预存）；dialect lib 105/105；ace-source PASS。③V-B-WEB 映射：census 修订 #4 数字=实测（75,8,1,27）；taxonomy 七族逐族有诊断证据（[59]=元素括号形语法缺口、[57]=f64 域、[65]/[66]=双 widget、[70]-[72]=控制流/FFI 体、[5]/[8]/[61]=空体、[25]/[29]=伪源）；vue 门 3 parse 红根因=state 无臂（修复后绿）、444=名册谎报（修复后绿）、vmodel=断言陈旧（刷新至 Plan 008 后来设计）。④机理安全：多赋值发射的控制流防线=语法层拒（age-view-stmt 只收平铺赋值——不可能静默展平，复核确认无展平路径）；临时名改号/簿记变量声明的 node 判别在永久 case（丢句/撞名即红）。⑤诚实边界：chart×5 不得单方弱化（归属官方包线）；taxonomy ①③④为语言面决策（跨 981 冻结 grammar/IF-981-MODULE/体模型），980 不冒闭。

2026-11-18 stage: merge（phase fold #9，WEB #6l 批增量正式落主；整包 executing 12/32 保持，唯一树保留）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（批增量交付）| delivery_commit: auto-ui master **7e329df9**（05bdf5b5 rebase onto 981 overlay-pin 落主后 master）、auto-lang master **3c2b2a867**（45ecbae87 rebase）| canonical: docs/specs/ace-migration/m2-autoui.md §13.13（master 上新节）| 落主方式：wt-guard 双树 clean + `git merge --ff-only`（无 merge commit；rebase 均无冲突=补丁全等）。**落地后核对**：双仓 tip=delivery commit ✓；rebase 基面（ui=981 overlay pin 数据提交、lang=docs-only）不触本批代码面——20/20/77/82/105/105/ace-source PASS 证据按输入同一性复用。

next_responsible_role: user（B-WEB-I/V/R 勾选前置=缺席 taxonomy ①（$event/括号属性/引号键语法缺口——跨 981 冻结 grammar 的语言面收敛裁定）、③（双 widget=IF-981-MODULE 域）、④（控制流/let/调用体模型扩展）的归属与开工决策；②float 域=981 不支撑、⑤空体/⑦伪源=语义无可拼接、chart×5=官方包线）。980 侧无已解锁未交付项。

2026-11-18 stage: work（WEB 批切片 #6m：年龄面语法收敛 I——用户续作指令授权 980 域内语言面收敛）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——③双widget/④体模型/⑥引号键族仍开）| worktree: lang-980 @ plan-980-dev | code_commit: lang（grammar+project+冻结值+规范 §13.14+永久用例）、ui **8111e551**（$event 归一化+census #5+永久 case）。

**交付。** ①语法：`age-event-key/mod`（事件键修饰符链，双 AGE 面）、`age-event-binding/args`（事件绑定调用形 `.Msg(args)`，仅 age-component；实参=宿主通道，投影丢弃）、`age-element-props-block`（元素括号属性行）——**多步 opt 不被组合器支持**（实测），括号/体块段归子规则。②`$event` 承载=UI 侧同长归一化（`$event`→`nevent`，span 零位移）——evt-dollar 全局 token 首版实测扰动**全部 15 profile** grammar-id 已撤回，终版仅 age-component/age-package 两值具名重基线（981 grammar_scope 测试内注明前值，其余 13 profile 逐字节不变实测对照）。③census #5：(75,8,1,27)→**(75,8,4,24)**。

**验证（冻结提交实测）。** UI ace_ts_provider **21/21**（CPU 5.7s；排窗浏览器档 16.5s 实跑含）；vue 门 77/82（chart×5 具名预存不变）；lang `cargo t ace-source ace-av` **双 PASS**（run-mc_iq43w，含冻结测试重基线后绿+新 6m 用例）。永久用例：lang `plan980_web_grammar_6m`（四形）、UI `p980_web_event_arg_forms`（三形端到端）。新 case 均 <0.3s。

2026-11-18 stage: review（#6m 批级复核）| review_scope: phase | reviewer: 同会话独立复核（工件重构）| outcome: **pass（批增量）** | reviewed_commit: lang 冻结提交 / ui 8111e551。①冻结核对：双树 clean、探针零残留、fmt clean。②跨计划冻结纪律：grammar-id 变更=**两值**且具名重基线（981 测试内注明 PLAN-980 #6m 与前值）——语义核准=本批 980 语法收敛合同（用户续作指令）；13 profile 逐字节不变有实测对照（探针转储）。③门禁：上列全绿复用输入同一性。④机理：$event 归一化同长零位移（span 合同保持）；事件绑定实参投影丢弃=宿主通道具名边界（规范 §13.14）。

2026-11-18 stage: merge（phase fold #10，WEB #6m 批增量正式落主；整包 executing 12/32 保持，唯一树保留）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（批增量交付）| delivery_commit: auto-ui master **8111e551**、auto-lang master **f16cb8ac2**（rebase 后全等落主；落地后核对双仓 tip=delivery commit ✓）| canonical: docs/specs/ace-migration/m2-autoui.md §13.14 | 落主方式：wt-guard 双树 clean + `git merge --ff-only`。

next_responsible_role: work（WEB 剩余=③双 widget 源（IF-981-MODULE 域 node 选择器）与 ④控制流/let/调用体模型（语言面体模型扩展）——多年面；⑥引号键/handler 族未收。裁定续作指令覆盖 980 域内收敛，下一切片按 ④→③ 顺序评估）。

2026-11-18 stage: work+review+merge（WEB 批切片 #6n：③ node 选择器；批级复核工件重构；phase fold #11）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——④外部名面/widget 头参数表为剩余）| delivery_commit: auto-lang master / auto-ui master（本收据提交后 ff 落主）。

**交付。** 语言面 `compile_component_scenario_states_at`（声明序选一编译，越界具名拒）；lane `compile_at`/`_spec_at`；`generate_sfc_with_lane` 多 widget 逐节点生成**聚合**。census #6：(75,8,4,24)→**(75,8,4,26)**。**④/⑥ 终局陈述（诊断驱动）**：④真阻塞=**外部名面**（fs.read_dir/listusers/complete 等 back.api/FFI/方法调用——纯结构扩展零语料收益）→需 use-import/foreign-contract（967/973 谱系）设计决策；⑥实查零拼接收益（全空体/PARAMS-B 受阻）；[65]/[66] 第一 widget 另缺 widget 头参数表+`;`（④子项）。

**验证（冻结提交实测）。** UI ace_ts_provider **22/22**（CPU 2.9s；排窗浏览器档 24.4s）；lang `cargo t ace-source ace-av`：ace-source **PASS 851**；ace-av 1369 例 1 红=sys973 wall-budget 族（隔离串行重放绿=环境归因，台账记）。永久 case `p980_web_multi_widget_node_selector`（逐节点发射/聚合/越界负控）。探针零残留、fmt clean、双树 clean 于提交。wt-guard 双树 clean + `git merge --ff-only` 落主（无 merge commit）。

next_responsible_role: user（**④外部名面设计决策**：widget 面的 use-import/foreign-contract 承载（967/973 谱系）——此为 B-WEB 全量 spliced 与 B-WEB-I/V/R 勾选的最后语言面前置；⑥零收益已闭格。980 侧无已解锁未交付项）。

2026-11-18 stage: work+review+merge（WEB 批切片 #6o：④语法收全·fail-closed 承载；批级复核工件重构；phase fold #12）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——调用表达式形（wide-ops pratt 无调用后缀）=下一层表达式面扩展）| delivery_commit: lang（含 lock 刷新 chore，982 先例法）/ ui 14050492。

**交付。** ①widget 头参数表（`age-node-params` 对 age-component 开放；`age-param-ty` 按 profile 分臂——float/f32 词仅包面保留字；参数=宿主通道 prop，不进字段/handler 语义）；②`;` 单行语句分隔（组合器约束：多步 many/alt 混排均不支持——分号归子规则尾单步 opt）；③调用语句 `age-view-call-stmt` **语法收全 + body_foreign fail-closed 承载**（检查函数不铸=部分体 verify 禁；场景派发具名拒 `component-unsupported`=部分重放禁——a2ts.foreign-unsupported 域合同，不做未验证旁路）；④981 lifecycle/watch `member_stmts` 适配 alt 解包（6 红修复）；⑤grammar-id 两值二轮具名重基线（f35cf593…/6c1bb477…；Core 等 13 profile 逐字节不变）；⑥root Cargo.lock 刷新（982 两次同因先例法）。census #7：(75,8,4,26)→**(75,8,7,23)**。

**验证（冻结提交实测）。** UI ace_ts_provider **23/23**（排窗浏览器档同绿）；lang `cargo t ace-source` **PASS**（981 lifecycle 族修复后）；永久用例 `p980_web_6o_face`（头参数+`;` 编译）+ `p980_web_head_props_semi_and_foreign_body_fail_closed`（拼接+调用体负控）。探针零残留、fmt clean。

next_responsible_role: work（调用**表达式形**（`on_select(next)` 作 RHS/pratt——wide-ops 无调用后缀）=表达式面扩展；随后 [65] 全解锁。④完整承载仍待 976 后继目标域决策）。

2026-11-18 stage: work（WEB 批切片 #6p：调用表达式形·通道调用承载；#6o 合同闭合）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（**B-WEB-I/V/R 保持未勾**——④完整承载（外部调用进受检体=976 后继目标域决策）与真语料全量 spliced 的生成器道（vue/script.rs：watch/setup/expose/ext_imports/style 族）+vue 门 chart×5 具名预存（PLAN-643 分歧，归属 chart/官方包线）为批剩余）| worktree: lang-980 @ plan-980-dev | code_commit: lang **42927d6f7**、ui **c80a3720**+锁 chore **f43ef59b**。

**事实修正（探针证伪 #6o 前提，adapt 表记录）。** "wide-ops 无调用后缀"**不成立**：pratt 调用后缀一直在场（文法 `call` 块+引擎 WantOp 臂）——`.x = on_select(1)` 解析通过、**Bind** 阶段 `source.undefined` 拒；调用**语句**形 #6o `age-view-call-stmt` 已解析（[65] 全源语言面编译 OK）。#6o 后实况缺口（master 在场缺陷，探针实证）：①通道调用（callee=widget 头参数/宿主通道 prop）无承载——检查函数只铸赋值、**调用语句被静默丢弃**（[65] `.Bump` 拼接体只有 `n.value = next;`，`on_select(next)` 丢句）；②`handler_check_fn` 不守 `body_foreign`（#6o 收据声称"检查函数不铸"，实际照铸→部分体 verify+部分拼接）；③域外调用 handler 进 ops=整 lane 编译失败=**兄弟 handler 连带全体缺席**；④#6o 收据引用的规范 §13.16 从未落档。

**交付。** ①投影：调用语句 callee∈头参数→`ComponentChannelCall` 行（原源 span；具名实参 fail-closed）；②实参验证=检查函数**死 let**（无注解推断；`on_x(typo)` 红 `source.undefined`——真核非文本拼，未验证拼写绝不进拼接体）；③lane：赋值片段+通道行按**源 span 序 merge**，`emit('<Pascal>', args)` 契约事件形（Pascal∈msg 变体）/`props.on_x(args)` 回调 prop 形双发射（老道 PLAN-668 R-23/Plan 444 同约定——SFC defineEmits/defineProps 声明机制承载），实参按精确 span 经 a2x-ts 发射+宿主映射；④fail-closed 闭合：body_foreign 检查函数不铸、值位置（RHS/嵌套实参）通道=域内具名拒、ops 派发通道=零效果具名拒、foreign lambda 值函数不铸（防丢句）；⑤ops 豁免=兄弟保全（方言预扫描：含调用语句的 unit handler 不进 ops）；⑥描述出口 `handler_channel_calls`（行+body_foreign+n_assigns）。**无语法改动——grammar-id 零漂移**（ace-source 冻结测试绿实证）。census 具名修订 #8：聚合 (75,8,7,23) **不变**——[65] 计数在 #6o 已计入（部分拼接态），本切片修**内容正确性**（`.Bump` 体补全 `emit('Select', next);`）与兄弟保全语义；判别力移至 `p980_web_channel_call_lane` 六臂断言。

**验证（冻结提交实测）。** lang `cargo t ace-source ace-av` **双 PASS**（run-hy8xo3yp；首跑=case-parent 锁陈旧工具准备失败，root Cargo.lock 刷新后绿——982/#6o 同因先例法，随批入库）；UI ace_ts_provider **24/24**（含新 case 0.34s）；vue 门 **77/82**（chart×5 具名预存不变）；dialect lib **105/105**。新 case 行为/耗时：lang `plan980_web_6p_channel_call` 6 例（行投影/值位置拒/ops 拒/死 let 判别力/域外标记/foreign lambda）合计 0.06s；UI `p980_web_channel_call_lane`（[65] 完整体/混合兄弟/域外兄弟保全/props 形/RHS 缺席/lambda 缺席）0.34s——均 ≤10s。探针零残留；本批 9 文件 fmt clean（仓内其余 fmt 分叉为预存，未纳入）。

**边界（诚实）。** ④完整承载（`fs.read_dir`/`listusers` 等外部调用进受检体）=976 后继目标域决策（a2ts.foreign-unsupported 域合同变更，§13.15 终局陈述保持）；[72] RHS `listusers` 仍缺席（④族）；lambda 面通道调用承载=后续切片（本批只闭"不铸"防丢句）；watch/lifecycle 体调用语句按方言合同跳过（#6o 边界）；`on` 块/lambda 体**单行多语句**形（`a; b` 不换行）=`list-nl` 文法既定边界（探针实测）。主检出存他会话 WIP（Cargo.lock/docs/design/**，#6j 收据已surface）原样保留未纳入。

next_responsible_role: review（#6p 批级复核：AC-B-WEB 映射/反例重放/边界核对）→ merge（phase fold #13）。

2026-11-18 stage: review（#6p 批级复核）| review_scope: phase | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——authored mutation 反例、冻结树核对、门禁复放、机械审计，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r16 | outcome: **pass（批增量；F-R1 复核修复后闭；B-WEB-I/V/R 保持未勾）** | reviewed_commit: lang 42927d6f7→复核修复 **25c53f77d**（rebase→3c7bf5688）/ ui c80a3720+f43ef59b。①冻结核对：双树 clean；ui 分支基=master tip 14050492 零漂移；lang 分支基 e05531572（master 仅 docs 前进）。②门禁复放：UI ace_ts_provider **24/24**（复核会话冻结提交实测 1.83s）；lang 双门禁（run-hy8xo3yp 双 PASS）按输入同一性复用——产品代码自该跑后零改动（F-R1 仅测试文件）。③**authored mutation 反例（判别力实证）**：M1=移除 handler_check_fn 的 body_foreign 门——首测**未红**（UI 侧旗标短路掩蔽）→ **F-R1 真缺陷**：语言侧"检查函数不铸"合同无持久反例且首版强化断言因 `sf_` 前缀空转（DIAG 实证 def 名=`sf___age_hv_*`）；修复=模块内容断言（`contains("__age_hv_")`/`__age_vl_`）+ 复验 M1 **红**（`sf___age_hv_0_0` 被抓）；M2=移除 lane ops 豁免——**红**（`sibling preserved: [".Go",".Stop"]`，既有断言即判别）。④终版门禁：ace-av 全组 1 红=mempeek/closures/audit1030 环境族（**隔离串行重放 8/8 绿**，#6j 同款台账归因）；conformance p980_web_6p **6/6**。⑤机械审计：探针零残留、调试打印零（eprintln DIAG 已随修复移除）、fmt clean（本批文件）、无 legacy 写入。⑥规范 §13.16 与 as-built 一致（含"无语法改动 grammar-id 零漂移"——ace-source 冻结测试绿实证）。新 case 计时：lang 6 例 0.03s、UI 六臂 0.24-0.34s（≤10s）。

**findings**: F-R1（已修复闭环——25c53f77d：语言侧 fail-closed 合同的铸造事实必须在受检模块内容上可判别，不能依赖消费侧旗标掩蔽）；无阻断缺陷。边界核对：④外部调用域合同+生成器道（watch/setup/expose/ext_imports/style 真实语料）=批剩余（用户本轮确认）；jade-garden 实测 0/91 拼接已立证（离 983 真实分母的距离）。

2026-11-18 stage: merge（phase fold #13，WEB #6p 批增量正式落主；整包 executing 12/32 保持，唯一树保留续 WEB 剩余/TAURI/PARITY，不归档/不清树）| plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（批增量交付）| delivery_commit: auto-lang master **3c7bf5688**（1974554c2 切片+3c7bf5688 复核修复；rebase onto 96750e6fa——master 因 982 docs 前进；`git range-diff` 全等 `=`+新增修复提交；ff-only 无 merge commit）、auto-ui master **f43ef59b**（c80a3720+f43ef59b，基=14050492 零漂移直接 ff）| canonical: docs/specs/ace-migration/m2-autoui.md §13.16（随 lang 落主）| 落主方式：wt-guard 双树 clean + `git merge --ff-only`；双主检出他会话 WIP（docs/design/**、Cargo.lock）原样保留——lang 侧脏锁与分支锁 blob 同一（c2e1a1ebf）stash-merge-drop 零损失；ui 侧脏锁为他会话 ace-host/auto-web 依赖刷新（982/983 谱系）按**并集**恢复工作树（不入本 fold）。落地后核对：双仓 tip=delivery ✓；24/24 与双门禁证据按输入同一性复用。**批剩余（用户 2026-11-18 确认）**：①④外部调用进受检代码的域合同（976 后继目标域决策——a2ts.foreign-unsupported 变更）；②Vue 生成器对 **watch/setup/expose/外部导入/样式** 等真实语料的覆盖（生成器道多切片）；③census 全量 spliced→B-WEB-R 终核→勾选 B-WEB-I/V/R。

next_responsible_role: work（生成器道扩面：watch/setup/expose/ext_imports/style 族逐片切——建议以 jade-garden 39 文件为正式分母；④域合同=用户/976 后继决策输入，建议先出合同草案）。

### 2026-11-18 B-AV 开工授权（用户确认）+ U01..06 分母冻结（B-AV-I 首件）

**开工门核验（用户 2026-11-18 状态更正，本会话对账确认）**：r15 表 ⑤ 链**全闭**——②981/A 已验收：MIN-A-AGE 整体交接收据 `e8d637bb9`（event A＋B-PROOF-I/V[A]，review-71 pass；985 r28 接收复跑 **PROOF A 4/4**、**p981 205/205**，r26 反馈①–⑤全闭，NavCall L-45 归 B/W1 起三波）；③985/A=M2A-08（`95c7fbad8`）；④981 EVENT-B 三件（`f4158e499`/`7b127afc8`/ui `ba92217b`）。**B-AV 按 r15 原门开工（非门修订）**：按组件组推进 B-AV-I/V/R，与 WEB 补覆盖/复审并行。UI-W1-AV 签发仍需：真实组件+窗口的 I/V/R 全分母 + 所需 981 B 阶段成果接齐。（本会话早前"②差 PROOF A 面"为陈旧 r26 口径，已由用户更正、对账落档。）

**分母冻结（35 编号例全量实测，`src/front/app.at` sha256/16 + 行数）**：

| 组 | 例（hash｜行） | 现有例级 oracle（旧道） | 冻结发现：域依赖 |
|---|---|---|---|
| U01 | 001 b2763be5178fa040｜35；002 04937c35325faa74｜79；003 2b50ef4456cb4409｜75；011 a0d687bd00b1db6c｜1526 | 001/002=ace_native_window（AV/桌面面 ✓）；003/010=example_theme_migration（主题面，旧道） | 003：**f64 字段+use-import+math.\***；011：**f64×14+use prog_util/themes+math.\*** |
| U02 | 004 f8dd0e99ceea559c｜105；005 446da5dc1bb68b6a｜134；006 238715165c514ce1｜69；007 4b378554dc81de51｜488；008 8576cd8d6c31e2f5｜375；009 f2406796a4f0bddd｜277；010 15a632b4cda27cd2｜177 | 010=theme 面；余无例级 | **全部 use-import**（themes） |
| U03 | 012 d131df41b5878b33｜700；013 dc7682c379fb6551｜132；015 70702d308c8957af｜125；016 1e2d42d8f827bcb4｜225；018 261e952afd2ae296｜112；022 a8ab50185e8bc602｜72 | 015=ui_snapshots（提取面）；余无 | **全部 use-import**；除 012 外含 ext-call（math/store/back） |
| U04 | 014 88841a8bb0904cf6｜863；017 3ffa54a22c80f4a9｜124；023 9e45d757ffa41a13｜91；025 0f3a4305e11672ab｜202；026 d37e703351dedfeb｜1087；027 6306d7d624aba15d｜1599 | 无例级 | 服务/数据族（014/017 归 U04 冻结——原 U03 范围"012–018"与 U04 显式列冲突按功能裁定）；use-import+网络/DB/文件 |
| U05 | 019 8e95ac35d2094b83｜160；020 d167cac22f8fd9a1｜203；024 b89e1aad4741b1a7｜249；029 69c5c86b33cf8bac｜805；030 fe9e687f303374af｜133；031i 7085e9e463cc3989｜368；031p d9873a662f7d91cd｜527 | 无例级 | 媒体/GPU 环境门+use-import |
| U06 | 032 7896f812ec4f0466｜53；038 623ee1f92941e6a1｜139；041 183f9e058eb227be｜306；021 df21a89a1d8be9fc｜84；028 4eaec20d6f87c52b｜899 | 无例级 | 剩余归 U06（"剩余原例"条款）；async/复杂混合 |

Widgets 画廊=auto-os 顶层 `widgets-gallery/`（PLAN-590 迁出；**OS064 域**——U01 行"Gallery button/label/input"跨仓归 OS064 owner，本批消费其正式接口）。特殊目录（ace-hello-minimal/ace-node-counter/ace-web-lane/stylekit/themes/remote-frame-e2e）为基建非分母例。

**结构性发现（冻结主结论）**：**全 35 例的受检 AV 承载压在 ④外部名面域合同上**——`use themes.themes` 普遍存在（U02/U03 全量实测）、math/store/back 调用广泛、003/011 另需 f64 字段域（981 裁定不支撑；982 round-137 f64/u64 值类已落 AV 运行时——**组件面字段域与 AV 值域的接通=981/用户域决策**）。与 WEB 的 jade-garden 实测（0/91、`use back.api`）**同一堵墙**：④域合同（use-import/外部调用进受检体，976 后继目标域决策）是 B-AV 与 WEB 的**共同关键路径**，杠杆高于任一批单独推进。

**U01 可执行面（无 ④ 前提）**：①011 的 pdigit 算法体（use prog_util 之外的字面算术子面）与 002 深化——U01 oracle 三件（原值/事件轨迹、中文/非法输入、两实例隔离）在已解析面上补永久例；②组件面 AV 对无导入简单形的承接普查。

next_responsible_role: work（**优先序修订建议**：④域合同草案=最高杠杆（两批共同前置，建议先出 use-import/外部调用的目标域合同草案供拍板）；并行 U01 可执行面（001/002 oracle 深化）。WEB 生成器道续行不变）。

### 2026-11-18 B-AV U01 首切片：文本输入组件的 AV 窗口面（work）

**stage: work | plan_id: PLAN-980 | plan_revision: r16 | outcome: pass（批增量；U01 输入面 CPU 档交付）| worktree: lang-980 @ plan-980-dev | code_commit: lang `bf95b9165`、ui `07df742c`。**

**交付。** ①UI 桥：`InputSpec` 投影（`input (value: .field) { oninput: .Msg }` → 值绑定字段 + typed 消息，目标编号接 buttons 后）+ `ScriptStep::TypeText`（真实键入与脚本同一边界——pending 文本 FIFO 与 provider 事件队列 (instance,target) 一一配对）+ typed 派发 `dispatch_with_args`（str 原样/int 桥面解析）+ **非法数值输入具名拒零效果**（"abc"→int 绑定：DispatchRefused、状态不动、零 apply）+ close 清队（键入配对随 session 死）；②窗口面：`NodeDesktopMsg::Type` + `View::input` 受控单向回显（iced INPUT_TEXT 同口——键入→typed 队列→AV 重放→apply→回显）；③e2e 例程 `type` 动作（文本=余段，中文可用）；④U01 四 oracle 永久 case：**中文原值回显**（"你好 Auto" str 载荷经 typed 队列→AV 场景重放→apply str）、**数值输入**（"42"/-5）、**非法输入拒零效果**（apply 证据面断言：str 4 次=2 键入+2 初值、int 4 次、非法不增）、**两实例隔离**（i0/i1 各自键入互不串）+ 静态面负控（无 model 源带输入目标=装载拒）。

**附带发现①（#6m 投影缺口，已闭）**：`input (value:.a){oninput:.SetA}` 的括号属性与体条目**自 #6m 起在 ACE 投影全丢**（`Element(input).items=[]`——消费全在方言侧故未暴露）。根因：`collect_view_items` 的 `unwrap_child` 假设单层 alt 包装——props 段条目（`age-view-entry` 具体规则，非 `age-view-item` 包装）被错误下钻到 `age-event-key` 后整条丢弃；体块（`age-element-props-body`）包装同理。修复（lang `bf95b9165`）：先按自身 kind 匹配、未匹配包装 raw 递归下钻。**零回归实证**：census 冻结聚合 (75,8,7,23) 不动（lane 按区间/函数发现，不受 view 树条目恢复影响）+ 979/981 冻结面全绿。

**附带发现②（文法边界，具名不改）**：model 多字段**逗号同行**不被支持（`list-nl`=换行分隔——与 on 块成员同族；探针矩阵二分定位）。夹具按换行形；此为既定文法边界非本切片回归。

**验证（冻结提交实测）。** lang `cargo t ace-source ace-av` **双 PASS**（run-spewnfcp）；UI ace_node_desktop **10/10**（串行 0.81–1.00s，冻结命令 `--test-threads=1`——并行跑=provider 单例竞态环境族，非判据）；ace_ts_provider **24/24**（1 红=node Temp host.mjs 竞态，隔离重放绿+全量重跑绿=环境归因台账）；vue 门 **77/82**（chart×5 具名预存）；ace_native_window **5/5**。新 case 计时：U01 主 case ~0.2s、静态负控 <0.1s（≤10s）。探针零残留；双仓 fmt clean（本批文件）。

**边界（诚实）。** e2e 排窗输入档（真窗真实键入回显）未实跑——窗口面代码已接线（Type 消息+受控渲染），排窗档随 B-AV 后续切片或里程碑跑；011-pdigit 子面、Gallery（OS064）未动；④域合同（use-import/f64）仍是 003/011 全量与 U02–U06 的墙（§9 分母冻结）。

next_responsible_role: review（U01 首切片批级复核）→ merge（phase fold #14）；后续切片=④域合同草案（最高杠杆）或 U01 余量（e2e 排窗档/011 子面）。

2026-11-18 stage: design（PHASE-④-EXT：外部名域合同草案 v0；仅设计/规划——零代码、零建树、零运行、零 fold）| plan_id: PLAN-980 | plan_revision: r17 | outcome: **draft（待拍板）** | 产出：§5 PHASE-④-EXT（N1–N4 名面/六轴/承载矩阵/三方案/条款 C1–C8/退出条件）、§6 V-DOMAIN-④、§8 阶段块、§10 D-④-1..7、本节 | 依据：35 例静态 census（34/35 含 use；137 符号；17 例真实引用；宽数值 2 例）、grammar profile 门实读、`age_component.rs`/`project.rs`/`ast.rs`/`fragment.rs`/`a2x-ts`/`m2-typescript.md`/`packages/binding.rs` 静态核对（package 面拒 `use.rs`）| 事实订正：003 `double`×2（非 f64）、011 `float` 去注释 7 处且 model 全 int/str/bool、031i `float`×3 字段漏列 | 未改动/未勾选任何原任务（12/32）；B-AV/B-WEB 状态、原 AC/V、语法/实现/规范主体不变 | next_responsible_role: user（D-④-1..7 裁定）→ 裁定后 ④-A 另立具名切片；U01 首切片的 review→merge（fold #14）不因本 phase 改变。

2026-11-18 stage: new/revision（PHASE-④-EXT 裁定落档 + ④-A 会签落档）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **裁定已落（7/7）；④-A 合同 v2（981 会签版）** | 七项裁定见 §10 裁定结果表（A 先行/开放符号表导入/宽数值保留拒/foreign 后置/TS 粒度维持/host 表不建/小范围验包绑定）| 981 会签（lang master `81b0a03b1`）修正实施路径：**不开 age-component 面，多模块闭包走 `age-package`，age-component 单文件 `use` 改具名拒，硬条件=惰性导入；grammar 零改动**（详见 §5 ④-A v2 S0–S8）| 本轮只落规划，无代码/建树/运行 | next: ①#14 收口（下表）→②④-A 首切片（示例以可解析性实测选例）。

2026-11-18 stage: review+merge（B-AV U01 首切片批级复核 + phase fold #14）| review_scope: phase | reviewer: 本会话独立复核（工件重构：diff 审计/门禁重放/预存缺陷扫描/authored 补证；限制：非独立上下文）| outcome: **pass（批增量交付；B-AV-I 保持未勾——全分母仍压 ④）** | reviewed_commit: lang **95b0ccb49**（rebase onto `81b0a03b1`）、ui **91d549ea** | delivery: auto-lang master **95b0ccb49** + auto-ui master **91d549ea**（双 ff-only，无 merge commit；wt-guard 双树 clean）。

**复核内容与结果。** ①冻结核对：双树 clean；lang diff=2 文件 +31/−2（`project.rs` 投影修复 + `age_component.rs` mut 清理）；ui diff=3 文件 +540/−16（桥 224/测试 318/示例 14）。②门禁重放（冻结提交）：lang `cargo t ace-source ace-av` **双 PASS**（run-22yj5299；首跑 ace-av 3 红=`bstr_1064_v01_large_64k_a2r`/`sys973_t08_seek_permissions_mtime`/`sys973_v03_process_spawn_capture_timeout_exit`，**隔离串行重放 3/3 绿**=环境/工具链竞争族，台账归因）；UI `ace_node_desktop` **11/11**（1.59s 串行）、`ace_ts_provider` **24/24**（3.67s）、真窗 counter e2e **1/1**（2.18s，`AUTO_UI_ACE_NODE_E2E=1`）。③**真窗输入档（用户本轮要求，补证）**：新增永久 case `p980_u01_window_input_e2e`（env-gated）——真窗进程 + `type` 脚本→typed 队列→AV 重放→ui980 FFI apply→受控回显；i0 `你好 Auto`/42（`abc` 具名拒零效果）、i1 `乙侧`/−5；**1/1 绿 6.38s**。边界：OS 级键盘事件不可自动化，键入经桥注入（与真实键盘同一注入边界）；窗口面验证=真窗渲染 input 目标 + FFI 提交后的状态行（回显源）。④**authored 扫描（预存缺陷，已修）F-R2**：`p980_node_desktop_permission_revocation_refuses_before_dispatch` 自 B-NATIVE-R fold 起**丢失 `#[test]`**（与兄弟函数重复属性并存），永久撤权控制**静默未跑**；恢复后 10 唯一用例（含撤权）全绿。⑤机械审计：复核轮仅测试文件 +112、无产品代码改动；探针零残留、无调试打印（仅 env-gate 注记）、changed files fmt clean；主检出他会话 WIP（lang `docs/design/**`、ui `Cargo.lock`/docs）原样保留。⑥边界：B-AV-I 全分母仍压 ④-A（17 例被引用导入）；本折只交 U01 输入子面（CPU + 真窗键入通道）。

**交接 981（按版本，reviewed_commit 绑定）**：①源/夹具：`ace-node-counter`（002 派生）、`ace_node_desktop` 测试内 `input_fixture`（U01 输入形）；②命令：`cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1`（CPU 11/11）；排窗=`AUTO_UI_ACE_NODE_E2E=1` + prep（exe `--prep-only`）；③oracle：`p980_u01_input_text_numeric_reject_and_two_instance_isolation`（四 oracle）、`p980_u01_window_input_e2e`（真窗）、`p980_node_desktop_permission_revocation_refuses_before_dispatch`（撤权）。981 仍缺=**typed UI/资源宿主面**与**980 合流 oracle**（本折只交输入子面；合流 oracle 待 ④-A/W1 全分母），已有证据先接、缺项继续跟踪。

**#14 可独立复验最小集（UI 侧 + 真窗；判据=输出数字，不采信摘要）**

```
# 0) 准备（排窗档需要真窗 exe；CPU 档不需要）
cargo build -p auto-ui-runtime --features ace-bridge --example ace_node_desktop_window
./target/debug/examples/ace_node_desktop_window.exe --prep-only        # 退出码 0

# 1) CPU 全套（reviewed ui 91d549ea）
cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1
#   期望：11 passed（含 permission 撤权、U01 输入四 oracle；两条 env-gated 窗口档未排时 ok）

# 2) 真窗键入档（U01 输入）
$env:AUTO_UI_ACE_NODE_E2E='1'
cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1 p980_u01_window_input_e2e
#   期望：1 passed，finished in ≈6.4s；证据 JSON 断言 i0 note=你好 Auto/count=42、abc 具名拒、i1 note=乙侧/count=-5

# 3) 真窗计数档（回归对照）
cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1 p980_node_desktop_window_e2e_counter_full_script
#   期望：1 passed，≈2.2s
```

语言侧对应 `cargo t ace-source ace-av`（reviewed lang `95b0ccb49`，双 PASS）。若某档数字不符，先核 `git rev-parse HEAD` 与上文 reviewed_commit 一致，再按跑法差异（串行/环境门）归因，不以父 exit 0 代替子项数字。

2026-11-18 stage: design-probe（④-A 首切片选例实测；仅临时探针，探针与 harness 注册已删净，零产品代码）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **blocked-on-routing（实测发现 ④-A 前置=包面 widget 体表面补齐）** | 证据：`plan-artifacts/plan-980/closure-surface-probe-20261118.txt`（原始输出+35 例规模）| 结论：age-package 面缺 顶层 `style`（20/35）、payload msg（17/35）、反引号文本（2/35）；001 唯一可编译且无 helper；`msg{A,B}`/`design`/`use m.n: A,B+msg` 在包面 OK。⇒ ④-A 顺序建议改为 S0.5a 表面补齐 → S0.5b 惰性绑定 → S1 真实例 helper+AV/TS。路由选项 §10 R1/R2/R3，未裁定前不改 grammar/身份、不以合成 fixture 冒充。| 清理核对：`zz_probe_980_closure.rs` 删除、`tests/conformance.rs` 还原；worktree clean。

2026-11-18 stage: revision（④-A 入口同步 + 形状准入补测 + 代码顺序落档；用户复核接受 981 修正）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **合同同步完成，代码未动** | ①同步：用户接受"多模块闭包整体走 `age-package`、作者保原 `use` 拼写"；§5 三方案 A 与 v0 条款改标历史、D-④-2 原行标"已被裁定结果取代"；S8 增用户最小验收三件（未引用 `themes` 不拉模块／被引用 helper 改结果／缺模块·缺符号·私有符号具名拒；引用驱动限 ④-A 符号导入、普通包面初始化语义不变）。②补测（包面真实形状）：`ace-node-counter`/`ace-hello-minimal`/001/legacy-on-block/canonical-msg-named **全 OK** ⇒ 形状墙不是 `widget`/`node` 或 `on` 块，而是匿名 payload msg（17/35）、顶层 style（20/35）、反引号（2/35）、package block（5/35）、`$`（026）。③代码顺序：**点分 `use` 投影 + 惰性绑定同批**（单加投影→34/35 例 eager 全红；单加惰性→无输入）→ 在已可解析真实例派生源接入真实 helper 验三判据 → AV 真窗 → 同源 SFC。④未完前置：整例真实源仍需 S0.5a 表面补齐（§10 R1/R2/R3 待裁）。| 未改 grammar/身份/产品代码；探针删净、harness 还原、worktree clean。

2026-11-18 stage: countersign+work（**F32 消费同窗 v1：980 `auto-ui-runtime` 侧回签 + 配对提交**；包桥 F32 不纳入）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **980 侧 ready（分支配对提交 `8a713b89`）；窗口待 981 先合、auto-ui 紧随** | 981 侧：`17fafb1d9`（`plan-981-f32-window`，未合）。

**回签内容（逐项接受 981 窗口合同）**：①`AceStateValue::F32(f32)` 与 981 **同策略按位相等**（手写 `PartialEq`/`Eq`：NaN payload 可判别、`-0.0 ≠ 0.0`；`Eq` 保留）；②`From<&ComponentFieldValue>` 增 `F32`；`InitialValue::Float` 映射两处（`ace_state_bridge`/`ace_native`）；③`ace_ts_provider` 类型串 `f32`；④`ace_node_desktop` 呈现=**Rust `Display`**（宿主显示模型只有 str 车道→F32 以格式化串经 `ui_apply_str` 提交；值语义仍在引擎）；输入绑定对 F32 保持具名拒（本窗不接 f32 输入）；⑤**组件桥验收 3/3**（`tests/plan980_f32_component_window.rs`）：`Float(1.25)` 种子 → `.Step`(+0.25) → 读回 `1.5` **bit-exact**；`Int` 种子具名拒（不隐式提升）；窗口显示 `1.5`（无 `1.500000`、无逗号小数）；⑥**包桥 F32 不纳入本窗**（包桥只观察 int、无 props/initials——其 F32 另案）。

**联合编译与窗口协议**：按协议以 **master + 981 f32 提交** 的 scratch 树联合编译（981 分支基点落后，直接 checkout 会红 root crates）；配对提交=`D:/autostack/.wt/lang-981-f32/auto-ui` 分支 **`f32win-ui` @ `8a713b89`**（未合）。窗口顺序=981 先合 `17fafb1d9` → auto-ui 紧随 `8a713b89` → 各自冻结门（981：`ace-source ace-av`；auto-ui：其固定组）；任一侧失败双方同窗回滚。

**窗口阻塞（须先解；非本窗引入）**：lang **master tip `4d9895712` 的 `auto-lang-ace-host` 已红**——experimental ace_core 新增 `Item::UsePy` 后 `crates/auto-lang-ace-host/src/session_bridge.rs` 两处 match 未补臂（E0004）。本回签在 scratch 树打了 **2 行未提交补丁**（`UsePy` 臂）才通过联合编译；正式开窗前须由该改动 owner（982 谱系）补臂或确认 master 恢复绿，否则 auto-ui 无法构建（与本窗无关，但会挡住窗口）。

2026-11-18 stage: work+merge（④-A 首片 #2d：**`<script>` 段浏览器执行档**；非 fold）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（脚本段浏览器证据已交；完整 SFC template 阻塞已实测）** | delivery: auto-ui master **`dbed5997`**（ff-only；rebase 后 `crates/auto-ui-runtime` 树与已测提交一致）| worktree lang-980 @ plan-980-dev。

**交付。** ①`tests/plan980_closure_browser.rs` + `examples/ui/ace-web-lane/tests/test_pkg_script_browser.mjs`：把包片段 lane 发射的 **同一份** helper 定义与 handler 调用行放进真实页面（playwright/chromium，`AUTO_UI_ACE_WEB_E2E=1`），点击一次按钮断言 `#count`：base helper（+2）**2→4**、edited helper（+10）**10→20**——浏览器执行面与 CPU/AV 观察同源同值。②完整 SFC template 的阻塞已**实测**：既有方言解析器（`auto_ui_dialect::UiDialect`）不认 AGE `node`/匿名 msg 源（`generate_sfc_with_lane` 在派生源上返回 20 个 parse 错误，含 `UndefinedVariable{Inc}`/`Expected term, got View`）⇒ "完整 SFC"需 **view 树→Vue 模板**生成器道（或方言 AGE 支持），列为下一片；不以手写 template 冒充。

**验证。** 浏览器档 1/1（3.3s，env-gated）；回归 `plan980_closure_sfc` 2/2、`plan980_closure_window` 3/3、`ace_node_desktop` **11/11**；changed files fmt clean。

**边界（诚实）。** 浏览器档执行的是 **SFC `<script>` 段**（页面宿主与按钮为测试装置）；完整 SFC（template 由 AGE 视图生成）未交；包桥仍只观察 int 字段、无 props/initials。

2026-11-18 stage: design-revision（**981 回签（有条件）+ 980 回执**；P3 收窄为 age-package-only；本轮不动文法）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **回执完成，落刀工作单就绪（§5 P3 七节）** | 981 回签：`edfa4220c`（981 §9 收据）。

**981 修订（三处）**：(a) profile 门收窄为 `[age-package]`，`age-component` 不开（组件面无语义 const/public 规则、const 由 packages 绑定器消费、`ModuleGlobalsPlan` 仅 AGE 包前门钉入）；(b) 身份做法 A（980 同提交更新冻结值，只动 age-package，13 邻居转储）；(c) 重名走 `source.module-duplicate-declaration`、style 函数返回类型待定、下游车道（AV/TS/SFC/A2R）const 读风险要选路；(d) 诊断码细化 + 建议 P3.1 调用点一律 `style-call` 具名拒（不折叠）。

**980 回执（两问）**：①(c)3 选 **投影期字面量替换**——P3.1 中 `style: NAME` 与 `style N = other` 在投影期替换为字面量；带参调用一律 `style-call` 具名拒；**明示撤回**"用法复用 mangle/可见性"的主张（声明仍落 const/fn）。理由：语料用法全是编译期 class 串，避免 TS/SFC/A2R 的 global 访问器缺口；跨模块常量折叠收益 0、身份/求值序风险不为 0。②(c)2 选 **`Str`（owned）**：参数化 style → `Item::Fn` 返回 `Str`；零参 → const `Str`（与文本叶 `.clone()` 约定同源，避免 view 生命周期）。

**落刀工作单**（§5 P3 七节）：文法三件（`style-head`/`style-item`/`style-params`，仅 age-package）→ 投影（const/fn + defaults/call 具名拒 + 声明/导出/重名）→ 用法投影期字面量替换 → 永久用例九项 + 13-profile 转储 + age-package ID 重基线 → 门禁 `cargo t ace-source ace-av`；981 在 980 文法提交上复跑 `p981_grammar_scope` 核 diff。**本轮未改文法/代码。**

2026-11-18 stage: work+merge（**P3 `style` 声明面落文法**：age-package only；981 回签 `edfa4220c` 收窄版）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（style 面已交；默认值/插值/调用形/导入引用=具名拒）** | delivery: lang master **`e8e4c16da`**（rebase 后 ff-only；合并 master 含 981 ABI/spawn 前进，披露冲突解决=冻结值按合并后文法重算）| worktree lang-980 @ plan-980-dev。

**交付。** ①文法（`grammars/auto-source.at`，**仅 age-package**）：contextual `style-head`（不进 token 表）+ `style-item` + `style-params`/`style-param`/`style-param-ty-ann`/`style-param-default`（组合器约束：多步 opt 拆子规则）+ 项位 alt 与 `public-body` 的 style 臂。②投影（`cst/frontdoor.rs`）：零参字面量→`Item::Const{Str}`；`pub style`→公共面 exports；参数化（无默认/无插值）→`Item::Fn{ret:Str}`；**默认值→`source.age.style-defaults`**（span=默认值 token）；**参数插值→`source.age.style-interp`**；**调用形→`source.age.style-call`**（组件投影前的 CST 预扫 + 声明初值两处）；本地 ident 复制延后解析（允许前向；未命中→`style-call`）；**导入 style 引用→`style-call`**；用法 `style: NAME` 在投影期替换为字面量（981 (c)3，无 module-global 读）；重名走现有 `source.module-duplicate-declaration`；`style` 作普通标识符仍合法。③身份（做法 A）：`plan981_grammar_scope.rs` 的 **age-package 单值** 重基线（合并 spawn 后基线 `0290d0ab…` → **`2d272979…`**，注记三次重基线）；**age-component 保持 `19a1376e…`**；13 profile 逐行冻结表绿=邻居未变。④永久用例 `plan980_style_surface` **9 例**。

**验证。** 目标 9/9；grammar 7/7；`cargo t ace-source ace-av` **双 PASS**（合并后提交上复跑）。

**边界（诚实）。** ①stylekit 的 3 条参数化 recipe 带默认值→**本刀具名拒**（007/016/017/029/031p 的 stylekit 面仍未解锁）；②参数插值未接（`style-interp`）；③带参调用不折叠（`style-call`）；④导入 style 引用不接（`style-call`）；⑤用法替换只走主 view 树（命名 view/其它出口未遍历——具名边界）。

2026-11-18 stage: fix+merge（**默认构建 E0433 红：④-A 包感知桥/包片段 lane 缺 `ace-bridge` cfg 门**；980 自身回归，非 982/983/man）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（默认与 feature 构建双绿）** | delivery: auto-ui master **`d188d695`**（ff-only）| worktree lang-980 @ plan-980-dev。

**缺陷与根因（用户实证 + 本会话复核）**：`auto-ui-runtime/Cargo.toml` 的 `ace-bridge` feature 把 `ace-av`/`ace-core`/`ace-a2x-ts` 等声明为 **optional**；`ui/mod.rs` 里 `ace_node_desktop` 由前置 `#[cfg]` 覆盖，但 #2b/#2c 新增的 `ace_package_bridge`、`ace_package_lane` **未加门** ⇒ 任何不激活 `ace-bridge` 的消费者（auto-ui 默认 `cargo check -p auto-ui-runtime`、man 的 `ace-candidate` 链）编译时这些 crate 不存在（E0433），round 176 首错 `ace_a2x_ts` unresolved 同族。

**修复**：`ui/mod.rs` 给两模块补 `#[cfg(feature = "ace-bridge")]`（与 `ace_node_desktop`/`ace_state_bridge`/`ace_ts_provider` 同一门控面）；不改依赖声明、不改桥/lane 代码。

**验证（合并后提交）**：`cargo check -p auto-ui-runtime`（默认 features）**0 error**；`--features ace-bridge` **0 error**；ACE 面回归 `plan980_closure_window` 3/3、`plan980_closure_sfc` 2/2、`ace_node_desktop` 11/11。

**man `ace-candidate` 是否转发 `ace-bridge`——980 决定：不转发。** 理由：桥/lane 依赖 optional 的 UI provider 面（含 iced/`auto-lang-ace-host` 等），man 的 candidate 链只需要 candidate/emit 与时序；转发会把 UI 栈拉进 man 的默认图并放大其 feature 面。若未来 man 某条腿确实需要桥行为，按腿显式启用并具名理由（不在本包预先扩面）。982 可在本提交后复跑 man 链（`plan983_c0_cli` ＋ `plan982_av_http_service`）；本红此前对 C0 的配对 worktree 证据不构成阻塞。

2026-11-18 stage: work+merge（④-A #3：**AGE `view` 树 → Vue SFC 生成 + 真实挂载**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（派生例 0→1→0 真挂载；002 首阻断具名记录）** | delivery: auto-ui master **`166acac0`**（ff-only）| worktree lang-980 @ plan-980-dev。

**交付。** ①`ui/ace_package_sfc.rs::generate_package_sfc(units, entry)`（`ace-bridge` 门内）：模板由入口 unit 的组件面候选逐项渲染——`col/row→div`（flex class）、`button/text/input` 保拼写、`style:`→`class`、`onclick/oninput`→`@click/@input` 指向生成的事件函数、文本叶=字面量或 `{{ 表达式 }}`（模板侧 ref 自动解包、无 `.value`）；脚本=`import { ref } from 'vue'` + model `ref(初值)`（标量 CPU 值优先，其余经同一包片段 lane 发射）+ handler（lane 在状态函数内按原 span 发射 RHS，lane 局部名改写为 ref）+ 被调 helper 定义（lane `emit_function_body`，发射名即调用名）+ 最小 `a2x_rt`（add/sub/mul）+ `defineExpose({ __state })`（组件公开状态，供宿主/验收读取，不是测试钩子）。②真实挂载档 `tests/plan980_package_sfc.rs` + `examples/ui/ace-web-lane/tests/test_package_sfc_mount.mjs`：`@vue/compiler-sfc`（inlineTemplate）编译 → 本地 HTTP 供页面（file:// 下 ESM 跨源被拒）→ playwright 挂载真 Vue 3.5 → 点 `+` 得 1、`Reset` 得 0、再两次 `+` 得 2（经 `__state()` 读真实响应式状态）；页面零 page error。

**验证。** 本文件 3/3（含 env-gated 挂载档 `AUTO_UI_ACE_WEB_E2E=1`）；默认 `cargo check -p auto-ui-runtime` 0 error（#2b 的 cfg 门保持）；回归 `ace_node_desktop` 11/11、`plan980_closure_window` 3/3。

**002 首阻断（具名，按用户要求记录）**：未改写 `examples/ui/002-counter/src/front/app.at` 经**包闭包路径**生成 SFC 时首错=`source.lex: unknown character \`\``（反引号模板文本不在 age-package 词法面）。**诚实归因**：002 是**单文件**工程，组件面（`project_component_candidates`/lane）支持反引号；因此这是"生成器走包面"的路径限制，**未必是 002 的语言缺口**。下一刀应决定 U0 走哪条：**A** 加"单文件 SFC 路径"（组件面描述 + lane，绕开包面词法）；**B** 把反引号文本补进包面词法（981 会签、影响面更大）。在决定前不宣称 002 已支持。

**边界（诚实）。** 模板只覆盖标量 model 与元素/文本叶；f-string/反引号文本、列表/存储/组件引用、`design/theme`、条件/循环、样式配方（style 面）不在本刀；`onclick/oninput` 以外的事件键按普通属性跳过（未具名拒——下一刀补）；`defineExpose` 暴露的是字段值快照（非双向绑定）。

2026-11-18 stage: work+merge（**U0 预验收（④-A #4）：用户裁定 A 路径——未改写 002-counter 经单文件组件路径 SFC 真挂载**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（页面 0→1→0 + 全 lane a2x 证据）** | worktree lang-980 @ plan-980-dev | code_commit: ui **`b3c07aeb`** + 锁 chore **`f6210107`**（lang 侧零改动——纯验收面）| delivery: auto-ui master **`9b0e9683`**（merge；落主树复跑 2/2 含排窗挂载档全判据绿，2.86s）。

**交付。** `tests/plan980_u0_002_sfc.rs` + `examples/ui/ace-web-lane/tests/test_002_sfc_mount.mjs`：**未改写** `examples/ui/002-counter/src/front/app.at`（反引号文本/顶层 `use themes.themes`/design 块原样）经**既有** `generate_sfc_with_lane`（编译器 SFC 管线 + `ComponentFragmentLane` 体拼接）产出完整 SFC → 真实挂载。挂载 rig 三步：`@vue/compiler-sfc`（inlineTemplate）编译 → esbuild 剥 TS 注记（单文件 SFC 为 `lang="ts"`——#3 包路径脚本是纯 JS 无此步）→ a2x_rt 注入（component 实际引用的 `a2x_*` 名 ∩ 官方 `script_runtime_body` 导出面，named import）→ 本地 HTTP（file:// 下 ESM 跨源被拒）→ playwright 真 Vue 点击。

**验证（用户判据逐项）。** ①生成档（CPU 0.18s）：`absent` 空=**9/9 全 lane**（入口对缺席 handler 保编译器旧道体——仅凭 DOM 变动不证明 ACE 覆盖，故 lane 覆盖显式断言）；六个计数 handler（`.Inc/.Dec/.Reset` + `__evt_onclick_{1,2,3}`）拼接体逐行=lane 体（`a2x_add`/`a2x_sub` 形），旧 TS fallback（`+= 1`/`-= 1`）断言不在位；Reset 族拼接体恰=`count.value = 0;`。②挂载档（排窗 `AUTO_UI_ACE_WEB_E2E=1`，8.6–11.5s）：页面文本 `Counter: 0` → `+` 得 **1**（加）→ `-` 得 **0**（减）→ `+,+` 得 2 → `Reset` 得 **0**（重置）→ `+,+` 得 2；零 page error。③回归：`plan980_package_sfc` 3/3、`p980_web_lambda_origin`（002 生成级 9/9 面）双绿；本刀零产品代码改动（纯测试+rig），changed files fmt clean。

**边界（诚实）。** ①单文件路径 SFC 脚本**不含 a2x_rt 定义**（包路径 #3 由生成器内联最小 rt）——宿主/挂载侧注入官方 rt（与 node 驱动同一合同，宿主接线非语言缺口）；②SFC 为 `lang="ts"`，浏览器执行需 TS→JS 转译（rig 用 esbuild 剥类型，语义零改写——真实产品链路的转译归属待 WEB 收口定）；③主题三键（UseAuto/UseSolarized/ToggleMode）生成级 lane 在位（bodies 含其 a2x_str_* 体）但挂载档未点击（U0 判据=计数三键）；④包面反引号仍拒（B 路径留给确需多模块的例，用户裁定）。

2026-11-18 stage: probe（**style 20 例复测：对齐 P3+981 G1/G2 合并后新基线**；临时探针已删净，证据=plan-artifacts/plan-980/style-retest-20261118-{run1,run2,pkg-product}.log）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **数据更新（包面 style 20/20 已过——P3 生效，卡点后移且真因另在）** | 基线 lang **`010e26f08`**（=P3 `e8e4c16da` + 981 G1/G2 `6e456794d` + 982）。

**复测矩阵（原始源双面 + 产品入口对照）。** ①**包面 parse 级**（`parse_package_source_age`）：20/20 的 style 行**全部通过**；首错后移——**16/20** `source.syntax: expected fn, found widget`、**4/20** `source.lex: unknown character $`（029/031i/031p/041——f-string `$` 不在包面词法）。②**包面产品入口对照**（`compile_age_package_with_scenario`+`LegacyWidgetAlias::Accept`，即 `generate_package_sfc` 同入口）：合成对照五例（bare/use/style/style+design 的 `widget` 头**全放行**、components=1）⇒ **"expected fn, found widget" 不是平坦 widget 墙，是 item 级回滚位**；最小 `double` 字段例给出**精确具名拒**（`source.age.component-unsupported: model field has type double…supported: int/i32/bool/str/f32`——D-④-3 宽数值拒绝按设计工作），而整例 003 仍塌缩为头部泛错——真因在例内（design if-else/f-string/`math.*` 按 census 候选，未逐例挖），与 S0.6"回滚位掩盖真因"结论一致并实证。③**组件面**（age-component，P3 按 981 回签收窄不动）：原始源 0/20 直解——9 卡 `expected fn, found style`（003/005/007/008/009/010/012/014/024）、7 卡 `source.unsupported: use is not part of this source profile`（011/015/016/017/018/027/030——与 981"单文件 use 具名拒"合同一致）、4 卡 `source.age.slot: expected node at the declared node slot`（029/031i/031p/041——`use { package }` 块形）。④剥离 style 变体（S0.6 口径）：**003 直解 OK（nodes=1）**与 S0.6 一致；其余下一阻断=text 槽位（005/007/008/009）、model 字段形状（010/012/014）、use 具名拒（7 例）、model/node 槽位（024/029/031i/031p/041）；包面剥离口径 16 widget+4 `$`（S0.6 旧基线为 18+2——基线差异非重测矛盾）。

**结论。** P3 style 在包面已整体消除"style 首阻断"（16 例卡点=回滚位泛错、真因=例内宽数值/词法等既有具名墙；4 例=`$` 词法）；组件面维持 style 拒（981 收窄）——**WEB 单文件路径（lane 走组件面）对含顶层 style 的 20 例仍是第一阻断**；这 20 例的包面路线解锁取决于例内既有墙（宽数值/`$`/design 面）而非 style 本身。U0 预验收（002 无顶层 style）不受影响。

2026-11-18 stage: merge（**F32 消费同窗收口：981×980 配对落主（新基线对齐后）**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（窗口闭合；980 侧组件桥 3/3 落主树复绿）** | delivery: lang master **`d0ed5ef0e`**（merge `f32win-lang`＝981 侧 F32 `caba15b52`＋对齐合并 `40f1f03fe`＋锁 chore `a73ef0549`）＋ auto-ui master **`0a04b086`**（merge `f32win-ui`＝980 侧 `8a713b89`＋对齐合并 `6191915f`）。

**对齐。** 981 G1/G2 落主后双 f32 分支各 merge 双仓 master（零冲突）；lang 侧 f32 worktree 中 `session_bridge.rs` 的 2 行未提交 UsePy 臂已丢弃（与 master `ff55579c3`/982 同内容重复——当时记录的窗口阻塞已由 982 在 master 解除）。

**门禁（窗口冻结门，f32 树 `a73ef0549` 实测）。** lang `cargo t ace-source ace-av`：**ace-source PASS**；ace-av 1408 跑 **1405 绿/3 红**——红项全为台账既定 C 工具链/A2R 外部腿族：`kv_zip_1064`×2（ExecutionWall=并行竞争，机器有他会话 cargo 在场，**隔离串行重放 2/2 绿**）；`sys_d01_d06_1064::v02_d01_d06_a2r_cross_end`（a2r 跨端 spawn 空返回，隔离串行仍红，但 **master 基线同红、主检出（工件热）绿**＝预存冷工件环境族——F32 diff 不触该腿，不判窗口，台账记 owner=工件缓存机制）。**发现并修复 master 级缺陷**：982 `c_transpile` 落主未随行 Cargo.lock（任何 `cargo t --locked` 工具准备即红）——f32win-lang 锁 chore `a73ef0549` 补 2 行（982 后续 merge 已含更全刷新，合并无冲突；主检出同内容脏锁随之转干净）。

**落主树复核。** lang `d0ed5ef0e`：`plan981_f32` 面 **6/6**（0.07s）；auto-ui `0a04b086`：`plan980_f32_component_window` **3/3**（0.27s；Float(1.25)→Step→1.5 bit-exact、Int 种子具名拒、显示 1.5）。

**遗留。** f32 双 worktree（`lang-981-f32` 组）已全并入 master、待退役；包桥 F32 仍另案（本窗合同边界不变）。

2026-11-18 stage: work+merge（**④-A #5：U0 产品出口——正式 Vue 生成链的 TS 与 `a2x_rt` 接线**；用户本轮主刀指令）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（002 产物从产品入口启动页面，零夹具注入）** | delivery: auto-ui master **`c035a458`**（merge `f93e94b9`）| worktree lang-980 @ plan-980-dev。

**交付。** `ace_vue_script.rs::generate_sfc_with_lane_product(src, path) -> LaneSfcProduct{sfc, runtime_mjs, lane_bodies, absent}`（`ace-bridge` 门内）：生成器（非测试夹具）承担 U0 夹具两步——①SFC 显式发射 `import { … } from "./a2x_rt.mjs"`（名字=脚本 `a2x_*` 词法整词 ∩ 官方 rt 导出面（rt 源自扫不复制目录），首现序稳定；零引用时不发射）；②官方 `ace_a2x_ts::script_runtime_body` 原样随产物返回（落盘约定=同目录 `a2x_rt.mjs`）。`lang="ts"` 保持——TS 处理由既有 Vue 构建链承担（`auto run` 链=vite；验收 rig 用同族 esbuild 转换）。absent 仍旧道承载（双轨合同不变）。

**验收（去注入）。** `tests/plan980_u0_002_product_sfc.rs` + `examples/ui/ace-web-lane/tests/test_002_product_mount.mjs`：SFC 与 rt 均**产品返回物原样落盘**，rig 零注入（不扫名/不写 import/不供 rt 内容；唯一改写=裸 `'vue'` 说明符 dev-server 式别名——vite alias 同职责）。①CPU 档：import 名集恰=五个被引官方名、rt 为官方产物（AUTO-GENERATED 头）、lane 证据保持（absent 空、a2x 形、`count.value ±= 1` fallback 不在位）。②挂载档（排窗）：页面从产品产物树启动——`Counter: 0`→`+`→**1**→`-`→**0**→`Reset`→**0**→`+,+`→2、零 page error。落主树复跑双档绿（1.82s 挂载）。

**交接。** 982=CLI 启动（`auto run`（Vue）经产品入口/或 tools 链内调；本函数只管生成与运行时依赖）；983=U0 产品验收（同一份产物）。

2026-11-18 stage: work+merge（**④-A #6：B-WEB 真实语料首刀——原样 Jade Garden `quick_switcher.at` 沿产品链 watch 行为验收 + 单文件生成道模块 fn 重挂 + 16 泛错最小复现材料（交 981）**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（watch 路径 9 判据绿；两具名发现）** | delivery: auto-ui master **`5d7725d9`**（merge `15262697`）| worktree lang-980 @ plan-980-dev | 语料=auto-down jade-garden `quick_switcher.at`（219 行，sha256 `8622184a…`，主检 出==组内 sibling；漂移以行数+锚点+生成级断言侦测，缺场具名跳过）。

**交付。** ①`tests/plan980_bweb_jg_watch.rs` + `test_jg_watch_mount.mjs`：原样源经产品出口生成（0 lane/7 缺席——`use {` TS-ext 块命中组件面单文件 use 具名拒（981 合同），lane 覆盖如实记零、行为走旧道）→ 真浏览器 9 判据：open=false 浮层隐藏；`watch(open)`=query 清空+focusSwitcherInput 被调；过滤 'alph'→2 项；ArrowDown→高亮第 2 项；'alphabe'→1 项且 `watch(filtered)` 把 selected_index 归零（高亮回首项）；零预期外 page error。宿主侧 ext stub（组合式 store/热键回调/hotkey 记录）=rig 夹具（真实链路 `@/ext` 别名同职责）。②单文件生成道补**同文件模块 fn 重挂**（`same_file_module_fns` 复刻 vue.rs 链——裸 VueGenerator 丢文件顶层 fn，jade-garden 实测 `filter_files is not defined` 暴露；PLAN-075 同病同修）。③16 泛错最小复现（981 材料，plan-artifacts/min-repro-families.log + style-retest-*.log）：**structure 族**最小复现=`var n = 0`（缺类型）与 `var ns [int] = []`（列表）→ `source.age.structure`（010/012/014）；**slot 族**=`use { package }` 后接 model → `source.age.slot: expected model`（024）；**double 族**=最小例在包面产品入口给**精确具名拒**（`component-unsupported: …supported: int/i32/bool/str/f32`）但整例（003 等）塌缩为 widget 头泛错——**塌缩条件本身=发现**（回滚位掩盖真因，981 修复时须同时修诊断不塌缩）；text-slot 族（005/007/008/009）合成形不触发——真源 view 形状特异，随附全源诊断 span 交 981；use 族 7 例（011/015/016/017/018/027/030）=981 单文件 use 具名拒合同内，无需复现。④**4 例 `$` 词法单列入账**（029/031i/031p/041）：f-string `$` 不在包面词法（组件面先脱糖不受影响）——包面词法域决策归 981（P3 会签同谱系）。⑤census **具名修订 #9**：(75,8,7,23)→**(75,8,8,22)**——lang 基线（981 G1/G2+P3）前进致组件面 lane 派生面变化，**A/B stash 实证与本刀 ui 改动无关**；落主树（lang 更新基线）24/24 复核成立。

**具名发现（预存缺陷，非本刀引入）。** ①**旧道生成器 `oninput: .Msg($event)` 双重解包**：模板预取 `$event.target.value`+脚本体再 `e.target.value` ⇒ 每击键 TypeError（v-model 掩盖状态面）——**真实部署件同形**（jade-garden front/src/components/QuickSwitcher.vue:144 实证），rig 具名容忍+此处台账；修复归属=生成器 input 面刀（B-WEB 生成器道）。②node Temp host.mjs 竞态族再现 1 次（隔离重放绿，r16 台账同族）。③三 rig 路径归一修复（正斜杠目录参数下 `file.startsWith(dir)` 假阴性 404——cargo 传反斜杠临时目录时掩盖）。

2026-11-18 stage: work+merge（**④-A #7：工具侧读键接线——982 round-184 合同收口（U0 真链最后一环的 980 侧）**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（双道接线+五判据验收）** | delivery: auto-ui master **`4dd2654c`**（merge `aa45ecf4`，纯加法 216+/2−）| worktree lang-980 @ plan-980-dev。

**交付。** ①`auto-ui-tools` feature **`ace-vue`**（非默认——792 交付闭包不含 experimental/ace 树，投影面零扩；需 lane 的工具按 `--features ace-vue` 构建）。②键门 `vue.rs::ace_vue_app_sfc(app_at, content)`：`AUTO_UI_ACE_VUE=1` 时经产品入口 `generate_sfc_with_lane_product` 生成 app SFC；三具名拒码=`ace.vue-lane-unsupported`（absent 非空，附缺席清单——run 键下双轨收紧，不静默落 legacy）/`ace.vue-lane-error`（生成失败）/`ace.vue-lane-unavailable`（工具未按 feature 构建）；键未设=`Ok(None)`，legacy 道逐字不变。③**双道接线**：增量 `incremental_compile_changed` Phase-2（app.at Ok 臂换产物，官方 rt 直接写 `gen/front/vue/src/a2x_rt.mjs`）+ 全量 `from_workspace`（根 widget i==0 臂换产物；`VueProject.ace_vue_rt` 字段携带，`write_ace_vue_rt` 于两个 App.vue 写盘点落同目录 rt——SFC 内 `./a2x_rt.mjs` 相对 import 的解析位）。

**验收。** `tests/plan980_ace_vue_tool_wiring.rs`（`required-features=["ace-vue"]`）五判据单函数顺序执行（env 键进程级不可并行切换）：①键关=002 legacy 道零变化（无 rt import/无 rt 文件）+增量有产出；②键开=增量道 App.vue=lane 产物（`a2x_add(count.value, 1)`+显式 import）+官方 rt 落盘（AUTO-GENERATED 头+a2x_add 导出）；③键开=全量道 `app_vue_code`=lane 产物、rt 随 Self、写盘幂等；④键开=F-1 形参遮蔽缺席源（`.Set(n)->{.n=n}`）**双道**具名拒（unsupported+缺席名）；⑤键关=同一缺席源恢复 legacy Ok。1/1 绿（worktree 1.04s；主树 `4dd2654c` 复跑 1/1，155.5s 含 feature 冷编译）；默认构建 `cargo check -p auto-ui-tools` 0 error。

**纪律（诚实）。** 首版提交含 rustfmt 对 vue.rs **预存漂移**的整体重排（123 行无关删除）——已 reset 重做为纯加法；vue.rs 基线 fmt 漂移为预存（本刀不整文件 fmt，避免无关 churn）。

**预存红具名（A/B stash 双向实证与本刀无关）**：tools lib 测试 delivery 投影族 **8 红**（`delivery_projection_strip_and_idempotent`/`p980_delivery_projection_strips_runtime_ace_bridge`（"producer 清单结构漂移，受管判红"）/bundle_end_to_end/gc×2/sidecar/redelivery/foreign_sibling/shell_pack/p060_receipt）+ display corpus **1 红**（"expected 149, got 123"）——带/不带本刀改动计数逐项一致；并行全量下另有 2 例抖动（plan609/plan842——过滤复跑双态绿）。归属=rust_ui/delivery 投影面与 display reader 面（具名待其 owner/下一刀核；本刀不冒充 tools 全套绿）。

**交接 982（去 ignore 前置已备）。** 工具以 `--features ace-vue` 构建后 `auto run --render vue`（分派键原样）即走产品 lane 道；未建面由 `ace.vue-lane-unavailable` 具名拒守。**边界**：真 CLI `0→1→0`+ACE 覆盖=982 去 ignore 用例本体；dev-server/npm 腿未在本刀验证；多 widget app.at 在本键下会经 splice 锚未命中走具名拒（产品入口返回首 widget SFC——多 widget 形=下一刀，不冒充支持）。

2026-11-18 stage: work+merge（**④-A #8：多 widget 接入产品链**（用户排程：982 跑 U0、980 同时做多 widget）；收口 #7 具名边界"多 widget=下一刀"）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（双道逐件接线+八段验收）** | delivery: auto-ui master **`70e9ec02`**（merge `cbe38068`）| worktree lang-980 @ plan-980-dev。

**交付。** ①runtime `ace_vue_script.rs`：`generate_sfcs_with_lane_product(src, path, rt_href_root, rt_href_component)` → `LaneSfcProductMulti{widgets: Vec<WidgetLaneSfc{name, sfc, bodies, absent}>, runtime_mjs}`——**逐 widget 独立拼接自己的 lane 体**（node 选择器=声明序；此前循环内已逐件拼接但弃掉非首件）；rt 全文件共享一份，**import 按产物位参数化**（根=src/App.vue→`./a2x_rt.mjs`；组件=src/components/{W}.vue→`../a2x_rt.mjs`；零 helper 引用的件正确地不发射 import——NOT 形内联无引用）；单件出口 `generate_sfc_with_lane_product` 改为首件委托（行为不变——回归实证）。②工具双道：键门升级 `ace_vue_app_sfcs`（多件版；缺席具名拒**附 widget 名**：`Panel[.Set]` 形）；**增量 Phase-2** 逐件落盘（根=App.vue（随 changed_files）、其余=src/components/{W}.vue 直接写——与全量道同布局；`app.at (ace lane: App + N component widgets)` 可见行）；**全量道** widget 循环前一次性多件生成 + `pick_ace` 声明序对位（对位失败=`ace.vue-lane-mismatch` 具名拒，不静默回落）。

**验收（`plan980_ace_vue_tool_wiring` 扩至八段，单函数顺序（env 键进程级））。** ①–⑤=#7 原五段（键关零变化/单件增量+全量/缺席拒/键关恢复）保持；⑥多件增量：根件含自己的 `a2x_add(count.value, 1)`+`./a2x_rt.mjs` 且**不含**第二件 handler；`src/components/Panel.vue` 含 `(!flag.value)`+`a2x_add(steps.value, 1)`+`../a2x_rt.mjs` 且不含根件 handler（**每 widget 的 handler 落在对应产物**判据）；⑦多件全量：root/app 项与 Panel 组件项均为产品件（组件项含组件位 href）；⑧多件缺席：具名拒携带 widget 名与 handler 名。1/1 绿（worktree 1.76s；主树 `70e9ec02` 复跑 1/1，207.9s 含冷编译）。

**回归。** 委托重构行为保持：`ace_ts_provider` **24/24**、U0 产品 **2/2**、jg watch **2/2**（含排窗浏览器档）；默认 `cargo check -p auto-ui-tools` 0 error；#7 预存红清单不变（delivery 投影族 8+corpus 1，A/B 已具名）。

**边界（诚实）。** ①逐件=**同文件多 widget**；跨文件组件（pages/、components/ 包、dep fronts）的产品 lane 道未接（仍 legacy——键只作用于 app.at 的 widget 集）；②多 widget 下的 store/子组件映射面（sub_widget maps）在产品件上不参与（lane 件由源直生——002/Panel 族形状已验；复杂互引形=真语料批次）；③dev-server/npm 腿仍归 982 真链。

2026-11-18 stage: work+merge（**④-A #9：跨文件组件走 ACE（components/ 通道双道）+ AutoNotes `FpNoteEditor` 输入链基线 + 原样 010 短回归**；用户排程第 1/2 项，选型更新=AutoNotes 为 980 首个产品消费者）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（通道接线+两例验收；两验收对象的 ACE 翻转具名阻塞于 981 域墙）** | delivery: auto-ui master **`01f85061`**（merge `951ca332`）| worktree lang-980 @ plan-980-dev。

**交付。** ①键门泛化 `ace_vue_sfcs(path, content, href_first, href_rest)`（缺席拒附**文件名**+widget 名）+ `ace_vue_component_sfcs`（components 件 href 恒 `../a2x_rt.mjs`）。②**components/ 通道双道**：增量 Phase 1b 逐 widget 产品件落 `src/components/{W}.vue`（各件自己的 lane 体；官方 rt 落 src/ 与 app 通道共享；per-widget code——ACE 侧顺带修掉 legacy 同码多写怪癖）+ 全量道 components 分支按名对位（`ace.vue-lane-mismatch` 具名拒）。③验收 `plan980_ace_vue_tool_wiring` 扩至**十段**（⑨components 正控=fp_editor 派生 lane-ready 件双道产物判据；⑩components 缺席负控=拒附文件名+widget 名）1/1 绿。

**AutoNotes FpNoteEditor 输入链基线**（`plan980_notes_fp_editor` + `test_notes_fp_editor.mjs`，排窗）：**只读消费原样链路现成产品产物**（auto-notes `gen/…/FpNoteEditor.vue`——真仓 gen 树，勿动他树；产物锚点钉 emit 契约）；宿主=methods 绑定（内联箭头 `$event` 包装会吞载荷——实测教训）；Button/lucide 具名 stub（被测链不经）。判据 4/4 绿：DraftEdit 载荷**逐字**（`你好 FP draft`）、随输入更新、SubmitText prop 往返（`seed-body`）、零 page error。**ACE 翻转具名阻塞**：原样 `fp_editor.at` 卡组件面 **view if-else**（`source.age.slot: expected text`——最小复现 `if cond { text "A" } else { text "B" }`，981 域 view 文法；裸 if 过、带 else 拒）；981 落面后 components/ 通道即翻转（rig 复用）。

**原样 010 短回归**（`plan980_010_form_short` 单函数三段）：①键开=**具名拒实证**（0/6 全量缺席清单——`.Submit/.SetTheme/.SetAccent`+3 输入绑定；fail-closed 合同可观察行为）；②键关=legacy 产物含三 handler（主题切换产物面证据——宿主通道无页面内触发件）；③排窗浏览器三判据绿（回执隐藏→三字段输入绑定→Send→回执可见、零 page error；`@/components/ui/*` 通用 stub=Button/Input/Textarea 原生元素+v-model 透传）。**ACE 断言今日不可能**（用户口径据实修正）：010 卡两族 981 域墙——`var faqs = [...]` 无类型列表字段（文件级 lane 编译拒）+ `.SetTheme` if-else 体（控制流；G2 只落 age-package 面）。

**纪律与边界（诚实）。** ①auto-ui 主树复放暂阻：lang 主检出有他会话**未提交半截 WIP**（ace-core `project_packages_front` 签名 6/5 不匹配——packages/frontdoor/binding/av 五文件；本刀 worktree 侧全套绿+落主内容=分支树，主树重放待该 WIP 收口）；②`ace_ts_provider` 23/24（1 红=node Temp 竞态族，隔离重放绿，台账在案）；③AutoNotes 全 app 的 ACE 化仍远（app.at 20 缺席、notes_widget 3 缺席、editor/sidebar 单文件解析卡 sibling 模块导入——跨文件模块解析属④域合同谱系）；④010 主题切换的浏览器内触发件不存在（宿主通道）——以产物面+键下拒清单为证。

2026-11-18 stage: verify（**U0 联合验收收据**，982 代记）| plan_id: PLAN-980 × PLAN-982 | 对签对象：④-A #7/#8 交接面（`AUTO_UI_ACE_VUE` 工具读键＋产品出口＋多 widget）| **闭合**：982 round-188 以 **70e9ec02 配对工具**（auto-ui 主检出该 commit 树上 `cargo build -p auto-ui-tools --features ace-vue`，即当时盘上 exe）跑通**原样 002-counter 真 CLI 链**——真 CLI `auto run --render vue`（候选腿，`AUTO_UI_ACE_VUE=1` 发射可见）→ pnpm（锁文件校验 442ms）→ vite v5.4.21（ready 722ms）→ 真浏览器 `Counter: 0→+→1→-→0`（`+,+`→2、零 page error）；磁盘断言 lane `App.vue`（显式 `from "./a2x_rt.mjs"`＋`a2x_add(count.value, 1)`）＋官方 `a2x_rt.mjs`（AUTO-GENERATED 头，body sha256 `97065d43…`）| 证据：man **`4a7a28e`**（`plan982_u0_vue_run_cli` **3/3**，master 实跑 100.66s 含 bin 构建；`#[ignore]` 已除，`u0_vue_real_chain.mjs` rig 随库）＋lang **`8e94acfaf`**（982 §9 round-188 全文收据）| **解除的 980 具名等待**：#7 交接注记"dev-server/npm 腿未在本刀验证"——真链已验（pnpm+vite+浏览器全链）| 边界保持：983 U0 产品验收消费同一产物树/同一链；工具 feature 非默认（792 交付投影约束，缺失＝具名失败带构建指引）

2026-11-18 stage: work（**开放项并行推进：#9 主树复放闭 + AutoNotes-ACE 墙复探与 view-if 刀材料完备**；与 U0 联合收据同窗并发落档）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **开放项 1 闭；开放项 2 材料升级（墙范围修正：bare if 同拒）；开放项 3 未动**。

**①#9 主树复放（闭）**：他会话 lang 主检出半截 WIP 已收口（ace-core 主检出编译恢复 0 error）——auto-ui 主树三面复跑全绿：`plan980_ace_vue_tool_wiring`（十段，177.4s 含冷编译）、`plan980_010_form_short`（三段含排窗浏览器，50.0s）、`plan980_notes_fp_editor`（1.0s）。

**②AutoNotes-ACE 墙复探（基线 master `600b7ee07`，worktree 双仓同步后实测）**：墙**未动**——fp_editor `.DraftEdit` 仍缺席、010 仍 0/6、view if-else 最小复现仍 `source.age.slot` 拒。**墙范围修正（重要）**：bare `if`（无 else）在 view 同拒——组件面 **view 级条件语句整族缺席**（`age-view-item` 无 if 臂；`age-if-expr` 仅表达式位）。影响面重估：quick_switcher 的 `if .open`、005/007/008/009 text-slot 族、fp_editor mode 分支——**一族墙覆盖四例语料**。

**view-if 刀提案（下一刀，981 会签纪律——#6m/#6o 先例）**：①文法：`age-view-item` 增 `age-view-if` 臂（`if <expr> { many view-item } [else …]`，else-if 链）；②投影：`ComponentViewItem` 增条件变体（then/else 子树+span）；③**消费者爆炸面（已查）**：lang 6 文件（frontdoor/age_component/cst-project/age_package/field_manifest+测试）+ UI 3 文件（ace_native/ace_node_desktop/ace_package_sfc）——WEB lane 侧 states-only 不遍历 view 项（模板走方言管线 v-if）；桌面/AV 渲染腿须各自定条件语义或具名拒；④grammar-id 两值重基线+13 邻居冻结表（#6m 同款）；⑤010 另两墙（无类型列表字段/on 体 if-else 组件面）独立跟进。

**③B-WEB-I/V/R 收口**：本轮未动（等 view-if 刀+④域材料）；U0 已由 982 round-188 联合验收双侧闭（上收据）——WEB 批与 U0 解耦明确。

2026-11-18 stage: work（**B-AV-R 签发确认（用户指令）：接收 981 W1 分组 AV 交接并入账**）| plan_id: PLAN-980 | plan_revision: r18 | 交接绑定 `3047b3481`（已在 master）| 复放基线=**master `81221364d`**（981 轮更后；证据在更新树上复验更绿）。

**接收（复放全绿）。** ①**G-INSTANCE 双实例 2:1**：`plan981_event_b_av`（两实例交错→[7,5]）+`plan981_cross_module_nodes`（**两真实模块** ForNode 2:1、包序随动、越界报包内组件数——含 981 跨模块身份修复的消费面）；②**G-LIFECYCLE 关开**：`plan981_gfx_host`（stop→session-stopped/丢弃/重注册）+`plan981_gfx_hooks`（epoch 归零/账本重置）；③**G-VIEW/FRAME/INPUT**：`plan979_*`+`plan980_*` AV 族（收据 75/75 @3047b3481；本复放按前缀**26+18=44/0 红**，另 `plan981_` 前缀**46/46** 含其新组——族超集全绿）；④**G-BADEDIT 源侧**：core `recovers_on_the_same_entry`/`recovers_after_the_import_is_added` 各 1/1 绿。复放命令与计数落 plan-artifacts（bavr-*.log）。

**UI-W1-AV 签发剩余（具名，故 B-AV-R 行保持未勾）。** ①**G-RESOURCE 无出据**（`leaked_regions_cleaned`/`live_mapped_at_exit` 归会话/AVHC 腿 owner——981 明确不交）；②**U01..06 全分母**仍压 ④外部名域合同（35 例 census 冻结在案：34/35 含 use、宽数值 2 例）——分组绿≠分母闭；③**G-BADEDIT 会话侧**（失败不动既有 ops）待 auto-ui **R64-1** 正式来源。

**同时入账（981 附带交付）。** 跨模块实例身份修复（包级组件声明序三处统一）——UI058 可把 Target/Player 拆回两真实模块；typed 队列可直接复用（作者内自派发按 host authority 不新增）。

2026-11-19 stage: work+merge（**view-if 刀：组件/包面 view 级条件语句整族落地**（用户指令；981 会签纪律——id 重基线随刀，先例 #6m/P3）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（fp_editor 原样翻转达成）** | delivery: lang master **`f169cf17c`**（merge `d69f29f91`，166+/4−）+ auto-ui master **`ae41e304`**（`b6d54261` 修正版，109+/0−）。

**文法+投影。** ①`age-view-item` 增 `age-view-if/if-then/else` 三规则（两 AGE 面同开；else-if 链=age-view-else 双 alt（各含 else-kw 序列——规避 mixed-children DSL 约束）；then/else 体走子规则 field 定位（防 collect 包装下钻误并））。②`ComponentViewItem::Cond(ComponentViewCond{cond, then_items, else_items, span})` 结构承载（不判真值）；文本叶随 collect 平铺进 texts（绑定验证同覆盖）；五处结构遍历进双子树（lambda 收集（铸造序=源序）/绑定验证/style 替换/字段 manifest/事件绑定收集）。③`view_contains_cond` 助手=渲染腿 fail-closed 判据。

**消费者九文件处置。** lang 6：age_component（枚举+助手+两遍历臂）、cst/project（投影+递归助手——else 臂按 kind 定位具体臂）、cst/frontdoor（绑定验证含 cond 表达式+style 替换遍历）、age_package（计数/事件绑定两臂）、field_manifest（遍历臂）、conformance（注册）；UI 3：**ace_package_sfc=模板腿真语义**（`<template v-if/v-else>`，else-if=嵌套）、**ace_native+ace_node_desktop 双装载点具名拒**（`ace.native/desktop.view-cond-unsupported`——不静默展开/双计）。

**重基线（981 台账同步项）。** grammar-id 两值：age-component `8292feae…`→**`620a0f73…`**、age-package `c0e635d2…`→**`30834de6…`**；13 邻居逐行冻结保持（grammar_scope 4/4 含全表断言）。

**永久用例与验收。** core `plan980_view_if` **4**（四形形状/bare-else 与缺块负控/文本平铺/深嵌可判）；UI pkg-sfc v-if 渲染判据（**4/4**）+ wiring **⑨c：原样 AutoNotes fp_editor.at 经 components 通道产出 lane 组件件**（DraftEdit 体+`../a2x_rt.mjs`）。**语料翻转实测**：fp_editor `.DraftEdit` 全派生（缺席 []）、007 handler 面清（bodies=0/absent=[]）；010（无类型列表字段+on 体 if-else）、quick_switcher（use 块）、005（另墙）不变——各归其既名墙。

**门禁（冻结门实测，worktree 树 `d69f29f91`）。** `cargo t ace-source ace-av`：**ace-source PASS**；ace-av **1411/1414**——3 红全具名：`kv_zip_1064`（ExecutionWall 族，**隔离重放绿**）+ `host_console` ×2（**A/B stash 双向实证预存**——982 console 谱系 owner；`entry_selection_cannot_bypass_main_policy` 与 `owned_front_door_enforces_console_entry_only` 在基线同红）。回归：UI U0 产品/notes fp/jg watch/U0 sfc **7/7**、ace_ts_provider 23/24（node Temp 竞态族隔离重放绿，台账）、**census 表稳定**（合成语料族无 view-if）；主树复验 view_if 4/4+grammar_scope 4/4+wiring 1/1（211.7s 含冷编译）。

**边界（诚实）。** ①本刀=parse+投影+结构遍历+模板腿渲染；**native/desktop 渲染腿条件语义未实现**（装载点具名拒——后续刀）；②010 的 view `style: if..else` 表达式位（age-if-expr）早可用（002 design 先例），本刀不动；③Cond 的 cond 表达式入绑定验证但**不入字段 manifest 出现表**（无 view_cond 槽——具名边界）；④981 台账旧值 `b0ae9d90/ecc64b8e`（其 936 行）与 master 实况的 reconcile 归 981 下轮。

2026-11-19 stage: survey+experiment（**剩余开口逐项勘明（含一次门开实验）；无落地代码**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **工单完备；两开口路由决策**。

**①010 墙①（`var faqs = [...]` 无类型列表字段）→ 路由 collections/864 域决策。** 组件面 `field_type` 值域=int/bool/str/f32（age_component.rs:2918）；TypeKind **有** `List`（864 谱系）但组件场景 state 值域不承载列表——修法=场景 state 值域扩展（属 collections 谱系的所有权决策，非 980 单独切片）。010 的 handler 均不触 faqs（纯 view 数据）——"view-only 字段不进 state 槽"亦可作替代设计（同样域决策）。

**②010 墙②+005 墙=同一刀：on-if 组件面完整链（全链勘明+实验证伪捷径）。** 005 精确根因=**lambda 体控制流**（`oninput: () => { if … }`——与 010 `.SetTheme` on 体 if 同族）。五环工单：**a**文法三门放宽（age-on-stmt 组件 alt+age-on-if+age-on-else——本会话实验开过、编译通过、已回退保 id 稳定）；**b**投影：widget 形 handler 现硬编码赋值集（project.rs ~6165，G2 注释明示）——改接共享 `project_on_stmts`（7106，If 臂 7156 在）；**c**降版 `hook_stmts_lower`→`Stmt::If`（2296）共享已备；**d**发射 a2x-ts `Stmt::If`（lower 412）+`emit_stmt_fragment`（pub）已备——包面消费先例=`PackageFragmentLane::emit_function_body`（ace_package_lane.rs 150）；**e**lane `lane_handler_bodies_spec` 加结构化支（检查函数语句发射+宿主映射过语句文本）。**实验结论：仅开文法门（a）→ 全部形状仍缺席**（投影平铺→铸造不铸）——五环缺一不可，无捷径。落成翻转：010 `.SetTheme/.SetAccent`（`.Submit` 平铺本可派）、005 lambda-if 族（`__age_vl_*` 值函数同治）；+id 第三次重基线+13 邻居表+永久用例。预估=view-if 同量级完整刀。

**③quick_switcher use 块**：981 合同锁（"age-component 单文件 use 具名拒"）——**路由决策非切片**（解锁=④域合同或 981 改约）。**④④域合同**（U01..06 全分母+WEB 共同前置）：用户/976 后继决策，census/最小复现材料齐。**⑤native/desktop 条件渲染语义**：静态形渲染器对条件 UI 的 oracle 语义（双臂并集？运行时估值？）=设计决策——现状装载点具名拒（诚实），决策后实现。

2026-11-19 stage: work+merge（**包版 SFC 条件渲染两边界收口（静态核对发现）+ 顺手修复预存 l1 映射缺陷**）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（单元 4+真 Vue 分支切换 8 判据全绿）** | delivery: auto-ui master **`d856529a`**。

**两边界（用户静态核对）。** ①条件经 `escape_attr`（含双引号比较表达式→`&quot;`——原裸插双引号属性即坏 HTML）；②有 else 时**保留前置 v-if 锚**（空 then 发空 `<template v-if>` 模板作锚——原跳过 then 即产孤立 v-else=Vue 编译错）；双空臂（无 else 且 then 空）整体跳过。

**顺手修复（预存缺陷，双字段源实测暴露）。** `rewrite_lane_locals` 硬编码 `l0___age_s_0_` 前缀——**第二字段起（l1/l2…）漏改**：`a2x_add(l1___age_s_0_n, 1)` 裸名进产物=运行时 ReferenceError（#3 单字段计数器从未暴露）。泛化为任意局部索引+实例段（字段名=剥局部段与实例段后的尾段）。

**验收。** ①crate 内单元 4：转义（`(mode === &quot;dark&quot;)`）/空 v-if 锚精确文本/双空跳过/**l1 泛化回归**（`l1/l2…` 与 `l0` 同改）；②`test_pkg_cond_mount.mjs` 真 Vue 编译+分支切换 **8 判据**：空分支收放×3（`flag!==true` 方向按真语义——空 then 在 cond 真时呈现、else 臂在假时呈现）、else-if 四态（n=0→many→one→two→many）、零 page error；③CPU 集成（空锚邻接+嵌套）与回归（package_sfc 6/6、U0 产品/jg watch/ace_ts_provider 全绿；主树复验 6/6）。

**具名边界（诚实）。** 包面 **checker 条件值域**：str 字面量比较（`Str vs str` 严格性）与 props 读进 view 条件（`self` 投影未定义）两形到不了渲染层——含引号条件的转义判据因此由**单元级直测渲染机械**承载（手造 Expr）；集成翻转待该值域墙（另一刀或 checker 放宽）。

2026-11-19 stage: work+merge（**on-if 刀：组件面 hook-local `if` 全链**（用户指令整刀；五环齐——a/b 实施于 980，c/d 复用 G2 共享层，e=lane 结构化收集）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（SetTheme 形全派生；text 族负控不破）** | delivery: lang master **`9468fdcf6`**（merge `110b877bc`+并集解冲突+`plan980_view_if` 注册去重）+ auto-ui master **`56725cd5`**。

**五环。** ①文法：`age-view-stmt` 增 `age-view-if-stmt`/`age-view-if-else` **镜像规则**（体=view-stmt 方言——`age-on-if` 是 979 钩子形的无点方言不通用，实测踩坑）；②投影：widget 形 handler **结构化语句面**（序保持 If/Assign；assigns 保持深平铺兼容面；分支内调用形→body_foreign fail-closed）；③④降版/发射=G2 共享层直用（hook_stmts_lower→Stmt::If；a2x-ts If 发射）；⑤lane：`stmt_fragments_in` **根块结构化收集**（If 整语句发射——修复扁平块迭代把分支赋值裸喷成无条件序列的缺陷）。

**Str==str 相等性放行（AGE 旗控）。** `t == "light"`（参数 owned vs 字面量借用）原被双层拒。放行以 **AGE 专用入口对**承载：typecheck `age_eq` 旗穿线（typecheck_with→skip→block→expr）+ verify `age_eq` 字段 + `compile_program_parts_age`/`verify_text_record_age` 变体（age_component/domain.rs 各调用点切换）；**text 族入口不变**——774 合同负控 `av_owned_mixed_eq_types_rejected` 实测仍拒。

**G2 负控合同修订（981 台账同步项）。** `p981_hook_let_and_if_stay_syntax_refused_on_the_component_face`——G2 拒 if 臂的理由是"不开能解析跑不了的半面"；本刀交付全链后拒因消除，负控修订为：**if 臂开（全链在案）、let 臂仍拒**（组件面无 let 消费）。

**重基线（第三次）**：age-component `620a0f73…`→**`e3859aed…`**、age-package `30834de6…`→**`7497650a…`**；13 邻居逐行保持。

**验收与门禁。** core `plan980_on_if` **4**（结构化投影/else-if 链/分支内调用 foreign/Str==str 编译放行）+grammar_scope 4/4+G2 修订控 2/2；UI `p980_on_if_param_handler_if_body_derives_structured`（a2x_str_eq 条件+if 分支+宿主映射）；实测 **SetTheme 形 bodies=1/absent=[]**（010 语料形状）。冻结门：ace-av **1412/1414**（仅 host_console×2=预存 A/B 实证）；ace-source 首跑红=G2 负控（合同冲突预期内，已按新合同修订后绿）；ace_ts_provider **25/25**、census 稳定、UI 五套 13/13、主树复验全绿。

**边界（诚实）。** ①**010 整例翻转仍压 faqs 无类型列表字段**（collections 域——SetTheme/SetAccent 形已就绪）；②**005 lambda-if**：文法臂已开（view-stmt 共用）但 lambda 投影仍 assigns-only+foreign——值函数铸造需结构化（下一小刀→**r19 已立 PHASE-LAMBDA-IF 承接，见 §5/§8**）；③分支内调用形=body_foreign（通道调用承载归 #6p 谱系后续）。

2026-11-19 stage: new（**PHASE-LAMBDA-IF 规划落档**；r18 边界②「下一小刀」按用户裁定入本包 phase，不另立新计划）| plan_id: PLAN-980 | plan_revision: r19 | outcome: pass（阶段合同就绪待实施；§5 设计/§6 V-LAMBDA-IF/§7 AC-LIF-01..07/§8 T-LIF-1..7）| 取材：§4 本轮授权小节（全 master 实测 file:line；五环 a/c/d 零改动与 e 已交实证、lane 零改动列为假设待 T-LIF-4）| 簿记：曾取号 PLAN-1071 同日撤销（`a66e8daae` reset，`.next-id` 恢复 1071，无新计划文件落库）| 本轮零代码/零建树/零运行/零合入 | next: work（T-LIF-1 起，唯一树 lang-980 @ plan-980-dev）。

2026-11-19 stage: work+merge（**lambda-if 刀交付：view 内联 lambda 值函数铸造结构化**——T-LIF-1..6 全交；与同日拆分裁定并行：合同已划归 1072，本收据为 980 名下的交付**历史事实**，残余义务随 §0 表移交 1072）| plan_id: PLAN-980 | plan_revision: r19 | outcome: **pass（AC-LIF-01..06 闭；UI 侧零产品代码改动实证；负控合同不变；grammar-id 零漂移）** | delivery: lang master **`347540437`**（rebase 一跳后 ff-only，含 (Plan 980) 标记）+ auto-ui master **`05ed07c3`**（同）+ 规范沉淀 `m2-autoui.md` §13.12（SD-LAMBDA-IF）| worktree lang-980 @ plan-980-dev（唯一树保持）。

**交付（三环）。** ①投影：`ComponentEntryValue::Lambda` 增 `stmts` 结构化面（`project.rs` 接共享 `project_view_stmt_children`——if 形进 `ComponentStmt::If` 序保持、调用形置 foreign=#6p 合同不变；`assigns` 深平铺兼容面由 `flatten_stmt_assigns` 生成，既有消费者默认路径不变）。②铸造：`lambda_value_fn` 语句面降版——Assign 保持既有 `rename_expr` 路径（发射文本不变）、If 递归 `Stmt::If`（分支独立——view-stmt 无 let=无 scope；Let 文法不可达=防御性不铸 fail-closed）。③验证面深走查：`validate_channel_value_use`+frontdoor `collect_item_exprs` 覆盖 If cond+分支 RHS（`walk_stmt_exprs` 提升模块级，handler/lambda 两面共享）。**零文法改动**（grammar-id 零漂移——ace-source 冻结面 PASS）。UI 侧**零产品代码改动**（T-LIF-4 假设变结论：`pair_lambda_regions` 配对/`lambda_fn_covering` 区间发现/`stmt_fragments_in` If 整语句发射全链复用 on-if e 环机械）。

**005 对账（诚实）。** str `!=` 条件最小形铸造绿（`sf___age_vl_0`）；**剥离墙片段 3/3 派生**（`__evt_oninput_{1,2}`+`.Submit`，absent=[]）——lambda-if 族机制面翻转实证；005 整例仍全缺席=**既名墙连带**（`use themes` 单文件拒[981 合同]+`style` 声明[P3 谱系]+design if-else——lane 整源编译失败即 fail-closed 全体缺席），各归其谱系非本刀缺口。census v2 具名修订 **#10**（lambda-if=1 行，auto-ui 侧）。探针摘要 `plan-artifacts/plan-980/lambda-if-005-probe-20261119.md`。

**验证（冻结提交实测）。** 新永久用例 11：core `plan980_lambda_if` **5**（0.03s）+ av `plan980_lambda_if` **4**（0.07s）+ UI `p980_lambda_if_lane`（0.21s：区间发现+If 整语句片段+`a2x_add(n.value,1)` 判别器+缺席负控）+ 浏览器排窗档 `p980_lambda_if_browser_mount`（**1.52s**，toggle 两分支真实走到）——全部 ≤10s。**红相证据**：stash 基线（藏三库文件）跑 av 铸造判别器=红（left:0/right:1——旧面不铸）。冻结门：`cargo t ace-source` **PASS**；`cargo t ace-av` **1422/1425**——host_console×2=预存（stash 基线同红实证，r18 收据同名对）、`v02_d06_spawn_with_output_av`=隔离重放绿（环境归因：门禁并发 spawn 族）——本刀触面零红。UI：ace_ts_provider 串行 **27/27**（5.74s；并行态 `p980_web_expr_fragment_binds` 偶发红=预存 pid-temp 竞态——基线全量同红/隔离绿/串行绿三证）；`plan980_package_sfc` **6/6**；冻结 census 聚合稳定。主树复验：core 5/5+av 4/4（ui 见 1072 侧执行记录）。fmt：本刀 changed files 零分叉（lang 仓 shim-metadata/a2r-actor-tests、ui 仓 auto-cosmic 与 ace_ts_provider:1995 分叉均预存）。

**边界（诚实，残余归 1072）。** ①lambda 赋值 owned-transfer wrap 保持既有语义（与 handler 面 `hook_stmts_lower` 差异=具名边界原 §10-1；str 字段分支赋值经浏览器档实测行为正确——后续统一归 1072 决策）；②分支内调用形=foreign 缺席（通道调用承载归 #6p 谱系后续=1072）；③005 整例翻转仍压 use/style/design 三墙（谱系不变）。

2026-11-19 stage: work+merge（**F-R21-2 收口：ace_package_sfc 脚本重放落 `handler_stmts` 消费门**——981 r21 复审 P1 finding 指派 owner 980/UI；选项 A 具名拒落地，非重划窗口）| plan_id: PLAN-980 | plan_revision: r19 | outcome: **pass（门=具名拒 `source.age.sfc-handler-stmt`；981 T-BASE-ONIF 可复核关闭）** | delivery: auto-ui master **`7b8456e5`**（ff-only）。

**门语义（遵 lang 侧合同）。** `age_component.rs` `handler_stmts` 契约明文：TS/WEB 复放车道只复放赋值，「消费方见 Let/If 时必须具名拒（部分复放=错状态值，禁止）」——`generate_package_sfc` 此前只消费平铺 `handler_assigns`（auto-ui 对 `handler_stmts` 零消费=缺口本体，r21 实读 ：52-98 双支裸喷路径）。落门：入口处逐 handler 查 `handler_stmts`，含 If/Let（If 本身即控制流含嵌套；Let=合同防御臂）→ 具名拒（错误名 handler+语句类）；既有 assign-only 包行为不变（6 既有例全绿）。

**验证。** 永久负例 `p980_package_sfc_if_handler_named_reject`（包面 widget 形 if 体——`age-view-if-stmt` 包面文法可达即 r21 所指暴露面；0.05s）；**红相证据**=stash 藏门文件跑负例红（expect_err on Ok——旧路径双支裸喷生成成功）；`plan980_package_sfc` **7/7**（worktree+主树复验 0.10s 双绿）；fmt 零分叉。**let 边界归档（r21 附带「UI 侧 Let 永久例」处置）**：widget 形 on 块 `let` 在文法层即拒（G2 修订合同——`age-on-let` 是包面**钩子形** `on Life.Tick` 体专属，钩子形不经本 SFC 车道；既有负控 `p981_hook_let_and_if_stay_syntax_refused_on_the_component_face`）——UI 侧 let 门臂不可达=合同防御，非可测面。r21 另提 `ace_ts_provider.rs` `n_assigns` 平铺计数=漂移面仅 census 计数非执行语义（`needs_fn` 判定保守正确），非 fold 阻断项，归 1072 记账。

**双树验收（2026-11-19 用户指令验收；收据已交 981 §9 handoff）。** ①主线（lang master `391ffa54e` / auto-ui master `7b8456e5`）：`plan980_package_sfc` **7/7**（0.07s）——普通赋值生成+if/else 具名拒+回归全绿；②981 并集树（lang `286ed0cb4`，含 lambda-if `flatten_stmt_assigns`→`flat_handler_assigns` 改道）：同套 **7/7**（0.07s；临时组 verify-981union={lang@286ed0cb4, ui@7b8456e5} 组内兄弟解析，四树 wt-guard clean 后整组移除）；③**新放开 let 路径（并集树确认）**：widget 形 `let` 体命中门具名拒 `source.age.sfc-handler-stmt: … contains a 'let' statement …`（验收探针用后即删；主线同源=文法层 `source.age.structure` 拒——「门 Let 臂主线不可达」归档由双树对照证实）。AutoUI master 合入：`7b8456e5`=master tip（ff-only，ancestor 核验 YES）。完整包版控制流发射（真复放）归 **1072**。

2026-11-19 stage: work（簿记补记，零代码）| plan_id: PLAN-980 | plan_revision: r19 | **T-HELLO-D 补记完成**（用户验收裁定：首个 Iced 窗口已落主、无需重实现）——落主收据=本节 HELLO phase fold #4（lang master `1991021a5` + auto-ui master `e529dde6`）；双 SHA 祖先链=用户核验+本会话 `merge-base --is-ancestor` 复核双 YES；§8 勾选+13/32 计数对齐（frontmatter current_step=13 先行一致）。拆分快照「原状态」节按历史原样保留不改。

2026-11-19 stage: work+merge（**F-R21-2 门 Let 臂永久例（r21 附带「UI 侧 Let 永久例」收口）**——981 fold 落主解锁）| plan_id: PLAN-980 | plan_revision: r19 | outcome: pass | delivery: auto-ui master **`d2e7d7a4`**（ff-only）。**前置变化**：981 r21 fold 已落 lang master（`33895d91c`/收据 `c84f25cf4`；`7fc15205a`=980→981 交接落账闭合）——`age-on-widget-stmt` 两面同开后 widget 形 `let` 体主线可解析并到达消费门，本刀早前「门 Let 臂主线不可达」历史归档收口。**交付**：永久例 `p980_package_sfc_let_handler_named_reject`（具名拒含 `let` 类+断言**非** `source.age.structure` 文法拒；=并集树验收探针的永久化）。**验证**：`plan980_package_sfc` **8/8**（worktree 0.08s + 主树复验 0.09s，post-fold lang `dce7492dd`）；ace_ts_provider 串行 **27/27**（3.07s——fold 对 UI 消费面零回归）；fmt 零分叉。完整包版控制流真复放仍归 1072。

## 10. 待澄清事项
## 10. 待澄清事项
## 10. 待澄清事项
Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

本次已明确缺口生产归属及无环交接，不再等待已归档960产生新交付。原I任务仍须冻结具体组合profile/API、诊断码与可见性语法/命令及资源边界；这是有界技术决定，不是新增立项或降低原验收。原已具名的公共语言/平台/产品策略未决继续保留，必要能力缺失必须绑定实际owner，不能以编号泛称取代可执行前置。

当前恢复边界以§5 EVENT-SPLIT为准：round39/40的等待选择留§9历史，不再是“三条件全部齐”的总门。B-AV取M2A-08＋本组实际接口；WEB取M2A-08＋981三个事件语义件/976；RUST W3取UI-W2-AC。#6i具名msg/带参语法已交，typed语义余量有本轮具体任务承接；本次不恢复自动目标。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
本次明确完整传统UI而非无限新DSL：已有语法/组件/模式承诺不能删，源/实现冲突保持具名待闭；未承诺平台不冒已支持，必需格缺环境不记PASS。新的完整Node语法/五view/Frame实证由981负责，980不能以该分工放弃原UI语义或复制第二解析器。

### PHASE-④-EXT 决策请求（2026-11-18；owner=用户/976 后继）

以下 7 项为 §5 PHASE-④-EXT 草案的拍板输入。未裁定前 980 侧除 U01 余量/生成器道外无新开工位；裁定结果转切片时原任务行/AC 不变（切片仍经 §9 收据+phase fold）。

| ID | 事项 | 选项 | 建议 | owner |
|---|---|---|---|---|
| D-④-1 | ④ 先行路径 | A（N2 闭包）/B（foreign 优先）/C（host 表）/组合 | **A 先行**（不动信任面，直接吃 17 例引用分母） | 用户 |
| D-④-2 | `use <mod>: syms` 是否对 age-component 开面（grammar-id 两值具名重基线，跨 981 冻结面） | 开/不开 | 开（作者面统一；与 ④-A 同批） | 用户（981 grammar owner 会签） |
| D-④-3 | 宽数值 model 字段（003 `double`×2、031i `float`×3） | 981 接/组件面映射/保持拒 | 保持拒（独立 981 决策，不入 ④-A） | 用户/981 |
| D-④-4 | N3 foreign 是否要 AV 调用入口（新 967/968 ABI 格+授权） | 要/不要/后置 | 后置到 U04 真实 consumer | 用户/967·968 后继 |
| D-④-5 | `a2ts.foreign-unsupported` 粒度 | view 级整拒（现状）/收窄到被调用声明 | 保持，待 976 后继 | 976 后继 |
| D-④-6 | N4 内建/宿主名表信任面 | 建/不建 | 后置；若建须能力行+审查+hash | 用户 |
| D-④-7 | 组件面是否升为 package unit（`use { package: … }` 形态，5 例） | 是/否 | 随 ④-A 一并评估 | 用户/981 |

### PHASE-④-EXT 裁定结果（2026-11-18，用户；取代上表"建议"列，决策表本身留档）

| ID | 裁定 | 落档要点 |
|---|---|---|
| D-④-1 | **A（N2 闭包）先行** | 组件真实绑定可见 Auto 模块符号，复用 960/981 包绑定与校验；17 例只是**可能受益的模块引用范围**，完整示例通过数须实跑确认（不得引用 17 作完成度） |
| D-④-2 | **开放 `use <mod>: syms`** | 981 会签 grammar 与 Schema 身份变更；以实际符号调用验证，覆盖缺模块/缺符号/私有符号三类负控。**981 回签（r18，`81b0a03b1`）修正实施路径**：不开 age-component 面——多模块闭包整体走 `age-package`（两形已在），age-component 保持单文件并把 `use` 改具名拒；硬条件=**惰性（引用驱动）绑定**（否则 34/35 例回归）；grammar 零改动。详见 §5 ④-A 合同 v2 的 S0–S8。**（本行已被下方"裁定结果"取代；用户 2026-11-18 复核接受 981 修正）** |
| D-④-3 | **首切片保留具名拒绝** | 003 `double`、031i `float` 交 981 独立定字段语义；982 已交配置宽标量可作参考，组件 model 仍需自己的验证 |
| D-④-4 | **后置到 U04 真实调用** | 届时按具体签名/效应/权限定接口，不预建 ABI 格 |
| D-④-5 | **维持现状** | ④-A 先验纯 Auto 模块的 SFC 生成；实际用例需收窄拒绝粒度时交 976 后继 |
| D-④-6 | **首切片不建 host 表** | `back.api`/剪贴板按真实消费者确认宿主端口与权限，尤其区分本地模块调用与服务端调用 |
| D-④-7 | **批准小范围验证，先保留作者语法** | 优先验现有组件入口能否复用包绑定；确需 package unit 再用实际示例定迁移范围；`use { package: … }` 五例继续单列 |

执行顺序：①U01 #14 收口（复审+真窗输入+未实跑项记录→合入→按版本交 981 可消费源/命令/oracle）→②④-A 首切片（合同见 §5，待 981 会签）→③UI058 T-02/981 联冻并行（980 WEB 收口不排其前）。

### ④-A 实测阻断与路由请求（2026-11-18，本会话实测；用户已选 **R1′**）

**用户裁定（2026-11-18，R1′）**：先做导入闭包，用**派生真实例**打通 AV 与 SFC；包面兼容补齐**并行**分切片（style/匿名 payload msg/反引号，981 会签文法并同步 grammar ID，按首个未改写示例实际阻断排顺序）；982 并行做带 `str` 实参 AV 执行再接真实 HTTP。`#14` 已落主，输入子面收据可交 981 接收。

2026-11-18 stage: work+merge（④-A 首切片 #1：点分 `use` 投影 + 引用驱动导入；非 fold）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（闭包机制已证；AV/SFC 承载未交=下一片）** | delivery: lang master **`0db44dfdf`**（ff-only，wt-guard clean）| worktree lang-980 @ plan-980-dev。

**交付。** ①**点分 `use` 投影**（`cst/frontdoor.rs`）：`age-use-module-item`（`use <module>.<name>: a, b`）此前包面**零投影**，现逐符号落成 `PackageImport{path:[module,name,sym]}`（重复名/导入顺序沿用现有具名拒）。②**引用驱动（惰性）导入**（`source/packages/binding.rs` + 前门）：源前门按"导入项之外的标识符叶"生成 `referenced` 集（注释/字符串/导入行不计），**仅 AGE 面**跳过未被引用的导入（不解析/不建边/不要求模块在场）；`assemble_unit` 装配路径**透传**该集（实测不清空——否则 binds 全空、跨模块调用变 `unknown function`）。其它面保持 eager。③**派生真实例永久用例** `plan980_closure_dotted`：`ace-node-counter` 派生源 + 点分 `use util.steps: step` + 惰性 `use themes.themes: Auto, Solarized`（模块**缺席**）；四判据全绿=模块图 `["app::main","util::steps"]`（themes 不在）、改 helper 源改受检程序、缺模块=`source.module-missing`、缺符号=`source.module-symbol`、私有=`source.module-private`。④**981 环用例按 A3 具名更新**：`p981_module_cycle_is_named` 改为**被引用**导入互指（保"环具名拒"原性质）；原"未引用互指"场景转 980 正例 `p980_lazy_unreferenced_cyclic_imports_do_not_form_edges`（A3 直接推论，不静默弱化原断言）。

**验证。** 目标用例 4/4；`cargo t ace-source` **PASS**（含 981 包面全套 + core conformance 862 例）；changed files fmt clean；探针零残留。

**边界（诚实）。** **S4 AV 真窗与 S5 同源 SFC 未交**（R1′ 第 2 步）；测试源明确标注"由真实例派生"，不宣称整例支持；包面 widget 表面（style/匿名 payload msg/反引号）仍未补（R1′ 并行分切片，需 981 会签文法与 grammar ID 同步）。

2026-11-18 stage: work（④-A 首切片 #2a：AGE 包闭包的 **AV 执行面**；非 fold）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（AV 执行面已证；真窗/SFC 未交）** | 交付：`experimental/ace/crates/av/tests/cases/plan980_closure_av.rs`（永久用例 2 例）| worktree lang-980 @ plan-980-dev。

**交付。** 复用既有 AV 消费端 `ace_av::age_package::observe_age_package`（AGE 包观察：逐实例文本 + int 状态，走 lower→MIR verify→ABC→AV）。派生真实例（与首片同源）+ 场景 2 实例 + `Send{0, Inc}`：①`step(0)`/`step(2)` → 观察状态 **[4, 2]**（helper 值进入 AV 真值）；②helper 改 `+10` → **[20, 10]**（AV 观察随 helper 源变化，非声明摆设）；③未引用 `themes` 模块缺席仍编译并执行成功。

**边界（诚实）。** ①这是 AV **执行/观察**级证据，**不是真窗**：UI 桥（`NodeDesktopBridge`/`AceStateEngine`）只吃组件面单源；AGE 包需要新桥路径（view 形状取入口 unit 的 `ComponentNode` + 包场景引擎 + 窗口 exe 多 unit 输入；包面场景**无 initials 面**——props 种子需具名拒或补面）。②**SFC 未交**：`generate_sfc_with_lane` 亦为组件面单源，需包感知入口（handler 片段引用被调 helper 的**发射名** + 把 helper 发射进 SFC 脚本）。③表面补齐（style/匿名 payload msg/反引号）仍是"整例真实源"前置。

**下一步（可执行顺序）**：**P1** 包场景引擎（`compile_age_package_with_scenario` + `observe_age_package` 驱动 dispatch/状态）+ `NodeDesktopBridge::load_package` + 窗口 exe `--package`（真窗状态随 helper 变）；**P2** 包感知 SFC 生成（同源断言：调用名=helper 发射名且函数在产物内）；**P3** 表面补齐首刀 `style`（981 会签 + grammar ID 同步）。

**落主与验证（补记）。** lang master **`4b2d3203c`**（rebase 后 ff-only；期间 master 已含 981 f32 AV 切片代码，故在**合并后提交**复跑门禁）：`cargo t ace-source ace-av` **双 PASS**；目标用例 2/2。计划收据提交 `850a49bf8`。

2026-11-18 stage: work+merge（④-A 首片 #2b：**AGE 包感知桌面桥 + 真窗**；非 fold）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（真窗已见证 helper 值变化；SFC 未交=下一片）** | delivery: auto-ui master **`145c4f13`**（ff-only；rebase 后 `crates/auto-ui-runtime` 树与已测提交**逐字节相同**）| worktree lang-980 @ plan-980-dev。

**交付。** ①`ui/ace_package_bridge.rs`：`PackageStateEngine`——包闭包 + 入口，dispatch 时按场景（实例数 + 定向 op）重编译并走 `observe_age_package` 取 AV 真值；**op 只有整条观察链成功后才提交**（失败不毒化槽）；边界具名=只观察 int 字段、包面场景**无 props/initials**（非空种子具名拒）、带载荷 dispatch 未接线（具名拒）。②`ace_node_desktop.rs`：`BridgeEngine` 枚举（组件/包）+ `NodeDesktopBridge::load_package(units, entry, instances)`（view 形状取入口 unit 的组件投影；状态/文本走包观察）。③窗口 exe：`--package "pkg::id=path; …" --entry pkg::module`（与 `--source` 二选一）。④永久用例 `tests/plan980_closure_window.rs`：CPU 面（装载→open→实例 0 `Inc`→`[4,2]`；helper 改 `+10`→`[20,10]`；initials 具名拒负控）+ **真窗 e2e**（env-gated，两次进程运行同一包闭包，base 与 edited helper 各一次）。

**验证。** 目标文件 3/3（CPU 2 + 真窗 e2e 1，真窗 38.3s 含两次 provider 预热）；回归 `ace_node_desktop` **11/11**、`ace_ts_provider` **24/24**；changed files fmt clean。

**边界（诚实）。** SFC 未交（见下）；真窗键入/输入面仍走组件面 U01 切片（包桥仅 unit 消息）；真窗 e2e 两次进程各自预热 provider（38s），非产品缺陷，后续可复用 `--prep-only`/缓存。**P2 设计（下一片）**：在包 checked 模块上建"包片段 lane"（等价 `ComponentFragmentLane`，取自 `__age_p_hidden_hv_*` 值函数），用 `a2x-ts` 片段发射器产出 handler 体文本（被调 helper 映射到其发射名），并把 helper 函数体发射进 SFC `<script>`；同源断言=SFC 含 helper 函数定义 + 调用名=映射名（缺任一即红）。**P3** 表面补齐首刀 `style`（981 会签 + grammar ID 同步）。

2026-11-18 stage: work+merge（④-A 首片 #2c：**包片段 lane + 同源 SFC script 段**；非 fold）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **pass（S5 script 段见证 helper 发射与调用名映射；完整 SFC template 不在本片）** | delivery: auto-ui master **`404dc442`**（ff-only；分支已含 master，树与已测提交一致）| worktree lang-980 @ plan-980-dev。

**交付。** ①`ui/ace_package_lane.rs`：`PackageFragmentLane`——以**包单元集合 + 入口**编译（实例 1 + 待定位 handler 的 msg 重放），持有 package checked + 脱糖源；`emit_expr_at_span(fn_suffix, span)` 在指定函数体内按**精确 span** 经 976 `a2x-ts` 片段发射；`emit_function_body(fn_suffix)` 发射整函数语句并回传**发射侧局部名**（参数名权威）；`find_span`/`function_names` 供定位。②永久用例 `tests/plan980_closure_sfc.rs`（2 例）：handler 体实际发射为 `const t0 = a2x_fn_p4_utilm5_steps_step(l0___age_s_0_count);`——**调用名=映射名**、无裸 `step(`；`fragment.helpers`=该映射名且可回溯到包视图函数；组装 script 段含 `function <映射名>(<发射参数名>)` 与同源常量（`2`；改 helper→`10`）。

**验证。** 目标 2/2；回归 `plan980_closure_window` 3/3、`ace_node_desktop` 11/11、`ace_ts_provider` 24/24；changed files fmt clean（`mod.rs` 的 fmt 分歧为预存 `dynamic.rs`，未纳入）。

**边界（诚实）。** 本片只交 **`<script>` 段**（handler 体 + 被引用 helper 定义），**完整 SFC 的 template/样式仍归既有方言道**；包片段 lane 只覆盖 976 fragment 域（float/复杂体等仍按该域既有拒绝）；helper 参数名取发射侧局部表（`p0_n` 形），不是源名。

2026-11-18 stage: design（P3 `style` 面**会签材料**；本轮不动文法/代码）| plan_id: PLAN-980 | plan_revision: r18 | outcome: **待 981 会签** | 产出：§5「P3 提案」（style≡str const/fn 映射、profile 门 `[age-component, age-package]`、两值 grammar-id 影响、验证计划、四点会签请求）| 实测：35 例 style **126 行**（125 字面量/1 ident 复制/1 带参调用；**0 参数化声明、0 `pub style`**）；stylekit **7 条 `pub style`**（4 无参 + 3 参数化含默认值与 `{p}` 插值）；移除 style 后下一阻断=包面 18/20 回滚位 `widget`、2/20 `$`，组件面 003/004 直接解析、7 例 `use` 面、5 例 model 字段、7 例槽位、2 例 `$` ⇒ style 是第一阻断但**不充分**，下一刀按实测排序。| 证据：`plan-artifacts/plan-980/p3-style-proposal-20261118.txt` | 未改文法/代码；探针删净、worktree clean。

§5 S0.5 实测：age-package 面在真实源上缺**顶层 `style`（20/35）**、**payload msg（17/35）**、**反引号文本（2/35）**；001 是唯一可直接编译的真实例且无跨模块 helper。⇒ ④-A 的"真实示例打通"被**表面补齐**前置，导入/闭包工作是其后。

| 选项 | 内容 | 代价/收益 |
|---|---|---|
| R1（建议） | **④-A 增补 S0.5a 表面补齐**：`style` 声明（→ const 形）+ 包面 payload msg + 反引号文本三件，按"980 域内收敛"处理 grammar（若改文法须 981 同提交重基线）；完成后按原 A1/A3 走真实例 | 中；直接解锁 17–20 例的可解析面，且是 B-AV 全分母共同前置（不只 ④-A） |
| R2 | 表面补齐归 **981 语言面**，980 等其交付；④-A 期间只用 001/合成闭包证机制 | 980 侧最快出机制证据，但真实例仍不可达，且与"不得以合成 fixture 冒充"冲突 |
| R3 | 换入口面：组件面接纳 package 闭包（per-unit face 改造，981 回签明确须另立评审） | 高；981 明确不涵盖 |

未裁定前 980 侧不擅自改 grammar/身份；④-A 开工停在 S0.5 结论与合同，不以 001 或合成 fixture 冒充打通。
