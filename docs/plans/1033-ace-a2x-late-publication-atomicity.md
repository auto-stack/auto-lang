---
plan_id: PLAN-1033
status: execution_done
feature_name: ACE A2X暖编辑与晚发布失败一致性
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-16
plan_revision: 3
current_step: 3
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-a2x/incremental-and-cache.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1033] ACE A2X暖编辑与晚发布失败一致性

## 0. 变更摘要

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F25，优先级**P1**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：F25全部A-required，先关闭原AXTX-01/02与对应独立oracle再供985/A；A仍有非图形A2R/SDK与生成服务；共享resident/anchor/root发布失败会破坏其增量/恢复，不属于图形W3可延项。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

source_sha/resident/fileset/anchor/root在一个接受点切换；assemble/provider/anchor晚失败保旧，restart后的下一次读仍可用。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。不改语言/目标family或引入第二缓存；旧store字节是否升级由真实语义变化决定版本。

## 2. 架构方案

沿TargetProvider和ManagedStore准备/发布边界build-aside；允许无根可回收副产物，禁止误报磁盘零副作用；其他target居民和read-only行为保持。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1033/auto-lang，建议分支codex/plan-1033-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1033/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs) | eccd5ff0f35f982e9a90c923e47cf2f2f23e2f1db0df129017f562c2c6e3ced6 | 静态取材，非运行证据 |
| [experimental/ace/crates/a2x-core/src/target.rs](../../experimental/ace/crates/a2x-core/src/target.rs) | 046d0a9180d0dbf1b2cb0f5d88f3a5a9c2013e8b6712866b3e1c3114ad051774 | 静态取材，非运行证据 |
| [experimental/ace/crates/a2x-worker/tests/cases/store_cache.rs](../../experimental/ace/crates/a2x-worker/tests/cases/store_cache.rs) | 5fa20b7f5623104273b2e97fc7a686a82124ade813222f00aff181869466231b | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-a2x/incremental-and-cache.md](../../docs/specs/auto-a2x/incremental-and-cache.md) | c8bdda7ae802b12e2d4f1cb16801104f359247b256ce8abd67f025a42bc8657e | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1把C/Rust及三个family笼统写成都有anchor，未固定真正晚assemble/anchor失败注入；Rust record-text/collections当前没有source-bound anchor，不能硬造。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/a2x-worker/tests/cases/store_cache.rs | 5fa20b7f5623104273b2e97fc7a686a82124ade813222f00aff181869466231b | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-worker/tests/cases/store_coll_933.rs | 885743805f7a90588c13b9ffafc6628d6b8b11323cde6198e5f53e34b26e70b3 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-store/src/anchor.rs | 17bc1142a5fd5e29ec41d436f70a3947ca167c36b333cabb75ea372adea657f1 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F25。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：source_sha/resident/fileset/anchor/root在一个接受点切换；assemble/provider/anchor晚失败保旧，restart后的下一次读仍可用。 沿TargetProvider和ManagedStore准备/发布边界build-aside；允许无根可回收副产物，禁止误报磁盘零副作用；其他target居民和read-only行为保持。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** A沿store_cache::COLD：helper(x)=x*2，main=helper(20)+1，结果41；B只把helper替换为x*3，结果61。冷装配后保存manifest/files逐文件SHA、source SHA/居民fp/last_anchor及其他target居民；用实现TargetProvider的test delegating provider只在指定assemble_fileset次数返回A2xReject，其余descriptor/key/runtime/fragment行为完全委托原provider。

**状态、故障与责任。** source/resident/fragment输入、origin/anchor/环境接受态清单逐项记录；attempt统计及无根CAS残留可存在，不能误许磁盘零副作用。提交点必须跟实际root/anchor发布合同一致，任何已可达发布不可只回滚内存假装从未发布。若现store没有完成单接受点所需操作，T-01先给最小可行设计，不增加第二缓存或一概事务化所有磁盘。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F25 → 原AXTX-01/02 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：A仍有非图形A2R/SDK与生成服务；共享resident/anchor/root发布失败会破坏其增量/恢复，不属于图形W3可延项。关闭谓词：prepare/assemble/store/anchor任一晚失败，源SHA/resident/文件集/可达root不部分发布；restart下一次真实读/运行旧产物可用，修复重试成功；允许具名无根CAS副产物。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-a2x/incremental-and-cache.md | 现规则/支持域保持；补充以下修复后的明确合同：明确target×family有无anchor；暖编辑assemble/publish晚失败保现接受态，恢复后首次使用仍旧A；允许无根可回收副产物，不虚称磁盘零写。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-a2x ace-a2c-abi。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| AXTX-01 | AC-02 / T-02 | 错误provider在resident已准备后assemble返回Err；anchor写入失败 | A2xSession::warm_edit/warm_edit_ranged及既有store/restored入口 | 旧SHA/居民/anchor/manifest保持；后续run/restore与旧结果一致 | 已实施并运行（audit1033_axtx_01/_c_scalar/_c_anchored_families，红/绿见§9） |
| AXTX-02 | AC-02 / T-02 | 原parse早拒绝、正常编辑、不同target居民、read-only miss | 同入口，真实C/Rust文件集/consumer | 早晚拒绝分别记录；原缓存/身份/目标隔离不变；失败后同批重试成功 | 已实施并运行（audit1033_axtx_02/_02_boundaries，红/绿见§9） |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

同一次暖编辑内，先让合法B进入prepare/fragment生成，再让provider assemble_fileset返回Err；不能用parse错误替代。第二条在真实anchor publish点一次性返回store错误；优先现store配额/受控返回，若不稳定则T-01设计局部cfg(test)发布seam，证明已到达那一次写入。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| AXTX-01 | C/Rust Scalar | 两端均有AXP1，失败保A SHA/居民/manifest/anchor，restore A运行41；清故障重试B得到61 |
| AXTX-01 | C RecordText/Collections | 各有AXP2/AXP3；补资源/seed/required闭包及旧anchor恢复，不拿Scalar代全family |
| AXTX-02 | Rust RecordText/Collections | 保原无anchor合同，只核居民/源/assembly及相应restart行为；不新增anchor当修复 |
| AXTX-02 | read-only/其他target居民/早parse拒绝/同源重试 | 其他target不变；read-only不写store不等于会话不变；原A→B→A与quota正反控保持 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| AXTX-01 | audit1033_axtx_01 | [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| AXTX-02 | audit1033_axtx_02 | [experimental/ace/crates/a2x-worker/tests/cases/store_cache.rs](../../experimental/ace/crates/a2x-worker/tests/cases/store_cache.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

T-01落点裁定：真实provider链无法寄宿于a2x-store内部（a2x-rust/a2x-c依赖a2x-store，dev-dep成环），故测试锚实际落位 `a2x-worker/tests/cases/audit1033_axtx.rs`（接线于真实conformance根 tests/conformance.rs）；lib.rs落位承接staging/commit重构本身。同族函数：audit1033_axtx_01_c_scalar / _c_anchored_families / _02_boundaries（"一个Case可需多个同族函数"）。

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

本轮A阶段交接：AXTX-01/02的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。（§8 T-01证据：漂移重核、适用表、双seam、接受态/副产物清单、单提交点决定=可行；测试落位裁定记录于§6）
- [x] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。（红/绿：未修代码区分断言FAILED→修复后5/5 PASS；C标量/RT/COLL/Rust无锚/边界各有独立oracle）
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。（ace-a2x组全绿含全部既有消费者腿；ace-a2c-abi预存红具名归属；SD-01已在worktree准备；G1002不触及——未改新作者格式/adapter接线）
- [ ] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、实施授权（/goal 2026-11-16）及当前源 | 冻结target×family适用表、A41/B61、委托provider及anchor晚失败seam，列完整接受态/副产物清单并完成单提交点可行性决定。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合。证据：开工漂移重核仅lib.rs（PLAN-1042/976加性consumer注册/片段层，发布路径未动），其余5个钉源/Spec字节一致；适用表=anchors_family/lib.rs；A41/B61沿用store_cache冻结fixture；seam=①set_target委托provider（仅指定assemble_fileset次数具名拒绝）②下一锚键预置同键异字节冲突工件（store永不覆写→真实Conflict→受控a2x.resource）；接受态=source_sha256/当前后端居民集/居民id绑定/last_anchor/返回文件集，副产物=B sem/frag无根CAS行（允许），env非接受态（prepare候选事务性已核core/source/adapter.rs:1005）；单提交点可行性=可行（staging+末尾不可失败提交块，publish_anchor保持为store侧最后可失败操作）。测试落位裁定见§6。 |
| T-02 / work | [x] T-01决定与可区分输入 | 按现prepare/assemble/publish/swap边界修复，分开ranged/core/有anchor与无anchor路径，补AXTX永久反例。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合。证据：warm_edit_targeted改staged返回（WarmStaged：下一居民集+id更新+manifest next_sha override）；commit_warm=单一接受点；ranged路径删除source预切换（错误恢复块随之删除），锚rebind读staged居民、required边随被编辑片段真实artifact id（修stale边），publish_anchor后一次性提交source/居民/ids/anchor；冷路径assemble_view_with_contexts同样staged后提交；produce_fragment*返回(片段,Option<id>)，公开checked-fragment API签名不变。永久反例5个（§6表）红/绿验证：未修代码上audit1033_axtx_01/02 FAILED（重试store-hit/whitespace居民复用/stale-base三项区分断言），修复后5/5 PASS（2.85s）。 |
| T-03 / work | [x] T-02 | 每类失败后读取旧files/manifest、运行或restore旧A，再清故障同批重试B61；核其他target/read-only/原quota，完成A2X和C ABI职责组。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合。证据：restore旧A纯P0+字节一致并真实运行（Rust 41/C 41/RT 2217/COLL 44），清障重试运行61/2218/49且锚重绑next_sha；失败后stale-base探针证会话绑定仍旧；跨target居民存活（C居民+1复用且含A体字节，Rust命名空间restart回原A字节）；read-only零写零锚（非会话不变性照实断言）；早parse拒绝具名。固定组：`cargo t ace-a2x` 全PASS（含全部conformance腿与新增5锚，T-03在实际组结果核锚点被发现/执行）；`cargo t ace-a2c-abi` 8/9，唯一红=KNOWN-DEBT-AND-RISKS.md:4568已登记master预存红（admitted_text认证pin bd260a8b…vs 0a4f68e3…，master@82d032c8e/Plan 972改runtime未重钉，owner=批量回归triage），在干净基线复现一致，非本Plan触面——具名排除，不充PASS。原quota/A→B→A正反控由ace-a2x组内既有测试保持绿。 |
| T-04 / review | [ ] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1把C/Rust及三个family笼统写成都有anchor，未固定真正晚assemble/anchor失败注入；Rust record-text/collections当前没有source-bound anchor，不能硬造。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1033 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F25/原AXTX-01/02明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

2026-11-16 stage:work | PLAN-1033 r3 | outcome:pass | worktree D:/autostack/.wt/lang-1033/auto-lang @ branch plan-1033-dev | base 64ed621a319cc945feede259bf797b237088b89b（master） | code_commit 66f3c6bfbdd5e7f93914952f1fc357aefc0f9a43 | deps: 无跨仓依赖 | task_ids T-01/T-02/T-03（3/6，AC-01..03 [x]，current_step=3） | 授权：/goal 完成计划1033（2026-11-16，全链：work/review/有界修复/merge）。

实现（a2x-store/src/lib.rs，292行差）：warm_edit_targeted→WarmStaged（下一居民集+id更新，manifest经next_source_sha override绑定next身份）；commit_warm=唯一接受点（居民集+id绑定）；ranged路径删source预切换与错误恢复块，锚rebind读staged居民、required边随被编辑片段真实artifact id（修stale边：原实现边指旧片段工件，GC后恢复缺件），publish_anchor保持最后可失败操作，其后一次性提交source/居民/ids/anchor；冷路径assemble_view_with_contexts同样staged后提交；produce_fragment*返回(ResidentFragment,Option<artifact_id>)（命中/新产返回id，复用/read-only为None），公开checked-fragment API签名不变。关闭缺陷：①晚assemble失败留B居民+回滚A绑定；②锚发布失败留source=B/居民=B/锚=A；③ranged锚stale required边。

永久反例（a2x-worker/tests/cases/audit1033_axtx.rs，接线tests/conformance.rs，落位裁定见§6）：audit1033_axtx_01（Rust标量AXP1：assemble晚失败轴+锚冲突轴，A41/B61真实运行，重试=byproduct store命中，stale-base绑定探针，restore旧A纯P0+运行41，净命名空间重试61）、_c_scalar（C标量双轴+restore 41）、_c_anchored_families（C RT AXP2/COLL AXP3：种子锚完整、盲恢复字节一致+运行2217/44、重试2218/49）、_02（Rust无锚RT fragment路径+Collections ranged路径：居民保全探针、manifest next_sha、restart纯命中字节一致）、_02_boundaries（早parse拒绝/混合target隔离/C居民存活+Rust命名空间回原A字节/read-only零写零锚非会话不变/清障同源重试）。seam均真实：委托provider（set_target注入，仅指定assemble_fileset次数具名拒绝）；锚发布失败=下一锚键预置同键异字节冲突工件（store永不覆写，红跑错误记录existing/attempted双id证明到达该次写入）。

红/绿：lib.rs回退基线后最终测试文件上audit1033_axtx_01/02 FAILED（区分断言：重试store-hit、whitespace居民复用、锚冲突后stale-base）——修复后5/5 PASS（过滤跑2.85-3.31s，5用例并行，单用例远低于10s门）。计时：无>10s新增用例；本次会话测试模块=a2x-worker conformance（audit1033过滤跑3次~3s+组跑）+ace-a2x-c lib单测1次。

固定组（T-03）：cargo t ace-a2x = PASS（全部组含conformance全部腿，新锚在其中被发现/执行）；cargo t ace-a2c-abi = 8/9，唯一失败ace-a2x-c::admitted_text::tests::runtime_admission_preserves_bytes_and_the_certification_pin为KNOWN-DEBT-AND-RISKS.md:4568已登记master预存红（pin bd260a8b…vs实际0a4f68e3…，master@82d032c8e/Plan 972改a2x-c/src/runtime.rs未重钉；干净基线64ed621a3复现一致，本Plan未触a2x-c/runtime字节）——具名预存排除，不判本Plan红、不充PASS，owner=批量回归triage。G1002：未改新作者格式/document/IR/adapter接线，不消费。

Spec准备：SD-01已落worktree（docs/specs/auto-a2x/incremental-and-cache.md新增"PLAN-1033：暖编辑晚发布失败一致性"节：适用面/单接受点/锚required边/永久反例），review核收敛后由merge正式沉淀。健康：改动crate零新警告、cargo fmt已施（范围外fmtchurn已回退）、无遗留调试输出。

具名未决（交review裁债务归属，非本合同缺口）：read-only ranged编辑会提交未持久化居民文本，其后续同片段真编辑经居民复用发布的锚对该片段restore按键查miss（预存行为，本Plan前已存在，未修不扩，无测试触及；与失败原子性合同正交）。

next_responsible_role: review（T-04，独立上下文，冻结提交66f3c6bfb）。统一回归门保持pending，owner=final review/merge。

stage: goal | plan_id: PLAN-1033 | plan_revision: r3 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1033/auto-lang | commit: 66f3c6bfbdd5e7f93914952f1fc357aefc0f9a43 | next: review

## 10. 待澄清事项

不改语言/目标family或引入第二缓存；旧store字节是否升级由真实语义变化决定版本。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。
