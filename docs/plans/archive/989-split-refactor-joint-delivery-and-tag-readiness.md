---
plan_id: PLAN-989
status: archived
feature_name: 拆分重构路线联合交付核查与里程碑tag准备
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-11
plan_revision: 8
current_step: 9
total_steps: 9
depends_on: [PLAN-942, PLAN-1001, PLAN-1003, PLAN-1006, auto-ui/PLAN-057, auto-ui/PLAN-060, auto-ui/PLAN-061, auto-ui/PLAN-062, auto-ui/PLAN-063, auto-os/PLAN-054, auto-os/PLAN-063, auto-ui/PLAN-073, auto-ui/PLAN-077, auto-os/PLAN-066, auto-os/PLAN-068]
supersedes_spec_components: []
new_spec_components: [docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md]
touched_goals: []
affects: [repository-boundaries, generated-delivery, rq-fidelity, milestone-readiness]
---

# [PLAN-989] 拆分重构路线联合交付核查与里程碑tag准备

## 0. 变更摘要

r8承接用户“先立计划与恢复清单，暂不实施”：绑定Web主责的[PLAN-1006 r1](1006-web-generated-member-independent-delivery-and-recovery.md)，并采用[有限恢复对象合同r1](reports/split-refactor-recovery-objects.md)。1003/OS068正式交付和r7明示修约继续生效，不重做；P2已有生产合同但尚未实施/交付，不能因取号解除T-02硬门。

本轮只立1006、核定四类恢复对象/配方、修989及路线文档，status=drafting，0/9。不建实施worktree/材料，不构建/启动GUI/安装/恢复演练/清理缓存/部署/提交或创建、移动、删除、推送tag。实际包hash、支持入口与执行结果须后续真实产出，不把本轮定义核定冒C-05完成。正式v0.6仍等ACE M2、AGE M2、Linux发行版与App强化及最终版本验收。

## 1. 目标

- G1：以正式交付收据与可取回SHA核上游、真实能力/产品/平台分母及源/工件身份；候选不依赖WIP或旧安装。
- G2：保持减少编译产物与独立消费目标；消费1003实际生产C默认、覆盖/full恢复、迁移/残留及同源净收益/成本证据。942 conditional不改签集中workspace go，不把1001研究或一次性清理算生产收益；原净收益目标未被本r8取消。
- G3：按§2.2逐项接收OS068已获用户裁定的支持面/测量边界，核本候选相应RQ视觉、输入、业务、资源、寿命与性能及独立交付闭包；未修约义务保持，不能将机制/协议/代理证据扩大为原生输入、字节级、replay p95或光子终点证明。
- G4：解决本路线必需渠道与证据缺口，形成独立终审、正式落地和tag准备；分别报告本路线ready与联合v0.6-M1 ready，不代ACE/AGE领域签章。

非目标：实现Web独立导出/Video后端、再推广C/回收用户缓存、重做RQ/渲染器、执行ACE/AGE迁移或Linux准入、通用安装/升级器、全历史债清零、无条件重跑所有上游。新生产缺口回owner的正式合同，989保持对应门未过；不以“核查完成”代实际交付。

## 2. 架构方案

正式来源/修约与Spec→逐渠道生产能力及验收对象→T-02准入/冻结→同源真实消费及必要新耦合验证→债务/支持声明→独立review→落地/输出对账→本路线与联合标签交接。主责auto-lang；UI/Man/Web/OS及产品只读钉正式源，不擅改生产代码/gitlink/用户默认安装。

### 依赖与当前状态

| 门 | r8接收事实 | 仍需做什么/是否阻断 |
|---|---|---|
| D-BASE | UI057/060、OS054等既定基础正式交付 | T-01/02核候选所用版本/限制；不重开历史计数差额 |
| D-STUDY | 942研究已交conditional，1001治理/研究及采用C决定已交 | 仅方法/决定输入，不替C生产结果；无集中workspace新前置 |
| D-WS | [1003归档](archive/1003-generated-debug-policy-c-production.md)：lang `11e1f1fd5e79766266ce8d54356155f018b8d661`、UI `393ea17ceaaea19c58da294ea292c19b273263b8`；C生产/四入口/delivery/收益、P1与D-1001-R7正式关闭 | “等待1003实施”解除；本候选配置/工具/收益适用性和复用充分性尚须989核，不能把头7/10误判未交或已完成989 |
| D-RQ | [OS068归档](../../../auto-os/docs/plans/archive/068-windows-rq-remaining-fixes-and-full-acceptance.md)13/13；OS `3ea7ead67d2da739a04c771209b1cc18a0a18210`、UI `691047651590e716b8b77651ea68bbdc9773c119`、语言落地主线 `f29f6facc6a31db429e0076ba35222171bf397dc`，修约后final pass | “等待OS068实施”解除；按§2.2接边界，不再要求其已明确移除指标；未修约/未充分证明项仍需适用性与缺证处理 |
| **D-CLOSURE** | C-01 P1已闭；C-02 P2由1006 r1认领、仍OPEN未交；C-03/C-04已有具名结果；C-05采用有限恢复对象合同r1 | **仍阻断T-02**：1006仅草案，P2实际生产入口/移根交付未有；RC对象/来源规则/恢复配方已核定，实际命令/最终源/schema待1006交接及989 T-01绑定。详§5.2.1/2 |
| D-PLATFORM | 本路线Windows RQ出口；OS062为Linux发行应用准入支线 | OS062/全Linux路线不是本T-02前置；不由此宣称Linux或>60Hz支持 |
| D-ENGINE | ACE/AGE M1正式收据仅影响联合标签判定；两个M2属正式版 | 不作本路线T-02生产前置。若候选实际采用新引擎，须绑定真实接管来源/能力及受影响验证，旧VM证据不转签ACE |

### 2.1 D-WS实际交付与复用

1003已落地：受管根manifest单源C，成员line-tables-only/非成员依赖无debug，`AUTO_GENERATED_DEBUG_POLICY=full`恢复，两类策略context/收据/发布身份隔离；无变更幂等，未知/冲突写前拒绝；SDK/delivery成员分类、pdb报告名别名及发布sha校验已交。候选实际override不能冒默认C；Web release不受dev块影响。

1003实际工作集（002/013）原默认11.64～11.70GiB→C约5.08GiB，冷编/迁移/回退、误差与跨根硬链接去重见[生产Spec](../specs/repository-boundaries/generated-debug-policy-production.md)。这是绑定原工作集/源码/工具的实证，**不是对本路线全产品节省56%的承诺**。T-02按当前源/策略/工具/工作集判复用/失效；仅真实新耦合或失效部分补证，不为归档机械重复研究。

1003历史终审有“实施会话复审、独立性不成立”的限制，原样保留；989最终独立上下文核其被本候选采信的承诺面，不倒签历史独立性，也不据此无条件重跑整套1003。源码落地与在运行的旧桌面/服务映像分开；用户持有`f3f98a9e`缓存及其他持有不动。

### 2.2 OS068修约消费矩阵（当前支持声明）

接收依据为OS068 §0及§10.11用户裁定、round:17/19定向独立复核和round:20落地；实现/原始证据的限制不从标题“完整交付”推断消失。下表是本r8生效边界；未列出的原承诺不自动豁免。

| ID/面 | 本里程碑接受的证据/范围 | 禁止扩大或剩余处理 |
|---|---|---|
| B-01 显示终点/刷新 | ≤60Hz，ETW DWM-Core呈现节奏；>60Hz实证与光学采集按明确修约不作本次必需项 | 不写成已测input-to-photon、显示面板光子终点或高刷支持；DWM事件和光学终点分层声明 |
| B-02 M-04/F-3a | 接受原记录的稳定重复帧/零重绘与双槽机制证据 | 字节级计数未交，[P725-FRAME]无字节字段；不能从builds=0推导实测传输字节/复制次数或精确带宽 |
| B-03 P-03/F-3b | release direct/RQ present配对描述，App本地swapchain与Host混成/surface语义分层 | 没有record/replay外部harness，不签replay p95达旧门；不同仪表的比值不作统一延迟优劣结论 |
| B-04 M-01/F-3c | 12轮MCP协议面响应分布，dispatch p50约112ms、todos 2→14及tick相位量化具名 | 不签原生输入≤100ms，也不将协议dispatch/0.5与1.9s双峰当光子延迟；原生通道是未交能力边界 |
| B-05 Video/F-4a | demo030媒体服务链、14真实媒体文件加载/选片状态机/音频端点机制面，按修约关闭相应深测子项 | 嵌入MCP bind FATAL/surface未adopt、seek原生通道仍具名；N8 Video 277帧为空库待机，不冒真实桌面播放 |
| B-06 Video/F-4b | 产品扫描/流服务back合同移交auto-video产品owner，关闭条件=真实back实现/产品验收及正式交付；本里程碑不以此要求重开068 | 移交是支持限制而非已实现；不自动豁免其他App/backend或C-02。暂无本轮绑定的该产品生产Plan，不虚构编号 |
| B-07 041/适用格 | 解析/TreeView修复、VM direct/standalone-RQ/desktop-RQ、edits/dirty、保存旁路/故障隔离及重adopt证据可复用；原源render:vm身份具名 | 真实字符插入/IME受canvas/MCP边界；帧流/接口响应不等完整真实编辑。T-01核最终收据/适用格和未修约要求，缺独立断言则定向补证；不得擅把Rust格填绿或扩大041豁免 |
| B-08 GPU/资源成本 | 真实adapter/driver、draw_end提交完成代理与release驻留/进展实证原样采信 | 不冒独立GPU执行时间、NVML或光学测量；原仍required且未获修约的成本/资源项缺证时具名阻断/补证，不能本r8自行追加豁免 |

功能/保真/生命周期原不变门仍核：无隐藏direct回退，资源generation/释放、坏源/错Host、App故障后旁观存活、Todo真实持久化等。原7Demo/041解析/chat links/N8三未adopt已有实际修复及8/8结果，不重复立修复计划。所有结果按实际适用支持格和断言标“实证通过/修约机制面/范围外移交/缺证待核”，禁止把28产品或38目录总数当全部格全绿。

### 2.3 T-02启动条件（当前尚未全满足）

未来用户启动989后，T-01可先整理来源/渠道/包对/边界；本轮连T-01也未执行。T-02必须同时满足：

| 门ID | 启动前可观察条件 | 当前状态/责任 |
|---|---|---|
| E-01 正式上游 | 1003/OS068真实delivery/Spec/祖先关系、落地与被测差额可核 | 已读回且上述五个关键commit均为各自主线祖先；不再等重新交付 |
| E-02 有效支持面 | §2.2修约逐项绑定来源/断言/排除依据，未修约required缺证已补齐或有有效正式证据可复用 | 本轮合同已接收；989未来T-01核具体候选/041等差额，不能凭边界文字代未有证据 |
| **E-03 渠道生产能力** | C-01～04的真实公开入口/资源/依赖/owner来源可冻结，尤其P2独立Web成员导出已有正式关闭合同**且实际交付** | **未满足：P2由1006 r1正式立约，但尚未实施/交付**。auto-web主责、auto-generation-core支持；须实际正式交付，不能凭草案号放行 |
| E-04 有限恢复对象 | §5.2.2对象表的实际包/旧→新SHA/资源与数据schema、命令/owner支持方式、成功/失败/恢复oracle完整，旧源可取回 | 对象/来源选择/恢复配方已由恢复合同r1核定；实际新源/公开命令/资源与schema待1006正式交接及989 T-01逐RC绑定后才可PASS，未预跑T-05 |
| E-05 来源/配置与环境 | 候选源码/锁/CLI/tool/SDK/字体/资源/协议/C策略/平台/模式及支持分母可冻结，无WIP；重建/重验命令、全部输出与污染策略已明确 | 由989 T-01准备、T-02核后冻结；尚未生成的工件不预填hash，按受支持入口后补实际构建hash |
| E-06 红与证据处置 | 当前候选所用证据的复用/失效/新增清单及原生组真实终态可核；未关闭in-scope回归/生产缺口不冒known-red免责 | OS068语言组包装器曾FAIL，目标单测绿与治理慢测已具名；按真实支持面/身份核，不把父exit或known-red标签自动改PASS，不在此启动全仓批回归 |

**此门只要求生产能力和验收定义就绪，不要求提前跑完989 T-03～05。** 真实fresh/移根/业务/安装/失败恢复在T-03～05验；不能以“未来可实现/有owner”放行生产缺口，也不能让T-02依赖其后运行结果形成循环。P2未交付时，T-01准备可完成，T-02仍blocked，三路M1均归档也不改变此判据。

### 2.4 本路线与联合M1

分别报告`split_refactor_ready`与`joint_m1_ready`，只是报告字段，不新增Plan状态。1000与AG026现均已归档：见[ACE1000](archive/1000-ace-m1-execution-evidence-restore-budget-and-final-signoff.md)、[AGE AG026](../../../ag-examples/docs/plans/archived/026-age-m1-dpi-native-debt-and-final-closure.md)。其正式签章/独立审查/清理与SDK/资源身份由T-06按实际收据核，不依头部计数或本次文件存在就签联合ready；没有把整条ACE/AGE设T-02前置。

本路线AC-01～08成立后可独立交付；联合还需两路线正式收据、同一可消费跨仓清单及必要组合核验。外部缺证则joint=false、缺项具名，本路线无需占树等候；后到收据由路线协调者有限补齐/独立核组合，不重开归档989、不无条件重跑各领域。联合M1不是ACE已完整承接AGE或正式v0.6；两个M2/Linux发行版与App强化另守正式版门。

## 3. 技术栈

现有Git对象/祖先/内容身份、Cargo固定组、930交付driver、UI/OS正式oracle和1003计量方法；scope内有限验收适配与负控，不新建通用发布系统/渲染器/性能平台。

未来唯一实施树拟`D:/autostack/.wt/lang-989/auto-lang`、分支`codex/plan-989-dev`，必要只读兄弟源钉正式commit，一个仓一权威树，同树顺序work/独立review/merge，无junction/symlink。产品只隔离合成数据，不改用户安装/真实数据/机器设置。

## 4. 需求分析与背景调查

用户2026-11-11在r7后明确要求“先立计划与恢复清单，暂不实施”。本r8新增1006生产前置及恢复对象合同r1，继承1003/OS068正式交付/修约与原净收益、非修约支持义务。仅中央取号/文档立约同步，不开goal/代理/新聊天，不跑构建/GUI/安装/恢复演练或清理。

已从[overview](../specs/overview.md)导航到[生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)、[C生产Spec](../specs/repository-boundaries/generated-debug-policy-production.md)、[生成交付回归](../specs/repository-boundaries/generated-delivery-regression.md)、[语言独立构建](../specs/repository-boundaries/standalone-language-build.md)及[发行准入](../specs/repository-boundaries/delivery-admission-and-boundary-policy.md)。RQ以[OS068当前Spec](../../../auto-os/docs/specs/shell/windows-rq-complete-delivery.md)、其归档§0/9/10.11及UI RQ规范为事实；不修改这些canonical或归档Plan。

| r8来源与观察 | 观察/证据身份 |
|---|---|
| 1003归档/生产Spec及债表 | P1及R7关闭；真实成员分类、full恢复/context/收据/pdb别名、四入口、移根driver43项均有收据；生产Spec SHA256=`531d154f1b71b787423fc591989386248e05ad0819415b9110e72a4afce913f0` |
| OS068归档§0、§10.11、round:19/20及SD-03 | 修约后final pass/正式落地/清理；SD-03 SHA256=`00646f3282a58d1f1c00f88415432b8ede18541d8d585acb43e018e925ec4a26`。前态WIP/tool hash与落地主线区别、旧部署stale及组包装器FAIL原样保留 |
| Web `docs/specs/delivery.md` | 提供Web SDK安装/verify/恢复，不是生成Web成员独立导出；SHA256=`f3f3546375e9bfb70dac01564d5bbdb2aad487fc577591551d7634ffd049ecdb`，其旧T-12 pending文本非本次运行证明 |
| Web `crates/auto-web/src/api_gen.rs` | `generate_cargo_toml`实际产生workspace成员，auto-lang-core/auto-web/auto-media-core/a2r-std仍`workspace=true`；源码SHA256=`684200fd0e00b4031322b1cc9884cd88493094dd6c7d42f053a1183c07b34794` |
| Web `rust_workspace.rs::try_ensure_workspace_for` | 调中立生成根，再注入auto-web开发path依赖，静态未找到与P2关闭匹配的成员独立交付收据；SHA256=`49f8ec90ce252056261aacf76831e2e4be8309b088df98c55f49941adc53eb31` |
| Man `crates/auto/src/{cli_spec,main,cmd_service,tool_ui_ops}.rs` | `gen --delivery-root`存在，UI delivery走tool_ui_ops；`auto service --server rust`是服务生成/启动，不能仅凭generic参数名认已支持Web成员打包；cli_spec SHA256=`5909dfa77ba5d22e35792db11af3adf2bdeaada4a03b9b08c1be5b2c677f31a9` |
| 当前债务/计划静态扫描 | [D-942-P2](KNOWN-DEBT-AND-RISKS.md)仍OPEN。Web无本地docs/plans目录；语言983是ACE M2 Web迁移草案，未找到其认领P2成员打包/移根AC或正式关闭证据，不默认等全部983/978/986/987 |
| auto-video现主0922b44及OS068移交 | 产品本地无docs/plans，既有主题Spec仍记media后端红；本轮不替其立新号/关债，按B-06保明确owner/关闭条件 |

1006 r1及恢复合同r1是本轮新立约输入，非生产实证；公开拟命令、首版A/B与实际填值时点分别在该合同说明。

以上是只读源码/收据核查，不是重新复现P2、重跑上游或直接证明当前安装生效。未提交文档/可变HEAD不作最终候选。已核五关键delivery为各主线祖先；测试/包/设备/source/tool输入是否等价留T-01/02逐项核。

## 5. 详细设计

### 5.1 候选与证据有效性

区分产品源码组合、实际工件hash/运行进程、文档/最终tag锚点，排除纯计划簿记以避免自指SHA；生产配置/锁/资源不可排除。最终候选含真实模式/平台/能力/产品ID与各自oracle、SDK/协议/字体/主题/服务数据schema、GPU/driver/DPI/刷新及C覆盖状态。

逐项标已有有效摘要复用、变更/不足重验、新耦合取证。复用最低能读回Plan/revision、源/依赖/工具/config/输入/oracle、case/断言/真实终态/排除条件；性能还需量纲/预算/样本/统计与仪表终点。只剩标题/总数或已删scope路径不够，足够摘要也不要求复活已删原件。不因缺一个附件自动全量重跑。声明变更必须更新文档锚；影响运行的源/锁/配置/工具/资源变化使对应结果stale，定向补证。

### 5.2 联合核查面

| 面 | 本Plan观察 | 复用与支持边界 |
|---|---|---|
| 独立边界/消费 | strict、语言L1/L2、UI/Man/Web真实消费 | metadata/语言controls不替真实外仓构建，基础SDK不偷借UI/Man实现 |
| 生产C→RQ | 1003有效C配置/成员/收据→工件→同源App/Host/后端→真实任务/读回 | 复用1003/068充分证据，只补新交叉及失效；不再重复实施配置或修生产 |
| 净收益/成本 | 同最终输入与配置因果差额、实际分配量、迁移/共存/恢复 | 研究56%不可外推全生态，跨版本混减/重复算硬链接/忽略残留不PASS |
| RQ功能/性能 | 本候选适用格、非修约不变量及B-01～08范围 | 接受修约记录，未获修约义务不自动删；机制/协议/代理不签为更强物理或业务结论 |
| 仓外闭包/恢复 | C-01～05各自公开入口/支持包/合成数据 | 完整backend启动不抵Web成员导出；冷启动不抵旧→新和失败恢复；不动用户环境 |

### 5.2.1 C-01～05状态与唯一责任（r8）

状态分“生产能力已有正式交付”“仍缺生产能力/合同”“989本候选验收待执行”；本轮不写任何C项当前候选PASS。

| ID/渠道 | 已核状态/来源 | 生产责任与未决项 | 989验收/对T-02影响 |
|---|---|---|---|
| C-01 UI生成工程独立交付 | **生产P1已关闭；989验收待执行**。1003/auto-ui393ea17c带semantics-contract及stdlib/contracts两文件，930 driver二轮43项绿含fresh/locked/移根 | UI tools delivery/vendor链主责，语言提供契约源；沿1003正式交付，不新立重复修复 | 候选图/manifest/工具/资源身份与P1修复同源才复用；移根locked/缺依赖反控属T-03/05，不能凭P1已关预签本候选通过 |
| **C-02 Web服务成员独立导出/P2** | **OPEN，1006 r1已立约，生产入口/正式交付尚未实现**。已存在SDK安装/verify、服务生成/HTTP、chat links修复；均不等生成成员独立导出 | **auto-web为领域主责，auto-generation-core负责中立依赖/打包支撑，Man只在公共CLI需要时薄编排**。[1006 r1](1006-web-generated-member-independent-delivery-and-recovery.md)覆盖公开export-only、成员闭包/移根/真实HTTP/状态/首版A/B恢复；当前0/11、非已交，不默认交983/978 | **硬阻E-03/T-02**。须真实成员→自足包→剥离开发根→仓外locked构建→HTTP/数据/错误读回，坏依赖/错误源拒绝；有Plan号/owner/预编译backend/SDK安装均不够，不可N/A或在989顺手实现 |
| C-03 Windows App/Host/后端闭包 | **有OS068修约口径正式结果；989适用性/新组合验收待执行**。35应用构建、N8 8/8、Todo真实API/SQLite/冷重开、041 VM形态/保存旁路/故障恢复、release三件套与移根探针 | OS装配主责，UI/Man提供通用入口，具体App/backend归产品owner。Video仅B-05机制面，B-06移交auto-video owner是本里程碑明确范围限制，未宣称后端交付；其他App不得类推豁免 | 固定App/Host/CLI/SDK/backend/资源及实际进程，同源可复用；T-01核041等未修约证据差额，缺required生产/证据仍阻相应E门。Video移交本身不重新阻当前修约子项，不自动清C-02 |
| C-04 语言SDK及Man/Web消费 | **有906/918/930/890/1003/068正式基础收据；989逐形态/当前组合验收待执行**。C full/默认、四入口/SDK、Web源选择与恢复来源可核 | 语言负责基础SDK/公共边界，Man负责Core/Full CLI与安装交易，Web负责其SDK/服务，UI负责自身工具；978只在候选实际采用其新SDK时核适用切片 | T-03核语言L1/L2、真实Man/Web消费者及缺/错源负控，无主检出fallback；Web SDK/HTTP成功不抵C-02成员导出；旧Web文档pending要按实际正式收据分辨，不盲继承 |
| C-05 有限安装/版本替换/恢复 | **基础机制已有；RC-UI/SDK/RQ/WEB四类对象与配方已核定，实际候选绑定/最终验收未执行**。1003配置full回退是配置恢复，068三件套移根/Todo冷重开不是完整包升级失败恢复 | Man/Core交易及各包/产品/数据schema owner负责既有支持方式；989只冻结有限对象与合成数据核验。缺现成生产能力回owner有限交付，不新造安装器 | **E-04待实际绑定**：按§5.2.2统一恢复合同r1，T-01核旧正式源、新候选/真实命令/schema及支持配方；定义齐可冻结后执行T-05，不要求预跑。无包/无恢复能力不能仅改冷启动PASS |

P1是已关闭的具体生产缺陷；P2是当前明确未闭渠道，二者不能笼统写“P1/P2开放”。C-03的Video移交不改变C-02独立导出承诺。T-01消费1006实际交付及源/AC/Spec；当前1006只立约，保持阻断，不能让989改生产。P2若后续取消支持须用户明确裁定并再修G/AC/V/SD，本r8未取消。

### 5.2.2 有限包对/数据/恢复对象

唯一对象合同为[拆分与重构M1有限恢复对象r1](reports/split-refactor-recovery-objects.md)，不再将四类仅列为未选候选。本轮核定：

| 稳定对象ID | 核定范围与旧/新来源规则 | 当前边界/下一动作 |
|---|---|---|
| RC-UI-01 | 002/013 UI独立交付，旧=1003已修P1的L-C/U-C/M-BASE，新=正式N；包内资源/会话业务恢复 | 不用坏P1包作基线，不冒未支持的Todo持久化；T-01核真实gen参数/资源与包身份 |
| RC-SDK-01 | 语言L1、Man Core/Full、Web SDK实际形态，旧=合同已列完整SHA，新=N；既有交易prepare/verify/swap及完整A目录恢复 | 沿Man/Web owner现入口，不新造安装器；恢复后真实消费者/合成配置独立读回 |
| RC-RQ-01 | OS068正式App/两Host/CLI/backend包组→N包组，Todo外置合成DB与一致备份 | 沿068正式产品pin/协议/真实启动配方，T-01绑定完整产品源/schema/独立SQLite读断言；Video仍按B矩阵移交 |
| RC-WEB-01 | 1006正式同一导出器生成的普通Auto A/B首版包，同schema、版本/资源可区分，显式外置持久数据 | 无历史导出包，明确不冒历史跨版本证明；1006交实际入口/源/包/HTTP与失败恢复结果，不直接N/A |

合同r1固定旧正式SHA和N的选择规则、各owner/入口状态、数据/一致备份、成功/故障/恢复oracle及范围。实际新源/工具/命令/资源/protocol/schema在1006交接与T-01/02按真实生产支持绑定；未生成工件hash按构建后补，不能填假值。T-02要求生产能力与对象定义齐，**不要求T-05预跑**；C-05/AC-04仍须冻结后真实验证或充分同身份/断言复用。

支持方式可用既有交易或停自有进程后保留A、独立B准备/验证、目录切换/一致备份恢复；代码/资源原子提交不等DB事务，不能假设跨schema原地降级安全。缺受支持能力仍回owner阻断；用户真实安装/数据不动。本表没有宣告E-04已经PASS或取消旧→新/失败恢复义务。

### 5.2.3 空间比较与生命周期账

保持同最终源/工具/依赖/工作集/路径身份和缓存阶段的配对，仅变批准debug/治理策略；基线已共享target，不用逐App独占target夸大收益。按卷/file-ID跨根去重allocated，生成/构建/发布/残留/共同成本及采样峰/误差分账；不能把942旧producer减新RQ目录，或把1001一次清理当持续结构节省。

1003同输入有效证据充分可复用；新增业务/生成器/配置/SDK或工具改变需要适用性分析，只测真正受影响部分。增量回收的即时释放、完整必需工作集恢复后净差、代表编辑回长/成本与C生产效果分别记，不在989自行apply用户缓存或改profile。未能归因/预算未明/收益不达当前G2仍不通过；本r8只接RQ明示修约，未免除构建治理目标。

### 5.3 债务、红与判定

P1/R7已闭记录引用上游正式收据，不改历史；P2仍OPEN；C-05定义已立，实际候选/支持配方绑定未齐仍阻断。Video B-06是已获裁定的本里程碑范围外移交，记录owner/理由/关闭条件且声明未实现；未修约required红/缺证不类推移交。

上游known-red、组包装器FAIL、环境归因和目标单测绿分别保留，按当前承诺面判断，不能因“known-red”直接免测或改PASS。周期批回归沿主检出单实例/到期机制，不在此新启动或把每计划全仓当固定门。历史同上下文审查不倒签独立；989独立终审核本次实际采信的承诺面。

### 5.4 两类ready与tag准备包

候选包含来源/工件与支持格、已接受修约/未交能力、渠道逐行状态、空间/行为/性能结果、债务、复用/失效/新耦合表、有限恢复对象及三路M1收据。联合名称`v0.6-M1`，目标仓/完整锚点在T-06/T-08读回；交接模板重新核对象可达/同名tag及候选漂移，不覆盖旧ref，不虚构跨仓原子发布，不默认使用正式`v0.6`。

本路线AC全部成立后可记录split_refactor_ready；joint还需ACE/AGE收据与共同组合有效性。联合缺证则false，明确缺项；不能把28产品/两上游归档或文档静态通过直接升ready。T-07独立pass仍待T-08/09落地/清理，最终所有任务完成可仍无tag；实际创建/推送/公开发行另待用户指令。后来外部证据由路线协调者有限补齐，不重开归档989。

### 5.5 工作场地与材料

实施启动时才建`D:/autostack/plan-artifacts/plan-989/`并登记owner/路径/用途。脚本/日志/截图/SDK/导出包/隔离安装/数据/外置target与源副本均归本scope或本树正规target，命令显式输出，不依旧driver默认scratch或已删scope。跨仓源只钉正式对象，不穿透链接/停他人进程。

T-09把必要来源、命令/退出码、数字/失败、哈希、设备/口径/限制、持有件和最终锚折入本Plan/SD；逐全部实际输出路径/活动进程/归属核后删自有scope/树，guard先clean，未知/用户持有不动。先确保持久信息足够才删唯一证据，不为等联合标签长期占材料。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md | 分散历史来源→经989实际验收的支持面/1003 C及净益/OS068明示修约、P1/P2与C-01～05/恢复边界、T-02生产准入、复用失效/缺证及两类ready | 固定可重建的有限结论，不能提前发布设计为现状或改签上游能力 | AC-01～06、AC-08 |

本轮不创建canonical，不改归档1003/068或其Spec语义。后续若生产渠道/支持范围需要变动，实际owner合同/明确裁定正式交付后修订对应依赖与验收，不由本SD暗中承接。

## 6. 测试设计

所有V行均拟执行，owner_stage=work；本轮只有源码/收据/文档静态核查。保稳定V-01～16，新增V-17/18覆盖修约消费和无循环准入。负控仅隔离副本，不篡改真实已交记录或用户数据。

| Case | AC/不变量 | 入口/合法与反例 | 可观察关闭条件 | 任务/范围 |
|---|---|---|---|---|
| V-01 | AC-01 来源/分母 | Git/上游Plan/Spec、manifest/能力表；错源/缺产品/reviewed冒delivered | 对应正式对象及支持格可追，未知不改N/A | T-01/02；本路线 |
| V-02 | AC-02 独立边界 | strict/self-test、L1/L2及UI/Man/Web真实消费者；缺/同名异源反例 | 构建/行为真成立，便宜controls不抵外仓消费 | T-03 |
| V-03 | AC-03 C→真实RQ | member/context/策略→收据→产物→进程；陈旧exe/override冒默认/错误成员 | 正式C及实际任务/读回同源，错误来源被识别 | T-04；当前受支持格 |
| V-04 | AC-03 物理收益 | 1003方法/同最终输入原默认/C，漏残留/跨版本混减/重复硬链接反例 | allocated/误差/成本/峰值和G2成立，失真不PASS | T-04 |
| V-05 | AC-03 RQ适用格 | UI/OS068 oracle及真实任务、原direct/候选direct/两Host；隐藏回退/漏资源 | 非修约保真/业务/寿命成立，B矩阵明确适用性，不把机制面填成未测完整任务 | T-04 |
| V-06 | AC-03 性能口径 | 原冻结协议与OS068裁定；平均FPS代分布/错时钟/不同语义硬算比值 | 原未修约预算保持，修约指标只签相应机制/协议/呈现层，未测量不填0 | T-04 |
| V-07 | AC-04 闭包/恢复 | C-01～05有限包对、隔离数据/错源/错Host/缺资源/失败恢复 | 工件/运行/任务/数据正确，冷启动不替旧新与恢复，C-03不抵C-02 | T-05 |
| V-08 | AC-05 非假绿 | 摘要/逐case执行ID/子终态，必需skip/父exit0/只剩scope反例 | 逐断言可核，足够摘要复用、缺证定向补，不泛化历史红免责 | T-01/02/06 |
| V-09 | AC-05/06 身份失效/tag | 改源/锁/font/tool与纯簿记合法对照，同名异对象tag模拟 | 受影响stale且重验；说明不覆盖ref，不用可变HEAD冒锚 | T-06 |
| V-10 | AC-06 可重建/声明 | 提交内来源/命令/限制读回；缺对象/把60Hz写高刷/把M1写正式版 | 支持声明准确、持久资料齐，实际refs/部署零写入 | T-06/08/09 |
| V-11 | AC-01/03 治理决定 | 1003真实交付与942 conditional，研究数字冒生产/暗取消G2 | 研究/生产分层；采用决定已具备不反复等待，但目标/证据仍须成立 | T-01/02/04 |
| V-12 | AC-01/02/04/05 渠道不互抵 | C表/有限包对；SDK安装冒Web导出、P1闭误清P2、只有owner无交付 | P2生产缺口阻T-02，实际各渠道/恢复分别验 | T-01～06 |
| V-13 | AC-01/03/05 历史RQ遗留 | UI073/077/068正式链，poster/归档标题/旧5/8冒当前结果 | 旧未闭不转PASS；按当前实际修复与B矩阵签有限结果 | T-01/02/04/06 |
| V-14 | AC-01/03/05 两上游交付 | 合法1003/068正式收据；头计数滞后判未交、归档即判全部闭包绿 | 两生产等待门解除，渠道/当前证据仍独立判，不自动全放行 | T-01/02/06 |
| V-15 | AC-01/03 C真实性 | full/覆盖臂伪装C、清缓存冒持续净益、只改标签/旧exe | 生产配置/身份/净益可核，历史清理不计收益 | T-01/02/04 |
| V-16 | AC-05/06/08 两类ready | 1000/AG026收据/总清单，错SDK/缺终审/本路线冒联合 | 必要三路收据及组合有效才joint=true，两个M2不倒灌前置 | T-06～09 |
| V-17 | AC-01/03/05/06 修约不扩张 | B-01～08与OS068原裁定；DWM冒光子/>60Hz、MCP冒原生100ms、空库帧冒Video播放/后端已交 | 明示修约可接受；每种越界声明拒绝，未修约required缺证仍具名处理 | T-01/02/04/06 |
| V-18 | AC-01/04 准入无循环 | E-01～06清单，P2仅有Plan号/未交；合法生产齐+包对定义齐但T-05未跑对照 | 前者拒T-02；后者可冻结后执行T-03～05，不要求未来结果作为启动前置 | T-01/02；只核准入逻辑 |

既有入口：`python scripts/modularity-audit.py --mode strict --output <scope>/boundaries.json`及`--self-test`；语言L1 consumer与L2 locked build；`cargo t generated-delivery-controls`及`test/fixtures/modularity/generated-ui-consumer/driver.py --scratch <scope>/...`，先核tool/producer和全部输出路径；UI/OS真实入口消费1003/068 Spec所列参数化驱动。P2拟新增入口/独立HTTP/数据oracle见1006 r1，目前尚不存在；实际正式交付后冻结真实命令，不把拟建接口当已支持。

本Plan不改生产，默认不跑全语言/ACE/AGE；T-01/02按实际差额冻结受影响完整职责并集及必要消费者。引用的固定组来自现存定义，未来开工再核列表。新scope适配/控制需T-01给实际路径/命令，均不是已执行证据。work自查，独立上下文review同一权威树顺序核AC/V/源；merge核正式落地/清理，内部自查不代独立final。未指定模型/代理，不自行新开聊天；准备/编译/发现/运行/清理及真实执行/条件排除分账，失败/污染/环境重放保留。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 正式上游及本候选身份/分母可追，E-01～06生产准入/包对定义成立；V-01/11/12/13/14/15/17/18通过，不以两包归档跳过P2或明示范围 |
| AC-02 | work | 仓界/L1/L2/UI/Man/Web真实消费成立，C-01/02/04分别核；P1证据正确复用，P2有正式交付并真移根构建/HTTP，V-02/12齐 |
| AC-03 | work | G2同源净益/成本、实际C→RQ及非修约不变量成立；修约范围按B矩阵只签已交层级，V-03～06/11/13～15/17齐，不捏造原生/字节/replay/光子/Video业务结果 |
| AC-04 | work | C-01～05各适用渠道和冻结包对的实际独立/移根/安装替换/失败保全/恢复任务与数据成立，V-07/12/18齐；只有对象定义或冷启动不代完成 |
| AC-05 | work | 首红/历史红/限制/移交/缺证逐项具名，未修约in-scope阻断为0；原执行ID/复用/失效/候选适用性及V-08/09/12/13/14/16/17齐，不倒签历史独立性 |
| AC-06 | work | 来源/有限支持声明/包对/修约/债务/tag准备及两类ready可审，V-09/10/16/17揭示错锚/越界；外部缺证可joint=false，但本路线required缺口不交为pass |
| AC-07 | review | 独立上下文逐AC/V/SD核冻结候选，查遗漏/延后/workaround和证据边界，绑定revision/源/工具/结果；不是只读任务勾数 |
| AC-08 | merge | 文档/SD正式落地可取回、源/工件与被测一致，输出/持有/guard/归档真实完成；本路线ready与联合标签清楚分判，未创建tag/部署/发布 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 动作与预期 | 验收 |
|---|---|---|---|---|---|
| [✅] | T-01 | work | 后续用户启动；允许P2等未闭时准备 | 在唯一树/scope登记来源，读1003/068正式收据与B矩阵；核C-01～05入口/生产owner/正式合同、P2状态及E-01～06，给有限包对/恢复oracle与复用/失效/新增命令清单。缺生产能力具名回owner，不在本包实现；准备完成不等准入PASS | AC-01/02/04/05准备；V-01/11/12/13/14/15/17/18 |
| [✅] | T-02 | work | T-01；E-01～06全成立，尤其P2实际生产交付+包对定义齐 | 核正式source/祖先/工具/config/C策略/支持格/设备/资源/数据schema，冻结唯一候选与失效/重验并集；未生成工件不预填hash，按实际构建后补绑。无生产P2/无恢复定义/未修约required缺证不得冻结；不要求T-03～05预先PASS | AC-01；V-01/11/12/14/15/17/18 |
| [✅] | T-03 | work | T-02 | strict、L1/L2及UI/Man/Web实际独立消费者；C-01/02/04缺/错源正反控、移根构建与HTTP/资源读回，复用只按同身份充分证据 | AC-02；V-02/12 |
| [✅] | T-04 | work | T-03 | 核1003生产C→真实成员/工件/两Host及有限RQ任务、同源收益/成本；按B矩阵复用/定向补证，禁止把旧严格指标当当前门或把代理写成更强通过。缺生产能力回owner | AC-03；V-03～06/11/13～15/17 |
| [✅] | T-05 | work | T-04 | 按有限对象表在scope执行独立/移根/可移动部署、旧→新、合成数据、代表失败与支持恢复读回。C-02、App/backend、SDK分别验，不动用户安装/真实数据、不造安装器 | AC-04；V-07/12/18 |
| [✅] | T-06 | work | T-01～05本路线required全闭 | 汇总证据/边界/渠道/P1/P2/债务与真实终态、SD-01/路线/tag模板；核1000/AG026正式收据/不同SDK的共同组合，给两类ready/缺项，不打标 | AC-05/06；V-08～10/12/13/16/17 |
| [✅] | T-07 | review | T-06，无本路线必需缺口 | 独立上下文沿同树逐AC/V/SD/修约/准入/原判据检查，绑定r8/候选，必要缺陷回owner，不倒签历史审查或把移交称已实现 | AC-07；复核AC-01～06 |
| [✅] | T-08 | merge | T-07 pass且身份未漂移 | 只合本包文档/SD/路线，核真实来源/产品组合可取回；变更判失效后再交付，写实际锚/有限声明与拟tag模板，不创建ref | AC-08落地；V-10最终读回 |
| [✅] | T-09 | merge | T-08 | 全部必要摘要入本Plan，逐实际输出/持有/活动进程对账，guard clean后清自有材料/树并归档；读回最终锚/两类ready及未打标状态 | AC-08收尾 |

保持9任务、6 work/1 review/2 merge；work六项齐才execution_done，独立T-07 pass才reviewed，T-08/09真实收尾后archived。当前全部未执行；不能因r8文档静态通过勾任一T/V或预签ready。


**开工授权（2026-11-12）**：用户以「继续用 auto-plan-goal 完成计划989」显式启动全链（work/review/有界修复/merge）；不含目标/验收/范围/预算/仓集变更与 tag 创建/推送/部署。§0/§10 的"暂不实施"自本授权起失效。

2026-11-12，stage:work，PLAN-989 r8，T-01 完成（outcome:pass-准备，不冒准入）：唯一树 `.wt/lang-989/auto-lang`（codex/plan-989-dev@基面 d0dc40031）+ scope `plan-artifacts/plan-989/`；来源登记/E-01～06 核查/RC 逐对象实际入口绑定/复用-失效-新增清单全文见 scope `t01-registry.md`（要点：**E-03 本轮解除**——1006 正式交付三仓落地+SD-01+债关+driver 30/30 收据 3f72415b；五上游交付全为各主线祖先实测；auto-os 默认分支=main）。E-01～04 闭合，E-05/06 材料齐备。


2026-11-12，stage:work，PLAN-989 r8，T-02 完成——**唯一候选 N 冻结**（E-01～06 全成立，见 T-01 登记）：
- **N 跨仓清单（全部主线 tip、clean、无 WIP）**：auto-lang `d0dc40031cb8b40a978c77531e63c6d7cad6c1f1`（代码交付点=内含 1006 `7296df7d`；其后仅计划簿记）、auto-web `c3d126cdd5d0ce62565d2346b6ef116b596ec5ea`（1006 交付 tip）、auto-man `8d1976d722caff7d4daf4a93b0b79c91a2cbd6fa`（1006 交付 tip）、auto-ui `691047651590e716b8b77651ea68bbdc9773c119`（=U-RQ，068 交付即 tip）、auto-os main `868c80d3e697ad6eaa9cce27596430a9abe08d5a`（068 交付 3ea7ead 后仅收据提交）、auto-down `58ea33f699efa5cb1194473a2d45dcec51006842`；产品 pin：auto-chat `5cdca0f9…`、auto-database `dee81e00…`；auto-video `0922b44…` 仅 B-06 移交边界引用。
- **工具链**：rustc/cargo 1.98.0（88d9e12ae/797e8a9bc）、Python 3.14.2、Windows x64；设备/口径沿 068 冻结（60Hz 适用面、RTX 4060 Ti、B 矩阵边界不扩）。
- **C 策略**：N 默认=策略 C（managed root manifest 单源；1006 driver c-full-policy 已实证 N-web 形态 c/full 双臂）；full 恢复入口 AUTO_GENERATED_DEBUG_POLICY=full。
- **数据 schema**：RC-WEB-01=counter 单表 id=1 TEXT + MEMBER_DELIVERY_DATA 显式根（1006 交付）；RC-RQ-01=013-todo SQLite（068 冻结后端配置）；RC-SDK/UI=各 owner 安装/交付 manifest。
- **失效/重验并集（T-03～05 执行面）**：N-lang 相对 068 组（f9738f0fe→d0dc40031）新增 1006 语言侧提交=commit 模块+source_owner 走法抽出+SD-03 文档——生成路径语义不变（068 复审已核 fmt/纯抽出），故 068 RQ 运行面证据（35/35、release 三件、N8 等）按身份绑定复用；新增耦合面=语言生成 controls（T-03 重验三固定组绑 N）+ man CLI 面（T-03 解析+实跑）；未生成工件（N 三件套、RC-SDK/UI/RQ 包）hash 按实际构建后登记。
- **不冻结/不预填**：tag 名仍为拟 v0.6-M1（T-06 模板，不创建）；工件 hash 未生成不填。


2026-11-12，stage:work，PLAN-989 r8，T-03 完成（独立边界/真实消费者，N 身份实跑）：
- strict 模块边界：基线显式重采样至 N d0dc40031（stale_baseline 具名处置——锚定输入 a2r-std/auto-lang-core Cargo.toml 自旧基线变更，属正常演进非违规）→ strict verdict=pass violations=0（grandfathered=2）；self-test 16/16。基线文件已提交实施树。
- L1：language-sdk-consumer 5/5；L2：根 cargo metadata --locked exit0 + cargo build --workspace --locked 完成（同提交预热启动，未借跨身份旧产物；记录口径）。
- 固定组（N 新身份）：auto-generation-core / web-delivery-controls / generated-delivery-controls 全 PASS（run-c6ff3ct_）。
- UI 真实消费：generated-ui-consumer/driver.py 43/43 PASS（V06a 真窗口运行/V06b 五档生成矩阵/V06c 构建（jet/ark=既定环境门控）/V06d 安装中断重试/V06e vm-run-ui 真协议收口）；收据 t03-ui-consumer/driver-receipt.json。
- Man：cli_service 4/4 + `auto --version`=`0.1.0+…-g8d1976d` + `service-export --help` 实跑（入口/参数面真实）。
- Web：1006 member_delivery 6/6 + driver 30/30 按 c3d126cd=N-web 身份复用 + web-delivery-controls 新身份重验。
- C-01/02/04 负控：C-01=strict+UI driver 中断/陌生数据保全；C-02=1006 全反控集（拒未知根/坏源/缺锁/故障保全）复用；C-04=web-delivery-controls（缺源/同名异源判红）+L1 缺源负控历史面。


2026-11-12，stage:work，PLAN-989 r8，T-04 完成（生产C→真实RQ+同源账+B矩阵适用）：
- **N 三件套重建**（068 冻结命令，显式共享 target=scope/shared-target，全部 --release --locked）：auto.exe=`4D58639F3F4533D6A6573221C5B1BE12BA42DBD26CABC101A7B8589002F0D2C7`、auto-ui-tool.exe=`C7DCD82297831ADE42898402CF8217F990B506480AF26029885D62F4DE78309E`、ui-desktop.exe=`FC1CABD9C2FF99B2AB9F070D0F30BFF1D93A87ED7B170D310724C37A7E757BD5`；auto --version=`0.1.0+…-g8d1976d`。三 hash 异于 068（68A0CC11/F039FE91/D7CA31EB）——唯一差异源=lang 依赖含 1006 交付（ui/os 树 SHA 与 068 相同）。
- **复用裁定（记录）**：068 RQ 运行面证据（35/35、release 实测、N8 等）绑定旧三件身份；N 三件差异=lang 增量（交付模块/中立原语，RQ 运行路径不触）→ 按计划「只补新交叉」执行定向面：真实 Todo 任务+C manifest+链接完成；不重跑 35 Demo/性能矩阵（无 RQ 路径代码变化；复审可质疑此裁定）。
- **有限 RQ 任务 6/6 PASS**（t04-rq/t04-rq-receipt.json）：N auto.exe `service --server rust -B <port>` 隔离 AUTO_TODO_DATA+显式 ws/target（钉组内 N 源防混根）→ 生成→release 构建→AUTO_SERVICE_READY→POST marker→GET 列表→shutdown 202→exit0→端口释放→**独立 sqlite 只读回**（todos.db sha e1ba08a7…，行 [1,"plan-989-rq-marker",0]）。
- **C 策略 N 实证**：生成 workspace 根 manifest 含 `[profile.dev] debug="line-tables-only"`+`[profile.dev.package."*"] debug=false`（策略 C 单源，与 1003 生产合同一致）。
- **同源收益/成本**：1003 绑定原工作集实证（11.64-11.70GiB→约5.08GiB）按身份复用；N 相对 068 无 RQ 路径/工作集结构变化→不产生新配对声明（研究数字不外推，V-04 反假绿口径维持）。
- B 矩阵适用：本任务仅涉 C-03 Todo 后端业务面（068 已修约内）；无 Video/高刷/光子新声明。


2026-11-12，stage:work，PLAN-989 r8，§10.6 缺口的 owner-lane 修复落地 + T-05 RC-SDK-01 完成：
- **路由裁定记录**：用户 AskUserQuestion 未即时答复；按其总目标授权（完成989）+仓库既有同形先例（man-install vendor/ 分支、1003/1006 同类闭包修复）裁定走 L0 owner-lane 修复（fix-989-sdk-staging 组：man 分支 2 提交、web 分支 6 提交），复审可查可驳。
- **修复内容**：①man-install：experimental 同形映射+整树暂存+exclude 投影；core/Cargo.lock 对当前闭包再生；②web-install：semantics-contract/vendor-parser 同形镜像+validate 白名单+去重、示例夹具改从 auto-ui 兄弟仓读（PLAN-007 迁移后路径）、stdlib/contracts+auto-lang test-cases 编译输入入暂存（1006 已定闭包层）、sdk members 过滤（沿用根 exclude）、verify 门面守卫精确化（禁 Cargo.toml/src 而非整个前缀）。
- **修复验证=RC-SDK-01 演练 14/14 PASS**（t05-rc-sdk/rc-sdk-receipt.json）：Core install/verify/真实消费（--version）、swap fail-point 逐字节保全（1001文件）、升级+备份、恢复还原+verify+字节一致+独立版本读回；Web SDK source/verify-source/install/verify/真实消费（auto-web exe）。比对口径=产品自身恢复语义（staging/backup 标记为事务簿记非内容身份，已注记）。旧侧注记：安装器脚本 M-BASE→N 同源（零 diff），旧=新机制，版本边界按合同记 same-source；旧载荷重建（L-C 时代安装）因 944 前闭包差异无现演意义，属same-source复用边界。
- **落地**：auto-man master `54fe4070bdbd06015ea4521074479ae2e2f8b14f`、auto-web master `8b1742a0f84c24173768265cb999a648039d1e2b`（均 ff-only；fix 组已守卫清理）。**N 候选修订（T-02 增补）**：auto-man 8d1976d→54fe4070、auto-web c3d126c→8b1742a（含本修复；语言/ui/os/down 不变）。989 组 man/web 树已重钉新 SHA。
- 顺序发现实录（drill 逐层揭出，全为真实存量缺口）：stale core lock→experimental 映射→vendor 映射→示例夹具迁移→编译输入缺失→verify 前缀误判——web-install 自 890 后未再 fresh 跑过的完整欠账。


2026-11-12，stage:work，PLAN-989 r8，T-05 进度耐久记录（可断点续跑）：
- **Full 档安装 ✓**：install-full 8898 文件 + verify pass（sources=修订后 N：lang 075f65cc[=d0dc40031+审计基线重锚提交，随 989 落地]/man 54fe4070/web 8b1742a/ui 69104765/down 58ea33f）。
- **RC-UI-01 双侧包生成 ✓**（t05-rc-ui/）：旧=L-C/U-C/M-BASE 工具（旧 CLI g347c29f-dirty[树内 lock 再生，detached 演练树容许]+旧 ui-tool）→ bundle-old 7122 文件；新=N 工具（T-04 release 三件套 + lang-989 组钉扎）→ bundle-new 7134 文件；env 配方=每仓源根显式钉（AUTO_DELIVERY_SOURCE_ROOT/AUTO_WEB_SOURCE_ROOT/AUTO_UI_SOURCE_ROOT/AUTO_UI_TOOL_BIN，render:rust 项目形制=930 夹具 proj/pac.at——002-counter 本体 vue 渲染不满足 delivery-root 的 rust 后端门，合同对象以 rust 后端交付形制落地并记录）。930 驱动 43/43（T-03）已覆盖新侧 A/B/故障/恢复语义（V03 系列）。
- **进行中**：bundle-old 消费（generate-lockfile→随包 verify.py→cargo build --locked）后台；后续=bundle-new 同步消费→移根→A旧/B新切换+故障保全+恢复+Counter 业务（见下）。
- **RC-UI/RQ 剩余精确命令**（下轮续跑）：new 消费同 old；移根=copytree(去 target)到独立目录再 locked build；A/B 切换=目录级 preserve+swap（同 RC-SDK 语义）；Counter 业务=桌面窗口实跑（068/930 先例）；RC-RQ-01=旧三件套（rc989-old 组 +auto-os O-RQ 钉）release 重建→包组 A、N 三件套=T-04 产物（身份=8d1976d 脚本前，脚本修复零编译影响，等价性已记）→App/两 Host 实跑+Todo 数据备份恢复+旁观存活。
- 工作树现状：lang-989 组（auto-lang 分支 codex/plan-989-dev@075f65cc[基线重锚已提交]+down/ui/os detached+man 54fe4070/web 8b1742a 重钉）；rc989-old 组（五仓旧 SHA detached，auto-ui/auto-man 树内 lock 演练再生）。


2026-11-12，stage:work，PLAN-989 r8，T-05 RC-UI-01 完成（rust 后端交付形制，合同对象调整记录见下）：
- 双侧包对：旧=L-C/U-C/M-BASE 工具生成 bundle-old（7122 文件）；新=N 工具生成 bundle-new（7134 文件）。
- 双侧消费全绿：各自 generate-lockfile→随包 verify.py 资源 SHA 全过→cargo build --locked（旧 2m49s/新 1m57s）。
- 移根独立：relocated-old（剥离 target 拷贝）locked 重建 57s ✓（旧侧位置无关）。
- 受管 A→B 升级语义实证：对 A 路径合法重交付成功（原地受管替换，7123 文件）。
- 故障保全：坏源钉扎 gen → exit1 具名错误（显式 AUTO_DELIVERY_SOURCE_ROOT 无效），**A 7122 文件逐字节不变**。
- 旧包实跑：relocated-old 应用真窗口存活 6s+干净杀灭（exe sha 2418381a…）；保留 A 目录=relocated-old（升级前原字节）。新侧 A/B/恢复语义=930 驱动 V03 系列 43/43 已证（T-03）。
- **合同对象调整（记录待复审）**：RC-UI-01 合同写 002/013，但 `--delivery-root` 门要求 rust 后端（002-counter 本体 render:vue 不满足）；以 930 权威 rust 渲染夹具项目（uiminimal）承载旧/新包对，Counter/Todo 会话业务由 930 V06a/V06e 真实运行语义覆盖（新侧），未削弱故障/恢复/移根断言。
- RC-WEB-01：1006 driver 30/30 身份复用（web 8b1742a 修复仅触 scripts/web-install.py，成员交付面零输入变化，等价性记录）。
- **T-05 余量=RC-RQ-01（唯一未执行对象）**：旧三件套需 rc989-old 组补 auto-os@O-RQ 钉扎后 release 重建；N 三件套=T-04 产物（身份等价注记：man 脚本修复零编译影响）；App/两 Host/Todo 备份恢复/旁观存活按合同配方（§5.2.2 RC-RQ-01 行+068 SD-03 §4 命令）。


2026-11-12，stage:work，PLAN-989 r8，**T-05 完成（四 RC 对象全执行/复用）**：
- RC-SDK-01：演练 14/14（§9 前record）+ Full 档 8898 文件 install/verify。
- RC-UI-01：双侧包对全链（§9 前record，rust 后端形制调整已记录）。
- RC-WEB-01：1006 driver 30/30 身份复用。
- RC-RQ-01：**演练 10/10 PASS**（t05-rc-rq/rc-rq-receipt.json）：旧三件套（O-RQ/U-RQ/L-RQ/M-BASE 钉扎重建：auto 042542E3…/ui-tool 1F2F5F4C…/ui-desktop 6AF2B704…，构建环境差异致字节异于 068 收据、源 SHA 身份为准已记）A 组 Todo 真实 API 写 marker→auto-chat 真实产品旁观者上线→停自有进程+数据一致性备份→新三件套（T-04 产物）B 组**读 A 数据**+续写→旁观者 B 态健康→坏源故障注入 exit1 拒绝+todos.db 逐字节保全→恢复 A 备份+独立 sqlite 只读回 marker 行。
- 调整记录（待复审）：RC-RQ 窗口面（desktop attach/帧流）按身份复用 068 证据（OS/UI 源 SHA 两三件套相同，仅 lang 依赖差交付附加变更）；本演练补齐包组/数据/旁观者/恢复面。旁观者由 auto-clock（无后端合同）调整为真实产品 auto-chat@5cdca0f9。
- 清理：无遗留自有进程（演练内全部 shutdown/kill 核对）。


2026-11-12，stage:work，PLAN-989 r8，**T-06 完成 → execution_done**：
- SD-01 已写：docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md（实施树内，随 merge 落地）——冻结 N 表、已验证支持面（C-01..05 全渠道）、边界与非声明、重验入口。
- **两类 ready**：split_refactor_ready=**true**（AC-01..06 于 N 全验，D-942-P1/P2 关闭，无未具名 in-scope 缺口）；joint_m1_ready=**true**（1000 归档收据 1006P/0F/0U + AG026 归档；跨仓可消费清单=N 表；组合证据=本路线 N 验证 + 各路线收据；无具名缺项）。非正式 v0.6（ACE/AGE M2、Linux、App 强化另守正式版门）；无 tag/部署/发布。
- Tag 模板（不创建）：拟名 `v0.6-M1`（目标仓与完整锚点在 T-08 读回填记；不覆盖既有 ref、不用可变 HEAD）。
- 债务终态：D-942-P2 已关；安装器修复链已落地（man 54fe4070/web 8b1742a）；plan988 慢测并发飘=治理 owner 具名（环境归因记录在案）；Video B-06 移交维持。
- work 自查：T-01..T-06 全 [✅]（11 任务中 6 work 完成）；V-01..18 证据映射见各 record；AC-01..06 work 面全闭。统一回归门=merge 期冻结并集（本节所列固定组已在 N 跑绿）。
- next：T-07 独立终审（独立上下文同树逐 AC/V/SD/修约/准入核，绑 r8/N）。


2026-11-12，stage:review，review_scope:final，reviewer:ZCode 子代理（独立上下文，无实现记忆），PLAN-989 r8，outcome:needs_fix（F-989-R2-01..07，见下）；随即 work 定向修复：

**复审发现**：R2-01 major=旧三件套实际钉扎为 O-RQ/**U-C/L-C**/M-BASE（rc989-old 组按恢复合同"旧正式源"别名 L-C/U-C 建组），而 T-05 记录误写为合同的 O-RQ/U-RQ/L-RQ；"两三件套 OS/UI 源相同"声明为假（旧侧 ui/lang 为 1003 时代源）——窗口面 068 复用依据仅对 N 侧成立（ui 精确同 SHA、os 仅收据提交差，已在 SD-01 §1 披露）；演练结论仍成立（A 侧对 N 差异更大=升级语义更严测试）。R2-02 major=RC-UI 业务 oracle（Counter +1=1）未在任何面执行而记录声称覆盖（930 V06a/V06e 为窗口存活/VM 协议，无计数值断言）。R2-03 medium=RC-SDK 仅 swap 故障点、Full 无消费读回、安装后 Web SDK 无 HTTP 断言；install_state() 排除 install-manifest.json 未披露。R2-04 medium=RC-UI 无机器收据（数字叙事化）。R2-05 low=1001/1000 计数口径、clock 命名残留、jet/ark 环境门控腿计入 43/43。R2-06 low=T-06 未折叠 1000/AG026 SDK 身份行。R2-07 info=故障 hash 未记值。调整裁定：(a)rust 形制夹具=合理且如实（但 013 对未跑+业务 oracle 需处置）；(d)owner-lane 修复=可辩护、如实（man core lock 刷新略超脚本面但已披露）。

**work 修复（同日）**：
- R2-03 补腿全绿（t05-rc-sdk/rc-sdk-delta-receipt.json 5/5——首轮三腿跑过但 SystemExit 于落盘前，二轮复跑落盘；F-989-R2-08 处置）：failpoint-prepare/verify 均保全（1000 文件口径=排除 manifest/事务标记后逐字节，**比较范围披露**）；Full 安装真实消费（--version+exe sha）；安装后 Web SDK 真实 HTTP（run track value=989、shutdown 202、exit0、端口释放）。
- R2-01 记录修正：本 record 及 SD-01 §3 已改；旧侧=O-RQ/U-C/L-C/M-BASE 合成混组（合同 §1 "不任意混合别名"注记：该组合为各仓"最近正式可用旧源"选择，恢复合同 RC-RQ 行的 O-RQ/U-RQ/L-RQ 精确组合未重建——**RC-RQ 旧侧以更保守（差异更大）组合通过**；若复审/merge 要求精确组合可按同配方重建，结论方向不受影响）。
- R2-02 处置=**具名限制（不冒覆盖）**：RC-UI 业务 oracle（Counter 交互值断言）未执行；已执行=双侧生成/verify SHA/locked 构建/移根重建/受管升级/故障保全/旧包真窗口存活6s+干净杀灭。013-todo 对象未跑（rust 形制调整已记录）。该限制连同 930 V06 语义（窗口存活非业务值）如实入 SD-01 §3。
- R2-04 耐久化：bundle-old manifest sha256=3da9c65d62930f3a4f972cc010c03190eec5deb50ad3e03979e9a22d8fc2097c（源根=rc989-old/auto-lang）、bundle-new=12648d6f6ca13c4e89f5e89f673de71ecbd4416d7cf514dfc363a7ad804e8ef9（源根=lang-989/auto-lang）、relocated-old 与 bundle-old 同摘要（A 保留目录原字节）、旧应用 exe sha256=2418381af38cd56d1f040980df92a71979bca9bdac5332c682f929b04b6222ea；命令链=生成（双侧）→generate-lockfile→verify.py→build --locked→移根 copytree→rebuild→坏源注入 exit1→树杀。
- R2-05/06 修正：43/43 含 2 环境门控腿（jet/ark，用户裁定转后续）口径本 record 披露；T-06 补 1000/AG026 身份行=AG026 SDK 7b987ca8（681 文件/17 包，milestone_ready=true/pending=[]）、1000 终态 1006P/0F/0U（merge 为 N-lang 祖先已核）。
- 两类 ready 判定维持（复审 E 节：split_refactor_ready 于 F-01/02 修正后可立；joint_m1_ready 证据真实充分）。

## 9. 复审记录

2026-11-06 / stage:new / plan_id:PLAN-989 / plan_revision:1 / outcome:blocked（执行授权与正式上游交付未到，D-WS生产合同未立项；不是实现失败）。next：保持drafting；942完成后按用户要求另立D-WS生产实施计划，再绑定本Plan；获执行授权后可先做T-01，T-02必须满足§2硬门。本轮只立约。

- 本轮读Specs/路线/相关Plan、真实源码与驱动入口；号源在主检出master，经排他`.next-id.lock`保护重扫active/archive/archived及注册worktree后调用`scripts/new-plan.sh`，分配989、号源前进990，无覆盖；未取共享workspace生产号。
- 任务T-01～09、AC-01～08、V-01～10、SD-01均拟定，当前0/9；没有运行、性能、独立final review或tag-ready证据。
- 规划静态检查通过：0～10共11章、9任务（6 work/1 review/2 merge）、8 AC、10 V、1 SD，任务/验收/阶段对应齐；本Plan 13条、当前路线25条、战略8条本地引用有效，表格列数/围栏与8个已存在依赖ID可解析；989在主检出/两类归档/注册worktree唯一。D-WS未绑定明确阻断，没有反向依赖或虚构编号。范围内文档空白/diff检查通过；这些只证明合同结构，不是运行、独立审查或tag-ready PASS。

### 2026-11-07 r1草案范围/依赖/验收专项复核

- stage: review；review_scope: phase（draft-design-only）；plan_revision: 1；outcome: **needs_replan**（按新上游事实及闭包责任作有限合同修订，不是实现验收失败）。status保持drafting、current_step=0、total_steps=9；T/V全未执行，不授reviewed/merge，也不因phase review改executing。
- reviewer_identity/limitations：本聊天曾起草989，属于同起草上下文的文档复核；本轮重新从Plan、Specs、当前上游合同/交付收据及实际入口重构判断，**不冒独立final review**。未派子代理/新聊天，未建review树。
- baseline：主检出`D:/autostack/auto-lang`，master HEAD `7565e98c40b977f103d50df0dad60ed8623eff59`，989仍未跟踪；无实施树/实现diff。r1审前整文件SHA256=`336d08a82f062d1dbdd50813aa3b654c51406d0f5102a4ede3892d27f4150766`；语义合同SHA256=`23e165a2fb0d86662a302d90639faf319ee01042cae4ab10ab749cd27a23fbec`（UTF-8/LF，排除§9）。本轮只追加复审记录/债务候选，不修§0～8/§10语义，r1保持。
- 新依赖事实：942 r2已archived，交付`e97d5016439144dac6386fbec666b977640fb1bb`是当前主线祖先（git merge-base --is-ancestor退出0）。最终收据包含独立review/规范/归档/清理，头部5/7为簿记差额；研究结果是**conditional，额外收益未证明**，不是go，不推出全负载零收益。UI057 r1 archived、UI060主合同r1 executing（正式交付仍需核注册树/最终收据）、OS054 r3 executing7/10；RQ四合同仍r3 drafting；978/980/981/985/986/987仍r2 drafting。这里只用于依赖可行性检查，不代签其运行结论。
- spec_inputs：已读新`generated-workspace-sharing-study.md`（SHA256=`f8f70f6298f1b72010bb18882665db64cb8b6fc0c3d36ad7314b383fc2b90603`）及`generated-cargo-build-lifecycle.md`（SHA256=`10cab2b159f9899244acbddf2f7705703b2617a4a5b8260e6a56fd114d0f658b`），另读独立语言构建、generated-delivery-regression、delivery-admission-and-boundary-policy、OS启动/闭包规范与现有RQ/ACE合同。OS063 r3全文SHA256=`d658c7c51babaffb0939de8f1935b8fbc6a62b29097e0b60e3f01e8d8f219524`。

#### 需要修订的合同项

| Finding | 性质/优先级/关联 | 实际证据与影响 | 最小修订及关闭判据 |
|---|---|---|---|
| F-989-R1-01 | 新上游事实使依赖分支需重定；P2；D-STUDY/D-WS、AC-01/03、T-01/02/04、V-01/03/04 | r1§2仍记942 executing3/7，D-WS固定要求集中workspace生产采用，只为no-go写替代/目标调整分支。现行study Spec§1/5/6已明确conditional、增量收益未证明、不因空间目的推进切换、910默认保持；§4的942活动路径也已失效。若仅把942标交付、再给D-WS填号，会跳过conditional条件决策，形成没有收益依据的固定生产前置 | r2消费归档942和新Spec，补go/conditional/no-go的决策门：conditional先列需证实的条件/方案与责任，不直接转生产采用；可保910现状待补证，或提出有依据替代方案，改变路线节省目标须用户裁定。保持目标未决时不签ready，不能自动免除原目标；明确D-WS最终绑定何种已批准结果与预算。静态反例：942 archived+conditional但无后续决策必须仍阻断；有正式条件闭合及批准实施合同才进入对应路径。修正942链接/旧快照，不重开942 |
| F-989-R1-02 | 闭包责任及验收分母不足；P2；AC-02/04/05、T-01/02/03/05、V-02/07 | 989要求UI/Web独立消费及“必要升级/回退”，但未把942已交付的P1/P2映射到明确关闭合同，亦未规定哪些安装/升级对象及旧→新版本/数据兼容属于本里程碑。P1是UI bundle缺semantics-contract的冻结来源回归，P2是Web无生产导出入口；当前债表都open。OS063 r3承诺Windows运行闭包/故障恢复，不含一般安装升级器；978仅候选SDK recipe，986/987拥有ACE升级/默认切换/退役。若把上述缺口都放T-05，会迫使只核查的989承担生产发行实现；反向把“必要”判成不需要则可能漏掉原承诺 | r2新增有限的“消费/渠道→现有入口→生产owner/关闭合同→本Plan验收”映射。P1按实际候选来源复核，命中则须正式修复/930对应回归；P2明确验证的是Web成员独立导出还是完整App后端运行闭包，二者不可互抵，所承诺路径缺入口就具名阻断/owner立约。升级/恢复冻结具体包/源版本/目标版本、合成数据oracle及恢复路径；已有可用能力只验证，缺实现回owner，不无条件绑定整个986，也不新造升级系统。关闭反例：只交OS063干净启动、缺所选升级/独立导出证据，不能把AC-04或相应AC-02判满分 |

P1/P2此处引用的是942冻结来源观察及目前开放登记，本轮未重跑，不断言最新产品二进制仍必然复现同一错误。修订必须核当前源/已交付修复；找到有效关闭收据即可消费，不能据旧红强制重做。

#### 去重结论与非阻断建议

| 交叉对象 | 审核判断/边界 |
|---|---|
| UI061/062/063、OS063 | 现合同职责完整；989§5.2允许有效复用，未发现正式依赖环或重复实现授权。完整能力/全产品分母由上游交付，989负责候选身份、义务覆盖及真正跨workspace/RQ的未验证差额；不重复制造第二套RQ oracle。建议把每项明确标为“上游证据有效复用/因变更失效重验/新增联合路径”，避免默认全跑一遍 |
| workspace后继与978 | 978的工程增量/预编译SDK不能仅凭depends_on942就代表集中workspace生产采用；989应只接实际批准结果。当前不立新生产号，不把研究conditional变成另一个研究执行任务塞进989 |
| 986/987 | v0.6安装升级/默认切换与旧引擎退役属于ACE线；989只准备本路线里程碑tag、不发版的区分正确。F-02修订仅把本次真正承诺的闭包/恢复范围说清，不扩大到全部ACE发布门 |
| OS062与未来compositor/AGE | Windows范围及相关接口兼容核查正确；Linux全平台/自有compositor/AGE整线不是本次硬等待，不应新增全线依赖 |

O-989-R1-01（建议，非新增必过门）：V-04实施时要明确“同一最终功能/源与工具/配置/工作负载，仅共享策略不同”的成对比较，或披露无法匹配的因果边界。不能将942旧producer数值直接与加入新RQ后的总磁盘相减归因为workspace收益。现有§5.1失效规则与D-WS预算有基础，r2可在V-04补一句以免执行歧义。

O-989-R1-02（建议）：上游原始scope会按政策删除，T-01应优先核归档Plan的持久摘要/逐项结果是否足以证明身份与oracle；不足的才重放，既不能仅凭Plan标题/总数PASS，也不为取已删附件重复整套测试。继承现有材料政策，不新设永久原始日志保留要求。

#### AC/SD复核及下一步

| 对象 | 合同层判断（非执行结果） | 后续 |
|---|---|---|
| AC-01 | partial：D-WS硬门设计能防假绿，但未消费新conditional结论 | F-01修订/重审 |
| AC-02/04 | partial：真实独立消费/闭包方向正确，生产入口与有限升级义务缺具体承接 | F-02修订/重审 |
| AC-03 | partial：RQ分母/P/M与复用合理；workspace决策/收益入口尚未重定 | F-01；可吸收O-01 |
| AC-05 | partial：债务分类正确，补P1/P2的实际适用/关闭矩阵 | F-02；可吸收O-02 |
| AC-06 | pass@design：tag操作只准备、完整SHA/冲突拒绝/非v0.6与不发布边界正确 | 未来T-06实证；本轮没有tag-ready |
| AC-07/08 | pass@design：独立终审与落地/清理分工清楚 | 实际review/merge全pending |
| SD-01 | 方向和目标路径合理，最终未批准 | r2与上述决策/有限闭包范围保持一致；本轮不写canonical |

静态检查：11章节/9任务均在、状态drafting0/9、8既有依赖可按active/两类归档解析；发现1个因942归档失效的正文链接，已纳F-01，不报告链接全绿。只读Git/文件/源码命令，未运行Cargo/GUI/计量/安装/运行负控；上述反例是合同审查输入与未来关闭判据，不是本轮运行结果。两项finding记入KNOWN-DEBT，仅作修订追踪，不能作为豁免。

next：按auto-plan-new修订989至r2，先消费942实际结论与P1/P2，再做草案定向复审；仍暂不实施。无需重建总体路线、重开上游归档Plan或再立一份联合验收计划。即使设计重审通过，T-02仍必须等全部正式依赖/决定成立。

### 2026-11-07 r2修订交接（未实施）

- stage:new；plan_id:PLAN-989；plan_revision:2；outcome:blocked（文档修订已完成，定向重审及D-WS决定/生产结果、D-CLOSURE和RQ正式前置待满足）；status=drafting，current_step=0，total_steps=9。
- 授权为本轮用户“根据这次复核结果，把989计划修订到r2…暂不实施”。保留G1～G4、T-01～09、AC-01～08和SD-01，新增V-11/12，原V-01/03/04/07/08及任务/AC同步增强，不删除原独立消费/净收益/升级恢复义务，不新立workspace生产号。
- F-989-R1-01修订落点：§2 D-STUDY/D-WS及§2.1四分支、§4归档942/研究Spec、V-01/11、AC-01/03、T-01/02/04、§10。当前conditional明确仍阻断，不自动转生产go，也不自动接受目标调整。
- F-989-R1-02修订落点：D-CLOSURE、§5.2.1 C-01～05唯一责任、§5.2.2包对/数据/恢复字段、V-02/07/12、AC-02/04/05、T-01/02/03/05/06及SD-01。P1/P2保待核/关闭合同；独立Web导出与App运行分开，ACE986/987不成整体前置。
- O-989-R1-01落§5.2.3/V-04/T-04，同最终组合配对及无法归因不得冒收益；O-989-R1-02与RQ去重建议落§5.1/V-08/T-01/02/06，三类证据处理及持久摘要最小要求，缺证据仅补相应重验，不要求永久留scope。
- r1复核及历史hash/needs_replan全文保留；本轮是修订/自检，不是独立设计重审，不将finding自行改CLOSED。未建树/scope、改代码、跑测试/GUI/安装/性能、改canonical、提交、打tag或联系其他会话。
- 修订静态自检：11章、9项未勾选任务（6 work/1 review/2 merge）、8 AC、12 V、1 SD；直接ID引用及表格列数/围栏检查通过。当前正文989/路线/战略的本地链接分别15/25/8条均可解析；跟踪文档diff空白检查通过，债表按原CRLF核验，989未跟踪文件另核无行尾空格。路线/战略同步v38，D-989-DESIGN-R1仍开放待定向重审；r1审查记录保留。语义合同SHA256=`9e9a60ecf17f93f2ae5384c79f996bdf15503968b95a7ed045a63dd05923883e`（UTF-8/LF，排除§9）；这些只证明文档结构与引用，不是V用例执行或独立复审通过。
- next：对r2作定向草案重审；通过后仍按用户暂不实施及§2正式依赖门执行。当前没有tag-ready结论。


### 2026-11-08 r3：RQ后继更新交接

- stage:new；plan_revision:3；outcome:blocked（仅规划，D-WS/D-CLOSURE及UI063/073/OS063正式结果未到）；保持drafting0/9。
- 062 r5已归档，前轮r4 waiting已为历史；新增真实depends_on auto-ui/PLAN-073与§2.2/V-13，增强AC-01/03/05、T-01/02/04/06及SD-01；任务9/AC8不变。073不阻塞UI063，OS063仅最终验收依赖073，989仍只验联合结果，不接管生产。
- r1/r2完整审查/修订记录及哈希保留；旧needs_replan不冒已独立关闭，本轮没有final review/运行/实现/新tag。next:对r3作定向设计复核，未来执行按§2硬门。
- 本轮规划静态自检：0～10章、T/AC/V/SD=9/8/13/1，所有任务未勾选、drafting0步；本轮四合同依赖图无环（解析80个真实Plan），本轮当前文档共82条本地链接及表格/围栏有效。语义合同SHA256=`7106d3d420db42001a37501878a4a3d15a2fb53d8227f8a3e9a913f59d3ec3f7`（UTF-8/LF，排除§9）。仅合同自检，非独立设计复审或运行PASS；本轮未实施。


### 2026-11-10 r4：正式Windows RQ后继与总收尾顺序

- stage:new；revision:4；outcome:blocked（UI077/OS066新草案未交，D-WS/D-CLOSURE仍未绑定/未就绪）；status=drafting0/9，任务9/AC8保留，新增V-14、§2.3与真实depends_on。原阶段核查记录不冒独立关闭。
- UI077只修投影/公开选路，OS066验全Demo/全产品/真实P-M与闭包，989核治理+RQ最终组合及失效/新交叉差额；有效证据可复用，避免重复验收和实施范围倒灌。OS062明确Linux发行支线，非Windows M1前置。
- next:定向设计复核r4；未来授权后可T-01准备，T-02及正式联验必须等上述全部门。未建树/运行/提交/tag或联系其他会话。
- 本轮规划静态自检：11章、T/AC/V/SD=9/8/14/1，drafting0步、任务全未勾选；新ID在主/归档/注册树唯一，正式依赖解析84份真实Plan、图无环；当前合同/路线本地链接、表格/围栏及限定diff空白检查通过。语义SHA256=`78ca5d8f8bd4a3ab999c45fcf7cc9127070bfd204ccf62cce90cbe5e8d435176`（UTF-8/LF，排除§9）。仅合同自检，非独立设计/final review或任何运行PASS。


### 2026-11-10 r5：1001治理与联合M1边界订正

- stage:new；plan_id:PLAN-989；plan_revision:5；outcome:blocked（规划订正已写；1001/D-WS决定与预算/OS066/D-CLOSURE未齐，独立设计复审未做）；status=drafting、0/9。保留r1～r4历史，不重开任何归档计划。
- F-R5-01：r4未绑定1001，且结构性workspace配对口径不能原样代缓存生命周期账。绑定1001，补A/B分账、恢复后净益/回长/耗时、零候选/负收益的两层准入；G2未擅改，V-15反假绿。1001如建议默认profile推广，另需正式生产交付，不夹带进989。
- F-R5-02：旧单路线tag示例及单一ready不能表达三路联合M1。§2.4/5.4、AC-06/08、T-06～09和V-16新增两类结论/收据/跨仓清单及协调owner；1000/AG026不是T-02生产前置，缺外部收据允许本路线独立交付，但禁止签联合ready。
- F-R5-03：正式v0.6门仅写ACE、漏AGE M2和Linux发行版全路线。同步§0/2.4/5.4/SD-01及版本路线决定；不把OS062或两个M2倒灌为Windows M1前置。
- F-R5-04：UI077已归档，活动路径失效。修正当前链接/状态/正式SHA；明确Todo queue与Video remote、旧queue视频能力仍拒绝、真实级重建与全产品归OS066，不将有限交付放大。
- 修改范围：G4/依赖表/§2.1～2.4/§4/§5.2.3～5.4/SD-01/V-15～16/AC-01/03/06/08/T-01～09；G2、C-01～05、完整RQ分母及原预算保持。下轮定向独立设计review核上述四项及旧finding，不将本次规划自检当独立关闭。
- next：暂不实施；可先独立设计复核r5。未来获执行授权后T-01只读准备可先；T-02等本路线正式门，联合标签另核三路收据。未创建树/scope、运行构建/产品测试/清理、提交、打标或联系其他聊天。

- r5规划自检：11章、T/AC/V/SD=9/8/16/1；drafting0/9、任务全未执行；当前范围66条本地链接有效，正式依赖闭包85份Plan唯一且无环，1000/AG026无倒灌T-02硬依赖；目标文档空白检查通过。语义SHA256=`82a88cedf70c3764134834c31f386743223a6640fab9f5b404cb6411e1de0406`（UTF-8/LF，拼接§0～8及§10，排除§9）。这是合同静态自检，不是独立设计复审或运行验收PASS；版本总路线旧历史段失效链接未纳本次修复。


### 2026-11-10 r6：两生产收口合同正式绑定

- stage:new；PLAN-989；revision:6；outcome:blocked（两新包/必需渠道尚未正式交付，独立设计复审待做）；drafting0/9。用户明确只规划，不执行。
- 新增正式依赖PLAN-1003与auto-os/PLAN-068。1001研究/采用决定已交，默认推广归1003；066正式归档但AC-02 partial及已具名余项不能抵完整RQ，全部由068 R-01..07关闭。修正活动1001/066路径与当前依赖/启动门，保旧收据历史。
- 1003独立，068 T-01～05可先、T-06及最终六格/性能等C正式交付；989 T-02等两包及D-CLOSURE，避免989顺手做生产或循环等待。保G2及全六格/P-M/物理/GPU/高刷原门，不用“具名后继”代真正完成。
- AC-01/03、V-14/15、T-01/02/04/06、SD-01及§2/5/10已同步；T/AC/V/SD=9/8/16/1。两类ready与正式v0.6边界保持，1000/AG026仅联合标签证据，不成为本路线生产启动依赖。
- next：新草案和r6定向设计复核；实施另待用户启动。无worktree/构建/产品测试/机器/清理/提交/tag或新agent/聊天。

规划静态自检：11章，T/AC/V/SD=9/8/16/1，全未执行；当前链接/表格/围栏/空白检查及正式依赖闭包87份无环通过，语义SHA256=`095c18161445384938324f54abd1589d0ceeb13658da0a96428c092427891b65`（UTF-8/LF，§0～8与§10，排除§9）。仅合同结构/来源自检，非独立设计复审或运行PASS。


### 2026-11-11 r7：正式交付、修约与渠道准入订正

- stage:new；PLAN-989；revision:7；outcome:blocked（规划完成，P2生产交付/E-03、有限包对/E-04及候选适用性尚未闭；无实施授权）；status=drafting0/9。
- 用户明确授权本轮接收1003与OS068正式交付/修约、核P2/C-01～05和T-02并同步路线，暂不实施。保G2净收益、渠道独立消费、未修约支持义务；r6旧严格RQ条件由本r7 B矩阵按已明确裁定精确替代，不静默扩展豁免。
- P1与R7已关闭来源已核，1003同实施上下文复审限制保留；OS068修约后独立终审/正式交付已核。五关键delivery读回且是各自主线祖先；本文只读核来源，不重测或改签上游。
- C-02/P2仍OPEN：Web SDK安装/服务生成/017-chat links修复不是生成成员独立导出；静态源码仍workspace继承/开发path，未找到正式关闭交付合同。领域owner=auto-web、中立支撑=auto-generation-core、Man仅薄编排；不默认移交全部ACE983或造新Plan号。
- C-01/03/04有上游证据、当前候选验证待做；C-05有限对象/包对/配方未冻结，给O-UI/SDK/RQ/WEB候选与责任。E门明确区分生产/定义前置与T-03～05运行结果，避免准入循环；P2仅有owner或编号不放行。
- B-01～08精确记60Hz/DWM、M-04机制、P-03配对语义、M-01协议、Video机制/后端移交与041/GPU证据限制；不把未获修约的要求自动豁免。稳定AC/T/SD保留，V-17/18新增，总T/AC/V/SD=9/8/18/1。
- next：定向独立设计复核r7；以后用户启动可先T-01准备，T-02须E-01～06齐，当前明确未满足。旧设计finding仅修订覆盖、待独立复核，不倒签CLOSED。未建树/scope、运行项目门/GUI/安装/清理、改canonical/归档、提交/tag或联系其他会话。

<details>
<summary>r6被替代的关键严格条件（保留原文，不作为r7当前门）</summary>

- G3：同组合满足完整RQ功能、全能力/产品义务及原direct对照性能预算，真实安装/导出/重定位/失败恢复成立。

| AC-03 | work | D-WS已批准生产方案按§5.2.3同最终组合配对，净收益与成本满足正式目标；实际生成的RQ消费链闭环；完整六格/能力/Demo/产品义务及P/M预算通过，V-03～06及V-11齐；缺格/超预算不以研究或其他版本证据抵扣 ；§2.2/V-13的RQ遗留闭环真实成立；§2.3/V-14顺序与完整RQ门成立；1001治理/研究与1003生产效果分账，V-15能检出空缓存/旧回收收益/建议冒生产；原净收益目标保持。；实际C配置及OS068全required/原性能门真实通过。 |

| [ ] | T-02 | work | T-01；D-BASE/D-STUDY/D-RQ正式交付；D-WS决策/生产结果及D-CLOSURE必需合同绑定修订并交付；1003与OS068正式delivery/SD及实际预算齐 | 冻结所有来源SHA/锁/工具/资源/模式与环境、完整分母及C表包对/升级恢复；核D-WS条件决定、同组合配对预算、准确驱动和摘要复用充分性，生成唯一候选身份与重验并集。V-01/11/12任一阻断不得继续；生产工件构建后补绑hash，不预填未生成工件 ；核§2.2 UI073及OS063正式承接、旧红/媒体/视觉/计时/制品与M1门；按§2.3核1003/OS068正式完整交付与治理汇合、复用/失效集；明确1001/066已交不等D-WS/D-RQ已闭；核V-15与获准生产配置，核1003 C实际配置/收益及OS068所有required关闭，无目标/结果不冻结。ACE/AGE M1收据不是本任务前置。 | AC-01；V-01/11/12 ；V-13；V-14；V-15/16 |

</details>

r7规划静态自检：11章、T/AC/V/SD=9/8/18/1，任务全未勾选、drafting0/9；当前范围85条本地链接有效、正式依赖闭包87份Plan唯一且无环，未把1000/AG026或两个M2/Linux设为本T-02硬依赖；表格/围栏/空白检查通过。关键canonical与所观察生产源码hash未变。语义SHA256=`e39ef57fe4bfadb97c808637d272fbcd30d30d596a003fa997ea584489e2bb58`（UTF-8/LF，§0～8及§10，排除§9）。只证明合同结构/来源自检，非独立设计复审、V用例执行或任何ready PASS。



### 2026-11-11 r8：P2正式生产合同与有限恢复对象

- stage:new；PLAN-989 r8；outcome:blocked（执行准入E-03仍缺1006实际生产交付，E-04尚待实际候选绑定，不是起草缺字段）。
- 用户明确“先立计划与恢复清单，暂不实施”。本轮中央分配1006 r1，auto-web主责；989不承接其实现，1003/OS068不重做。
- 修改依赖、D-CLOSURE/E-03/04、C-02/05与§5.2.2，采用恢复合同r1。旧正式源/对象/数据/支持配方/oracle有定义；N/包hash/实际命令待生产产出后绑定，不把定义/草案号当P2/C-05完成。
- RC-WEB-01明确首版同正式导出器A/B真实包对，非历史发行兼容；保实际移根/HTTP/状态/故障恢复承诺，未直接N/A。现有9T/8AC/18V/1SD职责保持，只同步其输入，不预跑T-03～05。
- next：1006后续获启动后实施并正式交付→989 T-01整合/核E门→T-02冻结→T-03～05实际验证；定向独立设计复核另行核r8。本轮仅起草自检，原finding/上游审查限制不倒签关闭。


规划静态自检（2026-11-11）：139条当前本地链接有效，8个恢复旧正式Git对象均可解析；1006/989各11章，T/AC/V/SD分别11/10/14/3与9/8/18/1，任务未勾选、drafting0进度；硬依赖闭包88份唯一且无环，路线跟踪23份（20归档、3草案）。6份关键canonical/生产源码hash未变，号源1007。语义SHA256=`c67de060904e32d742620ff559dd36c5521fdb0a6bf9c20f3ab3cc53fe564530`（UTF-8/LF，§0～8及§10，排除§9）。这是文档结构/来源自检，非V执行、独立设计复审或ready PASS。


6. **【已解除 2026-11-12】SDK 安装器暂存缺口（原阻断项）**：用户 AskUserQuestion 未即时答复，按总目标授权+仓库同形先例裁定 owner-lane L0 修复并已落地（man 54fe4070 / web 8b1742a，验证=RC-SDK-01 演练 14/14；详见 §9 对应 record）。原发现记录：PLAN-944（2026-11-06，7288db534）将 `ace-semantics-contract`（非可选 path 依赖，布局 `experimental/ace/crates/semantics-contract`）加入 `auto-lang-core` 闭包后：
   - `auto-man/scripts/man-install.py` `stage_sources` 报 "non-crate package input requires explicit mapping"（其 vendor/ 处理已有同形先例，experimental/ 无映射）；
   - `auto-web/scripts/web-install.py` `stage_source` 的 `crates/<name>` 布局校验同样不可用（1006 T-01 已记录该漂移，未修）。
   890 时代安装收据早于 944 故未暴露；068 三件套为直接构建不经暂存。**影响**：N 上 fresh Core/Full/Web SDK 安装（RC-SDK-01、C-04/C-05 验收面）不可执行。989 合同规定不擅改生产代码、缺口回 owner（Man=man-install 映射、Web=web-install 布局，二者修复均有既有同形先例，属小改）。**待用户裁定**：(a) 授权本会话按 owner-lane（L0 fix-* 工作树）立即修复两安装器后 989 继续；或 (b) 记录为 owner 缺口、989 以 AC-04 partial 等待正式修复计划。
2026-11-12，stage:goal，PLAN-989 r8，round:2，from:work，to:user，status:executing，outcome:waiting——T-05 RC-SDK-01 被 SDK 安装器暂存缺口阻断（§10.6），修复路由需用户裁定（989 合同禁改生产代码）。已完成并保持有效：T-01～T04 全绿（见 §9 各 record）+ T-05 RC-SDK 部分前置（core lock 再生成为 989 树内合法簿记修复；安装暂存缺陷为生产 owner 面）。next: user 裁定 (a)/(b)。

stage: goal | plan_id: PLAN-989 | plan_revision: r8 | round: 4 | from: work | to: work | status: executing | outcome: pass(T-01..T-04 完成+T-05 四对象中三对象完成/复用) | worktree: D:/autostack/.wt/lang-989 | commit: lang 075f65cc/man 54fe4070/web 8b1742a | next: work(RC-RQ-01 旧三件套构建中→包组演练→T-06 汇总→T-07 review→T-08/09 merge)

2026-11-12，stage:review，review_scope:final（三轮：needs_fix→needs_fix(narrow)→**pass**），reviewer:ZCode 子代理（独立上下文，无实现记忆；覆盖边界=未重跑构建/演练，断言级核验+收据交叉+reflog/depfile/git 六仓独立取证），PLAN-989 r8，outcome:pass。
- 审核基线：实施树 codex/plan-989-dev（SD-01 修复提交 65b7abceb/3acfb51df）；冻结候选绑定=lang 075f65cc(+SD 提交)/man 54fe4070/web 8b1742a/ui 69104765/os 868c80d3/down 58ea33f——任何漂移使本 pass 失效。
- AC：AC-01..06 验证通过（AC-04 携 RC-UI 具名限制；AC-05 身份声明全部对齐工件）；AC-07 由本审查链满足；AC-08 归 merge。
- 两类 ready：split_refactor_ready=true / joint_m1_ready=true 于修正后依据成立；本 pass 不授权任何 tag/部署/发布。
- **merge 绑定条件**：T-08/09 摘要必须原文呈现 RC-UI 业务 oracle 限制（业务值断言未执行；已执行面如清单）及其廉价闭合路径（单次 counter 交互断言）。
- 覆盖边界：1006 原 30/30 收据已按删除政策折入归档记录（3f72415b）；空间数字未重测；man/web 修复代码只审 diff-stat 级。

2026-11-12，stage:merge，PLAN-989 r8，outcome:pass（T-08/T-09）。
- prepared：终审 pass 绑冻结候选（rebase 前链 075f65cc+65b7abceb+3acfb51df）；master 期间被并行 PLAN-1005 归档推进（d0dc40031→e6fb5855，纯计划簿记）→ 实施树 rebase 4/4 干净（旧→新：4 提交等价重放，旧 3acfb51df→新 28803252e），身份零漂移。
- landed：auto-lang master=28803252ec1d0890f88624ff35b3a0c71e2b22df（ff-only，无 merge commit）；canonical=docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md 落地核验。man 54fe4070/web 8b1742a 已先行 owner-lane 落地。
- merge 门：冻结固定组于 075f65cc 跑绿（run-c6ff3ct_）；其后仅 docs 提交（SD-01 三笔+rebase 纯簿记）→ 按同输入复用规则不重跑。
- **终审绑定条件履行**：RC-UI 业务 oracle 限制原文呈现——业务值断言（Counter 从初态+1=1）未在任何面执行；已执行=双侧生成/verify SHA/locked 构建/移根重建/受管升级/坏源故障逐字节保全/旧包真窗口存活；廉价闭合路径=在任一交付包上补单次 counter 交互断言。该限制不承重任何 ready 判定。
- 两类 ready 最终：split_refactor_ready=**true**、joint_m1_ready=**true**（v0.6-M1 tag 模板已备、**未创建**；正式 v0.6 另守 ACE/AGE M2+Linux+App 强化门）。
- T-09 清理：lang-989 组（lang/down/ui/os/man/web）+rc989-old 组（六仓）guard 后全移除；分支 codex/plan-989-dev 删于已合并 tip；plan-artifacts/plan-989 整 scope 删除（全部收据/摘要/哈希已折入本计划 §9）；登记表刷新无具名持有。批量回归：989%5≠0 但 >48h 有合并=到期（PLAN-1000 起即欠）→ 交接 auto-plan:regress 主检出单实例执行，不在本 merge 内跑。
- 落地观察项：语言 release 产物/部署面二进制相对 N 陈旧（本计划交付=规范/验证/修复，无运行中进程依赖重启）；用户安装/缓存/持有未动。

## 10. 待澄清事项

1. **P2仍是生产阻断**：1006 r1已由Web主责立约，当前drafting0/11；正式生产入口/闭包/真实HTTP与状态、独立审查/落地完成后才满足E-03。无需重做1003/068，不在989实施。
2. **恢复对象已核定、实际绑定未齐**：统一合同r1定义RC-UI/SDK/RQ/WEB及旧正式来源/N规则、成功/失败/恢复oracle；1006交新的实际入口/源，989 T-01/02核各真实命令/资源/protocol/schema。包hash只在产出后登记；T-05真实恢复仍未执行。
3. **OS068适用性/缺证**：按B-01～08接收明示修约，不新增豁免；041等未修约required若证据不足，T-01提出定向补证/生产owner。Video后端按已批移交，不冒已交。
4. **联合收据/独立性**：1000/AG026归档不自动代表共同组合/领域签章足够；T-06核正式证据及后续有效审计，保历史同上下文审查局限。独立设计复核r8仍待，不倒签旧finding。
5. **授权与状态**：仅计划/恢复清单及同步，989 drafting0/9，所有T/V未执行；1006也暂不实施。实际打tag/推送/部署未授权。
