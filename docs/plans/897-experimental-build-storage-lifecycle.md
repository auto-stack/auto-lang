---
plan_id: PLAN-897
status: drafting
feature_name: experimental构建存储减量与缓存生命周期
plan_revision: 3
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
current_step: 0
total_steps: 11
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/repository-boundaries/build-cost-baseline.md, docs/specs/auto-test-case/test-purpose-and-admission.md]
new_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
touched_goals: []
affects: [test-infrastructure, experimental-workspaces]
depends_on: [PLAN-895]
---

# [PLAN-897] experimental构建存储减量与缓存生命周期

## 0. 变更摘要

r3落实用户新增要求：experimental属于ACE路线，只能在实际涉及ACE的计划/验证中构建；普通新worktree只保约10MB源码，不生成ACE编译产物。根Cargo.toml已exclude experimental，缺口在测试调度/准备边界而非源码复制。

r2针对用户追问“这些probe究竟是什么、是否每次/每树都要编译、一次性探测能否雪藏”改正r1重点：先减少**不必要的编译**，再缩小必要产物。仅调整PDB不是完整解法。

实测三树experimental唯一数据125.06GiB，源码约10MB/树，唯一PDB72.92GiB。直接发现两个构建放大点：module_command.py::refresh在目录过期时枚举全部recipe，Rust发现经cargo nextest list触发构建；实际执行虽选中叶模块，recipe仍可传--workspace --lib --tests，nextest的用例过滤不等于Cargo只构建所需测试目标。新树冷缓存、普通代码变更触发目录全局失效时，容易先编全套实验工程。

核心方案：用途分层与雪藏清单 → 先选所需模块/依赖再发现/构建 → 同树兼容target复用与普通debug=0 → 逐工程清掉旧布局。跨worktree只读复用仅在来源/profile/triple/依赖及可搬迁性全部认证时采用；不具备条件的相关模块仍在本树构建。不同树继续保可写隔离。

本轮仅起草；未修改构建配置、未删除缓存、未运行项目测试或建立实施树。

## 1. 目标

- G6：以真实触面划分legacy/ACE构建域。无ACE影响的计划，发现/选择/准备/实际执行整条自动路径的ACE编译调用=0、ACE新编译产物=0；只在ACE源码改动、真实必要消费者/共享改动或显式ACE任务触发相关ACE闭包。不能仅按计划标题分类，也不能用手动关ACE绕过必要验证。
- G0：回答每个目录/内部模块是当前ACE实现、长期回归、专项验证还是一次性调研；目录叫probe不足以决定用途。
- G1：普通experimental dev/test明确debug=0，不改opt-level、debug-assertions、overflow-checks、panic等行为参数；显式完整调试保留debug=2。正式release/benchmark合同保持。
- G2：普通定向命令先确定所需模块与真实依赖，再更新相关发现/构建；目录过期不得无条件构建所有recipe。有效目录的--list/--select-only不为列名字编译实验引擎。
- G3：一次性、已完成且不再保护当前不变量的研究代码，保留源码与结论，退出默认构建/发现/执行；需要时用独立手工入口恢复。专项验证按相关触面/明确专项触发，不混入普通无关任务。
- G4：同树兼容依赖共用target，普通构建完成后真正减少存量；旧target须迁移清掉，否则仅改配置会保留旧数据甚至产生新旧双份。
- G5：相同源码及冻结的必需测试/实际依赖闭包，在完整构建并执行后候选稳态唯一数据相对原入口/原profile至少减少50%（r2提出的验收目标，未冒称用户已批准）。不达标回报原因，不通过删测试/减样本或测空目录达标。

非目标：按probe目录名整包停用；跨树共用可写target；复制/硬链接活动cache；改Golden或正式资源/性能门；重构root成员；改变Windows计量禁wrapper策略；接管在途890/894。plan-artifacts最近5个规则不套用活动构建缓存。

## 2. 架构方案

### 2.1 先分清代码用途

| 当前目录 | 已核实角色 | 默认处理方向 |
|---|---|---|
| ac-av-probe | ACE AV执行核心、驱动与worker，含大量长期语义/增量测试 | 当前实现保留；仅AV及真实消费者触面构建；内部历史研究与专项测试另分层 |
| ac-mir-native | MIR→Cranelift/COFF→PE的实际native后端及worker | 仅native/共享lowering及真实消费者触面 |
| ac-cte-probe | 经AV执行的编译期求值、冻结值与CTE worker；依赖AV/MN等当前实现 | CTE相关才准备其闭包，不能仅看代码叫probe就雪藏整个工程 |
| ac-artifact-probe | 当前编译工件/ManagedStore/cache载体，已被AV/MN/worker直接依赖 | 实现依赖不能停；内部调研/专项测试逐ID审查 |
| auto-abi-probe | AutoABI↔AutoABI/C双向原生互操作验证宿主 | ABI及依赖用例/明确专项触发，普通无关任务不构建 |
| ac-case-worker | 运行AC TestCase的worker，实际依赖当前compiler/backend | 选到对应TestCase才准备 |
| ac-core / ac-syntax / ac-semantics / auto-mir-lower | 当前ACE的前端、语法、语义与共享lowering | 按直接改动/真实依赖闭包构建，不因目录在experimental就全编 |
| auto-a2x | 新的A2X共享表示/Rust输出实验链 | 仅相应翻译链及消费者触面 |

895已给用途/日常准入**建议**，尚未改变默认执行。复用test-purpose-policy.json与daily-admission-proposal.json，不重新发明分类。T-01输出逐ID保留/专项/雪藏清单与依赖证据；只有当前r2后明确批准的ID清单可更改默认注册/准入，不能把“probe是否可雪藏”的问句当成同意停用所有probe。真正引擎代码依赖与其测试用途分别判断。

### 2.2 切断“为了选少量测试先构建全目录”

有效的全局登记仍保留测试身份/条件/归属；目录元数据与现成测试二进制不绑定为一回事。普通命令先从源/配置指纹与变更路径定位候选模块和真实依赖，复用输入未变的目录分片，只对失效部分重新发现。必要的实际发现与执行仍核对完整测试ID/原子/消费者，新增/删除/改名/cfg/宏及build.rs变化必须失效对应闭包；未知情况有解释地扩大，不能复用过期清单漏测。

Rust实际构建在可识别时收敛到所需package/lib/test/bin目标，而不是始终--workspace --lib --tests；运行过滤器只负责用例选择，不能冒充编译范围已缩小。显式全量目录发布/周期义务可发现完整已启用范围，但不属于每次定向任务的前置。新树与同源第二树的目录元数据可复用；必须运行的源码依赖在本树仍照常构建或通过严格来源/可搬迁认证的只读工具复用，不接受错版本可执行文件。

### 2.3 ACE构建域默认隔离（本r3首要交付）

**判定输入**：实际changed paths、显式模块/用例与真实消费者闭包。为recipe、测试目标、tool及跨域边登记所需domain，而不是仅依靠顶级ID前缀；ACE宿主测试有的在lang/、TestCase有的在case/，同一个Rust包也可同时承载两域。

**非ACE路径**：可信目录分片/源配置元数据 → 选择legacy相关模块 → 仅刷新相关分片 → 仅准备/编译legacy目标 → 运行并核绑定。ACE目录可以被读取为元数据，但不得触发其cargo build/test/nextest list、ACE worker/shim准备或以experimental为源码的编译。global source_identity变化不再使所有域目录一起失效；按真实相关输入计算分片身份。

**ACE路径**：直接改experimental/ACE用例，显式选ACE模块/专项，或真实必要依赖边证明共享修改影响ACE时，选择收据记录changed path→module→target→tool及开启ACE原因，只构建该闭包。全量/周期批次属于明确全域任务，在其所属主检出/专门窗口执行，不能变成每个非ACE计划worktree的隐含全量构建。

**宿主测试解耦**：当前crates/auto-lang/src/tests.rs把AV/CTE/MN及ACE语料接线绑定在通用test-cases上。新增默认关闭的test-ace测试feature/等价隔离target，ACE接线才启用；普通VM/legacy TestCase保test-cases可用。明确建立ACE宿主recipe，保原原子身份的迁移/分区对账，不把移出普通配置的测试当成删除或永久忽略。不改变公开Case协议的功能定义；具体实现采用独立target或feature中的可核方式，不能通过默认启用test-ace抵消隔离。

**构建前守卫**：发现器、prepare_tools、Rust目标构建与会自行补建worker的宿主helper统一消费本次域/工具清单。在非ACE范围内请求ACE manifest/worker/lib或出现未声明的跨域调用时，构建前失败并说明来源，不静默转全量、不悄悄跳过真正必要的消费者。范围不明先归因；若确实影响ACE，明确升级为ACE相关验证，而非靠计划标题豁免。

**直接验收场景**：干净非ACE树运行一个普通VM/语言模块；通用test-cases场景；非ACE源变化令目录失效；显式--list/--select-only；显式ACE任务；共享接口真实影响ACE；伪造legacy标签但请求ACE工具。前四项无ACE编译/产物；后两种合法相关任务正确开启必要闭包；伪造范围红。核实际子进程/编译argv与新产物，不以“未运行ACE测试”冒充“未编译ACE”。

### 2.4 缩小必须构建的产物

普通dev/test的debug=0配置必须覆盖每个独立workspace及实际命令生效值；不能只改根Cargo.toml。完整调试以明确入口/profile生成debug=2，单独列占用。正式release本已无完整调试信息时不重复声称节省。

同一worktree内沿用并扩展target/test-tools/{regular,batch}等已有命名空间，把相关Rust模块构建/发现/worker/shim入口接到兼容共享target；profile与target triple仍按Cargo规则分目录，features/锁/源码兼容由Cargo指纹及调用方来源校验共同守护。特殊不兼容分区必须具名，禁止留下所有workspace各自默认target的静默旁路。不能仅把多个大目录移到另一个目录后声称空间减少。

### 2.5 清旧缓存与防止再长回来

按owner窗口逐树、逐工程迁移：确认未处于活动构建/预留测量阶段 → 提取必要收据/正在使用的固定程序 → 记录并回收旧普通target/profile缓存 → 用新配置构建并实测存量。不要一边留着全部旧target，一边再完整生成新布局导致临时翻倍。当前890/894不受本计划自动清理，待owner实际交接。

新树只带源码；不预复制完整target。只为本次所需模块创建缓存；结束回收自有缓存，不把target转存plan-artifacts。失去当前用途的一次性研究target可删，源码与结论留档。主检出缓存有空闲清理入口，活动/测量保留要可识别，未归属证据先核查。

### 2.6 数量级与验收口径

当前125.06GiB中PDB约72.92GiB；仅按这部分作库存粗算，去掉后约52.14GiB，再叠加避免无关构建/同树依赖复用/旧缓存回收。但这不是已实测的交付收益：当前库存含历史配置，显式完整调试仍有成本。正式AC-06比较相同必需执行义务与真实依赖下，完整构建/测试完成后的原/候选稳态；至少减半为r2的拟定硬门，不把当前脏库存与候选空目录相减。

## 3. 技术栈

现有Cargo profiles/target-dir、Python test-governance、原生Windows Job、SHA256输入/工具身份；现有worktree/guard机制。新增清理入口定为scripts/test-governance/build_storage.py（新），用途分层优先复用895已交付侧表。不引入新的全局编译产物服务。

## 4. 需求分析与背景调查

### 授权与边界

2026-10-31用户指出experimental在主检出、890/894重复占用、累计约120GB，延续其磁盘长期不可持续的诉求。此次已授权/执行的是本地只读核查与拟定修复合同；本r3的ACE域隔离/构建范围/雪藏ID/profile/入口改动尚待用户确认。不得把之前对plan-artifacts的删除授权扩大为删除活动构建目录。

主检出按仓规仅计划簿记。本轮无build/test/清理；实施拟唯一树D:/autostack/.wt/lang-897/auto-lang，分支plan-897-dev。890/894仍在途；893/896可能涉及同一prepare/identity/选择入口，须按实际符号取写面，不把这些整Plan设为逻辑前置，不替他方停进程或清树。

### 已观察事实（2026-10-31，文件数据而非NTFS分配簇）

| 检出 | experimental唯一数据 | 其中唯一PDB | target外数据/受管源码 |
|---|---:|---:|---:|
| 主检出 | 48.59GiB | 27.24GiB | 10,440,311B，803受管文件 |
| lang-890 | 32.03GiB | 19.00GiB | 10,189,621B，772受管文件 |
| lang-894 | 44.45GiB | 26.68GiB | 10,293,869B，778受管文件 |

三树全局按device/inode去重为134,286,219,955B；PDB唯一合计78,299,488,256B。按路径相加总量更大（同树内部已有硬链接），不能把路径和当物理节省。os-051/ui-003的只读依赖树各仅约9MB且没有target，佐证Git建树不带构建缓存。

原始核查只在本地：C:\Users\zhaop\AppData\Local\Temp\experimental-storage-audit-20261031.json；SHA256=3d141db63d318417fd59c8ff2491e601f6b30236168b71d70a4d4c5dcc191ac8。未把原始清单/构建证据纳入Git。

### 当前依据与实际路径

- 代码基面观察433efce4e5f5a9e36dfde9c48ecb9a825f212d81；work开工须重核，不能消费890/894未交付WIP。
- docs/specs/auto-test-case/module-selection.md：895顶级/叶模块、原子守恒、实际消费者闭包、来源/profile/env身份；Windows计量禁用wrapper，准备/执行与有界Job保持。
- docs/specs/repository-boundaries/build-cost-baseline.md与已归档873：先实测、不因本问题拆root crate；既有无共享缓存计量与日常sccache态分列。
- .cargo/config.toml（SHA前缀11c09ea176623298）没有全局target-dir。scripts/new-wt-group.sh只git worktree add，不复制target。
- experimental/ac-av-probe/Cargo.toml（43c8cd77a8cdc52e）、experimental/ac-mir-native/Cargo.toml（e1e58592e563edd2）等是独立workspace；修改根Cargo.toml不能冒充已配置这些工程。
- scripts/test-governance/toolchains.json（97154ba0e3046d29）已将运行worker放到本树target/test-tools/{regular,batch}；scripts/test-governance/prepare_tools.py（037f41da96a5dc62）显式传--target-dir并绑定二进制hash。不能忽略这部分已合并的复用，也不能让路径重构抹掉batch隔离。
- Rust模块发现/单测及手工构建仍可在各experimental workspace下生成debug/test target；module_command.py（7be4ddc0e9effc11）与实际discover/run路径由T-01复核。
- 有硬编码shim消费者：ac-av-probe/probe/src/text_runtime.rs::trace_support_lib的test-support/target/debug/ac_trace_support.lib；ac-mir-native/src/collections_runtime.rs的显式target-dir；scripts/verify-ac-741.ps1、verify-ac-data-resources-774.ps1及对应真实消费者。路径改变须逐一承接，不能只把工具build跑绿。
- r3补证：根Cargo.toml:exclude已含experimental；crates各Cargo.toml未见实验工程直接依赖。ACE入口主要是测试/动态worker。crates/auto-lang/src/tests.rs中ACE宿主绑定通用test-cases。module_catalog.py::source_identity对本仓与兄弟仓广泛输入生成一个全局code身份；module_command.py据此全局refresh；module_catalog.py::select对global_inputs/unmapped fallback可扩大为全部modules。必须将目录新鲜度/不明触面与无条件全域编译解耦，保真正共享修改的必要覆盖。
- 新增直接原因证据：module_command.py::refresh对所有recipe追加--recipe；其main在source_identity.code失效时先refresh后select。recipe_runtime.py::list_command将Rust发现转换为cargo nextest list；recipes.json中10个ACE workspace普遍带--workspace --lib --tests。run_selected.py虽只运行selected模块，但Cargo构建目标仍继承该宽args。故“只执行少量用例”不证明“只编译所需目标”。
- docs/specs/auto-test-case/test-purpose-and-admission.md明确895建议层尚未改变默认：research可保结论/手工入口，special按触面，不按probe名称分类。已核ac-av-probe/probe/Cargo.toml及ac-cte-probe/probe/Cargo.toml存在真实当前依赖，auto-abi-probe/src/lib.rs自述是互操作验证宿主。
- target中同时存在验证收据/探针产物（如ac-verify-receipts）及CACHEDIR.TAG标记的构建中间物；按目录名全部删除不成立。

## 5. 详细设计

### 下一任务入口与实现约束

T-11与T-09作为优先实施路径：先落ACE构建域/宿主接线隔离，再将发现/编译接到选择后的真实域。不得以先改PDB/搬目录替代AC-11。

T-01可先只读核现行Specs/895实际交付及890/894/893/896写面；真正编译探针使用897自己的树和受控窗口。冻结表列每个workspace的普通/调试/正式profile、target来源、必需lib/worker消费者、不可回收证据及正在运行/预留测量状态。

普通dev/test明确使用debug=0；T-01核实所有实际入口生效且不改变其他行为参数，不再把“是否减掉全量符号”留作开放调研。保留显式全调试路径，不修改现行正式release/benchmark输入。profile、target命名空间、compiler flags、effective env变化参与构建及测试身份；旧绿不能无条件复用为新profile证明。

同树复用仅在目标triple、profile、features、锁与源码闭包兼容时成立，保原regular/batch隔离。由Cargo负责中间物指纹；实际发布的worker/lib在消费前仍绑定来源和hash，另一构建不得静默替换当前执行物。若shim需要兼容出口，可发布来源可核的固定副本；禁止链接进worktree和无来源复制。

清理默认预览，输出实际解析路径、种类、预计回收数据、保护原因。只操作明确本仓构建缓存/自有闲置根，拒绝reparse逃逸、tracked源码、未知目录及活动/计量预留。单纯没有cargo进程不代表两阶段测量已结束。target内必要证据先核复制及读取方；已有计划的来源绑定/首次失败记录不覆盖。实际清理与准备/执行采用互斥协调，失败/取消准确留收据，不强杀别的任务、不“清理成功”掩盖半删。

新worktree不预复制全target；结束按所属仓guard回收，不把可重建产物搬到plan-artifacts绕过收尾。主检出的周期清理只在其owner确认的空闲窗口执行；不能借本计划自动删除890/894缓存；按§2.5交接后逐工程清旧重建，防止新旧缓存叠加。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/build-storage-lifecycle.md | 未定义→普通/完整调试/正式构建分级、命名空间、来源身份与缓存/证据生命周期 | 避免每树重复数十GB及误删 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md | 现有模块守恒→补profile/路径变动的准备与实际消费者验证、保Windows Job与计量禁wrapper规则 | 不能靠缓存减量削弱选面或身份 | AC-02/03/05/09 |
| SD-05 | modify | docs/specs/auto-test-case/module-selection.md | 全局code失效可牵引全recipe构建→按域/分片身份选后发现，非ACE自动路径无ACE构建、显式跨域原因可核；宿主ACE配置与通用TestCase分离 | 避免每树无关ACE编译 | AC-09/11 |
| SD-04 | modify | docs/specs/auto-test-case/test-purpose-and-admission.md | 895建议层→将逐ID批准的日常/专项/雪藏决定落实到发现、构建与运行入口；历史身份保留 | 避免一次性探测长期拖入日常 | AC-09/10 |
| SD-03 | modify | docs/specs/repository-boundaries/build-cost-baseline.md | 既有873基线保持→另附本问题有界前后存储/成本与适用输入，不改历史数字或member结论 | 避免把清空缓存冒充收益 | AC-01/06 |

## 6. 测试设计

下表均为拟执行，不是已过证据。实现与内部自检同一897树；最终review独立上下文、同一树冻结SHA。未指定模型或新聊天/agent授权。

| case | AC / 不变量 | 实际入口/输入 | 必需观测/区分反例 | 边界与owner |
|---|---|---|---|---|
| V-01 | AC-01/06，真实存储 | 相同完整选面冷准备→执行→温态，原/候选普通profile | 源与case/特性分母相同，列unique/PDB/incremental/其余与总成本、候选稳态至少减半；不能删除产物后单测目录 | T-01/06，单轮不冒P90 |
| V-02 | AC-02，诊断能力 | 普通dev/test与显式完整调试两入口 | 普通不产出完整PDB；调试入口生成与PE匹配的符号；错误输入仍给原诊断/退出，既有实际case oracle不变 | T-02，正式release不改 |
| V-03 | AC-03，发布/来源 | AV/MN/shim实际消费者，regular/batch及两个不同源码/feature构建 | 正确配置实际运行一致；另一配置/错库或错worker不可被按原身份接受；输入改变旧收据失效 | T-03/05，不新增跨树共享 |
| V-04 | AC-04，生命周期 | 活动编译、两阶段计量预留、空闲缓存、含收据target、链接/越界/未知/只读共享文件 | 活动/预留/未迁必要证据保护；空闲可重建项真回收；链接不穿透、不chmod污染外部；中途失败不报全清 | T-04，有真实负控与合法清理 |
| V-06 | AC-09，构建按需 | 干净新树只选无ACE依赖的语言模块；同源第二树、普通无关源码变化、AV单文件变化、宏/cfg/新增测试、强制全量目录 | 普通命令无实验无关编译；只重发现失效闭包；无效元数据拒绝/按真实依赖扩大；显式完整目录仍守恒 | T-09/05，核实际Cargo/Rustc argv与生成目录，不仅运行数 |
| V-07 | AC-10，雪藏不误伤 | 逐ID批准的一次性研究、活引擎依赖、专项ABI、手工恢复入口 | 已雪藏研究不参与普通发现/构建/执行；手工入口可重建；活引擎与专项触面仍可正确运行；无批准不可停 | T-10/05，source ledger与真实命令双核 |
| V-08 | AC-11，ACE域隔离 | 干净普通VM树、普通test-cases、非ACE代码使目录过期、列表/只选；显式ACE任务、真实共享依赖、伪造域工具请求 | 普通路径ACE编译0/新ACE产物0，legacy验证真完成；合法ACE仅构建必要闭包；伪造/未声明请求在构建前红，必要消费者不被关掉 | T-11/T-05，真实至少一项冷树端到端+其余区分控制；允许只读元数据探测 |
| V-05 | AC-05，正式模块守恒 | 原cargo tm选面/消费者及受影响完整并集，含必要shim | 新旧发现/选择身份守恒，编译/执行/通过分列，不能只用list-only或父包装exit0；Windows Job/资源条件保持 | T-05/07/08，未执行条件明确 |

门禁：先由变更路径文件通过cargo tm --changes <paths> --select-only --out <本地目录>确定完整模块/消费者并集，再运行实际选面；新清理/配置路由控制要有真实消费者。准备/构建/发现路径变动先核实际调用范围，不能为了核选择器重复全目录；修改所有workspace profile将扩大实际受影响闭包，明确列出，不因磁盘优化抽样冒全绿。正式收益矩阵仍是独立里程碑，不新增三轮全目录或降低样本。895模块用途建议不构成本计划默认准入修改授权。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 |
|---|---|---|
| AC-01 | work | 三树来源/目标缓存/证据/实际消费者清单冻结；只读事实、假设和保护窗口区分，核查口径可重建 |
| AC-02 | work | 普通experimental dev/test实际生效debug=0（其他行为参数保持），显式debug=2完整调试真实可用；正式release/性能档、原错误诊断及适用行为保持 |
| AC-03 | work | 同树兼容命名空间复用已实证；必要不兼容分区逐项具名；所有实际worker/lib消费者正确，错来源/配置拒绝且身份随输入失效 |
| AC-04 | work | 清理有真实预览/执行/失败收据，只清自有明确闲置缓存；活动/预留测量/必要证据/源码/外部链接受保护；不改890/894现场 |
| AC-05 | work | 受影响完整模块及消费者/集成闭包守恒、真实执行通过；未执行/旧红正确披露；Windows Job及计量wrapper策略不弱化 |
| AC-06 | work | 相同必需执行义务/真实依赖完整构建并执行后，候选稳态唯一数据≤原配置50%；分列PDB/增量/其余与构建成本，非空目录收益；原输入/正式性能合同不改。不达标不得完成或擅自放宽 |
| AC-09 | work | 普通定向任务在选择前不再无条件发现/构建所有recipe；有效列表/选择读取可信元数据；按失效闭包发现并按所需Cargo目标构建；新树/源变化/未知反控验证，无漏选/失真分母 |
| AC-10 | work | 逐ID用途及雪藏清单有来源/依赖/批准记录；批准的一次性研究退出默认发现/编译/执行并有手工入口，活引擎不按目录名停用；专项按触面准备，原断言与手工恢复正确 |
| AC-11 | work | 非ACE计划的目录发现/刷新/选择/准备/实际验证整条路径，ACE编译调用=0、新ACE编译产物=0，legacy TestCase保持可用；ACE接线默认隔离，真实ACE/必要共享触面显式启用相关闭包，伪造/未知域不能静默全编或漏测 |
| AC-07 | review | 冻结SHA独立逐AC/V/SD复审、源码/配置/行为/生命周期反例和遗漏/workaround扫描通过，无未闭必需finding |
| AC-08 | merge | 冻结并集门、实际合入、SD文档沉淀/读回与归档、fresh guard/树与分支收尾、可重建缓存不转存证据；报告真实释放量 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户确认r3、895交付与实际符号写窗 | 核现行树/清单及原source/env指纹；同897树做AV+MN+shim有界可行性，冻结目录角色/逐ID用途与雪藏建议、实际发现→构建放大点、固定profile/namespace/cleanup ownership路由与V-01基线 | AC-01/02/03/06/09/10 |
| [ ] | T-11 | work | T-01 | recipe/module/target/tool域映射、按域输入身份，宿主ACE测试与通用test-cases分离；module_catalog.py/prepare与动态补建入口共享构建前守卫；V-08端到端零ACE编译及正负控，不以Plan标题代替真实闭包 | AC-11/09/05 |
| [ ] | T-09 | work | T-01/T-11 | 优先修module_command.py/discover_modules.py/recipe_runtime.py/run_selected.py的先选后发现/按Cargo目标构建，复用未变目录分片与身份；V-06实际冷树/错元数据/新增cfg测试，不靠运行filter冒少编译 | AC-09/05 |
| [ ] | T-10 | work | T-01逐ID清单与明确批准 | 用895侧表落实已批准research雪藏/special触发、独立手工入口与目录身份；V-07，不停用仍被AV/MN/CTE消费的代码 | AC-10/05 |
| [ ] | T-02 | work | T-01/T-09 | experimental各workspace普通dev/test配置、显式完整调试入口；候选脚本路径T-01具名；V-02与原适用诊断正负 | AC-02 |
| [ ] | T-03 | work | T-01/02/T-09 | prepare_tools.py/toolchains.json、实际discover/run与硬编码shim消费者的target/profile身份承接；V-03、regular/batch和新鲜绑定控制 | AC-03/05 |
| [ ] | T-04 | work | T-01、清理入口实际owner约定 | 新清理入口/生命周期规则及worktree收尾接线；V-04真实合法/拒绝/失败，不触在途890/894；仅owner空闲窗口回收主检出 | AC-04 |
| [ ] | T-05 | work | T-02/03/04/T-09/T-10/T-11 | 源路径选cargo tm完整模块/消费者，跑实际入口与V-05/负控；修齐后冻结 | AC-05 |
| [ ] | T-06 | work | T-05 | 同闭包完整单轮稳态存储/成本对照，V-01/SD-01..05与全部AC自检；正式性能档保持；execution_done | AC-01..06 |
| [ ] | T-07 | review | T-06冻结 | 独立同树逐AC-01..11/V-01..08/SD-01..05/健康与遗漏审计，必要修复回同树定向验闭，不重复无关全档 | AC-07 |
| [ ] | T-08 | merge | T-07 pass | 冻结受影响完整并集与到期批次职责；实际合入/SD读回/归档/guard清树，不转存target；实际回收统计 | AC-08 |

## 9. 复审记录

2026-10-31 stage:new | PLAN-897 r2→r3 | outcome:blocked（实施待确认；仅修订合同） | 用户新增明确目标：ACE目录只能在ACE相关计划中编译 | delta:新增G6/§2.3 ACE域隔离、T-11/AC-11/V-08/SD-05；普通测试与ACE宿主接线解耦、按域目录身份、构建前守卫，非ACE冷树实际验证零ACE编译/产物为硬条件；优先于PDB优化 | 证据:Cargo根已exclude，跨域源自全局身份refresh/全recipe/通用test-cases及动态worker准备 | checks:只读检查Cargo/测试接线/selection/prepare；未运行Rust或改实现、清目录 | next:确认r3后T-01→T-11/T-09，再按批准清单T-10及其余任务；r2以下为历史。

2026-10-31 stage:new | PLAN-897 r1→r2 | outcome:blocked（实施仍待确认，未变代码/运行测试/清理） | 用户纠偏：先回答probe用途/是否该构建/一次性研究雪藏，不能只报体积或把解法留给调研 | delta:明确debug=0与显式debug=2、同树既有regular/batch命名空间复用、逐工程先清旧再重建；新增核心先选后发现/按Cargo目标构建与逐IDresearch雪藏，T-09/T-10、AC-09/10、V-06/07、SD-04；AC-06改为同必需义务稳态至少减半的拟定硬门 | 新证据：refresh遍历所有recipe且Rust发现走nextest list，执行过滤仍继承宽workspace/test目标；895分类仅建议未默认生效 | checks:只读源/manifest/依赖与合同静态核对；角色表区分实际引擎和验证载体，未因probe名称停代码 | next:确认r2后work先T-01→T-09，逐ID批准后T-10；r1为下方历史。

2026-10-31 stage:new | PLAN-897 r1 | outcome:blocked（仅指implementation handoff等待用户确认r1，不是技术负结果） | authorization:用户磁盘调查诉求；本轮只读核查/计划簿记，未扩大为活动缓存删除或构建配置实施 | observations:三树125.06GiB、PDB72.92GiB、源码约10MB/树；当前存在890/894活动构建/测试 | checks:来源/路径/清单/选测Spec静态核对，未build/test/删除/建实施树 | next:用户确认r1后work从T-01起；T-01先定有界可行性与具体路由，不以简单全局target-dir代替身份/隔离设计。

## 10. 待澄清事项

- 当前合同r3待确认；仓规要求计划先审阅后执行。本轮回答方案与修订计划，不把问句当成启动所有源码改动/停测授权。
- ACE域默认隔离、按需发现/构建、debug=0、同树兼容复用及旧缓存迁移已是具体设计；T-01负责核实路由与反例，不再决定“是否要解决”。跨树预编译工具复用仅限来源及可搬迁性认证通过者，不能仅比较文件名。
- 一次性研究应雪藏的原则已进入r2；具体测试ID清单及原默认/周期/注册变化须明确批准，未批准前不静默停用。其余非雪藏选择优化须保持必需义务不变。
- 候选稳态至少减半是本r2提出、待确认的验收目标，不冒称用户给定硬盘预算；不通过清空target、降低样本或取消必要测试达标。当前125→约52GiB仅PDB库存粗算，不是交付实测。
