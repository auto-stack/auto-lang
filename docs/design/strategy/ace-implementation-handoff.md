# ACE实现路线：决策与续接基线

> **2026-11-03 M2盘点交付（PLAN-922）**：922 已完成 work/review，交付[M2全场景替换盘点报告](../../plans/reports/ace-m2-replacement-inventory-2026-11-02.md)（分母 A-G 68行、能力/接线分账、REP-01..11+V06-ACE-01..05 门映射、后继 20 工作标签、退役 8 步序、关键路径/容量估量）与[盘点口径 Spec](../../specs/ace-migration/replacement-scope-and-evidence.md)（SD-01/02），本文件短窗同步。要点：AV 语义执行能力基本 IV 但**产品入口零接线**（唯一 ACE 接线=auto-case ac 适配器）；最大缺口=宿主 API 对等（A-10/C-07/E-02）、REPL/daemon/#[test]/config 求值、非 Rust/C 的 A2X provider（TS 为 UI 生成器生产依赖最优先）；"晚一周"发布无数据支持（估量乐观 6-10 周/保守 3-5 月，需扩 owner 并行），排期决策交用户。后继标签 M2-DEN/EXEC/HOST/A2X-R/T1/T2/UI/MAN/WEB/GEN/OS/REL/TEST-MIG/RETIRE 未取号；native 三格已由 923 NF/924 NC 认领（本报告 §2.3 已刷新引用）；§2 的 M2 定义与登记计数已按 r2/131 行校正。下方各块按日期新旧自上而下，较早快照按历史读取。

> **2026-11-03最新native交接**：main读取9c91a1b48；915/919实际交付5ba5d4cd2/4ea3daddf，旧required仅native R02/R03/R05三格。新立[923 NF](../../plans/923-native-allocation-fault-and-place-conformance.md)0/12与[924 NC](../../plans/924-native-closures-source-and-resource-closure.md)0/14，均drafting r1仅规划。923补native全非闭包fault/Copy元素受检Place，独立关闭R03并交版本化R02部分；924才实现原K02有限native闭包/env/真间接调用/clone/最后释放，补闭包fault且在最终库下重验923，独占回填R02/R05。不能只重跑42证明失败原子性，不能事后trap冒访问前拒绝；函数P/IO0、来源恢复后下一body编辑/修复再恢复和原资源时间门写入两包。N为923→924稳定符号窗，921主A2X可并行，公共source/凭证/lowering/父adapter/SDK/fixed/lock/登记取S/H实际SHA与全部consumer。922已有lang-922树，头drafting不当空闲，不改其进度/合同/报告。无D921/D922整包前置，不启动别会话/施工；设计/逐任务入口见[Design43 §21](../43-ac-av-typed-mir-and-abc.md#21-native-r02r03r05故障与place基础有限闭包分包2026-11-03target-state)，较早快照按历史读取。

> **本轮收口状态校正**：main随后推进c3304ed73，915已execution_done6/10待独立review，尚无D915；917仍实施。下条5a0ef7609保取材快照，919 AV依赖接口仍按实际冻结SHA/写窗，不把待review当已交付。919–922未实施。

> **2026-11-02最新下一波交接**：main5a0ef7609；D913=04f487b4b、D914=6ffbaf49e、D916=edfdabc2e均已合入；915/917实际实施，保其WIP/进度不代改。正式新立[919资源事件](../../plans/919-ace-resource-lifecycle-event-conformance.md)0/10、[920 B18旧VM诊断](../../plans/920-legacy-avm-immutable-diagnostic-closure.md)0/8、[921 CA-A](../../plans/921-shared-a2x-c-scalar-target.md)0/14、[922 M2盘点](../../plans/922-ace-m2-replacement-inventory-and-critical-path.md)0/9，均drafting r1仅规划。919 AV动作观察需与915逐符号稳定SHA交接，native/私有case可先；920旧codegen与917旧Rust主体独立；921主A2X，922只读跨仓/写自己报告不抢实现面；H全文件短窗/所有consumer/漂移STALE与正式采样错峰。旧M1六资源格/B18单格认领互斥，native三格仍未立项，不从native集合推定闭包支持；C B..E未取号，M2全场景/全目标/工程增量/真实安装与disable仍发布硬门。执行入口/AC/V/SD/树与材料政策见各计划，不以本次规划当实施授权或签章。

> **2026-11-02最新发布交接：v0.6必须交付M1（含A2C）与M2全替换/旧实现disable。** [发布门V06-ACE-01..05](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)绑定REP-01..11、全消费者/目标接管、真实工程增量/在线更新及安装发行。用户接受为此顺延进度（如一周），未确认七天足够；先核关键路径、跨仓窗口和验证容量，不到期自动减范围/留旧引擎兜底。AACE仍v0.7，未来M3/M4新增扩展不排。本轮只同步规划，实际迁移/disable须后续正式Plan与收据。

> **2026-11-02最新M2交接：补全ACE能力，并替换旧引擎AutoVM/A2X。** [M2合同r2/REP-01..11](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)把旧M4的迁移/部署/旧实现disable全部并入M2，M3/M4未来新增扩展暂不规划。必须覆盖AutoUI的现有模式/交互、AutoMan的实际配置求值/任务/生成与全部其他消费者，以及旧A2X所有现有目标/公开API；只新AV或A2R/A2C绿不能签全M2。入口可分批切，但M2结束时全部切到ACE，default/all-features/工具/安装发行/日常测试不能启用旧executor或AST emitter，无hidden开关/旧binary/自动fallback；薄compat只转ACE。共享Rust库/值类型/诊断/渲染器先归属，不盲删。旧M1登记历史冻结，新M2 successor ledger/recipe/证书迁移旧义务，不为测试永久编译旧引擎。稳定宿主Auto编辑Rust构建0、跨模块真实未改P/IO0、同PID热更新/旧链保活与原资源/周转门保持；发布/显式认证仍用各目标工具。M1 A2C与在途合同/原批准及AACE v0.7不变；本轮只更新规划，不实际disable/删代码/取号。

> **2026-11-02最新目标交接：A2C已批准加入M1。** [独立范围增补](../../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)保留旧批准件SHA/313 required、NA/accepted及旧A2R双轴；[Design51 r5 §13](../51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)、[C路线](shared-a2x-and-hir-to-c-roadmap.md)、[Design41 §10](../41-interface-metadata-and-foreign-abi.md#10-a2c第二目标的有限abi桥2026-11-02target-state)为新合同。预计CA-A共享目标接缝+C标量、CA-B record/text资源、CA-C集合/变体、CA-D有限ABI桥、CA-E新ledger/Q4/总签章五包，仅标签不预占Plan号；不是只交标量演示，也不要求完整通用FFI。现a2x-store仍依赖Rust TargetProfile/lower_functions，首包必须带C消费者做去耦；797是声明IR下有限候选三腿，不冒Auto源码FFI。797/892旧delivery非当前祖先，当前代码/Spec内容与集成新验收为基面；902/904实际delivery已在main。Q4分别对独立oracle，旧M1-base与新增C核心/资源/增量/恢复/ABI/Q4/登记门AND，新增门未实现则M1不可签，不能只读旧can_claim_m1。913主manifest/root/lease/恢复，914主场景/布局，915 AV资源，916/917旧A2R；设计读取main44a774998、913已3/12、914在途、916实际有树，公共HIR/资源/ManagedStore/A2X worker/adapter/SDK/固定组/登记全文件单写，正式采样错峰。私有C准备可并行、不新增整波前置、不消费WIP；本轮只改设计/路线/新批准增补，不改913–917合同/进度/current Spec/旧source/expected/binding，不取新号/建树/施工。下方旧状态与“剩余计划数量”仅为旧M1快照。

更新时间：2026-11-03（PLAN-922 交付短窗调和本文件：并入此前未提交的各会话交接块，未丢弃任何内容）。用于长会话结束后的上下文恢复；[ACE总路线](auto-native-backend-evolution.md)仍为统一入口，current Spec为现状知识，具体Plan和revision绑定的review/merge收据决定交付。当前M1缺口owner/立项状态、M2已推进能力及865/863后的收口队列见[收口路线对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)。当前入口为§1的908 r2批准/904已交付续接；其余旧状态按历史快照，接手时须重新核对活动目录与`docs/plans/archive/`，不据本文替代验收或重开已归档计划。

**当前更新（2026-10-30）**：865已交付5fa3133fa、863已交付2d5f3d842并完成清理。native第二阶段拆为[889集合/变体源码闭环](../../plans/889-native-collections-variants-source-closure.md)（executing r2）和[891宽标量typed入口](../../plans/891-native-wide-scalar-typed-entry.md)（draft r2/0-of-10，已做规划复核）。891用独立内部bit-carrier ABI/精确结果通道关闭W01–05，不含宽算术/源码/FFI或wide拥有集合组合。默认N按889→891单写，稳定已提交阶段可交具体符号；H与正在实施的888及889分窗，采样错峰。891本轮仅规划，不修改889/888实施合同。

## 1. 本会话收敛的架构决定

**2026-11-02下一阶段交接（优先于下方历史）**：D911ca4848f5d/10-of-10、D91213a465336/12-of-12已归档交付；913已开工T-01，actual tree lang-913，头drafting滞后不回改。main0e93e3cc9，34阻塞为静态读账，不能替正式父报告。新四包均drafting r1、0/10：[914 E01/E02四路径+I04/I09 AC/AV观察12格](../../plans/914-m1-evaluation-order-and-layout-observations.md)、[915 AV R02/R03故障/Place2格](../../plans/915-av-resource-fault-and-stale-place-conformance.md)、[916旧A2R解析/六诊断8格](../../plans/916-legacy-a2r-semantic-admission.md)、[917旧A2R R07/T08发射证据2格](../../plans/917-legacy-a2r-mut-record-and-text-evidence.md)。[Design43 §17](../43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)明确生产/观察/父准入/预算/阶段，三个职责域独立；916→917只是公共rust.rs准入接缝默认交接，不要求D916整包。913只占来源恢复/manifest/root/lease/accepted-context，915的av-core与914通用观察交集逐符号取S窗，公共adapter/合同/固定组/M1登记全文件单写、正式接口SHA/版本/所有consumer与漂移STALE收据必需。916/914私有部分可先与913并行；915按实际S释放推进；采样/显式认证错峰，不持写窗等机器。原日常a2r零编译，负例前端拒绝不造金样；新场景证书/复合证据隔离旧标量schema。四包认领24格，后续仍有R01/R04/R06 AC/AV六格、R02/R03/R05 native三格、B18 avm一格、独立M1签章与具名旧红；native R05与现NA白名单/能力边界先核不能自行免除或暗造闭包ABI。当前纯规划，无源码/expected/binding/current Spec或913合同/进度变更，无新实施树/材料/agent/chat。

**2026-11-02最新规划交接**：[912](../../plans/912-closures-function-production-closure.md)→[913](../../plans/913-closures-source-bound-recovery.md)，均drafting r1、0/12。main5a4eb4e25、D90959b8107eb祖先0/12-of-12归档，代码确认R1/R2/R4仍在source frontend，verify_function_ref目前排除Closures，R3的unit key依赖先encode flat；因此拆单函数生产与来源恢复两份可独立关闭合同。[Design43 §16](../43-ac-av-typed-mir-and-abc.md#16-909后继闭包源码生产粒度与恢复入口2026-11-02target-state)冻结目标。912主scoped源/CST/身份、受检闭包单函数context/资源事实与lowering/安装；913接受D912正式accepted-state/API/身份/预算后做parent anchor/manifest/required/恢复深验与下一改，不保存authority/live env/PASS，不从未知索引自授权。911已实施，仅其S1/2失败/attempt与公共H按稳定SHA/具体符号单写；N/parent可并行，未释放只等实际依赖任务。恢复重验R、源生产P、元数据M分账，暖未改P0、纯恢复全P0且R真实非零，不能改名洗零。64顶层/每owner3lambda、多步/四状态30/512与原资源/独立oracle同验。源码/expected/binding及911/910合同未动，本轮不施工/不建树；D909-1及其余能力缺口不转移给两包或签M1。

**2026-11-01最新续接**：[911 native源码诊断与M1 B17–B19](../../plans/911-native-source-diagnostics-and-m1-b17-b19.md)仅规划drafting r2、0/10；[Design43 §15](../43-ac-av-typed-mir-and-abc.md#15-native源码诊断事实通道与m1-b17b192026-11-01target-state)为目标合同。当前main367258e7a，905/906均已交付，908 archived12/12（dc8eca3ae祖先0），909用户确认执行中（实际lang-909树出现、header滞后不代改）。908已实证B17–19 native generic原因不足，因此下一包先做typed cause→worker→strict parent→原source/近邻/错因反控→三格父证据回填，原源/行为expected/NA/分母/其他路径不改，不接旧a2r或AV资源平台。warm/fallback需保失败candidate/source/step/span事实，失败不commit、不运行旧PE冒成功；必要AV只读诊断出口与909短窗，不改其捕获语义/生命周期，N/parent可先独立。公共contract/manifest/固定组与909/910按实际符号交接，无整包互等。取号时910已归他方Cargo治理，911唯一分配、下一号912，不预占后继。本轮只文档，所有实现/计量/验收待执行；908冻结core247/285、38阻塞不能因两个包在途称M1已过。 911 r2规划复核补齐操作级快照/真实终止profile与context、原始JSON互斥结果、typed拒绝与首失败停、失败全出口attempt、正式父事实重读及W-S1/2/H/M1稳定接口；明确标量worker与独立owned暖API，V扩至20，任务仍0/10，仅规划。

**2026-11-01新增规划续接**：[909 C-19／K06源码闭环](../../plans/909-shared-mutable-captures-source-closure.md)drafting r1、0/12，[Design43 §14](../43-ac-av-typed-mir-and-abc.md#14-c-19k06共享可变捕获源码切片2026-11-01target-state)冻结拟实施边界。K06是move私有可变环境原源42，不单独证明真实共享；新增非move局部var int/bool/char共享写，外部/兄弟写回、拥有cell逃逸、clone独立/内部共享与失败清理均验，原owned/Borrow保兼容；不扩共享owned/cyclic/跨挂起/线程/native/A2R。当前closures_source::apply_source整源编译且全函数lower/encode后比wire，必须同包补真函数生产0/对象/依赖/恢复门。865 archived9/9、823 archived11/11；旧865历史delivery祖先核验1，现行source Spec沉淀fe509bfff祖先0，以当前实现核，不据旧SHA冒新收据。908真实实施树WIP已确认，header滞后不代改；909主S/P并行，908主规则/manifest/852/场景，K06 AV check/token与公共注册H短窗，正式计量错峰，不设D908整包前置。本轮仅立项与设计，未施工/建实施树/改current Spec或原源/expected/binding；next后续实施909 T-01/02。

**2026-11-01当前续接**：[908 M1规则/登记/验收归一](../../plans/908-ace-m1-rules-registry-and-acceptance.md)drafting r2、0/12，G-DEC-1..5均按推荐正式批准，已有NA/R/双轴/I07/I08直接继承，不重复投票。批准原话与61唯一格、原103核心/扩28已承诺AV、O非AV45 future scope见[决策件§13](../../plans/reports/ace-m1-scope-decision-2026-10-28.md#13-补充裁定g-dec-15plan-9082026-11-01)；45仍gap/owner，131/524全登记保留。90310/10、89312/12、904 r2 14/14已归档；904 delivery12f9adf18在main，type/enum片段/完整source及execution_config证书、CollectionsBodyFacts资源/清理正式可消费，旧904树/窗已清，无D904等待，新producer不替旧a2r。当前lang-906真实WIP含test-groups/auto-case/features/Cargo，908私有规则/来源/852核算可先行，公共注册或观察桥按当前具体符号短窗；905研究execution_done不是已实现生产治理。顺序按908 §5.8：T-01基面批准→最小链/来源与窄诊断→迁移候选→入口/注册→消费者/独立复审→沉淀清理，再按实际required缺口安排能力包，全部所需门闭后才签M1。893成本/903审计仅重核904所影响输入，不复活raw/重跑13慢例或改冻结数。本轮仅文档，binding/源/expected/golden/cert/当前Spec未改，不自动实施。

2026-11-01复核交接：904已修订r2、drafting0/14，原20源/AC/数值门保留，V扩至17。新增关闭点为完整source/range连续编辑与env同代、同owner/context异body凭证拒、真实payload/producer、操作计费/取消/无余量清理、证书配置/mode隔离及每修订认证。893/903写面按904 §5.8精确路径和正式来源取窗；903树内审计Spec候选不被本包当已发布规范，893成本/903审计受新来源影响时仅具名重核受影响项，不互等整包。本轮只规划不施工，不联系他方、不取新号。

2026-11-01 AX-C交接：[904](../../plans/904-hir-to-rust-collections-variants.md)新立drafting r1、0/14，前包902已交付14/14（4303c4457050）。type/enum片段来源修复纳同包S前置T-02/03，含902 record/text-record自由函数兼容，不再新拆小包；不靠whole后片段命中或全span0冒前端0。Collections当前无TextResourcePlan是frame-owned合同，904须消费core受检拥有/guard/清理事实而非后端另造ownership。原14 L/V+6源、独立reference、资源失败/深链/候选发布、四状态30/P90≤2000ms及新strict recipe/fileset/cert按904。893慢测/接入、903 M1审计与904 P独立并行，S/H逐符号、正式采样错峰，无D893/D903整包前置；不动它们合同/原源/expected/binding/NA，本轮无施工。

2026-11-01续接：891已归档交付10/10，901已归档9/9，899树已接受U901；902由用户确认开工且lang-902存在，不以其未同步header反推未实施。本轮仅规划：[893 r3](../../plans/893-ace-test-framework-and-slow-case-attribution.md)0/12将13项归因/未来接入适配固定Cargo组、原生ID、现有Job/meter结果，禁止恢复tm/动态影响图/旧raw前置；两个828旧t07名静态未找到同名，T-01需证迁移/断言守恒，不削13项。[903 r1](../../plans/903-ace-m1-net-gap-audit.md)0/10核M1净缺口/批准范围/双owner/后继，不改生产、判定、binding或既有规则。当前131行524登记格与原M1适用分母分账，a2r金样/认证分轴；审计PASS不等M1签章。899工具/902受检资源与Rust/P/H按符号交接，893慢采样、902认证/性能及903必要窄重验错峰。AX-C在D902后立项，数值源码/多模块仅候选，未预占号。

2026-10-31 AX-B立项更新：[902 HIR→Rust record/text与资源](../../plans/archive/902-hir-to-rust-record-text-resources.md) drafting r1、0/14；892共享骨架、881方法和894文本资源已正式交付并清理，当前可达清理提交分别d6ef24227/c41c57890/02ab1a7cc均核为main祖先。902主写现有experimental/ace四A2X crate及最小S受检资源/模式事实出口，原R07/R08=42、T03=23/T05=42真源码认证/函数增量为required。899工具/profile与901语言feature/SDK/固定组按具体符号交接，无整包前置；日常emit/golden与新严格认证分轴，不改旧a2r/M1分母。当前合同与入口见[Design51 §5.3](../51-shared-a2x-and-hir-to-rust.md)及[A2X战略r2](shared-a2x-and-hir-to-rust-roadmap.md)；AX-C未编号，本轮只规划，不施工。下方892/894旧drafting或在途描述均按历史读取。

2026-10-30最新收据补充：888已归档10/10，delivery `5a6c23afa`、清理`129ef6057`均在main，原公共写面/测量窗已释放；894共享文本包已立draft r1、0/13（见§4.2）。下方旧888 executing/reviewed状态仅作历史，不再作为等待依据；893的13项稳定证据路径见[交接报告§7](../../plans/reports/ace-slow-tests-followup-2026-10-30.md)。未自动启动其他计划。

| 主题 | 已确认方向及边界 | 正式合同 |
|---|---|---|
| 名称与实现主体 | ACE是统一语言引擎；AC编译、AV解释、AJ按需编译/优化编译，A2X负责源码转译。当前Rust实现；Auto主体AACE及AAC/AAV子线延后v0.7，先满足Rust版全示例成熟度 | [Design43](../43-ac-av-typed-mir-and-abc.md)、[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md) |
| 共享执行语义 | source/CST/AST→解析/推导/实例化与CTE查询→Checked HIR；共享Typed MIR分别供ABC→AV及Cranelift→native。最小Typed MIR属于v0.6，已不采用“整个MIR等v0.7”或“新VM长期直接HIR→ABC”的旧方向 | Design43 §2–5、[Design49](../49-native-single-mir-lowering.md) |
| 转译与优化 | A2X主要消费结构化Typed/Checked HIR，不强迫从低层CFG恢复源语言结构。必要lowering/verifier/清理属于正确性；大规模优化、SSA/Machine IR、自主后端按收益后续实施。LLVM约80%/少量高收益pass是待基准冻结的研究目标，5%不是精确pass数量承诺；先实测现有Cranelift，不重复实现无收益优化 | [后端战略§5](auto-native-backend-evolution.md#5-v07按收益选择优化按需引入低层ir) |
| 共享A2X与HIR→Rust（2026-10-30设计） | 共享语义view/结构化prepare/依赖与产物服务，目标表示/合法化/runtime/AST独立；首期拟共享骨架＋Rust标量→record/text资源→集合/变体三包，AX-A已立892草案、AX-B/C未编号，均未实施；第二目标与产品替换另验 | [Design51](../51-shared-a2x-and-hir-to-rust.md)、[A2X战略](shared-a2x-and-hir-to-rust-roadmap.md) |
| CTE、泛型与meta | 语义查询调度经受检片段→MIR→ABC→AV→portable冻结结果→推导/HIR再验证。不可变语义事实、一次推导工作态、选择性runtime反射投影分开；不让Codegen/可变Universe拥有权威类型事实。完整泛型、类型/声明生成、双期反射仍需逐项扩面 | [Design42](../42-semantic-metadata-inference-and-comptime.md) |
| 语法平台 | Atom/Batom+Schema配置语言包，编译解析表/专用scanner或代码；不强制热路径解释动态Atom树。通用无损增量CST、语言AST投影、Checked AutoHIR分层。旧实现提供语法案例和可证明复用的算法，不继承旧AST/ABC/类型布局兼容假设 | [Design39](../39-ac-incremental-syntax-and-language-platform.md) |
| 内存与全局cache | 热点有界常驻、冷数据分块持久化/按需装入，不能默认加载整个仓库/依赖AST。各层身份/依赖/Schema/codec及预算统一；typed view、紧凑struct/enum、Atom调试格式按用途选择，不把所有内存结构或原生对象文件强制变成通用Node | [Design40](../40-compiler-artifacts-and-cache-lifecycle.md) |
| ABI/FFI | 语义类型、AutoABI、目标foreign ABI、wire与执行布局分域。有限AutoABI尽早冻结验证，C双向边界先行，Rust适配优先明确C-shim边界；Python/完整混合生态另有支持矩阵，不假设Rust私有ABI长期稳定 | [Design41](../41-interface-metadata-and-foreign-abi.md) |
| 分层执行与增量发布 | AV开发→热点AJ/优化AJ是目标；部署固定完整快照/profile，校验及复用可重定位产物，补编未执行冷代码/不兼容单元，再链接/打包。不是转储JIT地址，也不因源码相同就复用dev机器码；无须每次全量冷重编，可信冷构建仍可作验证 | Design43 §9–10、Design40 §3.1 |

CTE缓存命中要求**完整实际依赖**不变：除调用点源码外还包括helper body、类型/候选集合、target/layout、配置、VM/profile与受控外部输入。结果属于A05，类型/签名/推导/实例事实属于A06；求解器可变状态、活VM堆和信任凭证不持久化。未知副作用不能共享结果；取消/超限不缓存为语言非法。普通runtime程序仍实际运行，编译cache不变成任意程序结果cache。

全局存储沿已有store/CAS、索引和预算合同演进；Batom读优化或其他冻结块须实测并验证边界后才能view/mmap，不承诺任意Batom零拷贝执行。required-data强引用、失效依赖与provenance分开；自动清理覆盖数据及索引、WAL/staging/orphan等总占用，保护用户交付物和活lease。分层数据库/pack方案是待验证机制，不能以“缓存已设计”宣称整个健康治理产品完成。

## 2. 成熟度与迁移边界

M1为有限共同子集/核心链路与可审计对等基线（含新增C组合门）。**M2 定义已被 2026-11-02 合同 r2 取代：补全 ACE 能力并替换旧引擎（AutoVM/A2X），覆盖 AutoUI/AutoMan 等全部现有应用场景，旧实现彻底 disable/退役；原 M4 的迁移/部署/回退/退役职责完整并入 M2，不再有"等 M4 才 disable"的后门**；M3/M4 如续用编号范围另行确定，未来新增扩展当前不规划、不作为 M2/v0.6 前置。v0.6 发布谓词=既有版本门 AND V06-ACE-01..05（M1+M2 全闭）；AACE 仍 v0.7，完整AJ不是旧引擎替换的自动前置。

“骨架成型”“四路径M1收口”“全语言”“产品默认切换/退役”“自举”分别验收。当前是M1共同子集收口与M2源码扩面交叉阶段，**863完成不自动签M1或M2**。M1适用规则沿[2026-10-28决策](../../plans/reports/ace-m1-scope-decision-2026-10-28.md)及862现行Spec的NA-4；accepted-diff保留旧差异证据，a2r金样/独立认证分轴。原始登记现为131数据行（corpus+scenario+console O 族；920 后 B18 avm 格已闭），仍有能力/接线缺口混记；50/103、20/73均为历史快照，不当当前clean数。先按[逐组对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)核实际证据/owner，完整复算后才签M1。native闭包/Dyn/loader的已批准NA不变，a2r名解析真缺口不豁免。

新引擎实验代码进入仓库、接入可选adapter、产品切默认是不同动作。旧引擎可继续承担生产；不要求把每项ACE新增能力再复制到旧引擎。共用语言/stdlib/UI接口与数据合同，后端适配按真实消费者另验，分支/仓库归并按实际owner与迁移Plan办理，不在此新冻结永久分支策略。G7按入口/SKU切默认，G8在所有保留消费者、旧会话/回调/lease清空后才退役；不得遇unsupported就静默回退旧引擎。旧A2X源码转译职责不能以AV替换为理由取消。

AutoUI/AutoGraphics与ACE按受检语言/宿主接口集成；初版旧VM适配不证明ACE能运行UI，也不自动要求新图形系统重做一套a2r实现。具体仓库、迁移和产品接线随其owner/Plan，不能把ACE实验crate独立推成与拆分、SDK和公共测试无冲突。

## 3. 当前已交付基础与仍有限的能力

| 基础 | 当前可认领范围 | 继续工作时的限制 |
|---|---|---|
| 783/787/788 | 统一产物/store探针、有限Atom语法驱动增量词法/CST、文件化原生测试及计量基础 | 非完整Auto通用生成器，不等于所有源码已改用787前端；分层cache产品仍按实际producer核实 |
| 796/797 | 有限语义事实/推导查询；Windows x64 AutoABI v0.1候选及Auto/C双向互调 | 非全语言反射/泛型；非全类型/平台FFI、Rust/Python产品接线 |
| 798/828 | 共享Typed MIR/ABC/Rust AV、函数级全链增量及prepare/commit/snapshot/pin/lease | 不是持续运行程序中的完整在线热替换；该缺口由866负责 |
| 817/818 | AV驱动标量CTE查询/冻结cache；标量MIR→CLIF→对象→原生对拍/单元cache | 主要i32/bool，不随AV扩类型自动变成复合CTE/native |
| 820/821 | data/owned具名profile、record/text/Place/move/borrow/清理的有限合同 | 不自动包含所有旧方法、完整语言、运行期句柄防护或native ABI |
| 822/823/824 | 集合/变体、拥有闭包、显式Dyn/portable冻结的typed执行合同 | 集合/闭包/Dyn typed fixture不等于源码语法/推导/lowering已接通；不自动扩CTE/native |
| 825/826/827/852 | 标量LoadedProgram优化、紧凑ABC、真实scalar源码基线、四路径文件化语料与a2r独立认证 | 不把执行/编码优化计作语言覆盖；852是分母/判定底座，不是所有能力已绿 |
| **868 r1** | **已归档，merge收据delivery `7754c4e4c`；共享lowering实际位于`experimental/auto-mir-lower`** | 原probe路径已移除，三消费者只读消费；不重做搬移，不扩868原合同 |
| **862/864** | 已归档：AV入口/Unit/尾值/脚本，集合/变体/is/有限List for-in原源；864 delivery `6273cd4a4` | 入口是AV-only，集合不自动扩native；完整Unit/迭代器/用户泛型另验 |
| **866** | 已归档，delivery `3b4080916`：真实AV常驻、安全点更新、在途守卫/延迟安装和有界回收 | 不再把在线AV缺口记为未交付；非native loader/任意活对象迁移 |
| **870/881** | 已归档，delivery分别`bc90ef718`/`db0773130`：mn data/owned、static/view方法与真方法函数增量；R08/R07 ac/av已闭 | 881不补文本通用eq/ne等非方法gap；typed宽数值/集合native/完整方法体系仍有缺口；871退役另验 |

上述为各有限合同的交付/现状归纳，不把“归档”单独作为全部收尾已完成的证明。824归档文件仍有T-08未勾选、最近merge条目记录已landed但清理/批回归待办；825亦有计数与delivered收据不齐的簿记现象。若新任务依赖其最终收尾，先核实际Spec/合入及遗漏收据，不篡改历史、不自动重开或重做旧实现。

## 4. 当前主任务、剩余缺口与开工边界（2026-10-29）

| Plan | 读取时状态 | 工作及下一步 |
|---|---|---|
| [881](../../plans/archive/881-shared-record-method-lowering.md) | archived r2，9/9 | 方法producer已交付db0773130；方法/模式/资源/增量证据交871接受，不能再写“待立项”或重做搬移 |
| [871](../../plans/archive/871-ac-native-switch-and-old-lowering-retirement.md) | archived r4，9/9 | delivery 87dd2b6ad、清理5af8e0cbe；共享MIR唯一native源，原五profile迁移/旧HIR→CLIF退役完成，非旧AutoVM/A2X产品退役 |
| [865](../../plans/archive/865-av-source-closures-and-hof.md) | archived r2，9/9 | 闭包源码、捕获/逃逸/间接调用及增量会话已交付5fa3133fa，不含map/filter/reduce库能力 |
| [863](../../plans/archive/863-av-console-host-import-and-print.md) | archived r1，8/8 | AV host-import、print/eprint、精确字节父侧断言已交付2d5f3d842；native/legacy/a2r新增console profile的peer gap仍显式保留 |
| [889](../../plans/889-native-collections-variants-source-closure.md) | executing r2，5/10（本次读取） | T-06正式验收/性能修复仍在实施；shared source/ownership事实与H占用须逐符号重核，不据功能提交宣称D889；不含typed entry |
| [891](../../plans/891-native-wide-scalar-typed-entry.md) | drafting r2，0/10 | typed六标量entry及wide内部pure data闭包；默认889后取N，H另交888/889，非完整宽数值语言 |
| [884](../../plans/archive/884-language-structure-closure-without-crate-split.md) / [887](../../plans/archive/887-test-governance-and-ace-first-corpus.md) / [888](../../plans/888-module-test-selection-and-ace-batch-pilot.md) | 884/887已归档；888 executing r2、5/10（本次读取） | 模块/批量工具及正式成本/入口/验收未全部关闭；新A2X case须交实际H/发现/源锁及采样窗，不消费未交付配置 |

S=共享语义/IR/runtime/session，N=ac-mn，H=公共driver/SDK/测试/binding，P=登记私有模块。已交付862/864/866/868/870/871/881/865/863不再排重复实施。889实施S/N集合原源及真增量；891后接N typed数据，具体阶段释放按commit/符号验收，不消费WIP。H与888/889逐文件交接，不持一窗等待另一窗，正式采样错峰。完整顺序/立项表以[2026-10-29对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)及其后续更新为当前规划，旧[波次合同](../../plans/reports/ace-wave-handoff-2026-10-28.md)保留公共单写规则。

D门要求真实交付，不消费review/repair中的WIP，不提前合入未过审代码换取并行。前owner交接合入revision、Spec/API/profile/lowering版本、兼容结果、cache失效和限制，后owner记录接受；跨S/N/H同时取窗，不持一个窗口等另一个。新增公开枚举/签名需同步全部实际消费者的穷尽匹配，不把兼容编译失败留作他方债务。

871 **G-COV**列全core/scalar/text/record/text-record旧五域，含静态/只读方法、mut/move/clone、alias字段、Unicode、资源及全部协议/CLI/脚本/cache消费者；D870不是全集证明。**G-DELETE**要求全部覆盖/shadow/迁移及旧键恢复核验齐全；保留`text_resources.rs`的verify层`TextResourcePlan`及调用，只删旧native独占部分。

M1剩余按具名责任分组：native集合/变体与typed-entry、T03/T05共享文本真源、B17–19诊断/infra归因、场景trace/events/layout接线、资源fault/句柄代际、a2r解析/未定型发射/缺金样，以及完整分母适用性与证据归一。集合/变体由889认领、typed entry由891认领；其余完整能力/接线/验收包仍待真实复算，先复用871等已交付producer证据再判真缺口；242只是a2r长期tracker，887/888不自动承担全组。865/863已交付，先完整复算，再按剩余shared文本→诊断→场景/资源→a2r的默认优先级冻结有界包；863新增输出peer gap单列，不随AV交付冒绿。N/R/H独立范围可提前并行。M2数值/模块/stdlib/泛型-meta-反射/复合CTE/FFI及RF-I/RF-C继续按真实缺口规划，不预占编号，不以完整AJ/发布器/block增量作为统一M1门。

拆分线867/869/872/873与图形线独立推进，只有实际SDK、语义或测试写面重叠时按owner交接。新会话先查真实状态，不因本快照启动、暂停或修改他方任务。

**新增A2X规划（2026-10-30）**：[Design51](../51-shared-a2x-and-hir-to-rust.md)及[战略§3–5](shared-a2x-and-hir-to-rust-roadmap.md)是后续立项依据。首期AX-A现对应[PLAN-892](../../plans/892-shared-a2x-hir-to-rust-scalar.md) drafting r1/0-of-11，AX-B/C仍是未编号设计标签；后两项按正式接口/资源事实再取号。私有A2X/Rust可独立于N准备，S前端/受检函数/resource投影与H SDK/认证/测试/治理必须实际交接；D889/D891/U888不作整包统一前置，未交付的具体事实不能跨越。日常emit/金样spawn0与真实rustc认证分轴；不覆盖旧golden/expected或自动替换产品默认，本轮不施工、不改三项在途合同。

### 4.1 888 r3交接：ACE专属慢测（2026-10-30）

已接受用户经888定向交接的13个独立慢原子测试：syntax 1、semantics 1、AV 4、native 3、CTE 4，来源787/796/798/817/818/828；两项虽在language harness，实际归ACE。完整清单、历史日志/hash、静态归因入口及后续验收见[慢测后续处理](../../plans/reports/ace-slow-tests-followup-2026-10-30.md)。用户随后授权正式立[893](../../plans/893-ace-test-framework-and-slow-case-attribution.md)，drafting r2、0/12：复用888搭ACE共同接入/成本关联薄层，以原13例验证，不新造runner/meter；[设计](../testing/ace-test-framework-and-cost-attribution.md)明确单源边界。888负责分组/清单与证据交接；893负责框架增量与13例当前归因，产品算法/协议优化据证据后继拆包，不预占后继号。保留13叶身份和启用状态，原oracle/样本/数据规模/资源负控不变，暂关须用户逐项决定。本轮只规划，不施工。

888当前r3只要求模块归组/数量和每模块单轮墙钟、CPU、内存，已取消全目录三轮/P90与新A/B/C矩阵前置；上表r2快照按历史读取。该调整不改各ACE能力正式周转门。本轮888不再重跑这11项，历史85–526秒不能作为当前候选已测数据或单次请求延迟。当前其余模块采样现场冻结：先只读归因/规划，采样结束并取得实际写面收据后再安排实现或重型验证；共享S/N/H、worker/helper/注册逐符号交接，不与888测量并发。

### 4.2 共享文本T03/T05已立项（2026-10-30）

[PLAN-894](../../plans/894-shared-text-source-and-resource-closure.md) drafting r1、0/13，按[专题设计](shared-text-source-resource-contract.md)收原T03=23/T05=42。已静态核T03 Str局部`=`明确拒绝、lower Assign→Move与view eq/ne→StrCmp已存在；原源运行/资源/性能本轮未测，不据旧gap重复施工。范围为var/live局部Str完整替换，含RHS先物化、根借用/自move拒绝、故障与清理、AV/native真实函数级增量/恢复、文件化独立oracle；不扩891宽ABI或892 Rust资源发射。

主写S及独立测试，native必要最小owned-source单体桥；复用既有recipe/dispatcher/wire。若lowering契约真变，最小H身份镜像/stale同步必须纳入，不以“独立”漏升版本。本轮最终核实888已archived 10/10，delivery `5a6c23afa`、cleanup `129ef6057`均在main，原树已清且公共写面/测量窗释放；开工按实际提交/符号取S/N/H微窗，正式采样错峰，不设D888/D891/D892整包前置。当前只规划，无树/代码/测试，不把立项当缺口已关闭。

## 5. 首例即适用的验收和工程纪律

- 同一原始源及独立oracle，分别记录source、HIR/MIR、wire准入、AV/native执行、资源和真实消费者；不改expected、删import/分支或换fixture凑绿。新变体另编号，旧分母保留。
- fresh/hit/restart/body-edit、依赖/配置/权限改变、损坏缓存及强制退化反控从首例纳入。未改函数生产工作=0及IR/代码身份保持；全模块重算后某层cache命中不算全链增量。恢复decode+verify，不恢复trusted标记。
- 长期亚秒目标保持，**现开发期硬门P90≤2000ms**（PLAN-858及current Spec），P95/P99另报，代表负载/样本/结构与内存门不能删。T-02按上游预算交集冻结，拥有元素的clone/失败/清理、旧代回落和cache容量都要测，不自行扩大上限。
- 实施在每Plan唯一worktree，主检出仅计划/docs簿记；不建junction/symlink。保持他方未提交改动。开发/repair只跑定向检查；最终独立review后由merge在冻结提交跑裸cargo t及触面档，tf归主检出到期单实例；docs-only不跑cargo t/docs_gen。
- `attachments/**`原始源码/日志/JSON/截图证据只本地保留，禁止add/commit/push；Markdown摘要记录命令/退出码/数字/路径与hash。不自动新建会话或向其他会话发送消息；本次交接未启动施工。
