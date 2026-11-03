# 模块拆分与重构：当前战略及接续边界

> 当前战略v24，2026-11-03。唯一调度表见[路线v24](../../plans/reports/v06-extraction-execution-roadmap.md)；下方2026-10-30正文折叠保留为历史，不再按旧waiting/测试入口/证据留存执行。本轮仅核查、Plan立约及文档刷新，暂不实施。

## 1. 当前接受的有限交付

831/832/841–844仓界、867/869/872/873治理、876桌面入口、884/887结构与测试质量、890 Web完整搬迁均已归档。896/898固定测试组和tm退出、900/901语言UI依赖退出与feature拍平、906语言独立消费、UI018/014编译/结构、905研究/910有限Cargo治理、UI007示例归位与首批主题均已交付。873暂不进一步细拆crate结论保留，900-N7已由906两层门关闭。

这些是各合同有限结果：真实平台/原ignore/既有红/持有与未测原子义务不自动消失，不重新要求所有应用或ACE/AG全线完成来证明历史结构交付。

## 2. 后继工作与顺序

- 031/033交付，B1累计十例；[036](../../../../auto-ui/docs/plans/036-official-theme-distribution-and-input-closure.md)最新executing4/10、复审needs_fix返修，尚未交付，补包渠道/生成输入。
- 新立[037](../../../../auto-ui/docs/plans/037-business-examples-theme-migration.md)草案0/11，B2十二业务例保行为主题迁移；获授权可先path基线/源码，采用新渠道的最终门等U036。
- 新立[038](../../../../auto-ui/docs/plans/038-specialized-examples-theme-readiness.md)草案0/8，B3十二专用例先做六类真实支持核验，产前置/owner/后继分包，研究不是迁移完成。
- [OS053](../../../../auto-os/docs/plans/archive/053-host-theme-chain-and-calculator-adoption.md)已交付但G-1/G-2具名未闭，新立[OS055](../../../../auto-os/docs/plans/055-clock-theme-and-host-evidence-closure.md)草案0/11，真实clock＋系统事件/真实载体，正式包接U036；不重复OS054八产品装配。
- [918](../../plans/archive/918-cross-repository-admission-and-boundary-closure.md)已归档9/9，有限治理交付；后继真实来源仍按审计，Web schema等差额各归owner。
- 本轮仅调研/立约；顺序与静态依据见[第二阶段设计](../../../../auto-ui/docs/design/autoui/theme-rollout-stage-two.md)。State/Feature/Reload继续独立深化，不成为主题推广统一前置。

- **全实际产品独立线**：[OS Design03](../../../../auto-os/docs/design/03-product-theme-rollout.md)已盘28 active入口；calc已交1、055/clock待做1，其余26全量立056–061六批（生产力8/工具6/媒体3/Reader1/游戏3/外部5），均drafting0/9，暂不实施。外部Musk/Jade与local syslog不漏项，产品不从教学源码同步，不以Demo绿抵交付。
- 056最终OS pin/组合主题门等054正式业务组合，059接Reader007正式阅读表面；其他批不整体等待。六批采用036正式发行；055只有所需载体能力是对应腿前置，不把系统设置窗口扩成全产品门。

## 3. Owner与真实限制

语言基础不依赖UI/Man/Web实现。036拥有包渠道/中立输入接线，037拥有教学业务主题映射，038只核专用支持面，055拥有真实clock与宿主G-1/G-2；各接受正式来源/接口，实际共享文件/锁短窗单写，不借WIP。055系统事件届时要具体窗口许可，不能用provider注入/协议helper当真实闭环。

教学B1十例已迁，B2/B3二十四仍未迁；实际产品另按28入口逐项验收，研究/计划覆盖不改变任何已迁分母。055不升级054八产品pin，教学例不替真实产品。Web D-906-1与schema级预期ID集合、Man D-910-2、UI旧红/013生成与015编辑依赖保真实owner；所选必需面失败不能直接按历史债充PASS。批回归user-pending、910缓存持有与平台限制保持。

## 4. 工作/验收纪律

实际实施先唯一worktree/明确授权，同Plan同组顺序实现/独立review，实际共享manifest/锁/owner输入短窗串行。语言正式cargo t固定组，其他仓保owner入口，按真实修改完整职责及必要消费者验证，final review后冻结并集，不叠跑全目录。

材料仅统一plan-artifacts/plan-NNN并按repo登记；merge摘要折Plan后整删自有scope/树内产物，具名未闭持有有owner/理由/关闭条件。无链接，移除先guard clean；现知识在docs/specs，不写旧specs.json。新基线须绑定可解析的真实干净来源，不用宽豁免或刷新吸收未知违规。

<details>
<summary>2026-10-30战略原貌（历史来源；当前事实与规则以上述v21为准）</summary>

# 模块拆分与重构：当前战略及接续边界

> 更新：2026-10-30，当前入口[路线v20](../../plans/reports/v06-extraction-execution-roadmap.md)。UI014草案和U890→U014顺序保留；用户新增当前888/890完整交付后的[Cargo产物调研/设计](../../plans/reports/cargo-artifact-sharing-research-handoff-2026-10-30.md)，有限治理结果纳入本轮近期M1。只登记后继需求，不启动研究/实验/实施、不取号/建树/清cache，不改他方执行合同或联系他方。

## 1. 目标与已经完成的范围

目标是若干职责完整、依赖清楚、可独立构建/测试/发行的大模块，能够在自己的仓库持续演进。先证明契约、单一类型身份、真实消费和资源闭包，再搬仓；移走相同行数不算生态净删除，也不能以建立仓库代替产品验收。

M1 已由 831/832 归档交付：沿用既有 auto-ui、auto-man 仓身份与历史，legacy refs/恢复载荷保留，新实现以普通提交承接默认分支，不另建替代仓、不清空/orphan/强推。auto-lang 保语言 SDK/中立模型协议，auto-ui 保 UI 领域编译与运行，auto-man 保管理核心、adapter 与薄 CLI。三仓主分支目前为语言/UI/Man 的 `master`；auto-os 为 `main`。更名、远端 push、registry 发布不是已获默认授权的收尾。

M2 已由 841–844 归档交付：UI owned AST/Token/lowering 终态、旧兼容与发行路径退役、语言内部结构、跨仓去重与净瘦身。844 收据生产净删 1,221 行；这是该批有限交付，不代表全部潜在重复清零。867 已归档，完整交付收据确认测试等价出口成立；其冻结组合的执行通过仍保留 th、ui-tn、Windows aavm 等限制，不外推为所有环境和应用全绿。

允许的方向为：产品/CLI 装配→公开 SDK/工具；UI→语言基础 SDK/中立协议；Man 管理核心不拉完整 VM/renderer；语言基础不反向依赖 UI/Man/桌面产品。共享类型/算法只有一个 owner，不引入塞满跨界类型的 common 巨包。数据域及 runtime 域可先保持独立 crate；另建仓须真实收益论证和单独 go/no-go，不是 M1/M2 未完成的证据。

## 2. 当前计划与出口

以下数字是读取各 Plan 状态头得到的交接时快照，新会话先读回更新，不以此表覆盖后续进展。

| 计划 | 快照 | 交付与启动门 |
|---|---|---|
| [867 测试等价性](../../plans/archive/867-split-equivalence-test-closure.md) | r2，archived，13/13 | U867 已成立；合入/ledger v117/六树 guard 清理收据齐全，允许后继核门 |
| [869 crate 清单与减法批二](../../plans/archive/869-crate-inventory-and-subtraction-batch-2.md) | r4，archived，10/10 | 47 crate清单、生产净删2,799行及三仓合入/ledger v122/guard清理闭环 |
| [872 内建语义辨析](../../plans/archive/872-builtin-semantics-dual-track-ruling.md) | r3，archived，10/10 | 六族DISTINGUISH零行为变更，合入/ledger v124/guard清理闭环 |
| [873 构建成本与 member 决策](../../plans/archive/873-build-cost-and-member-split-decision.md) | r3，archived，8/8 | U873成立，暂不细拆crate；测量/模型/限制、合入54f4ae6e9/ledger v127及四树guard清理闭环 |
| [876 桌面入口归位与健康面](../../plans/archive/876-autos-desktop-entry-and-launch-health.md) | r3，archived，completion_kind: delivered | 三仓合入、规范/账本及四树 guard 清理已有收据；头部 current_step 仍为 12/13，不据此重开历史计划 |

U(N) 表示约定 AC、独立 final review、merge-owned 冻结门、实际修改仓合入、canonical/受管 ledger 与来源读回、归档和 guard 收尾全部闭环。work pass、list-only、阶段合入、SD 草案不等于 U。869/872/873 的门覆盖盘点、探针、测试和建树；门前只核前置收据和已授权规划。

主排程：已交付867/869/872及873 → **已交付884→已交付887→890 Web完整搬迁→auto-ui/014 UI/Man结构收口→核近期有限出口→按原门推进UI007**。结构正文在UI仓，Man参与CLI，语言仅调度/指针；不再分配语言891或Man重复合同。873不细拆crate、A1另签核及测量适用限制保持。

测试模块化是拆分质量基础设施，建议887完整交付后优先启动888有限支线，**不等Web/UI-Man全部结束**。模块映射服务完整测试义务，批处理首落ACE；888允许交接后新增最小test-only worker入口，原全局门/原子义务不取消。Web主线不设无关U888全局等待，设计可继续；共享脚本/注册短窗串行，后继接受正式交付工具。该建议不代表本轮实施授权或给887追加验收。

当前读取：884/887已归档并清理，887性能机制级负结果不等已加速。890已有全链授权，合同记录用户确认885收尾并关闭该历史guard证据启动缺口，收尾读回executing0/12；原始日志未恢复，不伪造。888已executing r2、2/10，按其owner继续；不设U888全局前置。UI014新立drafting0/11，整项实施等U890与后续明确实施授权；实际共享面逐符号交接，不认领他方或等无关ACE全线。

890为完整搬迁合同，端口/兼容/共享生成DAG先验证，测试owner/四仓组合由其交付；其既有执行授权继续，不受本轮“只规划UI014”限制。UI014自己闭合CLI/Vue/Aura/renderer现有结构、测试和消费，既不回搬Web算法，也不实施007的示例/新主题；全部工作含建树/探针严格等U890。应用继续原合同，不加Web/结构全局前置。

### AutoUI 架构重设计的切换点（2026-10-28 用户决定）

先达到近期有限收口，再重审AutoUI新架构。原三仓有限判据仍为既有交付及U884/UI-Man结构完整关闭，876已交付不再等待；本轮用户选择在884与UI/Man之间安排887和Web，因此当前总体排程会先完成这两包。新增Web独立交付单列，不重开三仓历史M1/M2，也不把Web扩展和全部测试改写无限追加到出口。

现有[Design47](../47-autoui-state-design-and-feature-architecture.md)、[Design38](../38-autoui-features-and-system-services.md)及宿主/Editing草案保留候选，不把新状态/Feature/Service/HMR并入UI014结构迁移。UI007的主题/示例范围已有独立合同，当前结构门具体为U014，之后按原合同确认与新基线重核；不扩大为整套新架构自动获准实施。

里程碑仍以有限合同的完整验收/独立终审/冻结门/合入/规范账本/归档guard为准。当前排程增加Web工作量，时间可能后移；不要求80万达标、历史债全清、数据/runtime另建仓或ACE/AutoGraphics全线。到点固定实际三仓及新增Web组合，读回剩余限制再重审架构草案；业务应用与必要修复继续，不等待所有治理结束。

### Cargo产物：当前批次后的研究与本轮近期M1出口

2026-10-30用户明确将生成Rust工程及Cargo重复产物问题列入本轮近期M1前需解决的事项。已读取其[来源对话](thread://01a15c20-6af6-7471-8af1-456696e05370?hostId=local)，其方案尚无源码/磁盘实测，“每工程约20GB”只作待验证观察。

U888且U890后研究，接受迁移后的真实公开基面；可与UI014无交集工作协调并行，正式构建/磁盘实验另预约，不污染888计量或在途冻结门。范围是稳定生成身份/路径与失效、共享target或兼容环境workspace候选、依赖/构建配置、并发隔离和有界回收、独立消费/发行/恢复及真实磁盘/周转收益。先查现有设施与Cargo实际合同，不预定唯一全局workspace，不自动改debug/incremental或清共享缓存。

调研/设计后按证据确认有限验收，需实施则主改面仓建立完整Plan，再以真实治理结果进入近期M1核查；研究文档/单候选no-go不代替问题解决。这里新增的是用户本轮近期里程碑要求，831/832历史M1及841–844有限M2交付不回退。888/890/UI014/007当前任务与AC不变，后续设计/计划同步具体里程碑门。详[研究交接](../../plans/reports/cargo-artifact-sharing-research-handoff-2026-10-30.md)。

## 3. 已裁定的验收口径

- **867 分母不缩减**：保留拆分前全部测试义务、来源/归属/实际执行状态；Man `t/tc/tcore/ti`、UI `tu/ti/tn` 及语言适用档/编译阻断等按 r2 完整登记。r1 的 8,315 枚举子集、list-only、将 th 编译红排除在义务外均不能证明全集运行。未解释缺口=0；新增红=0；既存红逐 ID 认证，消失红解释；编译阻断、环境缺失、ignored、未执行分列。未知或约定必需正控未执行仍阻塞。
- **869 规模与减法**：A 为约定范围全部 `.rs`（P/T2/XT/G/VEND，XO 另列）的披露/达标口径，B 为三仓生产 NET-PROD 不反弹约束，C 旧单 crate 仅历史对照；三仓同快照、同脚本/分类版本复算。测试、生成、第三方分列，迁移不算净删。历史约 100 万不能冒称当前实测；80 万是争取目标，不授权删功能/测试凑数或无限追加计划。具体完整口径仍看 869 §5.1。
- **869 批粒度**：每批 NET-PROD 净删≥2,000 行，安全候选不足则合批/重规划；结构性减法仍需替代路径、承诺迁移、门禁及用户按批签核。收据含前后值、删除 SHA、恢复配方/演练、推送状态；不强制 push。
- **872 保语义**：str.len、math、to_int/parse_int、time_now、HTTP error-reply、nv_to_value 六族全部 DISTINGUISH。先冻结名字→注册/覆盖→生效实现→语义→消费者→测试绑定表；旧名、优先级、错误和结果保持。MERGE 未授权，RETIRE 需无消费者证据与另行签核；plan755 HTTP body 不变。
- **873 先测后决定**：同版本分层测冷/热 check、build、测试二进制/链接、枚举与真实启动/初始化，至少七个真实源码编辑站点（语言≥4、UI≥2、Man≥1）；touch 只作诊断控制。独立 target/cache，禁止清共享 sccache；至少三个有效样本，污染样本作废。拆/不拆/需试点结论均允许，估算不冒充实测收益；不为测量新跑整档。

## 4. 两个结构工作包：884已收口，UI014已立约、实施等U890

语言结构包已由884有限交付：运行桥接、stdlib域装配、call_inner形状族与转译主管线职责收口，保872注册/名字/ID与生成结果。实际现状见language-sdk Spec与884归档收据，不再按旧“待立约”描述调度。禁止以873结论为由启动parser/vm/trans全面拆crate或新性能改造。

UI/Man结构包已正式落为[auto-ui/014](../../../../auto-ui/docs/plans/014-ui-man-structure-closure.md)，默认Man CLI→Vue生成→Aura构建→renderer。复用cmd_*/import_policy/shared，现有生产职责/状态/资源/测试与外部消费均须闭合，不以文件变短计功。设计见[UI仓结构边界](../../../../auto-ui/docs/design/autoui/ui-man-structure-closure.md)：890拥有Web领域迁移，014只整理其调用装配；007拥有全部examples归位与新主题，014只读回归现有示例，不提前搬迁/主题化。源码观察线索在U890后重新冻结。

873建议A1（根包默认ui-iced调整）另作签核候选，不并入当前结构/迁移包。细拆crate仅按873具名触发条件重评，mtime实验不动现网；测试治理按887义务守恒与实测范围推进，不以873重开分层或清共享缓存。869把U871泛化绑定A2R call_inner的旧前置已按实际写面纠正：871退役AC native HIR→CLIF，不自动退役Auto→Rust。当前范围见[路线v18](../../plans/reports/v06-extraction-execution-roadmap.md)及[结构包范围](../../plans/reports/post-873-structural-work-packages-2026-10-29.md)。

**语言结构884、测试治理887已归档，Web=auto-lang/890，UI/Man结构=auto-ui/014，主题/示例=auto-ui/007。** 不使用已给AutoGraphics的语言874/875，也不重复给014取语言号。Web保通用语言/Tauri/sidecar/shared workspace单源，语言基础不依赖Web实现、Man核心不拉VM/renderer；Design31远期范围不并入。现有配置/安全/预算/健康/关闭保持，见[Design50](../50-autoweb-service-framework-and-repo-boundaries.md)。ACE只按实际写面协调、不借WIP，独立终审/冻结门保持。

## 5. AutoOS 与应用可以如何独立继续

业务应用可以固定已交付三仓源码、锁、资源和工具版本，通过公开入口并行开发，不必等所有质量治理结束。共享 SDK/协议/renderer/native 修复由 owner 发布，消费方重放受影响范围；产品业务仓不复制私有模型/实现绕过接口。UI 已搬仓不表示所有应用、后端、平台、安装产物自动可用。

AutoOS [051 真实运行报告](../../../../auto-os/docs/plans/attachments/051/runtime-check.md)已经记录：新脚本真实构建三仓并启动 VM 桌面，桌面与独立 CLI 计算器均实测 2+3=5；OS main 交付 `3088dea`，报告 `234e5f6`。独立 VM 首次栈溢出在 UI 工具 owner 修复，auto-ui master `5a89bfd`，主栈 1 MiB→64 MiB，普通重建及真实交互复测通过。视频 playlist 解析错误、Music/Photos 扫描 404 仍 open，归 OS 债表；不称全应用通过。

876 已把 iced 桌面从语言 example 迁成 `auto-os/host/desktop` 的 `ui-desktop` bin，并交付启动健康面。合入收据为 lang `015884d45`、UI `f489c7b`、OS `1f9bd87`；后续主分支可再前移。只读核查确认 UI master 保留 `5a89bfd` 栈策略、OS main 保留 051 `3088dea`，现行两份 desktop 脚本 iced 轨均构建 `os-desktop --bin ui-desktop` 并从新产物路径启动。**051 报告里的旧 example 命令只作历史证据。** 本轮未重跑构建/GUI，不把代码及收据核对写成新运行通过。

AutoOS/桌面产品计划与债已按 876 在 auto-os 默认分支管理，auto-lang 只留指针；跨仓正文放主改面仓。OS876-APP-EXE/SUBMODULE 后续修复和守门也已有 OS main 收据；终端 Tick、use 解析及 Vue desktop-host 归位等仍按实际 owner 跟踪，不默认为 iced 迁移已覆盖，不追加为 869 的全局前置。

## 6. 验证与会话接续纪律

独立 scope 的 work/review 修复可以并行，同 Plan 同权威树单写者、冻结交接；共享 manifest、锁、注册、指纹/ledger 与默认分支接线排队。最终 review 前只做定向反例、模块测试、必要 check；final pass 后由 merge 在冻结提交统一裸 t 及实际触面档。tf 主检出到期单实例，873 机器测量另预约独占，不互相污染。

原始日志、探针输出、截图、gate/ledger 转储均本地保留；attachments 只允许提交 Markdown 摘要，含命令/退出码/数字/本地路径+hash。不为交接做证据历史过滤、force push、分支更名或缓存清理。主检出现有其他路线 WIP 必须保留。

新会话先读[当前路线](../../plans/reports/v06-extraction-execution-roadmap.md)、[交接prompt](../../plans/reports/extraction-refactoring-next-session-prompt-2026-10-28.md)、890及实际上游收据，以最新更新覆盖旧快照。维护总体规划与启动门；不要从777重新调研、自动执行后续治理包或认领其他会话实施。

</details>
