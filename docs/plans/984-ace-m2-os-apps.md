---
plan_id: PLAN-984
status: executing
feature_name: AutoOS全部应用与系统宿主迁移
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-17
plan_revision: 10
current_step: 4
total_steps: 25
depends_on: [PLAN-959, PLAN-969, PLAN-970, PLAN-971, PLAN-972, PLAN-973, PLAN-974, PLAN-975, PLAN-977, PLAN-978, PLAN-980, PLAN-981, PLAN-982, PLAN-983, auto-os/PLAN-064]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-os-apps.md]
touched_goals: [GOAL-002]
---

# [PLAN-984] AutoOS全部应用与系统宿主迁移

## 0. 变更摘要

2026-11-16 r9（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包与OS064仍覆盖本版实际桌面/App/服务及required A/B职责；目标维度按本轮范围，不把Godot/TSCN或独立无消费的a2ts目标作为全App迁移的额外阻塞。现用Vue/Ark/Web及AGE自有2D/3D仍完整保留，25任务不变。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r7当前：明确B-CLOSURE-V可在OS064正式来源齐后交AGE/AG031所需OS覆盖切片，不等本包其它非桌面consumer全迁；整项CLOSURE及本Plan仍须全部原范围。25任务、0完成，A/B原门不变，仅规划。

历史修订：2026-11-10 r6执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r5当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-07 r3：桌面/28应用实施正文拆至[auto-os/PLAN-064](../../../auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md)，本包保7个语言接口/非桌面消费者/总闭合批次。全部仅规划，原范围未删除。
历史修订：2026-11-09 r4：用户确认M2需实际承接AGE游戏、传统AutoUI App和AutoOS桌面本身。OS064 r2强化原桌面/Node根AV+A2R验收，本包CLOSURE接其真实收据，不能只核框架/28 App数量。25任务保留、drafting、0/25，仅规划，不另立OS迁移计划。

逐产品迁移桌面shell/launcher/窗口/终端、Editor/Reader/Charts及全部实际App，含musk/down/服务/书稿工具、子模块固定版本、sdk副本和安装消费者；按能力分批收据，不修改业务删功能凑覆盖。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓引擎使用各Plan唯一tree/显式SDK；App按OS064当地独立clone规约，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11当前授权：用户要求修正过时规划状态/981计数，并将国风首切片、正式光影与联合验收细化为可执行合同，复用已交研究、不重复立项；本轮只修改规划/导航，不启动实施、建树、制作素材、构建、运行或发布。既有其他会话WIP与归档/当前Spec保留。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮已读OS AGENTS、apps.manifest与独立仓/启动Specs；OS要求领域正文在其仓。984不再重复桌面任务，OS064正式交付是本包最终前置。另核auto-shell/auto-ai/auto-coder/auto-code-rs/auto-edit真实Cargo/README/源文件，区分Auto产品、Rust消费桥及历史snapshot；只有实际可达旧引擎义务需迁移。书稿等剩余项由959逐行映射，未分配阻闭合。
2026-11-09追加产品口径只修订规划：OS域正文留OS064 r2，桌面已有`shell/{shell,desktop,switcher,notification_center,dashboard}.at`应明确迁入ACE；普通Node桌面根/原source兼容及AV/A2R实证由OS验，本包核正式来源/结果。无实现/桌面启动/构建/测试授权。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-os/host`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-os/apps`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/crates/auto-ui-runtime`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/ace-migration/replacement-scope-and-evidence.md`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：桌面与完整应用生态迁移覆盖/收据汇合。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-os/host/desktop/src](../../../auto-os/host/desktop/src)：OS064独占真实桌面实施，本包不复制；目录职责已核，确切符号/闭包开工再核

- [auto-ui/crates/auto-ui-runtime/src](../../../auto-ui/crates/auto-ui-runtime/src)：公共宿主/嵌入接口；目录职责已核，确切符号/闭包开工再核

- [docs/specs/ace-migration/m2-denominator.md](../specs/ace-migration/m2-denominator.md)：959冻结分母/新增delta，不以manifest快照封顶；读取SHA256前16 `c3dd5491ec1395aa`

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

本轮用户授权：仅修订现有980、981、1068、OS064、985–987及相关路线，不新立项、不启动实施。采用A最小真实交付→985/A M2A-08→B完整UI/Widgets的AV→AC→A2R三波→AutoOS全量迁移→候选/安装/退役。读取语言主线aa257fa1425c31fab48bd544cbd116b2fdce7dcd及现有Specs/已交HELLO、AXR0、SOURCE；UI示例真实源在auto-ui/examples/ui，Widgets源在auto-os/widgets-gallery/src/front/{pages,components}，ui-gallery副本需和规范源去重。新增接面/用例/收据仅是合同，不代表已运行。语言/标准库/非图形FFI的既定基础与测试守恒仍保留，不能借UI延期把普通语言必需能力全部后移。

## 5. 详细设计

### 三波/OS接收边界（本轮当前）

本包A仍只EMBED的真实最小仓外consumer，原语言/库/文件/后台/配置/资源oracle保持；不等待980/981完整图形、UI三波或OS全量。B的UI消费者按980当前后端/域接口准备；OS实际产品只由OS064在UI-W1-AV→UI-W2-AC→UI-W3-A2R后迁，B-CLOSURE最终核其完整桌面Node根公共三端及每App声明模式/数据/双Host和登记增量，不能将旧“AV/A2R”当遗漏AC的豁免。

OS不依本包CLOSURE/985 B最终；本包CLOSURE单向收OS和原其他生态来源。AGE AG031消费OS相关正式切片不反成为UI波或A出口前置。原25任务/0完成不变；旧OS/App规则和非桌面SHELL/AI/CODER/CODERS/EDIT义务不删，不复制OS实施表。V-B-CLOSURE在原行补错波次/错后端/漏App控制，原R与T-05核，本轮未执行。

### M2-TARGET-SCOPE-20261116：本包当前边界

本包与OS064仍覆盖本版实际桌面/App/服务及required A/B职责；目标维度按本轮范围，不把Godot/TSCN或独立无消费的a2ts目标作为全App迁移的额外阻塞。现用Vue/Ark/Web及AGE自有2D/3D仍完整保留，25任务不变。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A仅EMBED，按959/969/970/971-A/973/978消费通用接口；不等OS064整包、SHELL/AI/EDIT或产品全验。其余批次全部B，CLOSURE保OS/全生态终验。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | B-EMBED-I, B-EMBED-V, B-EMBED-R |
| M2-B | B-SHELL-I, B-SHELL-V, B-SHELL-R, B-AI-I, B-AI-V, B-AI-R, B-CODER-I, B-CODER-V, B-CODER-R, B-CODERS-I, B-CODERS-V, B-CODERS-R, B-EDIT-I, B-EDIT-V, B-EDIT-R, B-CLOSURE-I, B-CLOSURE-V, B-CLOSURE-R, T-05, T-06, T-07 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


LNNN=auto-lang/PLAN-NNN，OS064=auto-os/PLAN-064。frontmatter是正式完成并集，各批按最小前置；各批可独立实施/验证/复核，不是独立Plan归档。本包唯一实施树D:/autostack/.wt/lang-984/auto-lang保留到全包完成；其他owner已有计划则用其唯一树，否则按当地规约建立一次权威改面，App遵OS064独立clone规则。接口准备可用冻结SHA，正式交付必须已落地。

OS064直接依969/970/978/980等上游，不依984整包；984/CLOSURE单向等待OS064。OS主机、28App、gitlink与双Host验收正文只在OS064，不复制第二任务表。
AGE M2的M2-OS-REAL联合门消费OS064实际桌面/各App证据和本包覆盖核对，不要求本包其它非桌面消费者反向成为979/980/981前置。语言支持、AGE渲染支持、真实产品装配分别验收；不能把979/980/981完成直接称桌面已迁，也不等待AGE/ACE完整M2标签才执行OS批次。

| 批次 | 交付 | 最小前置 | owner |
|---|---|---|---|
| 984/EMBED | 通用嵌入与候选SDK交接 | L959/L969/L970/L971/L973/L978 | auto-lang |
| 984/SHELL | AutoShell脚本与结构化管道 | 984/EMBED; L972/L973/L974/L975; 982/EXEC | auto-shell |
| 984/AI | AI服务与工具执行桥 | 984/SHELL; L974/L975; 982/EXEC | auto-ai |
| 984/CODER | Auto Coder原Auto工程 | 984/EMBED; L974/L975; 982/RUST | auto-coder |
| 984/CODERS | Auto Code Rust与Auto对照工程 | 984/EMBED; L974/L975; 982/RUST | auto-code-rs |
| 984/EDIT | 独立Auto Edit编辑器 | L980/L971/L978; 982/EXEC | auto-edit |
| 984/CLOSURE | 全生态与OS正式收据闭合 | 984/EMBED; 984/SHELL; 984/AI; 984/CODER; 984/CODERS; 984/EDIT; OS064; L980/L981/L982/L983/L977 | auto-lang协调 + 各产品owner |

### 984/EMBED：通用嵌入与候选SDK交接
改面（核存在）：`crates/auto-lang-core/src/execution_engine.rs`；`docs/specs/ace-migration/replacement-scope-and-evidence.md`。
I实现：补齐经959确认的非UI嵌入缺口，验证独立宿主消费公开ACE session/host/工具接口；形成source、候选SDK与兼容错误合同，不在本包重新实现OS host。
V验收：仓外独立consumer运行原多模块脚本，跨session隔离；错SDK、权限拒绝、坏更新与关闭资源有独立结果；969/978现有收据可复用但需实际外部consumer。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/SHELL：AutoShell脚本与结构化管道
改面（核存在）：`D:/autostack/auto-shell/ash/Cargo.toml`；`D:/autostack/auto-shell/ash/auto-shell`。
I实现：替Shell脚本运行和Auto值桥旧入口，保结构化管道/CLI与沙箱策略；ash-gui若含可达旧执行链映射980，不能只验命令Rust部分。
V验收：原脚本闭包/try/catch、结构化管道结果与独立expected相符；read-only拒绝不触写，真实错误不系统shell回退；取消子进程回收。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/AI：AI服务与工具执行桥
改面（核存在）：`D:/autostack/auto-ai/Cargo.toml`；`D:/autostack/auto-ai/docs/specs/auto-ai-cli/shell-execution.md`。
I实现：核实际ACE可达桥与ash发现/脚本工具，迁有旧依赖处；纯Rust daemon不因在生态内而强制改写，保服务协议、路由、取消与策略。
V验收：隔离真实daemon+可控本地协议服务验证流/取消/并发和工具错误；显式ash身份、拒绝不fallback、使用记录正确；既有真实上游腿若属原义务仍单列，缺凭据未执行。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/CODER：Auto Coder原Auto工程
改面（核存在）：`D:/autostack/auto-coder/coder/main.at`；`D:/autostack/auto-coder/coder/sse.at`；`D:/autostack/auto-coder/coder/runtime/session.at`。
I实现：用新A2R/ACE构建原多文件工程，保SSE/工具/权限/REPL/会话；不把main_merged或旧生成Rust当唯一入口。
V验收：两provider协议、分段SSE、取消/重试、隔离会话恢复与工具失败逐项；原Auto源无业务删减，生成物不手修。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/CODERS：Auto Code Rust与Auto对照工程
改面（核存在）：`D:/autostack/auto-code-rs/Cargo.toml`；`D:/autostack/auto-code-rs/auto/main.at`。
I实现：核Rust工程及Auto源生成入口的真实旧依赖，切新构建链并保原API/tool/runtime/CLI流程；snapshot只作历史，不假称当前生产入口。
V验收：原Auto入口生成后真实API/工具/会话执行，对独立expected；Rust参考结果另列不掩盖共同错误，坏源不破坏当前产物。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/EDIT：独立Auto Edit编辑器
改面（核存在）：`D:/autostack/auto-edit/specs/auto-edit/pac.at`；`D:/autostack/auto-edit/specs/auto-edit/src/front/editor_store.at`；`D:/autostack/auto-edit/tools/compare`。
I实现：迁独立AutoUI编辑器多文件store/actions/编辑器桥/工具入口；与OS064中的jade-edit分别记分，不按同类名字去重。
V验收：多tab输入/菜单/快捷键、dirty关闭/保存/撤销重做与Console实际运行；save路径读正文，原大文档性能门保留。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

### 984/CLOSURE：全生态与OS正式收据闭合

AGE交接切片：OS064正式收据齐后，B-CLOSURE-V先按959/实际可达OS分母核桌面.at/普通Node根、28 App快照及新增OS consumer/原模式数据，绑定repo/SHA/SDK/ABI/平台与原oracle，独立复核该切片并按owner流程落主后交ag-examples/PLAN-031。该切片不关闭B-CLOSURE-V整项/AC，不等SHELL/AI/CODER/CODERS/EDIT等非桌面产品全部结果；全生态CLOSURE仍核原并集。AG031不复制App迁移，也不反向成为本包前置；切片实施仍遵M2A-08。
改面（核存在）：`D:/autostack/auto-os/apps.manifest`；`D:/autostack/auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md`；`docs/specs/ace-migration/replacement-scope-and-evidence.md`。
I实现：对959每个可达产品/API/副本逐行核OS064和各批正式来源，包含书稿/插件/后台及当前漏登consumer；新增项须先追加具名批次或当地计划，不能T-01空承诺。
V验收：OS064当前正式HOST/桌面`.at`及Node根AV+A2R、28 App全部原AC与MIX-05/06、非manifest各入口齐；删桌面源/候选/模式结果或任一App行、错来源/旧dispatch控制红。未知/不可读阻984与985，不写NA；OS正文只在OS064，本包不重复实现。
R独立复核：绑定本批revision/source/工具/SDK/consumer身份核正反例，未执行/缺环境不PASS。

#### T-01 A期冻结记录（2026-11-17，goal 授权链开工）

授权：用户以 `/goal 完成计划984/A` 授权本链（auto-plan-goal 范式）——A切片= T-01 +
B-EMBED-I/V/R + A交接 phase fold；B期任务（SHELL/AI/CODER/CODERS/EDIT/CLOSURE/
T-05..07）不动，仍等 M2A-08。不授权改目标/验收阈值/兼容承诺/仓与动作范围/预算。

A最小前置核实：959/969/970/973/978 已归档；971-A 切片已 phase fold 落主（lang
`2f1eed27f`，spec `docs/specs/ace-migration/m2-tools.md` sha16 `c69d0afd1aa1fdf3`
在 master）→ §5 A批（984/EMBED）前置满足；M2A-08 未闭（985 A批在途）→ 按§5阶段表
**只启动 M2-A 批**。唯一实施树 `D:/autostack/.wt/lang-984/auto-lang`（分支
`plan-984-dev`，基面 master@`e85096628`）；材料 scope
`D:/autostack/plan-artifacts/plan-984/`（owner=本链，登记于本记录）。

A切片嵌入合同冻结（B-EMBED 实现/V 遵此，不另立）：

- **仓外独立consumer**：crate `p984-embed-consumer`（scope `embed-consumer/`），
  path deps 仅 `auto-lang-core`+`auto-lang-ace-host` 两公开 crate（指向本树），
  scope 内 `cargo build`（dev）冷构建，target 不入任何仓。腿=boot/run-project/
  sessions/deny/closed。
- **公开接口消费集**：969 facade（`run_ace`/`AceRequest`/引擎选择/args-input拒）、
  967 F0 provider 生命周期（`build_f0_text_provider`/`revoke_provider`/
  `readmit_provider`/`teardown_provider`）、970 会话（`ace_session_create/op/Close`）、
  971-A 工程面（`install_ace_project_executor(project_host())`/`run_ace_project`）、
  978 SDK 入口（`ac-build build|sdk|run|verify|restore`，本树 dev 构建）。
- **原多模块脚本**：原 `stdlib/ace/std/math.at`+`text.at`（文件原样，冻结 SHA256 前16
  math=`0fe2cfc94f75785d`、text=`9b86f97c9c639abb`）为 `std` 包模块 + 最小 entry
  （app::main，跨包 use）；独立 oracle：square(7)×100+byte_len("Auto")=**4904**。
  产品级多模块工程（coder 等 str/env/fs 域）归 B 批，不冒 A 切片。
- **错误合同（逐码）**：错SDK=`m2build.recipe.missing/integrity/extra-file`；权限=
  `m2build.selection.authorization`（错 grant）、`ace.grant/no-provider`、
  `ace.capability/unsupported`（foreign 车道未冻结 cap）、`ace.provider/revoked`；
  坏更新=`m2build.selection.pin`（工件更新+声明哈希更新而 pin 未重钉，不自动升级）；
  关闭资源=teardown 后 `ace.provider/unknown` HostFailure、会话 Close 后具名拒；兼容门=
  `ace.args|input/unsupported`；每次拒后合法近邻复跑。
- **SDK腿**：`run-sdk-legs.sh`（scope）驱动 9 腿；SDK 工程=单公共源（978 冻结单源模型，
  多源具名拒归 978 owner 域不扩）；隔离异根重定位运行，oracle=49（原 math 合同）。
- **命令冻结**：仓内 `cargo t integration ace-av ace-host`（本树，nextest）；仓外
  `cargo build` + 各腿二进制（scope）+ `bash run-sdk-legs.sh`；平台 Windows x64。
- **资源**：scope target/、sdk-legs/{root,out,负控目录} 为可删缓存；每例 ≤10s
  （2026-11-14 测试裁定）；仓内新增 4 例永久用例在 `ace-host` 组
  （`p984_embed.rs`，nextest 每例独立进程——同进程单例冲突由组约定隔离）。
- **规范落点**：`docs/specs/ace-migration/m2-os-apps.md`（SD-01 A切片段，树内准备，
  fold 时落主）。

B期登记（只登记不实施）：SHELL/AI/CODER/CODERS/EDIT/CLOSURE 各批输入/oracle/命令见
§5/§6 原文；开工门=M2A-08 正式收据；OS 产品正文在 OS064。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[auto-os/shell/desktop.at](../../../auto-os/shell/desktop.at)（下列节选L28–L75；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
widget DesktopSurface {
    msg {
        Init,
        ActivateApp(str),
        IconMenu(str),
        // PLAN-012 W5：拖拽态机（press-press 两段式——onclick 即 iced
        // press；按下拾起 → 按到目标卡落下交换；按回原卡 = 取消）。
        IconPress(str),
        BlankPress, BlankDrop,
        MenuOpen,
        MenuRemove,
        MenuWallpaper,
        // PLAN-526 T10：空白处右键（mouse-area oncontextmenu——布局件
        // 右键臂缺位的替代挂点；菜单态本地门控，IconMenu 同型）。
        BlankMenu,
        // PLAN-019 v1.7：壁纸 picker（栅格点选即应用 / 预览二级态 /
        // 浏览换目录 / 关闭——语义全宿主收口，本面零分支）。
        PickerApply(str),
        // PLAN-035 T-11：桌面右键菜单「桌面小组件」开关（dashboard_toggle
        // v1.8 词面；可见性判据 = 宿主投影 __wm_dashboard）。
        ToggleDashboardBlank,
        PickerPreview(str),
        PickerBack,
        PickerNav(str),
        PickerBrowse,
        PickerDismiss,
        // PLAN-014 W-05'：空白菜单「恢复默认图标」（清 hidden 单源 +
        // refresh_desktop_icons 即时重注入）。
        ResetIconsBlank,
        // PLAN-014 W-02：总线写单点（同 shell.at SendCmd——追加语义，
        // 同排空周期多命令不互相覆盖）。
        SendCmd(str)
    }

    model {
        var __desktop_cmd str = ""
        var __desktop_bg str = ""
        var __desktop_icons = []
        var __desktop_hidden str = ""
        // PLAN-012 W5：格子化图标表（宿主注入——拖拽换位渲染面 + 平行
        // 字符串列表供 handler 下标读，B12 规避）。
        var __desktop_cells = []
        var __desktop_cell_ids = []
        var __desktop_cell_cs = []
        var __desktop_cell_rs = []
        var menu_id str = ""
        // PLAN-012 W5：拖拽态（按下拾起的图标 id；"" = 无拖拽）。
        var drag_id str = ""
```

本包所需独立结果：原桌面图标/窗口/通知/配置协议逐操作保持；Notes真实编辑保存重开、Clock倒计时/关闭和两host格有独立数据/事件预期，不用空桌面或进程存在充全生态。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 原生态入口/现数据→OS064或真实产品owner执行→ACE候选/AV/A2R/SDK→原业务与独立oracle→覆盖/正式来源收据。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ConsumerCoverage { entry: EntryIdentity, source: SourceId, engine_path: ProducerPath, modes: ModeSet, oracle: OracleRef, owner_plan: PlanRef }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-os/host/desktop/src](../../../auto-os/host/desktop/src) | OS064独占真实桌面实施，本包不复制 |
| [auto-ui/crates/auto-ui-runtime/src](../../../auto-ui/crates/auto-ui-runtime/src) | 公共宿主/嵌入接口 |
| [docs/specs/ace-migration/m2-denominator.md](../specs/ace-migration/m2-denominator.md) | 959冻结分母/新增delta，不以manifest快照封顶 |

**状态、错误与资源。** 缺未登记consumer、旧engine可达、错pin/host/source或数据损坏须阻相应行；未知项不能因不在28快照而NA。 EMBED-A真实host proof，SHELL/AI/CODER/CODERS/EDIT/CLOSURE各原义务保；OS切片不等非桌面生态全包，源变只失效相关收据。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-os/shell/desktop.at](../../../auto-os/shell/desktop.at) | 桌面真实协议源 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `376b156d4f458699` | 拟ace.m2.p984.source01：consume-product-receipt（不重复实施）；负控/资源见V-DETAIL-02/03；原任务 B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V |
| [auto-notes/src/front/app.at](../../../auto-notes/src/front/app.at) | Notes原业务/持久数据 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `19f2c7af0e39a66a` | 拟ace.m2.p984.source02：consume-product-receipt（不重复实施）；负控/资源见V-DETAIL-02/03；原任务 B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V |
| [auto-clock/src/front/app.at](../../../auto-clock/src/front/app.at) | Clock所有时钟模式 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `84388e5a481552ca` | 拟ace.m2.p984.source03：consume-product-receipt（不重复实施）；负控/资源见V-DETAIL-02/03；原任务 B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V |
| [auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md](../../../auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md) | 复用OS独占合同与各App独立clone，不重复实施 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `a26910dd641815aa` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

本包主要交生态覆盖/接口与收据，不复制OS064/各产品的业务实现；OS来源切片不等非桌面全生态整包。实际产品fixture和独立oracle仍由具名产品owner交付。

**原任务内的决定和交接。** 按959双向追真实入口/调用/运行路径，分OS064和其他产品owner/原输入/oracle；App不能从examples几份计数替原产品全范围。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-EMBED-I`、`B-EMBED-V`、`B-CLOSURE-I`、`B-CLOSURE-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/m2-os-apps.md | 总口径→通用嵌入/非桌面consumer与OS正式收据指针，不复制OS产品规范 | 各owner唯一知识 | AC-01..06、全部AC-B-* |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 959分母→每consumer已验证新producer/差额 | 替换闭合 | AC-03/05/06 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-os-apps.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-os-apps.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-os-apps.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-OS-WAVES | modify | docs/specs/ace-migration/m2-os-apps.md（真实交付） | EMBED A不反等全UI；CLOSURE B单向收三波后的OS正式产品，公共三端与App模式分账 | 用户本轮三波与A最小出口裁定 | AC-B-CLOSURE、AC-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


所有命令未执行。各批I先读实际runner/help/配置固定参数、隔离数据、现有与新增永久fixture；缺业务驱动须实现再V，不能只测旧库。材料统一D:/autostack/plan-artifacts/plan-984/，按repo/批区分，owner登记；生成/安装/外置target均纳scope，正常target留唯一树。共享IR/执行修改跑受影响完整固定组及真实下游，App-only不跑无关语言测试。

| Case | AC | 入口 | 独立观察 |
|---|---|---|---|
| V-B-EMBED | AC-B-EMBED | lang: cargo t integration ace-av ace-host；独立consumer按978正式SDK入口在scope冷构建运行 | 仓外独立consumer运行原多模块脚本，跨session隔离；错SDK、权限拒绝、坏更新与关闭资源有独立结果；969/978现有收据可复用但需实际外部consumer。 |
| V-B-SHELL | AC-B-SHELL | shell: cargo test --manifest-path ash/Cargo.toml -p auto-shell；原ash CLI隔离临时文件输入，现有命令由I读README固定 | 原脚本闭包/try/catch、结构化管道结果与独立expected相符；read-only拒绝不触写，真实错误不系统shell回退；取消子进程回收。 |
| V-B-AI | AC-B-AI | ai: cargo test -p auto-ai-client -p auto-ai-cli；其余实际受影响包由I依Cargo/原测试固定；不读取生产key vault | 隔离真实daemon+可控本地协议服务验证流/取消/并发和工具错误；显式ash身份、拒绝不fallback、使用记录正确；既有真实上游腿若属原义务仍单列，缺凭据未执行。 |
| V-B-CODER | AC-B-CODER | 新CLI按982/RUST已冻结入口生成/构建coder/pac.at工程；本地真实TCP协议fixture+现有根test_*.at回归，I补产品永久驱动 | 两provider协议、分段SSE、取消/重试、隔离会话恢复与工具失败逐项；原Auto源无业务删减，生成物不手修。 |
| V-B-CODERS | AC-B-CODERS | code-rs: cargo test -p ac-api -p ac-tools -p ac-runtime -p ac-cli；Auto工程用982/RUST真实命令，新增端到端驱动由I固定 | 原Auto入口生成后真实API/工具/会话执行，对独立expected；Rust参考结果另列不掩盖共同错误，坏源不破坏当前产物。 |
| V-B-EDIT | AC-B-EDIT | 复用specs/auto-edit/README.md的Tests与tools/compare/bench/perf现有runner；I冻结逐模式命令，980入口构建原工程 | 多tab输入/菜单/快捷键、dirty关闭/保存/撤销重做与Console实际运行；save路径读正文，原大文档性能门保留。 |
| V-B-CLOSURE | AC-B-CLOSURE | 只读分母/收据/来源完整性审计及受影响真实consumer重验；OS实际命令与App正文仅在OS064，发行安装终门交986/987 | OS28条全部原AC与MIX-05/06、非manifest各真实入口齐；删一行或错来源/旧dispatch控制红；未知或不可读产品阻984与985，不写NA。 |

源/安装SDK/动态配置/任务/坏更新收据保原MIX-06；OS064已验且身份不变可复用，变化只重验受影响腿。准备/运行/清理、选择/执行/排除/子终态分别记录；旧引擎仅带版本调查对照，不是唯一oracle。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V | 原桌面图标/窗口/通知/配置协议逐操作保持；Notes真实编辑保存重开、Clock倒计时/关闭和两host格有独立数据/事件预期，不用空桌面或进程存在充全生态。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V | 缺未登记consumer、旧engine可达、错pin/host/source或数据损坏须阻相应行；未知项不能因不在28快照而NA。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V | EMBED-A真实host proof，SHELL/AI/CODER/CODERS/EDIT/CLOSURE各原义务保；OS切片不等非桌面生态全包，源变只失效相关收据。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 全分母/来源/owner/命令/资源/平台冻结 | T-01及各I |
| AC-02 | work | 原产品能力经真实原源/工程执行，无删功能 | 各V及OS064正式收据 |
| AC-03 | work | 所有consumer新接线、旧数据/失败恢复/隔离/无fallback | 各V与CLOSURE |
| AC-04 | review | 独立逐批及全包核AC/差额/Spec/健康，绑定revision/SHA | 各R/T-05 |
| AC-05 | merge | 正式来源/Spec沉淀/归档/清理齐 | T-06/07 |
| AC-06 | work | 原全产品批次/状态兼容义务全部闭合，无unknown | CLOSURE及OS064 |
| AC-B-EMBED | work | 仓外独立consumer运行原多模块脚本，跨session隔离；错SDK、权限拒绝、坏更新与关闭资源有独立结果；969/978现有收据可复用但需实际外部consumer。 | B-EMBED-V/R |
| AC-B-SHELL | work | 原脚本闭包/try/catch、结构化管道结果与独立expected相符；read-only拒绝不触写，真实错误不系统shell回退；取消子进程回收。 | B-SHELL-V/R |
| AC-B-AI | work | 隔离真实daemon+可控本地协议服务验证流/取消/并发和工具错误；显式ash身份、拒绝不fallback、使用记录正确；既有真实上游腿若属原义务仍单列，缺凭据未执行。 | B-AI-V/R |
| AC-B-CODER | work | 两provider协议、分段SSE、取消/重试、隔离会话恢复与工具失败逐项；原Auto源无业务删减，生成物不手修。 | B-CODER-V/R |
| AC-B-CODERS | work | 原Auto入口生成后真实API/工具/会话执行，对独立expected；Rust参考结果另列不掩盖共同错误，坏源不破坏当前产物。 | B-CODERS-V/R |
| AC-B-EDIT | work | 多tab输入/菜单/快捷键、dirty关闭/保存/撤销重做与Console实际运行；save路径读正文，原大文档性能门保留。 | B-EDIT-V/R |
| AC-B-CLOSURE | work | OS064正式桌面`.at`及Node根AV/A2R、28 App原AC/双Host/MIX-05/06与非manifest入口齐；删桌面或App/模式结果、错来源/旧dispatch控制红；unknown阻984/985，不写NA。 | B-CLOSURE-V/R；复用OS064 V-SHELL-AGE等正式收据，不复制OS实现 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |

## 8. 执行步骤

当前共25任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜owner_stage: work｜AC-01/AC-PHASE**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。
  [✅ 2026-11-17] A期冻结记录见§5"T-01 A期冻结记录"（前置核实：959/969/970/973/978归档+971-A已fold@2f1eed27f，M2A-08未闭→只启A批；嵌入合同/命令/平台/资源逐项冻结；B期只登记）。goal授权链=`/goal 完成计划984/A`；树 `.wt/lang-984/auto-lang`@plan-984-dev（基面 e85096628）已建。

- [x] **B-EMBED-I｜M2-A｜owner_stage: work｜AC-B-EMBED/AC-02/AC-03/AC-06**：补齐经959确认的非UI嵌入缺口，验证独立宿主消费公开ACE session/host/工具接口；形成source、候选SDK与兼容错误合同，不在本包重新实现OS host。 按§5路径补永久fixture，固定§6命令。
  [✅ 2026-11-17] 仓外consumer `p984-embed-consumer`（scope，5腿）+ 仓内永久fixture `crates/auto-lang-ace-host/tests/p984_embed.rs`（4例，nextest 各≤0.2s 绿）+ SDK腿脚本 `run-sdk-legs.sh`（9腿）+ 错误合同逐码冻结（§5记录）；spec delta `docs/specs/ace-migration/m2-os-apps.md`（A切片段，树内待fold）。多源SDK工程=978冻结具名拒，不扩其owner域。

- [x] **B-EMBED-V｜M2-A｜owner_stage: work｜AC-B-EMBED**：执行V-B-EMBED完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。
  [✅ 2026-11-17] 冻结命令全执行（scope 收据 v-groups{,-2,-3,-4}.log/consumer-*.log/sdk-legs-v.log）：lang 组并集= integration 3352/3355（2红=master基线既有：cb_web_mime 主检出复放红 30.1s、plan814 declared_path_deps 主检出复放红；plan707_spike_cancel_gap 轮间浮现/隔离复放3/3绿=负载环境归因；首轮 plan702 红=组内缺 auto-ui 兄弟检出所致跨仓扫描路径空——补 `.wt/lang-984/auto-ui` 兄弟后复跑绿）+ **ace-av PASS** + ace-host(=ace-conformance 固定组) 155/11/16——11红全为已登记基线债（D-1001-R2×4/D-932-2/1060-R4/R5族，零本切片触面）+ **ace-facade（真实受影响组，含 p984_embed 4新例）PASS 43.1s**。仓外 consumer 5/5 腿 PASS（boot/run-project 4904/sessions 隔离+关闭/deny 具名拒/closed 资源）；SDK 腿 9/9 PASS（3s）。新用例 4 例实测 0.032–0.201s（≤10s 门）。机器占用如实：门期他会话 auto-web 定向测试并行（3–19 进程波动），ace-host 组内红与之无关（基线债身份独立成立）。

- [x] **B-EMBED-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。
  [✅ 2026-11-17] phase 复审 pass（局限=实施会话内复审，从提交工件重建）：reviewed f35c318b1；复审者独立重跑 p984_embed 4/4、自造异 payload 复现 m2build.selection.pin 拒绝+完体SDK 49 控制、新文件零警告、fmt clean、恰2新文件；AC-B-EMBED/AC-01/AC-PHASE A切片逐项 pass；2条具名澄清（组名ace-host实为conformance组/supersedes 乃整Plan终态声明）非缺陷；收据见§9 phase review 行。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **B-SHELL-I｜M2-B｜owner_stage: work｜AC-B-SHELL/AC-02/AC-03/AC-06**：替Shell脚本运行和Auto值桥旧入口，保结构化管道/CLI与沙箱策略；ash-gui若含可达旧执行链映射980，不能只验命令Rust部分。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-SHELL-V｜M2-B｜owner_stage: work｜AC-B-SHELL**：执行V-B-SHELL完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。

- [ ] **B-SHELL-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **B-AI-I｜M2-B｜owner_stage: work｜AC-B-AI/AC-02/AC-03/AC-06**：核实际ACE可达桥与ash发现/脚本工具，迁有旧依赖处；纯Rust daemon不因在生态内而强制改写，保服务协议、路由、取消与策略。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-AI-V｜M2-B｜owner_stage: work｜AC-B-AI**：执行V-B-AI完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。

- [ ] **B-AI-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **B-CODER-I｜M2-B｜owner_stage: work｜AC-B-CODER/AC-02/AC-03/AC-06**：用新A2R/ACE构建原多文件工程，保SSE/工具/权限/REPL/会话；不把main_merged或旧生成Rust当唯一入口。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-CODER-V｜M2-B｜owner_stage: work｜AC-B-CODER**：执行V-B-CODER完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。

- [ ] **B-CODER-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **B-CODERS-I｜M2-B｜owner_stage: work｜AC-B-CODERS/AC-02/AC-03/AC-06**：核Rust工程及Auto源生成入口的真实旧依赖，切新构建链并保原API/tool/runtime/CLI流程；snapshot只作历史，不假称当前生产入口。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-CODERS-V｜M2-B｜owner_stage: work｜AC-B-CODERS**：执行V-B-CODERS完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。

- [ ] **B-CODERS-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **B-EDIT-I｜M2-B｜owner_stage: work｜AC-B-EDIT/AC-02/AC-03/AC-06**：迁独立AutoUI编辑器多文件store/actions/编辑器桥/工具入口；与OS064中的jade-edit分别记分，不按同类名字去重。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-EDIT-V｜M2-B｜owner_stage: work｜AC-B-EDIT**：执行V-B-EDIT完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。

- [ ] **B-EDIT-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **B-CLOSURE-I｜M2-B｜owner_stage: work｜AC-B-CLOSURE/AC-02/AC-03/AC-06**：对959每个可达产品/API/副本逐行核OS064和各批正式来源，包含书稿/插件/后台及当前漏登consumer；新增项须先追加具名批次或当地计划，不能T-01空承诺。 按§5路径补永久fixture，固定§6命令。

- [ ] **B-CLOSURE-V｜M2-B｜owner_stage: work｜AC-B-CLOSURE**：执行V-B-CLOSURE完整职责与consumer，独立expected/旧数据/资源/身份/终态齐。 按本轮UI三波/OS来源表核公共桌面三端和App实际模式，不以旧AV/A2R用语豁免AC，缺波次/App/producer须红。

- [ ] **B-CLOSURE-R｜M2-B｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/来源，无scope遗漏；缺陷回本I，不冒全包交付。

- [ ] **T-05｜M2-B｜owner_stage: review｜AC-04**：核全部批次及OS正式交付与959分母，审遗漏/延期/workaround、warning/fmt/debug/Spec，unknown阻通过。

- [ ] **T-06｜M2-B｜owner_stage: merge｜AC-05**：先核各owner已合入来源，落通用Spec与产品指针、归档，交985候选预认证。

- [ ] **T-07｜M2-B｜owner_stage: merge｜AC-05**：摘要折Plan、全输出路径对账，清scope/刷新retention，wt-guard clean后清本树，不删除外仓独立clone。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-EMBED-I/B-EMBED-V/B-CLOSURE-I/B-CLOSURE-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: merge | phase fold（A切片；plan 保持 executing，同树保留至B——不归档/不清理/plan-artifacts 在途保留） | PLAN-984 | r10 | outcome: pass (phase) | prepared: 审查基线=phase pass f35c318b1（range-diff 全等重写映射 f35c318b1→f77d87722）+ canonical spec=m2-os-apps.md（A切片段，40行，随交付提交入库） | landed: rebase 到 master tip 30a42bee8（他会话 docs 前移，零代码触面）后主检出 `--ff-only`，**delivery=f77d87722**；rebase 后冒烟 p984_embed 4/4 PASS 0.129s；master tip=delivery ✓ | canonical_verified: `docs/specs/ace-migration/m2-os-apps.md` @f77d87722，sha256前16 `6347d528dd72e3c3`；无 legacy ledger 写；specs.json 未触碰 | archived: N/A（phase——B期未完不归档） | cleaned: N/A（phase——worktree/分支保留：`.wt/lang-984/{auto-lang@plan-984-dev, auto-ui@detached}` 至B；scope `plan-artifacts/plan-984/` 在途保留；登记表刷新顺延至全包 closeout——registry 存在 PLAN-1036 已登记的既有损坏（line 314 截断），fail-closed 不盲修，在途登记以本计划 §5 冻结记录为准） | 批量回归: 不 due（984%5=4≠0；.last-batch-regression last_covered=1064@2026-11-16<48h，其后再无 %5 落地） | next: M2-B 全部任务阻塞于 M2A-08（985 A批未闭）——同目标恢复 B 期即续（resync 后核 A 接口无变化）；985/A 消费本收据时按其自身门核

stage: goal | plan_id: PLAN-984 | plan_revision: r10 | round: 1 | from: work(T-01) | to: merge(phase fold) | status: executing | outcome: pass(A批三任务 work + B-EMBED-R phase pass + A切片落主 f77d87722) | worktree: D:/autostack/.wt/lang-984/auto-lang @ plan-984-dev（保留至B） | commit: f77d87722 (delivery) | next: done（目标"完成计划984/A"达成——A切片已正式合入可消费；B期属 M2-B，不在本目标内，等 M2A-08）

stage: review | review_scope: phase（B-EMBED-R，A切片；整体Plan保持executing） | reviewer: 实施会话内复审（本环境无独立复审上下文——从已提交工件重建结论，不采信执行摘要；该局限随收据记录） | PLAN-984 | r10 | outcome: pass (phase) | worktree: D:/autostack/.wt/lang-984/auto-lang @ plan-984-dev（clean；同组只读兄弟 auto-ui@2f42d661 detached） | reviewed_commit: f35c318b1 | base: master@e85096628 | dependency_revisions: 959/969/970/973/978归档；971-A fold@2f1eed27f；M2A-08未闭（B期阻塞保持，正确未越界）
spec_inputs: m2-facade.md/m2-sessions.md/m2-host.md/m2-build.md/m2-tools.md（969/970/967/978/971-A 已验接口）；SD-01 A切片段=m2-os-apps.md@f35c318b1（40行）
acceptance_results: AC-B-EMBED=pass（仓外consumer原多模块4904=49×100+4、原模块sha16与仓树逐字节同、跨session隔离+Close具名拒、错SDK三负控/权限/坏更新pin/关闭资源逐码独立结果——复审者以**自造异payload**独立复现 m2build.selection.pin 拒绝+完体SDK 49控制）；AC-01=pass（§5冻结记录）；AC-PHASE A切片=pass（B任务零触、M2A-08门遵守、无范围缩减）
reviewer独立执行: p984_embed 4/4 PASS 0.029-0.137s@f35c318b1；新文件零警告（全部为依赖既有警告）；fmt clean；git show stat=恰2新文件355行
findings: 无阻断缺陷。具名澄清2条（非缺陷）：(1) §6 组名"ace-host"实为ace-conformance固定组（971已记录的同名陷阱）——A切片真实受影响组=ace-facade已补跑PASS，冻结记录已如实登记；(2) frontmatter supersedes(replacement-scope-and-evidence.md)=整Plan终态声明，A切片仅落m2-os-apps.md新文件，SD-02分母行更新保持B/CLOSURE期pending（与971 phase fold同形，非遗漏）。fixture对"零dispatch"不重复断言callee_count——该纪律由969 e0_facade既有收据持有（复用边界，如实记录）
pending_closeout_gates: 全包终审=T-05（B期后）；984%5=4≠0、无>48h合并到期 → 本切片phase fold不触发 cargo t all 批量回归
evidence: §5冻结记录+§8三[✅]行+§9 work收据（组并集数字/基线红分诊/计时表）；scope收据 v-groups{,-2,-3,-4}.log+consumer-*.log+sdk-legs-v.log
next: phase fold 合入A切片（rebase master tip→ff-only；plan保持executing；同树保留至B；985/A消费时按其自身门核本收据）

stage: work | PLAN-984 | r10 | 2026-11-17 | outcome: pass（A批：T-01 + B-EMBED-I + B-EMBED-V；B期不动） | worktree: D:/autostack/.wt/lang-984/auto-lang @ plan-984-dev（基面 master@e85096628；同组只读依赖兄弟 auto-ui@2f42d661 detached） | code_commit: f35c318b1（crates/auto-lang-ace-host/tests/p984_embed.rs + docs/specs/ace-migration/m2-os-apps.md 两新文件，零改动既有源） | scope: D:/autostack/plan-artifacts/plan-984/（embed-consumer + sdk-legs + run-sdk-legs.sh + v-groups*.log/consumer-*.log 收据） | task_ids: T-01, B-EMBED-I, B-EMBED-V | evidence: §5 T-01冻结记录 + §8 三任务[✅]行；组并集/基线红分诊/新用例计时见 B-EMBED-V 行；spec delta m2-os-apps.md（A切片段）树内待fold | dependency_revisions: 959/969/970/973/978 归档；971-A fold@2f1eed27f；M2A-08 未闭→B期阻塞保持 | blockers: M2-B 全部任务前置 M2A-08（985 A批）——按§5阶段合同不启动；本批无未闭缺陷 | next_responsible_role: B-EMBED-R（A切片独立复审，同树停写后进行；pass后 phase fold 合入A切片，同树保留至B）

本轮会话测试模块表（test-discipline 2026-11-14）：

| 测试模块/实际范围 | 结果与执行数 | 准备/构建 | 运行 | 总墙钟 |
|---|---|---|---|---|
| worktree `cargo t integration`（两轮） | 3352/3355×2；2红=master基线既有（cb_web_mime/plan814，主检出复放红证据）；plan707=轮间负载抖动（隔离3/3绿）；首轮plan702=缺组内auto-ui兄弟（补后绿） | 首轮冷编译~10m | 153.9s/121.1s | 含编译~18m |
| worktree `cargo t ace-av` | PASS（组内无FAIL行） | 共享首轮 | — | 15m38s 三组一次调度 |
| worktree `cargo t ace-host`（=ace-conformance 固定组，非本批受影响组） | 155 pass/11 fail/16 skip——11红全为已登记基线债（D-1001-R2×4/D-932-2/1060-R4/R5族） | 共享 | 117.1s | 同上 |
| worktree `cargo t ace-facade`（真实受影响组，含p984_embed） | PASS | 增量~40s | 43.1s | 43.1s |
| 主检出基线复放（cb_web_mime/plan814/plan702×2） | 红/红/绿/绿（判基线归属与兄弟修复） | 暖 | 30.1s最长 | ~6m |
| 仓外 consumer 5 腿（scope 冷构建） | 5/5 PASS（boot 126ms/run-project 277ms/sessions 218ms/deny 791ms/closed 626ms） | scope冷编译~4m（dev） | <2s | ~4m |
| SDK 腿 run-sdk-legs.sh（9腿） | 9/9 PASS（异根重定位49/错SDK三负控/错grant/pin未重钉/完体复跑） | ac-build dev 44.8s | 3s | ~1m |

新增永久用例（ace-host 真实落位=ace-facade 组，nextest 每例独立进程）：p984_embed_host_boots_all_public_faces=0.074s（三公开面单进程组合安装+幂等+真工程跑通）、p984_embed_original_stdlib_modules_run_cross_package=0.201s（原stdlib math/text跨包4904+std::math原身份诊断+恢复）、p984_embed_session_isolation_and_close=0.074s（双会话隔离/关闭具名拒/幸存者不受影响）、p984_embed_permission_refusals_are_named=0.032s（grant/caps/args/input具名拒+合法近邻）。全部≤10s门。

2026-11-16 stage: new/revision | PLAN-984 r9 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-12 stage:new / PLAN-984 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原25任务/全部AC与V/依赖/阶段/预算和drafting 0/25；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-984:r7 / outcome:pass（OS覆盖交接边界可审阅）；blocked（未授权实施、OS064及A接口未交） / next:保持drafting0/25，B-CLOSURE-V按§5先交OS相关正式切片给AG031，其余生态义务不减；原任务/AC/Spec目标保留，无新实现或证书。

2026-11-10 stage=new / r6：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部25个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r5：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/25；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r4 / stage:new：复用OS064 r2桌面/Node根AV+A2R及各App真实验收，CLOSURE强化产品证据而不重复OS正文；25任务及全部原AC/V保留，drafting、0/25。OS只依上游实际接口，AGE联合门消费结果、不反向当981开工条件；仅规划、无运行证据，实施handoff仍blocked（未授权/前置未齐）。

规划静态核验：25任务/13AC/7验证行，原ID全保留、无重复、0完成；本包与OS064可达205份跨仓Plan，无缺失引用/依赖环，新链接可解析。OS产品正文/资源仍归OS owner，未运行桌面/构建/产品测试。

2026-11-07 r3：0/25，7批。旧T-02/03/04/08及AC-02/03/06的OS域正文迁OS064 HOST/A01–28/ASSEMBLY；其余分配本批I/V，原T-01/05/06/07保留。仅文档静态检查，未实施；历史0/8不再是当前计数。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。


2026-11-11 / planning_validation: pass（仅文档静态预检）。本轮8份新建/修订合同§0–10唯一、119任务/frontmatter/当前AC到任务覆盖一致、全部drafting/0完成；原任务/AC/V保留（UI058仅新增AC-11/V-S08），4份新合同各10任务/9AC/6验证。当前四仓可达221个Plan节点，无缺失引用/依赖环；UI039原活动影子/归档双文件按既有同号身份处理，不新计交付。十九份触达规划/导航的393个本地文件链接可解析，16处旧归档链接已纠正，定向diff空白检查通过；研究Spec、r187的621原表和pipeline研究输入hash保持不变。批次级检查确认979/980/981不反等新增增强/AG031，985/A不反等G2/G3或AG031，984只交OS覆盖切片而不要求AGE等全生态。未构建/运行用例、制作素材、创建实施树/新证书/新canonical、提交/push或打tag；此记录不代实现独立final。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

2026-11-16 stage:new/revision | PLAN-984 r10 | outcome:pass（规划可执行合同，非实现/认证PASS） | 25原任务/0完成保留；仅同步A EMBED与B CLOSURE来源边界，OS主责仍OS064 | next:由原owner依新阶段/真实来源接续；本轮只规划，不实施/建树/测试/合入/清理/恢复目标。

本轮规划静态预检：14份计划/设计/路线文档，8份Plan共230个当前任务；仅980新增B-AV-I/V/R三项并按完整W3范围重开原RUST三项，旧AXR0收据保留；981仅核正原SOURCE三项已交的滞后计数。全部原任务ID/其余勾选、各Plan状态与整包依赖列表保留；新增本地链接/锚点15处、章节/表格/元数据计数/验收任务映射及行尾检查通过，A最小→A出口→W1→W2独立交接→W3→OS→候选/安装/退役阶段图16节点18边无环。原GUI/Widgets/Notes-Clock/语言/库/产品义务有具名阶段与原owner，不以延期冒PASS。仅文档预检，未实施/建树/运行测试或窗口/签阶段/合入/清理/恢复目标，未写源码、测试资产、canonical Specs或取号计数器；他会话并发修改不由本轮回退。

## 10. 待澄清事项

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

未决接口/工具与源码/Spec差额由具体批次I有界决定；当前批次已具名，不再把拆分推迟给总T-01。959新增consumer须追加有owner/原模式/命令/oracle的批次或当地Plan，逐AC迁移后再实施；不可读项保持阻签。后续实施须用户指令，本轮仅规划。
