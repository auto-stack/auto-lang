---
plan_id: PLAN-981
status: executing
feature_name: AGE Node直接接入ACE与AV开发及A2R游戏发行
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-18
plan_revision: 18
current_step: 3
total_steps: 34
depends_on: [PLAN-960, PLAN-961, PLAN-962, PLAN-966, PLAN-967, PLAN-968, PLAN-969, PLAN-970, PLAN-972, PLAN-978, PLAN-979, PLAN-980, ag-examples/PLAN-024, ag-examples/PLAN-025, ag-examples/PLAN-028, auto-ui/PLAN-061, auto-ui/PLAN-062, auto-ui/PLAN-063, auto-ui/PLAN-067, auto-ui/PLAN-072]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-age.md]
touched_goals: [GOAL-002]
---

# [PLAN-981] AGE Node直接接入ACE与AV开发及A2R游戏发行

## 0. 变更摘要

2026-11-16 r17当前：EVENT-SPLIT-20261116明确A/B逐切片等待，三消息缺口归981原LANGUAGE/SESSION、980原AV/WEB消费者及985原认证任务；复用980/NATIVE与#6i，A关最小真实事件/保真/安全拒绝，B关完整payload/绑定/WEB及三端产品。旧980/981/985整包互等不再作为前置；原任务/勾选/完成数和全部验收保留，仅规划。

2026-11-16 r15当前：980/NATIVE在A交可复用AV＋Rust FFI Node桌面入口；本包SESSION/LANGUAGE复用同宿主与事件/资源合同，原A2R仍独立。AC桌面由1068/T-04在M2A-08后补齐，本包FUSION/AUDIT核真实混合产品，985/B最终签。原34任务/勾选/当前计数不变，仅规划。

2026-11-16 r14最新阶段：AV/AC/A2R最终完整一致；A期AC基础以外差额可逐行交M2-B，1067/1068承接原生异步与三端补齐，A2C完整对齐增强。原任务/勾选/实际进度不变；新阶段条款不能冒实现或复审已过。

2026-11-15 r13（当前交接合同）：原960编号下尚未实现的“模块图×AGE领域”集成正式归本包SOURCE的B-SOURCE-I/V/R，不重开960、不新增计划。该原任务同时交IF-981-MODULE/REF/FIELDS/ORIGINAL，分别补多模块、实例可见性、完整AGE构造映射及g01原源CPU证明；LANGUAGE继承完整语义/真实产品验证，不把最小子面当全批完成。消费979已正式交付的基础接口，不等待979整包归档；交接后979仍逐项回验其全部AC/V。原十批34任务、0/34、drafting及全部原AC/V、A/B与M2A-08门保持；只规划，无实现/探针/建树/运行/合入/恢复授权。

2026-11-13 r12（本轮当前合同）：以共同AGE Node前门从Hello逐能力扩展UI/2D/3D，消费979/H0和980的已交接口，不等980整包才开始；明确NodeName位置/AGE上下文和旧widget兼容口径，不复制前端/宿主。 用户仅授权规划修订；保原任务ID/数量、既有AC/V与进度、完整分母及阶段门，不建树、不写实现/fixture、不构建/运行/合入。此前§0修订描述为历史，冲突以本次§5–8为准。

2026-11-12 r11：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r10当前合同：十批（SOURCE/SESSION/RUST/LANGUAGE/PROOF、CONTENT/RUST2D/RUST3D/FUSION/AUDIT）共34任务、0完成。AUDIT-I/V核其余九批正式复审/交接及全部required行，AUDIT-R后十批齐交T-05；不以旧8/9批计数遗漏PROOF。仅修正规划，不实施。新增增强合同UI080、AG029/030及联合AG031消费本包最小A/所需B正式来源，不成为本包或985/A反向前置。

历史修订：2026-11-10 r9执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r8当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r7：M2必须支持传统AutoUI及完整AGE作者语法，不能只支持三类view/游戏。新增LANGUAGE完整语法/语义批，接980传统UI全模式，补五类view/选择、if/#if、Start/Tick/Frame、新旧UI组件互操作；原8批和两游戏A2R验收保留，现9批31任务、0/31、drafting。只规划，不实施/新增计划。


承原M4/2026-11-07口径M3的融合义务，现归合并AGE M2，在共同Schema链支持Node/use/model/msg/on、view/view-2d/view-3d、Life.Tick、UI+2D+3D；替换driver真实AutoVM，保direct/RQ。现有执行迁移和两游戏新后端发行可先交，生产融合并行，不等待新增2D/3D增强全部完成；M1原游戏/包不重开。合并阶段恢复M2规划，仍不授权实施。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

2026-11-16 r16当前：A仅MIN-A-AGE与必要SOURCE/SESSION/LANGUAGE/PROOF部分；完整领域/示例/Widgets及图形A2R按W1 AV→W2 AC→W3 A2R迁B。原34任务不增；已正式交的SOURCE三项保留，current_step原0校正为3，不是新增实施。整包仍executing，全部旧产品/游戏/融合义务保留。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
新增明确交付：同一正常Node作者源（场景与逻辑、多模块use）经唯一受检语义，既可由预编译宿主+AV开发运行，也可由新A2R生成Rust并编译为两个独立Windows x64游戏包。不得将Rust编译的宿主内嵌AutoVM/AV称为A2R游戏。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。
开发链：Node源码→979共同前门/领域投影→Checked HIR→共享Typed MIR→AV→AGE typed宿主；普通行为body改源不再运行Cargo/rustc。发行链：同源Checked HIR→现有a2x-core/a2x-rust→生成Rust玩法和生命周期代码→rustc/必要Cargo链接AGE中立宿主→二进制+完整资源/许可包。两链共用领域语义与宿主端口，A2R不经AV求值来生成代码。内部领域描述/资源清单可序列化，禁止先生成旧world.at+.agscene再用旧Parser/AutoVM或旧Trans重解析；不禁止正常编译IR/缓存。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11当前授权：用户要求修正过时规划状态/981计数，并将国风首切片、正式光影与联合验收细化为可执行合同，复用已交研究、不重复立项；本轮只修改规划/导航，不启动实施、建树、制作素材、构建、运行或发布。既有其他会话WIP与归档/当前Spec保留。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。
2026-11-09用户进一步明确：只完善现有979/981的职责、依赖、并行顺序、AV开发和A2R发行及两游戏验收，避免重复立项，先规划不实施。本轮只编辑这两份Plan；不新增计划、worktree、产品代码、生成物、构建/试玩测试，不签AGE M1或ACE M2完成。
同日随后用户提出将AGE M2/M3合并，在M2并行ACE AV/AX接入和2D/3D扩展。r6按该方向修订规划，AX指A2X目标族，本包编译发行必验仍为A2R；未授权全部目标/跨端。图形增强合同可并行规划/后续按依赖实施，本次不制作素材、不运行先证、不创建树；不重复立融合或游戏发行计划。
用户再明确M2须支持传统AutoUI及完整AGE语法。r7将传统UI实际迁移明确归980、完整Node及两类UI/图形组合归本包；旧pilot的有限profile排除项不成为M2完整语言限制。仍只修订规划，无运行/实现授权。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
2026-11-06原调研读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。该轮现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。该轮引用会话及936/944状态核验保留为历史，本次基线见下。

实际入口/文件：
- `D:/autostack/auto-ui/experimental/autographics-auto-driver/src/lib.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/docs/design/autographics/m4-autoui-integration-roadmap.md`（本轮确认存在；开工重核符号/版本）。
- 本次核查语言HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；Specs入口`docs/specs/overview.md`及`ace-migration/{m2-modules,m2-numbers,replacement-scope-and-evidence}.md`、`auto-a2x/architecture.md`、`auto-a2r/project.md`。960/961已归档，其单独profile不等于组合支持；960的a2x全局cell/类型方法片段边界必须逐项处置。
- 已存在语言改面：`experimental/ace/crates/{core,source-query,lowering,av,av-core,a2x-core,a2x-rust,a2x-store,a2x-worker}`；`a2x-rust`包名`ace-a2x-rust`。新增Node投影/AGE typed端口/Rust发行适配均为拟实现，不能将同名运行时或有限标量工具认作整链。
- 内容真实入口：`D:/autostack/ag-examples/games/platformer-{2d,3d}/src/main.at`、各游戏`tools/verify_game{2d,3d}.py`、`tools/package_game_release.py`、`tools/verify_game_release.py`；引擎入口`autographics-auto-driver/src/bin/age-game-player.rs`与`tests/standalone_game_player.rs`。现有工具只证明旧链；新后端参数/fixture须由所属I任务补入既有入口并冻结，不预称当前可用。
- 现状以ag-examples仓`docs/specs/games/{platformer-2d,platformer-3d,windows-portable-release}.md`和auto-ui仓`docs/specs/auto-graphics/{ace-migration-feasibility,standalone-windows-game-player}.md`为准。旧设计中的“G3目录未建/未交付”是历史，不据此重建游戏。
- 完整语言目标补读auto-ui `docs/design/autographics/declarative-authoring-model.md`§1–2（AG-A01..09）、`multi-presentation-selection.md`（五view/选择/生命周期），以及真实WidgetDecl和UI组件/Store Specs。旧Spec明确未交view-portrait/view-widget/Frame等，仅说明旧profile现状；本包必须新增实现/证据，不能据旧排除条款删除用户新要求。尚未批准的任意扩展DSL/新平台不自动加入。

已完成工作复用：AG014/016的完整玩法与资源、AG024/025的M1收口证据、UI072公共player及AG028（archived 11/11）的两个便携包均保留。UI067已交研究结论及原基线no-go作缺口输入，不重复开展另一套迁移先证；本Plan只补新前门/执行/发行，既有资产许可与未变oracle可核身份复用，旧运行结果不充新后端PASS。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：AGE Node直接前门/AV会话/A2R发行与完整语言。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-ui/experimental/autographics-auto-driver/src/lib.rs](../../../auto-ui/experimental/autographics-auto-driver/src/lib.rs)：替AutoVM/AutoTask与旧Parser链；读取SHA256前16 `2b7af9b5c3ee7490`

- [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs)：Node/UI同一领域语义；读取SHA256前16 `1d10f5ff2566422e`

- [experimental/ace/crates/a2x-rust/src/provider.rs](../../experimental/ace/crates/a2x-rust/src/provider.rs)：新provider真实生成/编译，非旧AST emitter；读取SHA256前16 `3ea453dc2452576f`

2026-11-13本轮授权/取材：用户明确要求修订Node首Hello、Schema解析校验与位置/AGE上下文名称、widget兼容和963并行顺序，仅规划。读取基面`dcf0535d4d15c00628a60f1940c6c55a01a101ab`；当前Spec仍描述有限统一CST与旧UI parser/有限Node pilot，并未宣称本次机制已交；差额是本Plan目标，未来按SD沉淀，不改canonical Spec伪造现状。来源：`docs/specs/auto-syntax/{source-frontdoor,grammar-contract}.md`；`docs/specs/auto-ui-dialect/design/full-grammar.md`；`experimental/ace/crates/syntax-runtime/src/{program.rs,parse/mod.rs}`；`experimental/ace/crates/core/src/source/cst/frontdoor.rs`；`auto-ui/docs/specs/auto-graphics/{node-basic-ui,ace-migration-feasibility}.md`。963执行进度与其他会话WIP只读保留。

物理路径核对：原`D:/autostack/ag-examples`已更名为`D:/autostack/age-examples`；本轮修复旧样例/计划链接到实际文件，原AG计划ID/依赖身份与历史源码hash不改。执行者按现路径核正式来源，不能重新复制内容仓或生成替代旧用例。

### 2026-11-15 本轮交接修订取材与授权

用户明确要求修订979/980/981及相关路线，复用现有计划承接原960多模块缺口、消除循环、保留979完整验收；**只规划**。本轮不创建计划号/工作树/agent/chat，不实施、探针、fixture、编译/测试/GPU、提交/合入/发布，不恢复979目标或AG029暂停。读取语言基面`86c7fd50bc6dac5ff6d2a5c6215ad01fbfefc1f7`及当前文档WIP；保其既有改动、任务ID/勾选/计数与历史收据。已归档960只读。

已核规范/代码SHA256前16：`docs/specs/ace-migration/m2-modules.md`=`be25b0c69f2f8cc0`、`m2-schema.md`=`5797e15925b07e14`；`experimental/ace/crates/core/src/source/packages/binding.rs`=`afa4f886dfc2fab9`、`source/domain.rs`=`1613b66c2383bc40`、`source/age_component.rs`=`58081b0e1eb6347a`、`source/cst/frontdoor.rs`=`17016eecbc0fbd61`；`experimental/ace/crates/av/src/age_component.rs`=`55b84b8f1ca9d5a8`；auto-ui `docs/specs/auto-graphics/node-basic-ui.md`=`0a3e8616b8d2239b`、`crates/auto-ui-dialect/src/ast.rs`=`1d10f5ff2566422e`。这是读取身份，不是本轮运行PASS。960的普通模块与979的单模块领域接口已分别存在，跨模块AGE组合/字段适配/实例引用与原源CPU交接仍是拟实现；§5拟类型/样例不是现存API。现行Spec保已验事实，仅在本包规范增量表登记未来更新，不改canonical Specs伪造支持。

本轮取材/授权：2026-11-16用户要求明确AV桌面先交与B期AC桌面的责任、验收、顺序，仅规划。读取基面与已观察HELLO边界见[980 §4/5](980-ace-m2-autoui.md)；规范知识沿现有m2-autoui与auto-ui Node/Session Spec，`docs/specs/ace-migration/m2-age.md`尚待本包真实交付时新建，计划不能伪造其已存在。复用979/981 SOURCE正式接口与UI067研究，不重开归档/不新增Plan、不运行产品。

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

## 5. 详细设计

### AGE首批后能力与AutoForge接面（2026-11-17，分阶段接收）

统一[三路线strategy](../design/strategy/age-m2-three-route-strategy.md)与[公共架构](../design/autographics/age-m2-three-route-architecture.md)。保34任务、原十批、当前进度及A/B全部义务；本条不宣布新实现或解除实际未交接口。

| 现owner/原任务 | 新增精确交付 | 所用前置/不反向等待 |
|---|---|---|
| SOURCE/LANGUAGE/SESSION I/V/R | 消费UI058/080和UI099–102事实定义，补对应ResourceType/服务签名、float/向量/资源/clip typed字段、原span和唯一verify/自动Rust binding/owner端口。Forge要用的文本输入/选择/具名msg/参数payload沿原EVENT-B关闭 | 先生产最小接口及独立合法/错误/生命周期case，不等图形包/Forge终验；cap neutral事实可先准备，AV产品用M2A-08及其实际接口 |
| LANGUAGE/SESSION/FUSION AV部分 | 同源MAP/ASSET/PBR/CLIP的AV域接入/资源/错误与真实产品，由UI099–102给中立实现和AV作者场景；UI包完成需精确正式端口 | 接口先独立R/fold→UI真实AV→本包接正式cap-AV来源。阶段子面不等本包十批全交，不能让双方等对方整包 |
| LANGUAGE/FUSION AC部分＋1068/T-04 | IF-AGE-GFX-AC/<cap>：同源AC机器业务/同host资源事件、独立CPU/GPU/错误/恢复/包；原G2/G3子面同理 | UI-W2-AC、对应cap AV及其最小AC接口齐；W2原UI基线不等待全部新cap，本包不重复AC compiler |
| RUST/FUSION、所用RUST2D/3D/CONTENT部分 | IF-AGE-GFX-A2R/<cap>：同源生成/真实rustc/dispatch/GPU与资源冷包、独立oracle；旧A2R结果仅同身份复用 | UI-W3-A2R、对应cap AC及其A2R接口；不每个AV迭代编A2R，不等待AG031总签来生产 |
| AUDIT I/V/R | 将四cap的三端来源/具体V/AC差额交AG031原门＋新增扩展门，保旧完整语言/两游戏/包/OS/融合义务 | 逐行接正式已交，最后才核全包；原UI波先按原分母签。Forge仅消费接口，不成为本包/985 A出口前置 |

新增cap复用UI099–102同一源、资产、预算/profile和独立oracle；AV、AC、A2R分别交，禁AV回退冒AC/emit冒A2R。IF字段与命名见共同架构，不建第二领域模型/解释器/renderer。公共字段不足必须回本LANGUAGE/SESSION owner，不能在内容/pilot用字符串特判。阶段交付均独立R/fold并绑定实际source/Schema/SDK/ABI/工具/平台；完整任务必须所有适用格闭合才勾，原全包review/merge仍保留。

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

### 本包A先交，B事件producer先于完整consumer验收

- T-01先取980/NATIVE正式源与原Rust测试中的独立oracle，按A/B键拆`V_PENDING_INPUTS`：A列最小UI/资源/事件及当前场景必需；B列完整语言/参数/WEB/图形A2R。清单不能删除，分类/实际未交/producer必须可核。
- B-LANGUAGE-I/V/R[A]→B-SESSION-I/V/R[A]的实际依赖部分→B-PROOF-I/V/R[A]，可按稳定接口并行开发/逐项验证；彼此不得以“整任务未勾/整包executing”互等。MIN-A-AGE全required及IF-981-EVENT-A独立R/fold后交985/A；**A不等完整B-RUST、980 WEB/TAURI/PARITY或M2A-08**。
- M2A-08后，LANGUAGE[B-W1]先按下面具体设计完成三个EVENT-B producer，SESSION[B-W1]接AV typed队列与资源，短phase正式交980 B-AV/B-WEB；不是先等980完整验收才造事件源。完整W1由980/981各自领域组合后交UI-W1-AV，再进W2、W3，全部LANGUAGE/SESSION/RUST原AC/V最后仍逐行验齐。
- R59-5由现有B-LANGUAGE-I/V/R与syntax-grammar按真实A使用/诊断义务关闭，不在投影侧重复试修；非消息参数的SOURCE/node/helper语义沿原合同。不得把#6i语法交付当R59-5、参数绑定或资源owner已修。

### 三缺口的类型与代码设计（拟实施，保留旧API/已交结果）

| owner改面（已存在） | 拟定变更与不变量 |
|---|---|
| `experimental/ace/grammars/auto-source.at`、`core/src/source/cst/project.rs`、`core/src/source/age_component.rs` | 复用#6i `msg_name/params`字段。A在现有ComponentNode/受检描述保可选消息声明名及Ident/span，在ComponentHandler保有序`params: Vec<Ident>`，不另造AST/领域模型；唯一verify前检查声明/variant/handler arity。具名组作为AGE上下文声明，非全局关键词；完整类型可见性/quoted/外部emit语义在B按原Spec。字段名可等价调整，保留/核对原源身份不可省。 |
| 同上 `ScenarioOp`、`instance_stmts`及`age_package`重用路径 | A保持旧`Send { instance,msg }`的unit兼容和显式拒绝。B拟增`SendArgs { instance,msg,args: Vec<Expr> }`（既有Send仍等价空载荷），沿现Auto表达式/共享resolver把载荷类型拼写解析为共享类型；候选Expr必须唯一verify，不能信任字符串类型表。初始化/实参在其声明作用域求值一次、按位置绑定到卫生局部名，再用于handler体；`.field`仍指实例model，裸参数名不误重命名成字段。同实例不同事件/不同node不能串局部；派生driver只有被证无副作用、求值次数一致时可复用。 |
| `experimental/ace/crates/av/src/age_component.rs`、`av/src/age_package.rs`及同owner conformance | 重导出/调用新场景表示，真实AV算出独立值和轨迹；共享IR变化仍补必要内部消费者检查，但不在AV波每批构建完整A2R图形应用。原类型/位宽/预算/资源合同不改。 |
| `auto-ui/.../ui/ace_state_bridge.rs`、`ace_node_desktop.rs`与981 Session宿主owner | B在既有`AceStateEngine::dispatch`旁提供typed带参入口（拟`dispatch_with_args`），旧dispatch仍unit。复用967资源/968调度/970生命周期；队列拥有或按既有handle保留载荷，不能把临时borrow跨tick/close存队列。域ABI显式版本化/兼容，不偷偷重解释A的整数编码；错type/arity/实例/世代/权限在效果前拒，close/late回收、失败保旧与恢复可观察。 |
| `auto-ui/.../ui/ace_ts_provider.rs`及980 WEB owner | 现`ComponentFragmentLane::compile(...handler_msgs)`只拼消息名；B须接真实参数环境/受检函数入口，发射真实形参/参数引用，不能用默认值/常量场景替代动态参数语义。对SFC splice保原作者span/脱糖映射；铸造lambda锚点用原view属性span/来源链，不伪造作者on块。服务/生成consumer只消费已审typed结果，Core不反向依赖UI。 |

诊断沿现有owner：合法但当前行为车道不支持的payload/参数执行用`source.age.component-unsupported`并指向原声明/实参；未知消息、声明/handler arity、重复binder等领域错误沿`source.age.component-event`，共享类型错误仍由唯一verify报告原span。宿主继续原DispatchRefused/LaneError及状态码，编译/工具/权限/资源失败不伪装语法错误；新增精细子码只能版本化扩展原命名族并保原区分性控制。

实现前由原T-01/LANGUAGE-I核实际API漂移与等价机制；本表已决定数据流/责任/拒绝/求值/兼容，不把是否实现三项再次留给“后继待立项”。新增字段/入口都是拟议，不冒已存在或本轮已编译。

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

### M2阶段执行合同（本轮当前）

本包跨A与B三波；A取MIN-A-AGE的真实AV部分，完整SOURCE已交事实复用。SESSION/LANGUAGE/PROOF每项可先按A子面独立R/fold交接口，但整任务只有其完整责任闭合才勾，完整范围不再是A出口门。B分后端与产品覆盖，不另建每端源码/renderer；UI-W1-AV不等两游戏全离线包或AGE独立G2/G3里程碑。

| 阶段 | 原任务/关闭责任 |
|---|---|
| A | T-01；SOURCE已交I/V/R；SESSION、LANGUAGE、PROOF各自MIN-A-AGE子面I/V/R与正式phase交接，只AV |
| B-W1 | SESSION/LANGUAGE剩余完整AV、CONTENT的AV开发/原内容消费者、FUSION的AV域格；PROOF完整AV格；交980 B-AV-R汇入UI-W1-AV |
| B-W2 | 1068/T-04主责AC机器执行，本包FUSION/LANGUAGE/PROOF协作补所需AC域格，交UI-W2-AC；不等本包RUST/最终AUDIT |
| B-W3 | RUST公共生成链、RUST2D/RUST3D实际游戏包、LANGUAGE/PROOF/FUSION的A2R格；UI相关结果交980 B-RUST-R的UI-W3-A2R；游戏完整包原门独立保留 |
| B最终 | CONTENT/FUSION/AUDIT及全部原域/产品/后端行汇合，T-05/06/07最终；OS真实桌面归OS064，不反向当本包A/W1的条件 |

W1/W2/W3全部在M2A-08之后，A只允许MIN-A与来源/矩阵准备。先A切片正式交、后全包完成；同一唯一树phase fold/resync保留至B结束，不以A交付或换波次提前归档/删树。

### 职责交接与无环并行顺序

**r13覆盖说明。** 下面“979不等产品PASS”是指不等980/981整包或B产品；979 r11明确等待本节四个具名CPU交接与980字段清单，再自行完成全部原V。979已落主基础可先消费，不能以“979仍executing”拒绝本包SOURCE，也不能拿本包子面PASS替979最终review。旧r12泛称“L979合同”的地方均按已交接口解释，整包最终依赖并集不作为各子面的开工门。

979只交通用grammar/typed投影/唯一verify与最小双方言双后端证明，不承接本包完整AGE/GPU/游戏验收；981只扩该接口，不再开发另一套Node前端或通用A2R。语言仓维护绑定/类型/资源事实、AV和A2R合法化；auto-ui维护Node领域语义、场景/绑定描述、资源/输入/音频/窗口renderer及AV/Rust中立宿主；ag-examples维护作者源、原玩法oracle、资源闭包、各模式打包与最终ZIP验证。Core禁止反向依赖UI。967管通用FFI/handle，968/970管调度/会话，972管标准库，978管通用工程构建/SDK；981消费并补AGE实际组合与接线，不复制这些系统。980/RUST是UI静态工程，不代AGE A2R验收。

1. 实施获得授权后，T-01先核当前交付、建立源码/构造/宿主操作到上述owner的差额表；缺口必须具名，不把AG028完成改成A2R完成。979平台开发与981的领域合同、原oracle/包清单准备可并行；981实际消费等相应979接面正式合入。979的最小消费者不依赖981或980最终结果，不能互等。
2. SOURCE的共同受检语义就绪后，SESSION与RUST可按各自最小接口并行；RUST不依赖AV会话/热更新完成，SESSION也不等Rust发行。通用宿主/构建计划依赖稳定领域接口，不以981最终游戏验收作其完成前置。
3. CONTENT随SESSION就绪迁移AV；RUST2D、RUST3D随RUST就绪可分别准备/构建，不要求2D验收通过后才开始3D。各游戏最终V/R必须拿到同源同资源的AV对照和独立oracle；不得只拿旧VM结果替AV。
4. FUSION随SESSION与980/PARITY、现有RQ/UI接口成熟推进，不阻塞SOURCE/SESSION/CONTENT或两个游戏Rust包；其余九批R齐后进行AUDIT-I/V/R，十批齐后进入T-05→T-06→T-07，AUDIT不能等待自己的R才开工。不能等985最终认证、986正式发行或987删旧引擎后才构建候选，981负责交候选/证据。

5. 合并AGE M2另有G2/G3线：UI058先做中立查询/精灵/碰撞服务，三维正式光影先冻结资源与端口，均可与本包并行。其最终正常Node/AV/A2R接线消费本包正式交付；本包现有M1迁移/融合不反向等待UI058/AG023、新光影或国风首切片终验。979只证最小平台，不背完整增强GPU验收。新增typed端口/能力/资源schema须版本化登记并核AV/A2R支持；不为新增图形扩旧Node生成器或复制执行器。新增强合同仍drafting、未实现，不充本包已迁事实，已启用consumer必须有明确新producer/迁移owner，不能遗留无人接管的旧SDK依赖。

6. SOURCE仍是既有M1作者源迁移切片，不冒完整语法；LANGUAGE-I可与SESSION/RUST/PROOF在A期并行；CONTENT在A出口后于B推进，原CONTENT与RUST2D/RUST3D的内部并行关系保留，完整V须依相应后端和980正式接口，FUSION最终V核LANGUAGE组合结果。980不反向等LANGUAGE/FUSION产品PASS；979仅等本节具名IF后完整回验，不等两者整批/整包；AUDIT-I/V核其余九批R及全部required行，自己的R完成后十批齐交T-05。游戏迁移不先强制使用所有view，其通过也不替LANGUAGE。

并行指不同写面/接口就绪的工作，不授予另建agent/chat或多权威tree。共享schema/core/verify/host ABI、打包公共工具按owner短窗串行；同机GPU/窗口/音频实测单实例。稳定合同只允许受控准备，最终证据绑定正式交付repo/SHA、Plan revision、grammar/domain版本、编译/运行后端、host ABI/provider、源闭包与资产hash、平台/toolchain；不能用开发tip/旧SDK锁代替新来源。

### Node首Hello、位置名称与无重复接入（2026-11-13，当前）

共同前门先由979/H0交AGE Schema驱动的`node Hello { view { text "Hello" {} } }`及普通函数文本，980/HELLO接真实AV/FFI/窗口；这是本路线的Node最小基线，不先完成另一套widget引擎，再重做Node。981/SOURCE从同一版本接口扩use/model/msg/on/Life、实例与2D/3D，LANGUAGE扩完整普通UI/五view/条件/Frame；H0不冒完整SOURCE或LANGUAGE，980不等待本包整项PASS才交Hello。

沿979 NodeName合同：`widget/model/view/on`仅在AGE已激活、Schema声明的节点头名称槽及允许父规则内具领域意义；变量/形参/函数/字段/字符串及非AGE模块保普通Auto语义，text/其它组件按Schema解析，不在共享lexer硬编码领域保留词。Schema既驱动parser，也做结构/投影约束；普通Auto行为仍经唯一类型/效果/资源verify。旧widget是同一Schema兼容声明头别名，规范作者形式为node；原源直读与只改声明头的迁移副本分别验，不能以后者冒前者，不另造业务模型/源码预处理器。各旧case/check保原ID与expected，一对输入只累计一次义务。

与963并行的入口只限979/H0和967/F0的既有普通函数/text/scalar能力，以及980/981领域Schema、组件表/原oracle/宿主端口合同准备。公共grammar/CST/AST/resolve/source-query由963当前owner与对应计划短窗单写、固定SHA交接；不修改963计划、工作树或直接消费其未审产物。979/H0-D、F0-D、E0-D、B0-D齐才做980/HELLO；本包SOURCE实际接线等待其所需979通用接口及真实语言能力，不能从H0小例免掉963/966/会话原前置。SOURCE后SESSION/RUST依原接口分叉；LANGUAGE按980的已交DIALECT/STATE等接口推进，不等其整包模式终验，最终LANGUAGE/FUSION/AUDIT仍核完整兼容与产品门。980/979不反向依赖本包整包；979的具名IF回验输入按r13本节执行。

执行与验收必须可追到`原Node或widget源/hash→活动AGE Schema/上下文/兼容配置→CST原span→共同领域候选→唯一verify→AV或新A2R→同合同组件/宿主`。纯UI、UI+2D、UI+3D分别保独立状态/事件/资源oracle；头部改名不应改变业务结果。V-NODE-CHAIN由B-SOURCE-I/V承接，V-NODE-UI由B-LANGUAGE-I/V承接，原各R独立复核；最小Hello证据可按来源复用，不重复建设或重复计数。原十批34任务和M2A-08/B产品门保持，AG029暂停/UI080等待不由本修订放行。


### r13：SOURCE承接模块领域集成并向979正式交接

**实际承接。** 960已8/8归档，只提供模块基础；其归档没有交付AGE领域组合。所缺工作全部落实到本包原B-SOURCE-I/V/R，auto-lang在本计划唯一树中维护必要的共享grammar/CST/packages/verify/AV/A2R改面，auto-ui维护领域包/节点与实例语义。Core仍只含通用数据和接口，不依赖auto-ui crate。此分工继承979“各owner交付后组合收口”的裁定；979不代做下列生产工作，但收到接口后保留完整AC/V复核与回归责任。

| 正式交接键（非新任务） | B-SOURCE-I/V/R必须交付 | 输入/独立关闭oracle | 后续责任 |
|---|---|---|---|
| IF-981-MODULE | 普通模块图与AGE Schema的组合profile/前门、逐模块领域投影、跨模块名字与Node/资源导入、原文件source map、统一Checked结果与两后端消费接口 | 两文件helper+Counter、两实例Inc=2:1、各4次Life.Tick且每次+1，独立状态6/5（seed=0）、逐实例文本；改helper初值后两端实际变化；私有/缺失导入/环与失败恢复；真实AV与rustc后的A2R，不是拼接旧源或emit-only | 979 V-09④及T-03/T-08回验；完整SOURCE继续迁移所有原场景 |
| IF-981-REF | 同源typed节点/实例身份、作用域/可见性与声明引用的检查，不用宿主下标越界替源级引用语义 | 合法同型引用两端真值；未声明与已声明但不可见的同型引用在共享绑定/检查处分别具名拒，带原模块/字段span，零运行/零新工件；修复同入口恢复 | 979 V-10未闭部分回验；LANGUAGE原I/V/R继承五view/生命周期/完整互操作 |
| IF-981-FIELDS | AGE-L01..05、AG-A01..09及实际grammar/AST全部必需构造的来源/投影/检查/consumer表，接979通用覆盖机制；与980 IF-980-FIELDS并集 | 源/Schema可表达每类所需typed事实；实际删必需映射、删必需清单行均红，合法近邻与修复后绿；未知字段不得静默丢；未具备的产品运行格仍具名pending | 979 V-11/AC-08回验；LANGUAGE负责完整语义实跑，FUSION/AUDIT保原产品门 |
| IF-981-ORIGINAL | g01原主模块、resources/shapes等真实闭包在共同前门的typed投影、资源引用与CPU值读取；保持旧冒号use等已承诺作者表面 | 原四图形坐标/尺寸/颜色alpha/order与黄quad隐藏逐项独立断言；实例化Disc的公共定义/资源确来自导入模块，改被引用资源则值变化；错误报原文件span；完整原源hash记录 | 979 V-DETAIL-01按原CPU边界验；本包真实GPU/窗口/输入及两游戏仍由原SESSION/RUST/LANGUAGE/PROOF/B批验收 |

**已核基础与拟新增接面。** 现存960模块代码在`experimental/ace/crates/core/src/source/packages/binding.rs`，有`PackageModuleInput { package, version, id, source, provider }`、`bind_package_syntax`、`BoundPackages`及模块/source origins；现存979接口在`source/domain.rs`的`DomainPackage`/`DomainCandidate`/`compile_domain`、`source/age_component.rs`的`ComponentCompilation`及单模块编译入口，AV有`observe_compiled`。这些目前**不是**可直接组合使用的多模块AGE API。B-SOURCE-I先用原正例/非法近邻/恢复探明最长路径，比较“逐模块CST投影后接既有包绑定”与等价的共享包前门扩展，选择可保持所有下述不变量的一种；不能只改profile名字宣布完成，也不创建第二个绑定器/verify或Node解释器。

拟扩展语义数据如下（草图，未编译/未实现，允许复用等价现有类型）：

```rust
struct DomainModuleUnit { key: ModuleKey, source_hash: Hash, cst: CstRef,
    candidate: DomainCandidate, source_map: SourceMap }
struct AgePackageCompilation { entry: ModuleKey, units: Vec<DomainModuleUnit>,
    description: TypedDomainRows, checked: CheckedProgramRef, identity: DomainGraphIdentity }
struct DomainGraphIdentity { module_graph_hash: Hash, grammar: GrammarId,
    domain_version: DomainVersion, compat: CompatConfig, capabilities: CapabilitySet }
```

每个输入文件独立解析/保原span，领域回调只产未受信候选；普通项和领域行为脱糖共同进入既有包绑定与唯一verify。模块级Node定义/公共helper/资源的导入可见性须形成typed符号/引用关系，不能按名字字符串或Rust业务常量决定。继承960的公共/私有、导入顺序、包/模块身份与图/初始化规则；Node导入必须在共同Schema定义合法形式，旧g01的`use resources: ...`/`use shapes: Disc`由该Schema和绑定适配承接，不用字符串预处理改成另一源码再交旧parser。依赖图输入排序/宿主重定位不改变语义身份，改被引用文件/Schema/compat使相应证据失效；跨文件错误映回原模块，不能只报合成模块偏移。共享语言/后端缺口由本包对应原I任务和语言owner补齐，不能再空挂已归档960，也不把979最终归档当开工前置。

**拟最小两文件输入**（不是当前可运行声明；B-SOURCE-I负责接入永久case）：

```auto
// helper.at
pub fn seed() int { return 0 }
pub fn label(n int) str {
    if n == 6 { return "six" }
    return "other"
}
```

```auto
// main.at
use helper.seed
use helper.label
node Counter {
    model { count int = seed() }
    on Inc { count = count + 1 }
    on Life.Tick { count = count + 1 }
    view { text label(count) {} }
}
```

上述模块导入形式来自960，Node/model/view沿979及现行领域规范，Life.Tick沿AGE原合同；**组合能力正是本任务待实现项**。配置两实例并投递Inc=2:1、各4个Tick时应读6/5与`["six", "other"]`；配置驱动仅创建实例/投递事件/读回，init/helper/handler/文本必须由受检作者行为执行。改seed为10时独立状态16/15，文本都为other；坏私有helper、图环/缺模块先于运行拒绝。完整SOURCE还必须验证跨模块Node定义/typed资源导入及原g01完整闭包，不用helper示例代替全部构造/原源义务。实例引用的合法拼写、私有/公开边界优先沿原AGE定义/用例；若现规范未定，由B-SOURCE-I在字段与合法/非法同型例对上作有界决定后实施，不能用下标数组或仅越界检测冒源级语义。

**交接与非循环依赖。** 四个IF各自可在原I/V/R中作为有界子面冻结、真实验证、独立复核并phase fold，记录repo/正式SHA、r13、API/profile/grammar/domain/compat、源/模块/资源闭包hash、source map、能力/预算/后端与实际case/命令/结果、剩余未闭格；T-01与整批I/V/R仅在其全部原义务完成后勾选。981/REF/FIELDS的最低CPU交接先由SOURCE生产，LANGUAGE继承扩全，不等待其全量UI/SESSION/RUST/PARITY；IF-981-ORIGINAL是g01原源CPU投影，不先做GPU或B游戏。979通用代码若需扩展由本SOURCE原任务内的auto-lang owner修改/复核后交付，不让两个计划同时写同文件。

顺序是`960已交 + 979已交基础 → 本SOURCE四个IF`，并行`979已交基础 → 980 IF-980-FIELDS`，然后`979 T-03/T-08/T-04全V → T-05/T-06/T-07`；本包其余SOURCE/SESSION/RUST/LANGUAGE/PROOF按真实接口继续。没有`979归档 → IF生产`边，也没有`985/A或981产品B → 979回验`边；IF只是新增源输入证明，不关闭本包原A/B/产品义务、全部十批或AGE里程碑。HELLO仍只取979/H0等原最小接口，不新增本节多模块/引用/字段门。

### 981/SOURCE：AGE正常作者前门迁移
改面（均核存在）：`D:/autostack/auto-ui/experimental/autographics-node2d-pilot/src/load.rs`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/src/lib.rs`。
I实现：复用UI067既有差额，经979接入Node/use/model/msg/on/Life.Start/Tick与view/view-2d/view-3d；场景引用、组件实例、行为和普通helper共同绑定/verify，保f32/数组/bool及原span。静态场景与动态绑定进入typed领域描述，非旧源码/JSON生成后重解析。盘点旧生成骨架承担的输入快照、事件投递、巡逻/终局fresh Enter/音画收口：逐项归通用生命周期或显式作者逻辑，不能漏掉、继续依赖旧生成器或按Game/Coin字段名硬编码玩法。
V验收：普通多模块源含两个局部Counter、UI/2D/3D，定向事件2:1、相同实例多view不重复初始化/Tick；错误指原文件；无legacy Parser/AST emitter、手写HIR或Rust替代玩法；CPU独立oracle核值/事件/领域绑定。相同迁移后源码和资产快照供AV及A2R，语法适配差异须显式记录，不保两套业务源。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 981/SESSION：AGE Session与图形host替换
改面（均核存在）：`D:/autostack/auto-ui/experimental/autographics-auto-driver/src/lib.rs`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/tests/mixed_regions_lifecycle_flow.rs`。
I实现：替Session AutoVM/native注册为AV+typed AGE宿主；tick、回调、资源、同步/异步、同PID更新与停止恢复；复用预编译宿主，普通业务body编辑不调用Cargo/rustc。宿主不手写Rust玩法；RUST批由同源生成Rust属明确发行路径。
V验收：两组件局部事件2:1、N tick恰N次；旧在途/新代分离，late event不复活，多Run/Stop资源净归零；一次有效body编辑同PID产生新结果、cargo/rustc=0，错误编辑保旧，修复后正常更新；不兼容布局显式拒绝/可见重启，不能假称任意状态迁移。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 981/CONTENT：两游戏Gallery编辑与离线包
改面（均核存在）：`D:/autostack/ag-examples/tools/gallery_node_authoring.py`；`D:/autostack/ag-examples/showcases/ui-2d-3d/tools/verify.py`；`D:/autostack/ag-examples/showcases/ui-2d-3d/tools/package.py`。
I实现：内容作者源不减功能，逐游戏/Gallery/profile接SOURCE/SESSION的AV开发链与978候选SDK；编辑保存/SaveAs/搬迁/关开、旧冻结SDK与间接测试链全部纳迁移。按2D/3D分别产同源AV结果供RUST2D/3D最终比较；源码重建包、AV开发包、A2R二进制包显式分型，不改名互冒。
V验收：原M2-01..10/G-M01..09及实际变体有新producer收据；仓外离线冷包真实GPU/音频/输入/编辑；AV开发链不夹旧VM，实际改源可见；不以本批AV包称A2R发行完成。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

### 981/RUST：公共A2R编译发行链
改面（已存在）：语言`experimental/ace/crates/{core,source-query,a2x-core,a2x-rust,a2x-store,a2x-worker}`；UI `experimental/autographics-auto-driver/{Cargo.toml,src/lib.rs,src/bin/age-game-player.rs,tests/standalone_game_player.rs}`。新静态AGE入口/生成代码链接fixture由B-RUST-I接入这些owner，具体符号先冻结。
I实现：同一Checked HIR经现有新A2R provider生成实例状态、初始化/事件/Tick/绑定更新的Rust代码，调用与AV相同合同的typed AGE服务；复用978工程/工具链及967 FFI，不造第二个编译/安装系统。建立普通Node→Rust→编译→窗口/GPU/音频/输入最小闭环。明确960边界：当前全局cell/类型方法片段不足时，由本批语言owner实现所需受检实例状态/初始化/调用的目标合法化（可用显式状态参数等合法表示，策略先证）；通用数组等归962，标准库归972，不重开960/961，不靠手写world或Rust玩法补结果。两游戏使用的语言+领域组合和实际host import须在AV/A2R都闭合。
V验收：实际生成并编译运行，改变Node业务常量/条件后Rust玩法结果随之改变；执行闭包不含AutoVM/AV解释循环、旧Parser/Trans/源码生成器/未声明外部SDK依赖。审计生成函数→实际链接/dispatch，并在去掉或禁用VM执行provider的构建中运行，不能只grep包名。错host ABI/签名/handle代次、编译失败/取消、坏资源受控失败；失败不覆盖上一完整候选，修复后同入口重新构建运行。
R关闭：语言与UI各自提交来源、域组合/端口、编译及运行证据；仅emit、空选择、headless绿、Rust宿主内嵌VM均不能关闭。RUST公共批通过不替两个正式游戏验收。

### 981/RUST2D与981/RUST3D：逐游戏A2R整链与发行
改面（已存在）：`D:/autostack/ag-examples/games/platformer-{2d,3d}/{src,checks}`、`games/*/tools/verify_game{2d,3d}.py`、`tools/package_game_release.py`、`tools/verify_game_release.py`；原UI072 player外壳可复用，其原AutoVM二进制不能直接改名充A2R。两个批分别I→V→R，不合并完成计数。
I实现：各自消费同一SOURCE后的作者源/关卡/资源快照，经RUST/978真实生成Rust并编译release，复用AG028的闭包/许可/中文说明/打包验证结构，扩明确后端身份和启动合同。各交Windows x64便携ZIP、exe及全部资源/必要合法非系统DLL、许可/说明、逐文件hash与源→HIR→A2R/provider/runtime/toolchain→exe→ZIP来源；区分AV开发包和原VM发行包。只迁后端，不新增关卡、美术或扩展图形能力。
V验收：分别运行§6 V-R2-*或V-R3-*及共享V-R-PACK/V-R-FAIL；必须最终ZIP仓外冷解压、真实可持续游玩，原G2-01..09/G3-01..08逐项映射；与对应AV结果及独立oracle比较，不能删旧分母/降低预算。用户机器无Rust/Cargo/Python、无仓库/生产者缓存/网络仍可启动；资源相对exe定位，不受cwd/空格中文路径影响。A2R玩法在进程内实际执行，无VM补逻辑。
R关闭：各游戏独立绑定源/资产/最终ZIP SHA、实际平台/输入/画面/PCM/HUD/生命周期、全部正反例结果及已知限制；2D绿不替3D，自动轨迹/inspect不替真实键鼠/音频。未执行的物理试玩保持pending，不能据AG028旧试玩直接关闭新后端。

### 981/FUSION：AGE与生产UI direct/RQ融合
改面（均核存在）：`D:/autostack/auto-ui/docs/design/autographics/age-three-milestones-and-ace-m2.md`；`D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/tests/mixed_regions_lifecycle_flow.rs`。
I实现：承合并AGE M2的生产融合线，用现有M1 UI/2D/3D基础与980实际接口融合生产组件/主题/Node；direct/RQ分轴，普通UI宿主不拿GPU业务owner，定义device/epoch/输入路由。原组件/Store/事件/主题/布局/Reload实际兼容义务保留，不等UI058/AG023或新增光影完整终验。
V验收：MIX-05同状态多view不重复tick；UI→3D→UI焦点/捕获/IME正确；direct/RQ行为+GPU ROI一致；device重建不复用旧资源。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。

本轮阶段注记：下面AGE-L01..05完整语法/五view/全UI/复杂生命周期分母不删，A仅MIN-A-AGE所需构造；完整AV在W1，AC在W2，A2R在W3，不能反等完整RUST/PARITY才签A。

### 981/LANGUAGE：完整AGE语法、五类view与传统UI互操作

改面（现有owner）：auto-ui `crates/auto-ui-dialect/src/{ast.rs,grammar}`、`crates/auto-ui-runtime/src/ui`及`experimental/autographics-auto-driver`；语言979 grammar/typed投影、core/query/verify、AV/MIR与新a2x provider。新完整语言fixture/选择器/Frame调度接缝由B-LANGUAGE-I在这些owner下建立；不复制传统组件库或第二前端。

| 完整性族 | M2必需语法/可观察语义 | 关闭责任 |
|---|---|---|
| AGE-L01 声明与行为 | node定义/参数/普通use模块与组件复用，局部model/msg/on、普通函数/类型/表达式；属性/绑定/typed symbol引用、事件payload和独立接收者；不得强迫拆成scene+脚本工程 | SOURCE基础由LANGUAGE扩至AG-A01..09及完整语法构造表，语言owner保唯一绑定/verify |
| AGE-L02 完整普通UI | view使用980现有组件/布局/主题/资源/props/事件/Store等完整能力；传统widget原源无需重写，Node与传统组件能按共同typed接口组合，状态/事件/生命周期不分裂 | 980验传统UI，LANGUAGE/FUSION验新Node普通UI及新旧组件组合；不能只限row/column/text/button/HUD |
| AGE-L03 五view与组合 | view、view-2d、view-3d、view-portrait、view-widget；直属定义是对应出口/变体，普通view内2D/3D区域可同时嵌入；重复直属选择键拒绝，多同域内部区域合法 | LANGUAGE按既有多呈现设计实现，viewport局部环境不复制业务model或Tick |
| AGE-L04 选择与条件 | UI的widget→portrait→默认按用途/逻辑H>W选择，方形走默认；2D/3D按请求领域选择，无隐式跨域fallback；普通if运行时、#if依966编译期；固定目标裁剪真实代码/资源且保共享依赖 | LANGUAGE领域选择，966供CT语义，979供grammar/投影；错误/无候选具名拒绝，不吞程序错误 |
| AGE-L05 生命周期/场景 | Life.Start、固定逻辑Tick与呈现Frame分离；多出口同实例不重复初始化/逻辑或Frame时钟；实例/世界/应用/挂载/共享资源owner分开，场景复用/卸载、动态条件绑定/子实例及late/设备失败恢复保正确 | LANGUAGE联SESSION/RUST，按AG-A04..08；布局/数据发布/GPU不是同一个事务，不承诺外部效果任意回滚 |

I实施决定门：先冻结逐grammar规则/AST变体/领域结构→来源→投影/verify→AV/A2R→UI/图形/资源consumer→正反例矩阵；现有有限profile、两游戏、最小Node均不能代全表。Frame主时钟owner、无可见出口/零尺寸时策略、视图切换的挂载事件/资源策略、#if分阶段检查沿现行规则或形成有界决定后再实现；不能凭“暂不支持”删必需构造，未决项阻完整语言关闭。Frame不是GPU绘制回调，不按view数倍增：在明确定义的测试调度下5 Tick+3 owner Frame周期、两个出口，Start=1/Tick=5/Frame=3。

V验收：V-L-*全部实际普通源输入经共同前门，AV和A2R编译结果分别对独立值/事件/选中view/资源/画面oracle；完整纯UI Node、新旧组件互操作、UI内同时2D/3D及多出口切换成立，诊断回原文件。对已有平台/渲染支持格分别实测；portrait/widget是呈现意图，不据命名声称手机/OS小组件平台已产品化。缺必需语法、条件检查、Frame或传统UI格不得称“完整AGE语法支持”。

R关闭：核AG-A01..09、AGE-L01..05及真实grammar/AST/Spec双向覆盖；删任一构造/必需后端腿/传统UI行必须红。原兼容义务、已批准限额按适用profile保留；新语言不永久限为旧8状态/128行，资源计量/超限必须明确版本化，不能静默提额、删功能或用Rust玩法补逻辑。

### 981/AUDIT：AGE全旧消费者与发行交接
改面（均核存在）：`D:/autostack/auto-ui/experimental/autographics-auto-driver/Cargo.toml`；`D:/autostack/ag-examples/showcases/ui-2d-3d/showcase.json`。
I实现：所有旧driver/pilot/SDK副本/内容工具/安装包/日常tests映射到新producer，分AV执行迁移、A2R两个游戏发行及新增融合证据；交985/986/987候选安装/认证/退役输入，不接管其全生态任务。
V验收：删任一旧间接consumer行、任一必需游戏/后端腿或实际最终ZIP验收行检查红；新日常链及新包无旧执行/发射fallback，两个A2R包来源和获取路径准确。现有迁移不等合并AGE M2新增能力总验；总981仍保FUSION义务，局部交付不冒完整里程碑。
R关闭：独立核真实输入→新producer→consumer、正反例/来源/资源/退出。新adapter/fixture由I建立并接入既有target；缺工具/接口/平台保未执行或阻塞，不改NA。接口变化使受影响旧证据失效。
AGE当前两阶段路线见auto-ui/docs/design/autographics/age-three-milestones-and-ace-m2.md：历史M4/2026-11-07口径M3的融合现并入M2，原验收ID保留。SOURCE/SESSION/CONTENT/RUST/RUST2D/RUST3D不等AGE M2新增能力总验或L984/985才迁旧链；FUSION按既有成熟接口推进。完整981仍须融合，完整AGE M2另须UI058/AG023/AG029的G2、UI080/AG030的G3及AG031联合验收，两者不互充完成。UI067研究结论复用，语言缺口具名关闭，不另搭先证。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。


### PROOF分阶段：A代表性AV与B完整三波（新增验证未实施）

最小前置：A子面取SOURCE正式接口、MIN-A所需SESSION/LANGUAGE及980/NATIVE A/978、最小场景实际用到的UI061–063接面；不要求完整LANGUAGE、RUST或980/PARITY。B完整PROOF按当前波取所需后端和域接口，最终余量全保。实际改面：`D:/autostack/ag-examples/showcases/ui-2d-3d`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：A先按MIN-A-AGE取原showcase来源的真实AV最小场景；原完整同源AV/A2R真实UI+2D+3D验证及新增AC格在B各波建立，复用SOURCE/SESSION/RUST/LANGUAGE与renderer，冻结事件/帧/输入/更新及独立expected；不复制CONTENT的两完整游戏/全Gallery/编辑和离线包义务。
V：两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r11当前）

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

本包所需独立结果：原四图形值/可见性同源；含行为的两游戏还须真实AV与A2R编译/窗口/输入/音频及独立玩法oracle；同实例Start=1/Tick=5/Frame=3与五view选择原表齐。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 正常Node工程/资源→979统一投影verify→SOURCE→SESSION/RUST分别消费接口→LANGUAGE/PROOF→B内容/两游戏/融合/审计。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct AgeProgram { declarations: DomainIr, behaviors: CheckedRefs, views: ViewVariants, assets: AssetClosure, requirements: RequirementSet }
struct AgeInstance { model: OwnerId, logic_clock: TickClock, frames: MountClockSet, generation: Generation }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-ui/experimental/autographics-auto-driver/src/lib.rs](../../../auto-ui/experimental/autographics-auto-driver/src/lib.rs) | 替AutoVM/AutoTask与旧Parser链 |
| [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs) | Node/UI同一领域语义 |
| [experimental/ace/crates/a2x-rust/src/provider.rs](../../experimental/ace/crates/a2x-rust/src/provider.rs) | 新provider真实生成/编译，非旧AST emitter |

**状态、错误与资源。** 未知view/重复选择键、错引用/生命周期body/ABI/旧代输入具名拒；3D无隐式UI fallback，旧VM仍被其他产品编译不证明新链dispatch它。 Start/Tick/Frame与多出口不重复推进model；隐藏/零尺寸/晚事件/资源共享与设备epoch遵冻结规则；十批34任务保留、AUDIT核其余九批。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../age-examples/examples/g01-shapes/g01-main/src/main.at) | 原正常作者源/属性/引用 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `9b809a1c6d99985b` | 拟ace.m2.p981.source01：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/games/platformer-2d/src/main.at](../../../age-examples/games/platformer-2d/src/main.at) | 原2D完整玩法与最终ZIP | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `01b1ce4d34e6cf68` | 拟ace.m2.p981.source02：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/games/platformer-3d/src/main.at](../../../age-examples/games/platformer-3d/src/main.at) | 原3D完整玩法/资源/相机 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `c24f2e86cf5c71f1` | 拟ace.m2.p981.source03：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/showcases/ui-2d-3d](../../../age-examples/showcases/ui-2d-3d) | 混合原showcase，所需普通源与资源闭包完整 | 原项目/工具闭包；T-01核完整源/断言，目录存在不是执行证据；SHA256前16 `目录闭包待冻结` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** SOURCE/SESSION/RUST/LANGUAGE/PROOF仅A能力，B内容/发行/融合需原M2A-08；图形增强/UI080/AG029/031不反置为语言上游前置。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-SOURCE-I`、`B-SOURCE-V`、`B-SESSION-I`、`B-SESSION-V`、`B-RUST-I`、`B-RUST-V`、`B-LANGUAGE-I`、`B-LANGUAGE-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量
| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-age.md` | 分批沉淀实际支持/来源/拒绝及未闭行 | 不把计划当现状 | AC-01..06及所有AC-B-* |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 959行关联本批新producer/consumer与收据 | 替换追踪 | AC-03/05/06 |
| SD-03 | modify | auto-ui仓`docs/specs/auto-graphics/{ace-migration-feasibility,standalone-windows-game-player}.md`（领域owner） | 旧VM实际支持/先证no-go→实际已验证Node直接接ACE、AV开发及Rust静态宿主支持格；未闭项继续列缺口 | 执行与渲染职责、资源和后端身份清楚 | AC-B-SOURCE/SESSION/RUST |
| SD-04 | modify | ag-examples仓`docs/specs/games/{platformer-2d,platformer-3d,windows-portable-release}.md`（内容owner） | 保留原游戏/VM包交付，追加同源AV与各A2R包的真实来源/玩法/包/平台证据；不以计划改现状 | 两游戏逐项交付并可定位制品 | AC-B-CONTENT/RUST2D/RUST3D/AUDIT |
| SD-05 | add | auto-ui仓`docs/specs/auto-graphics/age-author-language.md`（拟新增，领域owner） | 旧有限pilot事实保留；新增实际已验完整Node/UI/五view/条件/生命周期及AV/A2R、传统组件互操作矩阵，未闭格不冒完成 | 完整AGE语言不能被三view/两游戏替代 | AC-B-LANGUAGE、AC-B-FUSION、AC-B-AUDIT |
| SD-PHASE | add | `docs/specs/ace-migration/m2-age.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-PROOF |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-age.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-NODE-CHAIN | add | `docs/specs/ace-migration/m2-age.md`（沿原owner，交付后沉淀） | 独立前门先后错觉→共同Schema Node入口/旧widget别名、按能力扩展与批次来源 | 用户本轮架构澄清 | AC-B-SOURCE/AC-B-LANGUAGE/AC-NODE-CHAIN |
| SD-HANDOFF | modify | `docs/specs/ace-migration/m2-age.md`及实际修改的`m2-schema.md`/`m2-modules.md`（真实交付后、各owner维护） | 960模块基础/979单模块领域接口→实际已验模块领域组合、引用、完整构造与原源CPU交接；显式保留全部未交产品格 | 现有SOURCE承接新集成，不重开960或循环等979终验 | AC-HANDOFF、AC-B-SOURCE、AC-B-LANGUAGE |

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

### 三端最终一致与AC阶段移交（2026-11-16当前优先）

本包适用范围为**UI/AGE领域及原生host/事件回调**。A按MIN-A-AGE及既定普通语言/库/FFI基础验收，完整图形AV/A2R与AC域格明确交B三波，A2PY标准库/AV三provider与原其他required保持。AC基础外剩余差额允许B后延，逐原行ID/输入/独立oracle/资源/错误/源身份记录B-owner与关闭条件，交[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)，异步调度交[1067](1067-ace-ac-native-async-and-task-parity.md)。不能以stub或原profile不支持签AC已实现；M2最终三端必须同平台可用，A后延只是时间调整。A2C完整对齐增强，既有代码/用例/支持事实保留，缺完整面不阻M2。

在本包原能力work/最终验证/review明确区分“A已验接口”“B后延AC行”：移交有正式producer/版本/所需前置，最终真实实现和补验归1068及原owner协作，不回落旧VM或重写归档Plan。985/A可收本包A正式切片，不反等1068；本包如已有B则保唯一树续行，A切片不能勾全包完成。

原NATIVE在历史中主要指AV原生窗口，不能自动当AC证明。本包SOURCE/SESSION/LANGUAGE work任务消费980/NATIVE并向1068交共享领域表示、ABI/typed事件回调/资源与真实产品oracle；1068接AC机器码业务，并按原I/V/R交正式来源。完整状态/事件/挂载/关闭与真实UI/renderer门不缩减，不能只测序列化文件。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/ace-migration/m2-age.md（实际交付） | A所有AC必齐→A基础/具名B移交，最终三端完整，A2C增强 | 用户明确排程 | AC-AC-STAGED |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-NODE-DESKTOP | modify | docs/specs/ace-migration/m2-age.md（真实交付时新建）及原UI/Session Spec | 单列AV/A2R→复用980 A入口/typed宿主，B期1068接AC，FUSION/AUDIT核真实三端产品 | 用户本轮阶段决定，复用现接口不重复建设 | AC-NODE-DESKTOP、AC-AC-STAGED |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AGE-WAVES | modify | docs/specs/ace-migration/m2-age.md（拟交付）及原UI/Session owner Spec | 完整LANGUAGE/PROOF/RUST在A→A最小AV真实场景，B分波完整领域/内容/三端和原游戏包 | 用户本轮三波与A最小出口裁定 | AC-AGE-WAVES、AC-PHASE |

### EVENT-SPLIT拟议规范增量（交付时由原merge任务沉淀）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-EVENT-PHASE | modify（拟议） | `docs/specs/ace-migration/m2-age.md`（原拟新增目标，当前未交Spec） | 交付只沉淀实际完成的A空载荷/声明保真/拒绝、B typed payload/参数/来源与各consumer；明确逐批解锁，r23分母与完整义务保持 | 三缺口具体owner与避免整包循环 | AC-EVENT-PHASE |


## 6. 测试设计

### V-AGE-CAP-HANDOFF（新增分阶段控制，原V全部保留）

按§5原任务冻结实际cap/owner/源/平台与各端oracle：原UI波基线齐且新cap pending可交对应UI-W1/2/3，但不能签cap/AGE全齐；cap合法来源可逐端交，删cap/错源/把AV冒AC或仅emit冒A2R必须红，恢复真实产物可通过。用于新增cap的typed接口须先有独立合法/错误/owner测试，消费方真实GPU/交互/资源/包另验；缺模型/Forge产品不影响基线。AC-AGE-CAP-HANDOFF由本表原owner及最终review核。

### EVENT-SPLIT补充验证（拟议，原任务建立并运行；本轮未执行）

| V / 阶段 / owner任务 | 真入口、输入与独立oracle | 覆盖边界/失败控制 |
|---|---|---|
| V-EVT-A-GRAMMAR-ID / A / T-01＋LANGUAGE I/V/R[A] | 取#6i差额与两条原冻结测试，比较两AGE profile新旧grammar-id/Schema/构造表/源span；其它profile按实际差额实报，非AGE位置的msg/on等普通标识符近邻不误改。核准后维护冻结期望/历史注释和SDK/旧缓存身份，实际重放原两例及相关正反源，独立R/fold。 | 不只更新hash求绿；将源/Schema/producer/SDK错绑旧身份必须被既有门拒绝，两个已报红不能豁免或挪B。本项只验证身份与原语义，不冒完整带参能力。 |
| V-EVT-A-UNIT / A / T-01＋LANGUAGE/SESSION/PROOF I/V/R[A] | 原980 Counter源与Hello双源、g01/Showcase选定MIN-A原源；匿名unit Counter真实0→1→0→0/双实例2:1。补具名unit镜面`msg Msg { Inc }`，业务体不变，源/span/声明名可读且真实结果相同。读取既有Rust测试中的独立expected，不靠worker自报。 | 不替完整原001/002、完整两游戏/五view；镜面是补充控制，不改原源hash或冒原源已全过。 |
| V-EVT-A-ADMIT / A / LANGUAGE I/V/R[A]＋SESSION V[A] | 复用#6i原`msg Msg { Add(str) } .Add(t) -> { .n = 1 }`，再补实际读取`t`与合法unit近邻；错arity/type、重复binder、未绑定名、未知消息/错实例均核原span。当前有参能力未实现时是具名unsupported且零状态/FFI/UI效果，不可因为body不读`t`就成功。 | 已支持有效形态不得退成一般语法错误；合法但未支持和真正坏源分态。静态解析/拼接可保原证据，但不能当动态支持。需在实际行为入口区分，临时错误映射不掩panic。 |
| V-EVT-B-MSG / B-W1 / LANGUAGE I/V/R | 复用vue_capabilities的具名/匿名、quoted/emit声明源与原expected，保类型/声明/引用/源span；补重名/未知变体/ref错误与恢复。 | #6i只语法来源；只数msg变体或拼接数不够，不造第二parser。 |
| V-EVT-B-PAYLOAD / B-W1 / LANGUAGE＋SESSION I/V/R | 补正常Node `msg Msg { Add(int) }`、`.Add(delta) -> { .count += delta }`；两实例依次(i0,+3),(i1,+5),(i0,+4)应[7,5]，且事件轨迹/提交数对应。原cap_event_arg_bool_and_float_literals、cap_emit_tuple_all_payload_types及多载荷/中文str原源/预期按各自支持域接回。 | 改任一实参应只改目标实例；删payload/置零/调换参数/重复求值必须被判错。错type/数量在效果前拒；取消/close/旧世代/撤权、owner释放及修复后同入口绿。多字段driver不得把一次事件实参重复求值。 |
| V-EVT-B-BIND / B-W1 / LANGUAGE＋SESSION I/V/R | 复用cap_unused_handler_still_emitted中的`.QueryInput(v) -> { .query = v }`，真实输入两组不同字符串分别得到对应query；原cap_emit_tuple_all_payload_types等多参数声明/函数签名/实参顺序保持。补param与model同拼写、不同handler同名参数及未知变量负控。 | 常量体原census仍保，但不能代本行；删除binder映射、把v当model或跨实例复用局部必须红，未调用handler亦按原声明语义verify。 |
| V-EVT-B-ORIGIN / B-W1 / LANGUAGE＋980 B-WEB | 原inline-lambda-only及002 onclick lambda/source anchors经共享typed入口有真实body/origin与参数环境；原span→脱糖→checked函数→SFC/body一一可追，浏览器真实事件/DOM对原expected。 | 不用假on块、默认参数、旧TS fallback充ACE；原cap_444_multi_param_emit_bridge与新旧alias源对照保全部义务。 |

现有测试owner复用`core/tests/cases/plan981_v_harness.rs`和plan981注册、`av/tests/cases/plan981_age_package_av.rs`、UI `tests/ace_node_desktop.rs`/`ace_ts_provider.rs`及既有Session bridge，不新增runner。LANGUAGE-I先给V_PENDING_INPUTS逐键分A/B、元数据/拒绝与永久区分性用例；V取实际受影响完整组`ace-source ace-syntax`及必要AV/消费者。AG窗口/GPU按原定向、单实例；完整图形A2R只在W3，非图形共享回归不取消。


### Node桌面接洽补充验证（拟议，未执行）

| V / AC / work owner | 输入/真实入口 | 独立oracle/负控与边界 |
|---|---|---|
| V-DESKTOP-A / AC-NODE-DESKTOP / SESSION、LANGUAGE原I/V | A仅MIN-A-AGE/980 MIN-A接口，完整原001/002/Frame/五view在W1；不要求图形A2R | A先真AV实际UI提交/事件/owner/坏更新恢复；原2:1/Start/Tick/Frame与完整语法/资源oracle在对应B波全部保留，不以byte_len/手写Rust玩法代替 |
| V-DESKTOP-AC / AC-NODE-DESKTOP / B-FUSION-I/V | 1068/T-04正式AC切片、同源原Showcase/混合场景经native runner＋同一宿主，在本包既有受影响GPU/窗口入口实跑 | 原值/输入焦点/捕获/IME、Frame/资源和ROI独立oracle，AC handler机器码实际dispatch；屏蔽AV/A2R guest执行、篡改ABI/代际必须分别证明真AC正常或拒绝，不能退回AV/读stdout绘图充原生产品。 |
| V-DESKTOP-AUDIT / AC-NODE-DESKTOP / B-AUDIT-I/V | 原旧consumer清单＋980 A/981 AV-A2R/1068 AC收据及最终真实包 | 每域/后端/包类型对正式source/producer；缺AC域行、用A2R标签顶AC、只CPU序列化或用Hello冒完整融合须红；既有两个A2R游戏包验收及AGE独立G2/G3/AG031不删。 |

B-SESSION/LANGUAGE按旧authoring_session/mixed_regions_lifecycle_flow等已存在命名target补例，B-FUSION/AUDIT扩同一fixture为AC消费者；新命令/AC product selector由1068/T-04与原owner在I冻结后方可运行，不虚构当前有AC窗口命令。UI沿980的ace-bridge及受影响串行case，语言按真实触面选`ace-source ace-av ace-native ace-a2x ace-abi ace-host`原固定组；AG仅命名受影响case、单实例GPU，原Linux/perf条件保持。原R/T-05独立审核，旧A证据不证明新增AC格。

### r13 SOURCE交接必验（拟执行，复用原V行）

| IF / 原V | work / review owner | 入口与区分性输入 | 独立oracle及覆盖边界 |
|---|---|---|---|
| IF-981-MODULE / V-B-SOURCE、V-NODE-CHAIN | B-SOURCE-I/V建/跑，B-SOURCE-R核并正式交接 | §5两文件普通源→拟包领域前门→共享绑定/verify→既有AV与新A2R真实rustc；seed=0与10各运行，两实例Inc=2:1、4 Tick；再取跨模块Node定义与typed资源 | 6/5、16/15与逐实例文本来自作者行为；模块输入换序/重定位语义不变、改被引用源身份和值变；无emit-only/legacy链，缺导入/私有/环具名拒且原模块span，不拿冷编译失败当语言负例 |
| IF-981-REF / V-B-SOURCE、V-DETAIL-02/03、V-L-COVER | B-SOURCE-I/V/R先交CPU，B-LANGUAGE-I/V/R继承完整面 | 同类型合法声明引用/不存在引用/已声明但不可见引用，通过实际SOURCE绑定/检查及两个后端；失败后修复同入口再运行 | 合法值分别对独立预期；拒绝是具名引用/可见性诊断而非偶然类型错，原span、运行调用数0且无新工件，旧合法产物保全；不能只测host实例下标越界 |
| IF-981-FIELDS / V-L-COVER、V-DETAIL-04 | B-SOURCE-I/V/R交清单/通用适配，B-LANGUAGE-I/V/R完整消费 | AGE-L/AG-A及实际grammar/AST逐项来源与required映射；实际删必需映射、删清单行、坏字段及恢复；与980字段表取有owner的并集 | 两种删除均被真实覆盖/分母检查拒绝，合法近邻保持绿；不能只比较行数；完整语法/五view/GPU/产品格未验仍pending，不以此提前闭LANGUAGE |
| IF-981-ORIGINAL / V-B-SOURCE、V-DETAIL-01、V-NODE-CHAIN | B-SOURCE-I/V/R；979自行回验 | g01原main/resources/shapes实际源闭包→同Schema共同前门→typed描述/CPU读值；改变导入资源、未知属性、恢复 | 保原红圆(-20,10)/直径30、蓝quad(0,0)/30×30、绿圆(25,-15)/直径24、黄quad(-45,-25)/12×12且hidden；颜色/alpha/order按原独立断言和资源原值，错误定位原文件；CPU面不充GPU图像/窗口PASS |

真实受影响语言职责至少核`ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x`并按实际改面补共享lowering/runtime/native消费者；复用现有core/av conformance与`a2x-worker/tests/cases/plan979_age_a2r.rs`的producer→rustc机制，不把旧a2r入口当新链。UI端复用原authoring_behavior/load及source/领域定向case，命令在B-SOURCE-I冻结；窗口/GPU仍按当地AG定向、单实例规则。上述case本轮均未创建或运行；B-SOURCE-R只核实际交付子面，979 T-05另核全AC/V。既有V-B-SOURCE、V-L-*、V-DETAIL-*及全部原游戏/产品验证保持。

拟永久检查键：`p981_domain_packages_two_files`（MODULE/AV+A2R与模块拒绝/恢复）、`p981_instance_reference_visibility`（REF/同型正反控）、`p981_required_age_construct_mapping`（FIELDS/完整表与删项反控）、`p981_original_g01_cpu_projection`（ORIGINAL/原闭包与资源变化）。各键由B-SOURCE-I建立、V实跑、R复核，均绑定AC-HANDOFF及上表原V，979按接口与真实case复核；复用等价既有case时记同义映射，不重复required。名字是规划追踪键，不代表已经存在或运行。

**已实现的同义映射（2026-11-15，用户裁定登记）**：上表四个规划键以下列真实用例承载，不重复计数、不改用例名（§6 允许的同义映射）：

| 规划键 | 实际永久用例 | 所在 crate |
|---|---|---|
| `p981_domain_packages_two_files` | `plan981_age_package_modules`（包图/拒绝/恢复）＋`plan981_age_package_av`（AV 真值）＋`plan981_age_package_a2r`（真实 rustc） | `ace-core` / `ace-av` / `ace-a2x-worker` |
| `p981_instance_reference_visibility` | `plan981_age_ref` | `ace-core` |
| `p981_required_age_construct_mapping` | `plan981_age_fields` | `ace-core` |
| `p981_original_g01_cpu_projection` | **尚未存在**（ORIGINAL 未交付；用户已裁定在本包内继续建 g01 语法面） | — |

979 按接口核对时以上表为准；ORIGINAL 一行在交付前保持空白，不得当作已有覆盖。


### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部命令为未来执行，本轮未运行。cwd前缀lang/ui/man/web/ag-examples分别为本Plan对应唯一tree。永久新fixture由I接到所列target；旧用例绿不能证明新ACE支持。参数尚待冻结的产品命令由I第一步读取现有runner/help与工程配置固定，之前不进入V，不声称未来runner当前已可用。

材料/外置target/生成安装副本只放D:/autostack/plan-artifacts/plan-981/下按仓/批次区分，普通target留树。AG仅受影响文件/case、窗口/GPU单实例；原ignore/平台/资源预算保留。发现/执行/排除、准备/运行/清理/总时与子终态分列，零选择/emit-only/未执行不能PASS。

语言职责组按实际改面选择：SOURCE拟`cargo t ace-source ace-syntax ace-semantics`，SESSION拟`cargo t ace-av ace-host ace-runtime`，RUST拟`cargo t ace-a2x ace-host ace-runtime`，LANGUAGE拟`cargo t ace-source ace-syntax ace-semantics ace-comptime ace-av ace-a2x ace-runtime`；改共享lowering/artifact补`ace-lowering ace-artifacts`，触已迁Rust宿主命名空间加`ace-rust-lowering`。组名静态核test-groups.toml，均未执行；T-01冻结职责/必要消费者并集，不以旧`cargo t a2r`证明新ACE，根无改面不跑根全套。auto-ui只跑受影响tests文件/命名case，窗口/GPU单实例，完整AG留里程碑；实现/各批review/最终独立review同树顺序停写核验，本轮未派agent或运行review。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-SOURCE | AC-B-SOURCE | B-SOURCE-I补新前门case，B-SOURCE-V运行ui现有authoring_behavior target及pilot受影响load用例，按新SOURCE接线 | 多模块普通Node含两个Counter和UI/2D/3D，2:1事件和一次Tick；非法同型事件/错引用报原span，无legacy Parser/手写HIR；typed场景描述与受检行为同源，CPU独立oracle核验 |
| V-B-SESSION | AC-B-SESSION | B-SESSION-I补用例，B-SESSION-V：ui `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test authoring_session --test mixed_regions_lifecycle_flow -- --test-threads=1`；新AV dispatch须实际命中 | 两组件事件2:1、N tick恰N次；旧在途/新代隔离、late不复活、多Run/Stop归零；同PID合法body更新新结果且cargo/rustc=0，坏编辑保旧、修复后恢复，不兼容布局显式处理 |
| V-B-CONTENT | AC-B-CONTENT | B-CONTENT-I扩现有gallery_node_authoring.py、showcases/ui-2d-3d/tools/verify.py/package.py和两游戏verify入口，B-CONTENT-V沿AG025原门与新AV选择 | 原M2-01..10/G-M01..09及实际变体保留；仓外AV冷包GPU/音频/输入/编辑，原游戏玩法/资源oracle逐项映射；新AV结果按2D/3D供同源A2R对照，无旧VM fallback |
| V-B-RUST | AC-B-RUST | B-RUST-I在既有a2x/standalone_game_player target接最小Node→Rust→编译→AGE宿主fixture，B-RUST-V跑语言职责组和该命名case；不是当前已有命令的能力声明 | 同源实例状态/初始化/事件/Tick/绑定进入生成Rust并实际dispatch；Node修改改变编译后结果；真实GPU/输入/音频和关闭释放，独立oracle；禁VM provider仍运行 |
| V-B-RUST2D | AC-B-RUST2D | B-RUST2D-I将下表V-R2-*、V-R-PACK/V-R-FAIL接入原2D verify及release工具，B-RUST2D-V运行最终ZIP | 原G2-01..09逐行、同源AV对照、独立oracle、物理试玩、资源/包闭包及无解释器执行证明齐全，未运行不计PASS |
| V-B-RUST3D | AC-B-RUST3D | B-RUST3D-I将下表V-R3-*、V-R-PACK/V-R-FAIL接入原3D verify及release工具，B-RUST3D-V运行最终ZIP | 原G3-01..08逐行、同源AV对照、独立oracle、物理试玩、资源/包闭包及无解释器执行证明齐全，未运行不计PASS |
| V-B-LANGUAGE | AC-B-LANGUAGE | B-LANGUAGE-I在既有领域/authoring target接V-L-*，B-LANGUAGE-V经正式前门/AV与A2R编译运行，命令在I冻结 | AGE-L01..05及AG-A01..09/grammar表完整，五view、纯UI与传统组件、条件/Frame/多出口/资源/诊断全部闭合，不用最小样本或已交游戏抵消 |
| V-B-FUSION | AC-B-FUSION | B-FUSION-V，须LANGUAGE及980实际结果；原RQ/UI061–063入口保条件，mixed_regions_lifecycle_flow等受影响case、真实双host错峰 | MIX-05同状态多view不重复tick；完整普通Node UI与传统组件的绑定/主题/事件/Store及UI→3D→UI输入/IME、direct/RQ行为/GPU ROI、device资源正确；不能仅限基础控件 |
| V-B-AUDIT | AC-B-AUDIT | B-AUDIT-I扩迁移/包/完整语言核对，V审构建闭包、实际dispatch及其余九批正式复审/来源收据（含PROOF）；987全生态门仍归987，本包范围不后推 | 删旧consumer、完整语法/传统UI行、任一后端腿/游戏最终包行即红；语言/AV/两A2R候选及融合来源齐，无旧执行/发射fallback |

### 完整AGE语言必验（均owner_stage=work，B-LANGUAGE-I建用例、V实跑）

| Case | AC / 输入 | 入口与独立预期 | 边界 |
|---|---|---|---|
| V-L-UI | AC-B-LANGUAGE；纯UI Node复用生产表单/列表/主题/路由/Store及传统组件，新旧组件双向状态/事件组合；无2D/3D也可运行 | 980已交原组件加同源普通Node工程，经共同前门AV与A2R编译后真实UI运行；原绑定/输入/主题/卸载oracle成立，两App不串、两实例2:1事件；传统源原样入口仍可运行 | 不能只有HUD/row/column/text/button或Rust手写控件业务；平台/模式按980原支持格 |
| V-L-VIEW | AC-B-LANGUAGE；五view、widget+portrait重叠、缺专用回退/方形、请求3D但无3D、重复直属键、view内多个2D/3D区域 | 同源AV/A2R分别核独立选择表、错误span和实际呈现/输入；widget优先、portrait次之、默认末，3D无UI隐式fallback；多个内嵌区域合法且同Tick/资源正确 | viewport环境独立；真实挂载/切换不以AST/CPU枚举值单独充画面证明 |
| V-L-LIFE | AC-B-LANGUAGE；同实例两个出口、5逻辑Tick/3 owner Frame周期、view切换/卸载/新实例和晚事件 | 普通on处理器的独立trace：初次Start=1/Tick=5/Frame=3，切view不重Start、不复制model；退出挂载释放自身资源，共享owner仍合法，late不写新代；AV/A2R分别核值/trace/资源 | 主时钟/隐藏与零尺寸策略先决定，不把Frame当绘图回调或按view倍增 |
| V-L-COND | AC-B-LANGUAGE；普通if改变model后选分支，#if固定目标与共同资源/分支专属依赖、同型非法条件/引用 | 共享检查按966分阶段规则报原span；AV/A2R实际分支/事件相符，固定排除分支从目标代码/资源清单去除，共享依赖仍在；非法源不能被renderer/Rust忽略 | 不执行运行时值冒编译期，不承诺未选分支可藏任意非法源码 |
| V-L-COVER | AC-B-LANGUAGE/AC-B-AUDIT；逐构造grammar/AST/投影/双后端consumer对账与最小跨模块+新旧UI+2D/3D+资源组合 | 既有覆盖工具由I扩字段，实际删除一行/禁用必需producer后检查红、合法近邻绿；修复后同入口真Run，冷包/来源身份正确；各完整语法格有实际值/事件/资源或诊断证据 | 只统计数量/最小fixture通过不够；未知/缺环境保pending，不能退回有限profile冒完整支持 |

### 两游戏A2R必验明细（全部拟执行，各自I任务建立可运行用例）

所有格均为owner_stage=work；B-RUST2D-I/V负责V-R2-*，B-RUST3D-I/V负责V-R3-*；V-R-PACK/V-R-FAIL两游戏各跑一次完整规定链。共同前提是迁移后的同一作者源/关卡/资产hash，输入序列/固定dt与原oracle不变。两端相等只是对照，还须独立expected防共同错误；数值与像素/PCM容差沿现行Spec，不为过关放宽。

| Case / AC | 区分性输入与状态 | 实际入口 / 关闭证据与预期 | 边界 |
|---|---|---|---|
| V-R2-RULE / AC-B-RUST2D | 2D原固定地图、左右/单跳/禁止二段跳、巡逻敌人、三不同币一次收集/销毁、缺币先到终点、危险/掉落、胜败冻结、fresh Enter与held不连开 | 现有`games/platformer-2d/tools/verify_game2d.py`由本批扩至AV/A2R同源运行，逐项原G2-01..09和独立轨迹/事件/终局断言；AV、A2R各自满足expected | 同一正式游戏，不能靠关掉巡逻/少收币/改地图/手写Rust规则凑绿 |
| V-R2-MEDIA / AC-B-RUST2D | idle/run/jump/fall与facing、相机夹界、jump/coin一次声音、n/3和胜败HUD | 2D原checks/oracle+最终A2R运行采样：对应状态像素/帧、声cue/PCM及HUD齐；真实键鼠分别玩出失败→新局、获胜→新局并听见音频，正常退出 | 不能只跑inspect、预录轨迹、离线PCM或旧AG028试玩；原资源/许可和运行预算不减 |
| V-R3-RULE / AC-B-RUST3D | 3D相机相对四向/对角归一、orbit/zoom夹界、单跳/地墙顶支撑、三币、早到终点不赢、危险/越界、终局fresh Enter | 现有`games/platformer-3d/tools/verify_game3d.py`由本批扩至AV/A2R，逐项原G3-01..08及独立位置/事件/状态oracle；地/墙/顶和相机反例保留 | 3D独立关闭，不能用2D或最小宿主fixture替代 |
| V-R3-MEDIA / AC-B-RUST3D | GLB+外部纹理、idle/walk/jump与朝向、jump/land/coin一次cue、n/3与终局HUD | 3D原checks/oracle+最终A2R采样，像素/动画/PCM/HUD及实际键鼠、鼠标、音频；失败→新局和获胜→新局，正常退出 | 模型纹理URI闭包和原素材/许可保持，不用Rust宿主按胜负另画HUD或补动画 |
| V-R-PACK / AC-B-RUST2D、AC-B-RUST3D | 每个最终release ZIP在无仓库/工具链/缓存/网络的Windows环境冷解压，换cwd及空格中文路径；连续游玩、resize、失焦/最小化/恢复、停止与再开 | 扩现有`tools/package_game_release.py`/`verify_game_release.py`：明确a2r后端身份后打包并校验，逐ZIP SHA、逐资源hash、许可/DLL清单与PE/链接闭包；同源AV对照及原次数/生命周期门，Run代次隔离、late不复活、资源归零、渲染不重复Tick | 各交exe+资源ZIP及解压/启动/操作方法；Windows x64实际OS/GPU/驱动/音频记录，不承诺未经测平台；源码包/AV包不代二进制包 |
| V-R-FAIL / AC-B-RUST、AC-B-RUST2D、AC-B-RUST3D | 共同ABI/签名或generation故障；各游戏缺件/篡改asset或manifest；一次编译失败/取消及修复；改一个真实作者业务参数再编译 | B-RUST-I做公共故障fixture，各游戏I接入release controls；源码非法在共享前门报原span，目标编译/链接故障与源码错误分列；坏候选拒绝且前一完整候选不变，恢复缺件/修复后同入口真Run；新exe结果随源码变化。查生成函数→链接map/dispatch并在无VM执行provider配置运行 | A2R包无运行期AutoVM/AV解释循环/脚本fallback；静态grep或单独Rust文件不足；失败路径不得跳过实际负控后写PASS |

具体未来命令不得虚称已支持：现有打包入口为`python tools/package_game_release.py --game <2d|3d> --player <exe> --out <scope>`，校验入口为`python tools/verify_game_release.py --package <zip> --out <scope> --controls`及既有`--qa-chain/--play-win`。当前合同绑定旧player，直接换exe不够；B-RUST-I联合两内容I先补新后端身份/构建输出接缝，各游戏I冻结正式a2r构建命令、打包schema/参数、对应AV命令、固定输入、期望与最终包路径后V才能运行。真实Windows/GPU/音频、物理操作时段及编译工具链是执行条件，缺少须逐游戏记pending和owner，不能用超时/缺设备充条件PASS。

预算沿原场景/资源/公共服务单位，AV新执行计量与旧VM steps分列并在SOURCE/SESSION冻结映射；不能删旧上限、按组件复制配额或把Rust发行宿主进程内存混作GPU资源账。性能需分前端/生成/Rust编译/加载/逻辑Tick/渲染，不把A2R编译时间算AV body更新，也不为新目标捏造性能提升承诺。

最终T-05核全批收据适用性、遗漏、组合并集和原资源门；证据身份不变可复用，不反复全仓。每批独立复核不替整包final review，未闭in-scope阻execution_done。


### A期PROOF验证（本轮未运行）

| Case | AC | 实际/拟补入口 | 独立oracle |
|---|---|---|---|
| V-A-PROOF | AC-A-PROOF | 原authoring_session/mixed_regions_lifecycle_flow及showcase真实入口复用；A只V-AGE-A-MIN真AV，完整原AV/AC/A2R分别在W1/W2/W3，参数由I核help冻结 | A最小场景/输入/更新/原span/恢复/关闭真实；原两个局部实例2:1、Tick/Frame不倍增及完整GPU/ROI/资源oracle在B按波关闭，headless/首帧不能代全产品 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 原四图形值/可见性同源；含行为的两游戏还须真实AV与A2R编译/窗口/输入/音频及独立玩法oracle；同实例Start=1/Tick=5/Frame=3与五view选择原表齐。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 未知view/重复选择键、错引用/生命周期body/ABI/旧代输入具名拒；3D无隐式UI fallback，旧VM仍被其他产品编译不证明新链dispatch它。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | Start/Tick/Frame与多出口不重复推进model；隐藏/零尺寸/晚事件/资源共享与设备epoch遵冻结规则；十批34任务保留、AUDIT核其余九批。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |
| V-NODE-CHAIN | AC-B-SOURCE/AC-NODE-CHAIN | 从979/H0的正式Node接口扩源，而非新建parser；可观察同Schema/投影/verify来源及原span；关闭/篡改Schema不得靠旧Parser继续执行，拒绝前无host调用，修复后真Run | B-SOURCE-I/V/R；首Hello不关闭完整SOURCE，真实CPU/宿主/AV/A2R沿原适用格 |
| V-NODE-UI | AC-B-LANGUAGE/AC-NODE-CHAIN | 同一旧UI case的原widget兼容源、声明头迁移Node源与混合场景复用组件；相同操作独立核值/事件/布局与不重复Tick，普通on/view标识符不被当领域节点，错误span保各自原源 | B-LANGUAGE-I/V/R；FUSION/AUDIT仍验原完整direct/RQ/资源/交互，不用Hello代全门 |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### SOURCE 交接件（实测，供 979 独立回验）

> **2026-11-16 更新（HEAD `2b172901b`，行 53）**：以下全部按本提交复跑过。

**g01 原源身份可核对**：`fixtures_g01/main.at` 的 SHA256 前 16 位 = **`9b809a1c6d99985b`**，
与 §5 要求的原源值**逐字一致**；`resources.at` = `6f993775477ebc39`（790B），
`shapes.at` = `dbdc5243ec590dab`（540B）。

**979 可直接跑的四个命令**（本提交实测结果在括号里）：

```
cd D:/autostack/.wt/lang-981/auto-lang/experimental/ace
cargo test -p ace-core --test conformance plan981                        # 147/147
cargo test -p ace-av --test conformance p981_age_package                 # 3/3
cargo test -p ace-a2x-worker --test conformance p981_age_package         # 2/2（真实 rustc）
cargo test -p ace-syntax-grammar                                        # 45/45
```

**四个 IF 的对应证据**：

| IF | 主要用例 | 覆盖 |
|---|---|---|
| MODULE | `plan981_age_package_modules`（9）、`plan981_age_package_av`（3）、`plan981_age_package_a2r`（2）、`plan981_multimodule_identity`（7） | 两文件闭包、跨模块 helper 改值传播、两实例 6/5、改 helper→16/15、AV 与真实 rustc 各有独立值；`symbols_by_module` 保留同名 helper 两个模块、场景元素解析到声明模块（含 `pub use` 链）、模块换序身份不变、模块环具名拒 |
| REF | `plan981_age_ref`（12） | 同模块免导入；跨模块**必须从声明模块导入**（含 `pub use` 链）；四类具名诊断互斥：undeclared / invisible（带声明模块与修复建议）/ not-a-node / **ambiguous**（同名多模块未导入）；诊断带原模块 span；修好导入后同入口恢复（轮 49 修 R47-1） |
| FIELDS | `plan981_age_fields`（10） | 必需清单 **12 行**（含 `view-2d`/`view-3d`），**分母冻结**（键集钉死）；逐行删投影反控覆盖全 12 行；纯场景 node 不冒充 UI `view`；合同必需项落在表内或显式范围外清单（L04→LANGUAGE，AG-A→980）；合法近邻绿 |
| ORIGINAL | `plan981_original_g01`（15）、`plan981_resource_identity`（8）、`plan981_model_field_types`（6） | 三文件原源闭包、四图形坐标/尺寸、颜色与 alpha 逐值、声明序、`visible:false`、资源来源与改值传播、坏 kind/未知资源具名拒（**span 锚定原 token**）、原源 **content hash** 钉死、恢复；资源构造的稳定 `op` + 逐位 `args`、场景资源引用解析；`model` 字段类型两面一致（int/bool 可编，float/f32 在类型 token 具名拒；str/Str 合法（980 面）） |

**已知边界（979 回验时须知，均不当作通过）**：

1. 含**模块级 `const`/`var`** 的包图会在 `SemanticView::from_checked` 被 `module.global.*` 拒（RUST 批）。
2. **g01 不能进 A2R**：补 `fn main()` 后报 `a2x.unsupported-expr: intrinsic age.material2d.color.v1 is outside the compute surface`。V-DETAIL-01 的 CPU 读值由描述面 `row.cpu` 满足（AV 腿对**没有 text 叶**的组件不执行函数——轮 48 实测 g01 的 AV 观察为 0 个函数，979 不要期待 g01 的 AV 执行证据）。
3. 模块级 `let` 只推断"资源服务初值"；其它形状具名拒绝。
4. **五 view 只实现 3 个**：`view-portrait` / `view-widget` 在 grammar 层不存在。
5. 非 `tint`/`material`/`mesh` 名字的资源型组件参数（如 `color: red`）报 `source.age.scene-value` "cannot type yet"（具名拒绝，非静默）。
6. GPU 图像/窗口/输入、platformer-2d/3d、ui-2d-3d 属 SESSION/RUST/LANGUAGE/PROOF。
7. 资源构造的**非字面量实参**在 `args` 里是 `Unresolved`（位置保留），此时 `components`
   为空——消费者不得按位置读局部列表。`resource_refs` 是引用解析面：979 回验应以它
   判定"指向哪个声明"，不要用 `AgeSceneValue::Symbol` 的末段拼写做名字匹配。
8. resync（`25293cded`）后两点已固定：① `str`/`Str` 是合法 model 字段（980 面，状态函数返回
   `Str`），但其**初值的 CPU 面仍是 `Unresolved`**（只有 int/bool 有值；author Expr 仍保留在
   `field_inits`）；② age-package **接受并描述** 980 的 widget 成员槽位（`msg`/`design`/
   `named-view`/`bind`/`routes`/`expose`/`lifecycle`/`computed`/`watch`/`timer`）——R52-1 的
   "接受但不描述"已由 B-LANGUAGE-I 反转为"接受且结构化投影"（§9 切片 1–6）；`#if`/L04 选择仍归
   966/LANGUAGE-V，不是缺口。当前 grammar 身份（B-LANGUAGE-I 后）：age-package `c152355d…`、
   age-component `0e799dc1…`（msg payload 修复 + watch 多行/多源）、age-h0 `64915bd1…`；前两项已由
   `plan981_grammar_scope` 钉住，邻接面 13 个冻结值不变。
9. `pub node X(params…)`（带参数的 pub node）在当前 grammar 不可解析，且误报为
   `source.unsupported`「`pub` is not part of this source profile」——属既有边界（`age-pub-node-item`
   没有参数槽位，resync 前即如此）；g01 的 `Disc` 是非 pub + 参数，IF-981-ORIGINAL 不受影响。
   修复 owner：B-SOURCE-I/LANGUAGE（若完整语法要求 pub node 带参数）。

**治理状态**：六次独立复审中，轮 48 判 `needs_fix`（R47-1 blocker）→ 轮 49 修复 → 轮 50 **phase pass**
（四个交接键在 r13 + `291f2b5cb` 上通过；R50-1 计数已改）。后续 979 的独立回验（收据 `fe1fbeb74`、
`75cee3baa`）在 `f94a29293` 上判 **V-09④ / V-10 / V-11 / V-DETAIL-01 四行 PASS**（V-11 带具名边界：
L04/AG-A 为范围外=AC-08 残留）。**AC-HANDOFF 仍不关闭**：981 尚未 phase fold 落主，按 979 §5
不得以分支 tip 当正式输入；落主后须按落主 SHA 复核。该 phase pass **不**授予 `reviewed`、不合并、
不勾 B-SOURCE-I/V/R。

**身份**：分支 `plan-981-dev` @ `2b172901b`（三次 resync merge，第二父 master `80870e748`；前置 `25293cded`/`a242bd645`，SOURCE 原基线 `ca88cde14`）；领域 `ace.age.package/v1`；源 profile `auto.source.age-package.draft`；脱糖前缀 `__age_p_hidden_`；HIR 域 `TextRecord`。

**入口**（唯一前门，不新建 binder/verify/解释器）：

```rust
pub fn compile_age_package(inputs: &[PackageModuleInput], entry: &str,
    alias: LegacyWidgetAlias, capabilities: &[String])
    -> Result<AgePackageCompilation, Vec<Diagnostic>>;
pub fn compile_age_package_with_scenario(/* + &ComponentScenario */);
```

`AgePackageCompilation { description, checked: CheckedModule, symbols, symbols_by_module, input_hash, desugar_hash, atom_text }`；
`symbols` 按**源名**索引（多模块同名有损，仅兼容）；`symbols_by_module` 的键是 `(模块全键, 源名)`，消费者用 `symbol_of_in(module, name)` 取跨模块 helper。
`AgePackageDescription { domain, grammar_id, alias, entry, components, modules, resources }`。
`AgePackageComponent { name, head_span, legacy_widget, fields, handler_msgs, handler_assigns, field_inits, field_init_cpu, public, initialized, params, refs, resource_refs, exit: AgeViewExit, texts, module, scene }`；
`AgeViewExit::{View | View2d | View3d}` 是**作者写的 view 出口**（M1 每 node 一个；由后缀 token 字节判定，未知后缀具名拒 `source.age.view-suffix`；`ComponentView.role` 恒为 `view`，不要拿它判出口）；
`AgeResource { name, module, port, op, arity, args: Vec<AgeResourceArg>, components, symbol }`；
`AgeSceneRow { kind, element: AgeSceneElement, name, attrs: Vec<(String, Expr)>, span, cpu: Vec<(String, AgeSceneValue)> }`（声明序 = 2D 绘制序）；
`AgeSceneElement::{Shape2d | Element3d | Node{module,node}}` 是场景行**解析后**的元素目标（`Disc` 在两个模块各有一份时不会按名字撞车）；`pub use` 重导出链解析到最终声明模块。
`AgeResolvedResourceRef { row, attr, spelling, target: AgeResourceTarget::{Resource{module,name,port} | Param{name,port}}, span }`（979 用它解析资源引用，不用 `cpu` 的末段符号）。

**IF-981-FIELDS 的 requirement 分母**（V-11）：`AGE_CONSTRUCTS` 覆盖 AGE-L01/L02/L03/L05 的
node 侧（12 行）；`AGE-L04`（条件/选择）与 `AG-A01..AG-A09`（UI 组件矩阵）登记在
`AGE_CONSTRUCTS_OUT_OF_SCOPE` 并具名真实 owner（981/LANGUAGE + 966；980 IF-980-FIELDS），
由 `p981_fields_requirement_split_is_exhaustive` 机械核对"只能落在表内或范围外清单之一"——
这是显式范围，不是删行把分母变小。

**能力键**（缺失即具名拒绝，实测不可绕过）：`ac.source.modules.packages/v1`、`age2d.resource`。

**命令与实测结果**（本提交，逐条按原文复跑过）：

```
cd experimental/ace
cargo test -p ace-core --test conformance plan981              # 147/147
cargo test -p ace-av --test conformance p981_age_package       # 3/3（AV 运行值腿）
cargo test -p ace-a2x-worker --test conformance p981_age_package  # 2/2（真实 rustc 腿）
cargo test -p ace-syntax-grammar                               # 45/45
```
`ace-core` 678/678（+79）；本批改面无编译警告（`ace-core (lib test)` 一条
`unused import: PackageAssemblyContext` 与 `ace-av` 的 30 条 test 警告均为 HEAD 存量，
不在本批改面）。

**grammar 裁剪作用域是不变量、不是声明**：`plan981_grammar_scope` 冻结了 14 个邻居面
（Core…AgeH0）的 grammar id；任何波及会立即变红并报出 profile 名。该断言已用"故意改坏
冻结值 → 变红 → 恢复 → 变绿"证明是真门。

**命令本身踩过的坑（写进来免得复查者重犯）**：`cargo test -p ace-av p981` 会跑该 crate 的
**其它**测试二进制并报 "N filtered out"，看起来像用例不存在；必须显式 `--test conformance`
才命中 `plan981_age_package_av` / `plan981_age_package_a2r` 的用例。我第一版交接件写错了
这条命令，复跑时发现并改正。

**具名未闭项**（979 回验须知，不当作通过）：① 含模块级 `const`/`var` 的包图会在 `SemanticView::from_checked` 被 `module.global.*` 拒（属 RUST 批；现有 A2R 用例输入不含此类全局）；② 模块级 `let` 只推断资源服务初值，其它形状具名拒绝；③ 场景属性类型面见 §7 V-DETAIL-01 行；④ GPU/窗口/输入、platformer-2d/3d、ui-2d-3d 属 SESSION/RUST/LANGUAGE/PROOF；⑤ 仍缺一次**独立上下文**复审。

**⑥ 2026-11-16 补测发现的真边界（此前从未验证）**：**g01 原源不能进 A2R**。给 g01 补一个 `fn main()` 后走
`compile_age_package` → `SemanticView::from_checked`，被具名拒绝：
`a2x.unsupported-expr: intrinsic `age.material2d.color.v1` is outside the compute surface`。
即 **A2R 的 compute surface 不含 AGE 资源服务 intrinsic**。这属 RUST 批的接面义务，不是 SOURCE 能在此修的。
影响：V-DETAIL-01 的"CPU 读值"目前由 AV 腿满足；A2R 腿只覆盖 counter（不含资源构造器）的形状。
**不得**据此宣称 g01 双后端闭合。

**不变式**：单一前门；前端不复制（非 AGE profile 的 grammar id 字节不变）；端口为名义类型不压平；任何未支持构造具名拒绝不静默丢弃；AGE 门在 `AgePackage` 下解析/检查/发射。

| V / AC | 原owner/输入 | 独立预期与边界 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原work及最终验证：A真实基础/AV-A2R结果＋完整AC后延账，缺owner/基础stub负控；B回验真AC | A基础真实且B每行去向/关闭条件齐才可交A；不得记AC PASS/NA或从分母消失；B真实三端与原独立oracle齐才关闭后延行。 |

### 最小A与B域格验证（拟议，未执行）

| V / 原owner | 关闭输入与oracle / 移交边界 |
|---|---|
| V-AGE-A-MIN / SESSION、LANGUAGE、PROOF的I/V/R A部分 | 普通多模块Node/helper→共享Schema/verify→AV＋980 UI-FFI→最小UI/2D/3D真实呈现；独立值/ROI、一次事件/更新、原span/坏更新保旧/修复及关闭owner。只AV，不要求完整五view/全UI/A2R；缺实际GPU/源/真实接口仍NOT_RUN，不能假签A。 |
| V-AGE-WAVE / 各波SESSION/LANGUAGE/PROOF/FUSION原I/V/R | W1补完整AGE-L/AG-A语法/绑定/五view/Frame/实例/输入/资源的AV适用格；W2与1068补AC；W3以RUST和980新A2R补。原Start=1/Tick=5/Frame=3、定向2:1与完整旧独立oracle保持，不以共同错误三端相等通过。 |
| V-AGE-TRANSFER / AUDIT原I/V/R | 原source/check/构造/游戏/包/域格逐项对应A或W1/W2/W3；删一required行、用AXR0/byte_len或仅CPU顶完整域、拿A2R包当AC须红。完整游戏/G2/G3和真实OS独立验收不消失，也不反卡UI波次收据。 |

新增case由各I在现有authoring_session/mixed_regions_lifecycle_flow及对应产品owner建永久命名例，公开后端选择/真实命令由原I核help冻结，未建入口不得报告已跑。旧V-DESKTOP-A/V-A-PROOF/V-L-*按本节分账：A只MIN-A，完整格迁当前B波；AV不跑图形A2R生成。GPU单实例/AG受影响例、语言固定职责组及Linux/perf原条件保持。

## 7. 验收标准

- [ ] **AC-AGE-CAP-HANDOFF / work**：V-AGE-CAP-HANDOFF及§5本包实际producer/consumer逐cap义务闭合；UI三波与新图形cap来源分开、无循环、原验收不减，新cap不成为985/A新增前置。阶段R/fold不冒整包final。

| ID | owner_stage | 标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/平台/输入oracle完整 | T-01及各I |
| AC-02 | work | 原整包能力经真实源码/工程运行，范围不减 | 全部I/V |
| AC-03 | work | 全consumer接线与权限/资源/恢复、无legacy fallback，跨仓身份准确 | 全部V及959行 |
| AC-04 | review | 批次及全包独立复核，原AC反例/health/Spec均核 | 各R及T-05 |
| AC-05 | merge | 正式来源/Spec/归档/清理完成 | T-06/07 |
| AC-06 | work | 原T-08补充义务分配至批次且全闭 | 所有AC-B-* |
| AC-B-SOURCE | work | Node场景与逻辑共同进入979/唯一verify，局部实例/模块/三view源与原span保真，typed领域描述和行为供两后端共用，无旧world.at/Parser生成重解析、手写HIR或Rust玩法替代 | B-SOURCE-V/R；V-B-SOURCE |
| AC-B-SESSION | work | AV真实运行：事件2:1、N tick恰N次，generation/late/资源正确；同PID有效body更新新结果且cargo/rustc=0，错误保旧/修复恢复，不兼容布局显式处理 | B-SESSION-V/R；V-B-SESSION |
| AC-B-CONTENT | work | 两游戏/Gallery/混合与原全部变体迁AV，原M2-01..10/G-M01..09及游戏分母保留；编辑/离线冷包真实GPU/音频/输入，无旧VM；逐游戏同源结果供A2R对照 | B-CONTENT-V/R；V-B-CONTENT |
| AC-B-RUST | work | 同源Checked HIR经新A2R→Rust真实编译链接并执行AGE端口，实例/事件/资源语义与AV一致且满足独立oracle；无AutoVM/AV执行循环/旧Trans，错误/取消保旧且修复后真Run | B-RUST-V/R；V-B-RUST、V-R-FAIL |
| AC-B-RUST2D | work | 原完整2D游戏同源AV/A2R及独立oracle、G2-01..09/音画/生命周期不减；最终Windows x64 A2R exe+资源ZIP仓外冷启动/真实胜败重开/无工具链/无解释器执行，完整来源/许可/hash/启动说明 | B-RUST2D-V/R；V-B-RUST2D、V-R2-RULE/MEDIA、V-R-PACK/FAIL分别落2D证据 |
| AC-B-RUST3D | work | 原完整3D游戏同源AV/A2R及独立oracle、G3-01..08/相机/音画/生命周期不减；最终Windows x64 A2R exe+资源ZIP仓外冷启动/真实胜败重开/无工具链/无解释器执行，完整来源/许可/hash/启动说明 | B-RUST3D-V/R；V-B-RUST3D、V-R3-RULE/MEDIA、V-R-PACK/FAIL分别落3D证据 |
| AC-B-LANGUAGE | work | AG-A01..09及AGE-L01..05完整Node作者语法/语义经共同前门，五view/选择/if/#if/Start/Tick/Frame、多实例/场景/绑定/typed资源与传统组件互操作在AV/A2R实际执行正确；逐grammar/AST/产品格无遗漏，无旧前门/解释器fallback | B-LANGUAGE-V/R；V-B-LANGUAGE、V-L-UI/VIEW/LIFE/COND/COVER；必需项未决不许声称完整支持 |
| AC-B-FUSION | work | 完整普通Node UI与传统组件/主题/Store/事件及2D/3D在生产direct/RQ接通，MIX-05不重复tick，焦点/捕获/IME、GPU ROI、device资源正确；不以基础控件/场景替完整UI | B-FUSION-V/R；须980完整结果与LANGUAGE组合证据 |
| AC-B-AUDIT | work | 全旧消费者、完整语法/传统UI互操作、AV及两A2R候选逐项交接；删必需构造/后端/最终包行即红，无旧执行/发射fallback；交985/986/987且不反向等待 | V-B-AUDIT及V-L-COVER，I/V核其余九批全部required收据（含PROOF）；本批R完成后十批齐交T-05 |
| AC-A-PROOF | work | A只V-AGE-A-MIN，真实AV代表性Node多模块/最小UI-2D-3D/事件/更新关闭；原PROOF完整2:1/Frame/全域/三端oracle逐项在B波关闭，不反卡A | B-PROOF原I/V/R按阶段独立phase，V-AGE-A-MIN及V-AGE-WAVE |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |
| AC-NODE-CHAIN | work | 复用979/H0与980共用组件/宿主，Node首Hello后按接口扩场景；位置/AGE上下文双条件及旧widget同Schema兼容有真实检查，无第二前端/业务模型且不反等980整包才开始 | B-SOURCE/LANGUAGE-I/V；V-NODE-CHAIN/UI与原全部V/各R |
| AC-HANDOFF | work | 原960多模块领域缺口由SOURCE实际实现，IF-981-MODULE/REF/FIELDS/ORIGINAL各有永久源/真实CPU或双后端证明、原span/身份/负控/恢复与正式来源；979可独立回验全部原义务，生产不依979归档，原完整SOURCE/LANGUAGE/游戏与A/B门不减 | B-SOURCE-I/V/R、§6交接必验；B-LANGUAGE继承完整面；979 V-09/10/11/V-DETAIL-01 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED闭合，AC基础真实、剩余差额完整移交1067/1068/实际owner，AV/A2R及原其他A-required不减；最终三端未验不得冒支持。原work落实、原独立review核，无新增任务或本轮完成数。

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-NODE-DESKTOP | work | V-DESKTOP-A齐并复用980正式AV宿主；V-DESKTOP-AC/AUDIT在B证明真正AC guest＋同宿主的完整适用领域格。A不等待AC，整包不能用HELLO/AXR0或1068 CPU格代真实融合/发行 | B-SESSION/LANGUAGE/FUSION/AUDIT原I/V/R、T-05独立final |

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-AGE-WAVES | work | A的V-AGE-A-MIN真实且正式可交；完整AGE-L/AG-A/旧内容/资源/错误由V-AGE-WAVE分波与V-AGE-TRANSFER逐项关闭，第三波后全后端完整原游戏包/产品义务齐 | SESSION/LANGUAGE/PROOF/FUSION/AUDIT各原I/V/R，原T-05 final |

### EVENT-SPLIT验收补充（owner_stage沿原work/review/merge，待实施）

| AC | 原承接任务 | 可观察关闭条件 |
|---|---|---|
| AC-EVENT-PHASE | T-01、B-LANGUAGE/B-SESSION/B-PROOF各I/V/R[A/B]；980消费者；985接收 | §5三缺口逐阶段/真实source/oracle/owner/现有任务可追；A真实正例/保真/安全拒绝齐，B支持/执行/资源/错误与完整原用例全部闭；语法/splice/历史或worker总绿不越级。对应本节V-EVT-*，独立review绑定revision与正式来源；当前规划不得勾PASS。 |



## 8. 执行步骤

当前共34任务，勾选仅按整任务计（仍 3 项：B-SOURCE-I/V/R）；A 子面交付见下方台账，不提前勾整任务。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### A最小与共享任务（完整余量按B波关闭）

**A 子面交付台账（2026-11-17，985 r26 要求；勾选仍按整任务，不提前勾）**

| A 子面 | 状态 | 绑定 revision | 复审 |
|---|---|---|---|
| SOURCE I/V/R | 已交（整任务已勾） | 979 核验落主 + `plan981_original_g01` | SOURCE 批复审 |
| B-LANGUAGE-I（含 event A） | 已交 | auto-lang `24b4ba4a7`/`75015a49b`、auto-ui `771be6a9` | review-62、review-65→66 |
| B-LANGUAGE-V[A]（V-L-* / V-EVT-A-*） | 已交 | `0b98c143b`/`230e4abc6`、`c286fb8d0` | review-67→68、review-69 |
| B-SESSION-I | 已交 | auto-ui `15102c48`→`4c629db0` | review-64 |
| B-SESSION-V[A]（单组件双实例/关闭恢复） | 已交 | `332ef407`/`0a87676b`、`9032d69c` | review-67→68、review-69 |
| B-RUST-I 公共链 + V[A] | 已交 | `eb487f3cf`/`399edd447`、`0b98c143b` | review-67→68、review-69 |
| **B-PROOF-I/V[A]（2D/3D 呈现事实 + UI 事件/更新/关闭）** | 已交（A 面） | auto-lang `ad7882cbe`/`6340934a1`/`0520c7f3b`、auto-ui `1eb27697`/`ab6fff5c`、overlay `dd4cf388`/`ab6fff5c`（1395 文件） | review-70 needs_fix→修补→**review-71 pass** |
| T-01（A 决定/源/命令冻结） | 已交 | 本台账 + §9 收据 | — |
**本轮执行入口**：A先复用已交SOURCE，只做MIN-A-AGE所需SESSION/LANGUAGE/PROOF的真实AV接口与验收，不等待完整RUST/五view/PARITY或976 WEB。三波在M2A-08后按§5执行；W1的980组件接口与981 AV领域格按组协作正式交接，981不先等完整UI-W1-AV才贡献同一收据。W2只等完整W1，W3只等W2正式切片。下面每个原任务只登记一次，跨阶段任务仅子面交付不勾整项。

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01/AC-PHASE**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。 EVENT-SPLIT：先核980/NATIVE正式源/原测试oracle，将V_PENDING_INPUTS逐键拆A必需与B全量；三个消息缺口归本包现有LANGUAGE/SESSION，缺件不泛称980未归档。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。

- [x] **B-SOURCE-I｜M2-A｜owner_stage: work｜AC-B-SOURCE/AC-02/AC-03/AC-06/AC-NODE-CHAIN/AC-HANDOFF**：按§5先落实IF-981-MODULE/REF/FIELDS/ORIGINAL的有界最长路径决定、模块图×领域组合、实例引用/完整构造表/g01原源CPU与永久正反例，消费已交979基础，不等其归档；auto-lang在本任务维护所需共享接口。继续完整SOURCE，领域/行为共同verify，保f32/数组/bool/实例/原span及旧生成骨架全部公开义务，不保旧重解析链。子面正式交接不提前勾整任务。

- [x] **B-SOURCE-V｜M2-A｜owner_stage: work｜AC-B-SOURCE/AC-NODE-CHAIN/AC-HANDOFF**：按§6交接必验运行四个IF的永久用例、真实AV/A2R和g01原源CPU面、缺/私有/越权/删映射/坏包/修复反控，绑定源/图/Schema/目标/资源与子终态；继续原V-B-SOURCE完整职责/必要集成，未验不得改NA或提前勾全批。

- [x] **B-SOURCE-R｜M2-A｜owner_stage: review｜AC-04**：同树停写独立核本批AC/diff/反例/跨仓来源；可先逐四个IF核子面并按原phase fold正式交979，留API/grammar/闭包/后端与实测来源，完整SOURCE全部原义务闭合才勾本任务；失败回I，不冒整包或979验收pass。

- [ ] **B-SESSION-I｜M2-A最小/B分波｜owner_stage: work｜AC-B-SESSION/AC-02/AC-03/AC-06**：按§5 SESSION替旧AutoVM为AV+typed宿主，保Tick/回调/资源/更新/停止恢复，补同PID body编辑cargo/rustc=0及坏编辑保旧用例；与RUST分写面并行，不手写Rust玩法。 按NODE-DESKTOP消费980的正式typed UI/事件/资源接口，建立V-DESKTOP-A；向1068交ABI与生命周期契约，不等待AC桌面。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A复用unit宿主/源并补最小资源/恢复；B接typed payload队列/dispatch_with_args的实际等价接口，沿967/968/970不造新FFI模型。

- [ ] **B-SESSION-V｜M2-A最小/B分波｜owner_stage: work｜AC-B-SESSION**：运行V-B-SESSION及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A事件/关闭/错误真实验证不等B三件，B另核载荷值、世代/权限/所有权和恢复。

- [ ] **B-SESSION-R｜M2-A最小/B分波｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：IF-981-EVENT-A与B支持分件审查，不用I/语法替V；正式phase落主供下游。

- [ ] **B-RUST-I｜M2-B-W3｜owner_stage: work｜AC-B-RUST/AC-02/AC-03/AC-06**：按§5 RUST接新A2R/provider与typed AGE静态宿主及978构建，语言owner关闭实际实例状态/初始化/调用的目标合法化缺口；建V-B-RUST和V-R-FAIL公共正反例，明确生成代码实际dispatch与禁VM验证；复用通用系统，与SESSION并行。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST-V｜M2-B-W3｜owner_stage: work｜AC-B-RUST**：运行V-B-RUST/V-R-FAIL公共部分和冻结语言职责并集，实际生成→编译→执行GPU/输入/音频；记录值/事件/资源、身份和故障恢复，emit-only或VM补逻辑不通过。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。 EVENT-SPLIT：完整本项为B-W3，不反置成MIN-A-AGE/A签章前置，旧真实rustc切片只复用原范围。

- [ ] **B-RUST-R｜M2-B-W3｜owner_stage: review｜AC-04**：同树停写独立核新A2R实际链、宿主资源/失败及无解释器执行证据，pass只关闭公共RUST批，不替两游戏。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-LANGUAGE-I｜M2-A最小/B分波｜owner_stage: work｜AC-B-LANGUAGE/AC-02/AC-03/AC-06/AC-NODE-CHAIN/AC-HANDOFF**：继承SOURCE已交IF-981-REF/FIELDS的引用规则/完整构造表，按AGE-L01..05及AG-A01..09扩完整语法/AST/投影/consumer分母；复用979/980，接五view/选择/if/#if/Frame/新旧组件与完整可见性组合，先闭主时钟/挂载/策略决定；永久V-L-*源/命令/span/oracle冻结，不复制前端/控件库，不要求979归档才允许所需接口消费。 复用IF-980-NODE-DESKTOP-A，领域构造/源身份与AC移交沿NODE-DESKTOP；不另建UI执行线。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：承担§5三缺口的声明/handler保真、A拒绝/实际必需最小子面与B真实payload/绑定；复用#6i，不重开979。B producer可先phase交980，不能等其完整WEB/AV验收。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。

- [ ] **B-LANGUAGE-V｜M2-A最小/B分波｜owner_stage: work｜AC-B-LANGUAGE/AC-NODE-CHAIN**：A取最小SESSION及980已交正式A源，实跑本阶段适用V-B-LANGUAGE/V-L-*；B逐波补完整V，当前波后端对独立语义、资源、真实UI/图形oracle，核删除必需构造/后端腿负控；未决/缺环境保pending，不用游戏或最小view样本替代。 本批V-DESKTOP-A的所需入口/事件格必验，AC桌面只移交B，原AV/A2R完整语言范围保持。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：先按V-EVT-A-UNIT/ADMIT只验A真实所需，完整V-EVT-B-*在相应B波验；不得以V-B-RUST全量/980整包作A门。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。

- [ ] **B-LANGUAGE-R｜M2-A最小/B分波｜owner_stage: review｜AC-04**：同树停写独立逐AG-A/AGE-L/实际grammar与产品格核源码→受检语义→当前阶段/波后端→实际consumer（A只AV，B最终各端齐），核传统UI及多呈现/Frame/条件分期；缺任一必需格不能关闭，pass只关闭本批。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A/B分别核AC-EVENT-PHASE和正式来源，A phase不勾完整LANGUAGE。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。

- [ ] **B-PROOF-I｜M2-A最小/B分波｜owner_stage: work｜AC-A-PROOF**：A用原showcase来源正常Node建立真实AV最小UI+2D+3D验证，B各波再补完整同源AV/AC/A2R组合，复用SOURCE/SESSION/RUST/LANGUAGE与renderer，冻结事件/帧/输入/更新及独立expected；不复制CONTENT的两完整游戏/全Gallery/编辑和离线包义务。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A只MIN-A-AGE AV组合，不要求完整RUST公共链/WEB/PARITY；实际源用到的事件能力仍须先真补。

- [ ] **B-PROOF-V｜M2-A最小/B分波｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。 核子终态与身份，未执行不PASS。 A桌面按980正式AV＋Rust FFI入口验；原已交A2R证据只复用其范围，新增完整图形A2R留W3、AC接入留W2，不反卡M2A-08。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A仅本阶段真实AV/UI-2D-3D及原错误/资源oracle，完整图形A2R留W3，普通语言/A2R原义务不取消。

- [ ] **B-PROOF-R｜M2-A最小/B分波｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。 本轮只将MIN-A-AGE AV子面交985/A，完整余量由V-AGE-WAVE在对应B波关闭；A部分R/fold不提前勾整任务，不等待RUST/PARITY/AC。 EVENT-SPLIT：A真实组合＋EVENT-A独立复审/fold交985，不等M2A-08或980全包。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-CONTENT-I｜M2-B｜owner_stage: work｜AC-B-CONTENT/AC-02/AC-03/AC-06**：按§5 CONTENT迁两游戏/Gallery/混合/全部profile至AV与新SDK，保编辑保存/SaveAs/搬迁/关开及旧间接链义务；按游戏提供同源AV收据，制品明确分型，不将AV包认A2R。

- [ ] **B-CONTENT-V｜M2-B｜owner_stage: work｜AC-B-CONTENT**：运行V-B-CONTENT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-CONTENT-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-RUST2D-I｜M2-B-W3｜owner_stage: work｜AC-B-RUST2D/AC-02/AC-03/AC-06**：消费RUST和原2D源/资源/已交工具，补既有verify/packager的新后端身份与V-R2-*、V-R-PACK/FAIL；真实A2R构建release并产最终exe+资源ZIP/来源/许可/说明，复用AG028而不重建玩法。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST2D-V｜M2-B-W3｜owner_stage: work｜AC-B-RUST2D**：对应AV证据齐后跑V-B-RUST2D及全部V-R2-*、共享V-R-PACK/FAIL的2D腿；逐原G2-01..09、独立oracle、仓外最终ZIP真实试玩/故障恢复、无VM执行逐项记录，缺条件保持pending。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST2D-R｜M2-B-W3｜owner_stage: review｜AC-04**：同树停写独立核2D完整链和最终ZIP SHA，旧AG028/AV结果不替新A2R，全部必需格闭合才pass本批。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST3D-I｜M2-B-W3｜owner_stage: work｜AC-B-RUST3D/AC-02/AC-03/AC-06**：消费RUST和原3D源/资源/已交工具，补既有verify/packager的新后端身份与V-R3-*、V-R-PACK/FAIL；真实A2R构建release并产最终exe+资源ZIP/来源/许可/说明，保GLB纹理闭包；无需等2D验收完成。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST3D-V｜M2-B-W3｜owner_stage: work｜AC-B-RUST3D**：对应AV证据齐后跑V-B-RUST3D及全部V-R3-*、共享V-R-PACK/FAIL的3D腿；逐原G3-01..08、独立oracle、仓外最终ZIP真实鼠标/键盘/音频和故障恢复，不能拿2D或headless绿替代。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-RUST3D-R｜M2-B-W3｜owner_stage: review｜AC-04**：同树停写独立核3D完整链、相机/音画/资源闭包与最终ZIP SHA，所有必需格闭合才pass本批。 图形新增A2R实现/验证在UI-W2-AC正式后；旧已验A2R能力复用，完整包和原oracle不减。

- [ ] **B-FUSION-I｜M2-B分波｜owner_stage: work｜AC-B-FUSION/AC-02/AC-03/AC-06**：承合并AGE M2融合线，依现有M1基础和980成熟接口接生产组件/主题/Node，保原兼容义务和direct/RQ/device/epoch/输入路由；不等待UI058/AG023或新增光影终验。按§5接永久fixture并冻结命令。 承接AC-NODE-DESKTOP，与1068/T-04按A期同版宿主接AC，建V-DESKTOP-AC；不等待1068全包或AAC完整代际。 AV/AC/A2R域格分别按W1/W2/W3协作，局部正式交接不等全包；整项最终才收全部。

- [ ] **B-FUSION-V｜M2-B分波｜owner_stage: work｜AC-B-FUSION**：运行V-B-FUSION及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 同时闭V-DESKTOP-AC适用真实AC窗口/域语义/事件/资源与负控，AV/A2R结果不能代替。 AV/AC/A2R域格分别按W1/W2/W3协作，局部正式交接不等全包；整项最终才收全部。

- [ ] **B-FUSION-R｜M2-B分波｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。 独立核真实AC guest dispatch及原三端域格，CPU/字节探针/旧A2R包不充本批AC。 AV/AC/A2R域格分别按W1/W2/W3协作，局部正式交接不等全包；整项最终才收全部。

- [ ] **B-AUDIT-I｜M2-B｜owner_stage: work｜AC-B-AUDIT/AC-02/AC-03/AC-06**：全旧消费者及全部十批新producer（AUDIT-I/V先核其余九批，含PROOF）/完整语言/传统UI/双后端/最终两ZIP逐项对账，补删除必需行负控；向985/986/987交候选/身份/实际证据，不以后继全生态门替本包AC。 新增AC正式consumer/后端/包身份映射由V-DESKTOP-AUDIT核，复用1068正式切片，不重复编译器/渲染系统。

- [ ] **B-AUDIT-V｜M2-B｜owner_stage: work｜AC-B-AUDIT**：运行V-B-AUDIT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。 须闭V-DESKTOP-AUDIT，原双后端条款按r15补真实AC适用格；缺AC实现/领域产品证据不能pass。 同时核V-AGE-TRANSFER；原双后端表述按三端最终合同补齐AC，OS实际产品证据只消费OS064，不复制实现。

- [ ] **B-AUDIT-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

### 2026-11-18 stage: handoff（**PLAN-980 → PLAN-981：`style` 声明面文法会签请求**；980 侧材料已备、**未动文法**）| 请求方 PLAN-980 r18 | outcome: **待 981 回签（或修订提案）**

**来源**：980 §5「P3 提案：`style` 声明面」（`docs/plans/980-ace-m2-autoui.md`，P3 节；提交 `231aedf44`，后续 `6bbb0fd8a` 已把 grammar ID 更新到 spawn/f32 后的基线）。**请求**：981 作为 grammar/identity owner 回签，回签后 980 才落文法与投影。

**提案摘要（详见 980 §5 P3 节，不在此重复全文）**
- 语义：`style` 不引入新 HIR 域，是 **`str` 常量/函数语法糖**——`style N = "s"` → const；`style N = other` → const 引用；`style N(p: str = d) = "{p} …"` → 函数（`{p}` 转 ACE f-string）；`pub style` → 公共 const/fn；带参调用仅在**实参全为字面量**时投影期折叠，否则具名拒。**默认值本刀不支持**（具名拒），因此 stylekit 的 3 条参数化 recipe 本刀**不解锁**。
- 文法：新增 `style-item` + `public-body` 的 `style` 臂；profile 门 `[age-component, age-package]`；`style` **不进 token 表**（只在声明头按 spelling 识别，普通标识符位置保持合法）。
- 身份：**移动两个 grammar ID**——现基线（spawn/f32 后）`age-component`=`19a1376e…`、`age-package`=`0290d0ab…`；其余 13 profile 必须逐字节不变。
- 实测：20/35 例含 style（126 行，125 字面量 + 1 复制 + 1 带参调用；0 参数化声明、0 `pub style`）；stylekit 7 条 pub（含 3 参数化/带默认值）；移除 style 后下一阻断异构（payload msg/`$`/model 字段/槽位）——style 是第一阻断但不充分。

**请求 981 回签的四点**
1. 认可 `style ≡ str const/fn` 映射与 profile 门；
2. 认可两值 grammar ID 同批重基线，并**指定做法**：980 在文法提交内同步更新 `plan981_grammar_scope.rs` 冻结值（注明前值，沿用 #6m/#6o 先例）**或** 981 在自己的提交里更新（请明确择一）；
3. 确认 const/fn 命名空间与现有身份/可见性契约无冲突；
4. 确认 `source.age.style-defaults` / `source.age.style-call` 两个新诊断码的归属与命名（981 若有命名规范冲突，回修订）。

**回复方式**：在 981 本文件 §9 追加一条带日期的回签（或修订）收据，引用 980 P3 节与上述四点；若有修订，逐点说明。**本请求不要求 981 现在改任何代码/文法**；回签后由 980 落文法 + 投影 + 永久用例，并在同一提交附 981 冻结值更新（若按做法 A）。


### 2026-11-18 stage: handoff（**F32 消费同窗 v1：981 侧已备 `17fafb1d9`（分支隔离，未合入）**；分工/呈现/验收/桥面按用户裁定落档；待 980 `auto-ui-runtime` owner 回签后开窗）| PLAN-981 r18 | outcome: **981 ready；合入窗口未开**

**用户裁定（2026-11-18）**：同意同窗方案——981 负责 `ComponentFieldValue`/`InitialValue`，980 负责 `AceStateValue`/状态桥/TS 类型/呈现；两边先**配对提交联合编译验收**，再**同一窗口**合入；呈现按 Rust `f32` 默认 `Display`（可往返、与地区无关）；验收用真实 AGE 状态源 `1.25` 经一次更新到 `1.5`，逐项核对种子/状态读回/界面显示；回签须写明所验的桥（**本窗=组件桥**；包桥 F32 另案）。**该口径尚未由两仓 owner 正式回签。**

**981 侧交付（分支 `plan-981-f32-window` @ `17fafb1d9`，基 master `216a79385`；`plan-981-dev` 与 master 未受影响）**
- `ComponentFieldValue::F32(f32)`：**按位相等**（`to_bits`；NaN payload 可判别、`-0.0 ≠ 0.0`），`Eq` 因此保留——auto-ui 无需为 f32 去掉 `Eq`，两仓语义同源。
- `InitialValue::Float(f32)`：props 种子；脱糖字面量走 `TypedLit{ty:Float, bits}`（按位，无十进制往返）；`validate_initials` 只接受 Float↔f32（`Int` 种子不隐式提升）。
- 窄观察通道 `observe_component_states` 映射 `f32 → F32`（与 `execute_component_def` 同一条执行链，不造第二条）。
- **验收用例**（`plan981_f32_surface.rs` **6/6**）：源内初值 1.25 + Tick(+0.25) → `F32(1.5)`；props 种子 `Float(1.25)` 同结果；`Int` 种子具名拒（`component-unsupported` + "must be `f32`"）。980/981 家族 av **50/50**、core **230/230**。
- **不改**：`execute_component_def` 宽值出口保留；文本叶 f32 仍拒；`f64` 不在面。

**980 侧待办（回签时逐项确认）**：①`AceStateValue::F32(f32)`（相等/`Eq` 策略按位对齐 981，避免两仓分叉）；②`ace_state_bridge::From<&ComponentFieldValue>` 与 initial 映射（`AceStateValue → InitialValue::Float`）；③`ace_native` 种子路径；④`ace_ts_provider` 类型串 `"f32"`；⑤`ace_node_desktop` 读状态/渲染（Rust `Display`）/数值输入四处穷尽匹配；⑥**组件桥**验收：种子 1.25 → Tick → 读回 1.5 → 界面文本 `1.5`（不是 `1.500000`、不用逗号小数）；⑦包桥 F32 用其自己的用例——现包桥只观察整数、无 props/initials，**不**纳入本窗回签。

**窗口协议（双方）**：①配对提交各在自己分支；②980 以 `AUTO_LANG_ROOT`（或同组 sibling）指向 `17fafb1d9` 联合编译+验收；③双方确认后**同一窗口**合入：981 先合（ace_av 单仓）→ auto-ui 必须紧随（否则 auto-ui master 的穷尽匹配红，这是窗口的预期代价，不得拖过窗口）→ 两侧各跑冻结门（981：`ace-source ace-av`；auto-ui：其固定组）→ 任一侧失败即**双方同窗回滚**，不留半状态。
**未合入声明**：`17fafb1d9` 只在 `plan-981-f32-window` 分支；**980 回签前不得 fold 到 master**（否则 auto-ui master 立即编译红）。窗口开启条件 = 980 回签 + 其 auto-ui 提交 SHA + 双方门禁命令。

**UI058 侧（用户同轮裁定）**：T-02 已在 r6 完成；`spawn` 文法与 f32 AV/A2R 已交付，UI058 可写真实 `render2d.spawn(...)` 与 f32 移动业务源，用 `execute_component_def` 核对 AV 数值；T-04 待 T-03 与所用正式接口齐备后跑正常 Node 整链（其 T-04 宿主端口请求见下一条收据）。

### 2026-11-18 stage: handoff（UI058 T-04 阻塞请求：AGE GFX 宿主执行端口 `IF-981-GFX-HOST-PORT`）| PLAN-981 r18 | outcome: **request（未实施；归 981 SESSION/LANGUAGE 后续任务）**

**来源/决定**：UI058 T-04 入口核对发现“正常 Node 源 AV 整链”当前无法执行；用户 2026-11-18 裁定路线 A=回 981 owner 补 SESSION/typed 宿主端口（UI058 不先建私有 runner/旁路）。T-03 已交（`auto-ui` commit `7165dbad`，PLAN-058 r7/3-10）；T-04 等待本请求的正式来源。

**缺口证据（只读）**：`lowering/src/hir_adapter.rs:5082` 对 `AGE_GFX_SERVICE_OPS` 的 `AgeGfxLane::HostAlgorithm`（15 op）与 `HostIdentity`（`image2d.png`）在 AV 观察车道具名拒 `lower.age-gfx-lane`；服务表 17 op/6 族/7 端口与 32 credits 已签（`ddbda0228`+`ed8b16e01`）。`experimental/age-ace-session/src/age_session_core.rs`（driver/bridge 共用）只做 AGE package 编译/调度/观察，无 gfx 宿主派发；AV 可执行面仅 `frame2d.at`+端口字段读。UI058 T-03 的中立单元是 driver 旧 API profile `autographics.topdown2d/1`，未接 AGE typed 调用。

**请求交付（981 侧）**：
1. **AV/SESSION 宿主派发**：HostAlgorithm/HostIdentity 调用点不再一概 `lower.age-gfx-lane` 拒；lower 成受检宿主调用（或 SESSION 派发），保留 import 身份/能力准入/mir_verify 面；无 provider 注册时仍具名拒零效果。
2. **typed 签名/端口**：严格取 `AGE_GFX_SERVICE_OPS` 与 `AGE_PORT_FIELDS`（Cast2D/Sweep2D/…），不新增第二套拼写/价目。
3. **provider 注册/生命周期**：SESSION 面注册 provider（每 Run 一个，Stop/Restart 归零），token/失效/世代语义沿 021；`execute_component_def` 观察与宿主派发是同一受检产物。
4. **预算**：宿主收费接口按 UI058 联冻口径（累计扣费；下一次扣费后累计>32 具名 fault 10、零效果、本 Tick 后续收费调用保持拒绝；按 Tick 重置；attempt 无回退；fault 终止 Run）；端口字段读 0 credit。
5. **身份**：`image2d.png` 的 HostIdentity（内容 SHA256+len+IHDR）由 host 提供或明确归 host，AV 不伪造。
6. **永久用例**：同一最小 AGE 源（sweep/segment/row/input/render2d/tick.dt）在 registered fake provider 下 AV 真跑出 typed 结果；未注册/错签名/错 kind/超预算/越界具名拒；改源结果随动；f32 `x=x+vx*dt` 与 gfx 同 Tick 组合。给出实际命令/exit/结果/耗时。
7. **交接**：交付 commit/SHA、provider 注册签名/ABI、支持域与拒绝码表，UI058 据此实现 provider（T-04）。

**补充（T-04 预备实测 2026-11-18，`auto-ui` commit `b071634c`；`t04-prep.md`）**：
- **G1（阻塞源侧声明）**：AGE `shape2d` 属性白名单（kind/position/size/visible/material/image/frame/flip）**无** render-only/非阻挡/碰撞层声明位，未知属性具名拒；T-03 的 `layer` 只在 UI058 driver 旧文档面。请在本端口交付中二选一并冻结：①`shape2d` 属性端口（如 `collision: false` 或 `layer: render-only`，进 `attr_port` 白名单并投影到宿主 typed 行描述）；②由受检场景/资产携带 per-row layer 事实（非 UI 私设）并给 typed 读取面。要求同一事实在 AV/AC/A2R 同源可用；否则 UI058 的正常 Node 源无法声明"可见不碰撞地面"。
- **G2**：`on Life.Tick` 体是 assignment-only（`age-on-assign`），自然写法（一次 sweep + 分支 + 同 Tick 呈现）不可表达；请给 hook-local `let`/memoized 结果，或书面确认 `pub fn`+模块 `var` 为正式写法。
- **G3**：行引用只有 i32 声明序 ordinal，无 `self.row`/typed 行句柄；请确认 T-04 可用 ordinal 冻结（重排即语义变化），或给 typed 句柄。
- **G4**：`render2d.frame(i32,str,i32,i32)` 的 `str` 是资产路径还是 `Image2D` 名端口，请以永久用例钉死。
- **G5**：UI058 的 `.sdk-source` pin 仍旧（无 seam/f32）；T-04 第 1 步刷新 pin，请随端口交付给准确 auto-lang commit/overlay 身份。
- **附带请求**：请以 `auto-ui` 树（`codex/plan-058-dev` @ `b071634c`）的 `experimental/autographics-auto-driver/fixtures/plan058/topdown-node/main.at` 做一次前门 parse/typecheck（可先用 master 前门），回报诊断/最小修正；该源是 T-04 的正式输入，若语法不合法请直接给可编译形态。

**边界**：不改 981 原有 17 op/价目/文法；不要求 A2R/AC 同时接；UI058 不先建私有 runner/旧生成器旁路。UI058 侧记录见 `auto-ui/docs/plans/058-age-topdown2d-queries-and-sprites.md` §9/§10 D-06。


### 2026-11-18 stage: work（**981 `render2d.spawn` 文法小切片**；用户裁定"先交 spawn 文法小切片，保留原拼写并同步两个 AGE grammar ID"）| PLAN-981 r18 | outcome: **pass** | code_commit `0e6c03c3d` | merge `8f44bbf08`

**交付**：`postfix-step` 增 AGE 两面的 `spawn` 成员位专用臂（`dot r-spawn`，**token 表零新增、021 拼写不动**；scheduler 的 `Task.spawn` 原臂保持）。`age-component` id `f35cf593…`→`19a1376e…`、`age-package` `2d1e0547…`→`0290d0ab…`（`plan981_grammar_scope.rs` 同提交具名重基线；其余 12 邻居与 `age-h0` 逐字不变）。
**永久用例**（`plan981_gfx_seam.rs`）：正例 `render2d.spawn("assets/x.png", 4.0, 6.0)` 命中 `age.render2d.spawn.v1` 的 typed i32 结果；2 参调用仍 `source.age.gfx-arity` 具名拒——证明走的是**真实服务声明**，不是"任意方法"放行。
**门禁**：`cargo t ace-source ace-syntax ace-av` = ace-source **PASS 864**、ace-syntax **PASS 159**、ace-av 1390/1391（红 `sys973_v03_process_spawn_capture_timeout_exit` = `ExecutionWall`，并发负载；隔离重放 13.1s 绿；同族 `D-1037-02` 已登记）。981 家族 core **230/230**、AV **29/29**。
**边界**：只开成员位；非 AGE 面仍拼不出 `.spawn`；服务价目/符号表未动。**UI058 现在可以写真实 `render2d.spawn(...)` 业务源**。

### 2026-11-18 stage: work（**981 f32 A2R 同源腿**；用户裁定"随后补 f32 A2R 同源实测"）| PLAN-981 r18 | outcome: **pass（`D-981-AGE-F32-SURFACE` 核心条件闭合）** | code_commit `a4950f0b5` | spec `8f6566d05`

**A2R 独有的三道闸（AV 车道不经过，故 AV 先绿）**
1. `SemanticView` 的 text-record 族把 `NumericConst` 与 typed `Num` 算子挡在 compute surface 外（该豁免此前只给 Closures/numeric 面）——f32 字面量/运算在**视图**层具名拒（`a2x.unsupported-expr` / `a2x.unsupported-op`）；
2. record-text 文件集**不带** `RT_NUMERIC`（只有通用 `assemble_file_set` 与 collections 族带）——生成代码引用 `a2x_lit_f32`/`a2x_div_f32`，rustc 期 not found（58 处）；
3. wrapper 入口合同仍是 `A2xResult<i32>` / `value:<i32>`（不新增 f32 入口合同）。

**修法**：①`SemanticView::op_admitted` 把 text-record 拆出并放行 `BinOp::Num{ty:F32}`（**只 f32**；f64/宽整型仍拒，语义合法性仍由前门 HIR verifier 先行）；②同一处放行 `NumericConst{ty:F32}`（`kind_name` 补 NumericConst/Cast，便于以后诊断不再显示 `non-core-scalar`）；③`record_text_runtime_source_with` 追加 `RT_NUMERIC`（与通用/collections 族一致）；④入口用 f32 状态 + 比较折叠成判别性整数（**不**扩 wrapper 合同）。
**证据**（真实 rustc，`plan981_age_package_a2r.rs` 新用例）：两次 Tick 后 `x=3.0`、`y=x/2.0=1.5`，入口 `x>2.5 && y==1.5` → `value:7`；**变异控制**同源只把 `vx` 3.0→1.0 → `x=1.0/y=0.5` → `value:0`（非常量折叠/打印端伪造）。同文件既有 3 例（双实例 6/5、改 helper 16、module globals 5）全绿。
**门禁**：`cargo t ace-source ace-syntax ace-av ace-a2x` = ace-source **PASS 864**、ace-syntax **PASS 159**、ace-av 1390/1391（`sys973_v03` ExecutionWall，隔离绿）、ace-a2x 316/317（`aggregates_965_control_effects_c` = `CToolError("cl -? spawn: Deadline")`，隔离单例 4.4s 绿）——两条红均为工具链/机器负载族，**不充"无红 PASS"**。
**债务**：`D-981-AGE-F32-SURFACE` **关闭**（三处面 + AV/A2R 各一条真跑齐）；跨仓消费窗口/呈现移出为 `D-981-F32-CONSUMER-WINDOW`（见下）。

### 2026-11-18 stage: handoff（F32 消费端**同窗**合同：状态读 + 呈现；待 auto-ui/UI058 回签后单窗口落地）| PLAN-981 r18 | outcome: **proposal（未实施；登记为 `D-981-F32-CONSUMER-WINDOW`）**

**为什么必须同窗**：`ComponentFieldValue` 是 auto-ui `AceStateValue` 的输入源，两者都靠**穷尽匹配**转换；任何单侧先加 F32 变体都会让另一仓编译红。因此 ace_av 的枚举扩展必须与 auto-ui 的四处匹配、TS 能力类型、呈现格式在**同一个合并窗口**完成。
**改动清单**：①`ace_av::ComponentFieldValue` 增 `F32(f32)`（建议 `PartialEq` 而非 `Eq`，NaN 按 IEEE；若必须 `Eq` 则改按位承载）；`InitialValue` 增 `Float(f32)` + `validate_initials` f32 分支；窄观察通道映射到 F32。②auto-ui：`AceStateValue::F32`、`ace_state_bridge::From`/initial 映射、`ace_native` 种子、`ace_ts_provider` 类型串 `"f32"`、`ace_node_desktop` 读状态/渲染/数值输入四处。③呈现：f32 → 最短往返十进制（Rust `{}`），不本地化；**文本叶呈现 f32 单列**（原子编码 i32 面）。④证据：ace_av 枚举/种子单测 + auto-ui 读-显示-种子用例 + 一条真实 AGE f32 状态源（UI058 T-02 先证路径）。
**不需要等它的事**：UI058 今天可用 `execute_component_def` 取 `Value::F32` 做先证；spawn 与 f32 语言面均已落主。**回签请求**：auto-ui/UI058（或 980 包感知桥 owner）确认窗口与呈现格式后，981 侧在同一窗口提 `ComponentFieldValue`/`InitialValue` 的改动与单测。

> **更新（2026-11-18）**：本 proposal 已由上方 **F32 消费同窗 v1** 取代——981 侧在分支 `plan-981-f32-window` @ `17fafb1d9` 备好（按位相等/`Eq` 保留、`Float` 种子、验收 1.25→1.5 三条），待 980 `auto-ui-runtime` owner 回签后开窗；呈现拍板为 Rust `f32` `Display`。

### 2026-11-18 stage: work（**981 f32 AV 切片**；用户裁定"优先排 f32 的 AV 切片，覆盖注解、model 字段和同型运算"）| PLAN-981 r18 | outcome: **pass（AV 面交付；`D-981-AGE-F32-SURFACE` 保持 open——A2R 腿与消费端枚举按原条件）** | branch `plan-981-dev` | base `b6ab516e6` | code_commit `be8480cae` | merge `6deef24b6`（并入 980④-A 首片 + 982/1026 前进后 ff 落主）

**范围**：f32 注解位、model 字段、同型四则/比较/一元负号，AV 真值执行；**不动** `AGE_GFX_SERVICE_OPS`，不改 980④-A 的宽数值边界（那是 `D-④-3` 的"宽数值保持拒"——本切片是**语言面 f32**，不是组件宽数值字段准入），不扩 `ComponentFieldValue`。

**交付**
- 字段域：`field_type` 增 `f32`/`float` → `TypeWord::Float`（两 AGE 面同一码；状态函数返回 f32）。`float` 与 `f32` 同宽（`resolve.rs:143`），不是第二宽度。
- 表达式面：`rename_expr` 放行 `ExprKind::TypedLit`——浮点字面量恒为此形；此前落 `_` 分支被当"未接线表达式"拒，**模型初值与运算右操作数都进不来**（这是比文法更靠前的真阻塞）。
- 文法：`ann-ty`/`ret-ty` 增 age-package 的 f32 臂（fn 参数/返回/`let` 注解；`float` 不跟进）；`age-package` grammar id 具名重基线 `6c1bb477…`→`2d1e0547…`（`plan981_grammar_scope.rs` 同提交更新；13 邻居与 `age-component` 逐字节不变）。
- Atom 描述符：`text-record.atom` 的 BinOp 增 11 个 f32 case + 分支（add/sub/mul/div/rem/lt/le/gt/ge/eq/ne），UnaryOp 增 `neg_f32`。复审发现 adapter 早已发 `NegNumeric(F32)` 而描述符未登记 ⇒ `x = -vx` 在绑定期 `bind.unknown-case`——**真实缺口，同批修**并进用例。该描述符由 TextRecord/AgeH0/AgeComponent/AgePackage 共用，属**词汇面扩宽**；非 AGE 面拼写不出 f32（`f32-kw` token 门控），故不额外开放语言面。
- adapter：text-record 面**按实际使用**补 `t_f32`（扫计划表达式/函数签名/局部；无 f32 的程序逐字节不变——core 全量 778 例绿无反证）。**潜在洞（记录）**：若 f32 将来经 record 字段/全局进入 text-record 面而行内无 f32 表达式，该启发式会漏声明 `t_f32`（今天不可达：该面拼写不出 f32 注解）。
- AV 出口：`ace_av::age_component::execute_component_def(compiled, def_id) -> Result<Value, String>`——共享 lower→MIR verify→ABC admit→AV 的动态值出口；窄枚举 `ComponentFieldValue` **不扩 F32**（auto-ui `AceStateValue` 对它穷尽匹配，扩了会跨仓编译红）；`observe_component_states` 对 f32 的错误消息改为具名指向该出口。
- 规范：`docs/specs/ace-migration/m2-schema.md` §6.2/6.3/6.4/6.5 同步（状态函数类型 / AV 宽值出口 / 拒绝面 / 边界）。
- 顺手修：`adapter.rs:1069` 的 `) -> … {    let` 行合并（`95b0ccb49` 在 master 引入的 fmt 断裂；本切片触面文件内一并修）。

**证据（`be8480cae`；合并树 `6deef24b6` 复跑）**
- 新 AV 用例 `plan981_f32_surface.rs` **5/5**：`x = x + vx*dt` 一 Tick→**1.0**、两 Tick→**2.0**、零 Tick→0.0；全算符一次 Tick 全落值（sum 11.0 / dif 7.0 / mul 18.0 / div 4.5 / rem 1.0 / neg **-2.0**，eq/ne/lt/le/gt/ge 全值）；混宽（f32+int）`source.type-mismatch` 具名拒。
- core `plan981_model_field_types` **6/6**：f32/float 两字段类型两面接受（包面字段+状态符号 / 组件面 states-only）；包面注解位（fn 参数/返回/`let`）通过 + 混宽拒。
- 981 家族：core **229/229**、AV **28/28**。
- 独立复审（fresh 子代理，非实施会话）：初判 needs_fix（A2R 腿缺、`neg_f32` 缺、只证 add/mul、窄通道错误未指向出口、`uses_f32` 启发式、新测试文件未 fmt）→ 除 A2R（用户明示延后）外**全部在同批闭合后复验**；复审自跑的 core 全量 778/778 绿（证明描述符扩宽与 `t_f32` 条件发射不扰动既有预期/身份）。
- 固定组（合并树）：`cargo t ace-source` **PASS 859**；`cargo t ace-av` 1387–1388/1389，红项为**已登记环境族**——`kv_zip_1064::v03…`（`D-1037-02` 的 2s wall 余量，既有登记）、`sys_d01_d06_1064::v02_d06_spawn_with_output_av` 与 `v02_d01_d06_a2r_cross_end`（进程/rustc 腿并发负载）；三条隔离单例重放全绿（3.0s / 4.7s / 14.2s）⇒ 环境归因，**不充"无红 PASS"**。旁证：三次整组跑分别红 1/1/2 条且**每次不同**（bstr_1064 → sys973_v03 → kv_zip+spawn），隔离全绿。

**未做/边界**：A2R 腿（用户明示"AV 证据可先交 UI058，完整关闭补 A2R"）；`ComponentFieldValue`/auto-ui `AceStateValue`/TS 能力类型/文本呈现的 F32 **跨仓同窗**；文本叶呈现 f32；`f64`/`double`；`render2d.spawn` 文法接收（下一小切片：保拼写、age-component/age-package 两 id 重基线）。

**下一步**：UI058 可用 `execute_component_def` 取 f32 真值（T-02 先证路径）；`render2d.spawn` 小切片；A2R 腿与消费端枚举同窗。

### 2026-11-18 stage: design（会签 ④-A 共享文法与绑定合同；981 grammar/binding owner 回签 980 r18 D-④-2）| PLAN-981 r18 | outcome: **会签通过（④-A 目标与"包绑定器为唯一权威"）＋ D-④-2 修正：不开 age-component 面，多模块闭包走 age-package 面；组件面单文件的 `use` 零事实改具名拒** | 零代码/零建树/零运行/零合入

**授权与范围**：用户本轮指定"981 会签 980 ④-A 共享文法，并与 UI058 对齐 T-02"。本节只做合同与证据核对（980 r18 §10 D-④-2 的 981 会签项），未改文法/AST/绑定器/Schema，未建树、未运行 Cargo/产品。基线：auto-lang master `a13a6e6a1`（本会话开工时读到 `f46560e00`，中途被 982 会话推进 4 个 docs 提交，不影响本节代码事实）。

**一句话结论**：④-A 的方向（组件作者面接纳 Auto 模块名、以 960/981 包闭包绑定/verify 为唯一权威）**通过**；但"先在 age-component 开 `use <mod>: syms`"这条实施路径**不成立**——按现绑定器的单面身份合同，它为 ④-A 买不到任何东西，却要先欠一次冻结身份重基线。建议 ④-A 的入口面改为 **age-package**（两形已开、多模块权威已验），age-component 面保持单文件并把 `use` 行从"解析后零事实"改成具名拒。

**证据（静态读取；逐条含路径）**

1. **文法面**（`experimental/ace/grammars/auto-source.at`）
   - `age-use-module-item`（`use m.n: a, b`）在 `age-component`+`age-package`（L329/L704）；`age-use-sym-item`（`use mod: a, b`）仅 `age-package`（L337/L1052）；包面 `use-item-p`（`use a.b`）不在 `age-component`（L338）。
   - `age-component` 没有 `public-item`/`const-item`/`var-item`（L333-335）⇒ **不能解析 `pub fn`/`pub const`/`pub type`**；普通 `fn-item` 全域（L311）。
   - 依赖模块的真实形状不可解析：`auto-ui/packages/themes/themes.at` 用 `pub theme Auto = { … }` + `extend(...)`，而 ACE 文法**没有 theme 规则**——依赖闭包里最普遍的那个模块（34/35 例都导入它）**今天连解析都不成立**。
   - `age-node-c-item`（`node` 全体成员）两面都有（L309），`age-package` 是 widget 源的**成员规则超集**（L446-450 才多出 view-2d/3d/portrait/widget）。

2. **投影面**（`core/src/source/cst/frontdoor.rs`）
   - 包面前门把 `age-use-sym-item` 逐符号投影为 `PackageImport{path:[mod, sym]}`（L1511-1553），与 `use mod.a` 同义；重复名 `source.module-duplicate-import`、声明后导入 `source.module-import-order` 都在此处拒。
   - `age-use-module-item` **没有任何投影分支**（前门只读 `age_use_sym` / `use_item_p` / `public_item` 三类）⇒ 34/35 例的 `use themes.themes: Auto` 在包面也是**零事实**（与 980 census 一致，不是新发现）。
   - 组件前门（`age_component.rs:461/878/889/916/946/972`）全部是**单源入口**（`src: &str`），不存在"组件 + 依赖模块"的多输入签名。

3. **绑定面**（`core/src/source/packages/binding.rs`）——已定型、可直接复用
   - `public_target`（L832-932）：fn/type/const 走 `unit.exports`；**store 恒私有**（L904-910，`source.module-private`，"module store … is private"）；**AGE node 导出先于 `exported` 闸**（L894-903，作者面 `node Disc` + `use shapes: Disc` 无需 `pub`，走 `Target::Node`）。
   - 失败码：缺模块 `source.module-missing`、私有 `source.module-private`/`source.module-reexport-private`、未知声明 `source.module-symbol`、自导入 `source.module-path`、环 `source.module-cycle`/`source.module-reexport-cycle`、身份不符 `source.module-stale`。
   - 同源永久用例：`plan981_age_package_modules.rs`（两文件 helper 闭包 / 缺模块与私有俱具名 / 环 / 输入换序身份不变 / 改 helper 改 `input_hash`）。

4. **身份约束（决定性的那条）**
   - 绑定器对**每个 unit** 断言 `u.grammar_id == 单一 face 的 grammar_id`，否则 `source.module-stale`（L657-669）；`Faces` 只有 Plain/Aggregates/AggregatesCt/Age（L439-444），`Age` 对应 `age-package` 裁剪（L590-593）。
   - ⇒ **混合面闭包今天不可表达**；而 `age-component` 解析不了 `pub fn` 依赖 ⇒ "入口 age-component + 依赖模块"这条路必须先改绑定器合同。
   - 文法身份只编码**存活规则**（`program_codec.rs:9-93`；`profiles:` 门不进字节，`syntax-grammar/src/lib.rs:33-38`）⇒ 给某规则加一个 profile 只会移动**那一个** profile 的 GrammarId；`age-package` 与 13 邻居逐字节不变（同一机制已有 `plan981_grammar_scope.rs` 的 13 行冻结值作对照）。

**合同条款（④-A，981 会签版；措辞即 980 切片的验收口径）**

- **A1 入口面**：多模块 widget 工程整体按 `age-package` 面编译（`compile_age_package`，带 `LegacyWidgetAlias`/scenario/模块图）。不新增第二绑定器、不源拼接、不"入口组件面 + 依赖包面"的混合闭包。
- **A2 作者形 → 绑定**：`use mod: a, b` ≡ `path=[mod, sym]`（同包 `mod` 文件）；`use pkg.mod: a, b` ≡ `path=[pkg, mod, sym]`（跨包）。两形**绑定同义**（同命名空间/同可见性/同错误），但**路径长度不同义**：`use a.b`（2 段、无符号表）在包面是 item 导入，不得被当成"模块 + 符号"。
- **A3 引用驱动（惰性）绑定**：未被子引用的导入**不解析、不建模块边、不要求目标模块在场**；被引用的名字必须解析，失败具名拒。依据：34/35 例含 `use`，其中 17 例纯惰性（只 themes 名、值用 `"Auto"` 字面量），而 `themes` 模块的 `pub theme` ACE 不可解析——**eager 绑定会让这 34 例全部变红**。这是 ④-A 唯一与现包面 eager 语义的差异，必须以 face 为界，不得改变 plain 包面。
- **A4 符号种类**：④-A 的值面=`pub fn` / `pub const`（`style` 作 const 形）；类型导入只作注解；`node` 导入不在 ④-A 值面（进入值位置须具名拒，不得静默）；模块 store 恒拒。
- **A5 失败面**：沿用 A3 的码表；新码必须具名且 span 到名字（不许 @0）。
- **A6 身份**：`ace.age.component/v1` / `ace.age.package/v1` / `ui.widget-decl/v1` **不变**；导入只进闭包身份（模块边/`input_hash`，改 helper 源必须变——`plan981_age_package_cross_module_helper_changes_the_checked_program` 已是同型正控）。若实现最终仍改文法，`age-component` 冻结值由 981 在**同一提交**重基线（`plan981_grammar_scope.rs:114`），13 邻居逐字不变。
- **A7 不做/另案**：`use { package: … }`（5 例，任何面都无规则）、`use auto.*`（命名空间形无规则）、N3 foreign（`use.rs/c`）、N4 宿主名表——保持 ④-B/④-C/976 后继域。
- **A8 fail-closed 补口（981 可独立做的小修）**：组件单文件入口今天对 `use` 行"解析后零事实"，应改具名拒（如 `source.import.unsupported`：单文件面不解析导入，请走 AGE 包入口）。这不是 ④-A 的必要条件，但 C8"不得把能解析当已绑定"要求在案。

**给 980 的回签文本（可直接并入 980 §10 D-④-2 行）**

> 981 会签（2026-11-18，master `a13a6e6a1`）：④-A 目标与"包绑定器唯一权威"通过；**D-④-2「开 age-component 面」建议改为不开**——多模块闭包走 `age-package` 面（两形已在、多模块已有永久用例），`age-component` 保持单文件并对其 `use` 行具名拒。若 980 坚持组件面入口，前置是绑定器 **per-unit face** 合同改造（今天 `binding.rs` L657-669 的单 face grammar-id 断言阻断；`age-component` 又解析不了 `pub fn` 依赖），须经 981 独立 review，成本/风险显著高于换面。④-A 的硬条件：A3 惰性绑定（否则 34/35 例回归）。

**边界（不冒充）**：本节是静态合同核对与既有永久用例的复用，**不是**新 PASS；未跑 Cargo/产品/GPU；④-A 的实现（投影 + 惰性绑定 + 入口接线 + AV/TS/A2R 三后端）仍属 980 切片；981 只签合同与冻结身份。若 980 选择 per-unit face 路线，本回签不涵盖该改造，须另立评审。

### 2026-11-18 stage: handoff（UI058 T-02 接口联冻：981 侧签署；四项中三项即刻冻、f32 面登记为 981 前置）| PLAN-981 r18 | outcome: **partial（接口表 981 侧已签；UI058 回签已并入本节；f32 实现进行中）**

**绑定身份**：981 接面=IF-981-GFX-AV-SEAM（`ddbda0228`+`ed8b16e01`，已含于 master `a13a6e6a1`）；auto-ui master `f43ef59b`；UI058 计划 `auto-ui/docs/plans/058-age-topdown2d-queries-and-sprites.md` r4（drafting、0/10、T-02 未开）。UI058 无实施树（`git worktree list` 无 ui-058），本会话**未确认其内部进度**——本表按该计划 §5 T-02 合同与其"冻结 profile/typed 端口/预算映射/数值/共同发布"条款对齐。

**服务面（唯一事实源 `core/src/source/age_resources.rs:219-361`，17 op / 6 族 / 7 端口）**

| 族 | 源拼写 → 结果 | 费 | 车道 |
|---|---|---|---|
| query | `query2d.segment(i32,bool,f32×4)→Cast2D`；`query2d.sweep(i32,bool,f32×2)→Sweep2D` | 8 / 16 | HostAlgorithm |
| input | `input.space()→i32`；`input.x()/y()→f32`；`input.pressed(i32)→bool`；`tick.dt()→f32` | 1 | HostAlgorithm |
| scene | `row.place/shift(i32,f32,f32)→i32`；`row.show(i32,bool)→i32`；`row.world/local(i32,i32)→f32` | 1 | HostAlgorithm |
| sprite | `render2d.spawn(str,f32,f32)→i32`；`render2d.despawn(i32)→i32`；`render2d.frame(i32,str,i32,i32)→i32` | 1 | HostAlgorithm |
| resource | `image2d.png(str)→Image2D`（身份=内容 SHA256+len+IHDR） | 1 | HostIdentity |
| value | `frame2d.at(Image2D,i32×4)→Frame2D` | 1 | ValueCtor（AV 可执行） |

端口（`AGE_PORT_FIELDS`）：Material2D/Mesh2D/Model3D/Cast2D/Sweep2D/Image2D/Frame2D。**AV 可观察面只有 `frame2d.at` + 端口字段读**；其余 16 op（15 HostAlgorithm + 1 HostIdentity）在 AV 观察车道是具名拒、归 UI058 宿主车道。

**四项联冻（981 侧签署）**

1. **f32 注解与运算 — 拼写冻结，能力未交**：`ann-ty`/`ret-ty` 的 float/f32 臂只在 numeric/aggregates/scheduler/packages-aggregates-ct（`auto-source.at:1375-1376,1400-1401`）；`f32` 在 age-package 是**关键字 token**（L155），而注解位无对应 alt ⇒ `fn f(x f32)`/`… -> f32`/`let x f32` 解析失败（静态文法推断，与 `age-model-ty` 缺 alt 时实测同机制；该面已补 token 但**字段类型仍具名拒**：`plan981_model_field_types.rs:116-123` 两面同码）。f32 **值**已通（字面量/场景行/端口字段/服务结果）。f32 同型四则与比较在 typecheck 已接受同型数值对（`typecheck.rs:489-532`），但 text-record **atom 面未登记 f32 运算**（`plan981_gfx_seam.rs:7-10` 边界声明）。⇒ 冻拼写 `f32`（不用 `float`/`f64`），三处注解位 + model 字段同词，运算=同型 `+ - * / %` 与 `< <= > >= == !=`（不做混型提升）；**UI058 在 981 交付该面前不得依赖 f32 业务式**（T-02 先证可用静态行值 + 宿主行写）。登记 `D-981-AGE-F32-SURFACE`。**进展（2026-11-18）：AV 面已交 `be8480cae`——f32 字段/注解/同型四则·比较·一元负号经 AV 真值验证（`plan981_f32_surface.rs` 5/5）；A2R 腿与消费端枚举仍按本项条件补。**
2. **`spawn` 拼写 — 保 021 拼写，改文法接收**：`r-spawn` 是全域保留 token（`auto-source.at:197` 无 profiles）⇒ `.spawn` 不能作成员名、`render2d.spawn` 不可解析（`plan981_gfx_seam.rs:96-98` 声明）。决定：**保 `render2d.spawn`**，仅在 AGE 两面的调用成员位接受 `spawn`（局部 alt、token 表零新增）；`age-component`/`age-package` 两个 grammar id 具名重基线（981 同提交更新冻结值）；`AGE_GFX_SERVICE_OPS` 的 callee/symbol **不变**。备选改名（零文法改动）不取：021 拼写/价目表改动面更大。**（2026-11-18 已交 `0e6c03c3d`：两面成员位臂 + 两 id 重基线 `19a1376e…`/`0290d0ab…`；永久用例证服务绑定与元数拒。）**
3. **getter 轴参数 — 冻分量字段**：端口把 021 的 `point(axis)/normal(axis)` 展开成 `point_x/point_y/normal_x/normal_y`（sweep 另有 `origin_x/y`、`start_overlap`）（`age_resources.rs:77-101`）；字段读**不构成服务调用=0 credit**。映射：`point(0)≡point_x`、`point(1)≡point_y`、`normal(0/1)≡normal_x/y`、`origin(0/1)≡origin_x/y`、`start_overlap()≡start_overlap`。**不做**动态轴 `point(i)`（无服务面、无零成本语义）。
4. **每回调预算 — 981 已冻，UI058 已回签（见下）**：`GFX_CALLBACK_CREDIT_BUDGET=32`；`segment=8`、`sweep=16`、其余服务=1、端口字段读=0（`age_resources.rs:378-394`）；语言侧静态准入=**每函数调用点合计**≤32，超界 `lower.age-gfx-credits` 具名拒零效果（`hir_adapter.rs:4383-4409`；永久用例第 33 次拒 `plan981_gfx_seam_av.rs:182-211`）。语义冻：32 credits/**世界回调（Tick）**、cast/sweep/getter 价目不动、静态界=**保守准入**（不作运行时计费）、宿主动态计费按同表、物理写 32 与 credits 分开计。**统一口径（UI058 回签原文）**：**下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝**——恰好 32 合法；两次 `sweep`（16+16）耗完 32 后，下一次收费调用即拒；旧表述"第 33 次"按**累计 credits** 解释、不按调用序号。

**给 UI058 的 3 个回签请求（T-02 冻结时确认）**
1. "32 credits/callback" 的解释域=**每 Tick / 每 world callback**（981 按此冻；若 UI058 是"每世界每帧"须显式改写，双方同改）。
2. `render2d.spawn` 保拼写、文法接收（**2026-11-18 已交 `0e6c03c3d`**：AGE 两面成员位可写、两 grammar id 已重基线；T-02 冻结期的符号 `age.render2d.spawn.v1` 不变，调用点现在可以落地）。
3. f32 面未交前，T-02 先证**不以 f32 业务运算为前提**；若 UI058 判定必须，回 needs_replan 并把 f32 列为 981 前置（不得用 int 近似代替 f32 语义）。

**UI058 回签（2026-11-18 已并入；PLAN-058 r5，auto-ui `91d549ea`）**：四项联冻接受——①预算域=每世界回调（Tick），统一口径「下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝」（恰好32合法；两次 sweep 16+16 后下一次收费调用即拒；981 记录的「第33」按累计 credits 解释、不按调用序号）；②`render2d.spawn` 保拼写、待 981 文法接收，T-02 冻结期用符号 `age.render2d.spawn.v1`；③getter 用分量字段（`point_x/point_y`、`normal_x/y`、`origin_x/y`、`start_overlap`）、字段读 0 credit、不做动态轴；④f32 拼写/同型四则与比较仍为 981 前置，T-02 先证用已有静态 f32 行值＋宿主行写（场景行/`AgeSceneValueFn` 真值；宿主行身份/可见性/字段/Descriptor，旧链先例 `tests/node_queries_flow.rs:101-131`），不依赖 f32 业务式；真实 Auto 移动积分（`x = x + vx*dt`、位置/速度 f32 字段）等 `D-981-AGE-F32-SURFACE` 的 f32 AV 切片，不用 int 近似；UI058 T-02 判定必须依赖 f32 业务式时回 needs_replan 并列为 981 前置。UI058 未建树/未跑 Cargo/GPU，T-02 及以后未勾。回签原文与逐项答复见 `auto-ui/docs/plans/058-age-topdown2d-queries-and-sprites.md` §9（2026-11-18 stage: handoff）。

**UI058 T-02 预算用例（2026-11-18 约定，先证时钉住）**：①两次 `sweep`（16+16）恰 32，边界值合法；②`sweep + segment = 24` 后，同一 Tick 内另一 Node 再 `sweep`（+16）应超额：具名拒、零效果；③紧接着仅收 1 credit 的调用也必须继续拒绝（sticky）；④下一 Tick 额度重置可再计费。该组同时检验全 Node 共享预算与超额后持续拒绝。

**边界**：未运行 Cargo/产品/GPU；未改 `AGE_GFX_SERVICE_OPS`/文法/Schema；未建 UI058 树、未代其勾选任何任务；T-03 起的宿主算法（v3/v4 查询、行状态、帧写、PNG 载入、输入喂值）仍归 UI058/PLAN-938 host authority。

### 2026-11-17 三路线修订的规划预检（不是实施验收）

本轮仅文档：统一strategy/架构、四个引擎AV新合同与四个新版AutoForge合同；保本包原任务ID/进度/全部旧AC/V/SD，新增阶段义务见§5–7。静态核对8新Plan共74任务、9既有Plan共140原任务ID、阶段DAG 44节点/90依赖无环、新增本地链接均可达，16份取材源码/规范/已归档合同身份未改变；四仓文档diff格式检查通过。981并发实施会话的T-01勾选保留，不计本轮交付。下方历史记录原样保留。

只说明规划合同可审阅；未运行Cargo/GUI/模型/构建/产品测试，未建实施树、未resume暂停工作、未合入/归档/签章。所需正式接口/最低负载/真实oracle仍须未来work→独立review→merge实际验证；原已存在历史失效链接不作为当前入口，新增链接已单独核对。

> EVENT-SPLIT-20261116历史解释：本节旧blocked/“等待980正式源”、round39/40及旧全量A门只证明当时核对/选择；当前依§5和§8逐切片合同，不用executing/整包未勾代接口缺失。旧产品结果不因本次修订自动扩大为参数/载荷或新A验收PASS。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r14：按用户最新三端目标修阶段/移交，增SD/V/AC-AC-STAGED；原任务/状态/进度保留，仅规划，无新增实际验证/签章。

2026-11-16 stage: review（B-SESSION-I 二次 phase 复审 / fold 门）| PLAN-981 | review_scope: phase | outcome: **pass** | commit `de7238a4`

**复审（独立 subagent，auto-ui 树）自跑 107 例 0 失败**：driver lib 47/47、s2_age_routing 6/6、
bridge 4+3+7、V 点名旧链 target `authoring_session` 4/4 与 `mixed_regions_lifecycle_flow` 7/7（旧链未受伤）、
`authoring_behavior` 12/12（模块级 `var` 旧源）、17 例独立探针全绿、`prepare_sdk --verify-only` 与
`verify_overlay` 双 PASS（681 base + 1361 overlay）、SDK 重建探针 PASS（2042 文件 0 mismatch）、
**变异负控**：把 `update_body` 改 no-op 后 4/6（2 红在 v1≠v2）⇒ R63-2 证据可判别。
**闭合**：R63-1（六项）、R63-2/3/4/5/6/7 原诉/8/9/12 closed；R63-10/11 partial（低危）。
**新 findings（V 前必修/前置，不阻塞本次 fold）**：R64-1 被拒调度不原子（`schedule` 先 extend 后编译，
坏消息毒化后续调度直到 restart）；R64-2 `de7238a4` 使 `Session::new` 对坏 AGE 源只报旧 parser 错（丢失
`source.module-symbol`；`AgeSession::open` 仍具名）；R64-3 宿主面单组件（`exits/request_layout` 只读首
组件，双组件 2:1 不可表达）；R64-4 双 node 同名 model 字段撞 `__age_state_0_<field>`（ACE owner）；
R64-5 更新是整段重放且 generation 混用；R64-6 overlay 无提交化物化步骤；R64-7 `tests/s4.rs` rustfmt；
R64-10 资源 owner/多 Run-Stop 净归零与 AGE 异步仍无宿主面。
**fold 判定**：分支 6 提交 vs merge-base `2fc427f6`，auto-ui master `dfc73cf1` 已前进 99 提交且零路径
重叠、merge-tree 无冲突 ⇒ 必须 merge。

2026-11-16 stage: blocked（tier-1 余量等 980 正式源）| PLAN-981 | outcome: blocked

**判定**：tier-1 内可独立完成的代码面已交并落主（SOURCE 979 已核；LANGUAGE-I review-62 pass，auto-lang
`1738c2a1c`→`f91b50fc9`；SESSION-I review-64 pass，auto-ui `15102c48`→`07821d49`；RUST 公共链 4/4 +
具名缺口）。**剩余必做项全部依赖 980**：V-B-LANGUAGE / V-B-SESSION / V-B-RUST / V-B-LANGUAGE-V 需要
980 的正式 UI/事件源，而 980 仍 `executing`（round 26，12/32）。
**恢复条件**：980 交付正式源后，按 §9「交接说明（980 就绪后 V 批的执行路径）」执行——替换
`V_PENDING_INPUTS` 三项 → 跑 V-L-* 与 V-B-SESSION（driver lib 47 / s2 9 / bridge 17 / 双层 SDK 校验）
→ 独立 phase 复审 → fold → 勾批。
**不冒充**：不把 I 切片当 V 交付；不用 981 自造样本代替 980 正式源；未改验收口径；未声称 plan-level
execution_done。**具名债务**：`D-981-MODULE-GLOBAL-A2R`、`D-981-SESSION-MULTI` ①、`D-981-EXIT-CONTRACT`、
R59-5（已附定位证据）、R62-7、正式 SDK pin（978/986/987）。
2026-11-18 stage: work（建议 #2 **B-LANGUAGE 语法余量对账** + #3 **V 批预演核对**；纯簿记无代码）| PLAN-981 r18 | outcome: **对账完成——LANGUAGE 语法实现面已基本齐，剩余是等 980 的验证面**

**#2 对账结论（B-LANGUAGE-I 义务 vs 当前文法/已交记录）**
- **五 view**：全部在 age-package 文法（view/view-2d/view-3d/view-portrait/view-widget）——IF-981-FIELDS 复审时"view-portrait/view-widget 不存在"的记录已过时（后续批交付）。
- **多呈现选择**：已交——D-981-EXIT-CONTRACT 关闭时落永久用例（`plan981_v_harness` view+view-portrait 并存/方形回退；`plan981_proof_a` View+View2d+View3d 三出口选择）。
- **if**（fn 体）：condition-expr 在 age-package 面 ✓；**#if**（编译期）：packages-ct 面 ✓（AGE 作者面 CT 用法归该合同）。
- **Start/Tick/Frame 生命周期**：差额表 #3c 已交（含 Pause/Step/Stop 语义）。
- **新旧组件互操作**：980 #6i/#6m/#6o 切片在持续交付同一文法面（事件键修饰链/元素属性行/调用语句）；grammar-id 冻结身份重基线由 980 round41→本包 LANGUAGE[A] 的承接记录管辖。
- **真实余量**＝V-L-* 完整验证与新旧组件组合矩阵（等 980 正式 UI/事件源，见 V_PENDING_INPUTS）——**验证面而非实现面**；此前把"选择/Frame"列为待实现是基于过时读数，特此更正。
**#3 V 批预演核对**：`V_PENDING_INPUTS` 现 2 项（typed UI/资源宿主面＝R64-10 异步面；980 合流 oracle）——均 980-pending、非本包可动；harness（`plan981_v_harness.rs`）与命令/源/oracle 清单无漂移，"980 就绪即可开跑"状态保持。

2026-11-18 stage: work（**D-981-MODULE-GLOBAL-A2R 关闭批**：用户指令"按照你的建议实施"→建议优先级 #1）| PLAN-981 r18 | outcome: **pass（债务关闭，路线①的 CheckedModule 层实现）** | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | base `183e3f653`（再 sync master 22 提交）| code_commit `694908d8c`

**实现（路线①：事实进 CheckedModule，不动 HIR 文本格式）**
- `core/verify.rs`：新增 `ModuleGlobalsPlan { init_order, init_fns, facts[(symbol,name,mutable)] }` + `CheckedModule.module_globals/set_module_globals`（沿 text_plan/coll_plan/foreign_plan 的 facts-plan 先例）；AGE 包前门在唯一 verify 后从绑定器（`BoundPackages.globals/module_init_fns/init_order`）钉入。
- `a2x-core/semantic_view.rs`：收集 `module.global.get/set:<sym>` 访问器 intrinsic；**仅当 plan 在场**才放行（无 plan 路径维持旧 compute-surface 拒——不静默扩大）；访问器符号必须命中 plan 事实；`globals_model()`（符号/类型/可变性）与 `global_init_fns()` 供代码生成。
- `a2x-rust`：`static mut AGE_GLOBAL_<sym>`（计算面标量零值初值；Str/record/容器型具名拒=剩余边界）+ get/set 调用降为 unsafe 块内 static 读/写（setter 产 0 保合同）；两条组装路径（rt.rs/provider.rs）的生成 main 均在入口前执行 init 链（init 失败按 trap 报告）。
**证据**
- 关闭用例（a2x-worker，真实 rustc）：`const K=6` + `var total=helper.seed()`(=40，跨模块 init) + 两次 `bump()`(set/get 双径) → `a+b+K-84=5`——init 链/读/写任一环节坏值即偏；**变异重放红**（禁用 init 链 → value:-81）。
- 旧 V-R-FAIL 负控（`p981_rust_public_chain_rejects…` 的 const 腿）按债务关闭条件**翻正**：同公共链真跑 value:1 + generated.rs 含 static 断言；恢复对照保留。
- plan981 A2R 家族 3/3；门禁 `ace-source` **PASS** + `ace-a2x` **PASS**（整组复跑；首跑两红为 958 认证用例满载并行资源竞争——隔离绿/家族 10/10 绿/复跑绿三重证据，环境归因，与 982 会话并发编译同时段）。
**下一步**：建议优先级 #2（B-LANGUAGE 语法余量**先对账 980 #6m/#6o 已交面**）与 #3（V 批 harness 预演核对）未动——按用户"按照你的建议实施"的顺序续做或按需暂停。

2026-11-18 stage: work（**语言侧 AV 残余批 #1–#5**：用户指令"按照这个顺序把1~5都做了"）| PLAN-981 r18 | outcome: **pass（#1–#4 交付+#5 更正收口；D-981-GLOBAL-CLONE-GAP 关闭）** | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | base `16d5f1114`（ff re-sync，383 提交）＋`a2894e35e`（再并 master `020cd3ec1` 根 lock 修复）| code_commits `106a0f6ac`/`900bbabd1`/`9af02eaef`/`78ffd6e28`

**#1 D-981-GLOBAL-CLONE-GAP 关闭（`106a0f6ac`，登记债已标关闭）**：CLONE 成为唯一视图→拥有跨接指令——mir_verify 放行 `Clone{Record(r)←RecordView(r)}`（COPY 不能跨视图故标量 record 同形、深克隆退化为值拷贝；异名仍 clone-type）；gfx 构造器 record 实参与 GlobalSet 视图写入改发射 Clone 跨接。三层证据：mir 单测（资源/标量准入+异名拒）、执行级深拷贝独立性（原件 Str 长度存活，浅别名会红）、AGE 源级 frame2d.at 整包过 verify + `var g Mesh2D` 视图参数写入过 verify + 异名端口前门拒；**变异重放**：禁用跨接臂两用例精确红。边界如实：frame2d.at 运行时执行待 UI058 宿主注入 Image2D 身份（T-03）。
**#2 精灵叶属性面（`900bbabd1`）**：`image:`/`frame:`（ResourceRef→Image2D/Frame2D 端口）+`flip:`（bool）入 shape2d 三表；模块 `let` 声明准入扩至静态资源车道 gfx 服务（image2d.png/frame2d.at；逐 Tick 动态 HostAlgorithm 保持具名拒）；引用事实沿 check_scene_resource_refs 同通道（resource_refs 面 Image2D/Frame2D 目标+原拼写）；2D-only 边界钉住（3D 叶上具名拒）。4 新用例。
**#3 服务级信用计量（`9af02eaef`）**：冻结 021 原额费率表（segment 8/sweep 16/基价 1、预算 32/callback，**与 UI058 T-02 联冻**）+ gfx 派发点每函数静态调用点合计准入界——超界 `lower.age-gfx-credits` 具名拒（构造性零效果）；宿主动态逐调用计费+sticky 消费同表（费率表冻结值有正控用例）。边界如实：静态界是保守准入面，动态计费物理派发点在 UI058 宿主。
**#4 query token 失效语义（`78ffd6e28`）**：同一 callback 内第二次 query2d.* 使先前 token 绑定失效（021 fault 13 语言侧缝）——源序走查+单调 generation（循环保守：循环内 query 使外层旧 token 失效），stale 读 `source.age.query-token-stale` 具名拒；新鲜读/重绑刷新合法。边界如实：age-package handler 体今日只收 `.field = expr`（age-view-stmt），查询在 handler 内不可表达，当前可表达面=pub fn；走查器遍历含 `__age_p_*` 隐藏 fn，文法扩展后自动覆盖 handler。
**#5 更正（无新工作）**：R62-7 已于 11-16 修复关闭（`536f5b316`，watch 修饰形独立规则，用例 plan981_members_slot 仍绿）；R59-5 有裁定"投影侧不可修、owner=syntax-grammar、不在投影侧重复试修"（每 token 皆 error 叶的决定性证据，实验已回滚）——**本批开始时把这两项列为待办是过时信息，特此更正**；两项均非本批可动对象。
**门禁（冻结并集，同步后基线）**：`cargo t ace-source ace-lowering ace-av` 全 **PASS**（并集墙钟 360.7s，含缓存回收后的工具重建）；plan981 全家 253/253（1.2–1.6s）；新用例单例 0.028–0.22s（全部 ≪10s 限）。变异重放 2 组（#1 跨接臂、#3 无）。
**过程事件（如实）**：开工前 ff re-sync 分支至 master（吸收 mir_verify 目录重组织等 383 提交）；中段 ace-source 组因根 lock 缺 `ace-a2x-ts`（16d5f1114 时点 master 自身已知问题）红 → 再并 master `020cd3ec1`（其正式修复）后绿；磁盘空闲一度 <2GiB 触发运行器预检拒跑 → 回收本 worktree 可再生缓存（ace/root incremental + 三个非本批工具 target ≈12GB），回收后磁盘空闲恢复正常。**环境观察（非本批）**：`mn-linux-av-runner.rs` 9 条 unused-import 警告（Linux runner bin，master 既有）。
**下一步**：批已完整落 branch 待 fold；981 整包仍 executing（剩余面按差额表：f32 model 字段等 T-02 联冻项、B-W3 的 MODULE-GLOBAL-A2R、V-B 验证面等 980 切片）。

2026-11-18 stage: work（review-D 复验 needs_fix 修复：F-D1/F-D2 关闭）| PLAN-981 r18 | outcome: **pass（2/2 findings 关闭，gfx seam 批恢复干净基线）** | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | base `1b70c9e1f` | code_commit `27f919b87` | fold `83f2adee1`（master `--no-ff`，第二父即 `27f919b87`）

**closure 行（finding → invariant → fix → 结果）**
- **F-D1** → 新诊断串不得含意外长空格串（③同类）→ `hir_adapter.rs` view-arg 消息 5×62 空格串→单空格（行首 60 缩进保留）→ 折叠 master 后该行 10+ 空格串=仅行首缩进 1 处 ✓。
- **F-D2** → diff 不得留缩进塌陷 → 3 行恢复对齐（`"model3d.glb…"`→40、`"lower.age-gfx-lane",`→44、lane 消息→48，均按上下文同级行对齐）→ master 按内容定位（4331/4429/4449/4451）逐一核 ✓。
- **零语义证明**：diff 4+/4- 单文件；诊断码（`lower.intrinsic`/`lower.age-gfx-lane`/`lower.age-gfx-view-arg`）与测试断言子串（`D-981-GLOBAL-CLONE-GAP`）逐字未动。

**门禁**：分支 `cargo check -p ace-lowering` 0.52s 零警告；nextest `plan981_gfx` **7/7 PASS**（0.041–0.046s/例）；折叠 master 后 `cargo check -p ace-lowering` 3.35s **ace-lowering 零警告**。折叠细节：与 master 侧 `e055315726`（980 WEB #6o）合并干净；branch↔master 该文件 76 行差额=982 `079b7410e` config 区域（master 独有，预期内）；981 修复 4 行两侧逐字一致（内容定位核对）。
**环境观察（非本批，归属他计划）**：master `ace-core` lib 现有 2 条警告——`project.rs:4329` unused import `ConfigPairValue`（blame `499335856d`，982 config）与 `project.rs:5861` unused doc comment（blame `e055315726`，980 #6o）；均在本合并前已落主，分支态零警告，归各自会话收据核对。
**下一步**：无待办——review-D 四项修复+F-D1/F-D2 全闭，gfx seam 批恢复 handoff 收据声明的干净面（diff 卫生现为真）。UI058 消费侧（T-02/T-03）零影响；981 整包保持 executing。

2026-11-18 stage: review（IF-981-GFX-AV-SEAM **review-D 复验**；独立上下文=fresh session，非 gfx seam 实施会话；静态+运行时+变异自跑）| PLAN-981 r18 | review_scope: batch（gfx seam 修复复验）| outcome: **needs_fix（2 low，F-D1/F-D2；四项修复实质全部成立）** | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | reviewed_commit `1b70c9e1f`（master fold `ed8b16e01` 第二父即它，修复腿 diff 逐字节一致已核）| base `1c35bbee6`

**四项修复复验（均实质成立）**
- **① unused-mut**：`resolve.rs` `mut arg`→`arg`。touch 两源文件强制重编 ace-core+ace-lowering（`Checking`×2 → Finished 0.97s）**零警告零错误**。
- **② ValueCtor AV 准入**：`p981_gfx_seam_av_value_ctor_lowering_admitted` 双向 oracle（前门编译 expect + observe expect_err+串内容断言）PASS 0.180s。**变异重放红**：`if slot_ty != want`→`if false && …` 后该测试精确失败，露出 `mir.aggregate-field-type` 底层兜底 ⇒ 视图缝为**双层 fail-closed**（命名拒 + mir_verify 兜底，永不静默错值），测试有判别力；变异已还原、树净。债务扩围实核：KNOWN-DEBT `D-981-GLOBAL-CLONE-GAP` 含"范围扩展（2026-11-18，IF-981-GFX-AV-SEAM review-D）"段 + 关闭条件（合法视图→拥有构造指令 + frame2d.at 端到端 AV 正例）✓。
- **③ lane 消息空格**：`lower.age-gfx-lane` 消息内容已收敛单空格 ✓。**④ 弱断言**：arity panic 消息与真实输入一致（3 参 `render2d.frame`）；未知服务断言要求 `message.contains("query2d")` 具名 ✓（实读代码 + 绿跑双证）。
- **门禁复跑**：nextest `-E 'test(plan981_gfx)'` **7/7 PASS**（AV 4 + 核心 3；单例 0.177/0.178/0.180/0.187/0.200/0.202/0.215s，全部 ≪10s 限）。编译准备 26.38s（test profile；与并发会话 `cargo t ace-source`、lang-980 release 构建共享 CPU，计时含竞争）。lib 零新警告；ace-av test 目标 42 条警告为既有 fixture 常量（本批未触碰，非回归）。
- 未重验面（复用初审判证据，修复未触碰）：注册表 vs 021/022 spec SHA16 逐项一致、grammar 冻结身份 7/7、master 固定组 ace-source/ace-syntax/ace-av——修复 diff 仅 4 文件（resolve 1 行/hir_adapter/两测试文件），不动登记表与文法。

**新 findings（修复提交 `1b70c9e1f` 自身引入）**
- **F-D1（low，③同类缺陷再犯）**：`hir_adapter.rs:4400` 新增 `lower.age-gfx-view-arg` 诊断串**内部 5×62 连空格**（`#{si}`⇤62sp⇥`arrives as a borrow view; owned`⇤…）；本提交一边收敛旧 lane 消息同类长空格、一边在新消息重新引入，边界命中时用户直接看到。串内容 rustfmt 不检查，现有门禁不可见。
- **F-D2（low，diff 卫生）**：同文件 3 行缩进塌成 1 空格（上下文应 ~40 列对齐）：`4302`（`"model3d.glb carries…"`——与原行**逐字同内容**，纯意外重写零语义）、`4420`（`"lower.age-gfx-lane",`）、`4422`（lane 消息行——内容修复正确、缩进丢失）。rustfmt 对超宽行 bail-out 不报告。
- 扫描收口：两提交新增串内 10+ 空格仅 2 实例（seam 的 lane 消息=③已修；fix 的新消息=F-D1）；塌缩缩进仅 F-D2 的 3 行；测试 fixture（`\n` 转义内缩进）为合法用例源，不计。

**对 handoff 收据的更正**：收据"复审确认干净面：…diff 卫生"一句被 F-D1/F-D2 证伪；其余干净面未被推翻。

**next**：work 在同一树修 F-D1（5 处串内空格→单空格）+ F-D2（3 行恢复对齐缩进）——纯格式 ~5 行零语义变更；重跑 gfx 7/7 后补 fold 提交；UI058 消费侧（T-02/T-03）零影响（接口/语义不变）。981 整包保持 executing，本批复验不开/关任何任务勾选。

2026-11-18 stage: handoff（IF-981-GFX-AV-SEAM 交付：UI058 AV 接面签名/受检投影/车道分类，交 UI058 T-02/T-03 消费）| PLAN-981 | outcome: pass（接面批；M2A-08 后正式在序）

**绑定 revision**：auto-lang master `ddbda0228`（gfx seam 主体）＋review-D 修补 `ed8b16e01`。**实现见该两提交**；差额表见前一条记录（六覆盖项逐列）。

**交付面（981 职责内；算法/载入/喂值不进 core）**
- **服务登记**：`AGE_GFX_SERVICE_OPS` 17 op——query2d.segment/sweep（Cast2D/Sweep2D typed 结果端口，字段=021 getter 面分量展开）、input.space/x/y/pressed、tick.dt、row.place/shift/show/world/local、render2d.spawn/despawn/frame、image2d.png（内容身份端口=022 语义）、frame2d.at（纯值构造）；能力键 age2d.query/input/scene/sprite。
- **受检投影**：resolver `RecvClass::AgeGfx`（命名空间分类、逐位类型核对含端口引用、typed 结果、gfx-arity/gfx-type/gfx-port 具名拒）；端口字段可引端口（Frame2D.image: Image2D）；adapter 同一声明行发射 intrinsic 签名；record 实参走借用视图纪律。
- **AV 车道分类**：ValueCtor→端口 record Aggregate（同 material2d 执行面；**record 实参命中视图→拥有缝时 `lower.age-gfx-view-arg` 具名拒并引用 D-981-GLOBAL-CLONE-GAP（范围已扩展）**，scalar-only 构造器今天可执行）；HostAlgorithm/HostIdentity→AV 观察车道调用点 `lower.age-gfx-lane` 具名拒（fail-closed 零效果，算法归 UI058 宿主车道/PLAN-938 host authority）；mir_verify 只额外准入 age.frame2d.at.v1 import 身份。
- **UI058 T-02 联冻项（具名边界，不冒已决）**：f32 注解位不在 age-package 基础文法（grammar 未动，冻结身份 7/7）；f32 二元运算不在 text-record atom 面（BinOp 无 f32 cases）；`render2d.spawn` 与保留词 `spawn` 冲突（文法接收或改名二选一）；getter 轴参数拼写（point/normal 分量字段 vs 021 axis 参数形）；预算/信用计量映射（原 32 credits/callback 语义，与 UI058 联冻）。

**独立复审（review-D）**：初判 needs_fix（2 medium）——①新 unused-mut 警告、②ValueCtor 零 AV 覆盖；均已修（`ed8b16e01`：mut 移除、value-ctor AV 准入测试——实现中发现 record 实参视图缝并按 fail-closed 处置为具名边界 + 债务扩围）＋③消息空格/④弱断言一并修。复审确认干净面：注册表与 021/022 spec 逐项一致（SHA16 核对）、fail-closed 全链、grammar 零改动、BorrowView 包裹不伤既有族（resource 参数全标量）、diff 卫生。

**门禁实测**：core+av+lowering 2071-2072 全绿（perf:: 排除；1064 sys/d0d6 墙钟抖动隔离绿=环境归因）；gfx seam 7/7（核心 3：typed 正例 + arity/type/wrong-kind/未知服务具名；AV 4：host-algorithm/host-identity 车道拒零效果 + value-ctor 准入/具名缝 + 普通包不受扰）；master 固定组 ace-source/ace-syntax **PASS**、ace-av **PASS**（982 会话再次刷新根 Cargo.lock 后）；grammar 7/7。

**交 UI058 消费（精确版本/已验范围）**：消费 `ddbda0228`+`ed8b16e01` 的 `AGE_GFX_SERVICE_OPS`/`RecvClass::AgeGfx`/车道分类——T-02 直接以本表联冻拼写/预算/边界（四个具名联冻项见上），T-03 起图形宿主车道按 PLAN-938 host authority 消费同一受检产物（intrinsics + 能力键）；宿主算法（v3/v4/行状态/帧写/PNG 载入/输入喂值）与 `UI-W1-AV` 汇总仍归 UI058/980 原任务。**不冒充**：不替 UI058 最低图形组合/GPU/交互/冷包验收；不签 UI058 或 981 整包完成；W1 其余（CONTENT/FUSION AV 域格）不在本批。

2026-11-18 stage: work（UI058 AV 接面差额表，IF-981-GFX-AV-SEAM 第一步；用户指令：981 既有 LANGUAGE/SESSION 任务内核对交付，不重复立项、不等整包）| PLAN-981 | outcome: partial（差额表提交；实现随后）

**取材**：UI058 plan r4 §5 当前阶段合同 + V-S01..08（`auto-ui/docs/plans/058-age-topdown2d-queries-and-sprites.md`，drafting 0/10）；`node2d-queries.md`（021 候选，SHA16 `0d930490f45d9a12`）；`node2d-sprites-and-frame-animation.md`（022 候选，`f997bae66305eef0`）；981 已交 SOURCE（IF-981-MODULE/REF/FIELDS/ORIGINAL）、MIN-A-AGE（PROOF A）、EVENT-B 三件（`f4158e499`/`7b127afc8`）、AV 余量（`62d34e3df`：场景 f32 观察/ForNode/异步面）；980 已交桌面宿主（`IF-980-NODE-DESKTOP-A`：`ui_poll_event` 打包事件→`AceStateEngine.dispatch`→`ui_apply_{int,bool,str}` FFI、close/reopen/late 世代纪律）。

**差额表（六覆盖项；列＝作者构造→Schema/签名→唯一verify→AV调用→宿主服务→返回/错误→owner→现有代码/版本→判定）**

| # | 作者构造/操作 | Schema/类型签名 | 唯一verify | AV调用 | 宿主服务 | 返回值/错误 | owner | 现有代码/交付版本 | 判定 |
|---|---|---|---|---|---|---|---|---|---|
| 1a | `query2d.segment(ignore, visible_only, ax..by)`、`query2d.sweep(mover, visible_only, dx, dy)`（021 §1；handle=`refs.<名>`/`self.row()`/`-1`） | 无——需服务登记 `age.query2d.{segment,sweep}.v1`（params I32 句柄/Bool/F32×4|F32×2；result=typed 端口 Cast2D/Sweep2D） | 需 resolver 准入 + 元数/形态/visible_only 字面量/未知行引用具名拒（原 span） | 需宿主服务口（算法不在 core）；无 provider 时具名拒（fail-closed） | UI058 中立单元（021 v3 segment/v4 sweep，objects2d/1 组装件，Rust） | typed 结果端口 + getter 面（见 1b）；fault 9/12 不降 miss | 签名/verify/接线=981 LANGUAGE/SESSION；算法=UI058 | 旧链 `node2d-lower --profile queries-2d`（021 spec 实测冻结）；新链无任何 query2d 登记 | **981待实现 + UI058待提供** |
| 1b | 结果 getter：cast `.hit/.node/.t/.point(axis)/.normal(axis)`；sweep `+.origin(axis)/.start_overlap()`；q 即 token（同 callback 第二查失效、stale fault 13） | 需 typed 结果端口（`AGE_PORT_TYPES` 增 Cast2D/Sweep2D，字段=hit:bool/node:i32/t:f32/point:(f32,f32)/normal:(f32,f32)/origin/start_overlap——getter 语义可先以端口字段读承载） | 需错 kind getter/重绑/var/再赋值守既有具名拒族 + 端口字段类型检查 | getter=端口字段读（Aggregate/FieldGet 既有车道可执行——**无宿主算法依赖**） | ——（纯值读） | 错 kind/stale 具名（fault 13 语义）；越界 axis 具名拒 | 981 LANGUAGE | 端口 record 模式已交（Material2D=字段 record+Aggregate，`62d34e3df`）；token 失效语义未交 | **981待实现**（token 失效语义）；getter 载体**已交可复用** |
| 2a | 行位置读：`world(axis)/local(axis)`、`refs.<名>`；场景行 x/y | 场景行 f32 值已可观察（`AgeSceneValueFn`+`observe_age_package` 场景真值）；model f32 字段**不可持有**（`field_type`=int/i32/bool/str） | 场景行字面量/参数式值函数已过唯一 verify（静态值已交） | 已交——g01 原几何 AV 真值（`p981_proof_a_av_scene_package_and_mixed_profile`） | ——（读面无宿主依赖） | 参数化行=Parametric 具名跳过 | 981 LANGUAGE | `62d34e3df`（scene f32 观察车道） | **已交可复用**（读面）；f32 model 字段=**981待实现**（若 UI058 以 model 承位置） |
| 2b | 移动提交：`place(x,y)/shift(dx,dy)/show(flag)`（行写） | 无——场景行是**静态值函数**，无可变行状态；需行写端口或 model→场景发布通道 | 需写端口受检投影（谁拥有行状态：会话资源管理=981 SESSION） | 需宿主服务口（行状态归图形宿主 SoA——021 `ag_set_local` 语义）或 981 会话行状态 | UI058 中立行状态单元（021 行 op 既有 Rust 语义） | 位置写零效果于失败；character 行位置写具名拒（021 语义保留） | 接线/会话资源=981 SESSION；行状态算法=UI058 | 无（旧链 `ag_set_local` 生成器路径不复用） | **981待实现 + UI058待提供** |
| 2c | 同 Tick 查询/精灵一致 | ——（组合性质：一次 Tick 内查询与表现共用同一提交） | —— | Tick 原子性已交（状态函数整段重放、R64-1 调度原子、失败零效果） | UI058 组合（共同发布点=其 T-02 冻结） | 不一致=UI058 oracle 红 | UI058 | `ScenarioOp::Tick`/调度原子已交（`62d34e3df`/`1775f96d`） | **已交可复用**（原子性）；组合=**UI058待提供** |
| 3a | 输入：`input.space()/input.x()/input.y()`、pressed/held | 无——需服务登记 `age.input.{space,x,y,pressed}.v1`（params 空；result F32 轴/Bool/I32） | 需 resolver 准入 + 调用点类型 | 需宿主服务口（逐 Tick 宿主喂值；无 provider 具名拒） | UI058 运输层喂值（旧链 `_ag_action(space,x,y,toggle)` 语义；980 桌面按钮事件已交但**无模拟轴**——非阻塞依赖，UI058 自有 runner 可先喂） | 轴归一语义=UI058 冻结（T-02）；服务面返回 typed 值 | 签名/verify/接线=981；喂值=UI058（980 桌面可选消费） | 980 桌面 `ui_poll_event` 仅按钮位（`7f431bd5f`）；新链无 input 服务 | **981待实现 + UI058待提供**（980 非阻塞） |
| 3b | `dt`（每 Tick 步长） | 无——`age.tick.dt.v1`（result F32）或 Tick 携带 | 同上 | 同上（宿主逐 Tick 值） | UI058 runner（固定步长先冻结） | F32 | 同 3a | 无 | **981待实现 + UI058待提供** |
| 3c | `Life.Tick`/Pause/Step/Stop | 已交——生命周期消息 + `ScenarioOp::Tick`/`ForNode{Tick}` | 已交（唯一 verify 全链） | 已交（AV 状态函数重放） | —— | `lifecycle_budget` Start/Tick/Frame 预算已交 | 981 SESSION | `ad7882cbe`（PROOF A）、`62d34e3df` | **已交可复用**；Pause 语义=stop 冻结+Step=tick(1) 已交（`1775f96d` s2） |
| 4a | `image2d.png("assets/x.png")`（身份=内容 SHA256+byte_length+IHDR） | 需资源端口类型 Image2D（字段 sha256:Str/byte_length:i32/w:i32/h:i32——Model3D 同型先例已交） | 需 resources 模块声明准入（引用/可见性沿 `check_scene_resource_refs`）；**文件读取/解码在宿主**（frontdoor 不做 IO） | 端口值=Aggregate 可构造（身份字段由宿主计算后注入——同 Model3D 拒绝纯 AV 构造的先例：`age.model3d.glb.v1` 具名拒） | UI058 driver 资源服务（PNG 载入/IHDR/缓存/许可——022 既有） | 缺文件/坏签名=载入层具名拒（022 语义）；AV 侧无文件不伪造 | 签名/类型=981；载入/生命周期=UI058 | Material2D/Mesh2D/Model3D 端口+Aggregate 已交（`62d34e3df`）；Image2D 未登记 | **981待实现**（类型/签名）；载入=**UI058待提供** |
| 4b | `frame2d.at(resources.x, x,y,w,h)` 帧表；view `image:/frame:/flip:` | 需资源端口 Frame2D（字段 img:Image2D/x/y/w/h:i32）+ 精灵叶属性面（view-2d 行 `image/frame/flip` 键） | 声明准入 + rect 越图像具名拒（需宿主给的 w/h 边界——签名面冻结，检查在宿主或受检注入） | `frame2d.at`=纯值构造→**Aggregate 可执行**（无 IO）；`image:` 绑定=资源引用事实 | UI058（帧表→场景 sprite 字段发布/退化矩形 fault 42 语义） | 越界 ordinal/flip 域具名（022 语义） | 签名/构造=981；发布/帧写=UI058 | 无 | **981待实现**（构造可执行）；组合=**UI058待提供** |
| 5 | `use` 导入/双实例局部状态/交互事件 | 已交——use/model/msg/on、逐实例状态函数、EVENT-B 载荷/绑定、`ForNode` 组件级定向 | 已交（同上全链） | 已交（[7,5]/双组件 2:1/双实例隔离） | 980 桌面 onclick→dispatch 已交 | 世代/所有权/关闭恢复纪律已交 | 981+980 已交 | `f4158e499`/`62d34e3df`/`1775f96d`；980 `7f431bd5f`/`403c936d` | **已交可复用**；最近目标/超距=Auto 逻辑（UI058 组合） |
| 6a | 查询费用/预算：原 32 credits/世界 callback（cast8/sweep16/getter1）、第 33 sticky fault 10 | 无服务级计费——AV `ExecLimits`（steps/frames/slot_bytes/wall）已交但**非 per-callback 服务信用** | 需服务调用计费面（每次服务调用记 1 条 credit、超界具名拒且零效果） | 计费在服务口派发点（981 接线）；费率表=UI058 T-02 与 981 冻结 | UI058 费率语义（原额保持） | 第 33 拒、sticky；失败零效果 | 计量接线=981；费率/映射=UI058 联冻 | `ExecLimits` 已交（av-core）；服务信用无 | **981待实现 + UI058联冻** |
| 6b | 旧句柄拒绝：render2d spawn/despawn 代次+1、旧代写 fault 33；query token stale 13 | 世代纪律模式已交（async 票钉 pid+body 代数、bridge mount 世代、晚到丢弃） | 已交（同型） | 已交（同型） | UI058 token 算法面 | 具名丢弃原因序已交（session>stale-generation>stopped） | 981 已交模式；具体 token=UI058 | `1775f96d`（R64-10） | **已交可复用**（模式）；组合=UI058 |

**判定汇总**：已交可复用=结果 getter 载体、场景行读、Tick/Pause/Step/Stop、use/双实例/交互事件、世代纪律、原子性；981待实现=query2d/image2d/frame2d/input/dt 服务登记与受检投影、f32 model 字段（若需要）、行写/移动提交接线、服务级信用计量、宿主服务口（无 provider 具名拒）；UI058待提供=v3/v4 查询算法、行状态/帧写/精灵发布、PNG 载入与生命周期、输入喂值、费率联冻；980 非阻塞（桌面按钮事件已交，模拟轴由 UI058 runner 先喂）。

**禁止项确认**：查询/碰撞/精灵算法不进语言 core（本表 1a/2b/4a 的算法列全归 UI058 中立单元）；旧 Parser/AutoVM/生成器路径不复用（021/022 旧链仅作语义冻结参考）；不冒 UI058 最低图形组合/GPU/交互/冷包验收；不签 UI058 或 981 整包完成。

2026-11-18 stage: handoff（B-W1 AV 余量交接：场景 f32 观察车道 + 组件级 2:1 调度 + AGE 异步宿主面，交 980 B-AV 汇入 UI-W1-AV）| PLAN-981 | outcome: pass（B-W1 AV 余量批；M2A-08 已签发后正式在序——985 `0db949754`/delivery `95c7fbad8`，T-A05 ✅）

**绑定 revision**：auto-lang master `62d34e3df`（代码 `c76c245fc`）＋复审修补 `637faa210`；auto-ui master `d75fba34`＋`1775f96d`（overlay 钉 `62d34e3df`→`637faa210`，1414 文件，`prepare_sdk --verify-only` PASS）。顺手修复（L0 `fix-av-configdocument`，已合入并清树）：982 合入漏的 `SourceProfile::ConfigDocument` 匹配臂（ace-av conformance 在 master 编译红）＋根 Cargo.lock 陈旧（config 文档 sha2 依赖，`--locked` 工具构建红）。

**交付面（producer→consumer：981 LANGUAGE/SESSION → 980 B-AV → UI-W1-AV）**
- **`D-981-PROOF-AV-SCENE` 关闭**：av.owned/1 lowering 映射 PLAN-961 数值具名类型表条目（对齐 mir_verify 既有"owned 吸收 data 面"契约）；AGE 端口构造器（material2d.color/mesh2d.quad/mesh2d.circle）准入并 lower 成端口 record Aggregate（字段按 AGE_PORT_FIELDS、arity 双侧核对；model3d.glb 保持具名拒——文件内容身份）；资源全局在 owned 车道真实执行（cell 拥有 record、借用写入按槽型对齐直写）；mir_verify 准入 module.global/age.* import 身份与 age2d/age3d.resource 能力（owned 限定）；**1064 redis 平台车道边界原样保留**（catch-all lower.intrinsic 拒绝恢复）。观察面：`AgeSceneValueFn` 登记进描述 + `observe_age_package` 逐属性取真值——g01 纯场景包返回**原几何真值**（红圆 -20/10/30、蓝 quad 0,0/30×30、绿圆 25/-15/24、黄 quad -45/-25/12×12 hidden），混合 UI+2D 包**一次观察**同时给状态/文本/场景 f32；参数化行具名跳过（Parametric，不猜实参）。
- **`D-981-SESSION-MULTI ①` 余量关闭**：`ScenarioOp::ForNode { node, op }` 组件级定向（未包裹 op 保"全组件重放"旧语义、对照断言兜住；未知组件具名拒 `component-scenario`；单 node 组件面对越界限定具名拒；`ForNode{Tick}` 按目标组件生命周期拼写归一）。宿主面：`AgeSession.send_to_component`/`send_args_to_component` + 驱动 wrapper；**永久双组件 2:1 用例**（s2：A=2/B=1、组件级带载荷 +5、未包裹对照 3/7）。
- **R64-10 异步面关闭**：`AgeSession.spawn_async/await_async/async_balance`（票钉 `(pid, body_generation)`；任务源先在 ACE scheduler 真实执行到静止；丢弃原因按精确度 session→stale-generation→stopped；停止会话拒 spawn）；**多轮 Run/Stop 后票账与 Run/Stop 双净归零**（spawned==settled、pending 0、runs==stops，丢弃票零调度）。

**独立复审（review-C，独立 subagent 自跑复现）**：verdict **pass**。逐项核关闭条件、车道完整性（f32 映射仅 owned 面、能力门 owned 限定）、Aggregate arity 双侧、全局 cell 拥有语义、ForNode 兼容对照、异步账本判别力（漏 await 必红）；无假值/无 debug 残留/格式干净。**发现处置**：①medium——GlobalSet Clone-on-mismatch 分支会发射 mir_verify 拒绝的 MIR（`mir.clone-type`/copyable-clone），当前覆盖下不可达（fail-closed）、登记为 `D-981-GLOBAL-CLONE-GAP`；②low——四条新诊断串长空格，已修（`637faa210`）；③info——982 ConfigDocument 臂已在本次顺手修复关闭；④info 边界——record 全局（含 Str 字段）无用户源用例、异步丢弃路径"任务真实执行"为代码级核（收据具名）。

**门禁实测**：lang worktree core+av+lowering+a2x-worker（perf:: 排除）全绿（全并行档 1064/975/946/958 系统类测试偶红、隔离复放全绿=环境归因）；master 固定组 `cargo t ace-source ace-syntax ace-av` **全 PASS**（ConfigDocument 臂+lock 修复后）；plan981 event/proof/for-node/dual-component 套件全绿。auto-ui：s2_age_routing **21/21**（含双组件 2:1 + 异步净归零）、driver lib 49/49、session s1/s3/s4 17/17、plan041 10/10、ace_ts_provider 14/14、ace_node_desktop 7/7（串行）。

**未闭（不冒充）**：`D-981-GLOBAL-CLONE-GAP`（新登记）；V-EVT-B-ORIGIN 与 TS 参数环境归 980 B-WEB；W1 其余（CONTENT/FUSION AV 域格、PROOF 完整 AV 格的 3D/参数化行、`UI-W1-AV` 汇总收据——等 980 B-AV-R 汇入）；981 整包仍 executing、不归档。

2026-11-17 stage: handoff（B-W1 事件三件正式交接：IF-981-EVENT-PAYLOAD-B + IF-981-MSG-B + IF-981-PARAMS-B，交 980 B-AV/B-WEB）| PLAN-981 | outcome: pass（B-W1 事件批，用户 /goal 直接授权开工，不等 M2A-08 排程）

**绑定 revision（语言面已落主；auto-ui 在 plan-981-dev 待 fold）**
- auto-lang master `f4158e499`（代码 `3519c2b56`）＋复审修补 `7b127afc8`（`beefe8a2c`）。
- auto-ui `d5e92deb`（plan-981-dev）：bridge/session/driver 三面 + s2/plan041/census 测试；overlay 钉 `f4158e499`（B1/B2 修补后重钉被 982 并行 WIP 未跟踪文件 fail-closed 拒绝；修补只触语言侧测试与保留名表，驱动消费面无差异，待 982 落主后可重钉）。

**三件交付面（producer→consumer：981 LANGUAGE/SESSION → 980 B-AV/B-WEB）**
- **IF-981-EVENT-PAYLOAD-B**：`ScenarioOp::SendArgs { instance, msg, args }`（`Send` 恒等价空载荷——向载荷变体空 Send 是 `source.age.component-event` arity 域错误）；`send_args_from_values` 宿主值抬升；载荷类型拼写经共享类型表（int/i32/bool/str 可承载；float/f32/ident 在派发时 `source.age.component-unsupported` 指向变体声明，fail-closed 零效果）；实参唯一 verify、`let` 一次求值、按位绑卫生局部 `__age_a_<inst>_<seq>_<pos>`；错 type 由唯一 verify 在 arg anchor 报 `source.type-mismatch`，错 arity/unsupported 在变体 span 报，全部效果前拒。SESSION：`AceStateEngine::dispatch_with_args`（槽队列**拥有** `AceStateValue` 拷贝，失败回滚、世代/所有权/晚到丢弃纪律不变）＋`AgeSession::send_args[_to]`/驱动 `age_send_args[_to]`（R64-1 原子调度、stopped 具名拒、restart 归初值）。
- **IF-981-MSG-B**：具名/匿名集合、多载荷 `(str, int)` 保拼写/声明序/原 span（span 断言指作者字节）；重名变体/未知变体引用/第二具名集合具名拒 + 同入口修复恢复（`msg-duplicate`/`msg-unknown`/`msg-set-duplicate` 沿 R65 族）。
- **IF-981-PARAMS-B**：handler 形参数 ↔ 载荷槽数**声明级**一致（未派发也核对，`component-event`）；重复 binder 拒（span 指第二形参）；裸形参名遮蔽同拼写 model 字段、`.field` 恒指实例 model；未绑定名由唯一 verify 在作者 span 拒；未派发 handler 铸隐藏检查函数 `__age_hv_…`（形参 = 真函数参数 + 声明初值，**不造默认值/常量场景**）进唯一 verify；owned-str 裸转移经 `.clone()`（单次求值后拷贝，非重求值）。

**独立 oracle（V-EVT-B 三行实跑）**：两实例 (i0,+3),(i1,+5),(i0,+4) → **[7,5]**（轨迹前缀 [3,0]/[3,5]、置零/换目标/换序负控由精确值兜住）；求值一次（状态依赖实参 + 双读取体，7≠10 判别）；`.QueryInput(v)` 两组字符串含中文 str 落各自实例（状态通道 `ComponentFieldValue::Str`）；bool 正例 true/false；多载荷 `(str, int)` 双通道对齐；param 同拼写遮蔽（5≠0 判别）；混合 unit+载荷队列 4/11。

**门禁实测**：plan981 全套 219/219（event 面 25 例：A 更新 3 + B 新 8 核心 + 7 AV + B1 bool）；ace-core 833/833；ace-core+ace-av（perf:: 按 PLAN-1016 排除）2057/2057（kv_zip_1064/sys_d01_d06_1064 仅全并行档红、隔离重放绿=环境归因；link:: 同前）；a2x-worker p981 5/5（真实 rustc 腿）；grammar 7/7（**零 grammar 改动**，冻结身份不变）；固定组 `cargo t ace-source ace-syntax ace-av` 全 PASS。auto-ui：s2_age_routing 19/19（vendored overlay）、driver lib 49/49、session s1/s3/s4 17/17、plan041 10/10（含新 typed 派发/回滚/世代例）、ace_ts_provider 14/14、ace_node_desktop 7/7（串行；并行 LNK1181=链接器临时文件竞争，环境归因）、`prepare_sdk --verify-only` PASS。**预存红（干净树 stash 验证，非本批触面）**：`cap_chart_snake_kebab_dual_acceptance`、`p980_hello_window_exe_bad_source_exits_named`、driver `authoring_feasibility` 编译缺 `hud_overlay`——归各自 owner plan。

**独立复审（review-B，独立 subagent 自跑复现）**：verdict **pass**。逐行核 §5 三键/§6 三 V 行与设计表不变量；无假值（hv 检查函数用真初值+真形参；`.clone()` 为单次求值后拷贝）、span 诚实（arity/unsupported 指变体、类型错指实参锚）、跨实例/跨 node 局部名不可串（fn 级隔离 + `state_fn_name_at` 消歧）、桥回滚只在成功观察后写状态、测试全为精确独立 oracle 非代理信号、census 修订具名有据（空 Send 语义升级 + TS 参数环境归 980 B-WEB）。2 low 已修（B1 bool 正例、B2 `__age_hv_` 保留名具名拒），修补后全套复跑绿。具名边界（不冒缺陷）：①多字段 driver 每字段状态函数各重放一次实参求值（各 fn 从初值独立重放，无可观察重复应用；判别测试为单字段）；②未派发的 unsupported 载荷 handler 体不进 hv 检查（其派发本就 fail-closed 拒）。

**未闭（不冒充）**：`D-981-EVENT-A-REMAINDER` **本批关闭**（载荷/参数执行、逐项 arity/type/dup/未绑定诊断、AV 队列/调度/世代——R69-2 替代口径的逐项诊断已由真实语义取代）；V-EVT-B-ORIGIN（inline-lambda/原 span→SFC 链 + 浏览器 DOM）与 TS provider 参数环境（census param-handler 拼接降级旧道）归 980 B-WEB 消费；W1 其余（CONTENT/FUSION AV 域格、PROOF 完整 AV 格、`UI-W1-AV` 汇总）不在本批；981 整包仍 executing、不归档。

2026-11-17 stage: housekeeping（981 worktree 构建缓存/大证据目录清理）| PLAN-981 | outcome: done

- **背景**：985/A 完成前，981 只做到 A 面并等待；两棵 worktree 的构建缓存已占 ~40 GB。
- **删除**（均为可再生缓存/证据副本，非仓库内容）：
  - `.wt/lang-981/auto-lang/target`（3.1 GB）、`experimental/ace/target`（16.8 GB）、
    `experimental/ace/crates/av/target`、`scripts/full-regression-driver/target`；
  - `.wt/lang-981/auto-ui/experimental/autographics-auto-driver/target`（15.6 GB）、
    `experimental/age-ace-session/target`（4.5 GB）；
  - `plan-artifacts/plan-981/{review65,review66,review67,review68,review-63,sdk-pin-*}`（探针 target/
    SDK 抽取副本，证据已折叠进 §9）。
  - 合计释放 **≈55.4 GB**；D: 可用空间回到 **270.2 GB**。
- **保留**：两仓源码/测试、`.sdk-source`（35.7 MB，两层 SDK 物化结果，可复现）、plan-artifacts 中
  小脚本与收据（17.6 MB）。两棵 worktree `git status` 均 0 修改，`wt-guard` clean。
- **恢复方式**：需要再跑测试时，在对应 worktree 直接 `cargo test`/`cargo t ace-source ace-syntax`
  会重建 target；SDK 需要重物化时跑 `python tools/refresh_overlay.py D:/autostack/auto-lang`。
2026-11-17 stage: done（tier-1 A 面终验：固定组干净 PASS）| PLAN-981 | outcome: pass（tier-1 A 面）

- **固定组（终验）**：`cargo t ace-source ace-syntax` → **gate_passed=true**（run-ip6ed8jk，353.3s）；
  两组 PASS，无并发改树干扰。
- **四命令**：`cargo test -p ace-core --test conformance plan981` **207/207**、`cargo test -p ace-core`
  **742/742**（+lib 79/1 ignored）、`cargo test -p ace-av --test conformance p981` **6/6**、
  `cargo test -p ace-a2x-worker --test conformance p981` **5/5**、`cargo test -p ace-syntax-grammar`
  **45/45**；auto-ui driver lib **49/49**、`s2_age_routing` **16/16**、bridge 4+3+10、
  V-B-SESSION 规定命令 4/4 + 7/7；overlay 1393 钉 `260070b10`，逐 blob 保真 PASS。
- **tier-1 A 面结论**：AC-HANDOFF/B-SOURCE 收口（979 已核）+ B-SESSION-I/V[A] + B-RUST-I/ V[A] +
  B-LANGUAGE-I/V[A] 全部交付、永久用例、独立复审（62/64/65/66/67/68/69）、逐批落主；B 波余量具名
  （B-SESSION[B] 两组件 2:1/异步资源、B-W1 载荷/参数执行、B-W3 运行时 dispatch/AGE 端口/GPU/音频、
  模块级 const/var A2R、R59-5、EXIT-CONTRACT 重核）。**未改验收口径、未声称 plan-level execution_done。**
2026-11-17 stage: review（review-69 = pass：tier-1 A 面可交；R69 处置）| PLAN-981 | outcome: pass（tier-1 A 面）

- **review-69 = pass**：独立复核确认 tier-1 A 面（SOURCE 收口、LANGUAGE-I/SESSION-I 记录与落主、RUST 公共链、
  V[A] 三腿、event A）可交；§8 仍 **3/35** 勾选、`status: executing`，**未声称 execution_done**。
- **R69-1（low）**：V-EVT-A-UNIT 改为"终值 + 单步 oracle"，新增 `Inc,Inc,Reset → 0` 判别例（Reset 若
  no-op 会剩 2 → 红）：`cedf5331c`→master `c286fb8d0`。
- **R69-3（low）**：`D-981-EXIT-CONTRACT` owner 由已归档 979 改判 **981/LANGUAGE**，关闭条件写明；
  R69-4（债务文件字面 `\n`）同批修正（`260070b10`）。
- **R69-5/R69-6**：review-68 的 R68-2 措辞项并入债务"下次触碰"；roadmap 里过期的 review-67/1391 口径
  已更新为 review-68 pass / 1393。
- **R69-2（low，契约口径）**：985 UI-EVT-BIND 的"逐项 arity/type/dup/未绑定"诊断在 A 以**一律不可执行
  的具名拒**兜住（fail-closed + 零效果），逐项诊断归 B-W1（`D-981-EVENT-A-REMAINDER`，已具名）；
  985/T-A03 记录该替代口径。
- **R69-7/R69-8（info）**：重复 pin 提交与既有警告均非 981 触碰面，保持记录。
- **最终证据**：plan981 **207/207**、plan980 13/13、AV p981 **6/6**、A2R p981 5/5、grammar 45/45、
  ace-core **742/742**；auto-ui driver lib **49/49**、`s2_age_routing` **16/16**、bridge 4+3+10、
  V-B-SESSION 规定命令 `authoring_session 4/4 + mixed_regions_lifecycle_flow 7/7`；
  overlay 1393 文件钉 `auto-lang 260070b10`，逐 blob 保真 PASS。
- **tier-1 结论（A 面）**：已交并全部独立复审（review-62/64/65/66/67/68/69）；**B 波余量**（B-SESSION[B]
  两组件 2:1 + 异步资源、B-W1 载荷/参数执行、B-W3 运行时 dispatch/AGE 端口/GPU/音频、模块级 const/var
  A2R、R59-5、EXIT-CONTRACT 重核）逐项具名，未以 A 面替代。
2026-11-17 stage: work（V-B-SESSION / V-B-LANGUAGE[A] 实跑证据）| PLAN-981 | outcome: partial

- **V-B-SESSION 规定命令实跑**（auto-ui 主检出 master）：`cargo test --manifest-path
  experimental/autographics-auto-driver/Cargo.toml --test authoring_session --test
  mixed_regions_lifecycle_flow -- --test-threads=1` → **authoring_session 4/4**、
  **mixed_regions_lifecycle_flow 7/7**，exit 0（A 子面 AV 宿主腿；两组件 2:1 与异步资源净归零
  仍列 B 波，见 V-B-SESSION[B]）。
- **V-B-LANGUAGE[A] 实跑**：① V-L 判据在真实 A 源上（`plan981_ma_sources.rs`，前一条收据）——
  出口选择/事件绑定/构造分母/原 span；② **V-EVT-A-UNIT**（auto-lang `0b98c143b`→master
  `230e4abc6`，`p981_evt_a_unit_counter_trace_and_named_mirror`）：匿名 unit Counter 轨迹
  **0→1→0→0** + 中间步 oracle（Inc→1），具名镜面 `msg Msg { … }` 同语义并暴露集合名。
- **V-EVT-A-GRAMMAR-ID**：已在 §9 前条（#6i 差额 + 两冻结测试 + 邻居不动 + 语义注释）。
- **V-EVT-A-ADMIT**：合法 unit 近邻可派发；带参/带载荷派发按 A 口径**具名拒且零效果**
  （`event-params-unsupported` / `event-payload-unsupported`）；**错 arity/type、重复 binder、
  未绑定名**的**逐项**诊断归 B-W1（`D-981-EVENT-A-REMAINDER`），A 以"参数一律不可执行"兜住。
- **门禁**：AV p981 **6/6**、plan981 207/207、plan980 13/13、A2R 5/5、grammar 45/45；
  auto-ui `s2_age_routing` 16/16、driver lib 49/49、bridge 4+3+10。
2026-11-17 stage: handoff（MIN-A-AGE 整体正式交接：IF-981-EVENT-A + B-PROOF-I/V/R[A]）| PLAN-981 | outcome: pass（A 面整体交接）

**去向**：985/A（T-A03/A04）按原子行接收；本包不新增 Plan、不勾整任务、不声称 plan-level execution_done。

**绑定 revision（全部已落主、独立复审 pass）**
- 语言/SOURCE 面：auto-lang `0520c7f3b`（代码 `6340934a1`、`ad7882cbe`）；event A `24b4ba4a7`/`75015a49b`；V[A] `0b98c143b`/`230e4abc6`。
- 宿主面：auto-ui `ab6fff5c`（代码 `658ad515`/`1eb27697`；SESSION-V `771be6a9`/`9032d69c`）。
- 两层 SDK：base 681 + overlay **1395** 文件，overlay 源 = `auto-lang 0520c7f3b`，逐 blob 保真 PASS；`.sdk-source` 复核 PASS。
- Schema/profile：age-package `145617968fcc6d06…5199`、age-component `96ecc4372a53665e…3708`（#6i 语义差已注释；13 邻居 profile 未动）。

**独立 oracle / producer→consumer**
- 2D：g01 原三模块源（`fixtures_g01`，与 SOURCE 批同源；SHA256 前 16 `9b809a1c…`/`6f993775…`/`dbdc5243…`）→ 前门 4 行场景事实（位置/尺寸/可见性 CPU 值对齐原 G01 基线，变异即红）；AV 腿：纯场景包观察为空、混合 UI+2D 具名拒 F32。
- 3D：g08/3d **派生真实源**（`fixtures_proof/g08_3d_cylinders.at`；来源 `main.at` `4bef459e4f854715`、基线 `scene.agscene` `46b2bb9ca50dffd1`）→ `view-3d` 4 行位置/缩放对齐原 scene。
- UI/事件：组合包（UI+2D+3D 三节点）出口选择互不跨域；1 vs 3 出口预算同为 Start=1/Tick=5/Frame=3；UI 事件 2:1 实例状态与文本；宿主腿 event→update→坏编辑具名拒保旧→修复恢复→stop 冻结观察/拒派发→restart 回初值。
- event A：见前条 IF-981-EVENT-A 交接包（具名/匿名 unit、重复/未知/双集合拒、参数保留但派发拒、payload 声明合法/派发拒、#6i 身份）。
- producer→consumer：981 SOURCE/LANGUAGE/PROOF（producer）→ AV 观察腿 + 985/A 认证（consumer）；980/A 源（`403c936d`/`7f431bd5f`）为正式输入。

**复审链**：review-62（LANGUAGE-I）、review-64（SESSION-I）、review-65→66（event A）、review-67→68（V 证据）、review-69（tier-1 A 面终审）、review-70→71（PROOF A，**pass**）；R69-1 修补已落主（`cedf5331c`），R65/R67/R69 低危项均已处置或具名。

**未闭（不冒充，按原 owner/波次）**
- `D-981-PROOF-AV-SCENE`：AV 的 2D/3D/混合观察格（f32 不在 `av.owned/1`）→ B-AV；A 只在门面描述事实，场景+model 在 AV 具名拒。
- `D-981-EVENT-A-REMAINDER`：B-W1 的载荷/参数**执行**、逐项 arity/type/dup/未绑定诊断、AV 队列/调度/世代。
- `D-981-SESSION-MULTI ①`：两组件事件 2:1 与异步资源净归零 → V-B-SESSION[B]；R64-10 异步面。
- `D-981-MODULE-GLOBAL-A2R`：模块级 const/var 的 A2R 目标合法化 → B-W3。
- B-W3：运行时 dispatch/AGE 端口/GPU/输入/音频；`camera3d`/`character3d`/`solid` 与 g03-3d 原始叶/层级集；完整 Showcase。
- R59-5（坏槽位诊断 @0，owner syntax-grammar）；`D-981-EXIT-CONTRACT` 已关（旧 979"恰一出口" oracle 失效，新契约永久用例在 `plan981_v_harness`/`plan981_proof_a`）。
- R64-1..7 已修（原子调度/诊断/多组件/命名/代数/overlay/rustfmt），R64-8/9 无登记，R64-10 见上。

**门禁实测**：plan981 **211/211**、plan980 13/13、AV p981 **9/9**、A2R p981 5/5、grammar 45/45、ace-core **746/746**；driver lib 49/49、`s2_age_routing` 17/17、bridge 4+3+10、V-B-SESSION 规定命令 4/4 + 7/7；`prepare_sdk --verify-only` PASS；overlay 1395 逐 blob 保真 PASS。
2026-11-17 stage: review（review-68 = pass；事件 A / V 证据批闭）| PLAN-981 | outcome: pass（批级）

- **review-68 = pass**：R68-1 闭包核销；三个真实源 session 测试全部 `include_str!` canonical 源、
  手抄副本 **0**；轨迹/隔离/重启断言与措辞核验通过；overlay 路径集合与逐 blob 保真
  **1393/1393 PASS**（`auto-lang 75015a49b`）；门禁全绿。
- **信息项（不阻塞）**：R68-3 收据里 driver lib 数字过期（`3e5398ab` 时 47/47，当前 **49/49**，
  因并行 plan-080 新增 2 个 lib 用例）；R68-4 代码注释仍把 `age_identity()` 的首值写作 PID
  （实为逻辑计数器），已并入 `D-981-EVENT-A-REMAINDER` 的下次触碰项。
- **批级结论**：event A（IF-981-EVENT-A）+ V 证据（B-LANGUAGE-V[A]/B-SESSION-V[A]/B-RUST-V[A]）**闭**，
  可绑定 revision：auto-lang `24b4ba4a7`/`75015a49b`、auto-ui `771be6a9`/`7de28330`。
  **不声称** B-LANGUAGE-V/B-SESSION-V/B-RUST-V 全项完成（B-W1/W3 余量见 §7/§8 与债务表）。
2026-11-17 stage: review（review-68 R68-1 修补）| PLAN-981 | outcome: partial

- **review-68 = needs_fix（仅 R68-1）**：R67-1/2/3/4/6/7/8 已核销；R67-5 只部分闭合——旧
  `s2_real_a_counter_dual_instance_2_to_1` 仍内嵌一份 818 B 手抄源。
- **修补**：auto-ui `f4edcf7b`→master `771be6a9`——该测试改为 `s2_real_a_counter_trajectory_and_restart`，
  直接 `include_str!` **canonical 源**，保留 Reset 归零轨迹覆盖，重启证据改述"逻辑 session id 稳定 +
  generation 递增"；三个真实源 session 测试全部读 canonical 文件，无手抄字面量。
- **overlay 复核**：`5cbe66ed`→master `7de28330`，1393 文件钉 `auto-lang 75015a49b`，
  `prepare_sdk --verify-only` PASS，逐 blob 保真 PASS。
- **门禁**：`s2_age_routing` **16/16**、driver lib 47/47、bridge 4+3+10。
- **待办**：review-68 复验 R68-1 闭包（已发复验请求）。
2026-11-17 stage: review（review-67 needs_fix → 修补；口径修正）| PLAN-981 | outcome: partial

**review-67 = needs_fix（8 项，全部修补）**：
- R67-1（medium，口径）：本批 A2R 证据改称 **B-RUST-V[A]**（真实 rustc 值切片，编译期展开重放），
  并把 **B-W3 余量**（运行时 dispatch、AGE 端口、GPU/输入/音频与关闭释放）加入未闭清单——
  不把值切片说成 B-RUST-V 全项。
- R67-2（low）：`V_PENDING_INPUTS` 的 session 项改写为 **V-B-SESSION[B]**（两组件事件 2:1 +
  多 Run/Stop 资源净归零异步面），A 子面已交（单组件双实例隔离/重启）不再列为未解输入。
- R67-3（low）：隔离断言可在"实例 1 派发被丢弃"时失败——首次 `send_to(1, Inc)` 后立刻断言 `[2,1]`，
  结束值保持非零。
- R67-4（low）：A2R oracle 从后缀判断改为**精确值**比较（`value:1` / `value:5`）。
- R67-5（low）：auto-ui 侧测试直接 `include_str!` **canonical 源**（examples 文件），ACE fixture 用
  长度 818 钉住并注明 SHA256 前 16；不再依赖三份手抄字面量。
- R67-6（low）：删除主检出里游离的 `tests/s2_age_routing.rs`（旧快照、未跟踪）；两个测试文件
  rustfmt 清理。
- R67-7（info）：重启证据改述为"逻辑 session id 稳定 + generation 递增"（`pid` 是逻辑计数器，
  不是进程身份）。
- R67-8（info，口径更正）：此前"1361/1361 保真"是**自参照分母**——在 `862f8694e` 三个根下实有
  **1386** 个跟踪文件，旧清单只 1361（漏 25，含整个 a2x-ts crate 与 `plan981_ma_sources.rs`），
  原因是旧工具复用既有清单路径而**不是** CRLF 污染；新工具用 `git ls-files` 派生路径
  （现在 1393，随后续提交增长），逐 blob 保真 PASS。
- **落主**：auto-lang `ec9874d7f`→master `24b4ba4a7`；auto-ui `9032d69c`→`9a0f0ed5`，overlay
  `731f5be2`→`3e5398ab`（钉 `24b4ba4a7`，1393 文件，保真 PASS）。
- **门禁实测**：plan981 **207/207**、plan980 13/13、AV p981 **5/5**、A2R p981 5/5、grammar 45/45；
  driver lib 47/47、`s2_age_routing` **16/16**、bridge 4+3+10；`prepare_sdk --verify-only` PASS。
- **未闭**：review-68（needs_fix 闭包核销）；B-W3 余量（R67-1）；`D-981-EVENT-A-REMAINDER`。
2026-11-17 stage: work（B-LANGUAGE-V：V-L 判据 + AV 腿跑在真实 A 源上）| PLAN-981 | outcome: partial

- auto-lang `81d423858`→master `777c68b3b`：`plan981_ma_sources.rs` 增加真实 A 源的 V-L 判据——
  V-L-VIEW（单 view 出口；UI 选它、场景请求具名拒）、V-L-WIRE（3 unit 消息/3 handler/onclick 绑定，
  绑定 span 指回真实源文本）、V-L-COVER（node/model/on/view 四类事实）、消息变体**原 span 保真**；
  AV 腿（`p981_ma_counter_av_leg`）用同一源同一 2×Inc+1×Dec 得状态 **1** 与真实 view 文本
  `"Counter"`，与 A2R 二进制 oracle 同值（AV/A2R 两腿互证）。
- `V_PENDING_INPUTS` 从 3 项降为 **2 项**：980/A 正式源已交（本项移除）；剩 typed UI/资源宿主面
  （R64-10 异步面）与 980 合流 oracle。
- auto-ui `9a8d12a3`→master `979d4cfd`：overlay 钉 `auto-lang 777c68b3b`（1391 文件），逐 blob
  保真 PASS；`.sdk-source` 由该提交的 `git archive` 抽取物化（工作树 EOL 差异不影响）。
- **门禁实测**：plan981 **207/207**、plan980 13/13、AV p981 **4/4**、A2R p981 5/5、grammar 45/45、
  ace-core **738/738**；driver lib 47/47、`s2_age_routing` 15/15、bridge S1 4 + S3 3 + S4 10。
- **未闭**：review-67（SESSION-V/RUST-V 批）待回；typed UI/资源宿主面与 980 oracle 仍 pending；
  B 侧载荷/参数执行（`D-981-EVENT-A-REMAINDER`）。
2026-11-17 stage: work（B-SESSION-V 双实例 2:1 + B-RUST-V 真实 A 源 A2R + overlay 字节保真）| PLAN-981 | outcome: partial

**B-SESSION-V（real A 源）**：auto-ui `332ef407`→master `0a87676b`——用交付的 980/A Counter 源开
两实例，2:1 定向事件（实例 0 两次 Inc；实例 1 一次 Inc 一次 Dec）→ 状态 `[2, 0]`；随后实例 1 的
Inc/Inc/Reset 不影响实例 0；Stop→Restart 后回到 `[0, 0]` 且 PID 不变。`s2_age_routing` **15/15**。
**B-RUST-V（real A 源）**：auto-lang `eb487f3cf`→master `399edd447`——同一交付 Counter 源（core 测试
fixture，与 `plan981_ma_sources.rs` 共享）经公共 A2R 链（SemanticView → lower → fileset → 真实 rustc →
运行）2×Inc+1×Dec 输出 **1**，对独立算术 oracle；`ace-a2x-worker` p981 **5/5**。
**两层 SDK 可复现性（R65-7/R66-3 收尾）**：overlay 路径改为 `git ls-files` 的三个根（1391 文件，
含 1064 新增 stdlib）；字节改由 `git archive <commit>` 抽取（避免工作树 CRLF 污染，保留故意 CRLF
fixture 的原字节）；auto-ui `736381e0`→master `4e7a2fb0`；`prepare_sdk --verify-only` PASS，
逐 blob 保真核验 **1391/1391 PASS**（`auto-lang 399edd447`）。
**门禁实测**：plan981 206/206、plan980 13/13、ace-a2x-worker p981 5/5、AV 3/3、grammar 45/45；
driver lib 47/47、`s2_age_routing` 15/15、bridge S1 4 + S3 3 + S4 10。
**未闭**：B-SESSION-V / B-RUST-V 的**独立复审**（下一轮交 reviewer）；B 侧载荷/参数执行、AV 消费
（`D-981-EVENT-A-REMAINDER`）。
2026-11-17 stage: review（event A 批：review-65 needs_fix → review-66 pass → R66 修补）| PLAN-981 | outcome: pass（批级）

- **review-65 = needs_fix（11 项）**：R65-1..11 全部修复（`a5214be59`→master `49409c6fb`；auto-ui
  `fdd3ba69`→`ff10e990`），详见上一条收据。
- **review-66 = pass**：11 项逐条核销（跨块重复、payload 判定、未知消息、双具名集合、#6i 注释、
  实测数字、overlay 逐 blob 保真、参数派发拒绝、簿记落主、注释措辞、覆盖）；证据含 `git archive`
  抽取上独立 probe（20 例）与 tool-guard 测试（派生 stamp/拒绝错 stamp/拒绝脏路径/非 git）。
- **R66 残余（Low/Info）修补**：R66-1 payload 守卫只对**真的处理该消息**的 node 生效
  （`ed807f393`→master `862f8694e`，+跨组件回归用例）；R66-2 payload 谓词标注为纵深防御并补注释；
  R66-3 `refresh_overlay.py` 文档写明 `--verify-only` 只比 manifest↔`.sdk-source`、不回溯 git；
  R66-4 #6i 注释恢复独立成行。
- **SDK 可复现性**：overlay 钉 `auto-lang 862f8694e`；`.sdk-source` 由 `git archive` 该提交物化
  （`plan-artifacts/plan-981/sdk-pin-862f8694e` 临时抽取，验后删除），1361/1361 blob 保真 PASS。
- **门禁（当前 master 实测）**：plan981 **206/206**、plan980 13/13、ace-core **737/737**、AV 3/3、
  A2R 4/4、grammar 45/45；driver lib 47/47、`s2_age_routing` **14/14**、bridge S1 4 + S3 3 + S4 10；
  `prepare_sdk --verify-only` PASS。
- **未闭**：B 侧参数/载荷执行与 AV 消费（`D-981-EVENT-A-REMAINDER`）；981 事件 A 交接包以
  `49409c6fb`/`ff10e990`（含 R66 修补后 `862f8694e`/`e450d8a0`）为可绑定 revision。
2026-11-16 stage: work（980/A 真实源编译 + review-65 needs_fix 修复）| PLAN-981 | outcome: partial

**批次 A（真实 A 源）**：`eb026ab6a` → master `a4d94b567`——把交付的 980/A 两源作为 fixture
（`tests/fixtures_ma/`，记录来源路径与 SHA256 前 16）纳入永久用例；`onclick: .Msg` 投影为
**事件绑定事实**（`view_events`）而不再当 `self` 字段引用（此前整源报 `no local named self`）。
auto-ui `c9b9876e`→`f6802437`：宿主可见绑定事实（`view_events_of`/`age_view_events`）。
**review-65 = needs_fix（11 项）**，本轮逐项修复（`a5214be59`→master `49409c6fb`；auto-ui
`fdd3ba69`→`ff10e990`）：
- R65-1 **跨 msg 块重复变体**具名拒（去重移到整节点范围）；
- R65-2 payload 守卫改 `find(|v| !v.payload.is_empty())`（unit 在前不能掩盖载荷变体）；
- R65-3 widget 形 handler 指向未声明消息 → `source.age.msg-unknown`；
- R65-4 第二个具名集合 → `source.age.msg-set-duplicate`（不再静默覆盖）；
- R65-5 grammar 身份基线补上 **980 #6i 语义差注释**（`b043117b8`，两段均 `[age-component, age-package]`，
  邻居 13 行未动）；
- R65-6 收据数字按实测重算（plan981 205/205、ace-core 736 总数）；
- R65-7 overlay 刷新工具改为 **HEAD 推导 stamp + 拒绝脏路径**；从落主 `49409c6fb` 干净检出重生成，
  并对 1361 个 blob 逐项核验 → **OVERLAY-FIDELITY PASS**；`prepare_sdk --verify-only` 仅证明
  manifest 与 `.sdk-source` 一致（已在工具文档注明）；
- R65-8 带参 handler 改为**解析/描述可达、派发时具名拒**（`event-params-unsupported`，span 指参数名）；
- R65-10 注释改为"前门编译期拒绝、宿主透出"。
**门禁（落主后实测）**：plan981 **205/205**、plan980 13/13、ace-core **736/736**（+79、1 ignored）、
AV 3/3、A2R 4/4、grammar 45/45；driver lib 47/47、`s2_age_routing` **14/14**、bridge S1 4 + S3 3 + S4 10；
`prepare_sdk --verify-only` PASS。
**R65-9**：本条与本批收据随下一次簿记提交落主，供 985 T-A03/A04 绑定 revision（985 计划文件的
EVENT-SPLIT 段由 985 owner 维护，本会话不代提交其工作树 WIP）。
**未闭**：B 侧参数/载荷执行与 AV 消费（`D-981-EVENT-A-REMAINDER`）；review 需独立复审判 needs_fix 是否闭合。
2026-11-16 stage: work（980/A 已交源 ⇒ 事件 A 子面落地 + IF-981-EVENT-A 交接包）| PLAN-981 | outcome: partial

**依据（985 r24 / EVENT-SPLIT-20261116）**：980/A 正式源已交（`IF-980-NODE-DESKTOP-A`，auto-lang
`7f431bd5f`／auto-ui `403c936d`），981/A 不再等 980 整包；本轮把三个 A 事件键在 981 侧交到可复审状态。
**auto-lang（`01c90e269`，master）**：
- `msg Msg { … }` 具名段投影（`message_set_name`）、变体/原 span 保真、**重复变体具名拒**
  （`source.age.msg-duplicate`）；
- handler 参数名/span 被读取保留（`handler_params`），但**带参 handler 在 admit 前具名拒**
  （`source.age.event-params-unsupported`，span 指参数名）——解析成功 ≠ 可执行；
- 向声明了 payload 的消息投递**无值 Send 具名拒**（`source.age.event-payload-unsupported`），
  不静默丢参跑常量体；
- AGE 两侧 grammar 身份按 980 WEB #6i（具名 msg 段 + handler 参数段）**重新基线**：
  age-package `14561796…5199`、age-component `96ecc437…3708`；13 个邻居 profile 身份未动，
  合法/坏源/普通标识符对照用例在 `plan981_event_a.rs`。
**auto-ui（`4e3ab1f7`→`90fdc84c`，master）**：宿主层 payload 投递具名拒且**零效果/零 dispatch**；
unit 投递仍成功；两层 SDK overlay 刷新到 A 源树（`tools/refresh_overlay.py`），`prepare_sdk
--verify-only` PASS（681 base + 1361 overlay，源 = auto-lang master `96e58cb67`）。
**门禁实际结果**：plan981 **198/198**（新增 `plan981_event_a` 5 条）、plan980 13/13、ace-core
**730/730**（+79）、AV `p981_age_package` 3/3、A2R `p981` 4/4、grammar 45/45；driver lib 47/47、
`s2_age_routing` **11/11**（+2）、bridge S1 4 + S3 3 + S4 10。
**IF-981-EVENT-A 交接包（供 985 T-A03/T-A04）**：
- repo/SHA：auto-lang `01c90e269`（+ `42b98f923`/`160f3bba1` 清理）、auto-ui `4e3ab1f7`/`90fdc84c`；
  A 源 `7f431bd5f`/`403c936d`；WEB #6i `b043117b8`/`876c99a3`。
- 源闭包 SHA256 前 16：`ace-hello-minimal/src/front/app.at` `7f9f79a7714a3fb5`、`app-body.at`
  `9a4d78141294526a`、`app-a2r.at` `614d7d5600f1b3e1`、`ace-node-counter/src/front/app.at`
  `6016eb863f94499e`（与 985 表一致）。
- Schema/profile：age-package `14561796…`、age-component `96ecc437…`（冻结测试内注明 #6i 语义差）。
- SDK/host/provider ABI：两层快照 base 681 + overlay 1361（`sdk/overlay/overlay.json` 记录源提交）。
- 用例/独立期望位置：`crates/core/tests/cases/plan981_event_a.rs`（5）、`plan981_grammar_scope.rs`
  （新基线 + 邻居不动）、driver `tests/s2_age_routing.rs`（+2，宿主零效果）。
- 明确支持/具名拒：具名+匿名 unit 消息、变体/span、重复拒、参数保留、参数执行拒、payload 声明
  合法 / 投递拒。
- 未闭（B 侧）：参数与载荷的**执行**语义、声明→typed 参数→verify 绑定、AV/WEB 消费（归
  B-LANGUAGE/B-SESSION [B-W1] 与 980 B-AV/B-WEB）；本交接不声明 MIN-A-AGE 全 UI/2D/3D 通过。
2026-11-16 stage: blocked（自动轮次终点：V 批仍等 980 正式源）| PLAN-981 | outcome: blocked

**终态核对**：auto-lang master `8d0468cdb`、auto-ui master `c74f8755`；本计划已落主提交仍是两树祖先
（`bd0698890` / `80d1cc06` 均可 ancestor 验证）。两仓工作树分别有 63/89 项**他人** docs WIP（未触碰）。
**已交并复审/落主**：SOURCE 批（979 已核）；LANGUAGE-I（review-62 pass）；SESSION-I（review-64 pass）；
RUST 公共链（A2R `p981` 4/4 真实 rustc，模块级 const/var 具名缺口）；R62-7 已闭；V harness 冻结非 980
判据（plan981 193）。
**阻塞（同一条件连续多轮）**：V-B-LANGUAGE / V-B-SESSION / V-B-RUST 需要 **980 正式 UI/事件源**；980 仍在
推进（已到 WEB 切片#5 落主、plan round 29），但 **status 仍 executing**，未交付正式源。
**恢复路径（§9 已写）**：980 就绪 → 替换 `V_PENDING_INPUTS` → 五组门禁 + 固定组 → 独立 phase 复审 →
fold → 勾对应批。**不冒充**：未把 I 切片当 V 交付；未改验收口径；未声称 plan-level execution_done。
2026-11-16 stage: work（最终门禁 + 状态定格）| PLAN-981 | outcome: partial

- **固定组（最终、干净树）**：`cargo t ace-source ace-syntax` → **gate_passed=true**（run-z73zct3o，
  603.3s；此前几次 `gate_passed=false` 均为其他会话并发改树的 `inputs changed` 环境归因）。
- **auto-lang master `bd0698890`**：plan981 **193/193**、plan980 13/13、ace-core **724/724**（+79、
  1 ignored）、AV `p981_age_package` 3/3、A2R `p981` 4/4（真实 rustc）、grammar 45/45。
- **auto-ui master `80d1cc06`**：driver lib **47/47**、`s2_age_routing` **9/9**、bridge **S1 4 + S3 3 +
  S4 10**、`verify_overlay` PASS（681+1361）。
- **tier-1 状态**：SOURCE 批已核；LANGUAGE-I（review-62 pass）、SESSION-I（review-64 pass）、RUST 公共链
  已交并落主；R62-7 已闭；R59-5 已证投影侧不可修（owner syntax-grammar）。**V 批仍等 980**（round 29，
  WEB 切片#4 落主，仍 executing）。
- **下一可用动作**：980 正式源就绪 → 按 §9 交接说明跑 V（替换 `V_PENDING_INPUTS`、五组门禁、独立复审、
  fold、勾批）；否则只余具名债务。
2026-11-16 stage: work（R62-7 修复 + R59-5 决定性证据 + 落主）| PLAN-981 | outcome: partial | commit `536f5b316`（master `bd0698890`）

- **R62-7 已闭**：`.a.b` watch 修饰形此前整体不可解析。改为独立规则 `age-watch-modifier`（`dot(same-line)`
  + `ident(modifier)`），源自身允许换行；投影从子规则找 modifier，行内保留 `alpha.x` 完整拼写。测试覆盖
  单行与多行两种写法（plan981 193/193）。
- **R59-5 决定性证据（保持 pending）**：加调试后发现坏声明里**每个 token 都是 error 叶**（0..4、5..6、
  …、119..120 共 22 个），投影层无法据此定位失败槽位；且 `structural_first_diag` 在 AGE 包面走
  `profile=TextRecord, age_face=true` 路径（此前 profile 判断漏掉）。结论：**投影侧不可修**，必须由语法
  引擎记录失败 token（syntax-grammar owner）或改进恢复。实验已回滚。
- **其它证据**：本树 plan981 **193/193**、plan980 13/13、ace-core **724/724**（+79）、AV 3/3、A2R 4/4、
  grammar 45/45；grammar 保持 LF。
- **债务更新**：R62-7 关闭；R59-5 补记 owner/证据；V 批仍等 980（round 28，仍 executing）。
2026-11-16 stage: work（R59-5 投影侧复探；无代码变更）| PLAN-981 | outcome: pending | tree = master

**复探结论（新增证据）**：在 `cst/project.rs` 的 `structural_first_diag` 里加 AGE 专属分支
（error 区域找成员关键字 / head 之后源文本行首成员关键字）后，坏槽位仍报 `@0..4 expected fn, found node`
——说明该诊断**不是**从 `structural_first_diag` 的 error 节点路径产生的（分支未命中），还有第三个产生者
（可能在 walk_expectations 或 `first_diag` 的另一条早退）。实验已全部回滚，树 clean、plan981 192/192。
**后续修法需先定位产生者**（建议：在 `Diagnostic::new("source.syntax", "expected fn, found …")` 的构造点
加临时断点/回溯，或在 `compile_age_package` 的错误汇合处按 code+message 打标记），再决定加 AGE 专属映射；
owner 仍为 981/LANGUAGE + syntax-grammar。
2026-11-16 stage: docs（交接说明：980 就绪后 V 批的执行路径）| PLAN-981 | outcome: handoff

**现状**：tier-1 的 I 切片与 SOURCE 已交并落主（auto-lang `f91b50fc9`、auto-ui `07821d49`），两棵 worktree
`wt-guard clean`、工作树干净；V 批等 980 正式源（980 `executing`，round 26 12/32）。
**980 就绪后的 V 步骤（不需重新设计）**：
1. 取 980 的正式 UI/事件源与 oracle 期望文件，替换 `plan981_v_harness.rs` 里 `V_PENDING_INPUTS` 三项；
   跑 `cargo test -p ace-core --test conformance plan981`（不依赖 980 的 6 条 V-L 判据已冻结）。
2. V-B-LANGUAGE：在 `experimental/ace` 跑 plan981/plan980/AV/A2R/grammar 五组 + 固定组
   `cargo t ace-source ace-syntax`；对 980 正式源核 V-L-VIEW/COND/TIME/WIRE/COVER 与真实 consumer。
3. V-B-SESSION：在 auto-ui 用 `tools/apply_overlay.py <auto-lang>` 物化两层 SDK，跑
   `cargo test --lib`（47）、`--test s2_age_routing`（9）、bridge（S1 4 / S3 3 / S4 10）；补
   `V_PENDING_INPUTS` ②（typed UI/资源接口 + 多 Run/Stop 净归零宿主面；R64-10 异步面）。
4. R 阶段：独立 phase 复审（沿用 review-62/64 的口径），pass 后 fold，勾对应批。
**不要做**：把 I 切片当 V 交付；用 981 自造样本替代 980 正式源；在 V 里改验收口径。
2026-11-16 stage: work（tier-1 终态审计：两仓一致性/交付/阻塞/债务）| PLAN-981 | outcome: partial

**两仓一致**：auto-lang master `f91b50fc9`（branch `plan-981-dev` tip `c76ed5d0b` 已是祖先）；auto-ui master
`07821d49`（branch tip `c58d9bb8` 已是祖先）。计划 §8 勾选如实：B-SOURCE-I/V/R `[x]`，其余批（SESSION /
RUST / LANGUAGE / PROOF / CONTENT / RUST2D/3D / FUSION / AUDIT）保持 `[ ]`——I 切片交付不等于批关闭。
**门禁（两仓 master）**：auto-lang plan981 **192/192**、plan980 13/13、ace-core 79(+1 ignored)+**717/717**、
AV `p981_age_package` 3/3、A2R `p981` 4/4（真实 rustc）、grammar 45/45；固定组两组 PASS（整门
`gate_passed=false` 仅因其他会话并发改树）。auto-ui driver lib **47/47**、`s2_age_routing` **9/9**、bridge
**S1 4 + S3 3 + S4 10**、`verify_overlay`/`prepare_sdk --verify-only` 双 PASS。
**复审**：LANGUAGE-I review-62 **pass**；SESSION-I review-64 **pass**；SOURCE 已由 979 核。收据 94 条。
**阻塞（不冒充交付）**：980 仍 `executing`（round 26，12/32）⇒ V-B-LANGUAGE / V-B-SESSION / V-B-RUST /
V-B-LANGUAGE-V 不能开跑；V harness 已把不依赖 980 的判据冻结（plan981 192）。
**具名债务**：`D-981-MODULE-GLOBAL-A2R`（模块级 const/var，RUST 批）、`D-981-SESSION-MULTI` ①
（组件级 2:1 调度需 ACE 场景扩展；② 已闭）、`D-981-EXIT-CONTRACT`（979 按落主 SHA 重核多出口契约）；
R59-5（坏槽位 @0）、R62-7（`.modifier` 语法）、正式 SDK pin（978/986/987）。
**下一轮**：在投影侧尝试 R59-5（把 AGE 面首错映射定位到槽位/成员头），仍不碰 V 的验收口径。
2026-11-16 stage: work（R64-10 部分：多 Run/Stop 账面净归零）| PLAN-981 | outcome: partial | auto-ui `c58d9bb8`（master `07821d49`）

- typed 宿主新增显式 Run/Stop 记账：`open` 算第一次 Run；`stop()` 冻结观察值并 `stops += 1`；`run()` 同 PID
  重启并返回 Run 计数；`run_stop_balance() -> (runs, stops, active)`，多轮 Run/Stop 后 `runs == stops`
  （无悬挂运行句柄）。资源事实此前已交；**AGE 异步面仍属 V 批**（R64-10 剩余）。
- **证据**：bridge **S1 4 + S3 3 + S4 10**；driver lib **47/47**、`s2_age_routing` **9/9**；落主
  `c58d9bb8` → auto-ui master `07821d49`，合后主检出同组复跑全绿。
- **未闭**：R64-10 的 AGE 异步/资源 owner 语义、R59-5、979 多出口重核、正式 SDK pin；V 批仍等 980。
2026-11-16 stage: work（V 批 harness：不依赖 980 的 V-L 判据先冻结）| PLAN-981 | outcome: partial | commit `c76ed5d0b`（master `f91b50fc9`）

- 新增 `plan981_v_harness.rs`（6 用例，注册进 conformance）：把 §7 中**不依赖 980 输入**的 V-L 判据
  冻结为可执行检查——V-L-VIEW（五出口 + 选择序 + 方形/2D-3D 无跨域）、V-L-COND（if 值 bool 位置；
  `#if` 归 966，age-package 面拒）、V-L-TIME（Start=1/Tick=N/Frame=M 不随出口倍增、0 出口拒）、
  V-L-WIRE（handler/msg/design 在描述面可读）、V-L-COVER（24 行分母 + facts）、以及
  `v_l_pending_inputs_are_named`（980 依赖清单**不可静默缩水**）。
- 清单（`V_PENDING_INPUTS`，3 项）：980 正式 UI/事件源、V-B-SESSION 的 typed UI/资源接口 + 多
  Run/Stop 净归零宿主面（R64-10）、与 980 合流的正式 oracle 文件。
- **证据**：plan981 **192/192**（新增 6）；合后 master 复跑 plan981 192、A2R `p981` 4/4、AV 3/3。
- 语义：V 就绪时**替换源/期望**即可，不重写判据；本 harness 不声明 V 已跑。
2026-11-16 stage: work（tier-1 收尾对账：两仓最终回归 + 交付/阻塞分账）| PLAN-981 | outcome: partial

**auto-lang master 最终回归**（`e36b7ffcd`）：plan981 **186/186**、plan980 13/13、ace-core lib 79/1
ignored + conformance **717/717**、AV `p981_age_package` **3/3**、A2R `p981` **4/4**（真实 rustc）、grammar
**45/45**；固定组 `cargo t ace-source ace-syntax` 两组 **PASS**，但整门 `gate_passed=false`
（error="code/config/inputs changed during execution"，因其他会话在跑同一 master 树——环境归因，非本批
失败；分支尖端独立复跑证据仍有效：run-dpr92hpu / run-6333fcds）。
**auto-ui master 最终回归**（`4c629db0`）：driver lib **47/47**、`s2_age_routing` **8/8**、bridge
**S1 4 + S3 3 + S4 9**、`verify_overlay`/`prepare_sdk --verify-only` 双 PASS（681+1361）。
**已交面（都有复审/门禁/落主）**：B-SOURCE-I/V/R（979 已核）；LANGUAGE-I（review-62 pass，master
`1738c2a1c` + 后续 `e36b7ffcd`，plan981 186）；SESSION-I（review-64 pass，auto-ui `15102c48` →
`3e63fa0d` → `3604f2f2` → `4c629db0`，driver 47 + s2 8 + S1/S3/S4 16）；RUST 公共链（4/4 + 具名缺口
`D-981-MODULE-GLOBAL-A2R`）。
**阻塞（不冒充交付）**：LANGUAGE-V / B-SESSION-V / V-B-RUST / V-B-LANGUAGE 需 980 的正式 UI/事件源
（980 仍 `executing`，round 24 MIN-A-UI 待复核）；V 前置 R64-10（资源 owner/多 Run-Stop 净归零/AGE
异步）；R59-5（语法恢复位置）；R62-7（`.modifier` 语法）；`D-981-EXIT-CONTRACT`（979 按落主 SHA 重核
多出口契约）；正式 SDK pin 归 978/986/987（本地为两层消费者快照）。
**下一轮**：准备 V-B-LANGUAGE/V-B-SESSION 的命令/源/oracle 清单与 harness，使 980 就绪即可开跑。
2026-11-16 stage: work（R64-4 多 node 同名字段消歧 + 落主）| PLAN-981 | outcome: partial | commit `d12b3d420`（master `e36b7ffcd`）

- **改面**：包内状态函数改为**按节点消歧**——node 0 保持历史拼写（A2R 冻结的 `__age_state_0_count`
  不变），其余节点用 `__age_state_n<node>_<inst>_<field>`；包前门、描述读回（`model.init` 事实）与 AV
  观察共用同一 helper。
- **效果**：同一 AGE 包内两个 node 使用**同名字段**不再撞 `source.module-duplicate-declaration`；两个
  组件各自可观察（AV 用例）。
- **证据**：plan981 **186/186**（双 node 同名字段编译）、AV `p981_age_package` **3/3**（两组件分开观察）、
  ace-core **717/717**（+79）、plan980 13/13、A2R `p981` **4/4**（真实 rustc，单节点 `__age_state_0_count`
  合同未变）、grammar 45/45；落主后同组复跑全绿。
- **债务**：`D-981-SESSION-MULTI` ②（双 node 字段撞名）**关闭**；① 的剩余部分（组件级 2:1 调度）仍归
  ACE 场景扩展。
2026-11-16 stage: work（SESSION 复审小项 R64-5/6/7 + 四次落主）| PLAN-981 | outcome: partial | auto-ui `6110e0ba`（master `4c629db0`）

- **R64-5**：body 代数与调度计数**分开**——`generation()` 只记已应用调度（tick/send），`body_generation()`
  记成功 update_body/restart；driver `age_identity()` 返回 body 代数，新增 `age_dispatches()`。不再混用一个计数器。
- **R64-6**：新增 `tools/apply_overlay.py`——从给定 auto-lang 检出按 `overlay.json` 的逐文件哈希物化第二层
  （先复制到临时目录、全量校验后才替换；实测复现当前 1361 文件 overlay），`verify_overlay`/`prepare_sdk
  --verify-only` 双 PASS。
- **R64-7**：`tests/s4.rs` 的 rustfmt 红已清（用 `skip_children` 避免误格式化无关文件）。
- **落主**：`6110e0ba` → auto-ui master `4c629db0`；合后主检出 `s2_age_routing` **8/8**、driver lib
  **47/47**、bridge **S1 4 + S3 3 + S4 9**、`verify_overlay` PASS。
- **仍开**：R64-4（双 node 同名字段，ACE owner；`D-981-SESSION-MULTI` ②）；R64-10（资源 owner/多
  Run-Stop 净归零与 AGE 异步仍无宿主面，属 V 批）；LANGUAGE-V 等 980 正式 UI/事件源。
2026-11-16 stage: work（R64-3 多组件宿主面 + 三次落主）| PLAN-981 | outcome: partial | auto-ui `14059d48`（master `3604f2f2`）

- 新增宿主面：`component_count/component_names/observe_component(index)/exits_of(index)/
  request_layout_for(index, req)/tick_instance(instance,n)/send_to(instance,msg)`；越界具名拒，不静默取首组件。
  driver 侧同名薄委托（`age_component_*`/`age_observe_component`/…）。
- **2:1 事件归属的现状（如实）**：当前 AV 场景 op 是**实例级**（`ScenarioOp{instance}` 会推进所有
  声明了该 handler 的组件）——多组件逐组件调度仍需 ACE 场景扩展；已在 `D-981-SESSION-MULTI` ① 记录。
  本轮可验证的是：逐组件观察/布局可达、同一次定向事件按 handler 声明归属推进每个组件。
- **证据**：bridge **S1 4 + S3 3 + S4 9**；driver lib **47/47**、`s2_age_routing` **8/8**；落主
  `14059d48` → auto-ui master `3604f2f2`，合后主检出同组复跑全绿。
- **未闭**：R64-4（双 node 字段撞名，ACE owner）、R64-5/6/7/10；LANGUAGE-V 前置仍等 980 正式 UI/事件源。
2026-11-16 stage: work（V 前置必修 R64-1/R64-2 + 二次落主）| PLAN-981 | outcome: partial | auto-ui `92b3d803`（master `3e63fa0d`）

- **R64-1**：`schedule()` 改为"先编译候选 ops、成功才提交"——被拒消息不再毒化后续 tick/send，
  generation 只在真正应用时递增；测试断言失败调度后 `tick(1)` 仍得 1、generation 只 +1。
- **R64-2**：AGE 形状判定改为**行首 `node <Ident>`**（注释/字符串里的 `node` 不再算）；坏 AGE 源在旧
  parser 也失败时保留 `source.*` 具名诊断；注释含 `node` 的旧 `var` 源仍走旧 VM 路径（双向测试）。
- **落主**：`92b3d803` → auto-ui master `3e63fa0d`（`--no-ff`；89 项他人 docs WIP 未触碰）。
- **合后主检出复验**：`verify_overlay` PASS（681+1361）、driver lib **47/47**、`s2_age_routing` **7/7**、
  bridge **S1 4 + S3 3 + S4 8**。
- **仍为 V 前置**：R64-3（宿主面单组件，双组件 2:1 不可表达——下一轮加组件选择器）、R64-4（双 node
  同名 model 字段撞 `__age_state_0_<field>`，ACE owner；改名会动 A2R 冻结的 `__age_state_0_count`
  合同，需 ACE 侧决定）、R64-5/6/7/10。
2026-11-16 stage: fold（B-SESSION-I → auto-ui master）| PLAN-981 | merge commit `15102c48`

- `plan-981-dev`（tip `de7238a4`）已 `--no-ff` 合入 auto-ui master（合入时 `dfc73cf1`；工作树 89 项
  他人 docs WIP 未触碰）；落：bridge crate、driver AGE 路由/12 个 `age_*`、s2 6 例、
  `sdk/overlay/overlay.json`(1361)、`verify_overlay.py`、`prepare_sdk.py` 两层校验、`prepare_sdk`/`tools`。
- **合后主检出复验**：把 two-layer `.sdk-source`（681+1361）装到 `D:/autostack/auto-ui` 主检出（旧 681
  快照备份到 `plan-artifacts/plan-981/sdk-main-checkout-681/`），`verify_overlay` **PASS**、
  `cargo metadata --locked --offline` **exit 0**、driver `--lib` **47/47**、`s2_age_routing` **6/6**。
- **fold 窗口不含**：`.sdk-source` 入库（gitignore）、从 master 重生成 overlay、Plan 改动/勾选、他人 WIP。
- **不勾** `B-SESSION-I`（V 义务未闭）；**V 硬前置**：R64-1/R64-2/R64-3/R64-4；其余 R64-5/6/7/10 随 V。
2026-11-16 stage: work（SESSION 复审小项 R63-7/8/9/11）| PLAN-981 | outcome: partial | auto-ui `de7238a4`

- **R63-7**：AGE 探测失败时**不再替换**旧路径诊断（此前注释里含 `node` 的旧源会改变错误消息）；旧
  parser/VM 的诊断是非包源面的合同。
- **R63-8**：`lifecycle()` 第三参改为**已声明出口数**（不是组件数），"0 出口"拒绝在宿主 API 可达；
  双出口用例确认 Tick/Frame 仍不乘出口数。
- **R63-9**：`open(_, 0)` 文档改为"编译并按默认单实例观察"（与 `observe()` 实际行为一致）。
- **R63-11**：新增 `exits()` 暴露全部已声明出口；`exit()` 保留首出口兼容含义。
- R63-12/13 仅措辞/证据级别：已在 §9 更正"零漂移"外推并说明 PATH 证据是路径级（非绝对路径隔离）。
**证据**：bridge **S1 4 + S3 3 + S4 7**；driver lib **47/47**、`s2_age_routing` **6/6**；树 clean。
**next**：SESSION-I 再次独立复审 → auto-ui fold。
2026-11-16 stage: work（R63-1 SESSION-I 六项义务落地）| PLAN-981 | outcome: partial | auto-ui `31247ef0`

**改面**（都落在单一实现 `age_session_core.rs`，driver 只加薄委托）：
① **Tick**：`tick(n)` 追加调度并进程内重跑受检场景（0→5→6 增量，不换身份）；② **回调**：
`send("<Msg>")` 触发已声明的 `on <Msg>`（同一 AV 链）；③ **资源**：`resources()` 暴露 (name, port,
module) 事实；④ **同步/异步**：Tick 与消息投递两条路径都由场景驱动、无手写玩法；⑤ **停止恢复**：
`stop()` 冻结最后观察值并拒绝调度/编辑，`restart()` 同 PID、Start 重新为 1、generation+1；⑥ **不兼容
布局**：`request_layout(AgeViewRequest)` 走 `select_exit` 的已声明出口选择，UI 请求遇 2D-only 出口
`source.age.exit-unavailable` 具名拒。
**证据**：bridge **S1 4 + S3 3 + S4 6**（增量 tick、消息回调、资源事实、布局拒、停止冻结/恢复、
open 时 ops 叠加后继续 tick）；driver lib **47/47**、`s2_age_routing` **6/6**（driver 层同一组义务可达）；
树 clean（`rustfmt` 的 `skip_children` 副作用已回滚，未污染无关文件）。
**未闭**：R63-7/8/9/11/12/13 小项；再次独立复审后 auto-ui fold。
2026-11-16 stage: work（SESSION 复审修复 R63-2/3/4/5/6/10）| PLAN-981 | outcome: partial | commit `d763dd57`

**已修**：R63-2 S2 测试改读**真实 AV 值**（v1→v2、坏编辑仍 v2；update 若是空操作必红）；R63-3
`AgeSession` 保留打开时 ops，更新复用（5 Tick 不再变 0，bridge/driver 同修）；R63-4 AGE 会话调
`start()` 具名拒绝且**不关闭**；R63-5 快照改为**两层声明**——681 文件 base + `sdk/overlay/overlay.json`
（1361 文件：ACE crates/auto-tool-execution/stdlib/ace，来源 auto-lang `a27bed79c`），
`prepare_sdk.py --verify-only` 与新 `tools/verify_overlay.py` 都**双 PASS**；R63-6 适配实现收敛为**单一
文件** `age-ace-session/src/age_session_core.rs`（bridge 与 driver 用 `include!` 共同编译）；R63-10
rustfmt（只格式化 touched 文件，修正了 cargo fmt 误改的 44 个无关文件）。
**证据**：driver lib **47/47**、`s2_age_routing` **5/5**、bridge **4+3**、`prepare_sdk --verify-only`
PASS（681+1361）、`verify_overlay` PASS。
**收据更正**：删除"零漂移"外推（图是新 ACE 叠旧 pin，锁每包一份成立）；备份陈述改为"681 pin 可由
`sdk/frozen` 逐字节重建，未保留独立备份"；**R63-1 六项（tick/回调/资源/同步异步/停止恢复/不兼容布局）
属 I 未实现，不再写成"V 批"**——已在下一轮作为 SESSION-I 的具名缺口登记。
**未闭**：R63-1/R63-7/R63-8/R63-9/R63-11/R63-12/R63-13；需再次独立复审。
2026-11-16 stage: review（B-SESSION-I phase 复审）| PLAN-981 | review_scope: phase | outcome: **needs_fix** | commit `9da96876`

**复审（独立 subagent，auto-ui 树）**：正面结论已复现——AGE 源真实进 typed 适配并取出 AV 真实值
（`[["v1"]]`/`[[0]]`）、模块级 `var` 旧源仍走旧 VM、坏 AGE 源具名拒且不回退、同 PID 更新值真变化/坏编辑
保旧、`PATH=""` 下 open/update/observe 成功、681 pin 与 `sdk/frozen` 逐字节全等、vendored ACE 与
auto-lang `a27bed79c` 全等、每包锁仅 1 份、`wt-guard` clean、未削弱任何既有断言。
**findings**：R63-1（§5 SESSION I 六项 tick/回调/资源/同步异步/停止恢复/布局未实现，V 只跑不实现）；
R63-2（S2 测试测不出更新是否生效）；R63-3（`update_body` 丢场景：5 Tick→0）；R63-4（AGE 会话调
`start()` 会静默关闭会话）；R63-5（overlay 不可复现/不可校验、`prepare_sdk --verify-only` 红、备份目录
空）；R63-6（适配层两份拷贝）；R63-7..13（路由子串判定、lifecycle 参数、instances=0 文档、
rustfmt、age_exit 未接 select_exit、"零漂移"措辞、PATH 级证据）。
2026-11-16 stage: work（SESSION S2 完成：driver AGE 路由 + 同 PID 更新）| PLAN-981 | outcome: partial | auto-ui `9da96876`

**改面**：driver `Session` 现在把 AGE 源路由到 typed 宿主（driver 内嵌 `src/age_session.rs`，从计划桥接
crate 迁入，避免第二个 crate 造成 auto-atom 双源锁冲突）；提供 `is_age/age_exit/age_lifecycle/
age_observe/age_identity/update_age_body`。非 AGE 源保持旧 parser/VM 路径；AGE 形状源若旧 parser 也解析
不了，则报共享 verify 的具名诊断（否则回退旧路径——宿主脚本的模块级 `var` 不受影响）。
**SDK 策略修正（重要）**：先用"当前源码全量快照"替换驱动 `.sdk-source`，`cargo check` 通过但 driver
lib 基线被打红（`var` 不再被接受等 3+ 例）——判定为快照漂移、**放弃该路线**；改为**保留 681 文件旧 pin**
并只把 ACE crates、`auto-tool-execution`、`stdlib/ace` 白名单式 vendored 进同一快照（单一 auto-atom 源，
零漂移）。旧 pin 备份仍在 `plan-artifacts/plan-981/sdk-backup-681/`，全量快照已删除。
**证据**：driver lib **47/47**（恢复基线）、新增 `tests/s2_age_routing.rs` **3/3**（AGE 路由；同 PID 更新
pid 不变且 generation+1；坏编辑拒绝且旧代保留、旧值仍可观察；scalar Session 上 AGE API 具名拒绝）；
`cargo check --lib` 绿。
**未闭**：S2 只覆盖 typed 宿主适配层；V-B-SESSION 完整运行（多 Run/Stop 资源净归零、坏更新恢复、真实
资源）仍待 V 批；B-SESSION-V 需要 LANGUAGE/980 的正式 UI 就绪。
2026-11-16 stage: work（SESSION S2 解阻：SDK 快照刷新，用户授权）| PLAN-981 | outcome: partial | auto-ui worktree

**授权**：用户选择"授权我在 auto-ui 侧刷新 SDK 快照"并指定下一优先=准备 LANGUAGE-V 前置。
**做了什么**：① 用当前 auto-lang worktree 源码（排除 target/node_modules/.git）物化 auto-ui 根
`.sdk-source`（17,634 文件 / 492.1 MB）；② 驱动原本依赖**驱动本地** `.sdk-source`（681 文件 / 13 MB 的
旧 pin）——把旧 pin 备份到 `plan-artifacts/plan-981/sdk-backup-681/.sdk-source`，再用新鲜快照替换；
③ 驱动 `Cargo.toml` 增加 `ace-av`/`ace-core`（路径指向**同一棵** `.sdk-source` 内的 ACE），彻底消除
"auto-atom 双源锁冲突"。
**证据**：`cargo metadata --manifest-path experimental/autographics-auto-driver/Cargo.toml` **exit 0**
（lock 新增 ace-* 38 包，不再报 collision）；`cargo check --lib` **exit 0**（1m20s，5 条既有警告）。
**状态**：S2 的**依赖阻塞已解除**，driver 可在同一图内引用 ACE AV/core；会话路由（Session→AgeSession）
本身仍待实现与验证。auto-ui 侧改动未提交（Cargo.toml/lock + 未跟踪 `.sdk-source` 被 gitignore）。
**风险/边界**：SDK 由"旧 pin"变为"当前源码快照"，驱动对 auto-lang-core 的 API 漂移风险由 `cargo check`
挡住一部分；正式 SDK pin/发布仍归 978/986/987，本刷新是**本计划消费者侧**的临时解阻，不冒充正式 SDK。
2026-11-16 stage: work（R59-5 追加定位；无代码变更）| PLAN-981 | outcome: pending | tree = `9e67676f6`

**R59-5 @0 的两个来源已缩定**：① 语法引擎在坏槽位处**没有触发规则级恢复**——`age-node-c-item` 加
`recover: true` 后重跑仍 `@0..4`，实验已回滚；② 诊断实际来自 `cst/project.rs` 的**首错映射**（error 叶在
声明起点 → `expected fn, found node`）。`frontdoor.rs` 里"领域 hint 取代首错"的分支只对
`AgeH0|AgeComponent` 生效；把 `AgePackage` 并入后仍无 hint（该失败形态下 `tree.diags` 为空；hint 只在
整次解析失败时并入）。
**后续修法（二选一，属后续批次）**：(a) 语法引擎侧让 `age-node-member` 在坏成员处产出可恢复 error 叶；
(b) 投影侧让 AGE 面的首错映射用"成员头 token"作定位。两法都要永久负例（坏 design/bind/routes/expose/
timer 各一）断言 span 落在槽位内而非 0。
**本轮代码零变更**：实验（recover、hint 分支）全部回滚，树与 `9e67676f6` 一致，plan981 185/185。
2026-11-16 stage: work（命名 view 结构：有界描述 + 树内表达式验证）| PLAN-981 | outcome: partial | commit `a27bed79c`（master `9e67676f6`）

**R61-4/R62-5（V 前置推进）**：命名 view 此前只留文本叶、结构体整个丢弃——于是 `view <name>` 内的
`class: nosuch_ident` 不验证，消费方也不知道它有没有结构。现在：① 投影保留每个命名 view 的结构树；
② 前门把树内所有属性/label/内联 lambda 表达式送进唯一 verify（未声明名在值 span 具名拒）；③ 描述面新增
`AgeNamedViewFact { name, texts, items }`（按声明序的有界事实）。**完整结构形状仍归 V 批次**（描述面不
冒充完整树）。
**测试**：plan981 **185/185**（命名 view facts：texts=1/items=3；坏属性 `source.undefined` 在值 span）；
plan980 13/13；ace-core **716/716**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**落主**：`a27bed79c` → `9e67676f6`（干净分离 worktree 合并；master 已含 980 T-01 簿记，无冲突）。
**边界（保持）**：`.modifier` watch 语法、R59-5 语法恢复位置、命名 view 完整树形状（V）。
2026-11-16 stage: work（R61-4 view 属性表达式进 verify）| PLAN-981 | outcome: partial | commit `e8a4febab`（master `b917f0e5c`）

**R61-4（note→已修，V 前置）**：view 元素的属性值、元素 label、内联 lambda 赋值 RHS 此前只投影不求值，
`class: nosuch_ident` 静默通过。现在遍历投影树，把每个属性/label/lambda 表达式都送进同一
`hidden_value_fn`：未声明名字在**值自身 span** 具名拒；字面量属性照常通过。测试
（`p981_view_attribute_expressions_are_verified`）。
**落主**：`e8a4febab` 经干净分离 worktree 合并为 `b917f0e5c`（master 已含 976 归档提交
`5cedb4715`；无冲突，主检出只刷新 `experimental/ace`）。
**测试**：plan981 **184/184**；plan980 13/13；ace-core **715/715**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**边界（保持）**：R62-7 `.modifier` 语法不可解析；R59-5 语法恢复位置；命名 view 结构树未投影
（其文本叶已验）。
2026-11-16 stage: work（R60-8 逐叶出口归属）| PLAN-981 | outcome: partial | commit `a6576d580`（master `21a9c977b`）

**R60-8（note→已修）**：多出口并存时 `texts` 是扁平表，消费方无法判断某个叶属于哪个出口。现在每个文本叶
带出口归属：`ComponentView.text_exits`（与 `texts` 同序，含普通 `view` 本身）→ 描述面
`AgePackageComponent.text_exits: Vec<Option<AgeViewExit>>`；命名 view 的叶仍用 role 归属（其出口项为空）。
测试：`view` + `view-portrait` 两个叶分别标 `View` / `ViewPortrait`。
**落主**：`a6576d580` 经干净分离 worktree 合并为 `21a9c977b`（fast-forward，主检出仅刷新
`experimental/ace`，他人 docs WIP 未触碰）。
**测试**：plan981 **183/183**；plan980 13/13；ace-core **714/714**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**边界（保持）**：R62-7 `.modifier` 形态语法不可解析；R59-5 语法恢复位置；命名 view 结构树投影。
2026-11-16 stage: work（R62-2 watch 源 span 保真 + R62-7 边界 + RUST 缺口登记）| PLAN-981 | outcome: partial | commit `3f833e0d6`（分支；落主待窗口）

**R62-2（note→已修）**：watch 源拒绝此前用整个成员 span；成员行新增 `source_spans`（与 `sources` 同序、
指向源 token），前门按源码 token 报错，测试断言 diag 起点=源名字节。修饰形 `.a.b` 现在按完整拼写投影
（`a.b`），只对头字段做 model 校验。
**R62-7（边界，如实记录）**：`.modifier` watch 形态**语法本身不可解析**（三种写法均 `source.syntax@0`
`expected fn, found node`）；投影层已能承载，但让语法接受它属独立语法任务，不在本批冒充。
**B-RUST-I 缺口登记**：模块级 `const`/`var` 的 A2R 目标合法化登记为具名缺口
`D-981-MODULE-GLOBAL-A2R`（`docs/plans/KNOWN-DEBT-AND-RISKS.md`；根因=HIR 不承载 global 的
init/mutability，A2R 消费 HIR；关闭条件=HIR 携带事实或 A2R 改消费含 globals 表的 MIR + 真实 rustc 腿）。
计划 §8 原文允许"或按具名缺口登记"，AC 未改。
**测试**：plan981 **182/182**（源 token span 断言）；plan980 13/13；ace-core **713/713**（+79）；AV 3/3；
A2R 4/4；grammar 45/45。
**落主状态**：`3f833e0d6` 已在 `plan-981-dev`；master 当时因其他会话暂存 976 归档重命名无法直接 merge，
后经**干净分离 worktree**完成合并（merge commit `eb950355c`，master 快进，主检出仅刷新 `experimental/ace`
路径，未触碰他人 WIP）。
2026-11-16 stage: fold（B-LANGUAGE-I → master）| PLAN-981 | merge commit `1738c2a1c`

- 分支 `plan-981-dev` 尖端 `6276e83f8`（四次 phase 复审 **pass**，review-62）**--no-ff** 合入 master
  （合入时 master `107b23d0f`，14+ 提交领先；merge-tree 无冲突；仅 `experimental/ace/` 17 文件）。
- 合后 master 复跑：plan981 **181/181**、plan980 13/13、ace-core lib 79/1 ignored + conformance
  **712/712**、AV 3/3、A2R 4/4、grammar 45/45（均 PASS）。
- 固定组：分支尖端 `6276e83f8` 由复审者亲跑 **PASS**（run-dpr92hpu，ace-source 791 / ace-syntax 155）；
  落主后在本机复跑因**其他会话并发改动工作树**被判 `gate_passed=false`（error="code/config/inputs
  changed during execution"）——环境归因，非本批失败；分支尖端证据有效。
- 工作树仅有他人 docs WIP（64 项未跟踪/修改，未触碰）；worktree 保留给后续批次（LANGUAGE-V /
  SESSION / RUST 未闭）。
- **具名待办（不随本批闭）**：① 979 需按**落主 SHA**复核"一个 node 恰一个 view 出口、混合出口必拒"
  的契约已被 r13 AGE-L03 撤销（现在不同键并存合法、同键重复仍拒）——owner 979，关闭条件=按落主
  SHA 重跑其 V-11/detail 行并改判；② R59-5 语法恢复位置（owner 981/LANGUAGE + syntax-grammar，已记
  复现与失败尝试）；③ R62-3 watch 源收紧到 model 字段需 980 确认；④ R62-4 980 面只投影不检查源；
  ⑤ auto-ui 桥接件（`bdba32cb`/`dc9251c0`）留其自身 fold 窗口。
2026-11-16 stage: review（B-LANGUAGE-I 四次 phase 复审 / fold 门）| PLAN-981 | review_scope: phase | outcome: **pass** | commit `6276e83f8`

**复审（独立 subagent，同树）**：R61-1 / R61-2 / R61-3 / R61-5 / R61-6 全部 **closed**（复审者自测：
单行多源 watch `sources=["a","b"]`、`.a, .nosuch` 两形态均 `source.age.member-field`、`if a`/`if "x"` →
`source.type-mismatch expected bool`、冻结 id 与 §6.7 一致、grammar **CR=0**、`-w` diff 81/34）。
**无 blocker**；10 条 note（R62-1 §9 措辞、R62-2 源拒绝 span 是整个成员、R62-3 watch 源收紧到 model
字段需 980 确认、R62-4 980 面只投影不检查源、R62-5 陈旧文档、R62-6 缺尾换行、R62-7 `.modifier` 死形态、
R62-8 watch fact 不保护多源、R62-9 重复条件诊断、R62-10 命名 view-only `exits=[]` 与 `exit=View`）。
**fold 门**：merge-tree **无冲突**；fold 模拟（master 的 3 行 grammar + TextRecord 冻结 `fb3d3883…`）
plan981 **181/181**、age 冻结 id 不需重绑。
**实跑**：plan981 181/181、plan980 13/13、ace-core lib 79/1 ignored + conformance 712/712、AV 3/3、
A2R 4/4、grammar 45/45；固定组复审者亲跑 **PASS**（run-dpr92hpu，ace-source **791** 执行 / 0 bad，
排除 1 具名 native，ace-syntax 155，gate_passed=true）。
**2026-11-16：B-LANGUAGE-I 落主**（见 §10 fold 记录）。**保持 pending**：LANGUAGE-V（980 正式源 +
SESSION/RUST）、R59-5、命名 view/属性验证、**979 需按落主 SHA 复核"混合出口必拒"契约已撤销**、
auto-ui 桥接件（S1/S3）留其自身 fold 窗口。
2026-11-16 stage: work（R61-1/R61-2/R61-3 修复 + R61-5/R61-6 前置清理）| PLAN-981 | outcome: partial | commit `6276e83f8`

**R61-1**：`watch` 体用 `list-nl` 导致"单行体 + 多源头"退化成 `@0`；改为语句 `many`（语句以 `.` 开头，
pratt 在下一个 `.` 自然停），单行/多行/多语句都过。
**R61-2**：只有第一个 watch 源进描述面与检查；成员行新增 `sources: Vec<String>`（按声明序），描述面
同步，前门对**每个**源做 model 字段存在性检查——`.a, .nosuch` 现在具名拒。
**R61-3**：`design` 的 `if` 条件从未要求 bool；用 `!cond` 把它放进布尔位置（同一 `hidden_value_fn`），
`if count` / `if "x"` 都具名拒。
**R61-5/R61-6**：§6.7/§9 grammar 身份重绑 `c152355d…`/`0e799dc1…`（改为"以冻结测试为准"），L04 改为
"选择已由 `select_exit` 落地、`#if` 归 966"；R59-9 收据数字改实测 69/28；grammar 恢复 LF（CR=0，
内容 diff 81/34）。
**测试**：plan981 **181/181**（单行多源 watch + 全源检查、非 bool if 条件）；plan980 13/13；ace-core
**712/712**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**fixed group**：`cargo t ace-source ace-syntax` **PASS**（run-c0ulvwzu，`6276e83f8`，clean tree）。**next**：四次复审。
2026-11-16 stage: review（B-LANGUAGE-I 三次 phase 复审）| PLAN-981 | review_scope: phase | outcome: **needs_fix** | commit `805ba942a`

**closed**：R60-1（`1 + 2` 投影 Binary、尾部名字在两处被 `source.undefined` 拒）、R60-2（命名 view 文本叶
role 归属 + 未声明名 `source.module-symbol@38`）、R59-2/R59-1（复测通过）。
**新 findings**：**R61-1**（needs_fix）多源 watch 的**单行体**仍 `@0`；**R61-2**（needs_fix）多源 watch 只
投影/检查第一个源（`.a, .nosuch` 被接受）；**R61-3**（needs_fix）design `if` 条件不要求 bool（`if count`
被接受）；**R61-4 note**（V 前置）命名 view 结构树与 view 元素/属性表达式未 verify；**R61-5 note**（fold
前置）§6.7/§9 grammar id 仍记旧值且 L04 措辞与 `AGE_CONSTRUCTS_OUT_OF_SCOPE` 矛盾；**R61-6 note**
grammar 又被写成 CRLF（R59-9 回退）；R61-7/8/9 陈旧文档与重复条件诊断。
**实跑**：plan981 179/179、plan980 13/13、ace-core 710/710（+79）、AV 3/3、A2R 4/4、grammar 45/45；
固定组复审者亲跑 PASS（run-6333fcds，ace-source 789 / ace-syntax 155）。
**next**：修 R61-1/2/3 + 前置 R61-5/6，再固定组与四次复审。
2026-11-16 stage: work（R60-3/R60-4 修复 + 陈旧文档刷新）| PLAN-981 r13 | outcome: partial | commit `805ba942a`

**R60-3**：`watch` 源要求 `.` 与前一 token 同行，导致多行 `watch {\n .count -> …` 与多源声明退化成
`source.syntax@0`；改为普通换行规则（两个面共享该规则，故 grammar id 双面更新），新增多行/多源声明
用例。
**R60-4**：`if` 值的 design 条目 `value=None`、条件/分支从不 verify——投影现携带 `cond/then/else`
（else 是 `age-if-else` 子规则而非字段），描述面同步暴露，前门把三段都送进同一 `hidden_value_fn`；
三段任一处未声明名字都在其 span 具名拒。
**顺带**：刷新陈旧"单出口"文档（`AgeViewExit`/project 注释/用例头）；L04 范围外注释更新为"`#if`/CT 归
966；选择已由 `select_exit` 落地"。
**测试**：plan981 **179/179**；plan980 13/13；ace-core **710/710**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
固定组与三次复审在下一轮。
2026-11-16 stage: work（R60-1/R60-2 修复）| PLAN-981 r13 | outcome: partial | commit `f5ec37ab9`

**R60-1（blocker）**：成员 `value` / 语句 `rhs` 被截断成第一个原子——根因是 `pratt` 字段先
`unwrap_child` 再投影；改为整树投影。负例钉住：`computed { d => 1 + 2 }` 投影必须是 `Binary`；
`computed { d => count + nosuch_ident }` 与 `.count = count + nosuch_ident` 两处右链都被
`source.undefined` 在名字 span 拒绝。
**R60-2（blocker）**：`view <name> { … }` 的体此前整个丢弃——现按 `view` 同一机制收集文本叶
（role `named-view:<name>`）并入受检文本函数；描述面按叶自身 role 归属（主 view 仍为 `view`，历史
合同不变）。负例：`view main { text nosuch_fn() {} }` → `source.module-symbol` @38。
**测试**：plan981 **177/177**；plan980 13/13；ace-core **708/708**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**未闭**：R60-3 watch 多行/多源不可解析；R60-4 design `if` 值未检查；R60-5..10 备注（赋值类型兼容、
design 值形状边界、陈旧"单出口"文档、多出口内容归属、select_exit 无消费方、L04 仍在范围外列表）。
**next**：R60-3/R60-4 → 同组门禁 → 三次复审。
2026-11-16 stage: review（B-LANGUAGE-I 二次 phase 复审）| PLAN-981 r13 | review_scope: phase | outcome: **needs_fix** | commit `8a3fbfd7a`

**复审（独立 subagent，同树）**：R59-1 / R59-2 / R59-4（单行单源）/ R59-7 / R59-9 **closed**；R59-5
pending 被接受（owner + 复现 + 三次失败尝试 + 回滚已验证：grammar 与 `8a3fbfd7a` 逐字节一致）；
R59-3 **未闭**。
**新 blocker**：**R60-1** 成员 `value` 与 lifecycle/watch `rhs` 被截断成**第一个原子**——`computed { d =>
count + nosuch_ident }`、`.count = count + nosuch_ident` 被接受，`1 + 2` 只投影 `IntLit(1)`（
`collect_member_rows`/`member_stmts` 在 `pratt` 字段上先 `unwrap_child` 再投影，左操作数吞掉右链）⇒
§9 的"成员值表达式进唯一 verify"是 overclaim。**R60-2** `view <name> { … }` 体在两个面都被丢且从不
verify：`view main { text nosuch_fn() {} }` 被接受（R52-1 原始例子仍然活着）。**R60-3**（needs_fix）
watch 多行/多源不可解析（`dot(same-line)` + 死掉的 `many{age-watch-source-tail}`），不得并入 R59-5。
**R60-4**（needs_fix）design 的 `if` 值仍 `value=None`，条件/分支未检查。备注：R60-5..10（赋值类型兼容、
design 值形状边界、陈旧"单出口"文档、多出口内容归属、select_exit 无消费方、计划 L04 仍在范围外列表、
R59-9 收据数字 72/31 实测 69/28）。
**实跑**：plan981 **174/174**、plan980 13/13、ace-core **705/705**（+79）、AV 3/3、A2R 4/4、grammar
45/45；固定组 `cargo t ace-source ace-syntax` 复审者亲跑两次均 PASS（run-wji2w__l / run-nfsidm49，
ace-source 784 / ace-syntax 155）。
**next**（下一轮）：修 R60-1/R60-2（带断言级负例：成员值必须是完整 Binary；`view <name>` 体内未声明
名字必须拒；多行 watch 必须可解析）→ R60-3/R60-4 → 复跑同组门禁 → 三次复审。
2026-11-16 stage: work（R59-5 现状：语法恢复位置，未闭但有具名证据）| PLAN-981 r13 | outcome: pending |

**复现**（10 类槽位坏形态均 `source.syntax@0` `expected `fn`, found `node``）：`design{theme}`、
`bind{"t" ->}`、`routes{"/a" ->}`、`expose{空}`、`timer{Tick(every_ms:)}` 等——错误位置退到源 0。
**已试并证伪的修法（均无效果，已全部回滚，grammar 与 `8a3fbfd7a` 逐字节一致）**：
① `age-node-c-item` 的 `lbrace(cut: true)`；② `age-node-c-item`/`age-node-c-body`/`item`/`design-member`
加 `recover: true`；③ `item` 规则整体 recover。语法运行时 `RecoverySpec.sync = ["semi","rbrace"]`
（grammar 头），成员失败会同步到 `}`，与 node 的结束 `}` 冲突，单点属性改不动顶层 items 的失败回退。
**结论/归属**：这是**语法恢复设计**问题（需要 sync 到换行/成员边界或 items 级恢复策略），owner =
981/LANGUAGE + syntax-grammar；不是成员投影缺陷。已登记为 R59-5 pending（有复现与失败尝试证据），
不冒充已修。
**状态**：tree 在 `8a3fbfd7a`（R59-3 尾项）；plan981 **174/174**、grammar 45/45；固定组 `cargo t ace-source ace-syntax` **PASS**（run-3wysc6k6）。
2026-11-16 stage: work（R59-3 尾项：lifecycle/watch 语句体进 verify）| PLAN-981 r13 | outcome: partial | commit `8a3fbfd7a`

**改面**：成员行新增 `stmts: Vec<(field, Expr)>`（lifecycle/watch 体内 `.field = <rhs>`，按声明序）；
Age 面把每个 RHS 送进唯一 verify（与 design/成员值同一 `hidden_value_fn` 机制），并拒绝给**非 model
字段**赋值（`source.age.member-field`，span 指向 RHS）。复审探针 `.count = nosuch_ident` 与
`.nosuch = 1` 现在都具名拒。
**测试**：plan981 **174/174**（语句 RHS 未声明名字 @名字、非 model 字段具名拒）；plan980 13/13；
ace-core **705/705**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**未闭**：R59-5 十类槽位坏形态仍退化 `@0`（语法恢复质量）——下一轮处理；之后固定组 + 二次复审 → fold。
2026-11-16 stage: work（R59-2 修复：多出口 + 选择语义）| PLAN-981 r13 | outcome: partial | commit `61155ce37`

**改面**：按 AGE-L03"重复直属选择键拒绝，多同域内部区域合法"把投影从"恰一个出口"改为**多出口**：
`ComponentExit{kind,span}` 收集全部直属出口（同键重复仍 `source.age.view-exit`），描述面新增
`exits: Vec<AgeViewExit>`（`exit` 保留首出口作主事实）；`select_exit(declared: &[AgeViewExit], request)`
按计划顺序实现 widget（仅 widget 宿主）→ portrait（仅 H>W）→ 默认 view、方形走默认、2D/3D 无跨域
fallback、无候选/零尺寸具名拒。
**契约演进（须 979 复核）**：SOURCE 时代"M1：一个 node 恰一个 view 出口"的**混合出口拒绝**被本批按 r13
AGE-L03 放开（`view`+`view-portrait` 等合法；`view-2d`+`view-3d` 也可并存，各按请求领域服务）。同键重复
仍拒。979 若在其 V-11/detail 行依赖"混合出口必拒"，需按落主 SHA 重核该行。
**测试**：plan981 **172/172**（多出口记录+首出口主事实、同键重复拒、选择序/方形/widget-only/场景混合/
无 fallback/零尺寸）；plan980 13/13；ace-core **703/703**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
**未闭**：R59-3 lifecycle/watch **语句体**仍未进 verify；R59-5 十类槽位坏形态仍退化 `@0`；之后重跑固定组
→ 二次复审。
2026-11-16 stage: work（B-LANGUAGE-I 复审修复：R59-1/3/4/7/9）| PLAN-981 r13 | outcome: partial | commits `0e35d24e7`、`b01af94c9`

**已修**：R59-1 `expose { a, b }` 只留第一个名字（`collect_expose_names` early-return）→ 收集全部 name
字段；R59-3（部分）`design`/成员**值表达式**进唯一 verify（`hidden_value_fn`：`fn …() int { let __v =
(<expr>); return 0 }`，诊断落回作者 span，`design{theme: nosuch_ident}` 现在 `source.undefined@名字`）；
R59-4 `watch` 行名字为空（源在嵌套 `age-watch-source.source`）→ 名字/声明文本都保留；R59-7 §6.7
重绑（age-package `c152355d…`、age-component `e81bc1ac…`）并新增 `p981_age_package_grammar_id_is_frozen`；
R59-9 grammar 恢复 LF（内容 diff 实测 69/28，行尾噪声消失）。
**未闭（blocker，下一轮）**：R59-2 `select_exit` 与"单出口"规则冲突且无消费方——需按 §5 决断（多出口
声明/选择语义）或把选择移入 host 并重写契约；R59-3 的 lifecycle/watch **语句体**仍未进 verify（值表达式
已进）；R59-5 十类槽位坏形态仍退化到 `@0`（语法恢复质量）。
**测试**：plan981 **172/172**（新增 expose 全名、watch 源、值表达式具名拒、age-package id 冻结）；
plan980 13/13；ace-core **702/702**（+79）；AV 3/3；A2R 4/4；grammar 45/45。
2026-11-16 stage: work（B-SESSION-I S3 + B-RUST-I 公共链）| PLAN-981 r13 | outcome: partial | commits auto-ui `dc9251c0`、auto-lang `18f4a8cb9`

**SESSION S3（auto-ui `dc9251c0`）**：`AgeSession::{update_body, pid, generation}`——同 PID body 更新：成功才
替换受检代并递增 generation，失败**保留旧代**（观测值不变），修复后恢复；**剥掉 PATH 后更新仍成功**，
实测证明更新路径不调用 cargo/rustc（ACE 编译/检查是进程内的）。测试 S1 4/4 + S3 3/3。
**RUST-I 公共链（auto-lang `18f4a8cb9`）**：新增 `plan981_rust_public`（2/2，真实 rustc）——同源 AGE 包经
`SemanticView → Rust fileset → rustc → 运行`，输出对**独立算术 oracle**（5 Tick ⇒ 5）；断言生成 Rust 不含
VM/解释器路径标记；V-R-FAIL：模块级 `const` 被 A2R 目标合法化**具名拒绝**（`a2x.*`）且**不写产物**，同一
入口换合法源后恢复并产出真实二进制（模块级 const/var 的 A2R 支持仍是 RUST 批具名缺口，不冒充）。
**门禁**：`p981_rust_public` 2/2、`p981_age_package` 2/2、`p979_age_a2r` 3/3（均真实 rustc）；
`plan981` 168/168；auto-ui 桥接 crate 7/7。
**next**：SESSION S2 仍阻塞于 driver 的冻结 SDK 与 live ACE 同名包冲突（978 已归档，消费侧 SDK 刷新
owner 待定）；RUST-I 继续补模块级 const/var 的目标合法化或按具名缺口登记；随后 LANGUAGE/SESSION
批级复审与 fold。
2026-11-16 stage: work（B-SESSION-I S1：AGE typed-host 桥接层）| PLAN-981 r13 | outcome: partial | commit auto-ui `bdba32cb`（auto-lang 侧本切片无改动）

**改面**：auto-ui 新增独立 crate `experimental/age-ace-session`（独立 workspace；组内相对 path 依赖
`../../../auto-lang/experimental/ace/crates/{core,av}`，不建 link）：`AgeSession::{open, open_with,
compiled, exit, lifecycle, observe}`——AGE 源一次过 ACE 共享绑定/verify，再经 `observe_age_package` 走
lower→MIR verify→ABC→AV 执行拿真实值；生命周期用 LANGUAGE 冻结的 `lifecycle_budget` oracle。
**实测（4/4）**：AV 观察 model 初值 3；**5 个 Tick op 经 AV 执行后状态 = 5**（真实 AV，不是 AutoVM）；
坏源保留 `source.*` 具名诊断；1 vs 2 出口的生命周期计数逐字相同。
**S2 的具体阻塞（实测，不是推测）**：driver 仍依赖冻结 `.sdk-source`；把 live ACE 加进同一构建图会得到
`package collision in the lockfile: auto-atom v0.1.0 (live) vs (sdk-source)`——同名同版本不同路径，Cargo
无法消解。因此 driver 接线以 **978 的 SDK 刷新**为前提；桥接层先独立成立，SESSION V 用其自身测试驱动。
**下一步**：S2（SDK 刷新后接线）→ S3 同 PID 热更新（cargo/rustc=0）→ S4 收口（AGE 路径不触 AutoVM）。
2026-11-16 stage: work（B-LANGUAGE-I 切片 6：AGE-L05 oracle 冻结）| PLAN-981 r13 | outcome: partial | commit `434a49fbe`

**改面**：新增 `age_package::{AgeLifecycleBudget, lifecycle_budget}`——把 §5 冻结的"5 Tick + 3 owner Frame、
Start=1"写成机械契约：Start 每逻辑 owner 恰一次、Tick/Frame 按调度次数、**出口数只做合法性检查**
（0 出口具名拒 `source.age.lifecycle`），不进入计数 ⇒ "Frame/Tick 不随 view 倍增"在宿主存在之前就可测。
真实宿主计数由 SESSION 接线后经 V-L-LIFE 与本 oracle 对照。
**测试**：`plan981` **168/168**（出口 1..=5 计数逐字相同、1 vs 5 相等、0 出口拒绝、零调度合法）；
`plan980` 13/13；`ace-core` **699/699**（+79）；AV 3/3；A2R 2/2；grammar 45/45。
**SESSION-I 设计草案**：`plan-artifacts/plan-981/session/session-adapter-design.md`——实测 driver 仍是
`Session{vm:AutoVM}` + 依赖 `.sdk-source`（不依赖 ACE）；方案是组内相对 path 依赖
`../auto-lang/experimental/ace/crates/av` + 窄 `age_host` 适配器，S1 适配器 / S2 AGE 源路由 /
S3 同 PID 热更新（cargo/rustc=0）/ S4 收口，不冒充非 AGE 迁移。
2026-11-16 stage: work（B-LANGUAGE-I 切片 5：其余成员槽位结构化投影）| PLAN-981 r13 | outcome: partial | commit `7de9f41d9`

**改面**：`named-view` / `bind` / `routes` / `expose` / `lifecycle` / `computed` / `watch` / `timer` 在
age-package 恢复，并以结构化行进入 `AgePackageComponent.members`（角色 + 名字 + 作者值/文本/数字 +
原 span）；构造表 **+8 行**、事实行按角色产生。**R52-1 的裁剪到此完全反转**：10 个 widget 槽位全部
"接受且描述"，不再有"接受但看不见"。
**顺带修复**：`expose { name }` 此前不产行（`collect_expose_names` 只查孩子节点，而名字是 member 的
直接 ident 子叶）；lifecycle/watch 体是 widget 语句形（`.field = expr`），测试与夹具按此写。
**测试**：`plan981` **164/164**（逐槽位投影 + span、按声明序共存、坏 route `source.syntax`）；
`plan980` 13/13；`ace-core` **695/695**（+79）；AV 3/3；A2R 2/2；grammar 45/45；固定组 `cargo t ace-source ace-syntax` **PASS**（run-jebfczjl）。
**next**：LANGUAGE-I 语义面剩余（AGE-L05 生命周期 oracle 需 SESSION 宿主；`#if` 归 966）+ 批级门禁/
独立复审/fold。
2026-11-16 stage: work（B-LANGUAGE-I 切片 4：`design` 结构化投影）| PLAN-981 r13 | outcome: partial | commit `c36b5e384`

**改面**：`design` 槽位在 age-package 恢复，并以**结构化投影**进入描述面：`AgePackageComponent.design`
（key + 作者表达式 `Expr` + 原 span，按声明序）+ 构造表 `design` 行 + 事实行。条目求值/消费归
980 面的 V-L-UI，不在投影层冒充。
**测试**：`plan981` **162/162**（design 值/span/顺序、坏条目 `source.syntax`、design+msg+model 并存；
R52-1 拒绝清单去掉 design——已交付）；`plan980` 13/13；`ace-core` **693/693**（+79）；AV 3/3；
A2R 2/2；grammar 45/45；固定组 `cargo t ace-source ace-syntax` **PASS**（run-3vd260yr）。
**next**：切片 5 = 其余成员槽位（`named-view` / `bind` / `routes` / `expose` / `lifecycle` / `computed` /
`watch` / `timer` / widget 形 `on`）的结构化投影；之后 LANGUAGE 批级门禁 + 独立复审 + fold。
2026-11-16 stage: work（B-LANGUAGE-I 切片 3：`msg` 变体完整投影）| PLAN-981 r13 | outcome: partial | commit `8500b3edf`

**改面**：`msg` 槽位在 age-package 恢复（R52-1 曾为消除"接受但不描述"而整体裁剪），这次以**完整投影**
进入描述面：`AgePackageComponent.messages`（变体名 + payload 类型拼写，按声明序）+ 构造表 `msg` 行
+ 事实行。
**语法修复（真实缺陷）**：旧的 `opt { lparen(cut) age-model-ty … }` 写法让 `msg { Set(int) }` 在**两个面
都不可解析**（整 node 退化成 `expected fn, found node` @0；980 面此前实际只接受 unit 变体）。改用独立
的 `age-msg-payload`/`age-msg-ty` 后，unit/单 payload/多 payload 都能解析，age-component 同步获得该
能力（其冻结 grammar id 已更新并注明原因）。
**测试**：`plan981` **159/159**（msg 投影、坏 payload 在原源具名拒、msg+model+on 并存；R52-1 拒绝
清单去掉 msg——已交付，不再是"接受但不描述"）；`plan980` 13/13；`ace-core` **690/690**（+79）；
AV 3/3；A2R 2/2；grammar 45/45；固定组 `cargo t ace-source ace-syntax` **PASS**（run-8_ryvdus）。
**next**：切片 4 = `design`/`on-widget`/`bind`/`routes`/`expose`/`lifecycle`/`computed`/`watch`/`timer`
的完整投影（每个**值表达式**都必须落进唯一 verify，而不是只投影形状）；随后 LANGUAGE 批级门禁 +
独立复审 + fold。
2026-11-16 stage: work（B-LANGUAGE-I 切片 2：AGE-L04 选择）| PLAN-981 r13 | outcome: partial | commit `95e2875bc`

**改面**：新增 `age_package::{AgeViewRequest, AgeSelectionError, select_exit}` 纯函数——UI 序
`view-widget`（仅 widget 宿主请求）→ `view-portrait`（仅 H>W）→ 默认 `view`，**方形走默认**；2D/3D
只接受各自场景出口、**无跨域 fallback**；无候选 `source.age.exit-unavailable`、零尺寸
`source.age.view-request`。决定门文档：`plan-artifacts/plan-981/language/L04-L05-decisions.md`。
**测试**：`plan981` **156/156**（+6 选择正反例：优先级/方形/widget-only/跨域/UI-on-scene/零尺寸）；
`plan980` 13/13；`ace-core` **687/687**（+79）；AV 3/3；A2R 2/2；grammar 本切片未改。
**注**：固定组在 B-LANGUAGE 批末复跑（本切片是 core lib 纯函数增量）。
**next**：切片 3 = AGE-L05 生命周期 oracle（`Start=1 / Tick=5 / Frame=3`、不随 view 倍增、Tick 边界
切换）——AV 侧当前**没有** Frame/Start 宿主实现（实测 `ace-av` 无该调度面），故 LANGUAGE 只冻结
oracle 与源形状，实际调度接线在 SESSION-I 落地后由 V-L-LIFE 复核；随后切片 4 = 980 widget 槽位
完整投影 + 唯一 verify 接回 age-package。
2026-11-16 stage: work（B-LANGUAGE-I 切片 1：五 view 面）| PLAN-981 r13 | outcome: partial | commit `470eea453`

**改面**：`age-package` 新增两个直属呈现出口 `view-portrait` / `view-widget`（grammar 成员 alt + 富 view
成员；出口事实按**后缀字节**落定——两条规则同形，按命中规则名会把 `view-widget` 记成 portrait）；
`AgeViewExit` 增 `ViewPortrait`/`ViewWidget` 与事实标签；重复出口/未知后缀诊断改述为五出口；构造表
**+2 行**（AGE-L03；denominator 增长，SOURCE 12 行不缩不减）。
**测试**：`plan981` **150/150**（新增 3 例：portrait/widget 出口与文本叶、重复/混合出口拒绝、
`view-2x` 仍在后缀 token 具名拒绝）；`plan980` 13/13；`ace-core` **681/681**（+79）；AV 3/3；
A2R 2/2（真实 rustc）；grammar 45/45；固定组 `cargo t ace-source ace-syntax` **PASS**
（run-jiwqgzsn）。
**注**：本批未收口；grammar id 随 LANGUAGE 切片变化，§6.7 的 id 在批末统一重绑（不做中途假精确）。
**next**：AGE-L04 选择——按 §5"I实施决定门"先冻结有界决定（widget→portrait→默认的用途序与方形、
无可见出口/零尺寸策略、view 切换的挂载事件/资源、#if 分期），再实现；随后 AGE-L05 的
Start/Tick/Frame 分离与 980 widget 槽位的完整投影（不再是 R52-1 的裁剪）。
2026-11-16 stage: work（SOURCE 收口：979 落主回验 pass，AC-HANDOFF 关闭）| PLAN-981 r13 | outcome: pass | landed `e352e462b`

**979 归档与回验**：`docs/plans/archive/979-ace-m2-schema.md` **status=archived（12/12）**。其 T-03/T-08
落主回验收据（2026-11-16）：landed `e352e462b`；`git diff 2b172901b e352e462b -- experimental/ace`
为空、`2b172901b` 为祖先、`age_package.rs`/`plan981_age_fields.rs` 与已复核 `f94a29293` 逐字节相同；
实跑 `plan981` **147/147**、AV **3/3**、A2R **2/2**（真实 rustc）、grammar **45/45**、core
conformance **678/678**，exit 全 0；g01 三件套 hash 不变；独立 12 行 fields 审计 **12/12 PASS** ⇒
**AC-HANDOFF closed**，V-09④/V-10/V-11/V-DETAIL-01 关闭；V-11 边界 L04/AG-A 归 AC-08 残留
（owner 981/LANGUAGE+966、980+981/LANGUAGE）。其 T-04 汇总在 `e352e462b` 跑六组并集 **5/6 PASS**
（唯一红=PLAN-1062 语料基线，具名排除、不充 PASS），T-05 final review pass + findings 修复后再复审
pass，T-06/T-07 归档/清理完成。
**本包据此勾 B-SOURCE-I/V/R**：四个 IF（MODULE/REF/FIELDS/ORIGINAL）各有永久正反例、真实 AV/A2R
腿与固定组证据（`cargo t ace-source ace-syntax` = source 757 / syntax 155 PASS，run-6qtf0682），
落主 `e352e462b`；SOURCE 子面交接完成。**LANGUAGE 继承完整面**（IF-981-FIELDS 的 AG-A/L04 行
由 B-LANGUAGE 消费并闭合，不在 SOURCE 提前声明）。
2026-11-16 stage: review（三次 resync 增量复审）| PLAN-981 r13 | review_scope: phase | outcome: **pass** | commit `2b172901b`

**为什么三次 resync**：二次 resync 通过（`a242bd645`）后 master 又落了 PLAN-975 B1 phase fold
（`8e0c4d1d6`，char.to(Str) + data-domain faces）与 docs 提交；按纪律再并一次（merge `2b172901b`，
父 `a242bd645`/`80870e748`），**取代前一 fold 目标**。仅 1 处冲突：`core/schema/text-record.atom`
按并集——`ExprKind` 收 975 的 `text_from_char` 并保留 981 的 `text_display`/`fstr`；`TypeKind` 保留
981 的 `f32` 与 975 列表；对 master 的 atom diff 仅 2 处增行、零删除。grammar diff 为空。
**增量审计**：43 个 master-only 文件逐个比对 0 不一致；另 4 个双侧文件（`a2x-worker`/`av` 的
conformance 注册、`core/src/source/{adapter,resolve}.rs`）为干净并集；R53 证据文件逐字节相同。
**975 面未丢**：`char_to_str_975` 2/2、`b1_slice_975` 1/1。
**实跑**：`plan981` **147/147**、`plan980` **13/13**、grammar **45/45**、固定组
`cargo t ace-source ace-syntax` **两组 PASS**（run-6qtf0682）。树 clean。
**next**：fold `2b172901b` → 979 按落主 SHA 重验 → AC-HANDOFF 关闭。

2026-11-16 stage: review（二次 resync 增量复审）| PLAN-981 r13 | review_scope: phase | outcome: **pass** | commit `a242bd645`

**为什么二次 resync**：复审通过（`38394f144`）后 master 已落 980 STATE 批（`97ca19c3b`），基线前进；
按纪律把 master 再并入分支（merge `a242bd645`，父 `38394f144`/`97ca19c3b`），**取代 fold 目标**。
**增量审计**：手工解冲突恰好 2 文件（`age_component.rs` 的 `instance_stmts` 可见性/`overrides` 参数、
`frontdoor.rs` 传空 map）；61 个 master-only 文件逐个与 `97ca19c3b` 比对 0 不一致；grammar diff 为空；
R53 证据文件与 `38394f144` 逐字节相同。
**981 修复**：`field_type` 前移到 init 脱糖之前——否则 980 STATE 的 init 形状诊断（同样的
`source.age.component-unsupported`，定位在字面量）会先命中、把"类型 token 处具名拒绝"挤掉。实测
`f float = 1.0`/`f f32 = 1.0` 两面均 @31 类型 token；`n int = x.y` 仍按 init 形状拒绝 @37（未被吞）；
`str`/`Str` 仍合法。
**实跑**：`plan981` **147/147**、`plan980` **13/13**、`ace-core` **678/678**（+79/1 ignored）、
AV p981 **3/3**（+980 STATE 面 7/7）、A2R p981 **2/2**、grammar **45/45**、固定组
`cargo t ace-source ace-syntax` **两组 PASS**（run-qa5zo9wl，wall 585.5s）。树 clean。
**next**：FF fold `a242bd645` → 979 按落主 SHA 重验 → AC-HANDOFF 关闭。复审者限制同上（同仓/同树/
同模型独立上下文；phase pass 不授予整包 `reviewed`、不替代 T-05）。

2026-11-16 stage: review（R53 关闭复审·三）| PLAN-981 r13 | review_scope: phase | outcome: **pass** | commit `38394f144`

**独立复审（第五轮，R52/R53 闭环）**：R53-1 ①–⑤、R53-2、R53-3 全闭。11 个槽位体逐个 cross-face
复测：age-component **11/11 `Ok`**、age-package **11/11 `Err source.syntax`**（含 watch 单行形态，
失明点消除）；`ds[0].code == "source.syntax"` 断言保留，文件 EOF 换行在。`git show --stat
38394f144` = test-only（1 file +1/−1）；`git diff df7550484..38394f144` 只含该测试文件，grammar/
core/src/conformance.rs 为空 ⇒ 固定组 `run-c2uejjpj` 复用成立（ace-source **757** / ace-syntax
**155**，两组 PASS，wall 760.5s）。复跑：`plan981` **147/147**（0.31s）、grammar **45/45**（0.47s），
均 exit 0；树 clean。复审者限制：同仓/同树/同模型的独立 subagent 上下文，phase pass **不授予整包
`reviewed`**、不替代 T-05 全批复核。
**上一轮 work 回执**：`93953b2f3`（R53-2：11 槽位 + `source.syntax` 断言）、`54ff7939c`（真实槽位
形态 + EOF 换行）、`38394f144`（watch 点号与头同行）——三轮均 test-only，grammar/lib 未动。
**fold 前置**：主检出 `experimental/ace` 干净（63 项未提交全在 docs/plans）；`av/tests/conformance.rs`
当前干净（fold 时再核 owner）→ FF fold `38394f144` → 979 按落主 SHA 重验 → 关闭 AC-HANDOFF。

**落主（2026-11-16）**：master 合并 `2b172901b` → **`e352e462b`**（`--no-ff`；master 当时在 `d71a776e5`，仅 docs 提交）。
落主后主检出实测：`plan981` **147/147**、`plan980` **13/13**、grammar **45/45**、AV p981 **3/3**、
A2R p981 **2/2**（真实 rustc）。`experimental/ace` 无 WIP。**待 979 在 `e352e462b` 上复核
V-09④/V-10/V-11/V-DETAIL-01 并关闭 AC-HANDOFF**；本批为 SOURCE 阶段 fold，Plan 整体仍 `executing`。

2026-11-16 stage: work（R52-1/R52-2 修复）| PLAN-981 r13 | outcome: partial | commit `df7550484`

**R52-1（blocker）**：把 980 的 widget 成员槽位（`msg`/`design`/`named-view`/`bind`/`routes`/
`expose`/`lifecycle`/`computed`/`watch`/`timer` 与 widget 形态的 `on`）的 alt 与规则门收窄回
`[age-component]`，age-package 只保留它**描述得了**的面（`model`/`on`/`view`/`view-2d`/`view-3d`）。
新增永久用例 `p981_age_package_rejects_widget_member_slots`（`design`/`msg`/`bind`/`expose` 四例，
必须在**解析期**拒绝，不得静默接受）。
**R52-2**：grammar id 记录更正——`age-component` `7a439096…` 等于 master 的 980 面（980 B-DIALECT
的语义扩展，**不是**本次门并集造成的——resync 前的 SOURCE 是 `9c78ebbd…`，master 已是 `7a439096…`，两者不同值）；`age-package` 在该合并提交为 `881e3506…`（R52 修复后 `ab740137…`）；
`age-h0` 仍 `64915bd1…`。
**R52-4 记录为已知边界（§6.7 第 8 条）**：`str`/`Str` 是合法 model 字段，但其初值的 CPU 面仍
`Unresolved`（只有 int/bool 有值；author Expr 仍在 `field_inits`）。
**验证（本提交实跑）**：`plan981` **147/147**；`plan980` **13/13**；`ace-core` **678/678**（+79）；
AV p981 **3/3**；A2R p981 **2/2**（真实 rustc）；grammar **45/45**；固定组
`cargo t ace-source ace-syntax` **两组 PASS**（exit 0，run-c2uejjpj）。工作树干净。
**next**：新基线（r13 + `df7550484`）独立复审 → fold（仍待 `av/tests/conformance.rs` owner 落定）
→ 979 按落主 SHA 复核。

2026-11-16 stage: review（SOURCE resync 独立复审）| PLAN-981 r13 | review_scope: phase | outcome: needs_fix | commit `25293cded`

**身份/限制**：同树独立 subagent 上下文复审；不授予 reviewed、不合并、不改勾选与 status。
**合并审计**：master 侧 1693 文件 100% 采纳；SOURCE 侧 50 文件与合并前相对 master 的集合一致（无静默回退/删除），零冲突标记；9 处并集逐项核对通过（`ComponentView` 四字段、双 `PackagesFace`/缓存、`aggregates` 车道、atom 并集、去重 texts）。
**实跑**：`plan981` 146/146；`plan980` 13/13；AV 3/3；A2R 2/2（真实 rustc）；grammar 45/45；`ace-core` 677/677（+79/1 ignored）；串行单例上界 ≤0.50s，无 10s 风险。
**逐 IF**：MODULE / REF / FIELDS / ORIGINAL 均 **pass**（FIELDS 的条件见 R52-1）。
**findings**：**R52-1 blocker**——门并集把 980 的 9 个成员槽位（`msg`/`design`/`named-view`/`bind`/`routes`/`expose`/`lifecycle`/`computed`/`timer`）带进 age-package：这些源能**解析并被接受**，但 `AgePackageComponent` 无对应字段、内部表达式不进唯一 verify（实测 `design{theme: nosuch_ident}`、`computed{double => nosuch_ident + 1}`、`lifecycle{mounted => {.nosuch = 1}}`、`bind{"title" -> .NoSuchHandler}`、`view main{text nosuch_fn(){}}`、`expose{nosuch_field}` 六例全部 ACCEPTED，且加 `design` 前后 `atom_text` 长度不变）；该族既不在 12 行清单也不在范围外清单，分母测试仍绿。R52-2 needs_fix——`age-component` id 变化实为 **master 980 面的语义扩展**（SOURCE 与 master 均为 `7a439096…`，并集未改），我此前"门列表字节"的说明错误；`age-package` 新 id `881e3506…` 未登记，§6.7 仍绑 `f94a29293`。R52-3 备注：`pub node X(params…)` 不可解析属既有边界（`age-pub-node-item` 无 params 槽），g01 的 `Disc` 非 pub+params，IF-ORIGINAL 不受影响。R52-4 备注：`str` 字段可编且 `initialized=true`，但 `field_init_cpu` 对字符串字面量仍 `Unresolved`（int/bool 有值）。
**规范增量**：0 个 docs/specs 改动 ⇒ SD-01/SD-02/SD-PHASE/SD-DETAIL/SD-HANDOFF 仍 **pending merge-owned**。
**next**：B-SOURCE-I 把 9 槽位 alt/规则门收窄回 `[age-component]`（保留 age-package 的 model/on/view/view2d/3d 面），修 R52-2 的措辞与 id 登记，重跑四命令+980+固定组后再交复审；979 仍待落主 SHA 复核。

2026-11-16 stage: work（resync：master 并入 plan-981-dev，SOURCE 批与 980/972 面接通）| PLAN-981 r13 | outcome: partial | commit `25293cded`（第二父 `fe75d239b`）

**做了什么**：按方案 1 把 master（43 个 ACE 提交：972/974/977/978/980/1043 等）合入 `plan-981-dev`，
10 个冲突全部消解，**未落主**。9 个代码/测试/schema 文件按"两侧行为都保留"合并（`ComponentView`
同时有 981 的 `exit`/`node_refs`/`scene_components` 与 980 的 `tree`；`PackagesFace` 双面；
`Faces::AggregatesCt` + 绑定器内 `aggregates` 车道；atom 的 `ExprKind` 并集、`TypeKind` 保留 `f32`）。

**grammar**：30 处 profile 门自动并集 + 14 处语义合并。合并设计 = **age-package 采纳 980 富 view 作为
超集**；三条顺序/归属决定：`age-node-ref` alt 必须排在 980 各形态前（否则 `Disc d1 {}` 被 text-bare/
element 吞掉）、符号表式 `use mod: a,b` 必须排在点分导入前（parser 在 `:` 失败后不回溯）、972 的
`packages-aggregates-ct-ops` pratt 表保持 ct-only。另删掉 project.rs 中 981 旧的扁平 texts 循环
（与 `collect_view_items` 重复收集）。

**行为对账（合并特有，不是回归）**：并入 980 面后 `str`/`Str` 成为合法 model 字段类型（状态函数按
`Str` 返回、`initialized=true`），981 原"str 具名拒"用例改为断言合并后行为并注明来源；`age-component`
冻结 grammar id 现等于 master 的 980 面 `7a439096…`（980 B-DIALECT 的语义扩展，非门并集所致；age-package 在该合并提交为 `881e3506…`（R52 修复后 `ab740137…`），age-h0 仍 `64915bd1…`）。

**验证（本提交实跑）**：`plan981` **146/146**；`plan980` **13/13**；`ace-core` **677/677**（+79）；
AV p981 **3/3**；A2R p981 **2/2**（真实 rustc）；grammar **45/45**；固定组
`cargo t ace-source ace-syntax` **两组 PASS**（exit 0）。工作树干净。

**仍未闭**：① 合并基线须**重新独立 phase 复审**（r13 + `25293cded`）后才可 fold；② 主检出
`experimental/ace/crates/av/tests/conformance.rs` 的他会话 WIP 仍未落定（fold 前必须由 owner 处理，
不得覆盖）；③ fold 后 979 按落主 SHA 复核，AC-HANDOFF 才关闭。

2026-11-16 stage: work（goal 轮 51，V-11 AGE 半：出口事实 + 清单行 + 范围登记）| PLAN-981 r13 | outcome: partial | commit `f94a29293`

**来源**：979 的独立回验（收据 `fe1fbeb74`）判 V-09④/V-10/V-DETAIL-01 PASS、**V-11 partial**，
并给出三个具体缺口。本轮按方案 1 在 981 树修复（不缩范围、不改验收）。

**缺口 1（`view` 事实只在有文本叶时产生）**：`collect_facts` 现在按组件**出口**产生
`view`/`view-2d`/`view-3d`；纯场景 node 不再被要求有 UI view，也不会因为没文本叶而缺事实。

**缺口 2（2D/3D 出口没有清单行）**：根因是描述面**根本没记录出口**——`ComponentView.role`
恒为 `"view"`，且 grammar 的 2d alt 也会匹配 `view-3d`（后缀才是作者输入）。新增
`ComponentView.exit` / `AgePackageComponent.exit: AgeViewExit`，按**后缀 token 字节**判定
（`2d`/`3d`），未知后缀具名拒 `source.age.view-suffix`；`AGE_CONSTRUCTS` 补 `view-2d`/
`view-3d` 两行，逐行删反控覆盖到 **12 行全列**。

**缺口 3（L04/AG-A 无行）**：不伪造行（本表只覆盖 age-package 可表达的 node 构造），改为
**显式范围登记** `AGE_CONSTRUCTS_OUT_OF_SCOPE`（L04 → 981/LANGUAGE + 966；AG-A01..09 →
980 IF-980-FIELDS + 981/LANGUAGE），并用 `p981_fields_requirement_split_is_exhaustive`
机械核对"合同点名的必需项只能落在表内或范围外清单之一"，防止分母自己变小。

**验证（本提交实跑）**：`plan981_` **146/146**；`ace-core` **664/664**；AV p981 **3/3**；
A2R p981 **2/2**（真实 rustc）；grammar **45/45**；工作树干净。新增/改动用例：three_views +2
（出口可辨、未知后缀具名拒）、age_fields +2（纯场景不冒充 UI view、requirement 分母）与
出口/清单删行反控。

**仍不关闭 AC-HANDOFF**：本轮只补 V-11 AGE 半；979 仍须按 §6.7 四命令对**本提交**复跑，
phase fold 落主后还要按落主 SHA 复核；`view-portrait`/`view-widget` 等边界不变。

2026-11-16 stage: review（goal 轮 50，SOURCE 批修复复审）| PLAN-981 r13 | review_scope: phase | outcome: pass | commit `291f2b5cb`

**身份与限制**：同一仓库/同一工作树上的**独立 subagent 上下文**复审（非独立模型/会话身份）；不授予
reviewed、不合并、不改 Plan 勾选与 status、不声称 AC-HANDOFF 关闭。

**基线**：worktree `D:/autostack/.wt/lang-981/auto-lang`，分支 `plan-981-dev`，提交 `291f2b5cb`（修复
`78cfc4da8..291f2b5cb`，1 commit / 10 files / +482−139），base `ca88cde14`；起点与终点 `git status --short`
均为空（4 个临时探针已删、两处 conformance 注册已还原）；改面无 docs/specs/Plan/其它 crate。

**R47-1..R47-8 闭合**：全部 **closed**（R47-6 为未变备注）。R47-1：`resolve_refs` 现按导入可见性解析——
未导入已声明 → `ref-invisible`（带声明模块 + `use libx: Far` 提示）；同名多模块无导入 → 新增
`ref-ambiguous`（列两模块）；`use zz:` / `use aa:` 分别解析到被导入模块（非模块键首个）；非 node 导入 →
`ref-not-a-node`；3 跳 `pub use` 链 → 声明模块、非 pub 中间导入不外泄（`source.module-symbol`）；加导入后
同入口恢复；两个旧 "imported" fixture 真加了 `use` 并断言 `refs[0].module`；scene 元素与实例引用两出口同判。
R47-2：`p981_fields_construct_table_is_frozen` 逐字冻结 10 行键集（删/改名必红），逐行反控补到 **10 行全列**；
探针实证"删表行 checker 仍 Ok"，故分母只能靠该冻结断言——它确实存在。R47-3：包面 `count int + flag bool`
与 bool-only 可编、`i32` 可编、`float/f32/str` 与未知 ident 类型都在**类型 token** 具名拒
`source.age.component-unsupported`、979 组件面同码同 span；bool model 能走完 AV→A2R(真 rustc)→`value:3`。
R47-4 §6.7 措辞已与 AV 实现一致；R47-5 g01 坏 kind span 已钉原 token + REF 恢复例；R47-7 原源
`content_sha` 已钉（复核三夹具与 age-examples 原源逐字节相同）；R47-8 换序/环已成永久用例。

**grammar 身份**：`auto-source.at` 仅 +7 行，`age-model-ty` 两条 `alt(profiles:[age-package])` 的 float/f32，
且该规则只被 `age-model-field` 使用。age-package id `6aff6320…`→`6a437142…`（有意、轮 49 已记录），
age-component / age-h0 / text-record / core **逐字节不变**；`plan981_grammar_scope` 14 邻居面冻结且 45/45 绿。

**实跑（cwd=experimental/ace，探针前后各一次，一致）**：`plan981` **142/142**（总 660；0.45s→1.95s，
体 0.24-0.25s）；AV p981 **3/3**（0.25-0.27s）；A2R p981 **2/2**（真实 rustc，0.61s/1.95s，体 0.33s）；
grammar **45/45**（0.58-0.64s）。改动模块串行（`--test-threads=1`）：age_ref 12 例 0.03s、age_fields 8 例
0.04s、package_modules 9 例 0.03s、original_g01 15 例 0.08s、model_field_types 6 例 0.04s ⇒ **单例上界
≤0.08s**，无 10s 风险。相邻面未回归：scene_elements 7、multimodule_identity 7、resource_identity 8、
model_handlers 4 全绿；清探针后强制重编零 warning。

**逐 IF**：MODULE **pass**、REF **pass**、FIELDS **pass**、ORIGINAL **pass**。

**findings**：R50-1 minor（§6.7 把 `plan981_model_field_types` 记成 7 例，实为 6 例）——**本轮已改**；
备注：N1 §6 REF"两个后端"子句仍无专例（已知、未新增、交接件未声称）；N2 age-package grammar id 无字面
冻结值（沿用既有设计）；N3 逐行反控命中判据用 `starts_with`（现实现无漏检）。

**规范增量**：修复提交 0 个 docs/specs 改动 ⇒ SD-01/SD-PHASE/SD-DETAIL/SD-HANDOFF 仍 **pending merge-owned**；
轮 48 设的三项落盘前置（R47-1/2/3）已闭合，可按实际行为沉淀，但保留 §6.7 已知边界与"979 未回验"口径。

**仍未闭**：979 独立回验（V-09④/V-10/V-11/V-DETAIL-01）；`view-portrait`/`view-widget`、GPU/窗口/输入、
两平台游戏、module 级 const/var、g01 的 A2R compute 边界、§6 REF 两后端子句（N1）维持 pending。
**next**：交 979 按其原 V 独立回验（四命令见上）；通过后才 phase fold / merge-owned SD 沉淀；本轮 pass
不替代 T-05 全批复核。

2026-11-16 stage: work（goal 轮 49，R47-1/2/3 修复 + R47-5/7/8 补钉）| PLAN-981 r13 | outcome: partial | commit `291f2b5cb`

**按轮 48 复审的三项修复，全部先复现再修。**

**1. R47-1（blocker，REF 可见性）**：`resolve_refs` 原先按**全包** node 名匹配、从不查导入。
实测复现：未导入的 `Far f1 {}` **被接受**；同名 `Widget` 在两个模块时静默绑模块键首个。
现在跨模块引用必须**从声明模块导入**（沿 `pub use` 链解析到真正声明地；不需要 `pub`，
与 2026-11-16 裁定一致）。诊断互斥且具名：`ref-undeclared` / `ref-invisible`（报文给出声明
模块与 `use <mod>: <Name>` 修复）/ `ref-not-a-node` / 新增 **`ref-ambiguous`**（同名多候选且
未导入，报文列出候选模块）。两个名为 imported 的旧用例**真加了 `use`**（此前 fixture 里没有
任何 use，等于没测）。

**2. R47-2（FIELDS 分母）**：新增 `p981_fields_construct_table_is_frozen` 冻结 10 行键集；
逐行删反控从 5 行补到**全 10 行**（`node`/`model`/`model.init`/`view`/`view.scene` 现在删投影
必红）。

**3. R47-3（model 字段类型两面不一致）**：状态函数返回类型原先**硬编码 i32**，
`flag bool = true` 被共享检查判成 `expected i32, found bool`；`float`/`f32` 在包面是关键字而
`age-model-ty` 无对应 alt，整个 node 退化成 `expected fn, found node @0`。现在状态函数返回
**字段自己的类型**，`age-model-ty` 补 `float`/`f32`（仅 age-package 面），两个 AGE 面都在
**类型 token** 处报同一具名码 `source.age.component-unsupported`。**grammar 身份**：`age-package`
的 grammar id 因该 alt 变化（其余 14 邻居面的冻结值不变，`plan981_grammar_scope` 绿）。

**4. 备注项**：R47-4 措辞已在 §6.7 更正（g01 的 CPU 值来自 `row.cpu`；AV 腿对无 text 的组件
执行 0 个函数）；R47-5 补 g01 坏 kind 的**原 token span** 断言与 REF 的"修好导入即恢复"例；
R47-7 把三个夹具的 `content_sha` 钉进用例；R47-8 补"模块换序身份不变"与"模块环具名拒
`source.module-cycle`"两条永久用例。

**验证（本提交实跑）**：`plan981_` **142/142**；`ace-core` **660/660**；AV p981 **3/3**；
A2R p981 **2/2**（真实 rustc）；grammar **45/45**；工作树干净。既有红同前（`unused import`
与 ace-av test 警告为 HEAD 存量）。

**仍未闭**：979 独立回验；五次复审无 CLEAN（本轮修复待轮 50 复审）；`view-portrait`/
`view-widget`、GPU/窗口/输入、两游戏、module 级 const/var、g01 的 A2R 边界按 §6.7 pending。

2026-11-16 stage: review（goal 轮 48，SOURCE 批独立复审）| PLAN-981 r13 | review_scope: phase | outcome: needs_fix | commit `78cfc4da8`

**身份与限制**：本次是同一仓库/同一工作树上的**独立 subagent 上下文**复审（非独立模型/会话身份），
第五次尝试 CLEAN；不授予 reviewed、不合并、不改 Plan 勾选与 status。

**基线**：worktree `D:/autostack/.wt/lang-981/auto-lang`，分支 `plan-981-dev`，提交 `78cfc4da8`，
base `ca88cde14`（55 commits / 49 files / +8181−180），起点与终点 `git status --short` 均为空；
改面无 docs/specs/Plan 文件。

**实跑（cwd=experimental/ace，两次一致）**：`ace-core --test conformance plan981` **128/128**（总 646，
0.45s/1.92s，用例体 0.24s）；`ace-av --test conformance p981_age_package` **3/3**（总 1111，0.29s/3.17s，
体 0.02s）；`ace-a2x-worker --test conformance p981_age_package` **2/2**（总 238，真实 rustc，0.59s/0.58s，
体 0.36s）；`ace-syntax-grammar` **45/45**（1.22s/0.57s）。与 §6 交接件逐项一致。新用例计时
（`--test-threads=1`）：`plan981_resource_identity` 8 例、`plan981_multimodule_identity` 7 例，
模块总各 0.03s、**单例最大 0.02s**，无 10s 风险。警告仍为既有文件，改面零新增。

**逐 IF**：MODULE **pass**（两文件闭包、跨模块 helper 6/5→16/15、AV 真值与真实 rustc、source map、
`symbols_by_module` 同名不覆盖、私有/缺失具名拒、同入口恢复；环与换序经探针实测成立但无永久用例）；
REF **fail**（未导入引用被接受、同名跨模块按模块键首个静默绑定、"已声明但不可见"一类不可达，
与 §5 oracle 及 2026-11-16"导入即意愿"裁定冲突；两个名为 imported 的用例实际不含 `use`）；
FIELDS **partial**（10 行清单与 `collect_facts`/`age_construct_coverage` 已行使、删投影必红；
但缺分母检查，10 行中 `node`/`model`/`model.init`/`view`/`view.scene` 删掉不会红；`model` 行只被
int 证明）；ORIGINAL **pass**（g01 三文件与 age-examples 逐字节一致、四图形几何/颜色 alpha/
声明序/`visible:false`、资源来源与改值传播、坏 kind/未知资源具名拒、恢复；错误 span 经探针
实测精确落原 token，但永久用例只断码）。

**findings**：R47-1 blocker（REF 导入/歧义，§5 line 196 + IF oracle）；R47-2 needs_fix（FIELDS 分母
检查缺失）；R47-3 needs_fix（`model` bool/float 在包面与 979 组件面不同判，float 退化成 @0
`expected fn, found node`）；R47-4..R47-8 non-blocking（AV 腿对 g01 执行 0 个函数、ORIGINAL 错误
span 未钉、A2R 只观测实例 0、原源 hash 未钉、环/换序仅探针）。

**规范增量**：本提交 0 个 docs/specs 改动 ⇒ SD-01/SD-PHASE/SD-DETAIL/SD-HANDOFF 为
**pending merge-owned**（不算失败）；但 SD-HANDOFF 落盘前须先闭 R47-1/R47-2/R47-3，
不得按现交接措辞写"三向可见性已验""model 字段全 typed 覆盖"。

**仍未闭**：979 独立回验（V-09④/V-10/V-11/V-DETAIL-01）欠；AC-B-SOURCE/AC-HANDOFF 不得自证关闭；
`view-portrait`/`view-widget`、GPU/窗口/输入、两平台游戏、module 级 const/var、g01 的 A2R 边界
维持 §6.7 pending。**next**：B-SOURCE-I 修 R47-1→R47-3，B-SOURCE-V 同入口复跑四命令＋新增
分母用例，交轮 49 独立复审。

2026-11-16 stage: work（goal 轮 47，多模块身份：场景元素解析 + 符号面）| PLAN-981 r13 | outcome: partial | commit `78cfc4da8`

**审计尺子转向"按名字的两张面"。** 两处丢失、一处错判（探针源与输出见
`plan-artifacts/plan-981/round47-probe*.txt`）：

**1. 符号面跨模块撞车**：`AgePackageCompilation.symbols` 的键是**源名**。两个模块各有一个
`pub fn seed()` 时映射只剩一条（实测 `{"seed": "p3_appm6_helper_seed"}`——`other::seed` 的
符号不可达）。已补 `symbols_by_module`（`(模块全键, 源名) → 符号`）与
`symbol_of_in(module, name)`；旧面保留但文档明示"多模块有损"。

**2. 场景行只有 `kind` 拼写**：`Disc d1 { … }` 不说明 Disc 声明在哪个模块；同包两个模块各有
一份 Disc 时消费者只能按名字字符串匹配（§5 禁止）。新增 `AgeSceneRow.element:
AgeSceneElement::{Shape2d | Element3d | Node{module,node}}`，由共享绑定事实解析，并沿
`pub use` 重导出链找到**最终声明模块**（实测 `mid` 重导出解析到 `app::shapes`，不是 mid）。

**3. 同名普通函数被当成场景元素**：旧判据只问"这个名字在导入集里吗"，于是 `use libx: Disc`
（libx 里是 `pub fn Disc`，没有 node）也能让 `Disc d1 { … }` 过关，产出一条指向空处的场景行。
现在必须解析到**真实 node 声明**，否则 `source.age.scene-element`。

**验证（本提交实跑）**：`plan981_` **128/128**（新增 `plan981_multimodule_identity` 7 case）；
`ace-core` **646/646**；AV p981 **3/3**；A2R p981 **2/2**（真实 rustc）；grammar **45/45**；
工作树干净。既有红同前轮（`unused import` 与 `ace-av` test 警告均为 HEAD 存量）。

**仍未闭**：979 尚未回验；四次独立复审均无 CLEAN（本包不得自证关闭 AC-HANDOFF）；
B-SOURCE-V 绑定的其余行、SESSION/RUST 不在本轮；`view-portrait`/`view-widget` 仍缺。

2026-11-16 stage: work（goal 轮 46，资源身份 + 实参逐位 + 引用解析）| PLAN-981 r13 | outcome: partial | commit `19d89a03b`

**机械审计的下一站：`AgeResource` 与场景行里的资源引用。** 上轮说"其余字段无缺口"，
本轮把同一把尺子对准资源面，三处静默丢失（探针源覆盖全部构造，逐字段打印核对）：

**1. 构造身份丢失（真实缺口）**：`AgeResource` 只有 `port`。`mesh2d.circle(30.0)` 与
`mesh2d.quad(30.0, 40.0)` 都是 `Mesh2D`，而端口字段表（`Mesh2D` = w,h）对 circle 的单个
直径根本不成立——消费者只能按实参个数**猜**形状。已补 `op`（声明表稳定符号
`age.mesh2d.circle.v1`，与 `AgeResourceOp.symbol` 同一身份，不另起命名）。

**2. 实参逐位事实丢失（更糟：静默错位）**：具名实参被 `Arg::Named => continue` 整体丢弃
（`material2d.color(r: 0.9, …)` → `components = []`）；非字面量实参被跳过且**不留位**：
`material2d.color(base.r, 0.3, 0.2, 0.5)` 原先是 `[0.3, 0.2, 0.5]`，消费者会读成
r=0.3,g=0.2,b=0.5。共享调用检查按**书写序**对齐实参，故同样按书写序取值。现在
`args: Vec<AgeResourceArg>` 逐位保留（认不出 = `Unresolved`，位置不丢），`components`
只在**全部**实参为 f32 字面量时才完整，否则为空；`symbol` 只在"恰好一个字符串实参"时给出。

**3. 资源引用只剩末段符号**：`material: res2.red` 的 CPU 面只有 `Symbol("red")`，两个模块
各有一个 `red` 时消费者无从判断——这正是 §5 禁止的"按名字字符串决定引用关系"。新增
`AgePackageComponent.resource_refs`（`row/attr/spelling/target`），target 是检查器已解析的
`Resource{module,name,port}` 或 `Param{name,port}`。顺带修同一段的可见性洞：限定写法原先
只按**裸名**导入集判，`use res1: red` + `material: res2.red` 会被接受（res2 从未导入，
只是它也有个 `red`）；现在必须**从限定词所指的模块**导入，否则
`source.age.scene-resource-invisible`。

**IF-981-FIELDS 分母同步**：新增构造行 `view.scene.resource`（consumer =
`AgePackageComponent.resource_refs`），并把"清空 `resource_refs` → 必须红"接进逐行删反控；
清单 9 → 10 行。g01 原源的两类目标同时被断言：`Main` 的 4 条 `resources.*` 全部解析到
`app::resources`，`Disc` 的 `material: tint` 解析为 `Param{tint, Material2D}`。

**验证（本提交实跑）**：`plan981_` **121/121**（新增 `plan981_resource_identity` 8 case）；
`ace-core` **639/639**；`ace-av` p981 **3/3**；`ace-a2x-worker` p981 **2/2**（真实 rustc）；
`ace-syntax-grammar` **45/45**；工作树干净。既有红（非本轮引入，HEAD 上已存在）：
`ace-core (lib test)` 一条 `unused import: PackageAssemblyContext`、`ace-av` 30 条 test 警告。

**仍未闭**：979 尚未回验；四次独立复审均无 CLEAN（本包不得自证关闭 AC-HANDOFF）；
B-SOURCE-V/R 其余行与 SESSION/RUST 不在本轮；`view-portrait`/`view-widget` 仍缺。

2026-11-16 stage: work（goal 轮 45，机械审计 + model 初值 CPU 面）| PLAN-981 r13 | outcome: partial | commit `31b35565e`

**改用可执行的审计法：写一个覆盖全部构造的源，把每个 `pub` 字段逐个打印出来**，问"作者的哪一句
输入没进来"。这比"再看一遍"有效——立刻两处收获。

**收获 1（真实缺口）**：`field_inits` 只有 `Expr`，**没有 CPU 面**。我在场景属性上立过"只给 AST
等于把活推给消费者"的规矩，model 初值上还留着。已补 `field_init_cpu: Vec<(String, AgeSceneValue)>`，
用同一个解码器，同样**不猜**：`n int = 42` 解码；`n int = seed()` → `Unresolved`，原始
`field_inits` 仍在。

**收获 2（补用例时暴露）**：`AgeSceneValue` **根本没有整数变体**——`n int = 42` 因此解成
`Unresolved`。解码器只处理 f32/bool/元组/符号。已加 `Int(i64)` 与 `as_int()`，并断言
`as_float()` 对整数返回 `None`（两种值不能悄悄混同）。

**其余字段核对结果**（无缺口）：`head_span`/`legacy_widget`/`public`/`initialized`/`params`/
`refs`/`texts.references`/`scene.cpu`/`resources.{components,symbol}`/`modules.{source_len,flat_start}`
全部承载了对应作者输入。

**验证**：`plan981_` **113/113**；`ace-core` **631/631**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 44，model 初值 + handler 行为面）| PLAN-981 r13 | outcome: partial | commit `20a1346da`

**上一轮我说"审计已基本穷尽"——那是个没有证据的结论。** 这轮对 `AgePackageComponent` 用同一把
尺子再问一遍，又撞到两例（第五、第六）：

```
fields       = [("count","int")]        ← 作者写的 `= 3` 不在描述面
handler_msgs = ["Inc","Life.Tick"]      ← `+1` 与 `+2` 的区别完全丢失
```

这两项恰好都是 979 要核的：
- "改被引用 helper 初值 → 两后端结果随之改变"需要初值；
- "2:1 事件、各 N 次 Tick 后两实例各自推进几次"需要知道每个 handler 加多少。

**源侧结构本来就带着它们**（`ComponentField.init`、`ComponentHandler.assigns`），只是描述面没透出。
已补 `field_inits` 与 `handler_assigns`（消息 → 其赋值列表，按声明序）。

实测：`init count = IntLit(3)`；`Inc -> count = count + 1` 与 `Life.Tick -> count = count + 2`
现在**可见地不同**；即使 `Life.Tick` 写在 `Inc` 前面，声明序仍保留。

**3 个永久用例**；同时断言既有 `handler_msgs` 契约仍在（新面没有取代旧面）。

**验证**：`plan981_` **112/112**；`ace-core` **630/630**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

**方法论修正**：不要再凭"已看过一遍"宣布穷尽。可执行的做法是**对每个 `pub` 字段问"作者的哪一句输入
没进来"**——问一遍就必有收获，这轮就是证据。

2026-11-16 stage: work（goal 轮 43，文本叶依赖面）| PLAN-981 r13 | outcome: partial | commit `f5e2a25ea`

**用同一把尺子审全部对外结构**："这个字段承载了**所有**输入，还是只承载了我当时想到的那种？"

对每个 `pub struct` 逐个问，撞到：`text label(c)` 的描述只有

```
role=view  kind=View  fn=__age_p_hidden_0_0  def=...    ← 再没有别的
```

**实参 `c` 完全没了。** 979 要核"文本是否由 model 字段驱动"时看不到这层关系，只能去反查
隐藏函数的 HIR 体——那正是"入口"应当消除的工作。

`AgePackageText.references` 现承载该文本表达式提到过的名字（按出现序、去重），由
`collect_expr_names` 提取（**只提名字**，类型与绑定仍归共享 resolver）。

实测：`text label(c)` → `["label","c"]`；`text "hello"` → `[]`；`text pair(a, b)` → `["pair","a","b"]`。
三个永久用例覆盖，含顺序/去重性质。

**验证**：`plan981_` **109/109**；`ace-core` **627/627**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 42，Model3D 路径不再丢弃）| PLAN-981 r13 | outcome: partial | commit `8baa75f44`

**又一处静默数据丢失。** 问"一个 `Model3D` 资源到底带了什么"时发现：

`collect_age_resources` 只收 f32 实参：

```rust
for a in args { if let Some(v) = f32_literal(e) { components.push(v); } }
```

而 `model3d.glb("assets/ship.glb")` 的实参是**字符串**，于是落空被丢弃。实测：

```
ship  port=Model3D  components=[]      ← 路径无处可寻
```

别处也没有承载它的字段——消费者无法知道该资源指向哪个 GLB，而 AGE 定义的资源身份
（"该文件完整字节的 SHA256 + byte_length"）因此不可达。

`AgeResource` 现增 `symbol: Option<String>` 承载字符串实参，`components` 的文档明确为
**数值实参**字段。两个方向都断言：`model3d.glb("…")` → `symbol = Some("assets/ship.glb")` 且
`components` 空；`material2d.color(…)` → 四个分量且 `symbol = None`。

**证伪**：让字符串分支重新静默丢弃 → 红，报 "the GLB path must survive into the description"；
数值用例仍绿。

**验证**：`plan981_` **106/106**；`ace-core` **624/624**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 41，**CPU 值面**）| PLAN-981 r13 | outcome: partial | commit `c9e4973a7`

**补上 979 V-DETAIL-01 真正要的东西："CPU 读取入口"。**

此前 `AgeSceneRow.attrs` 只有 `Vec<(String, Expr)>`——**原始 AST**。每个消费者都得自己再判
"这是不是 `TypedLit{Float}`、要不要剥一元负号、元组怎么拆"。那不是入口，是把活推给消费者。

现在 `AgeSceneRow` 另带 `cpu: Vec<(String, AgeSceneValue)>`，值已解码：

| 变体 | 例 |
|---|---|
| `Float(f32)` | `x: -20.0` |
| `Floats(Vec)` | `position: (1.5, -2.5)` |
| `Bool(bool)` | `visible: false` |
| `Symbol(String)` | `kind: circle`、`material: resources.red` → `"red"` |
| `Unresolved` | 形状无法在本批解码 |

**`Unresolved` 是重点**：参数引用（`position: (x, y)`）、方法链、计算值一律给 `Unresolved`，
**不伪造默认值**；原始 `attrs` 仍在、顺序一致，需要作者拼写时回看。

**"真可消费"的证据（不是一个新字段而已）**：g01 的几何断言已改为**读 `row.cpu`**，这使得测试
自己的 `f32_of`/`pair_of` AST 助手变成死代码——那正是 979 消费者本来要重写的那套。`pair_of` 已删，
`f32_of` 只在少数刻意检查原始 AST 处保留。

**4 个新用例**（`plan981_cpu_surface`）：各值种类解码；限定资源解成末段符号；参数引用保持
`Unresolved` 且原始表达式仍在；`cpu`/`attrs` 键序一致。`Unresolved` 那条**证伪过**：把那一路
改成伪造 `0.0` → 红，报 "a parameter reference must NOT be given a fabricated value"。

**验证**：`plan981_` **104/104**；`ace-core` **622/622**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 40，refs 归属修正）| PLAN-981 r13 | outcome: partial | commit `dbe76e4c9`

**修掉一个直接影响 979 V-10（实例身份）的缺陷。** `refs` 原先按**模块**聚合，于是模块里
**每个**组件都收到整份引用列表。实测（`A` 引用 `Widget x1`、`B` 引用 `Widget y1`）：

```
Widget    -> ["Widget:x1","Widget:y1"]   ← 被引用的节点收到了行
A         -> ["Widget:x1","Widget:y1"]
B         -> ["Widget:x1","Widget:y1"]
Unrelated -> ["Widget:x1","Widget:y1"]   ← 毫无引用的组件也收到
```

组件拿到**别人**的引用，被引用者看起来"自己引用自己"——这不是显示问题，正是 V-10 要核的
身份事实。

现按 `(module, node)` 键。同一 fixture 修复后：`Widget -> []`、`A -> ["Widget:x1"]`、
`B -> ["Widget:y1"]`、`Unrelated -> []`。

**新用例** `p981_each_component_gets_only_its_own_refs` 逐行断言这四个结果；**证伪**：恢复模块级
广播 → 精确红该条（报 `A's own only`），其余四条聚合用例仍绿。

**这推翻了我上一轮记的残留判断**（`840479e36` 说"按模块键是既有设计，改它是重新设计"）：
实际是一处受控的小修，而留着它等于交付一个错的实例身份事实。**"记为残留"不等于"可以不修"**。

**验证**：`plan981_` **100/100**；`ace-core` **618/618**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 39，两个真实缺陷）| PLAN-981 r13 | outcome: partial | commits `840479e36`、`1396b895e`

**目标是把场景出口加进构造表**（IF-981-FIELDS 要求表覆盖"实际 grammar/AST 的全部必需构造"，
而 `view.scene` 一直是缺的）。做这件事时暴露了两个真实缺陷。

### 缺陷 1：同模块多 node 互相覆盖 refs（静默数据丢失）

`resolve_refs` 原先对每个 node 做 `out.insert(module, resolved)`。同一模块的**第二个** node
会用**它自己的**（通常为空的）引用表顶掉第一个的结果。实测：

```
Widget + Counter（Counter 的 view 里有 `Widget w1 {}`）      -> refs = ["Widget:w1"]
同样的源 + 任意第三个 node（哪怕毫无引用）                    -> refs = []
```

**多声明一个 node 就会丢掉整个模块的实例引用**，零诊断。这与我的场景改动无关——加一个普通
`node Extra { view { … } }` 同样复现。已改为 `entry().or_default().extend(…)`。

**残留（未改，如实记录）**：`refs` 按**模块**聚合，同模块的每个组件都会拿到整份列表。
这是该表的既有设计局限（按组件归属属完整 IF-981-REF 面），改它是重新设计而非修缺陷。

### 缺陷 2：`view.scene` 不在构造表里

场景出口被投影、被断言、被消费，却**不在 `AGE_CONSTRUCTS`**，于是"清单覆盖"看不见它。
已补为一行（`AGE-L03`，consumer `AgeSceneRow`）+ `collect_facts` 对应事实。

补完后 `FULL` 正确变红（它没有场景出口），已加一个**独立** `Scene` 组件——独立是因为
一个 node 恰一个 view 出口（M1），场景不能挂在已有 `view` 的 `Counter` 上。

**证伪**：恢复覆盖式写入 → `plan981_ref_aggregation` 精确红三条，报
"declaring one more node must not erase the module's references"；两条基线正控仍绿。

**验证**：`plan981_` **99/99**；`ace-core` **617/617**；AV 3/3；A2R 2/2；零 lib 警告；工作树干净。

2026-11-16 stage: 验收对账（goal 轮 38，无代码改动，仅计划簿记）| PLAN-981 r13 | outcome: **partial（如实）** | HEAD `781d7588b`

**本轮做的是"把簿记与实际对账"，没有新增代码。** 三处发现：

**1. 计划里有一条过时的 `fail` 记录。** §7 对账表把 IF-981-ORIGINAL 记成
`fail（未交付）`，依据是轮 5 的实测（当时首诊断是 `source.unsupported: let …`）。**那早已过时**：
五片语法面全部落地，`plan981_original_g01` 14 case 全绿。已更正为 `pass（本批面）`，
并保留"当年为何是 fail"的说明——不留假记录，也不抹掉历史。

**2. IF-981-FIELDS 的实际覆盖比表格暗示的窄。** 构造表 `AGE_CONSTRUCTS` 只有 **8 行**
（node / model / model.init / on / on.lifecycle / view / view.text / view.node-ref），
覆盖的是 **AGE-L01 的组件形体**；而 **AGE-L03 要求五 view，实测只实现 3 个**：
`view` / `view-2d` / `view-3d` 通过，`view-portrait` / `view-widget` 在 grammar 层
**根本不存在**（实测报 `source.syntax`）。我新加的 `view.scene` 也还没进构造表。
已在此行如实写明"本批不宣称已覆盖"。

**3. g01 原源身份可被第三方核对。** `fixtures_g01/main.at` 的 SHA256 前 16 位 =
**`9b809a1c6d99985b`**，与 §5 要求的原源值**逐字一致**（另两份：`resources.at`
`6f993775477ebc39`/790B、`shapes.at` `dbdc5243ec590dab`/540B）。

**四条腿（本提交实测）**：`plan981_` **95/95**；AV **3/3**；A2R **2/2**（真实 rustc）；
grammar **45/45**；零 lib 警告；工作树干净。

**AC-HANDOFF 仍为 partial，不由本批自证关闭**：四次独立复审均未给出 CLEAN 结论；979 尚未实际回验。

2026-11-16 stage: 第四次复审的收尾（措辞修正）| PLAN-981 r13 | outcome: **NOT CONVERGED** | commit `781d7588b`

**`plan981_grammar_scope` 的措辞问题已修**（复审指出，它自己就在重犯本批反复出现的那个模式）：

1. 用例名曾是 `..._does_not_touch_non_age_profiles`，但列表里**包含 `AgeComponent` 与 `AgeH0`**——
   两个都是 AGE 面。名字声称了代码没有的覆盖。已改名 `p981_grammar_ids_match_the_recorded_frozen_values`。
2. 断言消息说 id "must be byte-identical to the frozen SOURCE-delivery value"。对 `age-component`
   那行**这是假的**——它的冻结值恰恰是**变后值**（`9c78ebbd…`）。该行若变红，消息会告诉下一个
   作者相反的事实。现改为"与**写下本检查时记录的值**相同"，并写明 `age-component` 记录的是变后值，
   所以红意味着**再次**漂移。

两处都不削弱检查——id 仍逐字钉住今天的 grammar，改的只是围绕它的**声明**。

**验证**：`plan981_` **95/95**；零 lib 警告；工作树干净；复审的 snapshot 目录已删。

2026-11-16 stage: **第四次独立复审 + 我引入并修回的过度收紧** | PLAN-981 r13 | outcome: **NOT CONVERGED** | commits `393a1a4c4`、`2ec226913`

### 第四次复审：NOT CONVERGED（它完成了 7 项变异中的 1 项）

它把变异 1（span）跑通了：**GENUINE** —— 恢复减法后**恰好一条**测试变红，报
`left: 245, right: 316`，读作 `"qqqqqqqqqqqqq"`，其余 88 条绿。这条通过。

其余 6 项它未跑（我中途叫停——它把我的工作误判成"外部 Codex agent 并发写"，实际是我）。
**它明确声明"不为自己没变异的项背书"**，这个态度是对的。

### 它抓到了我引入的真实回归（已修）

我在 `393a1a4c4` 里加 `attr_applies_to` 白名单时，把**组件实例**也套了进去。实测：

```
node Badge(label: int) { … }    node M { view-2d { Badge inst { label: 3 } } }
→ source.age.scene-attr: `label` does not apply to `Badge`
```

**合法的作者自定义参数名被拒。** 组件实例的元素名是任意用户 node 名、属性名是该 node 自己声明的
参数，白名单**无法穷举**——我的 `is_component` 兜底只救了碰巧列出的 14 个拼写。

修法：`attr_applies_to` 只对 `shape2d` 与冻结 3D 元素生效，组件实例一律放行；`attr_value_kind`
的 `None` 分支同样不再拒绝实例上的未知键。**两个方向都加了断言**（接受自定义参数 / 叶与 3D 仍受限）。

### 它另一条我也认同（未修，如实记录）

`attr_port` 只对少数键返回 `Some`，**其它键完全跳过端口比较**。所以"组件参数声明 `Mesh2D`、
却喂给 `tint:`"这类错配仍不被发现，非 `tint` 的资源型参数（如 `color: red`）报
`source.age.scene-value` "cannot type yet"——是**具名拒绝**而非静默接受，但该形状尚不可用。

### 计划 §9 的措辞问题（它指出，已采纳）

`p981_grammar_scope` 的断言消息对 `age-component` 那行**说反了**：写的是"must be byte-identical to
the SOURCE-delivery value"，而该行的冻结值恰恰**不是** SOURCE 交付时的值。测试名也声称"does not
touch non-age profiles"，却把两个 AGE 面列在里面。已在下一轮修正措辞（见下条）。

**验证**：`plan981_` **95/95**；`ace-core` **613/613**；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 36，第三次复审最后一处已修）| PLAN-981 r13 | outcome: needs_fix（待复验）| commit `daab4d7f5`

**比复审报的更糟**：它说**裸名**路径不查导入。实测**两种写法都不查**：

```
无 use，material: red          -> ACCEPTED
无 use，material: assets.red   -> ACCEPTED
有 use，两者                    -> ACCEPTED
```

所以早先 B5 的修复只校验"限定词等于声明模块"，**从未问过本模块是否导入了该符号**。与 B3
（场景元素种类）同一缺陷类：该用导入集的地方用了全项目名字集。现在两条路径都要求"本模块自有
**或**本模块导入"。

**修的过程暴露两件事**：

1. **`GlobalDecl.module` 是短模块名**（`main`），而 `comp.module` 是 `pkg::mod` 键
   （`app::main`）。直接比较会让**本模块自己的资源**被当成"外来且未导入"——实测
   `let ship = model3d.glb(...)` 加一行使用它的场景行，开始报 `scene-resource-invisible`。
   三个既有用例抓住了它；比较前现按短名归一。
2. **"存在但未导入"与"根本没声明"曾被合成一条诊断**。979 V-10 要求引用/可见性失败**互斥可辨**，
   现分为 `source.age.scene-resource-invisible`（报出未导入该符号的模块名，并提示加 `use`）
   与 `source.age.scene-resource-unknown`。

**证伪**：中和可见性子句 → 精确红两条新用例，其余八条仍绿。

**验证**：`plan981_` **89/89**；`ace-core` **607/607**；零 lib 警告；工作树干净。

2026-11-16 stage: **第三次独立复审（部分）+ 我的一次工作树事故** | PLAN-981 r13 | outcome: needs_fix | commits `4513b89ea`、`914efee47`

### 我的事故：`git stash` 污染了工作树

探测 grammar id 时我用了 `git stash push` / `pop`，pop **冲突**，把 `docs/` 下 15 个无关文件
注入工作树，`docs/roadmap.md` 里还留下字面冲突标记（`<<<<<<< Updated upstream`）。复审 agent
把它误判为"另一个 agent 在并发写"，其实**是我**。

处置：先把 15 个文件备份到 `plan-artifacts/plan-981/stash-accident-backup/`，再 `git checkout -- docs/`
还原；确认两个既有 stash（PLAN-002 / plan-972）都不是我创建的；工作树回到干净。教训：**探测不应
使用 stash**——用临时文件或 `git show <rev>:<path>` 取历史内容。

### 第三次复审：**NOT CONVERGED**（它只跑到静态阅读，未做变异——我要求的收尾）

它按构造阅读给出结论，其中两条必须由变异判定，我都跑了：

**Fix 1（span）——它预测"可能盲"，实测**不盲，测法是有效的。** 它的理由是 `compile_two` 把
`widgets` 放前面，于是 `main` 的 `flat_start` 可能是 `widgets.len()`。实测：`main`=0、
`widgets`=71，故障在本地 **316 > 71**，减法**确实发生**。恢复减法 → 红，报
`got Span { start: 245, end: 258 } which reads "qqqqqqqqqqqqq"`。

**Fix 4（`tint:`）——它说得对，我的测试是假的。** 删掉 `attr_port` 的 `"tint"` 分支后
**84 个用例全绿**。原因：我那条以 `tint` 命名的用例，源里写的是 `material: tint`（元素
`shape2d`、键 `material`），走的是**既有**分支；真正写着 `tint:` 键的源全是"正确类型的正控"，
走全局路径，删不删都过。**已补 3 条键就是 `tint:` 的用例**（组件实例 / shape2d 叶 / 同型正控），
删分支后两条负控真红。

**它另指一处未闭环（我未修，如实记录）**：`check_scene_resource_refs` 的**裸名**路径
（`module_spelling == None`）仍按**全项目** `bound.globals` 匹配，没有像 B3 那样查导入。这与
B3 是同一缺陷类，B3 修了、这条没修。

**验证**：`plan981_` **87/87**；`ace-core` **605/605**；零 lib 警告；工作树干净。

2026-11-16 stage: work（goal 轮 34，第二次复审剩余两项）| PLAN-981 r13 | outcome: needs_fix → 复审项清空 | commit `0a4d0bc2c`

**B3 比同族规则松（已修）**：`check_scene_element_kinds` 曾把**全项目** node 名并成一个集合，
于是"入口从未导入的某个模块里恰好同名的 node"也被当作合法场景元素——拼写撞上无关模块的名字
即可过关，而同族的 `resolve_refs` 是**查导入**的。现改用同一判据：元素必须是 `shape2d`、冻结
3D 元素、**本模块声明**的 node，或**本模块真实导入**的名字（`bound_name()`）。
证伪：恢复全项目行为 → 精确红"unimported node"一条，导入/同模块正控仍绿。

**`(red)`：复审问"应否拒绝"，答案是**不该**，且这点可验证。** 括号在投影后展开，`(red)` 与 `red`
是同一表达式。决定性证据：**坏名字加括号仍被同样拒绝**——`(resources.nope)` 与 `(nope)` 都报
`source.age.scene-resource-unknown`，与不加括号完全一致。所以括号**不绕过存在性检查**，它不携带
身份。拒绝它只是武断的严格，故**断言该行为**而非改动：四种合法写法全部解析，四种坏写法以同一
方式失败。

**验证**：`plan981_` **84/84**；`ace-core` **602/602**；`ace-syntax-grammar` 45/45；零 lib 警告；
两棵树干净。

**剩余已知缺口**："零警告"仅对 `ace-core` 成立；`ace-av` 30 条、`ace-native` 2 条——它们早于本批，
不在 SOURCE 范围内。

**第二次独立复审的 8 项 + 后续 4 项，现已全部关闭或如实记录。**

2026-11-16 stage: work（goal 轮 33，修第二次复审的 M5 第二半 + B4 `tint`）| PLAN-981 r13 | outcome: needs_fix | commit `b3a3ce171`

**M5 第二半：片段门从未接通——我上一轮的"修复"和"测试"都是盲的。**

`parse_front` 的**整模块**守卫在分派之前就拒绝了片段请求，因此经 `parse_front` **永远到不了**
`parse_fragment_front`。直接调用该入口实测（修复前）：

```
parse_fragment_front("fn f() int { return 1 }", AgePackage, …) -> Ok   （未拒绝）
```

所以上一轮我改的那处是**装饰性的**，而"验证"它的测试其实是被整模块守卫认证的。现在片段入口
**自己**拒绝（`source.domain-frontend`），测试**直接调它**、用**语法完全合法**的片段、并断言
**诊断码**（不是"是个 Diag"）。只回退片段守卫 → 红，报 "the fragment front door must refuse…"。

**范围克制**：另一处 `matches!(AgeH0 | AgeComponent)`（在**诊断优先级**处）**故意不动**——它选择先报
哪条诊断，不是拒绝；放宽会改变错误顺序。

**`tint:`：复审的实质对、机制说得不准。** 字面量 `tint: 5` 早已被**形状**检查拒（"must name a
resource"），因此从未走到端口表。真正漏掉的是**参数**路径：`node Disc(tint: int)` 喂 `material: tint`
能干净编译。`attr_port` 现把 `tint` → `Material2D`（不分元素名，因为组件实例用的是组件名，无法在
(element, attribute) 表里穷举）。

**新增永久用例**：`int` 参数喂 `tint` 被 `source.age.scene-resource-type` 拒；同型 `Material2D` 通过；
g01 真实组件实例形态（`Disc … { tint: red }` + `tint: Material2D`）仍编译。

**验证**：`plan981_` **81/81**；`ace-core` **599/599**；零 lib 警告；两棵树干净。

**仍未闭**：`(red)` 括号引用被静默接受；B3 的 node 集合不查 import；"零警告"仅对 `ace-core` 成立。

2026-11-16 stage: **第二次独立复审**（goal 轮 32）| PLAN-981 r13 | outcome: **needs_fix** | commit `98f468ed5`

**复审结论**：8 项中 **B3/B5/M1/M4/M6 判定 FIXED**；**B4/B6/M5 判定 PARTIAL**，并指出我**上一轮的 B1 结论是错的**。

### 关键：我上轮判断错误，已被证伪

我上轮测出"保留减法正常、去掉减法出错"，于是断定减法必需。**我的判据是盲的**：
减法是 `span.start - flat_start`，而 `find` 取"最后一个 `flat_start <= span.start` 的模块"。
故障在**入口模块**时 `flat_start=0`，减 0 等于没减；本地偏移小于后置模块 `flat_start` 时
`find` 也落在入口模块上——**我的两个 fixture 都落在这两种情形里**。

能看见缺陷的判据需要两条同时成立：故障在**非入口**模块，且其**本地偏移大于**其它模块的
`flat_start`。构造方式：`main` 很短，`widgets` 里先放 200 字节填充、`Nowhere` 落在本地偏移 316。
旧代码报：

```
报告 (245, 258) → 读出 "qqqqqqqqqqqqq"   （填充注释！）
正确 (316, 329) → 读出 "Nowhere m1 {}"
```

**已修**：`flat_to_module` 不再换算——span 本就是模块本地（binder 原样 clone 每模块候选）。
**新永久用例** `p981_ref_span_is_correct_for_a_deep_fault_in_a_non_entry_module` 编码该布局，并同时断言
span 指向 token **与** fixture 非盲（`want > main.len()`）。恢复减法即红，报 "qqqq"。

### 本轮仍未闭（复审指出，下一轮处理）

- **B4 缺 `tint:`**：`attr_port` 没有 `("shape2d","tint")` 条目，而 **g01 组件实例用的正是 `tint:`** ——
  `node Disc(tint: int) { … tint: tint }` 仍能编译。B4 的不变量没覆盖 g01 实际路径。
- **M5 片段半边恒真**：`matches!(frag, FrontOutcome::Diag(_))` 接受**任何**诊断；它之所以绿是因为
  探针源本身有语法错（`source.syntax`），与入口拒绝无关。应断言诊断**码**。
- **B6 的 `(red)`**：括号包裹的引用被静默接受（parser 解开了括号）。
- **B3 的 node 集合不看 import**：全项目 node 名并集成一个集合，未导入模块里的 node 也被当作合法元素
  （比同族 `resolve_refs` 松）。
- **"零警告"只对 `ace-core` 成立**：`ace-av` 30 条、`ace-native` 2 条。

**验证**：`plan981_` **78/78**；`ace-core` **596/596**；零 lib 警告；两棵树干净。

2026-11-16 stage: work（goal 轮 31，**M4 已修**，全部首次复审项关闭）| PLAN-981 r13 | outcome: needs_fix | commit `6f7ed207c`

**M4 我先测量再下结论，结果与复审所述有一半不同**：

变异：把 model 字段投影包进 `if false`（语法有效，能编译但永不记录字段），再跑套件——

```
p981_fields_full_source_satisfies_the_construct_table        FAILED  ← 能抓到
p981_fields_every_projection_row_is_individually_guarded      ok      ← 抓不到
```

所以复审说对的是**第二个**测试，不是第一个。总覆盖检查**确实会咬**（我此前"两个都绿"的印象是错的）；
逐行守卫只手工清空克隆出来的描述，从不触碰投影器，因此无法察觉"投影器停止发射"。

**修法**：逐行守卫保留其真实职责（证明 `collect_facts` 会对缺行反应），另加
`p981_real_projection_actually_produces_rows`——断言**真实投影产物**非空（源里声明了 model 字段、
文本叶、节点引用，投影就必须各产出一项），且 `collect_facts` 给出对应键。该测试**不手工编辑任何描述**。

用同一变异证伪：它以 "Counter declares a model field, so projection must produce one" 变红。现在
"投影被静默移除"会触发**两道**独立检查，而不是一道。

**验证**：`plan981_` **77/77**；`ace-core` **595/595**；零警告；两棵树干净。

**复审 8 项状态**：B3/B4/B5/B6/M1/M5/M6 **已修并有证伪**；M4 已修（范围经实测修正）；
仅余 **B1 短依赖匹配边界**（真实但窄，已写入代码注释）。

2026-11-16 stage: work（goal 轮 30，**M5 已修**）| PLAN-981 r13 | outcome: needs_fix | commit `f49ad7c45`

**M5（比死代码更糟：一个绿测试在为"从未发生的拒绝"作证）**。用**真实入口**实测（修复前）：

```
AgeH0        -> source.domain-frontend  (拒绝)
AgeComponent -> source.domain-frontend  (拒绝)
AgePackage   -> Ok                      (未拒绝)
```

通用入口接受 `age-package` 并返回**零项目、零领域行**——正是本工作声称要守住的输入的一条静默接受通道。

**两个成因，都已修**：
1. `domain_profile_refusal` 早已扩展到含 `AgePackage`，但**两个调用点的
   `matches!(profile, AgeH0 | AgeComponent)` 没同步**——helper 知道，调用者不知道。整模块与
   片段两个入口现都纳入。
2. `age_package_refusal` **零生产调用**，唯一使用者是一个**直接调 helper 而非调门**的测试。
   已删除该函数；那个测试改为驱动真实的 `parse_front`（整模块 + 片段两种 kind）。

**证伪**：只回退 `matches!` 守卫 → 精确红该测试，报"the generic front door must refuse the
age-package profile"。

**验证**：`plan981_` **76/76**；`ace-core` **594/594**；`ace-syntax-grammar` 45/45；零警告；两棵树干净。

**仍未闭**：M4 `AGE_CONSTRUCTS` 由自身输出派生（删投影+删 `collect_facts` 行仍全绿）；
B1 短依赖匹配边界。

2026-11-16 stage: work（goal 轮 29，**M1 + M6 已修**）| PLAN-981 r13 | outcome: needs_fix | commit `558b50483`

**M1（重复出口静默合并）**：`view_role` 只是被重复赋值，于是两个 `view-2d`（或两个 `view`、或
2D+3D 并存）合并成一条扁平列表，**丢失"哪些叶属于哪个出口"**。规范明说 *single view exit per
node*。现第二个出口是具名拒绝（`source.age.view-exit`），消息带**两个**字节位置。证伪：中和守卫
→ 三个重复出口控制全红、单出口控制仍绿。

**M6（恒真断言）**：原断言 `!consts.is_empty() || atom_text.contains("Material2D")` 的右项一旦端口
类型登记就恒真。替换它时**又暴露一个更糟的问题**：该用例名叫"**模块级** `let` 推断端口"，但源把
`let` 放在**函数体内**——那只是局部绑定，永远不进 `description.resources`。恒真断言**掩盖了名实不符**。

改用真正的模块级 `let`，并逐项断言推断出的端口：`[("a","Material2D"), ("c","Mesh2D"), ("m","Mesh2D")]`。
证伪：翻转一条期望即红，且报出该真实数据。

**4 个永久用例**（`plan981_view_exits`）：重复 view-2d / 重复 view / 2D+3D 混合各自被拒；
单出口与**无 view 的纯逻辑组件**仍编译。

**验证**：`plan981_` **76/76**；`ace-core` **594/594**；`ace-syntax-grammar` 45/45；零警告；两棵树干净。

**仍未闭**：M4 `AGE_CONSTRUCTS` 由自身输出派生（删投影+删 `collect_facts` 行仍全绿）；
M5 `age_package_refusal` 对通用 CST 入口是死代码；B1 短依赖匹配边界。

2026-11-16 stage: work（goal 轮 28，**B5 + B6 已修**）| PLAN-981 r13 | outcome: needs_fix | commit `7d272c80a`

**B6（引用形状）**：形状匹配以 `_ => continue` 结尾，于是 `resources.red.clone()`、`nope()`、
`resources.red.deep` **从未进入存在性检查**——方法/字段链里的未知资源干净编译。现全部具名拒绝
（`source.age.scene-resource`），并区分"计算值"与"链式表达式"两种措辞。

**B5（限定词通配）**：查找里 `decl.module == m || m == "resources"` 是**硬编码通配**——任何名为
`resources` 的模块无需 `use` 即可达，与"复用 binder 可见性事实"自相矛盾。现在限定词必须等于
声明所在模块。**该 bug 的真实形态**：模块叫 `assets`、源码写 `resources.red`，通配让它通过。

**两项都做了证伪**：只恢复通配 → 精确红 `p981_qualifier_wildcard_is_gone` 一条；把形状匹配退回
`_ => continue` → 精确红两条 call/chain 控制。隔离准确。

**同轮发现我自己的弱测试**：第一版"错误限定词"用 `othermod.red` 对模块 `resources`——**即使有通配
也会失败**，等于什么都没测到。已换成 `assets` + `resources.red` 这一只有通配才能通过的形态。

**7 个永久用例**（`plan981_resource_refs`）：方法调用、裸调用、深链各自被拒；通配形态被拒；
限定与裸写法仍可；g01 的 `tint: resources.red_alpha` 仍编译。

**验证**：`plan981_` **72/72**；`ace-core` **590/590**；`ace-syntax-grammar` 45/45；零警告；两棵树干净。

**仍未闭**：M1 重复出口键；M4 `AGE_CONSTRUCTS` 自派生；M5 `age_package_refusal` 死代码；
M6 一条恒真断言；B1 短依赖匹配边界。

2026-11-16 stage: work（goal 轮 27，**B4 已修**）| PLAN-981 r13 | outcome: needs_fix | commit `a3e5e6cdd`

**B4（复审 blocker）已复现并修复。** `material: tint` 里的 `tint` 是 node **自己的参数**，此前被当作"实例化时才绑定、无需检查"而 `continue`——于是
`node Disc(x: float, tint: int) { view-2d { shape2d d { material: tint } } }` **干净编译**：整数塞进 `Material2D` 位，零诊断。`Mesh2D` 参数同样能过。

**修法**：参数确实不是模块全局资源，但它**仍然**必须是该位置要求的端口。判据与具名资源**同一条**（`attr_port(element, attribute)`），只是事实来源不同（参数的声明类型 vs 全局的声明类型）。拒绝信息读作：
`material` on `shape2d` requires a `Material2D` resource, but the node parameter `tint` is declared `int`。

**证明断言是真门**：把新分支关掉 → 两个负控红、两个正控仍绿；恢复后全绿。

**4 个永久用例**（`plan981_param_ports`）：`int` 参数被拒、`Mesh2D` 参数被拒、`Material2D` 参数可、**g01 原形（`material: tint` + `tint: Material2D`）仍编译**——收紧没有破坏原源。

**验证**：`plan981_` **65/65**；`ace-core` **583/583**；零警告；两棵树干净。

**仍未闭**（复审提出）：B5 `resources` 模块通配；B6 方法/字段链中的未知资源；M1 重复出口键；M4 `AGE_CONSTRUCTS` 自派生；M5 `age_package_refusal` 死代码；M6 一条恒真断言；B1 短依赖匹配边界。

2026-11-16 stage: work（goal 轮 26，**B3 已修**）| PLAN-981 r13 | outcome: needs_fix | commit `d7855e84f`

**B3（复审 blocker）已复现并修复。** 投影层只看得到名字的**拼写**，于是任何首字母大写的
标识符都被当作组件实例：拼错的 `Discc` 与根本没声明的 `NotANode` 都能编译，产出一条绑到
空处的场景行。

**修法**：种类用 binder 的既定事实裁定——合法元素 = `shape2d` / 冻结 3D 元素 / **项目里某模块
真实声明**的 AGE node。跨模块沿用与 IF-981-REF **同一条**"符号表导入即可见"规则（不要求
`pub`），不新增第二套可见性。

**证明断言是真门**：把新加的调用注释掉 → 两个负控红、三个正控仍绿；恢复后全绿。

**5 个永久用例**（`plan981_scene_elements`）：拼错具名拒、未声明具名拒、跨模块 node 仍可、
同模块 node 仍可、`shape2d` 与 `cylinder3d` 不受收紧影响。

**验证**：`plan981_` **61/61**；`ace-core` **579/579**；`ace-syntax-grammar` 45/45；零警告；两棵树干净。

**仍未闭**（复审提出）：B4 `material:`/`tint:` 指向 node 参数绕过资源类型检查；B5 `resources`
模块通配；B6 方法/字段链中的未知资源；M1 重复出口键；M4 `AGE_CONSTRUCTS` 自派生；M5
`age_package_refusal` 死代码；M6 一条恒真断言；B1 的短依赖匹配边界。

2026-11-16 stage: work（goal 轮 25，**B1 复核：我无法按其所述复现，且我的"修复"是错的**）| PLAN-981 r13 | outcome: needs_fix | commit `695d297b2`

### B1 的真相（与复审报告不同，以探针为准）

复审报 B1 为"span 双减、随无关模块漂移"。我复现了那个**症状**，但进一步测量后发现边界不同：

- **减法是必需的**。保留减法时，`ref-undeclared` 与 `ref-not-a-node` 在依赖 73B / 157B 下都把 span
  精确落在出错 token（`at_token=true`）；去掉减法（我的第一版"修复"）反而在 40B 依赖下指向
  `"{\n    view {\n"`。
- 因此 `age_components` 的 span **是扁平坐标**，减法**对**。我把对的改坏了，已回退。
- **真实边界**：`find(|s| span.start >= s.flat_start)` 假定被减偏移**大于**任何前置模块长度。
  当依赖模块比该偏移更短时会匹配错模块。这是**边界 bug，不是"整体不该减"**。已写进
  `flat_to_module` 的注释，未修（属汇总/RUST 批）。

### 本轮真修的两处

1. **`p981_ref_diagnostics_carry_the_original_module_span` 此前不可能红**：它只断言 span 落在
   `main` 区间内，而 `main` 的 `flat_start == 0`，任何小 span 都满足。现锚定到**精确 token**
   （`main.find("Nowhere")`），偏移错一位即失败。这条复审说得对。
2. **删掉我自己刚写的 drift 测试**：它在 bug 态与修复态下**都是绿的**——测不出东西的测试比没有更糟。

### 仍未处理（复审提出、本轮未动）

B3 大写/拼错的场景种类静默接受；B4 `material:`/`tint:` 指向 node 参数绕过资源类型检查；
B5 `resources` 模块通配；B6 方法/字段链中的未知资源；M1 重复出口键；M4 `AGE_CONSTRUCTS`
由自身输出派生；M5 `age_package_refusal` 对通用入口是死代码；M6 一条恒真断言。

### 方法论

本轮最重要的产出不是修了什么，而是**没有盲从复审结论**：复现症状后继续测量，发现"双减"的诊断
是错的（减法必需），真正的问题是匹配边界。同时接受了复审关于"假测试"的判断——那条我确实写错了。

**验证**：`plan981_` 56/56；`ace-core` 574/574；lib 零警告。

2026-11-16 stage: **独立复审（真独立上下文）** | PLAN-981 r13 | outcome: **needs_fix** | HEAD `a19e9b844`

本轮起了一个**无本会话历史**的独立 reviewer 做对抗性复审。结论：**B1–B6 阻塞合并**。我逐条自行复核了其中三条，**全部成立**（不是"可能"）。

### 已复核确认的 blocker

**B1（我复核：成立）** —— 领域诊断的 span 被**双减**一次模块偏移。`flat_to_module` 对
**已是模块本地**的 span 再减 `flat_start`。决定性实验（主模块字节不变，只改**另一个模块**长度）：

```
dep=40B  → span=(25,38)  命中 "{\n    view {\n"     ← 错字节
dep=110B → span=(66,79)  命中 "Nowhere m1 {}"        ← 正确
```
span 恰好随**别的模块**长度漂移。当依赖模块足够长时 bug 消失——这正是
`plan981_original_g01` 与 `plan981_model3d` 一直绿的原因。这直接否证 AC-HANDOFF
"错误报原文件 span" 与 §5 IF-981-REF "带原模块/字段 span"。

**B2（我复核：成立）** —— `p981_ref_diagnostics_carry_the_original_module_span`
**不可能红**：它用一次成功编译的 `flat_start`（`main` 为 0）做比较，任何小 span 都满足，
包括 B1 的错误 span。它**通过 B1 的 bug 而通过**。

**B3（我复核：成立）** —— 拼错的组件种类与不存在的名字**被静默接受**：
```
kind=Discc（libx.Disc 的拼错）→ ACCEPTED rows=["Discc:d1"]
kind=NotANode（哪里都没声明）  → ACCEPTED rows=["NotANode:thing"]
```
大写首字母即被当作合法组件名，从不校验该 node 是否存在。产出一条绑定到空处的场景行。

### Overclaim（我复核：成立）

**grammar 作用域**：`age-component` 的 grammar id **确实变了**——我新增的 `age-view-item`
规则被该面继承（基线该规则出现 0 次，HEAD 出现 2 次）。因此：
- 交接件与 §7 的"其余 profile 字节不变"**是错的**；
- `plan981_grammar_scope` 的冻结值编码的是**变更后**的 id，**测不出它本该测的漂移**。

**"零编译警告"**：`ace-av` 有 30 条、`ace-native` 2 条——我此前只看了 `ace-core`。

### 未自行复核、由 reviewer 以证据提交的（采信，待修）

B4 `material:/tint:` 指向 **node 参数**时绕过全部资源类型检查（`int` 型参数可塞进 `Material2D` 位）；
B5 `resources.<name>` 对**任何**名为 `resources` 的模块硬编码通配，无导入即可达；
B6 `resources.nope.clone()` / `nope()` / `resources.nope.deep` 等方法/字段链中的未知资源**不被存在性检查**。
M4 `AGE_CONSTRUCTS` 由描述自身派生，删投影+删 `collect_facts` 行仍全绿；
M5 `age-package` 未被通用 CST 前门拒绝——`age_package_refusal` 只有测试调用，生产路径是死的；
M6 `p981_resource_services_compile_and_let_infers_the_port` 的断言恒真（`atom_text` 必含 `Material2D`）。

### 复审确认成立的部分

单一前门、两后端（AV 3/3 + A2R 真 rustc 2/2，读到 `6` 且随 helper 编辑变 `16`，非 emit-only）；
g01 三文件与 `age-examples` **逐字节一致**（reviewer 直接比对，不靠字节数断言）；几何/颜色/顺序/隐藏/资源来源断言为真；node 无需 `pub` 双向皆测；21/22 旧 profile 字节不变，`age-h0` 未变。

### 结论与下一步

**本轮不宣称交付。** 计划状态应视为 `needs_fix`：B1/B2 意味着"错误定位原文件 span"这条验收线**既未实现、又被假绿认证**，必须先修 span 模型（模块本地 vs 扁平只应有一次换算），并让 B2 的测试**能够**红。B3–B6 属静默接受族，同一批修。

2026-11-16 stage: work（goal 轮 23，`Model3D` 端口 + 两处静默接受）| PLAN-981 r13 | outcome: 进行中 | commit `a19e9b844`

**`Model3D` 是真实端口**（规范 `ptype_of_3d`：int/float/Model3D），但形状与 2D 端口不同：构造器收**路径**（`model3d.glb("ship.glb")`），身份是 GLB 的 SHA256 + byte_length，不是 f32 分量。故声明为 `Model3D{sha256, byte_length}`，能力键 `age3d.resource`——没有为"一致性"把它压成 2D 形状。

写它的负控**暴露了两处既有缺陷**，同一族（静默接受）：

1. **`check_scene_resource_refs` 硬编码 `material | tint`**：于是 `model:` 与 `mesh:` 被整段跳过——把不存在的资源绑到 `model:` 上能干净编译。现改用**与前门同一张**属性分类表，两者无法再漂移。
2. **端口字段类型映射是 `"f32" => F32, _ => I32`**：`Model3D` 的 `sha256`（声明 `Str`）被静默发射成 `t_i32`——每个非 f32 端口字段类型都是错的。现按拼写显式映射，未知拼写在构造期 **panic**，不再变成整数。

另修：属性的**期望端口**现在被检查，不只是"是某个端口"。`model3d` 的 `model:` 要求 `Model3D`，绑 `Material2D` 现按名拒绝（`attr_port` 声明映射）——此前任何端口在哪儿都被接受。

**4 个永久用例**（`plan981_model3d`）：GLB 构造产出端口；`Model3D` 可作参数类型；`model3d` 叶绑定具名资源；`Material2D` **不能**当 `Model3D`（正是抓出上述两洞的那条）。

**验证**：`plan981_` **56/56**；`ace-core` **574/574**；零警告；两棵树干净。

2026-11-16 stage: work（goal 轮 22，**补上 AC-B-SOURCE 的 3D view**）| PLAN-981 r13 | outcome: 进行中 | commit `d3e3d5de2`

**AC-B-SOURCE 要求 UI/2D/3D 三 view 都进同一 verify；此前只有 UI 与 2D 被验证过，3D 从未跑过。** 一探即发现真缺口。

**发现（我的实现错把两套规则合并）**：属性分派把 `kind:` 对**每个**场景元素都按 2D 几何符号表
（circle/quad）校验，于是**合法的 3D 源被拒**。两套作者面的种类规则本就不同，我却压成了一套：

- `view-2d` 的叶是 `shape2d`，种类是**属性** `kind: circle|quad`；
- `view-3d` 的叶是 `cylinder3d` / `model3d` / `group3d`——**元素名即种类**，且没有 `kind` 属性；
- 规范里**不存在 `shape3d`**。我探测时先臆造了它，被自己的校验拦下——**正说明该按规范探，而不是按假设探**。

**交付**：`AGE_3D_ELEMENT_KINDS` + `is_3d_element`；`attr_value_kind` 扩到 3D 面（`model:`/`mesh:`
作资源引用，`yaw`/`pitch`/`scale` 作值）；场景元素按 view 分派校验（2D = `shape2d` 或组件名，
3D = 冻结元素集），越界报 `source.age.scene-element`；`kind:` 在 3D 元素上具名拒绝
（`source.age.scene-attr`），不再静默接受一个只属于 2D 的属性。

**6 个永久用例**（`plan981_three_views`）覆盖三 view + 两个方向跨表拒绝：3D 元素名当 2D `kind`
被几何表拒；`kind:` 用在 3D 元素上被拒。

**验证**：`plan981_` **52/52**；`ace-core` **570/570**；零警告；两棵树干净。

2026-11-16 stage: work（goal 轮 21，把"声明"变成"机械检查"）| PLAN-981 r13 | outcome: 进行中 | commit `f4c675eab`

**本轮修的是一处"我自己也只声明、没验证"的空白**：交接件里写了"加了 `age-package` 裁剪后，其余 profile 的 grammar id 字节不变"——**没有任何断言支撑它**，只是叙述。

`plan981_grammar_scope` 把它变成不变量：14 个邻居面（Core…AgeH0）的 grammar id **冻结**在本交付的实测值。未来任何一次 grammar 改动若波及别面，立即变红并**报出是哪个 profile**，而不是等下游发现"另一个面的语法悄悄变了"。

**证明它是真门、不是同义反复**：我把其中一个冻结值改成全零，测试以
`Core: this profile's grammar must be byte-identical to the frozen SOURCE-delivery value; a change here means the age-package trim leaked into another surface`
失败；恢复后确认绿。

同文件另两条支撑不变量：`age-package` 不是任何邻居的**别名**（与 14 面全不同）；其身份在多次取用时确定且是 64 位 sha256 十六进制。

**另做了两项审计（无需改动）**：① AGE 投影提前退出路径上的每个 `continue`，要么**先发具名诊断**，要么是有意的"node 声明由领域前门投影"跳过——无静默丢弃通道；② FIELDS 的负控**真的删掉必需行并期望红**，没有一条被拉宽成必绿。

**验证**：`plan981_` **46/46**；`ace-core` **564/564**；零警告；两棵树干净。

2026-11-16 stage: handoff（goal 轮 20，交接件 + 命令复跑更正）| PLAN-981 r13 | outcome: 进行中（无新提交，计划文件改动）

**交接件已折叠进 §6**（979 独立回验的输入）：身份/入口签名/结果结构/能力键/命令与实测/具名未闭项/不变式。

**本轮的主要产出是发现自己上一轮的错**，两处：

1. **我误判"V-09 缺 A2R 双实例腿"**——实际早就存在：`plan981_age_package_a2r` 已用
   `compile_age_package_with_scenario` 走真实 rustc，并断言两实例 6/5
   （`assert_value("TWO_INSTANCES", &out, 6)`）。本轮先写探针准备新建，读完既有用例才
   发现重复，已删除探针未提交。教训同前几轮：**先读既有证据再判断缺口**。

2. **交接件命令写错、且复审记录的数字口径错了**：写的是
   `cargo test -p ace-av plan981`，实际该命令跑的是这个 crate 的**别的**测试二进制，
   报 "11 filtered out"，看起来像用例不存在。正确形式必须显式 `--test conformance`：
   `cargo test -p ace-av --test conformance p981_age_package` → **3/3**；
   A2R 同理 → **2/2**。此前记的 `ace-av 11/11` / `ace-a2x-worker 1/1` 是**整 crate**
   数字，不是本批用例数。两处均已在 §6 与 §9 就地更正，并把踩坑写进交接件。

**复跑确认（按交接件原文）**：`plan981_` **43/43**；AV **3/3**；A2R **2/2**；grammar 45/45；`ace-core` 561/561；零警告；两棵树干净。

**仍未闭**：一次独立上下文复审；979 对 V-09/V-10/V-11/V-DETAIL-01 的实际回验；GPU/窗口/输入与两游戏属其它批。

2026-11-16 stage: review（goal 轮 19，**独立复审**）| PLAN-981 r13 | outcome: 进行中 | commit `11545e9de`

**复审结论：四个 IF 的证据是真实的，但发现一处"两条都绿、中间有洞"的风险，已堵。**

**发现 1（已修）**：资源**存在性/类型检查**与资源**收集器**是两套判据，此前没有任何证据证明二者一致。若任一漂移，一行可以引用一个永远到不了 `description.resources` 的资源——两套绿测之间的静默洞。新增交叉不变量 `every_referenced_resource_is_on_the_resource_surface`：遍历场景行，断言每个被引用名字都在该面上。漂移现在会**挂测试**，不需要人工比对。

**发现 2（已修，注释）**：`scene_value_type` 的 `Unary` 分支保持**操作数类型**——对 `-20.0`（float）与 `!b`（bool）都正确，故它不是"负数字面量特例"。行为本就正确，是注释把规则说得比代码窄。

**复审取证（无需改动，登记为事实）**：
- 四条交付腿**不是 emit-only**：`ace-av` 断言运行态（`instance_states` 16/15）与逐实例文本；`ace-a2x-worker` 真实跑 rustc 并读回值。
- **能力门确实生效、不是摆设**：用错误能力集合编译 g01 闭包会以 `source.module-capability` 失败，消费方无法绕过准入。
- 35 个改动文件内未引入 TODO/FIXME/hack 标记。
- 未发现"删必需映射仍绿"或"拉宽断言求绿"的 workaround；`rejects_an_unknown_resource` / `rejects_a_bad_geometry_kind` / `resource_ports_are_nominal_not_interchangeable` 等负控都是真拒绝。

**四条腿实测**：`plan981_` **43/43**；`ace-core` **561/561**；本批 AV 用例 **3/3**、A2R 用例 **2/2**（`--test conformance p981_age_package`）；`ace-syntax-grammar` 45/45；零警告。

（更正：前一轮记的 `ace-av 11/11` / `ace-a2x-worker 1/1` 是**整个 crate** 的数字，不是本批用例数，且命令少了 `--test conformance` 会跑错二进制。以本节命令为准。）

**未闭项（如实）**：本轮只做了 SOURCE 侧自审。按 AGENTS.md 的独立复审门，正式合并前仍需一次**独立上下文**的复审（不是同会话自查），以及 979 侧对 V-09/V-10/V-11/V-DETAIL-01 的实际回验。GPU/窗口/输入与两游戏仍属 SESSION/RUST/LANGUAGE/PROOF。

2026-11-16 stage: work（goal 轮 18，**IF-981-ORIGINAL 验收面完成**）| PLAN-981 r13 | outcome: 进行中 | commits `0cdc19c2a`（S）、`8a862d058`（验收补全）

**资源值现在可见且可断言**（此前不可）：`GlobalDecl` 只留声明类型、丢掉初值，导致模块常量**实际持有的值**（g01 资源颜色）无法被任何断言看到。真实修法：`GlobalDecl.init` 带上初值（它本来就在同一处 AST 里），前门据此产出 `AgeResource { name, module, port, components }`——端口取自 `age_resources` 声明表，分量只收**字面量** f32 实参（非字面量留空，不猜）。

**IF-981-ORIGINAL 逐条对照 §7 矩阵，现全部有永久证据**（`plan981_original_g01` 13 case）：
| 矩阵要求 | 用例 |
|---|---|
| 原 main/resources/shapes 闭包经同一前门 | `compiles_through_one_front_door` |
| 红圆 (-20,10)/直径30、蓝quad(0,0)/30×30、绿圆(25,-15)/直径24、黄quad(-45,-25)/12×12 且 hidden | `geometry_matches_the_baseline` |
| 颜色/alpha 按原独立断言 | `resource_colours_match_the_baseline`（0.9,0.3,0.2,0.5 / 0.2,0.3,0.9,0.5 / 0.2,0.8,0.3,0.6 / 0.95,0.9,0.1,1.0） |
| order 按原独立断言 | `scene_rows_keep_declaration_order` |
| 实例化的 Disc 定义/资源确来自导入模块 | `rows_reference_distinct_resources` + `editing_a_resource_changes_the_consumed_value` |
| 改被引用资源则值变化 | `editing_a_resource_changes_the_consumed_value`（red α 0.5→0.75，green 不变） |
| 未知属性/坏 kind 拒绝 | `rejects_a_bad_geometry_kind`、`rejects_an_unknown_resource` |
| 错误定位原文件 span | 诊断 span 由 `flat_to_module` 归位（REF 组另有专测） |
| 完整原源 hash 记录 | `fixtures_are_the_recorded_original_sources`（字节长度 + 作者表面逐项钉住） |
| 恢复 | `recovers_after_a_rejected_edit` |

**四条交付腿实测**：`plan981_` **42/42**；`ace-core` **560/560**；本批 AV 用例 3/3、A2R 用例 2/2（真实 rustc）；`ace-syntax-grammar` 45/45；零警告。

**边界（不越界宣称）**：本轮证明的是 **g01 原源的 CPU 投影**——同一 Schema、共享绑定/检查、双后端消费面。GPU 图像/窗口/输入与两游戏仍属 SESSION/RUST/LANGUAGE/PROOF 各批，本次不充作 PASS。

2026-11-16 stage: work（goal 轮 17，IF-981-ORIGINAL 补完资源引用）| PLAN-981 r13 | outcome: 进行中 | commit `0e3c2b129`

**补上上轮具名指出的两项缺口中的第一项：资源引用此前只校验形状、从未解析。**

**真实缺陷（静默接受）**：`material:`/`tint:` 的引用从不被解析，故 `tint: resources.nope`（根本不存在的名字）能**干净编译**。修复前实测通过、修复后具名拒绝。

**我自己的真实缺陷（被测试抓住）**：我按 `Method { recv: Var, name }` 匹配引用形状，而 AST 把限定引用拼成 **`Field { base: Var, name }`**。匹配因此**一个都没命中**，所有引用静默通过——正是这个检查要堵的缺口。改成 `Field`（+ 裸 `Var`）才真正生效。

**交付**：`check_scene_resource_refs`——`material:`/`tint:` 必须指向已声明模块全局（否则 `source.age.scene-resource-unknown`，报出符号名），且其声明类型必须是 AGE 端口（否则 `source.age.scene-resource-type`）。判据用 binder 自己的 `globals` 事实，不建第二套可见性。

**`material: tint`（引用本 node 参数）正确**地**不**算资源引用——它在实例化时绑定，故跳过而非报缺失（g01 的 shapes.at 正是这么写的）。

**新增 3 个永久 case**（`plan981_original_g01` 现 9）：未知资源名被拒且诊断具名；裸导入名（`tint: red_alpha`）与限定写法同等接受；四行场景绑定**四个不同**资源（不塌到第一个）。

**验证**：`plan981_` **38/38**；`ace-core` **556/556**；`ace-syntax-grammar` 45/45；零警告；两棵树干净。

2026-11-16 stage: work（goal 轮 16，**IF-981-ORIGINAL 核心达成**）| PLAN-981 r13 | outcome: 进行中 | commit `483570888`

**g01 原源闭包端到端编译通过。** 三份逐字复制的 `age-examples` g01 源（resources.at / shapes.at / main.at）走**一个**共享前门，产出受检文档；实测 3 模块、两个 node、四行场景按声明序。

**上一轮认定的"跨模块 node 构造借用"是错的**（用测量纠正）：真正的阻塞点是 `resources.at` 的模块级 `let x = material2d.color(...)` 降级成**合成模块全局 setter 调用**，而 `CallGlobalSet` 把操作数当**裸值**规划。verify 要求记录实参以借用包装到达，`plan_arg` 只对**已解析 fn** 这么做，合成 setter 不走那条路 → `Material2D`（记录）裸传。现 `CallGlobalSet` 采用与 `plan_arg` 同一纪律：记录值加 view 借用，标量仍按值。这是 **PLAN-960 既有路径上的真实缺陷**，不是 AGE 专属绕法。

**场景行现已暴露**为 `AgeSceneRow`（kind / name / 属性按书写序 / span）——没有它无法断言 g01 的真实几何。顺序即声明序，亦即 2D 绘制序。

**新增永久覆盖** `plan981_original_g01`（6 case 全绿）：
1. 原源闭包经一个前门编译（3 模块、两个 node）；
2. 场景行保持**声明序**（red、blue、green、yellow）；
3. 逐值几何与 auto-ui 基线一致（red (-20,10) d30；blue (0,0) 30×30；green (25,-15) d24；yellow (-45,-25) 12×12 且 `visible: false` 作为行事实保留，不是 `if` 销毁）；
4. 坏 `kind:` 具名拒绝（`source.age.scene-kind`）；
5. 改一个值只改该值，顺序不动；
6. 被拒的编辑不污染下一次编译。

**验证**：`plan981_` **35/35**；`ace-core` **553/553**；`ace-syntax-grammar` 45/45；零警告；探针全删；两棵树干净。

2026-11-16 stage: work（goal 轮 15，路线 A 第 3 步：可见性裁定 + 面身份统一）| PLAN-981 r13 | outcome: 进行中 | commit `b7f5a913d`

**用户裁定（2026-11-16）：放宽 node 引用可见性——node 不需要 `pub`。** 依据是规范原文「接受普通 Auto 模块工程（…**普通 `use`**、具名资源与引用）」以及 g01 原源就是 `node Disc` + `use shapes: Disc`；符号表导入本身即显式跨模块意愿。

- 引用解析不再按 `pub` 过滤 node 目标，**与包 binder 对齐**。两层对同一份源给不同结论才是真正的缺陷。
- `plan981_age_ref` 中两个断言旧严格规则的用例改为**正向证据**（导入的 node 可解析并记录声明模块，有无 `pub` 皆然）；`ref-undeclared` / `ref-not-a-node` 的独立诊断保留。

**同轮两处结构性修正**：
1. **AGE 前门改用 `AgePackage` 身份做解析与类型检查**（此前传 `TextRecord`，而发射早已用 `AgePackage`）。这个不一致是"门说 TextRecord、面规则永不触发"这一整类缺陷的根源——我在端口、f32、move 模式上各撞了一次。
2. **端口类型按"文档可能提到"登记**（node 的 typed 参数在领域行上，不在 `program.items`）；**值函数只带本值真正引用到的参数**（带全部会让每个值函数都卡在它从不触碰的那个端口参数上）。

**g01 现状（未变）**：仍停在 `verify.ownership-required | record param 0 requires a borrow argument`，位于 main.at 三处实例化点。已测得它是**跨模块 node 构造**（记录实参需 view 包装）问题，与属性类型无关。

**验证**：`plan981_` 29/29；`ace-core` 547/547；零警告；探针全删；两棵树干净。

2026-11-16 stage: work（goal 轮 14，路线 A 第 3 步：参数模式缺陷 + 阻塞点精确定位）| PLAN-981 r13 | outcome: 进行中 | commits `64dcacb3a`、`197c7da38`

**本轮修**：`move` 参数在 AGE 面**静默退化成 `view`**。发射器只在 `profile == TextRecord` 时写 `move` 词，而 AGE 面在 `AgePackage` 下发射，于是所有权意图丢失。两个面共用 text-record 描述符（其 schema 本就接受 `move`），故门控一并放行。（这是在排查另一处所有权错误时发现的；缺陷本身独立成立。）

**同轮用测量否掉的假设与结论**：
- **`move` 不是端口的正确模式**：verify 明确拒绝「record 不拥有缓冲却声明 move；move 只用于 `Str` 与资源记录」。端口是纯 f32 记录，而 `record_resource_map` 以「拥有 `StrOwned` 字段」推导资源性，端口永远推不出来。我先试了 `move`，随即回退，模式保持 `view`。
- **g01 剩余阻塞点已精确定位**：`verify.ownership-required | expr eN: record param 0 requires a borrow argument (view or .mut)`，位于三处 `Disc` 实例行。记录类型实参必须以借用包装到达，而 `plan_arg` 只在 callee 是已解析 **fn** 时才包装——这些调用不走那条路径，于是记录值**裸传**。这是**调用计划（call planning）**问题，不是属性类型问题。

**验证**：`plan981_` 29/29；`ace-core` 547/547；零警告；探针全删（含一次误提交后补删，见 `197c7da38`）；两棵树干净。

2026-11-16 stage: work（goal 轮 13，路线 A 第 3 步：属性分类校验）| PLAN-981 r13 | outcome: 进行中 | commit `40b2288ab`

**本轮完成（大幅推进）**：场景属性按**它是什么**分类校验，而不是塞进一个伪造的值类型：
- `kind:` 是**冻结几何符号**（AGE 规范 §1 的 `circle|quad`）→ 查符号集，坏拼写报 `source.age.scene-kind`；
- `material:`/`tint:` 是**资源名**→ 字面量/计算值按形状拒（`source.age.scene-resource`）；名字是否存在与可见性仍归共享 binder 的导入事实，不建第二套检查；
- 其余（`position`/`size`/`x`/`y`/`diameter`/`visible`）是普通 Auto 值 → 降级成隐藏函数；
- 未知属性名**具名拒绝**（`source.age.scene-attr`，列出接受集），不再静默携带。

同轮：
- node typed 参数**穿进**值函数，故 `node Disc(x: float, y: float, …)` 里的 `position: (x, y)` 能把 `x`/`y` 解析成**声明类型**；
- 元组值属性**逐分量降级**（每个标量一个函数），既保住分量检查，又不必打开 aggregates 面（元组**类型**属那一面）；
- 负数字面量按其操作数定类型（`-20.0` 是 float）；
- typed 端口作参数时按**值传递**（它是资源句柄，不是可变聚合）；
- 隐藏名冲突检查改为排除**本前门生成的每一个函数**（按保留前缀）——原先的显式清单漏掉场景族，把自己的函数误报成用户冲突。

**g01 实测现状**：真实闭包已通过 parse/bind/resolve/typecheck/emit（场景行就位），停在 `verify.ownership-required | record param 0 requires a borrow argument`，位置正是三处 `Disc` 实例。**结论**：下一层不是属性类型，而是**跨模块 Node 构造**（把资源值传进 node 的 typed 参数）。

**验证**：`plan981_` 29/29；`ace-core` 547/547；零警告；探针全删；两棵树干净。

2026-11-16 stage: work（goal 轮 12，路线 A 第 3 步：场景属性值降级）| PLAN-981 r13 | outcome: 进行中 | commit `482eac90c`

**本轮完成**：场景属性值（`x: -20.0`、`position: (0.0, 0.0)`）降级成**普通隐藏函数**——与文本叶同一机制，共享 verify 检查、领域行引用 def id。不另建求值器、不把手写数值当常量。实测效果：g01 的三条假 `no record type named Material2D` **消失**。

**一次刻意的回退（记录在案）**：我先是把无法按形状定类型的属性值（裸符号 `kind: circle`、限定引用 `material: resources.red_alpha`、服务调用）也塞进声明返回 `i32` 的函数。随即撤回——**声明返回 `i32` 而 body 是 `Material2D` 引用会让共享 typechecker 误拒合法源**，那是静默降级的另一种形态。这类值是**名字存在性/可见性**问题，不是值类型问题，现在得到具名拒绝 `source.age.scene-value`（报出属性名），不猜。

**g01 现状**：闭包已越过资源声明、`let` 推断、node 导入、场景投影与字面量属性，停在 `kind: circle`——即上述符号型属性。**下一步的设计点明确**：符号型场景属性需要走**存在性/可见性检查**（复用 binder 的导入/常量事实），而不是值函数。

**验证**：`plan981_` 29/29；`ace-core` 547/547；零警告；探针全删；两棵树干净。

2026-11-16 stage: work（goal 轮 11，路线 A 第 3 步：端口注解 + 场景位置定位）| PLAN-981 r13 | outcome: 进行中 | commit `b8aad9292`

**本轮修**：端口类型登记漏了**注解位**。`shapes.at` 声明 `node Disc(…, tint: Material2D)` 却**没有任何 `material2d.*` 调用**，而登记判据只看"是否使用服务"，于是该模块没有端口记录，解析它自己的参数类型就报 `no record type named Material2D`。现按注解位（函数参数/结果/记录字段）一并登记（`ann_mentions_age_port`）。实测：`shapes.at` 单独编译**通过**。

**本轮排除的假设（用测量而非推理）**：我一度以为 `view-2d` 的成员被 `age-node-ref` 吃掉造成语法歧义。实测推翻了它——最小场景在 CST 层得到 `scenes=1 refs=0 attrs=1`，**场景投影是正确的**。同时确认三个最小形状（单实例、typed 实例、普通 view 里的 node 引用）**全部通过**。

**因此剩余失败的真实性质**：不是投影 bug，而是**发射/消费**缺一层——场景属性表达式（如 `material: resources.red_alpha`）只被带在领域行上，**没有被降级成受检程序项**，所以解析时找不到符号。下一步要把场景属性降级成可检查的求值单元（与文本叶/隐藏函数同一条链），而不是另建求值器。

**验证**：`plan981_` 29/29；`ace-core` 547/547；零警告；探针全删；两棵树干净。

2026-11-16 stage: work（goal 轮 10，路线 A 第 3 步：node 导入 + 场景出口）| PLAN-981 r13 | outcome: 进行中 | commit `edd13cc45`

**本轮打通两层，并修掉一处静默误投影**：

1. **AGE node 导入**。`use shapes: Disc` 里的 `Disc` 是 **node**，而包 binder 只认 fn/type/const/store，故报 `unknown declaration app::shapes.Disc`。新增 `Target::Node`（独立 node 符号命名空间），且**先于**通用 `exported` 闸解析——因为 **AGE 作者模型从不写 `pub`**：模块是声明的集合，**符号表导入本身**就是显式的跨模块意愿。已用原源核实：g01 `shapes.at` 是 `node Disc`（无 `pub`），`main.at` 是 `use shapes: Disc`，且**全部 AGE 示例无一处用 `pub`**。`pub node` 仍合法，但不是本面的门。
2. **场景出口**。`view-2d`/`view-3d` 的成员是**组件实例 + 属性名录**（`Disc red_disc { x: -20.0 … }`），不是文本叶/节点引用。它们此前落到普通 view 分支再进 `Item::Type` 兜底，把 `Disc` 变成一条假记录、其字段注解成了 `Material2D`——表现为三条 "no record type named Material2D"。现在显式识别并投影为 `ComponentSceneInstance` 行（kind + name + 属性表达式），且组件投影器带上作者面标志，使属性里的 `-20.0` 合法。
3. 删掉冗余 `is_node` 字段（node 性由 `Target::Node` 表达，留一个只写不读的字段等于说谎）。

**已知歧义（如实列出，未硬凑）**：`Meter m1 {}`（节点引用）与 `Disc red_disc { attrs }`（场景实例）共享 `ident ident lbrace` 前缀。当前 grammar 在 `view` 内把两者都路由到 `age-node-ref`，故 g01 的场景属性仍被当作节点引用体解析。解决它需要一个明确的语法决定（空花括号 vs 属性列表的区分，或不同的 view 上下文），是下一步。

**验证**：`plan981_` **29/29**；`ace-core` **547/547**；零编译警告；两棵树干净；临时探针已删。

2026-11-16 stage: work（goal 轮 9，路线 A 第 2 步完成 + 第 3 步起步）| PLAN-981 r13 | outcome: 进行中 | commits `87d27e733`（L）、`2e0849af8`（资源服务用例）

**第 2 步（推断与绑定）已完成**：资源服务调用**端到端编译通过**——`let a = material2d.color(0.9,0.3,0.2,0.5)`、`let m = mesh2d.quad(30.0,30.0)`、`let c = mesh2d.circle(24.0)` 同模块共存，模块级 `let` 从服务返回端口推断出类型。

修掉的**真实缺陷**（每条都是"静默降级/崩溃"级，非补丁）：
1. **端口按 profile 登记永远不成立**：包前门用 `TextRecord` 做领域解析（面在 profile 之上），故 `surface == AgePackage` 判据从不触发，端口静默回落 `VT::I32`。改为**按是否使用服务**登记；回落路径改成具名 `source.age.resource-port`，不再静默。
2. **端口记录必须带真实字段**：空记录被 profile 拒绝；字段就是构造器实参（`Material2D{r,g,b,a}`、`Mesh2D{w,h}`）。
3. **重复发射**：resolver 已有端口记录表，手工发射造成 `duplicate type id`——删除重复，保持单一事实源。
4. **intrinsic id 冲突**：`intrinsic_id_suffix` 只取符号末段，`age.material2d.color.v1` 与 `age.mesh2d.quad.v1` 都塌成 `v1`，同模块两处调用即 `duplicate type id t_i_v1`。AGE 符号改用全名（与 console/platform 同纪律）。
5. **端口类型不是模块符号**：binder 的 `type_name`/`is_type_name` 只认本模块声明与导入，端口按内建类型词放行。

**`let` 推断的边界（如实）**：只推断"初值是 AGE 资源服务调用"这一种形状（取该 op 的 `result_port`，与 resolver 分类同一判据）；其它初值形状**具名拒绝** `source.module-let-inference`。这里没有通用推断引擎，猜成 i32 会是合同禁止的静默降级。

**永久用例**：`plan981_age_resources` 6 case（三构造器 + let 推断、端口**名义**不互换、参数类型错、参数个数错、未知 op 不臆造、非服务初值具名拒绝），全绿。`plan981_` **29/29**；`ace-core` **547/547**；`ace-syntax-grammar` 45/45；零警告。

**g01 现状与下一层**：真实闭包已越过 `let`，停在 `source.module-symbol | unknown declaration app::shapes.Disc`——`use shapes: Disc` 里的 `Disc` 是 **node**，而包 binder 的 fn/type/const 表不含 node。这是下一层：**让 binder 认识 node 导出**（与 `pub node` 进 exports 同源，但符号表导入路径需单独打通）。

2026-11-16 stage: work（goal 轮 7-8，路线 A 第 1-2 步）| PLAN-981 r13 | outcome: 进行中 | commits `0d578fcd7`（J：资源声明 + 两处静默丢弃）、`3f4ed813c`（K：服务调用解析）

**第 1 步（领域声明接入）已完成**：`core::source::age_resources` 按 `platform_io` 既有先例声明 AGE 2D 资源服务——`Material2D`/`Mesh2D` 作为 typed 端口（依既有规范 §1 四类端口），`material2d.color→Material2D`、`mesh2d.quad/circle→Mesh2D`，含能力键。只声明，不做 GPU、不建资源系统。

**第 2 步（推断与绑定）部分完成**：
- 服务调用经新 `RecvClass::AgeService` 解析：接收者是**命名空间**不是值，故不求值（与既有 `Static`/`ListCtor` 同纪律），按声明表分类而非猜测。
- **四类错误已实测按预期具名报出**：参数类型（`must be f32, got i32`）、参数个数（`takes 4, got 2`）、未知 op（`source.undefined`，不臆造）、以及合法调用的绑定。
- `f32` 类型 case 补进 text-record 描述符（TypeKind + branch）——此前**完全没有**，这是所有 AGE 浮点绑定失败的真因。
- 端口类型在 resolver 登记为名义记录，并按其构造器实参发射字段（`Material2D{r,g,b,a}`、`Mesh2D{w,h}`）；既不是臆造的扁平化重模型，也不是空记录（profile 拒绝空记录）。

**具名未闭缺口（下一轮）**：happy path 仍停在 `verify.type-mismatch | expr e4: call type does not match callee result`。根因是**端口记录的类型身份顺序**：resolver 把端口追加在用户记录之后，而 HIR 类型表按不同顺序排列，导致调用点声明的返回类型与 callee 结果不一致。修它需要决定"内建端口类型在类型身份空间中的位置"——是设计决定，不是补丁。故本轮先提交声明与四个负控，让后续工作从已验基线起步。

**本轮同时修掉的真实缺陷（均可被合法输入触发）**：
1. `decode_typed_literal` 用 `.ok_or("carries no digits")` 拆后缀，导致**所有无后缀字面量**（`0.9`/`12`/`-45.0`）被报"没有数字"。无后缀是常态，已修。
2. `ComponentNode` 无 `params` 字段——语法解析 `node Disc(x: float, …)` 后**静默丢弃**；补字段后仍只收第一个参数（后续参数嵌在 `age-node-param-tail`）。实测 `Disc` 现得到完整四参：`[("x","float"),("y","float"),("diameter","float"),("tint","Material2D")]`。
3. 两处 `field_type(f).expect(...)` 可被包面合法输入触发 **panic**（`model` 字段为非 int/bool 时编译器崩溃）。已改为具名诊断。

**验证**：`plan981_` 23/23；`ace-core` **541/541**；`ace-syntax-grammar` 45/45；零编译警告；两棵树干净。临时探针全部删除（含一次误提交后补删，见 `b9db7511b`）。

2026-11-15 stage: work（goal 轮 8，auto-ui 兄弟树已建；**接线路线实测受阻，已定界**）| PLAN-981 r13 | outcome: 进行中 | auto-ui worktree `D:/autostack/.wt/lang-981/auto-ui`（branch `plan-981-dev`，base `2fc427f6c`）| auto-lang `D:/autostack/.wt/lang-981/auto-lang` @ `b21c7bedc`

**已完成**：按用户裁定建立 981 的 auto-ui 兄弟工作树（同组 `D:/autostack/.wt/lang-981/`，非 junction/symlink），基线与 auto-ui master 一致。

**实测受阻——两侧是不同的 parser 栈，不能"接线"直通**：
1. **auto-ui 侧是旧 parser 栈**：`auto-ui-dialect`（含 `node2d/parse.rs` 的 `view-2d`/`shape2d`/node 参数）依赖 `auto-parser-api`/`auto-ast`/`auto-val`，dev 才依赖 `auto-lang-core`；`autographics-node2d-pilot` 依赖 `auto-ui-compiler`/`auto-ui-dialect`/`auto-lang-core`。
2. **ace 侧是新 CST 栈**：`ace-core` 的依赖只有 `ace-syntax-runtime`/`ace-syntax-grammar`/`ace-foreign-contract` 等，**不依赖** `auto-parser-api`/`auto-ast`/`auto-lang-core`（Cargo.toml 明写"the graph stays acyclic"，方向为 core → syntax-{runtime,grammar}）。
3. **整个 auto-ui 仓没有任何 crate 依赖 `ace-core`/`ace-av`/`ace-lowering`**（全仓 Cargo.toml 扫描为空）。即"auto-ui 既有 g01 投影"跑在**旧链**上，与 979/981 的 ACE 新链之间**没有也已不允许有直接数据通道**（`auto-ui-dialect` 的闭包被 777 `module-boundaries.toml` 钉住：禁依赖 auto-lang/复制 Parser）。
4. **979 领域入口只吃源串**：`compile_domain(pkg, src: &str, …)`；唯一的 AST 级入口 `compile_program_parts(program: &ast::Program, …)` 收的是 **ace 自己的** `source::ast::Program`，不是 auto-ui 的 AST。故不存在"把 auto-ui 的 NodeDecl 直接喂给 979 前门"的接面。

**这意味着什么（诚实的结论）**：用户选定的"走 auto-ui 已有 dialect 解析器"在**架构上不成立**——不是工作量问题，是两条互斥的语法栈，且既有边界规则明确禁止 auto-ui 反向依赖语言新栈。要让 g01 经 979 前门，只能二选一：
- **(A)** 在 ace 侧为 g01 所需构造新增语法面（即我前几轮误判为"语言缺口"的方向，但它确实是获准路线的唯一可行实现）；
- **(B)** 由用户改派：把 IF-981-ORIGINAL 交回 auto-ui 域（旧链已证），979 的 V-DETAIL-01 改为消费 auto-ui 的既证产物而非经其前门重投影——这是**合同变更**，需用户裁定。

**我不自行选 (B)**：它改动 979 的验证口径。**也不继续 (A)** 而不经确认：那与我上一轮刚被告知的"不要造语法"直接冲突。故本轮**停在此处**等用户裁定。

**本轮未改任何代码**（auto-ui 树为空改动；auto-lang 树 `plan981_` 仍 23/23 绿）。本记录不是 execution_done，不是 979 回验 PASS。

2026-11-15 stage: work（goal 轮 7，IF-981-ORIGINAL **归属已查实并改道**）| PLAN-981 r13 | outcome: 进行中 | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `b21c7bedc`（本轮未改代码，仅核归属）

**用户质疑成立的核实（本节为更正）**：前几轮我把 g01 缺的 `view-2d`/`shape2d`/node 参数当成"语言 grammar 缺口"，方向错误。查实结果：

1. **这些面早已存在于 auto-ui 的 `crates/auto-ui-dialect/src/node2d/`**（不是 980，也不是语言 `auto-source.at`）：`view-2d` 39 处、`view-3d` 30 处、`shape2d` 10 处、`parse_node_decl` 直接支持 `node Name(param: Type) { model? msg? view-2d? on? }`；`load_project`/`load_project_mixed`/`load_project_dual` 为现成载入入口。
2. **g01 的既证证据也已存在且是绿的**：`auto-ui/experimental/autographics-node2d-pilot/tests/node2d_basic_gates.rs::g01_base_scene_matches_original_baseline_data` 于本轮实跑 **ok**（117.6s 含冷编译）。它逐项断言 red(-20,10)/d30/α0.5、blue(0,0)/30×30/α0.5、green(25,-15)/d24/α0.6、yellow(-45,-25)/12×12/α1.0 且 **visible=false**（hidden），以及 `behavior: null`。
3. **夹具与原源是同一份源**：`fixtures/basic2d/g01/main.at` 与 `age-examples/examples/g01-shapes/g01-main/src/main.at` 的**代码体 32 行逐行相同**（`Compare-Object` 无差异），只有注释头不同（PLAN-008→PLAN-019 的措辞）。故"等价夹具"的顾虑不成立——这就是原源。
4. **980 不负责这些面**：980/DIALECT 是传统 AutoUI 声明兼容 + WidgetDecl 字段，其 AC 明确写"纯传统UI工程（**无 view-2d/view-3d 声明**）"。把 g01 归 980 会与其自身分母冲突，故**不改 980**。

**因此 IF-981-ORIGINAL 的性质变了**：它不是"语言侧要新建五片语法面"，而是"**把 auto-ui 既有的 g01 领域投影经 979 共享前门接进 SOURCE**"——与 981 §5 已声明的改面（`autographics-node2d-pilot/src/load.rs`）完全一致。用户已裁定走该路线。

**仍未做（下一轮实际工作）**：在 auto-ui 改面（`autographics-node2d-pilot/load.rs`）建立 g01 → 共享前门 → typed 领域行的接线，使 g01 的四图形/alpha/order/hidden 事实出现在**本包 SOURCE 的受检描述**里（当前绿证据只在 auto-ui 自己的 lower 链里，尚未经 979 前门）。这需要 981 的 auto-ui 兄弟 worktree（同组 `D:/autostack/.wt/lang-981/`），本轮未建。
**未覆盖项如实列出**：`order` 断言与 v-color/v-position/v-visible 变体不在 `node2d_basic_gates.rs` 中（该文件只有 base + 再生幂等）；这些属 v-* 变体测试与原 oracle，交付 g01 交接键时须逐项核对，不得据 base 一行宣布变体已闭。

**本轮未改任何代码**（只读核实 + 记录）；`plan981_` 仍 23/23 绿，探针已全部删除。本记录不是 execution_done，不是 979 回验 PASS。

2026-11-15 stage: review | **review_scope: internal** | reviewer_identity: 实施会话自审（**非独立上下文**，故不能授予 final `reviewed`，也不能据此 merge）| PLAN-981 r13 | outcome: **needs_fix（本批未完成）** | worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | reviewed_commit `b21c7bedc` | base `ca88cde14` | spec_inputs: `docs/specs/ace-migration/{m2-schema,m2-modules,replacement-scope-and-evidence}.md`（未改）| next: 见 F-981-R01/R02 的关闭条件

**结论先写**：SOURCE 批**未完成**，本记录**不是** pass。`status` 由 `drafting` 改为 `executing`（工作已实际进行），但 `B-SOURCE-I/V/R` 三个任务**均保持未勾选**——按本计划自己的复规定，子面（sub-slice）交付不得勾选整任务。`current_step` 仍为 0，因为**没有一个任务真正完成**。

**逐 AC 核对（SOURCE 侧）**：

| AC / 检查键 | 结果 | 依据 |
|---|---|---|
| IF-981-MODULE（两文件 Node 工程、跨模块 helper、两实例 2:1 事件、N Tick、AV 与真实 rustc 各有独立值） | **pass** | `plan981_age_package_modules` 5 case；`plan981_age_package_av` 3 case（6/5、`six`/`other`、改 helper→16/15）；`plan981_age_package_a2r` 2 case（真实 rustc：`value:6`、改 helper→`value:16`） |
| IF-981-REF（同型合法/未声明/不可见引用互斥可辨、原 span、拒绝前无执行、修复恢复） | **pass** | `plan981_age_ref` 8 case，三种诊断码 `ref-undeclared`/`ref-invisible`/`ref-not-a-node` 均已实测分离；跨模块 `pub node` 本轮闭环 |
| IF-981-FIELDS（必需清单、删必需映射必红、合法近邻绿、未知字段不静默丢） | **pass（本批面）** | `plan981_age_fields` 7 case，删投影反控真实变红并报出被删行；与 980 IF-980-FIELDS 的**并集**仍在 980 owner 侧。**覆盖范围如实说明**：`AGE_CONSTRUCTS` 现有 **8 行**（node / model / model.init / on / on.lifecycle / view / view.text / view.node-ref），只覆盖 AGE-L01 的组件形体；**AGE-L03 的五 view 只实现了 3 个**（`view`/`view-2d`/`view-3d`；`view-portrait`/`view-widget` 在 grammar 层尚不存在，实测报 `source.syntax`）；场景出口 `view.scene` 尚未进构造表。这些属 LANGUAGE 批的完整面，本批不宣称已覆盖 |
| IF-981-ORIGINAL（g01 原源闭包 CPU 四图形/隐藏黄 quad） | **pass（本批面，2026-11-16 更新）** | 轮 5 曾记 `fail`：当时首诊断 ``source.unsupported: `let` …``，缺五片语法面。**五片已全部落地**（`view-2d`/`shape2d` 属性名录、node 参数、模块级 `let` 推断、符号表 `use`、场景出口），`plan981_original_g01` **14 case** 实测全绿：三文件闭包、四图形坐标/尺寸、颜色与 alpha 逐值、声明序、`visible:false` 作行事实、资源来源与改值传播、坏 kind/未知资源具名拒、完整原源 hash、恢复 |
| AC-HANDOFF | **partial（2026-11-16）** | 四个 IF 均已交且实跑（`plan981_` 95 case + AV 3 + A2R 2 + grammar 45）。**仍未闭的两条**：① 缺一次**独立上下文**的收敛复审（四次复审均未给出 CLEAN 结论）；② 979 尚未实际回验 V-09/V-10/V-11/V-DETAIL-01。**不得**由本批自证关闭 |
| AC-03（无 legacy fallback） | **pass** | `age_package` 现走共享 `bind_packages`+`emit_resolved_packages_parts`+`verify_text_record`；`plan981_age_package_profile_is_refused_by_generic_entries` |

**Findings（needs_fix）**：
- **F-981-R01（阻断，owner=本计划 SOURCE）**：IF-981-ORIGINAL 未交付。关闭条件：g01 三文件原闭包经本前门产出 typed 描述并对四图形坐标/尺寸/颜色 alpha/order/黄 quad hidden 作独立断言，或由用户在 Plan §10 具名裁定该键改由后续语法计划承接。**这需要用户决定**：它是一片语法面，不是接线。
- **F-981-R02（非阻断，需登记）**：本批新增用例未使用 §6 冻结的四个检查键名（`p981_domain_packages_two_files` 等），而是等价命名的 `plan981_*` 用例。按 §6 允许的"同义映射"处理，但**必须显式登记**映射关系，否则 979 的自动化核对找不到键：MODULE→`plan981_age_package_modules`+`_av`+`_a2r`；REF→`plan981_age_ref`；FIELDS→`plan981_age_fields`；ORIGINAL→**无对应**（未交）。
- **F-981-R03（流程，已修）**：`status` 长期停留在 `drafting` 而工作已进行，本轮已改 `executing`。

**测试成本（按 test-discipline 要求，本会话实测）**：

| 新 case 组 | 覆盖行为 | 实测耗时 | ≤10s 门 |
|---|---|---|---|
| `plan981_age_package`（3） | 裁剪并集/身份/往返 | <0.05s | pass |
| `plan981_age_package_modules`（5） | 两文件工程/跨模块改动/坏叶/模块图失败/恢复 | 0.08s（整组） | pass |
| `plan981_age_ref`（8） | 同模块/跨模块 `pub node`/三类失败/原 span/恢复 | 含于上 0.08s | pass |
| `plan981_age_fields`（6） | 完整源覆盖/删行反控×3/近邻/compat | 0.22s（整组） | pass |
| `plan981_age_package_av`（3） | 两实例 2:1+4 Tick 真值 | 15.66s（整组含链接预热；单 case 均 <1s） | pass |
| `plan981_age_package_a2r`（2） | 真实 rustc 编译+运行 | 1.91s（整组） | pass |

复用证据（未重跑）：轮 5/6 的 `cargo t ace-source ace-syntax` 两组 PASS（506.9s，基线 `b21c7bedc`）；`ace-core` 541/541、`ace-syntax-grammar` 45/45、`ace-av plan979` 26/26、`ace-a2x-worker plan979` 7/7。零编译警告。

**规范增量**：`SD-01/SD-HANDOFF`（`docs/specs/ace-migration/m2-age.md`）**尚未写入**——按计划只在实际交付后沉淀，而本批未完成，故不改 canonical Specs。`supersedes/new/touched` 维持 frontmatter 现值。

**下一步（不由本会话自行认定 pass）**：F-981-R01 需用户裁定（继续投入语法面 vs 改派方案）；F-981-R02 登记映射；之后由**独立上下文**重新评审，才可能转 `reviewed`。

2026-11-15 stage: work（goal 轮 6，跨模块 `pub node` 缺口已闭环；**上一轮的结论被推翻并已修正**）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `b21c7bedc`

**IF-981-REF 跨模块可见性（本轮闭环，实跑）**：`pub node` 现在**能投影、带可见性、且可被别的模块按 node 名引用**。

**上一轮我的判断是错的，此处更正。** 轮 4 我写"顶层 `pub node` 不是 CST 声明边界，投影看不到它"，并据此把它做成显式拒绝。本轮直接测量后推翻：`age-pub-node-item` **自己声明** `head`/`name`/`body` 字段（并不是把 `age-node-c-item` 包在里面），而投影却在找"嵌套的 `age-node-c-item` 子节点"，所以找不到——于是节点静默消失。实测对照：`plain-node` → 1 个组件；`pub-node` → **0 个组件但解析成功**（正是静默丢弃的来源）。修正为"直接投影该壳节点"（字段布局本来就与投影器一致）后，`pub-node` → 1 个组件。

- `pub node` 进本模块**导出面**（沿用 960 的 `exports` 事实），别的模块可按 node 名引用；`AgePackageComponent.public` 现在对 `pub` 声明是真的。
- 轮 4 那个"显式拒绝 + 钉死不得静默丢弃"的用例**已删除**——它只是用来围住 bug，不是规则。真正的规则现在是"能引用"。
- 用例：跨模块 `pub node` 引用解析到声明模块；非 `pub` 同名声明报 `source.age.ref-invisible`；`pub` 声明本身留下可见性事实。

**方法论教训（记入本计划）**：上一轮我把"没测出来的假设"当成结论写进了复审记录，并据此给项目留了一个假的具名缺口。本轮用最小探针（`plain-node` vs `pub-node` vs `pub-fn` 对照）**先测量再下结论**，5 分钟内就定位到字段布局不匹配。今后同类"看不出为什么丢"的缺口，先做对照探针，不得以推理代替测量。

**验证（本轮实跑）**：`plan981_` **23/23**；`cargo test -p ace-core` **541/541**（较上轮 +7）；`ace-syntax-grammar` 45/45；零编译警告。固定组 `cargo t ace-source ace-syntax` **两组 PASS**（506.9s）。探针用完即删。

**仍未交付**：IF-981-ORIGINAL（轮 5 已实测定界：g01 需要 `view-2d`、`shape2d` 属性名录、node 参数、模块级 `let`、符号表 `use` 五片语法面）。故 979 的 V-DETAIL-01 仍未闭；V-09/V-10/V-11 的 SOURCE 侧输入已具备（模块图×领域、同模块+跨模块引用可见性、构造表）。本记录不是 execution_done、不是 979 回验 PASS，未改 canonical Specs。

2026-11-15 stage: work（goal 轮 5，IF-981-FIELDS 已落；IF-981-ORIGINAL 已实测定界）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `fb2d59447`

**IF-981-FIELDS（已落，实跑）**：AGE 作者面现在有**版本化构造清单**（key / 作者拼写 / 必须留下的 typed 事实 / 下游 consumer / 规范条款）＋一个对着**实际产物**核对的覆盖判据：
- `AGE_CONSTRUCTS` 覆盖 `node`、`model`、`model.init`、`on`、`on.lifecycle`、`view`、`view.text`、`view.node-ref` 共 8 行，每行具名 consumer 与 AGE-L 条款。
- **兼容变体单列**：旧 `widget` 头不是 canonical 源必须覆盖的构造，放进 `AGE_COMPAT_CONSTRUCTS` 单独验（compat 开/关两态）。把它塞进主表会让一个真正全覆盖的 canonical 源看起来"缺一行"——这是本轮实测发现并修正的判据错误。
- **删行反控是真的删投影**：`collect_facts`/`age_construct_coverage` 读产物事实，所以清空 handler/字段/引用会让检查变红并**报出被删的那一行**；改清单只会缩小分母，证明不了覆盖。逐行反控覆盖 `model`/`view.text`/`view.node-ref`。
- `model.init` 只有在**每个字段都真有实例状态函数**时才计已覆盖，符号经绑定器 `origins` 表查（不猜命名）。
6 个新 case 全绿（含完整源覆盖、删 `on`、逐行反控、合法近邻子集、表结构自检、compat 双向）。`plan981_` 22/22；零警告。

**IF-981-ORIGINAL（本轮实测定界，**未**交付）**：用**真实** g01 三文件闭包（`main.at` 原源 + `resources.at` + `shapes.at`）实跑包前门，**首个**诊断即 `source.unsupported: \`let\` is not part of this source profile @ 0..3`。据此判定本批**不具备**交付该键的条件——g01 需要的是一整片语法面，不是接线：
1. `view-2d` 呈现面（本批只有 `view`）；
2. `shape2d`/`Disc` 之类的**组件实例 + 属性名录**（`x:`/`y:`/`diameter:`/`tint:`/`kind:`/`position:`/`size:`/`material:`/`visible:`，含 f32、tuple、`visible:false`）；
3. node **参数列表**（`node Disc(x: float, …)`）；
4. 模块级 `let` 资源绑定与 `material2d.color(...)` 调用；
5. 符号表式 `use resources: a, b, c`（冒号形式，不是 960 的点分导入）。
这五项各自都不是"补一条规则"，而是要新建呈现/属性/参数/资源语义——属 LANGUAGE/呈现批与本包后续有界决定，**不是**本轮能硬凑的接线。因此本键**保持未交付**，不用等价夹具冒充原源 CPU 证明（979 V-DETAIL-01 明确要求"原源闭包"，替身无意义）。探针用完即删。

**仍未交付（解锁 979 的全部前置）**：IF-981-ORIGINAL（上述语法面）、跨模块 `pub node` 可见性。故 979 的 V-09（模块图部分已可回验）、V-10（同模块引用部分已可回验、跨模块可见性未闭）、V-11（构造表已交，并集待与 980 合）、V-DETAIL-01（未交）**尚未整体解锁**；本记录不是 execution_done、不是 979 回验 PASS，未改 canonical Specs。

2026-11-15 stage: work（goal 轮 4，IF-981-REF 已落「已证半边」）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `30ee661dd`

**IF-981-REF（已证半边，实跑）**：view 成员现在可以是**节点实例引用**（`Meter m1 {}`），且 979 V-10 要求的三类失败**互斥可辨**（不是"偶然类型错"）：
- `source.age.ref-undeclared` —— 名字在任何模块都不存在；
- `source.age.ref-invisible` —— 声明在别的模块且不可见；
- `source.age.ref-not-a-node` —— 解析到已声明的**普通类型**而非 AGE node。
合法近邻＝同模块引用（**不需要导入**：节点身份自成命名空间，不借用 960 的 `use` 值/类型命名空间），解析结果带声明模块键；诊断带**原模块 span**；同入口失败后恢复。6 个新 case 全绿。

**具名缺口（本批**不**声称已交）**：跨模块 `pub node` 可见性**未实现**。实测根因：顶层 `pub node` 不是 CST 声明边界（`tree.decls` 由顶层规则调用产生，而该壳未登记为目标），投影根本看不到它——我的第一版实现因此把 `pub node` 源**编译成一个静默变小的程序**（节点消失、无诊断），这正是本合同禁止的静默丢弃。取舍：**宁可显式拒绝，也不静默丢**——保留语法形状但在投影处具名拒绝 `source.age.pub-node-unsupported`，并加用例 `p981_ref_pub_node_is_never_silently_dropped` 钉死"不得退化成静默丢弃"。真正交付它需要在 grammar 引擎层新增声明边界，属后续批次的有界决定，不在本轮硬凑。

**顺带修掉的真实缺陷**：`project_program` 的 `Item::Type` 兜底会把**未知项**当成同名记录类型——这正是 node 声明可能"无声消失"的通道；AGE node 项现在在该处显式跳过（`continue`），不再被兜底变形。

**本轮过程记录（如实）**：这是一次反复的调试——期间我一度把 `pub` 令牌整体判为语法错，误伤了夹具里合法的 `pub type`；也一度让 helper 脚本重复插入导致 4 处 `unreachable pattern` 警告。两者都已当场修正（重复臂已去重，最终零警告）。这些弯路说明 `pub node` 需要专门的语义决定，而不是补丁。

**验证（本轮实跑）**：`plan981_` 全套 **16/16**；`cargo test -p ace-core` **534/534**（较上轮 +6）；`ace-syntax-grammar` 45/45；`ace-av plan979` 26/26。固定组 `cargo t ace-source ace-syntax` **两组 PASS**（624.6s）。零编译警告。

**仍未做**：IF-981-FIELDS（完整 AGE 构造映射表 + 删行反控）、IF-981-ORIGINAL（g01 原源四图形 + 隐藏黄 quad 的 CPU 投影），以及跨模块 `pub node` 可见性。故 **979 的 V-09/V-10/V-11/V-DETAIL-01 仍未解锁**；本记录不是 execution_done、不是 979 回验 PASS，未改 canonical Specs。

2026-11-15 stage: work（goal 轮 3，B-SOURCE-I 阶段 C+D 已落）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `b0d13b488`（C）/`d12934690`（D）；阶段 A `819c5bc35`、B `bd421dccf`

**阶段 C（AV 真值腿）**：`core::source::age_package` 增 `compile_age_package_with_scenario`（消费者场景＝实例数＋定向事件，**复用** 979 的 `ComponentScenario`/`instance_stmts`/`rename_expr`/`state_fn_name`，不复制第二套）；新增 `av::age_package::observe_age_package` 走共享 `lower→mir_verify→abc 编解码→AV 执行`取真值。实测两实例 2:1 事件＋各 4 次 `Life.Tick` → 状态 **6/5**、文本 **`six`/`other`**（实例 0 越过 6、实例 1 未越），与独立算术 oracle 一致；改被导入 helper 初值为 10 → **16/15** 且两实例文本都变 `other`（证明确实来自作者行为，不是常量）。`ScenarioOp::Tick` 在 979 是"保留消息 `Tick`，运行身份归 981"——本批把它解析成 node **实际声明**的拼写（`Life.Tick`），未声明时才由共享检查具名拒绝。静态面（`instances: 0`）仍生成实例 0 状态：**"静态"＝不额外复制实例，不等于 model 字段可以无绑定漂浮**（这是我改坏又当场修正的一处真实回归，见下）。

**阶段 D（A2R 真 rustc 腿）**：同一包前门产出的同一 `CheckedModule` → `SemanticView::from_checked` → 新 A2R provider → **真实 `rustc`** → 运行 → 读到 `value:6`；改 helper 初值后同一链读到 `value:16`。入口 `fn main` 调用的是**同一次**受检程序里的脱糖实例状态函数，无手写 Rust。两个实例状态函数先经编译结果的 `source-name → HIR 符号` 表解析，缺失即具名失败，不会静默降级成更小的程序。**如实登记的缺口**：含模块级 `const`/`var` 的包图仍在 `from_checked` 被 `module.global.*`（`a2x.unsupported-expr`）拒绝——属 RUST 批，本用例输入不含全局并已注明，未伪称已支持。

**本轮现场处置（两件必须记录的事）**：
1. **主检出被误改并已还原**：早期用 `.NET` 写文件时按进程 cwd 解析相对路径，把阶段 A 的 5 行编辑写进了 `D:/autostack/auto-lang`（并留下一个 0 字节 `age_package.rs`）。已先备份到 `plan-artifacts/plan-981/main-checkout-wip/`，核对备份即我方内容后 `git checkout --` 还原，并删除该 0 字节残留（SHA256 为空串哈希，确认无内容）。当前主检出 `experimental/ace` 下 `git status` 为空；后续所有写入改用绝对路径/`edit` 工具。
2. **固定组首轮全红是磁盘而非代码**：`cargo t` 四组首跑报 `error: less than 2 GiB free before Cargo workload`（D 盘仅剩 1.0 GB）。这**不是测试失败**。先按"只回收自己树内的构建加速器"口径删除本 worktree 的两处 `target/debug/incremental`（7.1 GB，纯增量缓存，非产物），D 盘回到 17.4 GB；随后四组复跑全 PASS。未删除任何其他会话/计划的缓存或产物。

**验证（本轮实跑）**：`plan981_age_package_av` 3/3；`plan981_age_package_a2r` 2/2（两 case 合计 0.85s，含 rustc 编译运行）；`plan981_`（core）10/10；`cargo test -p ace-core` **528/528**；`ace-a2x-worker plan979` 7/7；`ace-av plan979` 26/26；`ace-syntax-grammar` 45/45。固定组 `cargo t ace-source ace-syntax ace-av ace-a2x` **四组 PASS**（457.8s，含冷编译）。新 case 均 ≤10s 门。

**仍未做**：IF-981-REF（实例引用/可见性）、IF-981-FIELDS（完整构造映射表）、IF-981-ORIGINAL（g01 原源 CPU 投影）三个交接键——这三个是 979 V-10/V-11/V-DETAIL-01 的输入，故**本包尚未解锁 979 的 V-09/V-10/V-11/V-DETAIL-01 回验**；本记录不是 execution_done，也不是独立 review PASS，未改 canonical Specs。

2026-11-15 stage: work（goal 轮 2，B-SOURCE-I 阶段 B 已落）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | code_commit `bd421dccf`（阶段 A `819c5bc35`）

**B-SOURCE-I 阶段 B（已实跑）：AGE 包前门 `compile_age_package`**
- 新增 `core::source::age_package`：`多模块源 → 一次解析 → 共享 bind_packages → 唯一 verify → 版本化描述`。领域与普通项**同一次** CST 投影；文本叶与 `model` 字段脱糖成**普通**隐藏/状态函数（复用 979 已证 helper：`instance_stmts`/`rename_expr`/`local_fn`/`state_fn_name`），由共享绑定器 mangle、共享 checker 判定——**无第二求值器/第二绑定器**。
- 绑定器只加面不加重：`Faces::{Plain, Aggregates, Age}` 走同一 `bind_package_syntax_inner`；grammar 身份、可见性、初始化序、mangle 全由 960 原规则裁定。
- 读回：源名 → 真实 HIR 符号经绑定器自己的 `origins` 表查（顺带取 adapter 唯一约定 `sf_<symbol>`），**不猜命名规则**。
- `on Life.Tick`：限定消息尾巴只在新裁剪存在（`age-on-msg-tail` 门 `age-package`），`age-component` 面规则与字节不变。
- **修正阶段 A 的一处真实回归**：阶段 A 曾把 AGE 三面加进 `parser.rs` 的 CST 分派臂，使 `source.domain-frontend` 具名拒绝臂变成不可达（编译器 `unreachable pattern` 警告暴露）——那会让通用入口静默丢弃领域声明。已还原为原拒绝臂并按传入 profile 报错。

**验证（本轮实跑）**：新永久用例 `plan981_age_package_modules`（10 case，全绿，单 case ≤0.05s）：两文件工程经唯一 verify、跨模块 helper 改动改变受检程序、非文本叶被共享 checker 拒、缺模块/私有 helper 在绑定期具名拒、同入口失败后恢复、widget 兼容闸、通用入口拒绝。`cargo test -p ace-core` **528/528 绿**（1 ignored 为既有）；`plan979` 57/57；`ace-syntax-grammar` 45/45；`source_cst_927` 13/13。固定组 `cargo t ace-source ace-syntax` **两组 PASS**（整命令 625s，含增量编译）。零编译警告。

**仍未做（本批后续）**：阶段 C（AV 消费端：两实例 2:1 事件/N Tick 的真实值观察）、阶段 D（A2R 真 rustc 腿 + `module.global.*` 具名缺口处置）、以及 IF-981-REF/FIELDS/ORIGINAL 三个交接键；本记录**不是** execution_done、不是 979 回验 PASS，也未改 canonical Specs。

2026-11-15 stage: work（goal 轮 1，T-01 有界决定 + B-SOURCE-I 阶段 A 已落）| PLAN-981 r13 | outcome: 进行中（**未** execution_done，非 review PASS）| worktree `D:/autostack/.wt/lang-981/auto-lang` | branch `plan-981-dev` | base `ca88cde14` | code_commit `819c5bc35` | 授权：用户本轮直接目标「完成计划981/SOURCE的部分，已解锁计划979的阻碍」= 对 SOURCE 批的实施授权（T-01/B-SOURCE-I 工作，`/auto-plan-goal` 链路）。

**T-01 有界决定（IF-981-MODULE，实现策略已冻结）**：读取实际代码后确定——`packages` lane 已经把整个包图**扁平化成恰好一个 HIR module**（探针实测 `modules = 1`，函数符号已按包/模块 mangle，如 `sf_p3_appm6_helper_seed`），因此"逐模块 CST 投影后接既有包绑定"与"扩展共享包前门"**不是二选一**：既有 960 绑定器已经是共享包前门，缺的只是**让 AGE 声明与包模块项在同一次解析里共存**。故选定：新增**一个**共享 grammar 裁剪 `age-package`（= packages-modules 表面 ∪ AGE 组件表面），由新的 AGE 包前门消费；不新建第二解析器/绑定器/verify。`a2x` 侧 `SemanticView::from_checked` 的 `single_module` 门因此天然满足；**具名限制**：含模块级 `const`/`var` 的包图在 A2R 端会被 `module.global.*` intrinsic 拒绝（`a2x.unsupported-expr`），且 init 链不由 `assemble_record_text_file_set` 的 wrapper 调用——这两项是本批 RUST/后续批的具名缺口，不冒充已支持。

**B-SOURCE-I 阶段 A（已实跑）**：
- 改动面：`experimental/ace/grammars/auto-source.at`（新增 `age-package` profile 声明；98 处既有 gate 列表按"packages-modules 或 age-component ⇒ 追加 age-package"规则扩展）、`core::source::{mod,parser,age_h0,cst/frontdoor,main}`（`SourceProfile::AgePackage` + slug/profile 往返/HIR 域/谓词、grammar 槽位、领域 profile 通用入口拒绝、parser 分派、standalone 二进制具名拒绝）。
- 新永久用例 `experimental/ace/crates/core/tests/cases/plan981_age_package.rs`（3 case）：裁剪并集（包模块面 5 条 + AGE 组件面 5 条规则同面存在）、三 AGE 面身份互异且 H0 仍无 model/on、profile 身份往返。
- **非漂移证据（关键）**：用独立 baseline checkout（`master`=`45ecb376a`，与实施基面 `ca88cde14` 的 grammar 源逐字节相同）跑同一 dump 探针，逐 profile 比 grammar id：**14/14 完全一致**（core `2296b4c5…`、scalar `fecc2ef4…`、text `7ba3cb83…`、record `00158ab0…`、text-record `d820ae7f…`、entry-unit `74b9f29a…`、host-console `69615a6a…`、collections `3e1193e9…`、closures `3ca6a928…`、numeric `40f10820…`、aggregates `1fd2c921…`、scheduler `f26f3786…`、age-h0 `64915bd1…`、age-component `b244ee16…`）；新面 `c7c17d6d…`（86 规则）为新增身份。探针用完即删。
- 命中并处置的**既有红**：`ace-syntax-grammar::profiles_927::profiles_declared_set_is_exact` 在 master 上**本来就是红的**（冻结镜像漏了 PLAN-968 的 `scheduler`，baseline 复现同一失败）。本批只做"补名字"刷新（同 961→941、962→1025 先例），并在注释里具名记录该漂移来源。
- 执行与计时：`cargo test -p ace-core --test conformance plan981_` 3/3 绿 1.4s；`source_cst_927` 13/13 绿 2.4s；`plan979` 57/57 绿 0.3s；`cargo test -p ace-syntax-grammar` 45/45 绿 0.71s；固定组 `cargo t ace-source ace-syntax` 两组 **PASS**（整命令 696.6s，含冷编译；准备/发现/运行/清理分项未单独采集，见下轮补齐）。新 case 均 ≤ 10s 门（单 case 实测 < 0.05s，setup/断言/清理全含）。
- 未做：阶段 B（`compile_age_package` 前门：逐模块投影 → 共享绑定 → 唯一 verify + source map）、阶段 C（AV 消费端）、阶段 D（A2R 真 rustc 腿 + 模块图负控/恢复）；本记录**不是** execution_done、不是 T-05/独立 review PASS，也未改 canonical Specs。

2026-11-15 r13规划静态预检：979/980/981原81任务的ID/顺序/勾选及状态/进度保持，原52项AC、66项V和22项SD均保留，各Plan仅增1项AC-HANDOFF及1项SD-HANDOFF；三份0–10章节/围栏、实际承载任务与新增7个本地链接/锚点核对通过，具名IF交接图静态拓扑无环。960归档文件及.next-id内容hash未变。只证明合同结构/交接边界，不是新接口、CPU/AV/A2R/UI/GPU实跑或独立实现review PASS；未创建任何产品用例/产物或实施树。

2026-11-15 stage: new/revision | PLAN-981 r13 | outcome: 规划合同已修订，非实现/产品review PASS。按本轮授权把原960组合缺口交981/SOURCE，980/981原任务生产具名接口，979保留全部原AC/V与自身组合回验；不以A切片归档，不新增整包反向依赖。原任务/勾选/状态/计数保持，旧revision的真实已验部分可按身份复用，但不能证明本次新增交接；未来独立review须绑定r13及实现SHA。仍只规划，next=由原owner获得后续实施指令后按§5–8执行；本记录不恢复任何目标/实施树或解除既有暂停。

2026-11-12 stage:new / PLAN-981 r11：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原34任务/全部AC与V/依赖/阶段/预算和drafting 0/34；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-981:r10 / outcome:pass（规划计数/责任修订，非实现final）；blocked（实施未授权，所需A接口待交） / next:明确启动后按现有T-01/§5最小接口。原十批34任务、全部AC/V及A/B分配保留、0完成；修正§5、V-B-AUDIT、AC-B-AUDIT、B-AUDIT-I的旧8/9批残留，显式含PROOF。不重开原交付；新G2/G3/联合合同只消费本包，不新增反向依赖/范围。历史r7–r9数字保留为当日记录。


2026-11-10 stage=new / r9：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部34个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r8：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE与A期PROOF I/V/R，当前0/34；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r7 / stage:new：按用户明确完整传统UI与AGE语法要求，增LANGUAGE I/V/R、AC-B-LANGUAGE及V-B-LANGUAGE/V-L-*、SD-05；保原28任务/14AC/全部原V，现9批31任务、15AC，drafting、0/31。980独立验传统UI、不反向等981，979只通用平台；LANGUAGE/融合实际验五view/条件/Frame和新旧UI，不以三view/游戏替代。仅规划，实施handoff仍blocked（未授权实施/接面及待决策略未齐），非运行或独立final PASS。

本轮静态检查：31任务/15AC/20验证行，原ID全保留，无重复、0完成；五份相关合同可达199份跨仓Plan，无缺失引用/环，LANGUAGE-I→后端/980就绪→LANGUAGE-V/R→FUSION最终V/R→AUDIT的图无环。修正旧汇总文字的潜在自等：AUDIT等前8批R，本批R后才9批齐交T-05。新链接/Cargo组名已核，SD-05为尚不存在的拟新增Spec；未运行完整语言或任何产品测试。

2026-11-09 r6 / stage:new：合并AGE M2/M3，8批/28任务/14AC全保留，0/28、drafting；本包承E/F、UI058/AG023及待冻结3D承G2/G3，公共服务可并行、最终增强作者接线消费本包。FUSION用既有M1基础，不反向等二维/三维新增切片，消除整阶段循环。没有重复立项、实施或运行；实施handoff仍blocked（无实施指令且实际接口未齐），非产品/独立final PASS。

2026-11-09 r5：stage=new；按用户方向补SOURCE共源、SESSION的AV开发、RUST公共编译发行及RUST2D/RUST3D逐游戏整链。原19任务/原AC全保留，新增9任务与AC-B-RUST/RUST2D/RUST3D、V-B-RUST*及V-R2-*/V-R3-*/V-R-PACK/FAIL，SD-03/04明确跨仓沉淀。frontmatter补962/969/972及已交AG028/UI072接口，按最小批次解耦；979不依赖981、SESSION/RUST可并行、两个游戏独立、FUSION不阻前述链、AUDIT后交985/986/987。已完成M1交付只复用，未重复立项。0/28、drafting。outcome=blocked（实施交接仍缺实施指令及正式接口/工具/平台收据）；next=获实施授权后T-01按§5就绪批次核对。此次仅文档静态复核，非产品/独立final review PASS，未运行构建或试玩。

规划静态核验：两计划可达192份跨仓Plan，引用全部解析、依赖环0；批次DAG无环。979为8任务/7验收/8验证行，981为28任务/14验收/14验证行，任务/AC/验证ID无重复、均未执行；固定Cargo组名存在。已交960/961、UI060/067/072、AG014/016/024/025/028只作正式来源复用，不重开或记成本包已完成。SOURCE→SESSION与RUST分支、按游戏AV对照→各Rust最终验收、FUSION及AUDIT汇合已核；未把后继985/986/987反置为本包前置。

2026-11-07 r4：细化5批，0/19。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

2026-11-07 AGE五计划复核联动：r3，只纠正新旧阶段映射、消除整阶段循环等待并明确UI067/959与981跨仓接收责任；原AC/V/任务及行为分母保留，drafting 0/8。stage=new；规划静态修订，未实施、未认领产品review PASS。


2026-11-11 / planning_validation: pass（仅文档静态预检）。本轮8份新建/修订合同§0–10唯一、119任务/frontmatter/当前AC到任务覆盖一致、全部drafting/0完成；原任务/AC/V保留（UI058仅新增AC-11/V-S08），4份新合同各10任务/9AC/6验证。当前四仓可达221个Plan节点，无缺失引用/依赖环；UI039原活动影子/归档双文件按既有同号身份处理，不新计交付。十九份触达规划/导航的393个本地文件链接可解析，16处旧归档链接已纠正，定向diff空白检查通过；研究Spec、r187的621原表和pipeline研究输入hash保持不变。批次级检查确认979/980/981不反等新增增强/AG031，985/A不反等G2/G3或AG031，984只交OS覆盖切片而不要求AGE等全生态。未构建/运行用例、制作素材、创建实施树/新证书/新canonical、提交/push或打tag；此记录不代实现独立final。

2026-11-13 / stage:new / PLAN-981 r12 / outcome:pass（规划合同预检，非实现或产品review） / changed: Node首源、Schema/NodeName/兼容与并行合同及对应AC/V/SD；原任务数量/状态/进度与旧义务保留 / next: 本轮仍只规划；后续获得实施指令且该任务最小正式来源齐时进入原work。全部新增验证未执行，未构建/运行/建树/派agent/合入或改963。

2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。

2026-11-16 stage:new/revision | PLAN-981 r15 | outcome:pass（规划合同/交接可执行，非产品PASS） | changed:SESSION/LANGUAGE/PROOF、FUSION/AUDIT接洽；AC-NODE-DESKTOP、V-DESKTOP-A/AC/AUDIT、SD-NODE-DESKTOP | preserved:原34任务ID/勾选及实际状态计数/历史收据 | next:依980正式最小接口推进A；M2A-08后1068 AC切片与本包B协作。用户当前仅授权规划，无实施/测试/建树/合入。

本轮规划静态预检：9份计划/设计路线文档；四份Plan共87个原任务ID/总分母与依赖列表保持，仅980的NATIVE I/V/R因新增合同重开（15/29→12/29），历史收据未删。新增本地链接/锚点、0–10章节、验收/用例/原任务映射、表格与行尾检查通过；A入口→A出口→AC切片→B产品/终验的阶段图无环。候选固定组名按scripts/test-groups.toml只读核实，新UI/AC入口或case明确由work落实，均未运行。本记录是文档预检，不是产品或独立实现复审PASS；未改canonical Specs/源码/测试资产/取号计数器，未实施/建树/构建/运行/合入/清理/恢复目标。

2026-11-16 stage:new/revision | PLAN-981 r16 | outcome:pass（规划可执行合同，非实现/认证PASS） | 34原任务ID/勾选不变；SOURCE原3勾选已正式交，修正旧元数据0→3，无新增实施；A最小、B三波和完整游戏/融合各有原任务承接 | next:由原owner依新阶段/真实来源接续；本轮只规划，不实施/建树/测试/合入/清理/恢复目标。

### r13开工分档历史（本轮阶段已取代，不作当前任务前置）

**A 期开工分档（2026-11-16 簿记，不改合同/任务文本/AC/分母/阶段门）**：A 期不是"整包等 979/980"，
按**接口就绪度**分三档推进；依据是 §5 r12/r13 的"消费已交接口、不等整包"与 B-LANGUAGE-V
"980 正式来源齐后"、AC-B-FUSION"须 980 完整结果"。

1. **立即可做（不等任何整包）**：B-SOURCE 收口（979 在落主 SHA 上回验 V-09④/V-10/V-11/
   V-DETAIL-01 → 关 AC-HANDOFF）；**B-SESSION-I/V**（AV+typed 宿主，接口来自 979/AV）；
   **B-RUST-I 公共链**（新 A2R/provider + 978 构建；完整职责待语言 owner 的实例状态/初始化/
   调用目标合法化决定）；**B-LANGUAGE-I**（继承 SOURCE 的 IF-981-REF/FIELDS，按 AGE-L01..05 与
   AG-A01..09 扩语法/AST/投影分母——只需 980 已交组件/宿主接口与 Schema，不需其完成体）。
2. **等 980 A 面正式交付（含其 976 前置）后**：**B-LANGUAGE-V**（"所需 SESSION/RUST 及 980 正式
   来源齐后"才实跑 V-B-LANGUAGE 与 V-L-*，必需项未决不得声称完整支持）；B-RUST-V 的完整职责并集；
   **B-FUSION**（B 期，AC-B-FUSION 须 980 完整结果 + LANGUAGE 组合证据）。
3. **等第 2 档结果**：**B-PROOF-I/V**（依赖 SESSION/RUST/LANGUAGE 三批结果与 renderer，
   两局部实例事件 2:1、Tick/Frame 不随 view 倍增、真实 GPU/输入/ROI）。

**不做的事**：不以"979 仍 executing"拒绝 SOURCE 收口；不以"980 未整包"推迟 LANGUAGE-I；
也不拿 LANGUAGE-I 的最小语法面冒充 LANGUAGE-V 的完整支持。


本轮规划静态预检：14份计划/设计/路线文档，8份Plan共230个当前任务；仅980新增B-AV-I/V/R三项并按完整W3范围重开原RUST三项，旧AXR0收据保留；981仅核正原SOURCE三项已交的滞后计数。全部原任务ID/其余勾选、各Plan状态与整包依赖列表保留；新增本地链接/锚点15处、章节/表格/元数据计数/验收任务映射及行尾检查通过，A最小→A出口→W1→W2独立交接→W3→OS→候选/安装/退役阶段图16节点18边无环。原GUI/Widgets/Notes-Clock/语言/库/产品义务有具名阶段与原owner，不以延期冒PASS。仅文档预检，未实施/建树/运行测试或窗口/签阶段/合入/清理/恢复目标，未写源码、测试资产、canonical Specs或取号计数器；他会话并发修改不由本轮回退。

2026-11-16 stage:new/revision | PLAN-981 r17 | outcome:pass（规划合同交接，非代码/产品/独立A验收PASS） | EVENT-SPLIT三缺口→原LANGUAGE/SESSION/AV/WEB/认证任务，A/B交付和逐批恢复边界明确；复用#6i与正式MIN-A来源，保全部原任务ID/勾选/状态/完成数/依赖/验收。新增AC-EVENT-PHASE与V-EVT正反控制均未实施。next:未来原owner按§5/§8在各自已授权范围执行，当前只规划，不建树/改代码/测试资产/运行/派agent/提交合入/清理/恢复目标。

本轮文档静态预检（非产品PASS）：8份规划/路线/债务文档；980/981/985原80个任务ID、勾选、32/34/14总数与12/3/4完成、状态及依赖并集保留，原AC/V/SD标识未删除。新增表格/引用/任务和阶段映射、Markdown围栏（含985原四反引号嵌套例）与空白通过；最终阶段图32节点37边无环。12份读取的代码/用例/规范内容hash未变，980并发round41收据完整保留；所报两红只完成归属规划、未修复/复测。新增V-EVT均未创建/运行，实际矩阵/SDK/目标状态及canonical Specs未改。next为原owner后续按现合同执行；当前只规划，不签代码/独立产品review或A出口。

## 10. 待澄清事项

**985/A接收反馈（2026-11-17，985 r26，需本包继续补充）**：review-68批级收据（event A＋B-LANGUAGE-V[A]/B-SESSION-V[A]/B-RUST-V[A]证据面闭，绑定版本已核落主）985已接收；但对照985 §5 WAVES的MIN-A-AGE合同（"SOURCE、SESSION/LANGUAGE/PROOF的A部分"），**PROOF批A面未交付**（B-PROOF-I/V/R未实施未勾、批级结论未提及）。待补五项（详985计划§5"r26增量"反馈块）：①B-PROOF-I/V/R[A]——g01/Showcase最小真实UI＋2D＋3D呈现＋一次事件/更新/关闭，只AV，错误编辑保旧/修复恢复，独立CPU值/可见ROI；②PROOF批fold后与event A批联合出**MIN-A-AGE整体正式交接收据**（985合同：MIN-A-AGE全required及IF-981-EVENT-A独立R/fold后交985/A）；③簿记同步（master版仍3/34、T-01未勾、已交A子面任务勾选未更新）；④具名债务A/B归属确认（D-981-EVENT-A-REMAINDER列A-required子集；D-981-EXIT-CONTRACT核979旧oracle失效；R59-5、R64-1..10现值）；⑤review-69收口（R69-1修补已落主，§9补pass行）。⑥保险条款（985 r26 L-45裁定）：PROOF批实际源（g01/Showcase）若出现NavCall/导航用法，按EVENT-SPLIT提升规则在本包LANGUAGE/PROOF A任务内具名提升并补独立正反例——不新增980任务、不把NavCall语法提前引入共同grammar（避免再次扰动#6i冻结身份）。985待交清单现为：本包PROOF批/整体交接＋1066/A两件。

本次已明确缺口生产归属及无环交接，不再等待已归档960产生新交付。原I任务仍须冻结具体组合profile/API、诊断码与可见性语法/命令及资源边界；这是有界技术决定，不是新增立项或降低原验收。原已具名的公共语言/平台/产品策略未决继续保留，必要能力缺失必须绑定实际owner，不能以编号泛称取代可执行前置。

历史039活动/归档同名歧义按归档来源与实际接口核验，不能列作未完成游戏工作或自动重开；T-01重核当前副本，但不阻本次规划、不清理他仓文件。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结十批及并行前置（AUDIT核其余九批，含PROOF），T-01只补符号/命令/平台与资源决定，不从零拆包或新增重复计划。SOURCE/RUST管既有源的共同投影/表示；LANGUAGE-I须关闭完整语法分母、Frame主时钟/零尺寸/不可见出口、挂载切换/编译期检查的具名策略，不得把未决点删出“完整支持”。普通语言/provider缺口归962/966/967/972/978等实际owner或本包合法化，不能凭960/961已归档略过；源码错误、目标缺能力、资源拒绝、编译/环境故障分列。
本包游戏发行保持Windows x64两候选；新增2D/3D由合并M2的原图形合同/待冻结合同规划承接，不移入本包重做。其他平台、新关卡/美术/商店发布不因合期获实施授权。候选生命周期/去向由T-01登记并由T-06/07按保留政策对账；既有AG028试玩包不属本包清理范围，不覆盖删除。

2026-11-16 **phase fold 阻塞（实测，未合入、树已回退干净）**：979 已在 `f94a29293` 上完成四行回验
（AC-HANDOFF 待落主），但直接 fold 不可行——① `plan-981-dev` 仍基于 `ca88cde14`，master 已前进
**43 个 ACE 提交**（974/980/977/978/972/970/1043 等，含 grammar 698 行改写），试合并 `git merge master`
报 **10 个文件冲突**；② 主检出 `experimental/ace/crates/av/tests/conformance.rs` 有**他会话未提交改动**，
而本分支也改该文件，fold 前必须由 owner 落定（不得覆盖）。

**resync 尝试（2026-11-16，按方案 1，未提交、已 abort 回 `f94a29293`）**：9/10 文件可机械合并，
决策如下（下次 resync 直接复用）：`core/tests/conformance.rs` 两侧 mod 并集；`profiles_927.rs` 取
master 的 scheduler 注释；`cst/mod.rs` 并集 `parse_packages_front_age` + `parse_packages_front_aggregates_ct`；
`packages.rs` 用 master 的 re-export 列表并补回 `bind_packages_age`/`PackagedAgeComponent`；
`age_component.rs` = `exit`+`node_refs`+`scene_components`+`tree` 并存，并保留 980 的
`ComponentViewItem/Element/Entry/MsgVariant/DesignEntry/MemberRow`；`frontdoor.rs` 两份 grammar 缓存槽、
`PackagesFace::{Age, AggregatesCt}` 与两条 match 臂都保留；`project.rs` 两套声明与两套 view 字段并存；
`binding.rs` 用 master 的 `aggregates, asm_lane: in_face && aggregates`；`text-record.atom` 为
`ExprKind`（含 `text_display`,`fstr`）+ `TypeKind`（含 `f32`）并集。

**剩余关键：grammar 的 14 处**不能靠并集——980 的 B-DIALECT 把 view 重写成富树
（`age-view-item` 的 entry/text-props/text-bare/element + msg/design/bind/… 成员槽位），而 981 的
SOURCE view 是 text/node-ref（外加 view-2d/3d 直属出口）。合并设计应为：**age-package 采纳 980 的
富 view 作为超集，再加 `node-ref` alt 与 view-2d/3d 出口**；同时 980 的那族规则 gate 列表要补
`age-package`。这是两个 Plan 的**面格并集决定**，必须同时以 980 的 `plan980_dialect_full`/
`plan980_field_manifest` 与 981 的 `plan981_*`、grammar 45/45、固定组（`ace-source ace-syntax`）验证，
不能静默取一侧。另注意 master 的 profile 列表里存在重复的 `packages-aggregates-ct` 项（master 现状），
并集时按原样保留，不在本批"顺手修"。closure：专用 grammar pass（约一次专注会话）→ 四命令 + 980 两套
用例 + 固定组 → 新基线独立 phase 复审 → 主检出干净窗口 fold（先等 `av/tests/conformance.rs` owner 落定）
→ 979 按落主 SHA 复核关闭 AC-HANDOFF。

**resync 完成（2026-11-16，提交 `25293cded`，未落主）**：`git merge master`（第二父 `fe75d239b`，
含 972/974/977/978/980/1043 等 43 个 ACE 提交）已合入 `plan-981-dev`，10 个冲突全部消解：
① 9 个代码/测试/schema 文件按"两侧行为都保留"的并集（`ComponentView` 同时保留 981 的
`exit`/`node_refs`/`scene_components` 与 980 的 `tree`；`PackagesFace::{Age,AggregatesCt}` 双面；
`Faces` 补 `AggregatesCt` 并在绑定器内派生 `aggregates` 车道；atom 的 `ExprKind` 取并集、`TypeKind`
保留 `f32`）；② grammar 30 处 profile 门自动并集 + 14 处语义合并，按"**age-package 采纳 980 富 view
作为超集**"：`age-node-ref` alt 置于 980 各形态**之前**（否则 `Disc d1 {}` 被 text-bare/element 吞）、
符号表式 `use mod: a,b` 置于点分导入**之前**（parser 在 `:` 失败后不回溯）、972 的
`packages-aggregates-ct-ops` pratt 表保持 ct-only；③ project.rs 删掉 981 旧的扁平 texts 循环
（与 `collect_view_items` 重复收集）。**行为对账**：并入 980 面后 `str`/`Str` 成为合法 model 字段
（状态函数按 `Str` 返回，`initialized=true`），981 原"str 具名拒"用例改为断言该合并后行为并注明来源；
`age-component` 的冻结 grammar id 现等于 master 的 980 面 `7a439096…`（980 B-DIALECT 的语义扩展，非门并集所致；age-package 在该合并提交为 `881e3506…`（R52 修复后 `ab740137…`），age-h0 仍 `64915bd1…`）。
**门禁**：`plan981` 146/146、`plan980` 13/13、`ace-core` 677/677（+79）、AV 3/3、A2R 2/2、grammar 45/45；
固定组 `cargo t ace-source ace-syntax` **两组 PASS**（exit 0，results 在 worktree `target/test-groups/run-cfnpamu4/`）。仍未闭：① 合并基线的**独立 phase 复审**（新基线
必须重审后才可 fold）；② 主检出 `av/tests/conformance.rs` 的他会话 WIP 仍未落定（fold 前必须由 owner
处理）；③ fold 后 979 按落主 SHA 复核。resolver 脚本与决策在 `plan-artifacts/plan-981/resync/`。

2026-11-16 待办（979 观察，非缺口）：`p981_fields_requirement_split_is_exhaustive` 比对的是硬编码
requirement 向量，不是从 Spec 合同产物派生。LANGUAGE 批实际纳入 L04/AG-A 时应改为从规范合同表派生，
避免"冻结的是当下认知"。owner：981/LANGUAGE（closure：合同项清单有 Spec 来源并在用例中断言来源）。
