---
plan_id: PLAN-832
status: drafting
feature_name: auto-man 独立仓及三仓组合发行交付
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-22
plan_revision: 1
current_step: 0
total_steps: 12
supersedes_spec_components: [docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/auto-man/project.md, docs/specs/auto-man-core/project.md, docs/specs/auto-cli/project.md, docs/specs/tool-execution/project.md, docs/specs/tool-protocol/project.md, docs/specs/language-sdk/project.md]
new_spec_components: [docs/specs/repository-boundaries/man-three-repository-delivery.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-man/repository, auto-cli, tool-execution, repository-boundaries]
depends_on: [PLAN-809, PLAN-814, PLAN-815, PLAN-816, PLAN-831]
---

# [PLAN-832] auto-man 独立仓及三仓组合发行交付

## 0. 变更摘要

完成R09并吸收R07 Man/CLI发行：管理核心/adapter、执行编排/client/发现、auto CLI及资源/测试/安装/CI承接到**既有auto-man仓**，保旧仓legacy与来源；语言仓迁出相应实现，三仓固定版本完成独立与组合消费。交付Core SDK/管理无UI与完整UI工具profile、包管理/构建/生成/服务/watch/升级恢复，而不只是复制crate目录或生成一个安装包。

可与815/816做无共享写面的准备；完整导入等U815/U816，最终三仓UI组合等831真实发布，不等其清理。UI生产实现归831，不静态搬进Man；UiAst终态、兼容退役、大规模去重与远端发行不在本Plan。

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

用户2026-10-22授权建立本Plan及831，包含发行验收和并行边界。**本轮只规划，不执行旧仓/跨仓写入、切换默认分支、推送或安装替换**。未来work在授权范围内建立隔离候选/独立验证，本地默认分支实际交付由final review+冻结门后的merge按§5组织；若有WIP不能安全推进，不覆盖或reset。

当前依据：[Spec入口](../specs/overview.md)、[Man](../specs/auto-man/project.md)、[CLI](../specs/auto-cli/project.md)、[执行设施](../specs/tool-execution/project.md)、[协议](../specs/tool-protocol/project.md)、[SDK](../specs/language-sdk/project.md)、[旧仓](../specs/repository-boundaries/project.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)。总览有旧数字/聚合描述，以模块Spec+实际代码/版本收据为准。809/814已交付；815 A及816 A已发布，完整B尚未合入，主检出尚无816最终管理服务包，不能为未来包名伪造执行证据。

已观察：根Cargo与`crates/auto/src/{main,cmd_watch,cmd_service,cmd_ui,cmd_bp,cmd_autodesk}.rs`、`crates/auto-man`及core/锁/resolver/toolchain/assets、`auto-tool-{protocol,execution,client}`、`auto-ui-tools`/API-gen的声明残边。最终服务/CLI拆分写面由816实际交付重采，归本Plan生产搬仓，不接手其未完成findings。旧auto-man Cargo/CLAUDE已读，784 refs/WIP/ignored快照仅历史证据，本轮未重采git状态，T-01/T-02负责刷新。

输入SHA/hash/状态见[本轮输入](attachments/repo-delivery-batch-20261022/inputs.json)，旧仓恢复约束沿[784报告](reports/784-repo-preservation.md)。除本Plan管理/CLI归属范围不迁UI实现，不覆盖其他路线的未提交文件/分支。

## 5. 详细设计

### 5.1 启动门与下一任务入口

U815/U816为完整B实际合入与Spec/版本来源可读，定义见本批§1。**T-01现在可只读、T-02/03在独立组可与815/816并行**；不得复制其WIP接口/抢改源清单。T-04/05完整候选导入/管理包装等U815+U816，可与831后半候选/验证并行。T-06完整UI/三仓集成另等831 T-11真实UI交付；其余不受影响准备/审查/修复无需停工。

T-01新增`attachments/832/contract-baseline.md`，从真实CLI command树/当前已发布owner形成矩阵：裸脚本/REPL/透传、语言trans/docs/stdlib、管理new/init/build/run/test/clean/add/fetch/deps/export、UI/bp/autodesk、service/serve/req/watch及既有挂载命令。逐项记录flags/机器输出/环境/目录/依赖工具/错误与资源寿命/支持平台、所有source/锁/安装入口和旧oracle；orphan未挂载文件不假新增。完整816未知项pending且逐项交T-04重采/实际probe，不把缺证据划为“不支持”。T-01关闭当前基线/目标合同及未知项归属，可立即放行独立T-02/03，不隐性等待U816。

### 5.2 数据流、安装及组合

| 规则 | 实际producer→consumer/状态 | owner/关闭条件 |
|---|---|---|
| C1 管理独立闭包 | 固定SDK+轻请求→管理核心/语言adapter→CLI；UI工具实现仅另进程 | 832；Core clone/install无UI源码/静态库；命令实际成功，锁/类型来源唯一，不用空stub替代管理能力 |
| C2 完整UI发现 | 固定831 tool/协议/资源清单→816真实discovery/client→UI生成/build/run | 832；首次开发/已安装/升级/重定位均用实际匹配tool；错误能力/版本/producer先于副作用拒绝，无大库fallback |
| C3 服务/执行退出 | CLI argv/cwd/env/stdio→owned设施→进程内VM/子进程/reader/watch→结果 | 832；保!Send VM线程亲和，正常/失败/取消/超时实际退出；service不杀占用进程、不假ready；仅清理本次owned对象 |
| C4 三仓版本/恢复 | SDK commit+UI release+Man候选+锁/二进制hash→三仓外部工程/安装→升级/切换 | 832；固定组合可重放；冲突/半安装/中断保旧或明确恢复、未全部verified不声称三仓交付 |

Core与完整profile按816冻结合同继承：Core不需要UI checkout/二进制即可执行约定语言/管理命令；完整profile包含匹配UI tool及其必需输入。分包并不许可默认UI消失、安装顺序靠用户猜、旧flags/JSON/exit漂移。`auto service`、`auto serve`、`req`及watch/HMR有不同既有角色，不能顺手合成daemon或扩大legacy端口强清策略。auto-down等外部依赖按profile固定来源及明确平台前置，不借任意主检出。

### 5.3 来源、切换与R07验收

T-04冻结U816真实源码与包列表导入Man旧仓候选，不在库中复制UI/轻协议；T-05建立独立workspace/锁/安装/CI入口与非UI对照，若需要831未发布轻声明接口则仅该scope pending。T-06固定831真正UI版本后完整三仓验证，T-07覆盖fresh/locked/重定位/依赖fetch/升级/损坏输入与受管失败恢复。

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
| V-03 | AC-03 三仓UI/安装 | 真实auto安装二进制→831固定UI tool→生成/build/run；首次开发/安装/升级/重定位；坏版本/缺件/错能力 | 真产物/运行oracle通过且来源hash匹配；错误先拒绝副作用；Core不拉UI，完整默认UI可用，不静态fallback |
| V-04 | AC-04 发行/失败 | 三仓clone fresh/locked→发行包A搬B，坏锁/hash/源/patch/半安装/取消/交换与回滚故障 | 固定版本可重放；失败具名非零，保旧交付/用户文件或实际可行恢复点，陌生目录不删，恢复后复绿 |
| V-05 | AC-05 历史/多仓事务 | 当前refs/WIP/ignored快照与bundle恢复、目标已落但消费者未切/切后验证失败 | legacy树/refs/保留字节可恢复；状态收据准确、重试幂等，不能只一仓落地就标全交付；不覆盖在途变更 |
| V-06 | AC-02,06 寿命/测试/CI | service vm/rust配置/port冲突/ready、serve/req、watch/HMR、child/reader超时取消，测试ID/适用CI入口 | 真资源退出/VM线程亲和，service不杀占用者/假ready，原测试选择不丢；CI本地执行与远端未跑分别披露 |

实施/修复与独立review可跨Plan并行，每Plan同组每仓一权威树，冻结提交后不可同时改；共同反例集和影响scope证据按共用合同§5。所有work正反控与必要实机先完成；final review独立上下文验全AC/SD。merge在冻结三仓输入上统一裸cargo t+实际触面的tv/tu/tt等与目标仓适用门、串行窗口，tf到期主语言检出单实例，重型/前台OS资源预约独占。缺环境/缺上游的最终正控pending，不标整体pass。未指定agent/model，不创建会话；本轮纯规划无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 独立Man/Core真实管理消费，闭包无UI实现/隐式邻仓，模型/协议/SDK单源，无UI/语言反向依Man；V-01 |
| AC-02 | work | 旧命令/API/动态配置/参数/结果与服务/stdio/线程/进程寿命保持，完整profile真tool可用；V-02/03/06 |
| AC-03 | work | 固定831实际交付+SDK+Man的真三仓生成/build/run、首次bootstrap与独立安装闭环；V-03 |
| AC-04 | work | fresh/locked/重定位、版本/资源/producer/升级与失败恢复正反控满足profile合同；V-04 |
| AC-05 | work | 旧Man新状态保全/恢复、全来源映射与多仓切换故障演练有效；V-05 |
| AC-06 | work | 语言迁出管理/CLI实现实际删除、测试ID/CI/777/792覆盖、命令/资源/许可/版本/SD与三仓handoff完整；V-06 |
| AC-07 | review | 独立最终review绑定r1/三仓冻结commit及实际UI release、全AC/SD/遗漏/健康审查pass，merge门pending |
| AC-08 | merge | 冻结门通过、目标Man与语言消费实际合入/三仓版本与来源读回、Spec/受管ledger、归档及guard清理齐备，M1实际交付可追溯 |

## 8. 执行步骤

12项。粗粒度depends_on表示整体终验关系，前置未全归档不阻断T-01..03；任务门与共用合同为准，不声称后端有阶段调度功能。

| 状态 | ID | owner_stage | 依赖/启动门 | 写面/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在可只读 | attachments/832/contract-baseline.md（新增）：实际包/命令/旧oracle/安装/owner/DAG与支持矩阵，区分observed/hypothesis与831共享切口，未闭项具名交接，最终816源码/probe交T-04补齐 | AC-01..06 |
| [ ] | T-02 | work | T-01旧仓事实；可与815/816并行 | 重采旧Man指令/refs/WIP/ignored并保全/恢复，建立或认领唯一隔离树与preservation.md；不动旧默认代码树 | AC-05 |
| [ ] | T-03 | work | T-01；与T-02/815/816独立 | 新三仓consumer/安装/故障/命令与测试ID设施，先用809/814/协议A已交付输入做真实范围控制，不改生产热点 | AC-01,03,04,06 |
| [ ] | T-04 | work | T-02/T-03+U815+U816 | 以816最终实际包图导入管理族/CLI/client/发现/资源到Man候选（共享std执行设施留语言），重采全部source/hash/许可与832 owner；不复制UI/中立模型，不借831 WIP | AC-01,02,05 |
| [ ] | T-05 | work | T-04；可与831候选/审查并行 | 独立workspace/锁/固定语言source、Core打包/安装与非UI命令V-01/02；依831未发布切口的子项pending，完整profile不假绿 | AC-01,02,04 |
| [ ] | T-06 | work | T-05+831 T-11真实发布/来源读回 | 固定UI工具/版本/能力/producer安装与真实三仓命令/服务/watch/UI组合V-02/03/06；不等831归档清理 | AC-02,03 |
| [ ] | T-07 | work | T-06 | 独立clone/第三方工程及安装fresh/locked/重定位/升级失败/坏源负控V-04，复用792与owned失败清理 | AC-03,04 |
| [ ] | T-08 | work | T-07；共享接线窗口且831实际交付 | 语言Man/CLI/执行实现删除/消费者切固定源，别名/测试ID/777/792/CI/docs更新，三仓切换失败/历史恢复V-05/06 | AC-01,05,06 |
| [ ] | T-09 | work | T-08 | 最终来源全部V复核、支持面/原件删除与非净删统计，SD-01..06、版本/工具hash与attachments/832/handoff.md（新增）送审 | AC-01..06 |
| [ ] | T-10 | review | T-09冻结提交与真实831交付 | 独立三仓全AC/SD/错误与资源/来源和恢复审查，最终pass仅表示merge门pending | AC-07 |
| [ ] | T-11 | merge | T-10 pass；共享合入窗口 | 并集影响复审，冻结统一门；本地Man分支与语言消费有序合入、三仓组合/版本/安装来源读回，新增three-repo-release.md | AC-08 |
| [ ] | T-12 | merge | T-11全部交付收据 | canonical Specs/受管ledger与索引沉淀、归档；guard clean逐树清理本Plan临时树/分支/组，保legacy/恢复/实际交付分支 | AC-08 |

执行组`D:/autostack/.wt/lang-832/{auto-lang,auto-man}`，各仓分支`codex/plan-832-dev`；UI消费用831已交付fixed-rev clone/包，需附加只读UI树则本Plan该仓仅一份，不checkout831在途分支。必要auto-down兄弟按既有解析补detached，不改实现。已有784准备树/分支先核owner与保全后认领或另建本Plan分支，不强移；唯一树贯穿work/review/merge，不建第二审查树，无junction/symlink。主检出只簿记，逐树fresh guard clean才移除。

## 9. 复审记录

2026-10-22 / stage: new / plan_id: PLAN-832 / revision: 1 / outcome: pass（规划合同预检，非实现验收）/ next: work T-01及门内T-02/03；T-04起等U815/U816，T-06完整三仓等831实际交付 / changed: T-01..12、AC-01..08、SD-01..06、V-01..06。见[预检](attachments/repo-delivery-batch-20261022/preflight.md)。当前无实施/验收任务完成、未运行未来driver/Cargo/搬仓。

## 10. 待澄清事项

无须用户重复批准的当前规划问题；T-01按实际U816确定包名/安装/profile/平台支持与832 source清单，831提供最终无环的共享声明与UI release。若必须破坏命令/默认支持、改变线程/取消合同或跨出旧仓承接范围，具体needs_replan，不把缺工具/环境当静默缩面。远端push/更名/registry发布按具体授权另行执行，不以其未获授权阻断本地三仓独立交付。
