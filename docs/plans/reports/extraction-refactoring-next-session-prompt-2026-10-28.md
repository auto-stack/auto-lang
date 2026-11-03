# 新会话接续 Prompt：auto-lang 拆分与重构路线

以下正文可直接作为新会话首条消息。状态快照为2026-10-28，新会话先读回文件和实际交付；不要把快照当实时状态。

> **2026-10-30 Cargo产物后继要求（优先于下方里程碑口径）**：用户要求当前888/890完整交付后，再调研/设计生成Rust工程与Cargo产物共享；已读[来源对话](thread://01a15c20-6af6-7471-8af1-456696e05370?hostId=local)，尚无源码/磁盘实测，不承诺“约20GB”节省。已登记[研究交接](cargo-artifact-sharing-research-handoff-2026-10-30.md)及[路线v20](v06-extraction-execution-roadmap.md)：研究可与UI014无交集工作协调，正式实验另排；根据研究确定有限实施/验收，治理结果和U014一起进入本轮近期M1出口。历史三仓M1/M2不回退，不往888/890/014/007当前合同追加Cargo实现，不提前测量/改workspace/清target或cache。此项未编号、未开始研究/设计或实施，不自动新建Plan/树/agent或唤醒后继任务。

> **2026-10-30 UI/Man正式立约更新（优先于全部下方旧快照）**：[auto-ui/014](../../../../auto-ui/docs/plans/014-ui-man-structure-closure.md)已立drafting r1、0/11，主改面UI、Man参与CLI，语言只留路线指针，不另取同包语言号。顺序U(auto-lang/890)→U(UI014)→按UI007原门推进；014收CLI/Vue/Aura/renderer职责与唯一状态/资源、测试守恒和真实消费，不重搬Web、不迁整棵examples或实现新主题。890现有全链授权有效，合同记录用户确认885收尾并关闭历史启动缺口，收尾读取executing0/12、尚非U；888 r2 executing2/10按其owner继续，不设U888全局前置。UI014全部实施含建树/探针仍等U890与后续明确实施授权，本轮仅规划，无构建/测试/GUI/agent/聊天通信/产品或他方执行合同修改。当前入口[路线v19](v06-extraction-execution-roadmap.md)、[UI结构设计](../../../../auto-ui/docs/design/autoui/ui-man-structure-closure.md)；下方“未编号UI/Man”“885启动仍未处置”“888未开工”等是旧时点，不复活其阻塞。

> **890规划最新更新（优先于全部下方快照）**：884 archived10/10、887 archived r2/10/10且有guard收据；887性能以用户批准的机制级负结果关闭，不能称已显著加速。885已合入归档，末尾cleaned文字仍待核准确收尾锚。已立[890 Web完整搬迁](../890-auto-web-independent-repository-delivery.md) drafting r1、0/12，承接[Design50 r2](../../design/50-autoweb-service-framework-and-repo-boundaries.md)；主线为已交付884/887→890→未编号UI/Man。888 executing0/10仍遵用户“先核查，暂不交接共享写入”及X888/M888，不因865归档头自动放行；U888不作为Web全局前置，真实共享窗短串行。890 T-01核来源/仓身份/上游收尾/X，T-02先验无环端口、旧SDK兼容和共享生成，再大规模迁移。本轮仅规划，未建仓/树/agent、未实施或运行项目测试、未改他方合同/产品/canonical/ledger、未提交/push；保全部现有WIP。当前入口[路线v18](v06-extraction-execution-roadmap.md)，下方Web未编号/887待交付等均属历史。

> **本轮测试优化最新补充（优先于下方快照）**：[884已归档10/10](../archive/884-language-structure-closure-without-crate-split.md)；887读取为r2/reviewed9/10，独立终审pass不等U887，性能机制级负结果不等已加速。已建立[888模块选测与ACE成组试点r1](../888-module-test-selection-and-ace-batch-pilot.md)，本轮仅规划，G888=U887+后续明确执行授权，另须真实X888与性能M888。建议其为U887后优先质量支线，不等全部Web/UI-Man拆完；Web主线不加无关U888门，设计可继续，真实脚本/注册交接短窗串行。首批15个AV正例比较独立与k1/3/10、3条纯表达式有限C；原子义务/精确定位/全局冻结门保持，不改887/885/ACE合同或建agent/仓/树。见[路线v17](v06-extraction-execution-roadmap.md)及[专题设计](../../design/testing/module-selection-and-ace-batching.md)。下方v16等为前一时点记录，状态/授权不能覆盖此处及正式Plan。

> **2026-10-29最新更新（优先于全部下方旧快照）**：869/872/873已归档，873暂不细拆crate；884已获执行授权并实际开工，继续原合同，不按滞后drafting头判未实施。[887](../887-test-governance-and-ace-first-corpus.md)已立约，整体实施等U884及明确指令。用户本轮确认排程**884→887→auto-web有限迁移→UI/Man结构包**；[Design50](../../design/50-autoweb-service-framework-and-repo-boundaries.md)给出首期仓界/验收及885/887交接，887结束后再取号冻结正式迁移Plan，Web/UI-Man均未编号。885与BLOG-004继续，不加Web前置；Web包自己负责新测试owner/四仓组合，不扩887合同。原三仓有限收口与新增Web交付分列，整体到点可能后移。详[路线v16](v06-extraction-execution-roadmap.md)、[战略](../../design/strategy/modularization-and-refactoring-strategy.md)和[范围](post-873-structural-work-packages-2026-10-29.md)。本轮仅设计/路线，不建仓/树/agent、不联系其他会话、不改在途Plan；A1及数据/runtime另签核，应用/ACE不等无关全局门。

> **同日后续更新（优先于下方原交接快照）**：867 r2已归档13/13，merge/ledger v117/六树guard收据完整，U867成立；876 r3已三仓合入/归档/四树guard清理，UI master保留5a89bfd，OS脚本iced轨已改用os-desktop/ui-desktop。869 r4、872 r3、873 r2仍drafting；下一主线为869 T-01/T-02，873可预约独占窗口在减法前测固定基线，872仍等U869。本次仅更新启动门跟踪，未认领实施；以[路线图v13](v06-extraction-execution-roadmap.md)及两份归档收据为准。867冻结测试组合早于876的UI交付，不外推当前HEAD通过。AutoUI新架构仍等待已约定的有限里程碑。

---

请接续跟踪 auto-lang / auto-ui / auto-man 的拆分与重构路线。工作目录 `D:/autostack/auto-lang`，兄弟仓位于 `D:/autostack/`。本会话主要负责总体规划、启动门和独立复审/交付判断；不要自动认领其他执行会话、重复实施或创建新agent。

先阅读：

1. `D:/autostack/auto-lang/AGENTS.md`。
2. `D:/autostack/auto-lang/docs/design/strategy/modularization-and-refactoring-strategy.md`（长期决定、当前阶段、真实消费与归属）。
3. `D:/autostack/auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md`（当前调度唯一入口，v16，尤其§1/2/5/6/9）；Web新增范围另读`docs/design/50-autoweb-service-framework-and-repo-boundaries.md`，不误当现行已建仓。
4. `D:/autostack/auto-lang/docs/design/37-v06-modularization-and-repo-boundaries.md`（§8.15与§9为当前入口，早期批次为历史）及 `docs/specs/repository-boundaries/` 下相关现行Spec。
5. 下列在途Plan状态头、当前任务/AC及最近交接记录；按需读代码和相关Spec，避免重读全部历史。

长期目标和已完成阶段：

- 目标是几个职责完整、依赖清楚、能在各自仓库持续开发/构建/测试/发行的大模块。后续按较大的完整模块粒度规划，生产接线、旧实现删除、真实消费者和验证在同一Plan闭合，不为每个probe/函数/验证阶段取号。
- 831/832已归档：UI/Man已迁入原有独立仓，沿用旧仓身份/URL/历史，legacy refs及恢复载荷保留；不再重新建仓或清空主分支。841–844也已归档：领域模型/lowering、兼容退役、语言结构、首批净瘦身已有限交付。844生产净删终值1,221行。不要重新规划正式搬仓或把这些归档项目写成待实施。
- M1/M2交付不等于测试全集或所有应用/平台全绿。正常依赖仍是产品/工具→公开SDK；UI→语言基础/中立协议；语言基础不反依赖UI/Man/桌面，管理核心不拉完整VM/renderer。类型/算法单源，不造common巨包。数据域/runtime独立仓仅候选，另需go/no-go，不算前三仓交付欠账。

交接时的近期计划：

| 计划文件（均在auto-lang/docs/plans/） | 状态快照 | 启动与后继 |
|---|---|---|
| `867-split-equivalence-test-closure.md` | r2 executing，2/11 | U844已成立，继续原权威树；当前最先收口 |
| `869-crate-inventory-and-subtraction-batch-2.md` | r4 drafting，0/10 | 整项等U867 |
| `872-builtin-semantics-dual-track-ruling.md` | r3 drafting，0/10 | 整项等U867且U869 |
| `873-build-cost-and-member-split-decision.md` | r2 drafting，0/8 | 整项等U867；实际测量另需M873独占，可早于869 |
| `876-autos-desktop-entry-and-launch-health.md` | r2 execution_done，9/11 | 另一会话已实施，待final review/冻结门/三仓merge及收尾；不能当已交付 |

U(N)=完整AC、独立final review、merge-owned冻结门、实际修改仓合入、canonical/受管ledger及来源读回、归档/guard收尾全闭环。阶段pass/list-only/代码提交/SD草案不等于U。869/872/873门覆盖全部盘点/probe/测试/建树；门前只做上游收据核查和已授权规划。

既有裁定请继承，不重复询问：

- 867：完整测试义务分母及迁移前后逐ID总账；未解释缺口=0、新增红=0，既存红逐ID认证，消失红解释。编译阻断、环境缺失、ignored和未执行分列。r1的8,315枚举子集、list-only或排除th编译红不能证明全集执行。具体档位含Man t/tc/tcore/ti和UI tu/ti/tn，依r2合同核验；未知或必需正控未执行仍阻塞。
- 869：A约定范围全部.rs作披露/80万达标口径，B三仓生产NET-PROD不得反弹，测试/生成/第三方分列；C旧单crate仅历史。三仓同commit组、同脚本/分类版本复算，XO排除范围不得静默改变。每批生产净删≥2,000，不足合批/重规划；结构性减法仍按替代路径/承诺迁移/门禁/用户按批签核。收据含删除SHA、恢复配方/演练与推送状态，不强制push。80万是争取目标，不靠删功能/测试刷数，也不授权无限减法。
- 872：str.len、math、to_int/parse_int、time_now、HTTP error-reply、nv_to_value六族全部DISTINGUISH，零行为变更；先冻结真实注册/覆盖/生效实现与消费者绑定表。保旧名、优先级、结果和错误，plan755 body字节保持。MERGE未授权；RETIRE须无消费者证据另签核。
- 873：先测后决定是否拆member，不承诺拆或加速比例。固定版本、隔离target/cache，禁止清共享sccache；冷/热check/build、二进制构建/链接、枚举/真实启动及registry成本分层；至少7个真实源码编辑站点、每组至少3个有效样本。touch只作控制。构建/测试/重负载独占窗口，污染样本废弃；不新跑整档，引用可比merge收据。后续结构设计冻结前须U873并核最终869/872输入差额。

后续仍保留两个未编号大工作包：

1. 语言结构收口：call_inner、stdlib/http_server/trans入口、lib glue，先区分843合同残项/验收差额/新目标；须U869/U872/U873与实际共享面交接。
2. UI/Man结构收口：renderer/vue/aura_view_builder/ui_gen/Man入口，保守等语言结构包完整交付；按职责、状态owner和真实消费者重构，不以文件切块计完成。

**不要称它们Plan874/875：两个号已给AutoGraphics。** 目前只保留工作包，用户要求立项时new-plan.sh原子取届时实际编号。868已归档、870/871属独立AC/MIR线，按真实路径/符号重叠协调，不设无关全局等待。主治理线867→869→872→语言结构→UI/Man结构，873为测量支线。

刚完成的真实消费验证及新风险：

- auto-os计划051复审通过并合入main，交付`3088dea`、实测报告`234e5f6`，未push。报告在 `D:/autostack/auto-os/docs/plans/attachments/051/runtime-check.md`。复审是原会话重新审计，未另开独立模型，独立性限制在报告中如实披露。
- 新脚本真实构建Man auto.exe、UI auto-ui-tool.exe、语言ui_desktop宿主并启动VM桌面；桌面及独立auto run -r vm计算器均实测2+3=5。独立VM首次因UI工具Windows主栈1MiB溢出，在工具owner加build.rs主栈策略64MiB，auto-ui master `5a89bfd`已落地，常规Cargo重建/PE检查/真实按钮复测通过，未push。
- 视频playlist.at解析错误、Music/Photos扫描接口404仍open，债在 `D:/autostack/auto-os/docs/plans/KNOWN-DEBT-AND-RISKS.md`。无旧组合对照，不能认定既有基线或迁移新回归；不把空媒体视图当成功。
- 876候选将桌面入口从语言example迁到auto-os/host/desktop（bin ui-desktop），加启动健康面和缺产物语义。其OS基面早于051，UI基面早于5a89bfd；复审/合入须检查051三仓脚本/候选目录/错误码/环境恢复合同和UI栈修复是否保留、证据是否因变更失效。不要用旧基面覆盖新交付，051旧example命令也不要当876交付后指南。
- 876完整交付后，OS/桌面产品计划和债在auto-os独立取号、跟踪、归档；auto-lang只留指针，跨仓正文放主改面仓。归属已写入876 r2，仍需检查实际主分支落地。Vue desktop-host归位等剩余事项按其合同另跟踪。
- 业务应用可固定已交付三仓公开版本并行开发，不必等所有治理；SDK/协议/renderer/native共享问题由各owner发布修复后重放，不复制私有实现。

运行证据位于 `D:/autostack/.verify/os-051-20261028/`，完整版本/hash收据为runtime-final-receipt.json，截图和日志仅本地。冻结依赖树为 `.wt/os-051/{auto-lang,auto-ui,auto-man,auto-down}`；旧OS实施树已guard清理，依赖树因运行保留。最后观察桌面/计算器仍在跑，但PID/端口不是持续有效保证，先查进程及归属再操作，不误停876或其他会话的桌面。

工作纪律：

- 主检出有大量其他路线文档/计划WIP，交接更新也可能尚未提交；先读git status/diff，保留现有文件，不reset/stash/全量git add。不要把他人改动纳入自己的提交。
- 代码/构建/测试在权威worktree，禁止junction/symlink，删树前fresh wt-guard。独立scope审查/修复可并行，同Plan同树单写者、冻结交接；共享接口由owner交付后再消费。
- work/review只做相关反例/模块测试/必要check；final pass后merge统一裸t及实际触面档，tf主检出到期单实例。不得以测量或一条finding触发重复整档。
- attachments原始证据不得git add/commit/push，只提交Markdown摘要（命令/退出码/数字/本地路径+hash）。不自行push、改默认分支名、过滤历史、清共享缓存或新建数据/runtime仓。

本轮请先只读核对最新Plan状态、实际交付、启动门和876/051/栈修复交接，给出最先该收口什么、哪些可并行及文档差额；按核查结果维护设计/战略/路线。暂不自动执行尚未满足门的计划或新建后续计划。后续我会逐项指定复审或实施。
