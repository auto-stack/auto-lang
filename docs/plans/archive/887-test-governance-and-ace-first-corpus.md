---
plan_id: PLAN-887
status: archived
feature_name: 测试治理与ACE首批消费（守恒、提速与补覆盖）
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29

plan_revision: 2
supersedes_spec_components: [docs/specs/auto-test-case/project.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/plans.md]
new_spec_components: [docs/specs/auto-test-case/governance-and-inventory.md]
touched_goals: []

affects: [auto-test-case, test-infrastructure, repository-boundaries]
depends_on: [PLAN-884]
current_step: 10
total_steps: 10
---

# [PLAN-887] 测试治理：完整盘点、可信裁判、执行提速与ACE首批消费

> 本轮仅立计划，暂不实施；不建实施树、不运行测试、不联系其他会话、不创建agent。
> 884继续按原合同收口；887不向884追加验收项，不取代其独立终审和冻结门。
> 整体实施启动门 G887 = 用户后续明确执行授权 ∧ U884；写入另须 X887实际写面释放/接受。
> U(N)包含完整验收、独立终审、冻结门、实际合入、规范/账本、归档及guard收尾；状态头、阶段pass、list-only不代替U。

## 0. 变更摘要

将后续主要质量投入从待替换AutoVM/A2R的广泛内部审查，转向可跨引擎保留的测试资产。本Plan作为一个完整工作包交付：三仓及ACE入口全域盘点与义务对账、批量选择守卫修复、首批中立语料与ACE真实消费、有限成本优化、反控与覆盖矩阵、消费/规范闭环。包内探针和阶段不单独取号。

全域盘点不等于改写全部测试；首批实施限语言仓测试基础设施与§5.3语料。UI/Man、旧VM/A2R和ACE产品代码保持各自owner。未交付能力、产品缺陷和跨仓测试重写不因盘点自动纳入。沿用873不细拆crate结论、现有分档/测试时序与ACE既有性能/资源门。

## 1. 目标

- G1：入口、子case、断言、条件、owner和正式档位可追溯，约定盘点闭包内未解释失联项为0。
- G2：完整义务分母和选择守卫可信，缺项、重复身份、未批准退役、零选择、局部冒充完整执行均失败。
- G3：复用现有TestCase和正式ACE路径，交付§5.3首批；旧侧适用检查保留，既有复用与新增消费分账。
- G4：同覆盖、样本和裁判条件下取得至少一项可复核的实际周转改善（或给出阈值不可达的机制级证据，不承诺整套倍数提速）。
- G5：补强弱负例、发现遗漏、身份漂移和执行报告完整性；未覆盖格与未执行项具名保留。

非目标：不重构VM/A2R生产算法、不拆crate、不改默认feature/API或默认引擎、不新增ACE能力/profile/wire/schema、不改预算；不把所有内部Rust测试改成黑盒，不另建执行器/PASS协议，不裁定语义争议，不自动修全部历史红，不重写UI/Man/OS或做全应用验收。不清共享缓存、不推送或改历史，不因本Plan反复跑整档。

## 2. 架构方案

沿用Design35与现行auto-test-case：协议核心负责发现/准入/比较/报告，适配器在`crates/auto-lang-core/src/test_cases/`，ACE经既有受控worker消费；期望仅在父侧，禁止发给worker。单向依赖、进程隔离、deadline和资源合同不变。

拟新增`scripts/test-governance/`，仅承载清单、映射、校验和成本编排，不实现语言执行或结果比较；`inventory.py`、`check_inventory.py`、`README.md`为拟新增路径，T-02冻结具体接口。原始枚举/计时仅本地保存，长期工具与反控放受管源码目录。`run-full-regression.py`仍是完整批量入口，不另造旁路完整门。

首批suite拟在`crates/auto-lang/test/cases/governance-887/`，引用已有source/oracle，不拷贝后失去身份。原M1 base/binding/能力suite不重生成，历史base数量不当当前完整分母。包装型入口遵§5.3，不能冒称原字节同源支持。

## 3. 技术栈

现有Rust/libtest/nextest、Python标准库、PowerShell验证入口、AutoDown TestCase、源/锁/worker哈希与既有性能收据。不增加测试框架、构建缓存后端或全仓覆盖率服务；行/分支覆盖仅作辅助，不替代语义、生命周期及真实路径证据。

## 4. 需求分析与背景调查

### 4.1 授权和读取基线

2026-10-29用户明确要求“按这个范围为测试治理建立正式计划，明确验收标准、首批优先接入ACE的用例范围，以及与884和ACE在途计划的交接边界。先立计划，暂不实施，不干扰其他会话”。本轮仅主检出创建计划、原子更新号源；不改在途合同、产品、canonical Spec或受管ledger，不提交他方WIP。没有实施、代理、跨聊天通信或push授权。

读取时语言HEAD `0023e21ca9eb3610a8c224f79c395c819649601f`、UI `d5c7b865c442d8f82a4afebb972f7f6d27bd1c89`、Man `e6cc06fbc496f366edde7929abfd49254ec13647`；多会话仍在写入，T-01重绑输入。884已有执行授权和实施提交，主计划头drafting 0/10滞后，不能按头判未开工；本Plan不代其owner改状态。871 executing r4，863/865 r2头drafting不证明无WIP。885为Blog Rust链包，886已预留VM poison包，均不认领；881方法已落现行M1 Spec，是否仍有写面/收尾按执行时真实树和收据判断。

| 来源 | 现状/约束 |
|---|---|
| `docs/specs/overview.md`、`docs/specs/auto-test-case/{project,architecture,profile-v2-native,ace-m1-conformance,a2r-golden-certification,plans}.md` | 现行父侧判定/身份/不缓存PASS；部分历史路径/数量/状态滞后，仅校正本次核实触面 |
| `docs/design/35-autodown-test-cases.md` | 已有全域分类和统一设计，本Plan是有限续作，不另立重大架构；历史文件统计/缺陷只是核查线索，不当今天实测 |
| `scripts/full-regression.md`、runner、`full-regression-*-{ids,selected}.txt`、`pre-split-universe/`、`split-{closure,golden-ledger}.py` | 沿用867完整义务与迁移/退役账目；list-only不证明执行，聚合子case须展开 |
| `full-regression-retirements-{844,869}.json.txt`、`tier-selection-minimums.json.txt`、`.last-batch-regression.json` | 880收据记录language 3202<3205、missing/suppressed各16；当次仅语言tf/tt/tb直接执行，不认证三仓全绿。当前runner仅加载844退役，与869清单脱节，仍须逐ID证明，不凭摘要降floor |
| 三仓`.cargo/config.toml`及nextest配置 | 语言t/tv/tt/tb、UI t/tu/ti/tn、Man t/tcore/tc/ti有独立选面；语言tu已删；根档不等于全部成员、integration和独立workspace |
| `crates/auto-lang/src/tests/{vm_file_tests,a2r_tests}.rs`、pilot/legacy-map.tsv | 聚合和ignored包装并存；any-Err弱判为历史线索；a2r实编门含外部依赖skip/known-broken，外层PASS不代表子case全绿 |
| 873、884、ACE波次交接合同、885/886 | 873未完成逐测试成本归因；884 H/I交界须释放；只消费正式交付ACE，不借WIP或改既定队列 |

内容SHA256：`architecture.md`=`e8b73a3af6756d7059e891c343d431a86b9e772f70091bf43e777e637a6dd5a7`；`profile-v2-native.md`=`a3b21891b605628e7cd92a41690d4a137dc9d310a349749381593b346fc708f8`；`ace-m1-conformance.md`=`aa5c8263b2a692028297727a24fa7dbec0b05c819d50346ff84e12942a6e4afc`；runner=`7c937da93a665182efc73e177d5a1d49b2e46c8982481eb5407d92f7fccbdab1`；minimums=`4f1a035321aa35a7c6f46282ee2db8ddfe4651ba0065b5ecf0ba3699d8c5d858`；884=`2034580bfa787b452dc1eeccd9c6271ec8df7e04b09d48919b06439b069b7e4b`。

### 4.2 有限盘点闭包

覆盖三仓当前受管第一方测试、fixture、suite、比较器、脚本和档位；语言全部workspace成员、显式exclude的ACE独立workspace、实际外部consumer入口、UI工具前端测试均登记。vendor/第三方、SDK冻结副本、生成物、缓存、本地运行证据排除或引用，不重复当第一方测试；归档附件仅作来源。OS/down等其他仓仅登记实际消费链接，不扩成全仓审查。

不以rg标记或文件数声明动态全集。T-01冻结受管基面和发现规则，真实动态枚举在G887后的权威树执行。宏生成、自定义聚合、脚本动态发现、条件目标须有发现器或人工逐项证明；不能枚举者记具名UNRESOLVED并阻塞AC-01。未执行可以具名保留，未发现/未解释不能算守恒。

## 5. 详细设计

### 5.1 G887、X887及实际交接

保守顺序为整体实施等U884，含建树、实施盘点、探针、测量和测试；本轮立项背景调查不构成开工。U887不是884、UI/Man结构包或ACE全线的新前置。原近期拆分里程碑仍按U884+后继UI/Man有限出口；具体认证缺口按对应门处理，不把整个测试治理追加进去。

| 写面 | 887权限 | 前owner/交接要求 |
|---|---|---|
| 新governance工具、suite、独立integration | 正式启动后单写；不覆盖他方同名WIP | U884正式基面核路径无占用后X887登记；未来唯一树`.wt/lang-887/auto-lang`、`plan-887-dev` |
| entry、stdlib、native注册、trans/rust、manifest/loader、公开SDK/生产算法 | **产品代码只读**，不为测试提速改行为 | 884及885/886相关owner；消费正式版本，必要产品修复回原owner或重规划，不写入本Plan |
| auto-test-case、core/test_cases、auto-case、公共测试注册/指纹 | H/I共享；仅必要发现/准入/比较/报告/适配与纯测试开销修复，协议语义不变 | 884释放其注册/指纹交界后逐符号核863/865/871及真实owner；默认只读，取得短窗才写，schema/profile变更needs_replan |
| binding.tsv、ace_m1_tests、原能力suite、expected、证书 | 默认只读引用，不改原required分母/词表/oracle | 863/865/871等H窗；优先新suite/integration，确需兼容改动逐文件交接；复用不计新增覆盖 |
| experimental/ac-*、auto-mir-lower、native CLI/worker产品和缓存/资源合同 | **排除写入**（S/N/driver） | 871切换/退役、865闭包、863宿主等原队列和G-COV/G-DELETE不变；仅消费已交付profile，不等无关整Plan |
| runner、迁移/退役/floor清单、cross-repo脚本、测试配置 | 本Plan治理写面，逐ID与用途证明后短窗写 | 核相关merge/批次owner已释放，不抢冻结批次；不持窗等待ACE，floor变化逐档说明，不代替ID守恒 |
| UI/Man所有文件 | 只读盘点和固定版本验证，不改测试/产品/规范 | 各仓owner；发现缺口登记入口与责任，不借主检出WIP。跨仓代码修改成为必需AC条件时needs_replan，不能静默延期 |

X887最小记录：前owner/Plan修订、释放文件/符号、已合入SHA、接口/profile/wire/源锁配置/指纹身份、887接受SHA和差额、消费者/受影响ID、剩余限制及证据失效条件、最终回交。“无重叠”须有真实diff依据；drafting或Git可合并不代表释放。只用已有计划/正式交付/受管交接记录，无用户授权不跨聊天发消息。

### 5.2 义务清单与可信分母

清单至少包含repo/commit/入口、harness或suite、子case/check、source/oracle哈希、断言强弱、profile/backend/真实起点、feature/平台/环境、正式档位、owner、选择/执行/结果及迁移/退役来源。义务、发现、编译、选择、执行、通过分别计数，skip/known-broken/未执行不算通过。内部Rust、随机/性质、性能/生命周期测试保原断言、seed和样本义务，统一索引不强迫统一格式。

仅源码相同、oracle/环境/断言/路径不同的检查不能合并义务；真重复可共享source/准备产物，映射仍完整。退役需已有授权和替代/零消费者证据，不能因未来ACE替代删除现役测试。

逐项校验844/869退役、841迁移及后续正式变化，拒绝重复、无来源、跨owner误用、漏映射与不适用tier。冻结baseline不为接受删除而再生；floor只是逐ID对账后的派生守卫。新增case也须正式选中，不能只保历史最小数。receipt绑定实际仓组合、配置、输入、选面和真实执行范围；失败后未到达的后续腿保持NOT_RUN。

### 5.3 首批ACE范围与覆盖分账

| 组 | 固定用例及oracle | 真实消费要求 |
|---|---|---|
| A 标量/控制流复用 | M1 B01–B12，值依次14/-21/42/42/42/0/30/7/120/42/7/-2 | AV与native既有已交付路径逐项消费；B08/B11旧侧差异保留，不能追随旧VM改期望；不计新增ACE覆盖 |
| B 边界/诊断复用 | B13 overflow、B14 div-by-zero、B15 rem-overflow、B16未执行分支查名、B17 bool/int、B18不可写、B19缺返回、B20实际执行短路RHS陷阱 | AV原已支持路径必验；native按正式交付状态，B17–19 pending差异不冒绿；旧i64/a2r差异保持，infra不当语言负例 |
| C 入口复用 | U01–U10：42、Unit、尾值42、Unit、trap与五类入口诊断，原entry-862 suite | AV entry-unit真实recipe；Unit不等于零；其他路径沿NA-4，不要求新增native/legacy能力 |
| D 旧语料新增ACE消费（必做） | `vm/01_basics/002_arithmetic/arithmetic.at`的`1 + 2`→3；`006_unary/unary.at`的`-2 * 3`→-6；`007_group/group.at`的`(1 + 2) * 3`→9；pilot ID为`lang.basic.arithmetic/unary/group` | 各新增≥1个ACE真实源码执行check，旧AVM适用检查保留。T-02证开工基线未接线，T-04新增；若前序已正式接入，保留事实并在同一基础整数族冻结等量尚未接入旧ID/独立oracle，禁止重算旧功；无法冻结则needs_replan |
| E 裁判补强 | §6 V-04..V-08八类控制与合法对照 | 验证本次治理真实路径，已有控制复用、新增断言分列，改名不算新增覆盖 |

A+B=20条、C=10条均复用；D=3条新增消费义务，不声称覆盖全部语言。引用原source/expected，既有M1参数、binding和oracle不改。emit、typecheck、build-run、AV/native与source/typed分别报告；emit绿不代运行。证书只按现行认证合同复用，不能将全部a2r实编检查改成快照。

D组优先用已交付脚本尾值入口保持原字节；T-02先证明三源真路径可行。如现行正式入口要求main，可仅作入口包装（原表达式原字节嵌入，不改运算/分支/实参），记录原source与包装/hash/行映射后真实执行；不得冒称原字节M1四路径或裸脚本支持。包装选择、合法语法和独立oracle在T-02冻结核验；需要新语言能力、预算或改算法时needs_replan，不以typed fixture替代。

本批不新增863 stdout/宿主、865闭包/HOF、871未交付profile、任意FFI/网络/GUI、多平台或重型自举能力；在全清单和覆盖矩阵保留owner/gap。复用项原性能样本数和资源合同不减，不要求重跑全部ACE能力suite。

### 5.4 成本归因与有限优化

T-02冻结两类以内实测候选，优先重复初始化、worker/工具准备、重复转译/构建和无意义等待。已有收据仅定位候选，不能直接宣布瓶颈。编译、链接、启动、setup、执行、比较/报告、teardown分相计量，不改产品加opcode/源码桩，不停用身份/隔离/截止/清理/资源断言。

拟定局部性能验收下限：同机器/源锁/profile/义务/输入/工具/并行条件，warm定向family每侧≥7个有效且全对样本；至少一个候选端到端P50降低≥10%、P90不增加超过5%，且波动/背景负载不足以解释差额。T-02先冻结样本总数N/顺序，所有预定样本必须有结果，失败或缺样本不能筛掉后补成绿；本项P50/P90按升序第ceil(0.50N)/ceil(0.90N)项，不更改既有ACE统计口径。分相、峰值内存、冷/温和预热/次序分别记录，不能把工作移到计时窗外。它不是整套提速承诺，不替代ACE开发期P90≤2000ms及其原样本数/端到端边界。

baseline在同权威树改动前保存具名产物、配置与哈希，顺序比较，不另建同步实现树、不借他方target/rlib、不清共享cache。需要独占测量时安排具体可用窗口，不占在途门/GPU；空闲检测不等于已取得窗口。两候选内不能达到下限则给证据needs_replan，不能自行以负结果豁免G4。

可选机制由实测决定：摊销纯初始化、内容身份绑定的构建产物复用、同义务重复准备消除。全局env/需要隔离的测试不可盲合单进程；不缓存PASS、不减少真正执行/断言/正式性能样本，不扩大ignore或缩filter。新性能验证为显式定向入口，不给每条日常测试加计时税。

### 5.5 覆盖补强和有限出口

矩阵轴：能力/语义义务×正常/异常/边界×后端/profile×source/typed/emit/build/run×fresh/hit/edit/restart（仅适用者）。首批每项有强oracle/真实起点/执行状态，其他格具名owner/原因，不用全笛卡尔积无限扩范围。弱负例强化保留原输入/判法、独立审计理由，不从被测输出自动更新golden；语义未知回用户裁定，不以多数引擎输出定正确。

V-04..V-08覆盖发现、强负例、warm重判、身份和receipt。基线确有盲点者至少一个旧判会漏/新入口能拦的真实反例；已修复者绑定真实修复/反控，不能伪造历史漏检。内部引用计数/借用/非法图及ACE增量/资源/会话专用断言保留，不能只留黑盒结果。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/governance-and-inventory.md | 分散入口→实际盘点闭包、子case身份、迁移/退役、成本口径 | 分清完整/局部证据 | AC-01, AC-02, AC-05 |
| SD-02 | modify | docs/specs/auto-test-case/architecture.md | 实际owner路径与治理层边界、已验证优化/失效轴；保单一裁判/隔离/不缓存PASS | 避免第二执行协议 | AC-03..AC-06 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原分母/裁定不变，补887引用/新增消费来源/适用起点 | 防包装/typed/快照冒领 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-test-case/project.md；docs/specs/auto-test-case/plans.md | 本范围核实的旧路径/状态校正及治理交付、剩余缺口 | 可消费正式版本 | AC-01, AC-07 |

SD均为proposal，work不发布canonical；review核最终事实，merge沉淀。`scripts/full-regression.md`随实际runner修复同步，完整门语义不放宽。

## 6. 测试设计

未指定模型/执行者，未授权agent；实施、自检、独立终审按未来指令在同一权威树顺序进行。internal pass仅可送审；final pass明确merge门pending。以下均为未来验证，不是本轮已跑证据。

| Case ID | AC/不变量 | 输入/反例 | 真实入口/可观察oracle | 覆盖边界/owner_stage |
|---|---|---|---|---|
| V-01 | AC-01/07 全域清单 | 独立workspace、聚合/宏、ignore/条件、SDK副本；漏一个已知入口 | 受管基面→发现器→真实枚举/人工映射，遗漏被校验拒，副本不重复计；UNRESOLVED关闭 | 约定三仓/ACE闭包，不证明全执行；T-01/07 work |
| V-02 | AC-02 逐ID守恒 | 合法844/869退役/841迁移；无来源、重复、跨owner、缺ID、suppressed、零选择 | runner的baseline_for/validate_ids/check_minimum及Python控制；缺口0，非法必红，不能旁路正式守卫 | 每受影响tier；T-03 work |
| V-03 | AC-03 首批行为 | A/B/C原source/profile；D三源3/-6/9 | auto-case/既有recipe→真实worker→父侧oracle，逐check执行/差异保持；新增ACE≥3，旧AVM保留，包装身份分列 | 30复用+3新增消费义务；T-02/04 work |
| V-04 | AC-04 八类控制1–2 | 必需source/oracle缺失；重复ID；合法同source不同oracle对照 | 发现/准入/治理真实入口：缺失和重复身份失败，两个合法不同check保留两义务 | 临时独立fixture，不改正式金样；T-03/06 work |
| V-05 | AC-04 控制3–4 | typed语言错误合法正控；fault worker超时/panic/坏响应 | 父侧judge按stage/code/matcher判语言负例；错stage/code必红，infra永不满足语言负例 | 复用既有fault控制，无新产品注入API；T-06 work |
| V-06 | AC-04 控制5–6 | 合法warm命中后改父侧oracle；再改source或worker身份 | 实际reuse仍重比，wrong-oracle FAIL；旧报告stale拒绝，无缓存PASS或少比身份轴 | 现行适用模式/身份轴；T-04/06 work |
| V-07 | AC-02/04 控制7 | subset、仅emit、后续腿未到达、ignore/外部依赖skip | 正式receipt读回只给实际范围，未执行NOT_RUN/gap，外层PASS不抹子case，emit不冒运行 | runner/语料报告；T-03/06 work |
| V-08 | AC-04 控制8 | D组数值3→4错oracle及合法对照 | 真实正常路径原值PASS错值FAIL，复位hash一致；expected只在父侧，worker输入不携带期望 | 与V-06分别验正常和warm；T-04/06 work |
| V-09 | AC-05 性能 | 两类以内冻结候选，BASE/NEW各≥7有效样本 | ≥1候选给出机制级证据（实测分相/计数/根因）；阈值达成或授权面内不可达负结果记录（2026-10-29 用户裁定），正确性/身份/清理/资源保持，分相与噪声证据齐 | 定向family，不重复整档；T-02/05 work |
| V-10 | AC-06 消费/隔离 | 原pilot/适配器代表正负、固定消费者、worker故障退出 | 触面定向检查及源锁/隔离/回收合同；协议不漂移、新红0，新integration真实进正式入口 | 实际受影响面，非全应用；T-07 work |
| V-11 | AC-07..AC-09 关闭 | 所有AC/SD/V、并集/收据 | 独立review重验，冻结门/合入/规范/ledger/归档/guard读回，未验不勾 | review/merge分阶段 |

未来真实命令（T-02补最终filter与选择数；新增文件明确为拟建）：

- 现有Python：`python -m unittest discover -s scripts -p test_full_regression.py`；拟新增`scripts/tests/test_test_governance.py`，用`python -m unittest discover -s scripts/tests -p test_test_governance.py`。
- 协议触面：`cargo test -p auto-test-case --test discovery_suite`，及实际受影响的`--test contract_decode`/`--test contract_v2`。adapter/root用真实枚举冻结命名filter，不以裸t/tt代定向。
- 拟新增`crates/auto-lang/test/cases/governance-887/first-wave.suite.ad`：`cargo run -p auto-lang --features test-cases --bin auto-case -- lint --suite <suite>`，list/run同CLI；run带`--out <本地独立目录>`，与source/oracle不相交，只用正式worker身份。
- 拟新增`crates/auto-lang/tests/plan887_test_governance.rs`：`cargo test -p auto-lang --features test-cases --test plan887_test_governance`；必须登记正式触面/批次选择，不能只手跑却被`--lib`永久漏掉。接线属于本Plan任务，不是本轮修改别名。
- ACE既有脚本只选实际触面scope/case；无安全定向面时复用精确命名case，不擅跑全套。UI/Man固定版本只按现有入口验证，不跑GUI/原生全档。

**merge-owned门**：final pass后Rust触面统一裸`cargo t`及实际触发的`tv/tt`；显式新integration、协议包及ACE独立命名入口不可被根t替代。仅脚本/资产遵AGENTS Category A，不机械加Rust门；tb/docs_gen/taa/语言已删tu不因Spec触发。runner完整批次验收属于merge，不在work/review跑。因为runner本身是候选改动，合入前必须用**候选冻结提交的runner**验收，不能误跑主检出旧runner：复用867已交付的`--frozen-trees` clean组合接口，`python <权威树>/scripts/run-full-regression.py tf --batch --frozen-trees --output <本地目录>`，`AUTO_LANG_ROOT`/`AUTO_UI_ROOT`/`AUTO_MAN_ROOT`绑定相同组的冻结正式树，三仓代码/配置clean、候选runner/hash/锁/选面全部入收据，机器级单实例串行。此为既有冻结接口的merge验收，不放宽日常tf主检出单实例纪律；不得另开第二个批次。合入后来源、依赖、配置、输入和选面不变时，按既有证据复用规则把这一批收据用于到期判定；并集改变才重审/重验相应面，不为仪式重复整批。G887不代表独占窗口已取得；required腿未执行不能认证完整批次，沿现行政策保留已裁定ui-tn手工pending/Windows限制，不扩大豁免、不称全环境绿。

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 约定闭包有文件/入口/子case/check/条件/owner映射，动态枚举与静态清单对账，UNRESOLVED/未解释失联0；未执行/skip/既存红具名保留，V-01有效，不以文件数充动态分母 |
| AC-02 | work | V-02/07与逐ID证据正确处置批次缺口；正式runner消费全部适用批准清单，原必验无未授权减少；缺ID/零选择/错owner/伪完整必红，不旁路、不重生成baseline接受删除 |
| AC-03 | work | A/B/C共30条复用按原profile实际判定并保差异；D三项新增ACE源码消费真实通过、独立期望及旧AVM可回放；既有/新增/包装分账，source/hash/逐check证据齐，未支持项不冒绿 |
| AC-04 | work | 八类控制全有合法/非法区分证据，覆盖矩阵可复算，新增与已存断言分列；无未裁定oracle修改，无typed/emit/外层PASS冒领源码运行或子case绿 |
| AC-05 | work | ≥1冻结候选给出机制级可复核证据（实测分相/计数/根因）；阈值（P50≤0.90×base、P90≤1.05×base、每侧≥7有效样本）达成则关闭，未达成须给出诚实负结果记录+根因+授权面内不可达论证（2026-10-29 用户裁定修订：负结果记录亦可关闭）；覆盖、正式样本、预算、裁判和清理保持，无窗外转移，不夸整套收益 |
| AC-06 | work | V-10及触面协议/消费者、新测试正式接线通过，新红0、触面新增warning0、fmt/diff clean、无debug/临时反控；产品写面排除，ID/强断言/源oracle只读保持 |
| AC-07 | work | G887/X887/回交齐，首批、完整性、覆盖、成本、限制和SD冻结；遗漏/延后/workaround扫描/KNOWN-DEBT归属清楚，合同缺项不转债冒完成，最终commit/tree clean可送审 |
| AC-08 | review | 同权威树冻结SHA独立终审AC-01..AC-07/V/SD/真实输入与交接；final pass明确merge门pending，必需面未验不pass |
| AC-09 | merge | §6冻结触面与正式批次验收完成，实际合入、canonical/ledger/index读回、归档及guard清理齐才U887；不把局部绿写全生态绿、不替他方关闭债或改里程碑 |

## 8. 执行步骤

10项=8 work+1 review+1 merge，当前全部未执行。所有实施准备受G887约束，写入受X887逐面约束。

| 状态 | ID | owner_stage | 依赖 | 路径/交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G887 | ✅ U884核验（归档10/10收据全pass，master tip=884收尾）；权威树 lang-887 @ 69d90c52f + 只读兄弟 auto-down c3464d0/auto-ui b00fd30/auto-man e6cc06f detached；885=产品面drafting无树、886=vm/codegen.rs已合入归档，均零文件交集；闭包/发现规则/867义务/floor冻结；X887记录就绪（拟新增路径无同名WIP，接受=U884基面无占用）；V-01首轮盘点+4项具名UNRESOLVED（vm_file_tests包装展开口径/a2r skip与PASS差异/独立workspace consumer数/ui-tn手工限制）；摘要落 attachments/887/t01-closure-freeze.md（commit 5c4df00fa） | AC-01, AC-07 |
| [x] | T-02 | work | T-01；X887相应面 | ✅ 清单/映射接口与发现规则冻结（inventory.py/check_inventory.py 字段+守恒规则，T-03实现）；first-wave.suite.ad=30复用（B01-B20原文件+U01-U10 entry-862）+3新增D组，D三源原字节消费决策（sha256与pilot legacy_source_sha256逐字一致，script-tail entry U03先例，兜底包装方案已设计）；成本候选冻结C1重复转译/C2 worker准备；命名filter=suite `governance-887-first-wave`+integration `plan887_test_governance`，正式接线设计齐；设计落 attachments/887/t02-design.md+suite文件（commit d4c98643b） | AC-01, AC-03..AC-05, AC-07 |
| [x] | T-03 | work | T-02 | ✅ runner退役通道修复（844+869双清单，跨清单重复/缺evidence拒）；权威树枚举 listed=3703/selected=3203，runner validate_ids PASS（missing=0/suppressed=0，880时16/16），check_inventory conserved:true；floor 3205→3189逐ID派生（869退役−16，理由落档）；新inventory/check_inventory/9项控制测试（ghost/missing/zero/suppressed/stale全red）9/9 OK+既有runner测试25/25 OK；摘要落 attachments/887/t03-runner-conservation.md（commit 72d357e49） | AC-01, AC-02 |
| [x] | T-04 | work | T-03；对应H窗 | ✅ first-wave suite（准入契约自包含修正）+ D组三 wrapper 源（原表达式字节嵌入 main，原路径/原sha= pilot记录/wrapper sha/嵌入行/独立oracle 映射记录）+ plan887_test_governance integration 6/6（D suite 真实 worker 执行 av exit 3/-6/9+scalar 全 PASS；30 复用经原 ace-m1-base + entry-862 suite 实测全绿；V-08 错 oracle→exit1 红/合法→绿；V-06 warm 重判+stale 拒+身份变更重判；原 vm 源逐字节未动）；worker 预构建+env 覆盖（AC_AV/AC_MN_CASE_WORKER）消除构建墙；fmt clean；commit 8c5a11a00/33ed3d2db | AC-03, AC-04 |
| [x] | T-05 | work | T-04；测量窗口 | ✅ 定向测量入口 scripts/test-governance/measure-first-wave.py；BASE 10 样本全绿 P50=1583.8/P90=1855.9（单 out 目录 warm 协议）；C1 store 内容身份复用实测 cache_hit 恒 0（worker session-less 强制 per-request 全新 store 并删除，av_probe_worker.rs:774-778）无收益已回滚；C2 worker 会话复用域外（experimental/ac-*=S/N 排除）+ session-serve 协议不兼容；机制级负证据落 attachments/887/t05-perf-negative.md，依用户裁定（plan_revision 2）关闭 AC-05/G4 | AC-05, AC-06 |
| [x] | T-06 | work | T-05 | ✅ V-04..V-08 八类长期控制落地 integration（plan887_test_governance 12/12 ok）：缺失源/重复ID→exit2、同源双 oracle 两义务保留、typed 诊断正控+错 code 红、warm 重判+stale 拒+身份变更重判、subset NOT_RUN 显式+emit 不冒运行、D 错值红/原值绿；覆盖矩阵（scripts/test-governance/coverage-matrix.json，-f 入库）+弱判差额记录（B08/B11 差异保留、B17-B19 pending、D 包装 provenance）；不批改 golden 不复制裁判；附件 t06-controls-coverage.md（commit 82cb1fdbd） | AC-02, AC-04 |
| [x] | T-07 | work | T-06 | ✅ V-01 关闭（inventory missing=0/unresolved=0，check conserved:true；四项 UNRESOLVED 逐一解决/具名保留；独立 workspace 202 测试文件/11 bins 枚举登记）；V-10 协议零漂移（discovery_suite 4/4+contract_decode 23/23+contract_v2 20/20）+固定 consumer 原样消费+new integration 显式接线；并集=26 文件全在 887 写面、与 885/871/886 零交集；fmt clean+测试目标 0 新增 warning；附件 t07-closure-wiring.md（commit 7aae86117） | AC-01..AC-06 |
| [x] | T-08 | work | T-07 | ✅ 全 AC/V 自检（AC-01..07 逐项证据映射，见 attachments/887/t08-ac-selfcheck.md）；未执行/既存红/债分列（merge-owned 批次/手工 ui-tn/既有红登记、B17-B19=语言裁定非 887 债）；SD-01..04 proposal 冻结（sd-proposals.md）；X887 回交（产品/experimental/UI/Man 零写入，diff 26 文件全在 887 写面）；成本与首批报告冻结；最终 commit 后 tree clean 可送审（commit e47ffb2e3） | AC-07 |
| [ ] | T-09 | review | T-08；冻结SHA | 同树独立审AC/SD/V/profile/选择账目，相关反例/必要check；findings同树修复重审，不触整档 | AC-08 |
| [ ] | T-10 | merge | T-09 pass；输入/机器窗口 | 并集核验→§6冻结触面及候选runner的一次正式冻结三仓批次→实际合入/证据适用读回→Spec/ledger/index→归档→fresh guard clean/remove/分支清理；输出限制/消费版本 | AC-09 |

未来权威实施/自检/独立复审均用`D:/autostack/.wt/lang-887/auto-lang`、`plan-887-dev`，同组UI/Man/down仅固定正式SHA只读消费，不借WIP，不建junction/symlink。本轮不建树。独立review期间停止写入，不建review副本；计划簿记在主检出。原始枚举/JSON/计时/日志/截图/探针仅本地保存，摘要记命令、退出码、选数/执行数、路径/hash；attachments只提交Markdown摘要。保留WIP，不reset/stash/全量add，不自动commit/push。

## 9. 复审记录

2026-10-29 / stage: new / plan_id: PLAN-887 / plan_revision: 1 / outcome: pass（合同静态预检，非实现验收/实施授权） / scope: T-01..T-10、AC-01..AC-09、V-01..V-11、SD-01..SD-04 / checks: 0–10章节唯一有序、10任务/9AC/11验证/4SD编号唯一且引用存在、各AC有任务承接、887活跃/归档唯一、号源888、既有语料/命令目标路径存在、围栏/行末空白与diff检查通过；阶段门和读写权限人工核对，无产品/在途合同写入 / authorization: 仅立项、暂不实施、不干扰他方 / next: 用户审阅；未来明确执行且U884成立后work T-01，写入另须X887；未建树、未运行Cargo/Python测试/性能测量、未创建agent或联系其他会话、未改canonical/ledger、未合入归档清树或push。

stage: goal | plan_id: PLAN-887 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-887/auto-lang | commit: 8c5a11a00 | next: work 继续（T-01..T-04 完成；T-05 负结果待用户裁定）

2026-10-29 / stage: revision（bounded，用户裁定） / plan_id: PLAN-887 / plan_revision: 1→2 / outcome: pass / scope: 仅 AC-05/G4/V-09 与 D-03 处置 / authorization: 用户明确裁定"修订 AC-05 为机制级证据+负结果记录" / delta: AC-05 与 V-09 增加"未达成须诚实负结果记录+根因+授权面内不可达论证亦可关闭"；G4 增加"或给出阈值不可达的机制级证据"；D-03 关闭 / next: work 继续（T-05 依修订关闭，T-06 起）

stage: work | plan_id: PLAN-887 | plan_revision: 2 | outcome: **pass → execution_done** | worktree: D:/autostack/.wt/lang-887/auto-lang | branch: plan-887-dev | base: 69d90c52f | code_commits: 5c4df00fa (T-01) → d4c98643b (T-02) → 72d357e49 (T-03) → 8c5a11a00/33ed3d2db/20598655a (T-04) → e200e85b1 (T-05 负证据) → 82cb1fdbd (T-06) → 7aae86117 (T-07) → e47ffb2e3 (T-08) | dependency_revisions: auto-down c3464d0a / auto-ui b00fd302a / auto-man e6cc06f（detached 只读兄弟） | task_ids: T-01..T-08 | evidence: T-01 闭包冻结+X887；T-02 设计冻结；T-03 runner 844+869、validate_ids PASS（missing=0/suppressed=0）、floor 3189、控制测试 9/9+既有 25/25；T-04 first-wave D suite 6/6 真实 worker（3/-6/9）+30 复用原 suite 全绿+integration 12/12（V-03/06/08）；T-05 负证据（BASE 10 样本 P50=1583.8、C1 cache_hit=0、C2 域外）；T-06 V-04..V-08 控制+coverage-matrix；T-07 协议 47/47 零漂移+并集 0 交集+fmt/warning 0 新增；T-08 AC-01..07 自检+SD 冻结+X887 回交 | blockers: 无（r2 修订经用户裁定） | merge-owned pending: 冻结提交裸 cargo t + tv + tt + plan887_test_governance 显式门 + 协议触面 + tf --batch 冻结三仓批次（候选 runner 验收）+ canonical/ledger/索引 + 归档 + guard/清树 | next_responsible_role: review（T-09，独立上下文同树冻结 SHA 复审）

2026-10-29 / stage: review / review_scope: final / reviewer: 独立 agent 上下文（general-1，同树冻结只读核验，turnCount 46）/ plan_id: PLAN-887 / plan_revision: 2 / outcome: **pass** / reviewed_commit: e47ffb2e3（修正附件后 a0ccfb812 仅簿记）/ base_commit: 69d90c52f / acceptance_results: AC-01..07 全 pass（逐项实证：check_inventory conserved:true/missing=0、retirements 844+869、floor 3189 理由落档、wrapper 原表达式字节+原 sha+oracle 对上、原 vm 源零改动、integration 12/12 复跑×2、协议 47/47、rustfmt exit 0、测试目标 0 新增 warning、SD/X887 与交付一致） / findings: R887-F1 LOW（diff 文件数 23→26 计数笔误→已落正）、R887-F2 LOW（§8 T-07/T-08 哈希笔误→已落正）、R887-F3 INFO（lib 基线 131/整包 fmt 预存漂移均非 887）、R887-F4 INFO（V-06 warm-oracle 轴由 v06+v08 联合覆盖；worker 身份轴由协议测试覆盖）/ pending_closeout_gates: 裸 cargo t+tv+tt、plan887_test_governance 显式门、协议触面、tf --batch --frozen-trees 三仓冻结批次（候选 runner 验收）、SD-01..04 canonical/ledger/index、归档+guard 清理 / next: merge（T-10）

stage: goal | plan_id: PLAN-887 | plan_revision: r2 | round: 2 | from: work | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-887/auto-lang | commit: e47ffb2e3 | next: merge

## 10. 待澄清事项

| ID | 事项 | 归属/关闭方式 |
|---|---|---|
| D-01 | 实施时真实输入和共享窗口 | T-01逐树/符号核对，不按drafting假定空闲，仅正式版本/X887，不等无关ACE全线 |
| D-02 | D三源尾值/包装可行性 | T-02完整路径与独立oracle冻结；不改算法/能力/预算，不可完成三项新增消费则needs_replan，不以gap代AC-03 |
| D-03 | 实测收益候选 | T-02有界两类调查，T-05达到AC-05；不能自行豁免阈值，失败具体重规划 | **已关闭（2026-10-29 用户裁定修订 AC-05/G4）**：C1 机制级不可达（worker session-less 强制 per-request 全新 store，cache_hit 恒 0）；C2 域外（experimental/ac-*=S/N 排除）+ 协议不兼容；BASE 10 样本全绿 P50=1583.8；修订后 AC-05 以机制级证据+负结果记录关闭（attachments/887/t05-perf-negative.md） |
| D-04 | 历史floor/ID差额合法性 | T-03原Plan/Spec/合入逐项证明，未知不降floor；真缺失恢复或回义务owner，不扩大产品写面 |
| D-05 | 必需AC依赖跨仓修复或语义裁定 | 给具体输入/owner/受阻AC并needs_replan；不擅改UI/Man/ACE、不静默后移合同义务 |
| D-06 | 正式批次窗口和既存手工限制 | merge安排可用时段，沿既定required/手工边界具名保留；局部证据不认证完整批次，不占他方冻结门 |
