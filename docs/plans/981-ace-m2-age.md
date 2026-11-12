---
plan_id: PLAN-981
status: drafting
feature_name: AGE Node直接接入ACE与AV开发及A2R游戏发行
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 11
current_step: 0
total_steps: 34
depends_on: [PLAN-960, PLAN-961, PLAN-962, PLAN-966, PLAN-967, PLAN-968, PLAN-969, PLAN-970, PLAN-972, PLAN-978, PLAN-979, PLAN-980, ag-examples/PLAN-024, ag-examples/PLAN-025, ag-examples/PLAN-028, auto-ui/PLAN-061, auto-ui/PLAN-062, auto-ui/PLAN-063, auto-ui/PLAN-067, auto-ui/PLAN-072]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-age.md]
touched_goals: [GOAL-002]
---

# [PLAN-981] AGE Node直接接入ACE与AV开发及A2R游戏发行

## 0. 变更摘要

2026-11-12 r11：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r10当前合同：十批（SOURCE/SESSION/RUST/LANGUAGE/PROOF、CONTENT/RUST2D/RUST3D/FUSION/AUDIT）共34任务、0完成。AUDIT-I/V核其余九批正式复审/交接及全部required行，AUDIT-R后十批齐交T-05；不以旧8/9批计数遗漏PROOF。仅修正规划，不实施。新增增强合同UI080、AG029/030及联合AG031消费本包最小A/所需B正式来源，不成为本包或985/A反向前置。

历史修订：2026-11-10 r9执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r8当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r7：M2必须支持传统AutoUI及完整AGE作者语法，不能只支持三类view/游戏。新增LANGUAGE完整语法/语义批，接980传统UI全模式，补五类view/选择、if/#if、Start/Tick/Frame、新旧UI组件互操作；原8批和两游戏A2R验收保留，现9批31任务、0/31、drafting。只规划，不实施/新增计划。


承原M4/2026-11-07口径M3的融合义务，现归合并AGE M2，在共同Schema链支持Node/use/model/msg/on、view/view-2d/view-3d、Life.Tick、UI+2D+3D；替换driver真实AutoVM，保direct/RQ。现有执行迁移和两游戏新后端发行可先交，生产融合并行，不等待新增2D/3D增强全部完成；M1原游戏/包不重开。合并阶段恢复M2规划，仍不授权实施。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

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

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A=SOURCE/SESSION/RUST/LANGUAGE/PROOF；LANGUAGE-A验证不等CONTENT/RUST2D/RUST3D/FUSION/AUDIT。LANGUAGE原全产品组合义务由B/FUSION及AUDIT接收；A仍核完整语法/组件互操作，不能用最小例减范围。B=CONTENT/RUST2D/RUST3D/FUSION/AUDIT，消费A正式收据及原AG/UI所需接口。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-PROOF-I, B-PROOF-V, B-PROOF-R, B-SOURCE-I, B-SOURCE-V, B-SOURCE-R, B-SESSION-I, B-SESSION-V, B-SESSION-R, B-RUST-I, B-RUST-V, B-RUST-R, B-LANGUAGE-I, B-LANGUAGE-V, B-LANGUAGE-R |
| M2-B | B-CONTENT-I, B-CONTENT-V, B-CONTENT-R, B-RUST2D-I, B-RUST2D-V, B-RUST2D-R, B-RUST3D-I, B-RUST3D-V, B-RUST3D-R, B-FUSION-I, B-FUSION-V, B-FUSION-R, B-AUDIT-I, B-AUDIT-V, B-AUDIT-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan局部批次可独立实施/验证/复核，不新增Plan状态，也不等于可单批归档/删树。本Plan唯一worktree保留至全部批次及最终review/merge完成。分支冻结SHA只供受控准备；下游最终验收须核已合入来源，批次review不冒正式D。

frontmatter是整包最终依赖并集，各批按下表最小接口前置启动。LNNN=auto-lang/PLAN-NNN；UI/AG/OS标仓身份。依赖收据必须包含repo/commit、Plan revision、接口/schema/能力域和实际consumer/oracle。只有计划名字不等接口就绪。

唯一协调组`D:/autostack/.wt/lang-981/`，主实施树`auto-lang`、兄弟`auto-ui`/`ag-examples`各一权威tree，拟分支`codex/plan-981-dev`；尚未创建。实施/review顺序复用，公共文件单写。外仓开工重核当地AGENTS和唯一树，不建junction/symlink。OS/App仍归OS064，不在本包另建实现。

| 批次 | 交付结果 | 最小前置 | owner |
|---|---|---|---|
| 981/SOURCE | Node场景和行为直接进入ACE共同前门 | UI067既有结论; L979投影合同; L960/961/962及L967的实际所需接口 | auto-ui领域主责，auto-lang通用语义 |
| 981/SESSION | AV开发运行与AGE typed宿主/在线更新 | 981/SOURCE; L967/968/969/970；不等RUST或980整包 | auto-ui + auto-lang会话owner |
| 981/CONTENT | 两游戏/Gallery/编辑/离线开发链迁AV | 981/SESSION; AG024/025原义务映射; L978候选SDK；复用UI072/AG028 | auto-ui + ag-examples |
| 981/RUST | A2R→Rust编译链接AGE宿主的公共发行链 | 981/SOURCE; L967的Rust typed host、L969构建入口、L978构建/SDK；实际使用的L960/961/962/972支持格 | auto-lang新A2R主责，auto-ui宿主主责 |
| 981/RUST2D | 原2D游戏同源AV/A2R对照及最终Rust便携包 | 981/RUST; CONTENT对应2D的AV证据（仅最终V/R需要）；AG014/025/028与UI072的已交接口/原oracle | ag-examples发行主责，语言/UI各修自身接面 |
| 981/RUST3D | 原3D游戏同源AV/A2R对照及最终Rust便携包 | 981/RUST; CONTENT对应3D的AV证据（仅最终V/R需要）；AG016/025/028与UI072的已交接口/原oracle | ag-examples发行主责，语言/UI各修自身接面 |
| 981/LANGUAGE | 完整AGE语法/语义及传统UI组件互操作 | I：SOURCE、979合同、980/DIALECT/STATE接口与L966；V/R：LANGUAGE-I、SESSION/RUST、980实际交付及所需980/PARITY来源（A语义）；完整产品组合交B/FUSION/AUDIT | auto-ui领域主责，语言共同语义/AV/A2R，ag-examples联合样例 |
| 981/FUSION | AGE与生产UI direct/RQ融合 | 981/SESSION; 980/PARITY; UI061/062/063适用接口 | auto-ui |
| 981/AUDIT | AGE全旧消费者、完整语言及AV/A2R发行交接 | CONTENT; RUST/RUST2D/RUST3D; LANGUAGE; FUSION | auto-ui + ag-examples + auto-lang |
| 981/PROOF | RG-A04真实混合作者工程 | L981/SOURCE, L981/SESSION, L981/RUST, L981/LANGUAGE, L980/PARITY, L978, UI061, UI062, UI063 | 原owner；RG-A具名真实验证 |

### 职责交接与无环并行顺序

979只交通用grammar/typed投影/唯一verify与最小双方言双后端证明，不承接本包完整AGE/GPU/游戏验收；981只扩该接口，不再开发另一套Node前端或通用A2R。语言仓维护绑定/类型/资源事实、AV和A2R合法化；auto-ui维护Node领域语义、场景/绑定描述、资源/输入/音频/窗口renderer及AV/Rust中立宿主；ag-examples维护作者源、原玩法oracle、资源闭包、各模式打包与最终ZIP验证。Core禁止反向依赖UI。967管通用FFI/handle，968/970管调度/会话，972管标准库，978管通用工程构建/SDK；981消费并补AGE实际组合与接线，不复制这些系统。980/RUST是UI静态工程，不代AGE A2R验收。

1. 实施获得授权后，T-01先核当前交付、建立源码/构造/宿主操作到上述owner的差额表；缺口必须具名，不把AG028完成改成A2R完成。979平台开发与981的领域合同、原oracle/包清单准备可并行；981实际消费等相应979接面正式合入。979的最小消费者不依赖981或980最终结果，不能互等。
2. SOURCE的共同受检语义就绪后，SESSION与RUST可按各自最小接口并行；RUST不依赖AV会话/热更新完成，SESSION也不等Rust发行。通用宿主/构建计划依赖稳定领域接口，不以981最终游戏验收作其完成前置。
3. CONTENT随SESSION就绪迁移AV；RUST2D、RUST3D随RUST就绪可分别准备/构建，不要求2D验收通过后才开始3D。各游戏最终V/R必须拿到同源同资源的AV对照和独立oracle；不得只拿旧VM结果替AV。
4. FUSION随SESSION与980/PARITY、现有RQ/UI接口成熟推进，不阻塞SOURCE/SESSION/CONTENT或两个游戏Rust包；其余九批R齐后进行AUDIT-I/V/R，十批齐后进入T-05→T-06→T-07，AUDIT不能等待自己的R才开工。不能等985最终认证、986正式发行或987删旧引擎后才构建候选，981负责交候选/证据。

5. 合并AGE M2另有G2/G3线：UI058先做中立查询/精灵/碰撞服务，三维正式光影先冻结资源与端口，均可与本包并行。其最终正常Node/AV/A2R接线消费本包正式交付；本包现有M1迁移/融合不反向等待UI058/AG023、新光影或国风首切片终验。979只证最小平台，不背完整增强GPU验收。新增typed端口/能力/资源schema须版本化登记并核AV/A2R支持；不为新增图形扩旧Node生成器或复制执行器。新增强合同仍drafting、未实现，不充本包已迁事实，已启用consumer必须有明确新producer/迁移owner，不能遗留无人接管的旧SDK依赖。

6. SOURCE仍是既有M1作者源迁移切片，不冒完整语法；LANGUAGE-I可与SESSION/RUST/PROOF在A期并行；CONTENT在A出口后于B推进，原CONTENT与RUST2D/RUST3D的内部并行关系保留，完整V须依相应后端和980正式接口，FUSION最终V核LANGUAGE组合结果。980不反向等LANGUAGE/FUSION产品PASS，979不等两者；AUDIT-I/V核其余九批R及全部required行，自己的R完成后十批齐交T-05。游戏迁移不先强制使用所有view，其通过也不替LANGUAGE。

并行指不同写面/接口就绪的工作，不授予另建agent/chat或多权威tree。共享schema/core/verify/host ABI、打包公共工具按owner短窗串行；同机GPU/窗口/音频实测单实例。稳定合同只允许受控准备，最终证据绑定正式交付repo/SHA、Plan revision、grammar/domain版本、编译/运行后端、host ABI/provider、源闭包与资产hash、平台/toolchain；不能用开发tip/旧SDK锁代替新来源。

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


### A期PROOF批次（新增、未实施）

最小前置：L981/SOURCE, L981/SESSION, L981/RUST, L981/LANGUAGE, L980/PARITY, L978, UI061, UI062, UI063。实际改面：`D:/autostack/ag-examples/showcases/ui-2d-3d`；`D:/autostack/auto-ui/experimental/autographics-auto-driver/tests/authoring_session.rs`（均核存在）；永久ACE fixture由B-PROOF-I在原owner建立，临时材料仍归本Plan的plan-artifacts scope，不新增第二runner/语义实现。

I：用原showcase正常Node源建立同源AV/A2R真实UI+2D+3D验证，复用SOURCE/SESSION/RUST/LANGUAGE与renderer，冻结事件/帧/输入/更新及独立expected；不复制CONTENT的两完整游戏/全Gallery/编辑和离线包义务。
V：两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。
R：独立核普通源→新producer→真实consumer、原expected/资源/退出及source/SDK身份；本批只关闭RG-A义务，不冒全部产品迁移。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r11当前）

**具体输入与期望。** 原件[ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../ag-examples/examples/g01-shapes/g01-main/src/main.at)（下列节选L7–L38；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

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
| [ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../ag-examples/examples/g01-shapes/g01-main/src/main.at) | 原正常作者源/属性/引用 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `9b809a1c6d99985b` | 拟ace.m2.p981.source01：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/games/platformer-2d/src/main.at](../../../ag-examples/games/platformer-2d/src/main.at) | 原2D完整玩法与最终ZIP | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `01b1ce4d34e6cf68` | 拟ace.m2.p981.source02：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/games/platformer-3d/src/main.at](../../../ag-examples/games/platformer-3d/src/main.at) | 原3D完整玩法/资源/相机 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `c24f2e86cf5c71f1` | 拟ace.m2.p981.source03：run-av-age/run-a2r-age；负控/资源见V-DETAIL-02/03；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |
| [ag-examples/showcases/ui-2d-3d](../../../ag-examples/showcases/ui-2d-3d) | 混合原showcase，所需普通源与资源闭包完整 | 原项目/工具闭包；T-01核完整源/断言，目录存在不是执行证据；SHA256前16 `目录闭包待冻结` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V |

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

外仓知识由owner落其模块Specs：UI ui/runtime/build、AGE auto-graphics、Man cli/modules、Web architecture/http-contracts/testing；语言侧只保通用接口和迁移指针，不复制第二份领域规范。

## 6. 测试设计

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
| V-A-PROOF | AC-A-PROOF | ui AGE: cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test authoring_session --test mixed_regions_lifecycle_flow -- --test-threads=1；showcase原verify/run参数由I冻结并执行真实AV/A2R两腿；窗口单实例 | 两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 原四图形值/可见性同源；含行为的两游戏还须真实AV与A2R编译/窗口/输入/音频及独立玩法oracle；同实例Start=1/Tick=5/Frame=3与五view选择原表齐。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 未知view/重复选择键、错引用/生命周期body/ABI/旧代输入具名拒；3D无隐式UI fallback，旧VM仍被其他产品编译不证明新链dispatch它。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | Start/Tick/Frame与多出口不重复推进model；隐藏/零尺寸/晚事件/资源共享与设备epoch遵冻结规则；十批34任务保留、AUDIT核其余九批。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

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
| AC-A-PROOF | work | 两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。 | B-PROOF-I/V及V-A-PROOF；R独立复核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共34任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [ ] **T-01｜M2-A准备｜owner_stage: work｜AC-01/AC-PHASE**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。

- [ ] **B-SOURCE-I｜M2-A｜owner_stage: work｜AC-B-SOURCE/AC-02/AC-03/AC-06**：按§5 SOURCE接979共同前门，领域描述/行为共同verify，保持f32/数组/bool、实例与原span；逐项迁旧生成骨架语义，创建V-B-SOURCE永久正反例，不保旧源码重解析链。与语言owner冻结共源AV/A2R接面。

- [ ] **B-SOURCE-V｜M2-A｜owner_stage: work｜AC-B-SOURCE**：运行V-B-SOURCE及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-SOURCE-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-SESSION-I｜M2-A｜owner_stage: work｜AC-B-SESSION/AC-02/AC-03/AC-06**：按§5 SESSION替旧AutoVM为AV+typed宿主，保Tick/回调/资源/更新/停止恢复，补同PID body编辑cargo/rustc=0及坏编辑保旧用例；与RUST分写面并行，不手写Rust玩法。

- [ ] **B-SESSION-V｜M2-A｜owner_stage: work｜AC-B-SESSION**：运行V-B-SESSION及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-SESSION-R｜M2-A｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-RUST-I｜M2-A｜owner_stage: work｜AC-B-RUST/AC-02/AC-03/AC-06**：按§5 RUST接新A2R/provider与typed AGE静态宿主及978构建，语言owner关闭实际实例状态/初始化/调用的目标合法化缺口；建V-B-RUST和V-R-FAIL公共正反例，明确生成代码实际dispatch与禁VM验证；复用通用系统，与SESSION并行。

- [ ] **B-RUST-V｜M2-A｜owner_stage: work｜AC-B-RUST**：运行V-B-RUST/V-R-FAIL公共部分和冻结语言职责并集，实际生成→编译→执行GPU/输入/音频；记录值/事件/资源、身份和故障恢复，emit-only或VM补逻辑不通过。

- [ ] **B-RUST-R｜M2-A｜owner_stage: review｜AC-04**：同树停写独立核新A2R实际链、宿主资源/失败及无解释器执行证据，pass只关闭公共RUST批，不替两游戏。

- [ ] **B-LANGUAGE-I｜M2-A｜owner_stage: work｜AC-B-LANGUAGE/AC-02/AC-03/AC-06**：按§5 AGE-L01..05建立完整语法/AST/投影/运行consumer分母；复用979/980，接五view/选择/if/#if/Frame及新旧组件互操作，先闭主时钟/挂载/检查策略决定；在既有target建V-L-*普通源正反例，命令/原span/独立oracle冻结，不复制前端/控件库。

- [ ] **B-LANGUAGE-V｜M2-A｜owner_stage: work｜AC-B-LANGUAGE**：所需SESSION/RUST及980正式来源齐后，实跑V-B-LANGUAGE和全部V-L-*，同源AV/A2R各对独立语义、资源、真实UI/图形oracle，核删除必需构造/后端腿负控；未决/缺环境保pending，不用游戏或最小view样本替代。

- [ ] **B-LANGUAGE-R｜M2-A｜owner_stage: review｜AC-04**：同树停写独立逐AG-A/AGE-L/实际grammar与产品格核源码→受检语义→双后端→实际consumer，核传统UI及多呈现/Frame/条件分期；缺任一必需格不能关闭，pass只关闭本批。

- [ ] **B-PROOF-I｜M2-A｜owner_stage: work｜AC-A-PROOF**：用原showcase正常Node源建立同源AV/A2R真实UI+2D+3D验证，复用SOURCE/SESSION/RUST/LANGUAGE与renderer，冻结事件/帧/输入/更新及独立expected；不复制CONTENT的两完整游戏/全Gallery/编辑和离线包义务。 按§5路径补永久fixture，冻结V-A-PROOF真实命令/输入与平台。

- [ ] **B-PROOF-V｜M2-A｜owner_stage: work｜AC-A-PROOF**：运行V-A-PROOF及完整受影响职责/必要consumer，记录两个局部实例事件2:1、Tick/Frame不随view倍增；真实GPU/输入/ROI与CPU状态一致，原span、坏更新保旧、关窗回收；headless或首帧不够。 核子终态与身份，未执行不PASS。

- [ ] **B-PROOF-R｜M2-A｜owner_stage: review｜AC-A-PROOF**：停写后独立核本批全源路径/正反例/差额，绑定revision与SHA；owner按阶段fold正式合入后，才交985/A。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-CONTENT-I｜M2-B｜owner_stage: work｜AC-B-CONTENT/AC-02/AC-03/AC-06**：按§5 CONTENT迁两游戏/Gallery/混合/全部profile至AV与新SDK，保编辑保存/SaveAs/搬迁/关开及旧间接链义务；按游戏提供同源AV收据，制品明确分型，不将AV包认A2R。

- [ ] **B-CONTENT-V｜M2-B｜owner_stage: work｜AC-B-CONTENT**：运行V-B-CONTENT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-CONTENT-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-RUST2D-I｜M2-B｜owner_stage: work｜AC-B-RUST2D/AC-02/AC-03/AC-06**：消费RUST和原2D源/资源/已交工具，补既有verify/packager的新后端身份与V-R2-*、V-R-PACK/FAIL；真实A2R构建release并产最终exe+资源ZIP/来源/许可/说明，复用AG028而不重建玩法。

- [ ] **B-RUST2D-V｜M2-B｜owner_stage: work｜AC-B-RUST2D**：对应AV证据齐后跑V-B-RUST2D及全部V-R2-*、共享V-R-PACK/FAIL的2D腿；逐原G2-01..09、独立oracle、仓外最终ZIP真实试玩/故障恢复、无VM执行逐项记录，缺条件保持pending。

- [ ] **B-RUST2D-R｜M2-B｜owner_stage: review｜AC-04**：同树停写独立核2D完整链和最终ZIP SHA，旧AG028/AV结果不替新A2R，全部必需格闭合才pass本批。

- [ ] **B-RUST3D-I｜M2-B｜owner_stage: work｜AC-B-RUST3D/AC-02/AC-03/AC-06**：消费RUST和原3D源/资源/已交工具，补既有verify/packager的新后端身份与V-R3-*、V-R-PACK/FAIL；真实A2R构建release并产最终exe+资源ZIP/来源/许可/说明，保GLB纹理闭包；无需等2D验收完成。

- [ ] **B-RUST3D-V｜M2-B｜owner_stage: work｜AC-B-RUST3D**：对应AV证据齐后跑V-B-RUST3D及全部V-R3-*、共享V-R-PACK/FAIL的3D腿；逐原G3-01..08、独立oracle、仓外最终ZIP真实鼠标/键盘/音频和故障恢复，不能拿2D或headless绿替代。

- [ ] **B-RUST3D-R｜M2-B｜owner_stage: review｜AC-04**：同树停写独立核3D完整链、相机/音画/资源闭包与最终ZIP SHA，所有必需格闭合才pass本批。

- [ ] **B-FUSION-I｜M2-B｜owner_stage: work｜AC-B-FUSION/AC-02/AC-03/AC-06**：承合并AGE M2融合线，依现有M1基础和980成熟接口接生产组件/主题/Node，保原兼容义务和direct/RQ/device/epoch/输入路由；不等待UI058/AG023或新增光影终验。按§5接永久fixture并冻结命令。

- [ ] **B-FUSION-V｜M2-B｜owner_stage: work｜AC-B-FUSION**：运行V-B-FUSION及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-FUSION-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **B-AUDIT-I｜M2-B｜owner_stage: work｜AC-B-AUDIT/AC-02/AC-03/AC-06**：全旧消费者及全部十批新producer（AUDIT-I/V先核其余九批，含PROOF）/完整语言/传统UI/双后端/最终两ZIP逐项对账，补删除必需行负控；向985/986/987交候选/身份/实际证据，不以后继全生态门替本包AC。

- [ ] **B-AUDIT-V｜M2-B｜owner_stage: work｜AC-B-AUDIT**：运行V-B-AUDIT及完整职责/必要集成；记录独立结果、源/目标身份、资源、子终态与条件排除。

- [ ] **B-AUDIT-R｜M2-B｜owner_stage: review｜AC-04**：暂停写入，在同一树独立复核本批AC/diff/反例/跨仓来源；失败回I修复，pass只关闭本批，不冒全包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：全包独立final，核遗漏/延期/组合、warning/fmt/debug及Spec增量，缺必需批收据不得pass。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：按owner依赖次序合入、落canonical并核实际消费者/接口同版本，全部齐才归档。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan，逐输出路径对账/删除scope/刷新retention，guard clean后清树；部分完成保留树。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-SOURCE-I/B-SOURCE-V/B-SESSION-I/B-SESSION-V/B-RUST-I/B-RUST-V/B-LANGUAGE-I/B-LANGUAGE-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

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

## 10. 待澄清事项

历史039活动/归档同名歧义按归档来源与实际接口核验，不能列作未完成游戏工作或自动重开；T-01重核当前副本，但不阻本次规划、不清理他仓文件。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结十批及并行前置（AUDIT核其余九批，含PROOF），T-01只补符号/命令/平台与资源决定，不从零拆包或新增重复计划。SOURCE/RUST管既有源的共同投影/表示；LANGUAGE-I须关闭完整语法分母、Frame主时钟/零尺寸/不可见出口、挂载切换/编译期检查的具名策略，不得把未决点删出“完整支持”。普通语言/provider缺口归962/966/967/972/978等实际owner或本包合法化，不能凭960/961已归档略过；源码错误、目标缺能力、资源拒绝、编译/环境故障分列。
本包游戏发行保持Windows x64两候选；新增2D/3D由合并M2的原图形合同/待冻结合同规划承接，不移入本包重做。其他平台、新关卡/美术/商店发布不因合期获实施授权。候选生命周期/去向由T-01登记并由T-06/07按保留政策对账；既有AG028试玩包不属本包清理范围，不覆盖删除。
