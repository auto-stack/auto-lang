---
plan_id: PLAN-832
status: reviewed
feature_name: auto-man 独立仓及三仓组合发行交付
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-25
plan_revision: 3
current_step: 10
total_steps: 12
supersedes_spec_components: [docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/auto-man/project.md, docs/specs/auto-man-core/project.md, docs/specs/auto-cli/project.md, docs/specs/tool-execution/project.md, docs/specs/tool-protocol/project.md, docs/specs/language-sdk/project.md]
new_spec_components: [docs/specs/repository-boundaries/man-three-repository-delivery.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-man/repository, auto-cli, tool-execution, repository-boundaries]
depends_on: [PLAN-809, PLAN-814, PLAN-815, PLAN-816, PLAN-831]
---

# [PLAN-832] auto-man 独立仓及三仓组合发行交付

> **r2启动锁：G832未满足前，T-01及全部准备/实施任务均禁止开始。** 必须等待831整Plan完整交付合入并收尾，不能只等其UI版本发布。 旧r1提前准备/并行规则已撤销；执行前先核验§5.1。

## 0. 变更摘要

完成R09并吸收R07 Man/CLI发行：管理核心/adapter、执行编排/client/发现、auto CLI及资源/测试/安装/CI承接到**既有auto-man仓**，保旧仓legacy与来源；语言仓迁出相应实现，三仓固定版本完成独立与组合消费。交付Core SDK/管理无UI与完整UI工具profile、包管理/构建/生成/服务/watch/升级恢复，而不只是复制crate目录或生成一个安装包。

全部准备和实施统一等待831完整交付合入及收尾（同时核实815/816完整交付）；G832满足后才从T-01开始，不与831候选/审查/清理并行。UI生产实现归831，不静态搬进Man；UiAst终态、兼容退役、大规模去重与远端发行不在本Plan。

## 1. 目标

- 管理/CLI有自有workspace、固定语言SDK与轻协议版本、完整测试/资源/安装与CI入口；管理Core无UI静态实现，UI经真实tool能力消费。
- 真正从auto-man独立clone、无UI源码的Core安装及完整三仓profile完成现行命令；不靠兄弟仓path/主target/未发布工具兜底。
- source/AST/Value/协议单源，锁/producer/二进制及资源身份可追溯；UI/语言仓不因管理迁移反向依Man实现。
- 两旧仓可恢复，跨仓切换/升级失败不丢旧可用交付与用户文件；真实删除语言仓迁出实现，三仓交付收据可重放。

## 2. 架构方案

遵循[两Plan共用合同](reports/repo-delivery-parallel-order-831-832.md)。Man仓承接auto-man-core、auto-package-lock/resolver、auto-toolchain-core、auto-man-assets、816最终管理服务/协议client/发现包与`crates/auto`；std-only `auto-tool-execution`为UI/Man双消费者的中立设施，留语言基础共享，不因逻辑域为man-ecosystem而物理搬入Man，以实际最终包名为准。旧auto-man兼容facet与纯管理服务分开，不以仍重导出UI的旧包宣称纯Core。

中立I-PROJECT/I-TOOL、值/AST/SDK仍由语言固定源提供，auto-api-gen等语言服务adapter经831去Man声明残边后消费；UI/上层auto-lang兼容与UI app归UI仓。协议只依单份轻数据，不复制到Man；UI完整profile安装匹配auto-ui-tool并通过已有client调用。三个仓内package DAG与正常/构建/开发闭包都要验证，而非把repo互相path引用勉强编过。

## 3. 技术栈

现有Rust/Cargo、809管理核心与各轻库、816 owned进程/协议client/服务adapter、815实际UI工具；784保全工具、777域边审计、792受管发行机制与现有CLI/服务定向设施。包版本渠道为本地固定Git commit/可重建镜像与内容寻址发行，HTTP远端/registry不作隐含前置，不另写安装/runner平行实现。

## 4. 需求分析与背景调查

**r3 有界合同修订（2026-10-25，用户裁定）**：用户在 T-09 唯一开放项后明确“jet/ark 真实构建环境缺失，暂时不测，等v0.7再考虑”。据此将 jet/ark 的**真实构建验证**自本 Plan AC-03/V-03 移出，转 v0.7 环境门控渠道；只延该两项构建，不延生成、vue/rust/vscode 构建、VM 运行、Core/full 安装、三仓独立交付、失败恢复或其他 AC。jet/ark 工具链在场且实际执行失败仍判红，未执行不冒称通过；v0.7 承接项登记 KNOWN-DEBT。其余目标、任务 ID、SD ID 和 r2 范围不变，T-01..T-08 与已有证据不因语义修订作废。

用户2026-10-22进一步明确采用保守串行方案：**815/816完整交付合入→831完整交付→832开始**。本r2取代r1提前准备/并行开工授权；目标、原任务/验收/规范增量ID和全部发行范围不变。本轮仅修订调度，未实施；[本轮来源](attachments/repo-delivery-r2-serial-20261022/inputs.json)。

用户2026-10-22授权建立本Plan及831，包含发行验收和并行边界。**本轮只规划，不执行旧仓/跨仓写入、切换默认分支、推送或安装替换**。未来work在授权范围内建立隔离候选/独立验证，本地默认分支实际交付由final review+冻结门后的merge按§5组织；若有WIP不能安全推进，不覆盖或reset。

当前依据：[Spec入口](../specs/overview.md)、[Man](../specs/auto-man/project.md)、[CLI](../specs/auto-cli/project.md)、[执行设施](../specs/tool-execution/project.md)、[协议](../specs/tool-protocol/project.md)、[SDK](../specs/language-sdk/project.md)、[旧仓](../specs/repository-boundaries/project.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)。总览有旧数字/聚合描述，以模块Spec+实际代码/版本收据为准。809/814已交付；815 A及816 A已发布，完整B尚未合入，主检出尚无816最终管理服务包，不能为未来包名伪造执行证据。

已观察：根Cargo与`crates/auto/src/{main,cmd_watch,cmd_service,cmd_ui,cmd_bp,cmd_autodesk}.rs`、`crates/auto-man`及core/锁/resolver/toolchain/assets、`auto-tool-{protocol,execution,client}`、`auto-ui-tools`/API-gen的声明残边。最终服务/CLI拆分写面由816实际交付重采，归本Plan生产搬仓，不接手其未完成findings。旧auto-man Cargo/CLAUDE已读，784 refs/WIP/ignored快照仅历史证据，本轮未重采git状态，T-01/T-02负责刷新。

输入SHA/hash/状态见[本轮输入](attachments/repo-delivery-batch-20261022/inputs.json)，旧仓恢复约束沿[784报告](reports/784-repo-preservation.md)。除本Plan管理/CLI归属范围不迁UI实现，不覆盖其他路线的未提交文件/分支。

## 5. 详细设计

### 5.1 启动门与下一任务入口

**启动门是整Plan硬门，覆盖T-01..T-12及所有准备工作。** U815/U816/U831均指对应整Plan全部约定验收与最终独立审查通过、merge-owned冻结门通过、实现/Spec/ledger实际合入、完整版本/来源交接收据读回且归档收尾闭环；必须查真实交付与归档记录，不能只看状态字样或勾选数。阶段A/B局部发布、某项pass、分支HEAD或仅T-11代码落地均不代替整项完整交付。

**G831 = U815且U816；G832 = G831且U831。** 前者未满足，831不得开始；后者未满足，832不得开始。等待期间只允许读上游计划状态、归档/合入与交付收据以核实启动门，或用户授权的计划簿记；不能开展本Plan T-01实质盘点、保全/恢复演练、fixture/driver编写、候选导入、测试、分支/工作树创建或认领。不得以“只读准备”“不改共享文件”或旧r1允许并行为例外。

启动时先在主检出核实G和完整源版本，将上游Plan revision、实际delivery commit、Spec/ledger与归档/交接收据记录到本Plan基线，再进入T-01。门未满足时保持drafting/current_step 0（如误建树则保全停止写入并如实报告，不自行删除），只报告等待上游，不标实施开始、不勾任何任务、不启动work agent。现有depends_on现在也是全Plan启动依赖，不只是终验摘要；这是Markdown合同/调度规则，未新增后端状态或自动检查能力。

G832满足后，T-01以831、815、816全部完成后的实际源码与交付收据为基线，新增attachments/832/contract-baseline.md：完整CLI命令/flags/机器输出/环境/依赖/寿命/支持矩阵及source/锁/安装入口，orphan不假新增。831共享声明与UI版本此时必须已实际交付，不能再以等待其未来切口为由提前实施部分832；必要新问题交owner，不借WIP。

### 5.2 数据流、安装及组合

| 规则 | 实际producer→consumer/状态 | owner/关闭条件 |
|---|---|---|
| C1 管理独立闭包 | 固定SDK+轻请求→管理核心/语言adapter→CLI；UI工具实现仅另进程 | 832；Core clone/install无UI源码/静态库；命令实际成功，锁/类型来源唯一，不用空stub替代管理能力 |
| C2 完整UI发现 | 固定831 tool/协议/资源清单→816真实discovery/client→UI生成/build/run | 832；首次开发/已安装/升级/重定位均用实际匹配tool；错误能力/版本/producer先于副作用拒绝，无大库fallback |
| C3 服务/执行退出 | CLI argv/cwd/env/stdio→owned设施→进程内VM/子进程/reader/watch→结果 | 832；保!Send VM线程亲和，正常/失败/取消/超时实际退出；service不杀占用进程、不假ready；仅清理本次owned对象 |
| C4 三仓版本/恢复 | SDK commit+UI release+Man候选+锁/二进制hash→三仓外部工程/安装→升级/切换 | 832；固定组合可重放；冲突/半安装/中断保旧或明确恢复、未全部verified不声称三仓交付 |

Core与完整profile按816冻结合同继承：Core不需要UI checkout/二进制即可执行约定语言/管理命令；完整profile包含匹配UI tool及其必需输入。分包并不许可默认UI消失、安装顺序靠用户猜、旧flags/JSON/exit漂移。`auto service`、`auto serve`、`req`及watch/HMR有不同既有角色，不能顺手合成daemon或扩大legacy端口强清策略。auto-down等外部依赖按profile固定来源及明确平台前置，不借任意主检出。

### 5.3 来源、切换与R07验收

T-04冻结U816真实源码与包列表导入Man旧仓候选，不在库中复制UI/轻协议；T-05建立独立workspace/锁/安装/CI入口与非UI对照，所有831共享轻声明接口必须已随其完整交付可消费；缺失则回查G832并停止受影响任务，不借其WIP。T-06固定831真正UI版本后完整三仓验证，T-07覆盖fresh/locked/重定位/依赖fetch/升级/损坏输入与受管失败恢复。

T-08在隔离组实现语言仓Man/CLI/管理执行编排原件删除，根成员/测试与CI/命令文档/777/792精确更新，实际默认分支推进归T-11。UI源码删除由831负责；轻基础留语言不因“迁出行数”也复制至Man。迁前后测试ID逐项映射、source与生成/第三方净统计分列，跨仓日常/触面/批量编排保持旧覆盖，不因根workspace成员删除而漏测。跨仓版本清单/安装说明与失败恢复记录必须可从目标仓找到。

发布复用792受管事务和已有安装机制；若CLI安装没有受管交换则以已承诺语义有界扩展，明确准备/校验/交换/收尾各失败出口，不承诺跨Git原子提交。目标Man提交先存在，再改语言消费/删源，错误时保旧可用版本/新提交恢复，不reset已有默认历史。U831实际交付是UI依赖，不授权回头修改其已发布语义；发现上游缺陷交owner修复发布。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/man-three-repository-delivery.md | 单仓工具→固定三仓组合/安装profile/source/版本/升级恢复及独立外部验收 | R07 Man+R09 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-man/project.md, docs/specs/auto-man-core/project.md | 实际管理库/服务/adapter仓属、旧compat及中立依赖与资源/测试owner | 独立管理交付 | AC-01,02,06 |
| SD-03 | modify | docs/specs/auto-cli/project.md, docs/specs/tool-execution/project.md | 816已交付薄CLI/client/发现与管理编排→Man仓安装；std执行设施留语言共享，保原角色与寿命合同 | 生产入口可用 | AC-02,03 |
| SD-04 | modify | docs/specs/repository-boundaries/project.md | 旧Man准备→实际保全/本地承接/来源与切换恢复事实 | 保历史 | AC-05,08 |
| SD-05 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | UI生成发行→Man安装与三仓固定版本交付、适用失败闭包 | 完整发行 | AC-03,04,06 |
| SD-06 | modify | docs/specs/tool-protocol/project.md, docs/specs/language-sdk/project.md | 中立单源/固定source和实际外部consumer接线；不另定义协议 | 依赖清晰 | AC-01,02 |

## 6. 测试设计

所有V项owner_stage=work，新增`test/fixtures/modularity/three-repository-consumer/`与跨仓安装/失败driver由T-03建立，T-04..09按实际最终source接线。复用legacy-preservation inventory/preserve/verify/recover-drill、777 strict、792 generated-ui-consumer与816真实CLI/owned进程fixture；不用fake覆盖最终UI正控。固定Git镜像/独立clone验收清AUTO根/邻仓/原target；T-01冻结真实包名/命令后使用目标Cargo check/包测试与实际auto二进制命令，不写虚构的新包已测证据。

| Case | AC / invariant | 输入/真实入口 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 Core闭包/单源 | 独立Man/Core安装，无UI源码/工具；normal/build/dev metadata，双source/漏轻包负控 | 真实语言/管理执行成功，单一SDK/请求类型、零UI静态实现/隐式path；负控受控红，不能仅cargo check |
| V-02 | AC-02 命令兼容 | 全挂载命令矩阵代表成功/错误/动态配置/源根/Unicode空格路径、flags/JSON/stdin透传；旧新入口对照 | 输出/产物/错误阶段和exit符合原合同，动态未知不静默当默认，UI/管理/语言各真实owner，无第二算法 |
| V-03 | AC-03 三仓UI/安装 | 真实auto安装二进制→831固定UI tool→生成/build/run；首次开发/安装/升级/重定位；坏版本/缺件/错能力 | 真产物/运行oracle通过且来源hash匹配；错误先拒绝副作用；Core不拉UI，完整默认UI可用，不静态fallback。r3：六profile真实生成+vue/rust/vscode真实构建+vm真实运行为必需面，jet/ark真实构建转v0.7环境门控渠道 |
| V-04 | AC-04 发行/失败 | 三仓clone fresh/locked→发行包A搬B，坏锁/hash/源/patch/半安装/取消/交换与回滚故障 | 固定版本可重放；失败具名非零，保旧交付/用户文件或实际可行恢复点，陌生目录不删，恢复后复绿 |
| V-05 | AC-05 历史/多仓事务 | 当前refs/WIP/ignored快照与bundle恢复、目标已落但消费者未切/切后验证失败 | legacy树/refs/保留字节可恢复；状态收据准确、重试幂等，不能只一仓落地就标全交付；不覆盖在途变更 |
| V-06 | AC-02,06 寿命/测试/CI | service vm/rust配置/port冲突/ready、serve/req、watch/HMR、child/reader超时取消，测试ID/适用CI入口 | 真资源退出/VM线程亲和，service不杀占用者/假ready，原测试选择不丢；CI本地执行与远端未跑分别披露 |

831/832严格按整Plan串行；与其他已获准路线的独立scope审查/修复不豁免本Plan启动门。每Plan同组每仓一权威树，冻结提交后不可同时改；共同反例集和影响scope证据按共用合同§5。所有work正反控与必要实机先完成；final review独立上下文验全AC/SD。merge在冻结三仓输入上统一裸cargo t+实际触面的tv/tu/tt等与目标仓适用门、串行窗口，tf到期主语言检出单实例，重型/前台OS资源预约独占。缺环境/缺上游的最终正控pending，不标整体pass。未指定agent/model，不创建会话；本轮纯规划无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 独立Man/Core真实管理消费，闭包无UI实现/隐式邻仓，模型/协议/SDK单源，无UI/语言反向依Man；V-01 |
| AC-02 | work | 旧命令/API/动态配置/参数/结果与服务/stdio/线程/进程寿命保持，完整profile真tool可用；V-02/03/06 |
| AC-03 | work | 固定831实际交付+SDK+Man的真三仓生成/build/run、首次bootstrap与独立安装闭环；r3：六profile生成+vue/rust/vscode构建+vm运行必需，jet/ark构建转v0.7；V-03 |
| AC-04 | work | fresh/locked/重定位、版本/资源/producer/升级与失败恢复正反控满足profile合同；V-04 |
| AC-05 | work | 旧Man新状态保全/恢复、全来源映射与多仓切换故障演练有效；V-05 |
| AC-06 | work | 语言迁出管理/CLI实现实际删除、测试ID/CI/777/792覆盖、命令/资源/许可/版本/SD与三仓handoff完整；V-06 |
| AC-07 | review | 独立最终review绑定r3/三仓冻结commit及实际UI release、全AC/SD/遗漏/健康审查pass，merge门pending |
| AC-08 | merge | 冻结门通过、目标Man与语言消费实际合入/三仓版本与来源读回、Spec/受管ledger、归档及guard清理齐备，M1实际交付可追溯 |

## 8. 执行步骤

12项；**本表每一行都先满足G832，包括T-01/T-02/T-03准备**。既有任务ID/AC/SD和工作内容保留；依赖列不能解释成允许跳过全Plan启动锁。

| 状态 | ID | owner_stage | 依赖/启动门 | 写面/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G832完整核验 | [✅ 已完成] [contract-baseline.md](attachments/832/contract-baseline.md) @`5d06fc94a`：G832/固定版本、41成员实际DAG、完整命令与flags、1214行实际help、锁/清单hash、安装/寿命/支持矩阵与测试CI基线；observed/inherited分列，收据差异及T-02..09输入具名 | AC-01..06 |
| [x] | T-02 | work | G832；T-01旧仓事实 | [✅ 已完成] [preservation.md](attachments/832/preservation.md) @`cecb7091d`：29 refs/零WIP/零stash/两项ignored重采；完整快照58c2185bab58ae63（两保留文件、bundle hash/31 refs）verify+recover-drill全过，恢复29 refs/23 tags/tree一致；唯一双仓树已建，旧默认未动 | AC-05 |
| [x] | T-03 | work | G832；T-01 | [✅ 已完成] [t03-verification.md](attachments/832/t03-verification.md) @`1cf2a9b97`：新three-repository-consumer+分阶段driver；预迁移10/0/3pending，consumer3/3+闭包PASS、自变负控与require-complete exit1；安装/故障/迁后未假绿 | AC-01,03,04,06 |
| [x] | T-04 | work | G832；T-02/T-03 | [✅ 已完成] [import-ledger.md](attachments/832/import-ledger.md) @Man`041043bbf`+lang`087c40c9e`：9包+6consumer导入，Man metadata 9成员；185项=165 same/10 manifest/10 hygiene、0缺失/0异常；中立/UI零复制、许可与旧legacy映射在案 | AC-01,02,05 |
| [x] | T-05 | work | G832；T-04 | **R2已核验：F832-R1-01 CLOSED**；[✅ R1修复] [repair-r1](attachments/832/repair-r1/report.md) @Man`ebfc88f8`+lang`0fd092bd`：独立Core workspace/lock/target复用同一CLI源；无UI/down fresh clone 3/3、安装SDK locked check与真实命令通过；Core 640 files/29包仅两仓来源，full 6608正控保持 | AC-01,02,04 |
| [x] | T-06 | work | G832；T-05；831完整交付来源读回 | [✅ 已完成] [t06-verification.md](attachments/832/t06-verification.md) @Man`a8373e66`+lang`b69da4dde`：full安装6608文件/37包/125path；client30+CLI7+compat33全绿；Vue生成/build/首帧、VM首帧、service ready与端口关闭通过，旧新失败阶段对照一致 | AC-02,03 |
| [x] | T-07 | work | G832；T-06 | **R2已核验：F832-R1-02 CLOSED；原受管事务证据保持，手工恢复文档残余归T08**；[✅ R1修复] [repair-r1](attachments/832/repair-r1/report.md)：Core删除全部原target后重定位通过；Windows PID4 AccessDenied/注入unknown具名保留，活跃保留、确认死亡回收；旧事务恢复证据按未变面保留 | AC-03,04 |
| [x] | T-08 | work | G832；T-07；共享接线窗口且831实际交付 | [✅ R2修复] [repair-r2](attachments/832/repair-r2/report.md) @Man`f52cd93b`：直接CLI以current_exe开发布局发现owner，永久测试无注入；生产七格+CI1/1通过。真实backup文档恢复后verify/旧命令通过，陌生extra仍拒绝，full永久driver通过 | AC-01,05,06 |
| [x] | T-09 | work | G832；T-08 | [✅ R2修复交接] [二次诊断](attachments/832/repair-r2/diagnosis.md)+[handoff](attachments/832/handoff.md)+[结果](attachments/832/repair-r2/results.json.txt)：连续失败覆盖调整已验证，F03/F04闭环，F01/F02保持；AC01..06 work面pass，T10/merge门pending | AC-01..06 |
| [x] | T-10 | review | G832；T-09冻结提交与真实831交付 | **[✅ final R3 pass]** [独立终审](attachments/832/review-r3/review.md) @Man`f52cd93b09401f358a89a19058fbf1fa7673fd16`+lang`e320abf4fa1ae2d00d3e2bc91a35eea53345f409`：F01..04全CLOSED；真实直接CLI/安装/显式/缺件及release布局、无主检出回退对照通过；文档PowerShell真实恢复verify/旧命令与严格extra负控通过；AC01..07/SD审查通过，**merge-owned统一门pending，非已合入/交付** | AC-07 |
| [ ] | T-11 | merge | G832；T-10 pass；共享合入窗口 | 并集影响复审，冻结统一门；本地Man分支与语言消费有序合入、三仓组合/版本/安装来源读回，新增three-repo-release.md | AC-08 |
| [ ] | T-12 | merge | G832；T-11全部交付收据 | canonical Specs/受管ledger与索引沉淀、归档；guard clean逐树清理本Plan临时树/分支/组，保legacy/恢复/实际交付分支 | AC-08 |

**G832未满足前不得创建/认领下述组、分支或实施会话；路径只登记未来执行位置。**

执行组`D:/autostack/.wt/lang-832/{auto-lang,auto-man}`，各仓分支`codex/plan-832-dev`；UI消费用831已交付fixed-rev clone/包，需附加只读UI树则本Plan该仓仅一份，不checkout831在途分支。必要auto-down兄弟按既有解析补detached，不改实现。已有784准备树/分支先核owner与保全后认领或另建本Plan分支，不强移；唯一树贯穿work/review/merge，不建第二审查树，无junction/symlink。主检出只簿记，逐树fresh guard clean才移除。

## 9. 复审记录

2026-10-25 / stage: work(G832 启动门核验) / plan_id: PLAN-832 / plan_revision: 2 / outcome: **G832 满足，进入 T-01；Plan drafting→executing 0/12** / gate_evidence: U815=r3 archived 17/17、delivery `33c9451fb` 为 master 祖先、merge 门/ledger v68/清理闭环；U816=r2 archived 14/14、delivery `ac7c1369c74eab41c9f1a6911d74e2029a772a12` 为 master 祖先、merge 门/ledger v76/清理闭环；U831=r3 archived 12/12、lang delivery `d7f378b676e3a9dac7257f609936455e5d8248fe` 为 master 祖先、UI delivery `aee499e87093ff727f68ac8467c506e9ecf6ecdc` 为 UI master HEAD、ledger v84/Spec 索引/归档/三树 guard 清理与 receipts-final 读回闭环 / baseline: auto-lang master `45558ab628bb29a8719f8467046b3ae0d5e36d7a`；auto-man master `51fafe82e8e0c4e66325ee14088eed8c3270e07e`；auto-down fixed `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` / worktrees: `D:/autostack/.wt/lang-832/{auto-lang,auto-man}`，分支均为 `codex/plan-832-dev`，门后新建且根/分支断言通过 / observations: 主检出既有他 session WIP 与未跟踪文件不纳入；`ui-release.md` 固定版本表的 lang `e1c64047` 实为历史 T-09 R6 提交而非当前 master 祖先，T-01 基线须以实际交付/收尾提交链核对并具名记录该收据差异，不据此替换已核验 delivery / next_responsible_role: work T-01

2026-10-25 / stage: work(T-01) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 1/12） / worktree_path: D:/autostack/.wt/lang-832/auto-lang / branch: codex/plan-832-dev / base_commit: 45558ab628bb29a8719f8467046b3ae0d5e36d7a / code_commit: 5d06fc94a8c4c06f5688cd913aa8695d268e2622 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8、Man=51fafe82e8e0c4e66325ee14088eed8c3270e07e / task_ids: T-01 / evidence: contract-baseline+实际1214行help+24/122/4测试选择面；cargo build -p auto 成功、metadata 41成员、版本/锁/清单hash读回；文档资产检查PASS、diff-check PASS；Category A未跑测试档 / blockers: 无；merge统一门pending / next_responsible_role: work T-02

2026-10-25 / stage: work(T-02) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 2/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: lang=5d06fc94a、Man=51fafe82e / code_commit: lang=cecb7091dcff2af206492e9ef2054042c64e615c / task_ids: T-02 / evidence: 当前29 refs/零WIP/零stash/两ignored盘点；完整快照58c2185bab58ae63 verify ok+recover-drill ok（29 refs/23 tags/tree=4017c1c84e5b66370953266765de2e7f6f968437）；首次参数覆盖造成的不完整快照已披露并弃用，完整快照ignored-keep=2；旧默认代码树未动 / blockers: 无；merge统一门pending / next_responsible_role: work T-03

2026-10-25 / stage: work(T-03) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 3/12） / worktree_path: D:/autostack/.wt/lang-832/auto-lang / branch: codex/plan-832-dev / base_commit: cecb7091d / code_commit: 1cf2a9b97b0ddf4155fc1ab56b259a88c701f891 / task_ids: T-03 / evidence: 新three-repository-consumer与three-repository-fixture；预迁移10 pass/0 fail/3 pending，consumer3/3、依赖闭包与shell入口PASS、命令/JSON/测试ID/固定源全过；自变负控PASS、require-complete exit1；pending不冒充安装/故障/迁后验收 / blockers: 无；merge统一门pending / next_responsible_role: work T-04

2026-10-25 / stage: work(T-04) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 4/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: lang=1cf2a9b97、Man=51fafe82e / code_commit: Man=041043bbf99ca698540e17fdfba89aa4a00e1ab6、lang=087c40c9e92796b2ccf1dc391a547547f8f4c981 / task_ids: T-04 / evidence: 9 package+6 consumer候选导入；metadata 9成员；185项源对账165 same/10 manifest/10 hygiene、0缺失/0异常；中立协议/执行/SDK与UI零复制，许可/legacy映射和完整hash入台账 / blockers: 无；独立构建/安装未在本任务冒称完成，merge统一门pending / next_responsible_role: work T-05

2026-10-25 / stage: work(T-05) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 5/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=041043bbf、lang=087c40c9e / code_commit: Man=2419e57e22b5539726416fe7d44e982096dcf63f、lang=e4332bfaf128d587636e9822672205fbf42b1644 / task_ids: T-05 / evidence: 独立锁与normal/build/dev Core闭包；管理库252+CoreCLI18+外部consumer23全过；Core安装643文件/29包/66本地path、安装SDK locked check与脚本/脚手架/trans/JSON/UI负控通过；资源262 same/3 hygiene/0 drift / blockers: 无；full UI归T-06，merge统一门pending / next_responsible_role: work T-06

2026-10-25 / stage: work(T-06) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 6/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man,auto-ui} / branch: Man/lang=codex/plan-832-dev、UI=detached fixed / code_commit: Man=a8373e660f47ce1c0e90722c5240359155b81409、lang=b69da4dde32d1e7226f836e4086c3ebe03ad7ba1 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-06 / evidence: full安装6608文件/37包/125本地path；client30+CLI7+compat33全绿；Vue生成/build/Playwright首帧、VM MCP首帧/select_rect/退出、service ready200与4173/4180关闭；旧新失败阶段一致 / blockers: 无；merge统一门pending / next_responsible_role: work T-07

2026-10-25 / stage: work(T-07) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 7/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=a8373e66、lang=b69da4dde / code_commit: Man=3f85c634448a642b264a684d896f9e5bbe18356b、lang=50f49d575128282682d70e59ef314bfea5f435a0 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-07 / evidence: 最终独立发行矩阵14/14（fresh/locked/重定位/第三方/升级/失败恢复）；真实取消+死亡staging清理+活跃staging保留；swap回滚与每失败verify通过 / blockers: 无；merge统一门pending / next_responsible_role: work T-08

2026-10-25 / stage: work(T-08) / plan_id: PLAN-832 / plan_revision: 2 / outcome: pass（任务域完成；Plan 保持 executing 8/12） / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=3f85c634、lang=50f49d575 / code_commit: Man=356f2c8186d07bc18567b18179c3cad7bda2d483、lang=ba1ce5901d76643a8878f5d0f4663c3ca4904ea5 / task_ids: T-08 / evidence: 9包+6fixture实际删除、32成员locked metadata、切换consumer/check/测试ID全过；777 strict 0v/4gf与真实资产20/410397B、CI323目标、792 37/37；jet/ark缺工具链仅生成，未冒称支持面完成 / blockers: jet/ark真实构建环境缺项待T-09记录；merge统一门pending / next_responsible_role: work T-09

2026-10-25 / stage: work(T-09) / plan_id: PLAN-832 / plan_revision: 2 / outcome: **blocked（AC-03 jet/ark 构建环境）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=356f2c81、lang=ba1ce5901 / code_commit: Man=356f2c8186d07bc18567b18179c3cad7bda2d483、lang=6fbe483218fa147edab1b4a4fc5b91e8250ed870 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-01..T-08 completed；T-09材料 / evidence: [handoff](attachments/832/handoff.md)+[spec-delta](attachments/832/spec-delta.md)+[最终来源](attachments/832/t09-source-audit.json.txt)：185迁移文件均在Man且lang全删，SD-01..06 canonical/61项索引，全部V除jet/ark构建外有实测；AC-01/02/04/05/06有证据，AC-03 partial / blockers: 本机无Gradle/Java/ANDROID_HOME与hvigorw/DEVECO，jet/ark仅生成而真实构建未通过；832无831 r3延期授权，需工具链实测或用户正式修订合同 / next_responsible_role: 用户裁定环境或合同；Plan保持executing 8/12，T-09不勾、T-10不启动、不合入

2026-10-25 / stage: new（r3 有界合同修订） / plan_id: PLAN-832 / plan_revision: 3 / outcome: pass（用户正式批准延期，允许按修订范围继续） / changed: AC-03、V-03、AC-07、§4/§10、SD-01..06、KNOWN-DEBT / decision: 用户指示“jet/ark 真实构建环境缺失，暂时不测，等v0.7再考虑”；仅jet/ark真实构建转v0.7环境门控，六profile生成、vue/rust/vscode构建、VM运行及其他AC不变，未实测不冒称通过 / next: work T-09 按r3收口并交T-10独立终审

2026-10-25 / stage: work(T-09) / plan_id: PLAN-832 / plan_revision: 3 / outcome: **pass（work面完成，execution_done 9/12）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=356f2c81、lang=ba1ce5901 / code_commit: Man=356f2c8186d07bc18567b18179c3cad7bda2d483、lang=cd603e418accde3bb85327cc1b6affd49b2ce93f / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-09 / evidence: handoff+spec-delta+最终来源审计；全部V按r3必需面通过，SD-01..06 canonical与61项索引，185迁移文件及支持资产/生成物/第三方分列，独立发行14/14与792 37/37；AC-01..06完成，AC-07/08 pending；merge统一门pending / blockers: 无；jet/ark经正式延期非实测通过，v0.7承接在案 / next_responsible_role: review（T-10独立终审；非merge/交付）

2026-10-25 / stage: review / review_scope: final（T-10 R1） / reviewer_identity: 当前Codex聊天独立重建，未参与本轮实现；未创建agent/第二权威checkout，未虚构模型/会话ID / plan_id: PLAN-832 / plan_revision: 3 / outcome: **needs_fix（executing 5/12）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / reviewed_commit: lang=cd603e418accde3bb85327cc1b6affd49b2ce93f、Man=356f2c8186d07bc18567b18179c3cad7bda2d483 / base_commit: lang=45558ab628bb29a8719f8467046b3ae0d5e36d7a、Man=51fafe82e8e0c4e66325ee14088eed8c3270e07e / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；四树进出HEAD/clean一致 / spec_inputs: [baseline/锁/Spec/技能hash](attachments/832/review-r1/baseline.json.txt)、[冻结SD](attachments/832/review-r1/spec-delta-reviewed.md)、[实际规范diff](attachments/832/review-r1/spec-delta-reviewed.patch.txt) / acceptance_results: AC01 fail、AC02/03 partial、AC04 fail、AC05 pass、AC06/07 fail、AC08 merge-owned pending / findings: **F832-R1-01(P1)** Core关闭UI特性仍要求邻仓UI manifest，Core安装无条件读取UI/down，四仓clone正控未覆盖无UI独立来源；**02(P2)** Windows OpenProcess权限拒绝被当死亡，真实System PID4/GetLastError5反例误删合法staging，活控/死亡控成立；**03(P2)** Man ti/跨仓入口未接UI tool/lang te-child，清AUTO后真实gen定向选1败1，显式UI tool对照1/1；**04(P2)** 目标README仍指导am/根cargo install（真实virtual manifest exit101），旧release脚本及固定版本/profile/恢复入口未承接 / evidence: [完整R1/关闭oracle/AC-SD矩阵](attachments/832/review-r1/review.md)、[真实反例和对照](attachments/832/review-r1/results.json.txt)、[定向Rust/边界](attachments/832/review-r1/targeted-results.json.txt)；185检出hash全部吻合（两.at Git LF与WindowsCRLF口径单列，无语义丢失）、Core/full verify及Unicode脚本通过、runtime2/2、777 strict0v4gf、自测通过、独立bundle恢复29refs/23tags/tree一致 / pending_closeout_gates: final pass后merge-owned冻结语言裸t/实际触面档、Man适用t/ti及consumer/UI门、有序合入/来源读回/Spec-ledger/归档guard，tf仍主检出到期批量；本轮均未执行 / next: 同树work一次修齐四finding，定向反例/合法对照后提交新SHA、更新SD/handoff再做受影响复审；**T05/T07/T08/T09重开，T01/02/03/04/06完成保留，T10未通过**；jet/ark r3延期继续有效，不要求重测该环境面。未修实现/合入/发布canonical或ledger/归档/清树。

2026-10-25 / stage: work（R1 needs_fix 修复） / plan_id: PLAN-832 / plan_revision: 3 / outcome: **pass（F832-R1-01..04 work修复完成，execution_done 9/12）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=356f2c8186d07bc18567b18179c3cad7bda2d483、lang=cd603e418accde3bb85327cc1b6affd49b2ce93f / code_commit: Man=ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f、lang=0fd092bdccf91902bd7744b4024929b63ddd1c4b / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-05/T-07/T-08/T-09 / evidence: [R1闭环报告](attachments/832/repair-r1/report.md)+[机器结果](attachments/832/repair-r1/results.json.txt)+[handoff](attachments/832/handoff.md)+[SD](attachments/832/spec-delta.md)：无UI/down fresh Core 3/3+重定位1/1、Core640/full6608安装、原CI反例1/1、full driver7/7、tcore23/23、discovery6/6及client/helper、真实AccessDenied/live/dead/unknown控制、目标文档与退役release全部通过；未变证据复用边界在案 / blockers: 无；完整统一回归/落地/ledger/归档/guard为merge-owned pending，未在本repair执行 / next_responsible_role: review（T-10受影响独立复审；非merge/交付）

2026-10-25 / stage: review / review_scope: final（T-10 R2，受影响复审） / reviewer_identity: 与R1同一独立审查聊天，未参与实施或修复；未建agent/第二权威checkout、未虚构模型会话ID / plan_id: PLAN-832 / plan_revision: 3 / outcome: **needs_fix（executing 7/12）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / reviewed_commit: lang=87f82d637ecd586c17a01c1c975e1456bc346991、Man=ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f / base_commit: repair-base lang=cd603e418accde3bb85327cc1b6affd49b2ce93f、Man=356f2c8186d07bc18567b18179c3cad7bda2d483 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；进出四树HEAD/clean一致 / spec_inputs: [baseline/锁/输入](attachments/832/review-r2/baseline.json.txt)、[冻结SD](attachments/832/review-r2/spec-delta-reviewed.md)、[实际候选diff](attachments/832/review-r2/spec-delta-reviewed.patch.txt) / findings_closure: **F832-R1-01 CLOSED**（独立Core workspace/lock/同源target、无UI/down metadata/640安装verify/Unicode脚本/真实new命令，当前Core check通过；fresh/冷构建/重定位按输入等价复用），**F832-R1-02 CLOSED**（实际PID4 unknown具名保留、live/dead/注入unknown均通过）；**F832-R1-03 PARTIAL/OPEN(P2)**（原CI测试1/1及helper1/1通过，但生产CLI清CARGO_MANIFEST_DIR/AUTO后exit1仅找Man邻接target；仅加tool/manifest/UI根各0并产App.vue，安装邻接0、显式缺tool1；测试先resolve再注入路径掩盖真实入口）；**F832-R1-04 PARTIAL/OPEN(P2)**（首页/am/release退役已修；真实Core升级backup按新文档移回后verify exit2 extra .man-backup.marker，移走唯一marker其余不变即640/exit0） / acceptance_results: AC01/05 pass，AC02/03/04/06 partial，AC07 fail，AC08 merge-owned pending / evidence: [完整R2/闭环/二次覆盖诊断](attachments/832/review-r2/review.md)、[实际生产发现六格对照](attachments/832/review-r2/discovery.json.txt)、[Core/owner/备份往返及定向结果](attachments/832/review-r2/results.json.txt)、[fixture字节等价与真实管理](attachments/832/review-r2/supplemental.json.txt)；原始错误探针的ui/front目录错误已单列排除，以合法src/front重新完整对照；无语言生产diff、UI/down/其余有效证据按未变输入保留 / pending_closeout_gates: final pass后merge-owned裸t/实际触面、Man t/tcore/ti及consumer/UI门、合入/来源/Spec-ledger/归档guard；tf仍主检出到期批量，当前未执行 / next: 同树work只修F03/F04剩余范围，先记录两次final非通过的覆盖诊断和oracle调整、重放实际CLI/文档恢复正反控，提交新SHA并更新终稿后受影响复审；**T05/T07保留完成，T08/T09重开，T10未通过，current_step7/12**；不得机械重跑已闭Core/owner、六profile/全发行，jet/ark延期保持。未改实现/合入/canonical/ledger/归档清树。

2026-10-25 / stage: work（连续两次 final needs_fix 覆盖诊断） / plan_id: PLAN-832 / plan_revision: 3 / outcome: diagnosis recorded before R2 repair / diagnosis: [二次失败诊断与oracle调整](attachments/832/repair-r2/diagnosis.md)：R1测试由Cargo父进程提供manifest并先解析后注入tool，未覆盖直接生产CLI；fresh/install检查未从真实带marker backup执行新恢复文档。调整=current_exe开发布局解析+无注入子进程正反控，以及真实升级backup→文档marker处置→verify→旧命令。F01/F02保持CLOSED，不机械重开 / next_responsible_role: work T-08/T-09 按调整实施并定向验证

2026-10-25 / stage: work（R2 needs_fix 残余修复） / plan_id: PLAN-832 / plan_revision: 3 / outcome: **pass（F832-R1-03/04 work修复完成，execution_done 9/12）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / base_commit: Man=ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f、lang=87f82d637ecd586c17a01c1c975e1456bc346991 / code_commit: Man=f52cd93b09401f358a89a19058fbf1fa7673fd16、lang=e320abf4fa1ae2d00d3e2bc91a35eea53345f409 / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-08/T-09 / evidence: [二次失败诊断](attachments/832/repair-r2/diagnosis.md)+[R2闭环](attachments/832/repair-r2/report.md)+[结果](attachments/832/repair-r2/results.json.txt)+[handoff](attachments/832/handoff.md)：直接生产CLI七格全过、永久无注入gen1/1；真实Core恢复640verify/旧命令/陌生extra负控通过，full永久恢复driver通过；F01/F02保持CLOSED / blockers: 无；统一回归/合入/ledger/归档/guard为merge-owned pending / next_responsible_role: review（T-10受影响独立复审；非merge/交付）

2026-10-25 / stage: review / review_scope: final（T-10 R3，受影响复审） / reviewer_identity: 沿用R1/R2独立审查聊天，未参与实施/修复，未建agent/第二权威checkout或虚构模型会话ID / plan_id: PLAN-832 / plan_revision: 3 / outcome: **pass（reviewed 10/12；merge-owned门pending，非交付）** / worktree_path: D:/autostack/.wt/lang-832/{auto-lang,auto-man} / branch: codex/plan-832-dev / reviewed_commit: lang=e320abf4fa1ae2d00d3e2bc91a35eea53345f409、Man=f52cd93b09401f358a89a19058fbf1fa7673fd16 / base_commit: repair-base lang=87f82d637ecd586c17a01c1c975e1456bc346991、Man=ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f；原整体base见baseline / dependency_revisions: UI=aee499e87093ff727f68ac8467c506e9ecf6ecdc、down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；四树进出HEAD/clean和锁一致 / spec_inputs: [冻结输入/锁/源码/技能/二进制hash](attachments/832/review-r3/baseline.json.txt)、[冻结SD](attachments/832/review-r3/spec-delta-reviewed.md)、[实际SD patch](attachments/832/review-r3/spec-delta-reviewed.patch.txt) / acceptance_results: **AC01..07 pass，AC08 merge-owned pending** / findings_closure: **F832-R1-01/02 CLOSED保持**（Core/installer/锁/语言生产及owner机制未变，不机械重跑）；**F03 CLOSED**（原七格实际生产发现+release布局/平放无回退共九格符合期望，新App.vue含本轮marker；永久测试清Cargo/AUTO发现环境且不注入tool，独立1/1）；**F04 CLOSED**（真实升级产生backup，以新文档PowerShell操作保留并移出owned marker，恢复640verify0/原manifest字节相同/旧脚本实际成功；陌生extra2、移出后0，full永久driver已核验未变机制） / evidence: [完整R3/AC-SD/复用边界](attachments/832/review-r3/review.md)、[实际结果及控制](attachments/832/review-r3/results.json.txt)、[实际恢复PowerShell](attachments/832/review-r3/documented-restore.ps1)；二次失败覆盖诊断与实际oracle调整已核，当前full构建/rustfmt/repair diff-check/Python AST通过；既有告警归因保留，不冒称零警告；jet/ark r3延期如实保持 / pending_closeout_gates: **T11/T12未执行**；merge先核并集等价/冻结输入，统一语言裸cargo t+实际适用触面、Man t/tcore/ti及必要consumer/UI/跨仓发行门，通过后才有序合入/三仓版本来源读回/Spec-ledger索引/归档guard清理；tf仍主语言检出按到期批量单实例 / next_responsible_role: merge（T-11/T-12）；本轮未改实现/跑整档/合入/发布canonical或ledger/归档/清树，最终交付仍待上述门与收尾。

r1以下记录为历史合同预检，其提前准备/并行放行和pass仅适用于旧合同，已由r2启动锁取代，不是当前执行授权。

2026-10-22 / stage: new / plan_id: PLAN-832 / revision: 1 / outcome: pass（规划合同预检，非实现验收）/ next: work T-01及门内T-02/03；T-04起等U815/U816，T-06完整三仓等831实际交付 / changed: T-01..12、AC-01..08、SD-01..06、V-01..06。见[预检](attachments/repo-delivery-batch-20261022/preflight.md)。当前无实施/验收任务完成、未运行未来driver/Cargo/搬仓。

2026-10-22 / stage: new / plan_id: PLAN-832 / revision: 2 / outcome: blocked（仅执行启动门；r2文档合同预检通过）/ next: 等待G832真实满足后重新核验完整交付/归档与源版本，再work T-01 / changed: T-01..12全局启动依赖、§0/4/5/6/8/10调度；原目标、AC/SD/V ID与发行验收保留。当前815/816仍executing、831尚未交付，故不启动实现/准备/建树/agent；Plan保持drafting 0/12。见[预检](attachments/repo-delivery-r2-serial-20261022/preflight.md)。

## 10. 待澄清事项

调度已明确，无须重复确认；G832未满足前不得以规划准备名义开工。T-01按实际U816确定包名/安装/profile/平台支持与832 source清单，831提供最终无环的共享声明与UI release。若必须破坏命令/默认支持、改变线程/取消合同或跨出旧仓承接范围，具体needs_replan，不把缺工具/环境当静默缩面。远端push/更名/registry发布按具体授权另行执行，不以其未获授权阻断本地三仓独立交付。

**已裁决（2026-10-25，r3）**：jet/ark 真实构建按用户指示“暂时不测，等v0.7再考虑”，转 v0.7 环境门控渠道并登记 KNOWN-DEBT；不视为已实测，不影响其他 AC 继续验收。该面后续落实工具链时须真实执行并保留失败非零语义。其余事项无须重复确认。
