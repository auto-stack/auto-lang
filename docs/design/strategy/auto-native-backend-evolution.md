# ACE架构总路线：统一编译、解释、JIT、转译与增量热重载

> 2026-11-09当前M2排程：A先交完整语言/新库/FFI及具名真实消费者，B集中接管所有产品、发行默认与旧实现退役；四端语言/资源/原正式计量仍分别验收。AC的Auto主体AAC/ACC与更大AACE仍独立后续v0.7，非M2硬门，不因A语言完整自动认领自举；不同时强加自研机器码后端。详[Design55 §16](../55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)。此条替代早期全面交错排程，不改原已交实现。

> **2026-11-06 M2完整重规划（仅设计/草案，当前入口）**：[Design55](../55-ace-m2-complete-language-and-product-replacement.md)与[执行路线](ace-m2-delivery-roadmap.md)将完整语言四端、新标准库、Schema/AutoUI/AGE、全部产品/目标、安装退役拆为959–987；945–948 r2保基础职责并指定后继。936/944/950已归档；938/941在途、939草案，原M1合同不改。旧“未立项”描述仅为历史。全部新包drafting，未实施/切默认/签章；REP/V06原门不减。

> **2026-11-05 CA-E／ARCH正式立项（仅规划，旧快照保留为历史）**：[941最终组合验收](../../plans/941-ace-m1-expanded-and-architecture-acceptance.md)drafting r1、0/15，新增独立target/机制registry与acceptance/3父侧总门，旧313 required/批准hash/NA/accepted/future及source/expected不改；Q4各对独立oracle、完整Rust/C认证、真实函数级P0/IO0、无预热恢复首编辑/首坏修复、原资源/样本/P90与同PID HMR均有独立反控。只读基面5f6737467：929 D29a7d7007与937 Dfa35943ac已归档交付；934已reviewed待D，935/936 executing1步，938/939仍drafting。941私有登记/reader/oracle可先，最终联合取D934/935/936/938/939；公共S/N/X保持原owner，H全文件与正式采样分窗，不把reviewed/WIP当D。另静态核出原313能力token无非绿，但38旧A2R cert-gap仍被924现行守卫阻签，按CE-BASE-01显式保留、不得以新HIR证书替旧域或默撤守卫；验收系统交付不等M1签章。详[基线/38项/写面排期](../../plans/reports/ace-ca-e-arch-baseline-and-schedule-2026-11-05.md)。M2全部旧场景接管/disable仍是v0.6硬门，本轮不施工。

> **2026-11-05 CA-D正式立项（本条为当前快照，下列早期状态保留为历史）**：有限ABI/FFI桥拆[937 CA-D1共享事实/标量底座](../../plans/archive/937-shared-foreign-interface-and-scalar-abi-bridge.md)→[938 CA-D2受检源码foreign标量](../../plans/archive/938-checked-source-foreign-scalar-calls.md)→[939 CA-D3资源/句柄/同步回调](../../plans/archive/939-foreign-resource-handle-and-callback-bridge.md)，均drafting r1、0/12、0/13、0/14，仅规划。937带797 native与新A2C真实consumer/source导出、独立C认证，保持不改core/source/shared lowering/a2x-session恢复，可与929/934主体并行；H整文件与正式采样仍需短窗。938取D937及929稳定source接口，934若已接管则消费其facts权威，939取D938及已交925/924资源底座；不无条件等935/936整波，也不绕开实际读到的实例/CT依赖。完整CA-D须三包齐，最终CA-E/ARCH组合签章仍未立项，不预占号。当前main122725e9a：929executing r2 10/14由934获授权收尾；934 header drafting1/13但已有执行授权，935/936 drafting0步；928/931/932/933已交付。未跑实现/测试，不改在途合同/旧expected/分母/NA；797仍candidate/descriptor3/原预算，v0.6 M1含C与M2替换旧引擎disable硬门保持。详[基线/拆包/排期](../../plans/reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。

> **2026-11-04 934–936 r2复核（当前规划优先）**：三包仍drafting/0步，仅规划。934补完整scope/body-node事实端口、切断core旧类型综合的真实四向反控、928目录来源ID与语义LogicalId分离；935补逻辑实例/有效实现/target引用与实例集合恢复；936补子CT→实例checked版本→外CT失效、ExecKey/UseBinding分层、真实AV边界/严格bool投影、显式Hash相位和累计deadline。source-query prepare与selected-target宿主安装分工、929/934/935/936新页required/R准入责任已明确；恢复首编辑无预热、原P0/I/O0/预算/P90门不松。D928=6b429502f、D931=84c894b92已交付，931 /2和D-931-1/3余账继承，932已执行、933审计已有oracle；929恢复纪律不代解除。建议929公共来源+934私有query并行，934端口释放后935模板主体与936 AV服务可分窗；core/source-query/grammar/Cargo/父decoder/H不并写，最终真恢复/真实实例CT组合不能pending冒PASS。详[复核/新增反控/实际写窗](../../plans/reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)。下列r1与旧状态保为历史，不以草案代实施。

> **2026-11-04 普通源码SQ/实例/CT正式三包规划（仅规划）**：已核796真实有限query/S796与817真实AV/冻结/回灌基线，历史delivery b76eca8d2/ec6a54c5d在当前图非祖先，按当前Spec/源码hash重核继承，不用旧PASS背书新链。正式立[934 SQ-A源码query/依赖失效](../../plans/archive/934-source-semantic-query-and-invalidation.md)、[935 SQ-B有限真实标量实例](../../plans/archive/935-source-scalar-template-instances.md)、[936 SQ-C普通源真实AV CTE/冻结恢复](../../plans/archive/936-source-av-cte-query-and-frozen-recovery.md)，均drafting r1 0步，设计见[Design42 §10](../42-semantic-metadata-inference-and-comptime.md#10-普通源码sq真实实例与av-cte接线2026-11-04target-state)。实际顺序934→935→936组合；935/936私有准备及936最小AV服务调查可分窗并行，公共AST/grammar/恢复/H单写；新source-query无backend依赖，CT最小服务通过中立端口注入，不让core反依赖comptime或A2X目标依赖MIR。928/929原owner及929挂起保持，931–933各保资源/typed/oracle职责。最终都核真实依赖、函数P0、R深验和无预热恢复首编辑/失败修复、原预算/P90；不改旧expected/分母。详[基线/拆包/写窗与M1余账](../../plans/reports/ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)。SQ职责已立项不再列未取号；CA-D有限ABI与最终ARCH组合门仍待立项，M1/M2/v0.6硬门不变，下列状态为历史快照。

> **2026-11-04 931–933 r2复核（当前状态优先）**：D927=`5d5054d5d`已交付归档/清树；928 executing 4/12，929 executing 1/14按用户裁定挂起等D928；930 drafting r1为独立生成交付治理。931–933均drafting r2、0步，仅规划：931补真实消费点/派生view/短路隔离/片段原modes和资源事件；932继承914“有效typed策略拒绝为verdict”，修坏typed准入与911旧句/t11旧Error断言漂移，不回退批准；933逐项/逐字段独立oracle，不复制actual。三包主体可并行，S/H单写；933先交904独立材料，931/932最终恢复首编辑联验接929稳定接缝，无需整包互等才开工。**933完成不等于M1**：仅933先交仍有928/929/931/932；整波后仍需SQ/INST/CT、CA-D有限ABI、CA-E/ARCH组合准入三条未立项机制线，粗拆4–5后继包待可行性核清，不预占号。详[本次复核/写窗/距M1](../../plans/reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)。原预算/分母/批准/P90与v0.6 M1+M2硬门保持；下列初订/早期状态均为历史快照。

> **2026-11-04下一波正式规划（本条优先）**：main aa6791649核D923=ddd42ef2b、D924=48c2d8066、D925=a37245cd6、D926=3fce61b9a已交付归档；927 reviewed r2/10-of-12待原owner合入，928已获goal授权等待D927、头仍drafting，929仍drafting。新立[931 Rust临时消费者生命周期](../../plans/archive/931-a2r-owned-temporary-consumer-lifetime.md)、[932 native暖编辑准入/分类](../../plans/archive/932-native-warm-edit-admission-and-refusal-closure.md)、[933跟踪oracle/新树重放](../../plans/archive/933-ace-tracked-oracle-and-fresh-checkout-closure.md)，均drafting r1未施工，优先闭D-925-2/D-927-2/D-925-1及904缺manifest。原语法927合入→928→929继续，可与Rust私有/native反例/证据面分窗并行；933稳定材料先供最终consumer验证，不构成代码语义循环。CA-D前置已具备，建议后续按真实typed foreign标量与资源生命周期拆包，尚未取号；SQ/INST/CT与组合签章继续待立项。930是他方生成交付治理，不是本波ABI号。详[调查/优先级/完整排期](../../plans/reports/ace-next-wave-correctness-and-admission-2026-11-04.md)，原规则/预算/P90/v0.6 M1+M2硬门保持。

> **2026-11-03 M1架构机制审计与语法正式拆包（本条优先）**：已按源码/Spec/收据完成[剩余机制调查](../../plans/reports/ace-m1-architecture-gap-audit-2026-11-03.md)，真实语法链拆为[927基础七profile](../../plans/archive/927-unified-source-syntax-foundation.md)→[928集合/闭包与嵌套单元](../../plans/archive/928-unified-source-collections-closures-syntax.md)→[929全入口增量/恢复/重复parser退役](../../plans/archive/929-unified-source-entry-incremental-retirement.md)，均drafting r2、0步，仅规划。已有scalar AV热CST、真实CTE、cache/HMR机制不重做；普通源码语义查询/实例/CT接线、CA-D有限ABI桥及最终ARCH组合验收仍待立项，不因三包完成签M1。923 executing3/12、925头drafting但2/14且有实际树，按在途对待；924/926draft。语法私有研究可并行，公共S/H按稳定SHA/完整consumer取窗，native923→924、C925→926语义顺序不变。新M1仍需原门+C门+SYN/ARCH门，M2全覆盖/替换disable与v0.6硬门保持；不改原计划进度/实现或批准。 本轮r2补实片段上下文/typed资源态、同文lambda身份与payload复用、无预热恢复首编辑/原子来源发布及预算适用域；[完整复核、近期状态与安排](../../plans/reports/ace-927-929-plan-review-and-schedule-2026-11-03.md)列全部七个未交付包与908–922已交付基线。

> **2026-11-03当前执行序（926新规划）**：921 execution_done11/14待独立review/合入→[925 CA-B](../../plans/archive/925-a2c-record-text-resource-closure.md)→新立[926 CA-C集合/变体](../../plans/archive/926-a2c-collections-variants-resource-closure.md)，后两包drafting r1/0-of-14，只规划；native923 executing2/12→924 drafting0/14独立主线并行。公共S/H具体符号/全文件单写、实际SHA与全部consumer交接、正式采样错峰，无D923/D924整包挡A2X。CA-D有限ABI桥得A/B稳定接口后可与926分窗，CA-E准备随首例、最终在旧M1/native三格及C A/B/C/D/Q4全部门后，D/E未取号。见[Design51 §15](../51-shared-a2x-and-hir-to-rust.md#15-ca-cc集合变体与近期实施顺序2026-11-03target-state)。922盘点已交付，M2执行/宿主/TS/应用与发行/disable仍需独立计划，不以C线完成代v0.6全替换。取材main8717bd5d3、收口003e1babe；本轮只设计/取号，不改在途进度/源/规则或运行测试。

> **2026-11-03后继A2C与ABI/FFI规划**：921 CA-A在开发，分支已有scalar C真生成/共享target/来源恢复但未交付；923实施/924等待沿用户状态，922盘点已归档9/9。新立[925 CA-B](../../plans/archive/925-a2c-record-text-resource-closure.md) C record/text/参数模式/static-view方法/资源，drafting r1仅规划，见[Design51 §14](../51-shared-a2x-and-hir-to-rust.md#14-ca-bc-recordtext资源与后继abi接缝2026-11-03target-state)。统一接口metadata/分域ABI/FFI adapter与module生命周期见[Design41 §11](../41-interface-metadata-and-foreign-abi.md#11-a2c统一abi与ffi的近期结构2026-11-03target-state)：CA-B不冒公开ABI，CA-D仍M1必需，待A/B稳定接缝与双真实consumer可行性后正式立项，可与CA-C分窗，不预占C/D/E号。922还列M2-EXEC/HOST/SESSION/TEST与TS provider/产品迁移等未立项后继，不能从native三格收口推断整条路线结束；M2/disable仍v0.6硬门，未使用新生态不加前置。本轮只文档/取号，921/923/924进度与WIP不动。

> **2026-11-03 native剩余三格正式分包（本条优先）**：D915=5ba5d4cd2、D919=4ea3daddf已主线交付，917/920亦归档；旧required静态仅R02/R03/R05 ac三格。新立[923故障/Place基础](../../plans/archive/923-native-allocation-fault-and-place-conformance.md)与[924有限native闭包](../../plans/archive/924-native-closures-source-and-resource-closure.md)，均drafting r1未实施，设计见[§21](../43-ac-av-typed-mir-and-abc.md#21-native-r02r03r05故障与place基础有限闭包分包2026-11-03target-state)。native今天只有profile4，拒env导入/闭包指令，不能从919事件观察推定闭包已支持。923独占关闭R03，交R02非闭包事实但保持R02 gap；924补原K02 creator退出/真间接调用/最后env清理与逐fault，再按最终runtime重验923全域，唯一关闭R05和完整R02。N先923→924稳定短窗，私有研究可先；921 A2X主体独立，公共S/H逐文件/符号单写且核所有consumer，922已有实际树按在途对待，不改其合同/进度。无D921/D922整包前置，正式采样错峰。两包后旧三格全闭仍不签含C的新M1/M2/v0.6；AACE v0.7/旧批准/expected/分母保持。本轮只设计/立项，以下早期“native三格未立项”保历史。

> **本轮收口状态校正**：main随后推进c3304ed73（915实施记录），915头已execution_done6/10、待独立review，仍无主线D915；917继续实施。下条5a0ef7609为本波取材快照，不能据915工作完成推定共享接口已合入或整包已交付。919–922仍只规划。

> **2026-11-02下一波正式规划（main5a0ef7609快照）**：913/914/916已交付归档；用户确认915/917实际实施中，头状态滞后不代改。新立[919资源事件六格](../../plans/archive/919-ace-resource-lifecycle-event-conformance.md)、[920旧AVM B18一格](../../plans/archive/920-legacy-avm-immutable-diagnostic-closure.md)、[921 CA-A共享目标+C标量](../../plans/archive/921-shared-a2x-c-scalar-target.md)、[922 M2全场景替换盘点/关键路径](../../plans/archive/922-ace-m2-replacement-inventory-and-critical-path.md)，均drafting r1未实施。见[Design43 §20](../43-ac-av-typed-mir-and-abc.md#20-915917在途时的下一波2026-11-02target-state)。先可并行921/922私有写面与920旧codegen；919 native/fixture先行、AV与915按实际符号取窗，公共H全文件单写/认证错峰。旧required静态14格中915/917各2、919六、920一，native R02/R03/R05三格仍具名open，由922核能力/拆包，不自动NA；C后继B..E与M2工程/应用迁移继续另立，不由本波签M1/M2/v0.6。以下旧状态由此更新，本轮只规划/取号，无代码/current Spec/原批准/expected/登记变更。

> **2026-11-02最新版本约束：M2全替换是v0.6正式发布硬门。** 见[v0.6发布门V06-ACE-01..05](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)。M1含A2C及M2 REP-01..11全部完成，旧AutoVM/A2X实现退出构建/调用/发行后才发布；不是只切默认或保留隐藏legacy。用户接受为此调整进度（如晚一周），一周并非实测工期，须核全场景关键路径再冻结日期。M3/M4新增扩展暂不规划，AACE仍v0.7；下方早期迁移非发布前置/可延期表述由本裁定取代。本轮仅规划，不改变在途Plan或实际引擎状态。

> **2026-11-02当前主线：M2补全ACE能力，并彻底替换AutoVM/A2X。** 见[M2合同r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)与[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)。M2须接管AutoUI/AutoMan等全部现有应用、所有在用转译目标/CLI/SDK/嵌入与工具入口，完成工程增量/在线更新/资源/周转、实际部署切换及旧执行/AST转译实现disable；不是只到“可替换”。旧M4迁移/退役职责全部并入M2，未来M3/M4新增扩展暂不考虑，不接收M2遗留。默认开发快路径预构建ACE/宿主→Auto增量→AV或支持入口AC，稳定宿主Auto编辑rustc/cargo0；现有各A2X目标仍由新provider提供发布/转译，不取消模式或留旧emitter兜底。M1含A2C、原批准/矩阵/独立oracle/资源/P90≤2000ms与AACE v0.7保持；本轮只规划，不取号/施工/切默认。下方较早里程碑表述按本r2解释。

> **2026-11-02新增用户裁定：A2C第二目标加入M1。** 见[范围增补](../../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)、[Design51 §13](../51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)及[C实施路线](shared-a2x-and-hir-to-c-roadmap.md)。首期预计CA-A目标合同/去Rust耦合+C标量→CA-B record/text资源→CA-C集合/变体，CA-D有限797 ABI桥按A/B稳定接口分窗，CA-E自首例接新增登记/Q4并在全部门后收口；五个工作标签均未分配Plan号。Q4=AC-native/AV/新A2R-HIR/新A2C-HIR，各对独立oracle；旧A2R、AVM、旧A2C不替新身份。新M1必须原313门AND新增C核心/资源/增量/恢复/有限ABI/Q4认证及登记门，旧acceptance/2只能报告旧基面，不能单独签扩大后M1。通用FFI/header importer/Rust shim/Python/跨线程callback/loader不随C加入自动承诺，AACE仍延后v0.7。main44a774998读取时913恢复3/12、914场景在途、916已有树，915/917仍drafting；头滞后不代改，共用S/H取具体SHA/全部consumer窗，无D913–917整波前置。旧34阻塞/剩余10格估算只计旧M1，须另列新增C门。本轮仅设计/规划，不改current Spec/源/expected/binding/旧批准hash或在途合同，不取号/建树/施工；以下早期“第二目标Python/可选C/M1旧门”由本增补取代。

> **PLAN-858 口径更新（2026-10-27）**：本文历史 ACE P95<1s 目标在开发初期由用户调整为 P90≤2000ms；原正式样本和正确性、结构、资源约束保持，架构基本成形后以实测另定长期目标。其他 P95 数值作为历史或诊断保留，不再作为 ACE 开发期交付硬门。当前规则见 [TestCase v2 周转口径](../../specs/auto-test-case/profile-v2-native.md)。

> 2026-10-24用户裁定：v0.6先完善Rust版ACE语言引擎，按M1–M4推进并复刻现有AutoVM/A2X全部示例；Auto版AACE实施延后到v0.7。本文旧“本版AAC自举/小子集满足即可提前自举”排期由[新版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)取代，技术验证与既有Plan范围保持。

> 2026-10-21 版本范围更新：按[月度滚动规划](../../roadmap.md)，v0.6 仅保留工程重构、ACE、AutoGraphics/Forge 雏形与 Linux 四条主线。本文下方旧 v0.6/v0.7 版次和日历按历史排期读取；其余目标进入[远期 TODO](../../roadmap-future.md)，不自动承诺 v0.7 全部交付。技术合同与已批准在途 Plan 的范围/验收保持，四线所需局部能力随所属主线验收。

## 0. 总入口与后续规划导航（2026-10-20）

**2026-11-02下一阶段正式规划（以本条优先）**：911 archived10/10（Dca4848f5d）、912 archived12/12（D13a465336）已在main；913已开工T-01/1-of-12，计划头drafting滞后不代改。当前main0e93e3cc9，binding/applicability静态复算34阻塞，非新正式验收报告。正式立[914命名求值顺序/布局观察](../../plans/archive/914-m1-evaluation-order-and-layout-observations.md)12格、[915 AV故障/Place防护](../../plans/archive/915-av-resource-fault-and-stale-place-conformance.md)2格、[916旧A2R静态合法性](../../plans/archive/916-legacy-a2r-semantic-admission.md)8格、[917旧A2R mut-record/文本证据](../../plans/archive/917-legacy-a2r-mut-record-and-text-evidence.md)2格，均drafting r1、0/10。设计见[Design43 §17](../43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)；先916私有准入/914私有fixture可与913并行，915主av-core资源面取913实际S短窗，917取916已提交稳定公共入口后接发射，不互设整包前置。H按可合入任务逐文件单写，正式采样/认证错峰。四包24格互斥，全部交付仍预计余10格（资源事件6、native R02/R03/R05三、B18 avm一）及M1独立验收；native R05闭包适用边界先核真实规则/能力，禁止自动扩NA。新HIR-A2R不替旧身份、38 cert-gap/future45及具名旧红保留；日常零Rust编译、原source/expected/分母与P90≤2000ms/资源/恢复/零生产合同不降格。本轮只有设计/计划/路线文档，无实施/测试/建树材料或当前Spec变更，下方较早状态按历史读取。

**2026-11-02当前后继立项**：[912闭包源码单函数生产](../../plans/archive/912-closures-function-production-closure.md)与[913来源绑定恢复](../../plans/archive/913-closures-source-bound-recovery.md)均drafting r1、0/12，设计见[§16](../43-ac-av-typed-mir-and-abc.md#16-909后继闭包源码生产粒度与恢复入口2026-11-02target-state)。909 archived12/12、delivery59b8107eb已在main；其语义/K06/owner cache交付保持，R1模块plan/bind、R2整HIR verify、R4兄弟lambda重走及整源body parse由912收口，R3先lower/encode求key由913收口，不重开909。已有verify_function_ref不准Closures，912须补真单函数授信/事实及scoped消费而非仅改域match；913从parent已授权root强摘要进入，所有持久数据深验、无P源生产后实际执行并能继续下一单body编辑，source-only自报hash不可授信。顺序912→913，911 N/parent在实施可并行，S计量/失败快照/公共H取具体稳定符号窗，不设D911整包依赖。新增front16MiB、恢复manifest4MiB/自有namespace128MiB及原8MiB code/value等预算和四状态30样本P90≤2000ms同验；至少64顶层函数/每被测owner3兄弟lambda的真实规模与原样本保留。D-909-1 native旧红、完整C-19/旧a2r/scenario等另具名，不因A/B规划或交付签M1。当前只规划，无源/expected/binding/current Spec或施工变更；以下较早状态按历史快照读。

**2026-11-01当前下一包**：[911 native源码诊断事实与B17–B19收口](../../plans/archive/911-native-source-diagnostics-and-m1-b17-b19.md)drafting r2、0/10，[Design43 §15](../43-ac-av-typed-mir-and-abc.md#15-native源码诊断事实通道与m1-b17b192026-11-01target-state)。908已归档12/12、delivery dc8eca3ae在主线；其冻结核算core247/285、38阻塞、extension28/28、future45、cert38 gap，不能冒M1。909由用户确认在实施，共享捕获/K06归该包；911选现有compiler原因被mn.unsupported.frontend有损包装的三格，新增显式严格观察模式保raw stage/code/span/候选身份，默认兼容、近邻2/2/0与错族/infra/旧worker反控、真函数增量及失败后恢复，不新增语言checker或按expected猜原因。N/parent主体可与909并行，AV通用session只读诊断事实出口、contract/M1登记/manifest/固定组H逐符号短窗，不改909捕获/资源生命周期；910另属生成Cargo治理规划，非语义前置。路线顺序是909+911分窗→按真实required结果立场景观察/旧a2r合法性等净缺口包，AV资源先核已有profile机制→适用门全闭再M1签章。911本轮只规划无施工/当前Spec/原源/expected/binding变更，下方未刷新的状态作历史读取。 911 r2规划复核补齐操作级快照/真实终止profile与context、原始JSON互斥结果、typed拒绝与首失败停、失败全出口attempt、正式父事实重读及W-S1/2/H/M1稳定接口；明确标量worker与独立owned暖API，V扩至20，任务仍0/10，仅规划。

**2026-11-01 C-19源码切片正式立项**：[909共享可变捕获／K06](../../plans/archive/909-shared-mutable-captures-source-closure.md)drafting r1、0/12；[Design43 §14](../43-ac-av-typed-mir-and-abc.md#14-c-19k06共享可变捕获源码切片2026-11-01target-state)为拟实施合同。先区分原K06 move私有环境跨调用写入（原源42、外部不写回）与真正非move标量var共享cell（外部/兄弟闭包写回、拥有逃逸、clone独立/内部别名、最后owner清理）；原owned move/clone/Str借用兼容，循环图/共享owned值等C-19大类仍后继。909须修现有closures source先整编译/lower/encode再比wire的真实增量缺口，不用submitted单元数冒未改生产0。865/823已归档，当前唯一lowering路径为experimental/ace/crates/lowering，旧路径按历史；865历史SHA非当前祖先时以现行Spec沉淀/代码身份核，不重开旧包。908已真实实施（树有WIP，header滞后不改其状态）；909 S/P可并行，binding/规则/852/40场景由908，K06 AV check/token、recipe/worker/固定组H逐符号短窗。下一步是909 T-01/02最小全链，待后续实施指令；本轮只规划，无生产/current Spec/原源/expected/binding更改，不冒M1或完整C-19交付。

**2026-11-01当前收口入口**：[908](../../plans/archive/908-ace-m1-rules-registry-and-acceptance.md)已修订drafting r2、0/12，用户将G-DEC-1..5全部按推荐裁定、已有批准直接继承（[决策件§13](../../plans/reports/ace-m1-scope-decision-2026-10-28.md#13-补充裁定g-dec-15plan-9082026-11-01)）。61唯一适用映射=2旧批准+59本次批准；131/524全登记保留，原103核心+扩28已承诺AV双层required，O非AV45列future仍保gap/owner，a2r金样/认证继续分轴，不冒执行PASS。90310/10（18bac254b）、89312/12（b71b7343e）、904 r2 14/14（12f9adf18）均已归档且交付在主线；904 type/enum片段、资源事实及新recipe/fileset/cert已可消费，不再等待D904或旧S/H窗，新HIR producer仍不替旧M1 a2r身份。908后续实施从T-01冻结批准/正式基面，先最小strict链与tracked来源/窄诊断，再按批准迁移候选→原852入口/公共注册→消费者/独立复审→沉淀清理；P面可先行，906当前test-groups/宿主构建WIP按具体符号取短窗，905研究不设整包前置。908交付后按真实required结果先分证据/观察/infra与能力，再给旧a2r检查/发射、AV资源等后继；门全闭才签M1，console45/native loader等保产品/M2队列。当前binding/源/expected/golden/证书均未迁移，本轮只修规划与决策；以下旧状态按历史快照读取，822实际已归档的勘误不改903冻结数。

2026-11-01 AX-C规划复核：[904 r2](../../plans/archive/904-hir-to-rust-collections-variants.md)仍drafting0/14，20源及原预算不减，新增V-13..17明确同context异body凭证拒、完整snapshot/range连续编辑来源、真实payload/producer、无余量清理/工作取消、证书execution_config/mode与每采样修订独立认证，以及893903精确路径/符号/受影响来源回交。[Design51 §5.4](../51-shared-a2x-and-hir-to-rust.md)与[A2X战略r4](shared-a2x-and-hir-to-rust-roadmap.md)同步。type片段修复仍同包有界S前置，本轮只规划、无实施/测试/新号或其他Plan修改。

2026-11-01 AX-C正式立[904](../../plans/archive/904-hir-to-rust-collections-variants.md)，drafting r1、0/14：[Design51 §5.4](../51-shared-a2x-and-hir-to-rust.md)和[A2X战略r3](shared-a2x-and-hir-to-rust-roadmap.md)明确原20源、集合/变体/资源/真函数增量与strict认证。902已archived14/14、delivery4303c4457050在main；含type片段缺口纳904同包有界S前置，先最小真链再扩面，不另拆号。893/903用户确认实施中，按实际source/观测/H注册符号与正式运行窗交接，不互设整包等待、不消费WIP/改M1分母；本轮只规划，current Spec不据此冒实现。

2026-11-01近期规划刷新：891已归档实交（最终10/10，旧header5/10非当前完成数），889/892/894及901均已交付；899 executing2/7且已接U901，902用户确认已开始、lang-902树存在（计划头可能尚未同步，不代改其合同）。[893 r3](../../plans/archive/893-ace-test-framework-and-slow-case-attribution.md)仍drafting0/12，本次改为固定cargo t/原生命名导航与只读归因，不恢复tm/动态目录；[903](../../plans/archive/903-ace-m1-net-gap-audit.md)新立drafting0/10，负责当前M1登记/批准适用性/能力与证据净缺口、owner和后继顺序，不改binding/规则或签M1全绿。原登记现131行/524格只是raw范围，新增义务适用须按原批准合同核。两包静态规划/只读工作可与902并行，S/H符号和正式慢采样/认证错峰；899/902/893/903不互设整包前置。AX-C等D902资源/共享接口后立项；数值源码/多模块保未编号候选，不预占号。本轮只规划，不施工；下方旧状态按历史快照读取。

2026-10-31 AX-B已正式立[PLAN-902](../../plans/archive/902-hir-to-rust-record-text-resources.md)，drafting r1、0/14：[Design51 §5.3](../51-shared-a2x-and-hir-to-rust.md)与[A2X战略r2](shared-a2x-and-hir-to-rust-roadmap.md)明确record/text、参数模式/受检资源事实、Rust合法化/清理、真函数增量与独立认证。892/881/894交付基线已核；899/901分S/P/H微窗、不设整包依赖；旧core/scalar与旧A2X保持，AX-C仍未编号。仅立项，无实施/测试结论，旧状态导航以本次及实际Plan/Spec收据为准。

2026-10-30状态补充：888已归档10/10（delivery `5a6c23afa`、cleanup `129ef6057`均在main），原公共写面/测量窗释放；894共享文本T03/T05包已立draft r1、0/13，见§7.10。旧888等待快照不再表示当前前置；其他计划按实际新owner/符号交接，不自动开工。

**本文是ACE（Auto Compile & Execution）架构及AAC/AAV后续规划的统一阅读入口。** ACE统一编译、解释、JIT、源码转译与增量/cache/热重载，AC/AV/AJ保留组件职责。历史“AC/AV/AJ总路线”即本入口，文件路径保持。技术合同按主题落在下列设计中；本文维护职责、阶段和引用，不复制各篇的完整合同，也不作为已实现能力清单。

| 要回答的问题 | 继续阅读 | 使用边界 |
|---|---|---|
| AC/AV/AJ如何共用语义、MIR、ABC、CTE和增量发布？ | [Design43统一架构](../43-ac-av-typed-mir-and-abc.md) | 架构合同及近期/后续边界；完整AJ并非798或AVM退役的自动前置 |
| A2R/A2Python等怎样共享HIR与转换，HIR→Rust先做几包？ | [Design51共享A2X](../51-shared-a2x-and-hir-to-rust.md)、[首期实施战略](shared-a2x-and-hir-to-rust-roadmap.md) | HIR语义/结构化准备/编译服务共用，目标表示/AST/runtime分开；首期AX-A=892 drafting、AX-B/C未编号，S/P/H按889/891/888实际交接，未施工 |
| AV怎样表示数据、读写Place、管理move/borrow/drop和Dyn？ | [Design44数据与所有权](../44-av-data-layout-place-and-ownership.md) | 数据合同及820–824安全扩面、825/826性能分工 |
| 当前这一波完成后能运行多少语言、还缺什么？ | [AV能力矩阵](av-language-capability-matrix.md) | 区分源码/内核/库/产品层，记录语义迁移差异；目标表不等于已通过 |
| 下一阶段先做什么、怎样运行真实程序并替换旧AVM？ | [AV源码与替换路线](av-source-programs-and-avm-replacement-roadmap.md) | G0–G8、优先级、候选工作包、产品入口和验收；G7按入口切默认，G8才是全部退役 |
| 用哪些真实程序证明进展？ | [AP程序目录](../../plans/reports/av-source-program-catalog-2026-10-20.md) | 23个初始样本及oracle缺口，不是全部产品/语言库存，也不是运行通过记录 |
| 第一项源码验收计划是什么？ | [Plan827归档基线](../../plans/archive/827-av-source-program-baseline.md) | G0/G1有限scalar源码已交付；后续862–866/870/871等见续接基线，不能沿用初稿“均未分配ID” |
| 828之后如何排期、哪些可以并行？ | [执行顺序与交接合同（2026-10-21更新）](../../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md) | 优先817/818、827及820功能线；825/826择机插入，公共核心单写，独立消费层可并行 |
| 语法、全局cache、ABI和元编程的共同约束在哪里？ | [Design39语法/增量](../39-ac-incremental-syntax-and-language-platform.md)、[40产物/cache](../40-compiler-artifacts-and-cache-lifecycle.md)、[41 ABI](../41-interface-metadata-and-foreign-abi.md)、[42语义/CTE](../42-semantic-metadata-inference-and-comptime.md) | 按涉及的接缝读取，不另建重复类型系统、store或解释器 |
| Auto/Atom如何成为通用parser generator及语言工具平台？ | [Design39长期定位与路线](../39-ac-incremental-syntax-and-language-platform.md#language-tool-infrastructure) | CST/AST/HIR、生成器与小运行库、ABC/A2X插件和auto-edit/语义LSP分层；787有限实验与完整目标区分，本次仅设计 |
| AACE与Rust版ACE如何分版？ | [版本/里程碑合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)、[主体/代际合同](auto-acc-bootstrap-contract.md)、本文§0.7 | v0.6先Rust版全示例成熟度与迁移，v0.7再实施AACE；自举、产品替换和后端更换分别验收 |
| 现在实际已交付什么？ | [Spec总览](../../specs/overview.md)、[Spec索引](../../specs/INDEX.md)，按任务进入auto-ac/auto-hir/auto-semantics/auto-lang/vm/stdlib等模块 | current Spec为现状知识；代码/测试核实观察，Plan与收据确认具体交付。auto-av等拟议Spec必须先确认存在，不把计划中的路径当已交付 |
| M1还缺什么、哪些M2能力已推进、865/863之后怎样收口？ | [2026-10-29 M1/M2收口对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md) | 按缺口族列能力/判定owner、已立项/待立项与关闭条件；当前登记116行，旧gap不等净能力缺口；不是新验收通过记录 |

**当前续接（2026-10-30）**：先读[ACE决策与实现续接基线](ace-implementation-handoff.md)和[M1/M2缺口/收口表](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)，再核[公共写面合同](../../plans/reports/ace-wave-handoff-2026-10-28.md)。862/864/866/868/870/871/881及865、863已交付；889 executing r2、888 executing r2，891 drafting r2。当前处于M1共同子集收口与M2源码扩面交叉阶段；865/863完成不自动等于M1全绿或M2完整。可复制[续接prompt](../../plans/reports/ace-next-session-prompt-2026-10-28.md)恢复上下文，状态/实际D门须执行时刷新，不自动授权施工。

**native第二阶段更新（2026-10-30）**：按[拆包决定](../../plans/reports/ace-native-phase2-split-decision-2026-10-29.md)，[889集合/变体源码闭环](../../plans/archive/889-native-collections-variants-source-closure.md)正在实施；[891宽标量typed入口](../../plans/archive/891-native-wide-scalar-typed-entry.md)已复核为r2、未实施；[规划复核](../../plans/reports/ace-891-plan-review-2026-10-30.md)补全物理ABI、精确指纹、实际PE结果绑定与暖/冷增量分账。N默认889→891单写，稳定阶段可按实际commit/符号交接；H与888/889分窗，正式采样错峰。891新增内部bit-carrier ABI、独立精确结果通道及W01–05机器证据，不包含宽算术/源码推导、wide拥有集合组合、公开AutoABI/FFI或整个M1收口。

**A2X设计更新（2026-10-30）**：[共享A2X与HIR→Rust路线](shared-a2x-and-hir-to-rust-roadmap.md)已建立，首期拟AX-A共享骨架＋Rust标量闭环、AX-B record/text资源、AX-C集合/变体三包；AX-A已立[892](../../plans/archive/892-shared-a2x-hir-to-rust-scalar.md) drafting r1/0-of-11，AX-B/C未编号。新P-A2X/Rust主体无需等待native私有ABI；S受检事实/单体前端、H SDK/认证/测试治理须与889/891/888逐符号交接，不消费WIP。日常emit/金样与独立rustc认证分轴，首例具备结构增量/cache/恢复/预算；旧A2X产品迁移及第二目标共享实证另验。本轮不自动启动任何包。

### 后续Agent的规划读取步骤

1. 先读仓库指引、Spec总览和本文，再按任务选上表相关主题。AV扩面优先对照能力矩阵、G阶段/工作包和AP/EP清单；AJ/自举任务另走对应共同合同与主体清单，不能套用AV替换门代替其验收。
2. 核实相关模块current Spec、实际代码/测试和依赖交付证据。路线/矩阵中的日期与Plan状态是快照；按Plan ID在`docs/plans/`与`docs/plans/archive/`查当前路径、revision、frontmatter及review/merge收据。工作完成或分支通过不能代替D-N交付。
3. 检索是否已有相同目标的活动Plan，优先复用/修订已确认匹配项。仅追踪任务相关依赖、引用决策和消费者；不递归加载所有历史附件。Spec缺失/过时、目标态与现状冲突时显式记录差距，不静默改写任一合同。
4. 选择前置已具备的最小真实程序/消费者切片，再依`/auto-plan:new`冻结目标、非目标、写面/owner、oracle、资源与性能预算、依赖和Spec增量。导航不会自动授权实施、扩张在途Plan或选择所有候选工作包。
5. 新设计/计划应回链本入口及所属专题；阶段交付后经merge流程沉淀current Spec，更新矩阵的实证与路线门证据，并维护本导航。Plan归档时修复入口链接，以Plan ID继续追溯，不在总路线复制易过期的进度百分比。

可直接给后续Agent如下提示：

> 请以 `docs/design/strategy/auto-native-backend-evolution.md` 为总入口，按其中“后续Agent的规划读取步骤”核实相关Specs、设计、活动/归档Plan及交付证据，选择AV真实程序路线中下一个可执行切片；先复用已有Plan，没有匹配项再规划新Plan。说明所选阶段、真实程序、依赖、验收及尚缺能力，本轮只规划、不写代码。

若用户指定具体主题（例如AJ、CTE、自举或Place），将示例中的“AV真实程序路线”替换为该主题即可。给出这个入口足以定位相关文档；它不保证一次加载全部材料或自动完成依赖核验，Agent仍须遵守上述步骤并报告缺失引用/证据。

### 0.1 统一平台的早期结构约束（2026-10-20澄清）

层级/缓存粒度/格式的具体规划基线见[Design40 §3.2–3.6](../40-compiler-artifacts-and-cache-lifecycle.md#32-首批层级与粒度决策基线2026-10-20)：所有已实现阶段进入统一依赖与计量，持久化按收益分层；HIR/MIR/ABC的查询、产物、页与装载单位分别登记。下一轮RF-I/RF-C规划必须以此核对已有producer与缺口，不另建一份平行产物目录。

增量编译、层级化全局cache、解释器热重载是ACE从早期就必须考虑的基础，具体责任以[Design43 §6.1](../43-ac-av-typed-mir-and-abc.md#61-早期架构约束与分期实现2026-10-20澄清)为准。统一架构名采用ACE（Auto Compile & Execution），不改AC/AV/AJ职责或现有命令。

实现可分期，身份/依赖/生命周期和共同服务不能延期到完整语言或AJ之后再补。后续规划先核“已实现的增量粒度与cache层、仍待消费的共同合同、最小解释器在线热重载证据”三份清单，再选择语言/产品扩面；最小AV热重载验证应较早安排，不等UI/服务全齐。798实际热重载非目标及现有Plan合同保持，不能据本文冒称其已交付，也不把完整平台改造直接塞入798。

### 0.2 粒度演进：函数级先行，显式block与自动分块后续

已认可方向为“声明/函数级首版→显式block区域→自动语义分块”，具体架构空间见[Design43 §8.1](../43-ac-av-typed-mir-and-abc.md#81-函数级首版与block级扩展方向2026-10-20)及[Design40 §3.7](../40-compiler-artifacts-and-cache-lifecycle.md#37-函数内区域与block级扩展空间2026-10-20)。后续RF-I/RF-C/RF-H规划应核根区域/owner、可分解manifest、边界摘要和代际接口，避免把首版整函数布局固化为公共ABI。

block级编译复用、后续入口采用新版与活帧中途迁移分别验收。先从实际支持的显式block验证，不以新增语言语法、自动分割、OSR或每块间接调用作为当前Plan前置；自动分割未来复用同一合同。语言/数据/后端扩面保持既有依赖与单写纪律。

### 0.3 周转性能的分期策略（2026-10-20 用户澄清）

开发态继续以**初始化后的连续编辑到当前结果/准确诊断 P95<1s**为目标，不改为平均值。当前已实现流程先以其真实入口/冻结负载实测达标，未来热重载/数据/CTE新增工作的成本估计用于判断余量和安排次序，接入后重测；估计不能替代PASS，超标先定位优化，目标调整须显式决策。具体口径和预估边界见[Design43 §6.3](../43-ac-av-typed-mir-and-abc.md#63-周转目标与分阶段扩展策略2026-10-20-用户澄清)。

当前就保留逻辑身份/版本、增量依赖、编译—装载—候选安装—执行及generation/lease生命周期接缝，完整热重载能力可分期实施。798既有正式P95门和实际热重载非目标保持，不因本澄清新增实施phase，也不以跨进程恢复代替持续存活VM的在线更新证明。

2026-10-20进一步明确的函数级首版必须是**源码编辑到执行安装的全链路增量**：签名与真实依赖事实不变时，仅变更函数重做语义/HIR/MIR/ABC和受检安装，其他函数IR/已加载代码保持同一身份；整模块重算后命中某层cache不算完成。具体结构/双门验收见[Design43 §6.4](../43-ac-av-typed-mir-and-abc.md#64-真正函数级增量的全链路合同2026-10-20)与[828](../../plans/archive/828-av-function-incremental-session.md)。798 r6保留基础合同并准确标注边界，828在D798后交付完整切片，不反向阻塞798；825的运行优化、826格式和827源码入口都不能替代该切片。完整目标在828验收前未交付，P95仍为稳定会话编辑至当前结果<1s，首启动单列。

### 0.4 近期执行批次与核心骨架里程碑（2026-10-21）

2026-10-26实际依赖审计后，优先功能/源码路线保持，正式数据集成为820→821→822→{823，824}；824的白名单不消费823闭包，移除D823。823/824可在D821/D828上先做各自T-10私有同源模块，T-02均等D822；825/826可在D798/D828上并行做派生PC表/codec，826正式路由/LoadedProgram/AV仍等D825。优化集成依实测择机，不抢功能/源码前门。D-N始终要求真实交付，独立模块或局部review通过不解锁对应正式集成门，见[依赖审计](../../plans/reports/ace-parallel-dependency-audit-2026-10-26.md)。

817/818/827可并行实现各自独立消费层；共享source/HIR/语义、MIR/ABC/AV及测试注册默认单写。先满足真实D门，再核已交付基面/Spec/API版本、前owner的合入/兼容证据与窗口释放，后owner记录接受；不持窗等上游、不复制WIP、不要求在途消费者追逐新profile。不同worktree不改变公共合同互斥；拆分按实际迁移owner交接，不增加等待整批的门。各Plan T-01/T-02准备/刷新交接映射，既有兼容/coverage及review/merge核交接记录，具体以共同合同为准。

823 closures、824 dynamic/portable、825 loaded_program、826 compact_abc私有新源及各自parallel/<id>测试壳登记后可并行编译同一生产代码；不注册公共类型/opcode/lib，不复制IR/VM，不占公共核心窗。前项review/repair期间继续这些独立任务，公共定义未合入不交下游；实际mir/ownership/codec/verifier/dispatch及公共注册由单一核心owner串行接入。最后集成者重核已交付profile/消费者并集，闭包未交付与已交付时的Dyn拒绝分别按实际类型可达性验证，不留future缺口。

798、828、817/818及820–827全部实际交付后，ACE可达到**首版核心骨架基本成型**：共享受检IR/ABC/AV、函数级增量与统一cache生命周期、有限CTE/native消费者、数据/所有权与装载/编码内核、真实scalar源码验收形成可验证基础。它们的profile各自有限；typed fixture不等于完整源码支持，824冻结不自动让817/818支持复合值，独立通过不代表任意组合贯通。

后续仍按源码/stdlib与编译器主体的真实缺口安排泛型/元编程/反射、复合CTE/native、宿主和产品入口、自举等切片。完整AJ/优化JIT、通用增量发布器、block级增量和任意活对象/活帧迁移另立范围；旧AVM切默认/退役按G7/G8。这是核心里程碑，不等于完整ACE产品、AAC/AAV或本版全部完成。

#### 0.4.1 2026-10-29后继波次、M1收口与M2扩面（当前交接）

862入口/Unit、864集合变体原源、866真实AV在线更新、868单源lowering、870 mn data/owned、871旧native迁移退役、881 static/view方法与真方法增量、865闭包源码及863宿主输出已交付，均只认各自有限合同。889接集合native与真函数增量S/N，891后接typed数据N；H的SDK/driver/注册/binding与888及实际owner短窗串行，不因独立worktree视为无冲突。实际owner/立项状态见[当前对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)，[旧波次合同](../../plans/reports/ace-wave-handoff-2026-10-28.md)保留单写/交接纪律，不据历史快照重复启动。

864原L04/L05及List<Str>/拥有型type为实际交付范围；881方法producer已实交，不再记“待立项”。871已按旧core/scalar/text/record/text-record的G-COV/G-DELETE及消费者/cache合同交付，保留text_resources.rs的verify分析；不是旧AutoVM/A2X产品退役。未由方法包覆盖的文本/资源/诊断缺口不随D881或D871自动归绿。

**865/863之后的M1出口**：先以当前登记/正式suite复算，再冻结原源文本、诊断、场景观测/资源代际、a2r解析/发射/金样及适用性回填的剩余工作包。116条binding原始gap混有已批准NA/旧差异和待接线证据，不是116项都需新能力，也不能沿用50/103等旧比例。当前无新的全分母acceptance收据；M1签章须所有适用格真执行、阻塞gap/probe/wait/pending清零、NA/accepted-diff有规则、a2r金样与认证分轴，以及原增量/恢复/资源/开发期P90≤2000ms门。887只承担其测试治理和首批消费，不能自动替代完整M1验收。

M2已推进入口、集合/变体/is/有限for-in、record/text方法与拥有值源码；闭包/host-import由865/863继续推进。宽数值源码/算术、模块/全局/stdlib/HOF、泛型/meta/反射、复合CTE与必要FFI仍须按真实缺口另立包；不是typed能力自动扩面。M1收口与M2/M3扩面可交叉，完整AJ/发布器/block增量不成为全部旧能力兼容的自动前置；M4产品全示例/入口切换另验。

各阶段从首例接文件化AutoDown、cache恢复重验/增量结构、现行P90≤2000ms和内存/资源预算；亚秒长期目标不伪称已达标。SDK/公共测试写面需拆分owner实际交接；旧产品默认、M1已批准NA/accepted-diff和a2r金样/cert双轴不因排波而变化。Rust版ACE v0.6成熟度、AACE v0.7边界保持。

#### 0.4.2 共享A2X与HIR→Rust并行线（2026-10-30，仅设计）

新A2X按[Design51](../51-shared-a2x-and-hir-to-rust.md)共享受检语义view、必要结构化准备及查询/依赖/cache/来源/产物服务，各目标负责合法化、类型/资源表示、runtime映射与Target AST/打印；不把共有实现简化为一个Trans接口，也不重建前端或共享万能目标树。共同pass保持求值/短路/资源合同，能力和依赖按实际查询进入key；Rust/Python等不能因同源误命中目标产物。

首期三包各有真实闭环，默认AX-A→AX-B→AX-C，随后以第二目标Python scalar验证共享抽象，再择闭包/console、模块/库/泛型/复合CTE/FFI等实际缺口。包拆分、前置、首例必验及后继边界详见[战略§3–6](shared-a2x-and-hir-to-rust-roadmap.md)。新A2R不依赖D889/D891整项，但消费的未交付S修复需正式收据；H与888/889/891按实际owner分窗，计量错峰。不得因新backend首例通过把M1旧a2r列改绿或宣布全A2X产品退役。

### 0.5 基础图形之后的跨生态 CAP（2026-10-22）

后续统一机制为 **Context-Aware Programming System（CAP System）**：函数/语言特性 effect 推导、场景约束求解、合法实现选择、stdlib/包依赖裁剪。完整合同见[Design46](../46-context-profile-capability-cross-ecosystem.md)，定位与阶段见[跨生态战略](cross-ecosystem-programming-strategy.md)。现有 HIR/MIR 格式/执行 profile 与拟议场景 profile 身份分开，不能把当前有界能力门等同完整 CAP。

CAP 复用 ACE 的共享语义、Checked HIR、适用 Typed MIR/ABC 与各 backend lowering，以及函数依赖/cache/受检安装。effect 摘要变化即使签名不变，也应使真实依赖的 caller/根准入失效；target/profile/backend/variant 与数值策略参与产物身份。GPU stage/地址空间/barrier/uniformity 需要受检扩展，不假设当前标量 MIR 已能承载全部 device 语义；A2X 保持 HIR 结构化入口。

实现顺序为**新版 ACE 架构完成 → 基础 AutoGraphics 建立 → CAP 实现 → 跨生态扩面**。ACE核心骨架成型与新版架构实际完成分别认领，CAP进入门在后续Plan绑定冻结交付范围；不要求先完成无限扩展的AJ/自主后端/所有加速器。当前仅记录元信息与依赖接缝，不为798/828、817/818或820–827新增求解器/包系统任务，也不改变§0.4排期。基础图形可独立并行，完整CAP仍等两项交付门。

AutoShader的近期边界见[最小设计草案](autoshader-minimal-language-and-renderer-interface.md)：现在只设计，未来从ACE受检HIR/来源与具体类型支持lower到WGSL，经局部adapter接现有renderer。无需完整CAP全部扩面后才验证最小切片，也不假设当前标量MIR已支持GPU；基础图形可先用手写WGSL。830 r2只补资源/布局和交接，不向当前ACE Plan新增实现任务。

### 0.6 Auto语言工具基础设施（2026-10-22）

长期以[Design39](../39-ac-incremental-syntax-and-language-platform.md#language-tool-infrastructure)的Auto/Atom语言包统一grammar、树Schema、结构查询和可选语义模块，服务Auto编译器、auto-edit及其他编码工具。Grammar编译器承担解析算法，ACE编译生成器与Auto扩展代码并复用共同artifact/query、cache和生命周期；CST/语言专用AST不直接等同Checked AutoHIR。源码转译保持受检HIR入口，适用MIR/ABC/AV承载可执行扩展，不为parser产品另建语义系统、VM或私有store。

部署按需求分别验证小型原生运行库及绑定、ABC/AV插件、Python/C/Rust等A2X代码输出。生成器不随每个解析器发行，纯数据grammar不要求完整ACE；ABC插件须有宿主适配、能力/资源准入与代际合同，不能直接等同WASM或宣称任意IDE即装即用。完整CAP/AJ不是所有切片的统一前置，所需实际profile/API由后续Plan冻结。

[P1–P4路线](../39-ac-incremental-syntax-and-language-platform.md#language-tool-roadmap)为已有语法底座的Auto/编辑消费→差异语言与通用模型验证→ACE部署/插件→可选语义服务/LSP。787已交付有限实验，完整通用产品进入远期池；本次不创建实施Plan、不改变§0.4或798/828、817/818、820–827的执行顺序/验收，不新增v0.6完整生成器或Rust LSP门槛。auto-edit保持轻量消费，重型语义服务按需外置。

### 0.7 Rust版ACE与AACE的版本边界（2026-10-24）

用户已提高v0.6成熟度要求：M1核心框架成立，M2现有语言能力完整，M3 Auto/C/Rust/Python混合生态可用，M4产品迁移与旧框架替换；四项均为Rust版ACE，不是Auto主体自举。M2/M3交叉推进，基础FFI/必要宿主随语言扩面提前；M4按入口持续验证后收口。全示例按源码/依赖/资源/后端/平台/feature建立分母和独立oracle，不以typed fixture、改写示例、skip或隐式旧引擎fallback冒认兼容。

**AACE是Auto语言实现的ACE主体，明确延后至v0.7**；AAC（旧称ACC）/AAV是其组件子线。先完成Rust版ACE现有AutoVM/A2X全示例及成熟度验收，再实施主体迁移，不再因编译器源码小子集可用就提前并行。v0.6只保留将来Auto实现可消费的接口/格式，Rust版作为后续对拍基准。主体/宿主桥与代际范围由v0.7 Plan冻结，不把包装Rust前端或迁CLI当自举。

完整AJ/优化JIT、OSR/deopt、自主后端等不因兼容成熟度提高自动成为前置；所有现有Plan保持任务/进度/预算，后续按M2–M4缺口另拆Plan，既定D门与核心单写规则保持。完整合同与未达目标的处理见[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)。

## v0.6 路线阶段合同（2026-10-24分版更新）

**2026-10-26历史排期（本批已进入后继波次，当前见§0.4.1及续接基线）**：798/828基础后，优先功能/源码；正式数据集成820→821→822→{823，824}，优化集成825→826择机。四个T-10私有模块可按§0.4与[统一交接合同](../../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)并行，公共核心/注册仍单写；原进度/预算和全链路增量/P95/内存/反控门保持，不能用模块绿代替首例或完整Plan验收。

本节补充阶段目标/验收/拆分边界，不扩大已批准 Plan，也不将实验通过写成正式交付。总体承诺与候选阶段见[路线矩阵](../../roadmap-v0.6.md#7-v06路线阶段与拆分边界矩阵2026-10-16)。

| 子线 | v0.6 目标 | 验收与证据 |
|---|---|---|
| Rust版ACE成熟度；AACE移至v0.7 | 本版M1–M4、现有AutoVM/A2X全示例与对应消费路径真实兼容；先不实施Auto主体迁移 | 冻结示例/目标/依赖/宿主分母与强oracle，源码/语义/运行/转译、资源和迁移证据；v0.7另验主体/宿主桥与Gen1→Gen2→Gen3，不以Rust包装冒充AACE |
| 语义 meta/推导/实例化/双期反射 | 保留Design42本版元编程目标，按支持矩阵逐项推进 | 796有限探针及后续语言扩面分别验收；正反例、求值顺序、类型/声明身份、来源与失效可检查；编译期查询和runtime反射访问分别用真实消费者证明，不以描述所有类型证明已执行所有类型 |
| Typed MIR/ABC/Rust AV→CTE | 已确认进入本版；先建立执行子集，再接真实编译期执行 | 798 的 Checked HIR→MIR→ABC→AV 与 native/独立 expected 对拍、恢复重验和资源拒绝；后续 CTE 需实际影响编译结果，验证权限/确定性/预算/依赖变化，不能用测试模型冒充 AV 执行 |
| 增量/cache/语法服务 | 从首例纳入，不在末期补性能 | 冷/热、进程重启/跨工程、源码/依赖/配置改变真实命中和失效；首条用例纳入 AutoDown；沿PLAN-858/current Spec的开发期正式样本P90≤2000ms验收，长期亚秒及P95/P99分列；保留代表负载、样本、结构和内存门，记录完整请求而非只计编译阶段 |
| 发布/重载/分层 AJ | 现有 native 支持面逐步落实增量发布；完整 AJ 后续专项 | 发布固定完整输入/闭包，真实链接运行，冷函数与缺 cache 不漏编；重载独立验证旧代存活/一致切换/失败回退。完整热点调度、OSR/deopt 和性能优化不自动纳入有限 AV 探针 |

依赖顺序：已有AC/profile/store→796语义、797 ABI、798 MIR/AV独立阶段；按各Plan真实D门接CTE/语言/宿主与消费者扩面，再完成Rust版M4，v0.7才实施AACE主体迁移。797/798不是互相全局前置，既有合同不因本文改变；状态以当前Plan/Spec及review/merge收据为准。

拆分边界：实验核心保持独立 workspace、公开数据合同与受管 store，不引入旧 auto-lang AST/Value/UI/man 私有依赖。788 的正式 runner/adapter、任何生产前端或 VM 接线仍有协调面，不能声称整条线零影响。解析器/语义/IR可以独立推进；将旧 A2X/AutoUI 全迁移至新 IR 另立验收，不强加为自举前置。
待决：完整自举主体/语言支持清单、CTE扩面及发布/重载的具体profile，随专项Plan冻结；近期批次优先级已由§0.4及2026-10-21共同排期明确，不据此扩张各Plan范围或自动启动实现。

> 更新：2026-10-16（AC/AV/AJ统一服务、分层执行、增量发布及近期/后续实施边界）。具体接口、优化集与选型探针按专项 Plan 验证。
> 记录目标架构，不表示全部能力已实现。AC 指 Rust 实现的首版原生编译器；
> ACC 指 Auto 实现的编译器主体，现有 roadmap 的 AAC 与之指同一条自举路线，命名另行统一。
> 历史技术方向（2026-10-16）：最小Typed MIR、新ABC/Rust AV与CTE进入v0.6，语言级优化/SSA/自主低层后端按收益分期；其中AAC/AAV原排期已由§0.7的“v0.7 AACE、先Rust版成熟度”取代。技术层次见[Design43](../43-ac-av-typed-mir-and-abc.md)，不回溯扩大在途Plan。

关联：[AC 子集与目标代码](auto-ac-subset-and-target.md)、[HIR](auto-hir-design.md)、
[Native ABI](auto-native-abi-rfc.md)、[v0.6 roadmap](../../roadmap-v0.6.md)、
[旧 Native 战略](native-backend-strategy.md)。

## 1. 实现语言、执行引擎与代码生成的演进轴

AC → AAC（旧称ACC）和AV → AAV分别改变编译器与新版VM主体的实现语言；Cranelift → 自主后端改变机器码生成实现。AV是长期产品名，先Rust实现，不使用navm；现有AVM/AAVM是旧实现，不因新名称获得兼容/自举授信。
自举与后端重写分开验收；不要求实现 ACC 时同时替换机器码后端。
Auto 掌握源语言语义、HIR、编译流程、ABI、运行时与驱动；外部后端库和系统 linker 可继续复用。
ACC 的能力声明必须包含实际主体清单，不能只用 Auto 包装 Rust 前端或迁移 CLI 就称自举完成。
“完善”以约定语言覆盖/平台/编译器主体及其验收为边界，不等于整个 Auto 生态已原生支持。

### 1.1 统一编译服务与分层执行

AC负责语义/CTE、提前编译及发布驱动，AV负责新ABC解释执行，AJ负责按需快速/优化编译；共同消费Typed MIR、语义/Meta和全局cache，不维护三套类型系统或优化器。在线AJ与提前编译复用编译核心，触发和装载策略不同；无需先运行代码才编译冷函数。应用按部署profile裁剪，不要求携带完整AC。完整合同见[Design43 §8–10](../43-ac-av-typed-mir-and-abc.md#8-acavaj统一平台与分层执行)。

长期开发路径为T0解释启动→T1热点快速编译→T2收益驱动优化；若已有有效原生cache可直接使用，不重复热身。不同层共享语义/逻辑签名，不强制同一帧或对象布局；入口adapter、代际/lease及内联依赖显式。首版允许旧帧留旧代执行完，模块发布仍遵守一致快照事务。OSR/deopt/任意帧迁移和完整热点调度是后续专项能力。CTE先只用AV；以后采用AJ也不得绕过权限、确定性和资源计量。

### 1.2 发布是一套配置，不是每次冷编

发布固定源码/依赖/target/CPU/ABI/优化与runtime profile，计算入口、导出、初始化、反射和实例等完整闭包；校验缓存后只补编缺失/不匹配部分，再真实链接/验证/打包。未执行冷函数也须覆盖；关闭解释/JIT的native profile不得依赖回退到AV。

开发期优化产物满足相同发布合同才能复用；调试/插桩、热重载调用、推测假设或跨模块优化上下文不匹配时可只重编该层，上游语义/CTE/MIR继续命中。推测版本须证明条件或保留合法guard/回退；JIT内存不可直接存盘缝合，须可重定位代码/数据与符号、ABI/runtime、依赖及recipe。

统一管理逻辑查询单元、优化/编译单元、装载单元，不将每个MIR基本块强制作为最小持久单元。跨函数内联允许，但记录body/layout/假设依赖。native发布与portable ABC包分别声明target/runtime需求，仍优先复用平台linker；全局cache不自动等于自研全增量链接器。无cache构建用于一致性/可复现验证，不是每次release默认步骤。

## 2. 实现与参考路线

| 路线 | 核查结果与工程判断 | 定位 |
|---|---|---|
| Cranelift | Rust 库、AOT/JIT；ObjectModule 有 COFF 路径，Windows unwind 有专门约定；实际 target/调用/链接须探针 | v0.6 Rust AC主后端；v0.7 AACE可按显式桥接继续复用 |
| Zig | 自主后端/快速编译/增量更新值得参考；代码生成入口依赖 Air/Zcu/InternPool，抽取为 Auto 库需要适配 | 架构与目标代码生成参考 |
| QBE | 简单 SSA IL、优化/寄存器分配、输出汇编交给系统工具；官网 AMD64 平台列 Linux/macOS | 自主低层 IR 与轻量后端参考；不能直接承诺 Windows 主线适用 |
| MIR（vnmakarov 项目） | 主线为轻量 JIT；README 保障平台不列 Windows，未核实可直接承担我们的 COFF AOT | 未来 JIT/热点编译研究参考；不是 Rust 的 MIR |
| LLVM | 完整代码生成体系；集成与工具链成本需评估 | Cranelift 必需能力不成立时的备选，以及后续优化/目标扩展候选 |

以上不是本项目实测性能排名；不套用 Zig 或 QBE 的他项目基准作 Auto 性能承诺。
Rust 原型先用 Cranelift，版本/API/SDK 由实测锁定；若必须能力失败，记录证据并修订方案。
不默默切为 C/Rust 转译、解释或 JIT 来代替 AOT 验收。
无需现在完整 clone Zig；深读时可使用独立参考目录、固定提交，不引入产品依赖。
Zig 已迁至 Codeberg，旧 GitHub 仓不再镜像；0.15.1 发布说明仅作历史演进案例。

## 3. v0.6：Rust版ACE正确性、兼容成熟度与可持续开发基础

~~~text
Auto source → resolve/typecheck → Typed/Checked HIR
                                      ↓
                       最小 Typed MIR / 语义资源 lowering
                                      ↓
                      Cranelift adapter → CLIF/SSA
                                      ↓
                  Cranelift 支持的优化与机器码生成
                                      ↓
                        object → link → native
~~~

这是未来共享MIR的native路线；既有HIR直达Cranelift成果按原合同保留，接线迁移另立Plan。MIR另经ABC lowering接Rust AV，并由AV支撑CTE；A2X主要消费Typed HIR。详见Design43。

公共 HIR 保留结构化控制流、语言类型、所有权/资源与效果信息，低层 SSA 不反向成为所有后端
的公共数据模型。接入 Cranelift 本身就需处理基本块、分支、循环与 SSA 值；
FunctionBuilder 可维护可变源变量与 SSA 值的对应，不必先自己写完整 SSA 构造算法。

v0.6 要建立下列基础，按后续计划逐项实现：
- HIR 语义不变量和 effect 契约：数值/溢出、trap/错误、短路、调用顺序、内存、借用/移动与释放。
- 明确阶段输入/输出、profile、来源映射、失败分类与产物身份；为将来pass保留合同，不强制本版实现通用优化框架。
- 以最小Typed MIR承载正确性必需的控制流/求值顺序、调用、所有权/清理；后端负责具体ABI/表示合法化。MIR默认有类型，v0.6不要求SSA或通用优化框架。
- 若启用后端优化或实施语义改写，仍须重验溢出/trap/副作用/资源，记录配置与版本、失效分析缓存；不能借分期放松正确性。
- 从 HIR 向 CLIF/平台 ABI 的降级与隔离接口，公共 HIR 不泄漏后端私有结构。

不能仅按“read/write”猜所有 effect；trap、资源释放、外部调用和重载边界也影响合法变换。
常量求值须保持精确宽度/溢出/错误语义；未执行分支中的 trap 不能被提升成无条件编译错误。
禁止通过排序字段重排初始化或命名参数副作用。内联不得破坏公开/可重载入口边界。
复杂优化由已选后端承担其支持的部分，v0.6 不要求自写完整通用优化器。
2026-10-16分期：v0.6不以LLVM运行性能对标或通用优化框架为门槛；开发周转、内存/磁盘有界与增量正确性仍属本版基础，不随优化工作延期。不回溯扩大已交付计划。

### 3.1 首条正式集成管线的交付门（2026-10-16）

语义侧前置：[Design42](../42-semantic-metadata-inference-and-comptime.md)要求元信息/类型推导/实例化与CTE共享查询/身份/store，runtime反射由语义事实投影。[Design43](../43-ac-av-typed-mir-and-abc.md)确定先设计最小Typed MIR和新ABC、实现Rust AV子集，再接真实CTE；checked HIR片段→MIR→ABC→AV→冻结结果返回语义查询，不等待全程序final Checked。完整旧AVM迁移可分期，新AV必要子集不能一起延期。

用户要求增量/cache与新测试从开始进入编译器。现有有限profile/783探针不追溯称为下面全部完成。下一条正式集成的source→HIR→object→link/run，即使仅返回常量/hello-world，也须同时通过：

| 门 | 必需能力与区分性验收 |
|---|---|
| G-01 正确性 | 显式profile、真实verify/native、阶段负例、trap/副作用/清理；禁止VM/转译替代 |
| G-02 增量/cache | 真实统一store与阶段依赖；fresh与incremental语义/诊断等价；同进程/重启/跨工程命中；body/接口/config/target变更分轴失效；计量真实跳过阶段 |
| G-03 生命周期 | 有界热点/索引/暂存、root/lease、自动回收、坏块恢复、不删用户数据；压力有界退让 |
| G-04 AutoDown | AC source/HIR/native adapter及后续syntax adapter；实际source/oracle与产物链、fresh/缓存模式；required缺例/skip/工具错误不得绿 |
| G-05 ABI | 私有实验ABI标清；公开独立模块/互操作前冻结有限AutoABI profile，独立编译双方互调及target/布局/ownership/版本错配拒绝 |
| G-06 周转 | 从首例记录关联端到端trace和各阶段耗时/等待、命中/重算、I/O及内存；代表编辑负载以P95<1s为目标，P50/P99与压力长尾另报；阶段都<1s不能替代总周转，未测/未达保持明确状态 |

文件级失效可作早期增量粒度，但不能每次全量重编后伪称缓存命中；未实现层级保留future，不能写空工件充覆盖。私有hello不要求先完成全部公开模块ABI。

### 3.2 v0.6与后续AJ/发布能力的边界

现在设计共同编译服务、逻辑单元/产物身份、跨层签名与代际、内联/假设依赖、机器码重定位和发布闭包/profile。v0.6先交最小MIR/ABC/AV、AV驱动CTE和实际store/AutoDown；在已承诺native profile的后续有限发布计划中落实缓存复用、补编、真实链接和发布快照。首探针只验证合同/有限产物，不交完整发布器。

完整T1/T2热点调度、推测优化、OSR/deopt和所有平台portable包另行实现，不新增为796/797或首个AV计划的交付门。AJ可先共用Cranelift，不要求自研codegen。阶段实施顺序不能抹去首例cache/资源/端到端P95<1s的要求；发布时延和冷构建单独量化，不保证任何首次release亚秒。

## 4. ACC 通过后端桥接继续使用 Cranelift

Rust AC 可直接调用库；Auto ACC 通过明确桥接继续调用它。
优先评估薄 Rust 库提供的受限 C ABI；初期独立后端进程也可作为实现阶段方案。
接口描述版本、target、输入 profile、能力、错误与产物；不暴露 Rust Vec/String/enum 或私有布局。
模块生产 ABI 与编译器工具内部桥接协议分别定义，后者不能冒充已冻结的生产 Auto ABI。

若跨进程，可以用 Atom/Batom 承载编译数据并校验；不要求每次标量运算/普通 native 调用序列化。
进程内接口使用明确宽度、缓冲区/句柄、所有权与释放契约；支持批量输入，评估边界开销。
具体编码/桥接模式不在本战略冻结，不能让后端把 Auto 名称解析/类型检查重新做一遍。

自举顺序：Rust AC 编译 ACC 第一代 → 第一代编译自身得到第二代 → 重复构建/语料验证。
控制非确定性，比较约定语义、诊断、HIR 与 native 行为；字节固定点若作为门禁，
另明确路径/版本/时间戳等可复现条件，不默认任意构建字节必须相同。
保留依赖库/运行时/桥接的来源与版本清单，自举不等同全部依赖都改写成 Auto。

## 5. v0.7：按收益选择优化，按需引入低层IR

2026-10-16用户提出“LLVM约80%的执行性能、只取约5%的主要优化pass”。采纳少量高收益优化方向；80%是待基准合同冻结/验证的目标，5%没有天然可比的分母，不作精确pass数量保证，不能拿LLVM pass数与egraph规则数换算。

建议按吞吐定义P=几何平均(T_LLVM/T_Auto)，目标P≥0.8，即几何平均耗时比≤1.25；不等于所有程序最多慢20%。每项/分组回退、编译耗时、增量P95、峰值内存与代码尺寸另报。代表基准/权重/单项阈值由计划先冻结；固定同机/target/CPU/工具链/输入与语义。若LLVM基线来自等价C/Rust，须保留Auto溢出、边界、trap、ownership与同等runtime成本，不能借UB/省检查取快；无LLVM adapter时不得宣称同源后端对拍。

先测Cranelift none/speed/speed_and_size，再查Auto lowering的多余分配/复制/调用及信息丢失，只补后端看不到且实测有益的语言级变换。达到目标可保留Cranelift；最小Typed MIR已由AC/AV共同执行语义需求确定，是否引入SSA/更低层IR和优化集仍按收益决定，不以自己造优化器为交付目标。

Cranelift本身有优化器：锁定0.126.2的Context::optimize含控制流处理和egraph入口，并有别名分析/冗余load处理；AC当前native两处opt_level=none。启用speed需正确性对拍及新cache配置键，不能据上游能力宣称本项目达到80%。后端也不能重建已丢失的Auto ownership、公开接口/重载边界和语义依赖，这些仍归Auto。以下是最小MIR之上按必要性扩展的优化/低层路线：

~~~text
Typed/Checked HIR → Typed MIR → 按需低层 IR / SSA
                              ↓
                    Auto 分析与优化 passes
                              ↓
                  ┌───────────┴───────────┐
                  ↓                       ↓
            Cranelift adapter        自主机器码后端
~~~

若决定建设自己的SSA/优化体系，先接Cranelift，用同一HIR/语料隔离验证，再评估自主代码生成；无必要性证据则不启动此分支。
Typed MIR的最小块/值/类型、effect/资源与verifier在v0.6专项Plan定界；phi或block arguments、SSA及Machine IR等进一步选择仍按探针/优化需求决定。ABC有独立执行格式和验证器，不与MIR共享物理指令编号或版本。
v0.7 按性能证据选择内联、公共子表达式、循环等优化，而非先列全套算法作为发行保证。
低层 IR 也可用 Atom/Batom 描述，其 Schema/profile 与公共 HIR 分开版本化。

自主后端先限定 target/profile，与 Cranelift 对拍；可以先采用简单栈槽策略，
复用寄存器分配/对象库/linker，避免第一步重写所有基础设施。
regalloc2 和 object 可作组件候选，但其组合不自动提供指令选择、ABI、unwind 与完整链接。
不承诺 v0.7 完全替换 Cranelift，也不因此替换 v0.7 知识管理主线；
编译器研究按独立计划/容量安排，与应用产品工作分别验收。

## 6. 从实验到主后端的门槛

切换由证据触发，不与“AC 完善”“ACC 自举”或某个版本号机械绑定：
1. 覆盖实际语言 profile、目标、公开 ABI、静态/动态模块与必需运行时路径。
2. 数值/trap、调用/栈/展开、布局/重定位、模块生命周期与错误语料稳定通过。
3. 与 Cranelift 的差异有解释，在编译延迟、增量性、制品体积或独特能力上有实测收益。
4. 维护成本可承担，不拖累 AutoOS/生态主线；有能力矩阵、诊断、回退与发行策略。

可以按 target/profile/构建模式逐步采用；选择必须显式、制品记录后端/版本/配置。
不支持的能力明确报错或在生成前按明确配置选择另一后端，不静默改执行模式。
Cranelift 可长期保留为参考/备用，后端更换不自动改变源语言语义或公开 ABI；
保持兼容需要共同 conformance 验证，不能只写相同 ABI 版本号。

## 7. 与 Plan 741 和后续计划的关系

Plan 741 只验收有界 Atom HIR → 校验 → Windows 对象/可执行闭环。
本战略不增加其 pass 框架、ACC 桥接、自举、自主 SSA 或机器码重写验收。
上述分别形成后续计划；已有计划进度由各自执行/复审维护。
实现与架构决策状态分开；设计确认不等于实现通过。

### 7.1 对 Plan 741 的影响

本次路线确认不改变741的任务、AC或revision，不要求返工或追加完整pass框架、
ACC桥接、自举、自主SSA。741提供有界native基础，后续能力逐步增加。
2026-10-04检查时其frontmatter为executing/revision1/current_step7；
执行记录T-01至T-07已填，T-08独立复审待完成。这是进度记录，不代替复审/合入收据。

### 7.2 v0.6 工作量与计划数量的初步估算

以约定编译器主体的原生自举、Windows首平台及静态/动态模块/热重载为边界，
预计26–36个正式计划，包含741；主体排期可按约30个考虑。
若边界需拆分、返工或集成修复，预留35–45个计划的容量。
这是早期native-only工程估算，不是已经批准的合同数量，也不是用计划数保证工期。2026-10-16新增最小MIR、ABC/AV、CTE依赖后，该估算不覆盖新增工作；须在首个全路径探针后重新估算容量和日历，不宣称原工期不变。

| 工作组 | 独立可验收成果 | 预计计划数 |
|---|---|---:|
| Atom/HIR与pass基础 | 741、所需数据契约、typed HIR、阶段验证、基础变换与诊断 | 3–4 |
| 源码前端与计算子集 | AST→HIR、精确类型映射、名称解析、函数/控制流/运算扩面 | 3–4 |
| 聚合类型、泛型与所有权 | record/enum/Option/Result、方法、泛型实例化、借用/移动/清理 | 4–6 |
| Native运行时与编译器所需库 | 字符串、集合、分配/释放、文件/进程服务、错误处理 | 3–4 |
| ABI、独立模块与热重载 | 布局/调用、静态库/DLL、常驻服务、迁移/在途调用/安全回收 | 5–7 |
| ACC主体迁移与自举 | 后端桥接、Auto前端/类型/HIR/lowering/驱动、代际自编译 | 6–8 |
| 集成与发布收口 | 完整语料、兼容性、构建复现、工具链集成与修复 | 2–3 |
| 合计 | 包含741 | 26–36 |

741收口后，对应剩余约25–35个主体计划；是否合并/拆分由实际独立验收边界决定。
不同组会复用契约和工具，不能把同一共享工作重复计数。

估算不含整个Auto生态全部原生化、全部A2X/AVM迁移、自主机器码后端，
也不重复计算完整Atom/package/UI等其他主线工程。所需共享Atom/HIR基础列在第一组；
完整Batom/通用Schema引擎若另线实施，应明确依赖与归属。
WSL/Linux若成为新增native发布门禁，或自举主体扩至更大语言范围，重新估算相应增量。

主要不确定性：泛型/所有权、热重载生命周期、ACC主体真实能力清单。
不能用741的小闭环速度推算全部计划；ABI或并发资源计划可能比算术扩面显著更长。
约30个计划分布在约三个月只是每周2–3个的吞吐参考，不是排期保证；
独立复审、真实平台与修复必须保留时间。

### 7.3 推进顺序与紧接着的候选工作包

~~~text
741独立复审/收口
  → ACC主体与能力清单、HIR阶段契约
  → Auto源码计算子集跑通
  → 类型/运行时扩面，同时推进ABI/模块
  → ACC按模块迁移并接后端桥
  → 原生自举、代际验证与发布收口
~~~

以下保留2026-10-04的候选拆分分析；已创建计划及现行编号以§7.5为准，其余保留工作包层级：

| 候选标签 | 范围与可评审成果 | 前置与安排 |
|---|---|---|
| NEXT-A | ACC主体/语言能力清单与HIR阶段边界；明确保留依赖、桥接职责和真实验收语料，形成后续实施合同 | 读741复审/Spec成果及现有Auto编译器；清单可离线调研，接入决策待完整v0.5核对 |
| NEXT-B | Auto源码→Checked HIR→native的最小adapter；明确int/i32兼容，编译函数/循环示例并验证结果 | 依赖741稳定API与清单；旧前端接线须基线恢复，必要时拆为adapter和计算扩面两个计划 |
| NEXT-C | 生产ABI的标量/聚合间接传参与返回、嵌套调用和C双向互调探针；输出明确布局/错误/工具链证据 | 可在741基础上与前端调研并行；遵守Native ABI RFC，不冒充完整模块/热重载完成 |

NEXT-A是否单列设计合同或并入最先的实现计划，按成果大小决定；
NEXT-B/C不要求都在下一编号连续出现，各线仍需统一独占取号。
源码计算示例及Auto lexer/parser代表程序开始真正native编译后，重校26–36的估算。
本轮不自动创建、批准或执行这些候选。

### 7.4 已创建后续计划的检查快照（2026-10-04）

检查范围：auto-lang主检出与四处已登记worktree的docs/plans（含archive），
主检出顶层Plan frontmatter，以及本地编号分支。
观察：仅找到PLAN-741；没有742及之后的计划文件或对应本地plan编号分支；
主检出.next-id=742。战略中的后续工作包不是已获编号、已评审或已开工的计划。
未fetch不可访问主机器，不据本快照推断其尚未push的新增工作。
后续正式计划创建/归档后更新队列，执行状态以其合同和复审证据为准。

### 7.5 现行AC/ACC队列与编号映射（2026-10-13）

默认分支合流保留发布线767–769的原义，开发线标量计划767改为772、AutoDown试点768改为773、共同契约769改为774。**文本770与record771没有改号**。旧附件/报告/已存在的验收脚本可保留原文件名；活跃计划入口、执行指令和依赖使用现行编号。

| 现行计划 | 成果与状态 | 下一步/依赖 |
|---|---|---|
| [741](../../plans/archive/741-ac-hir-native-core.md) | Atom HIR→Checked→Windows native首个闭环，archived | 已交付基础 |
| [743](../../plans/archive/743-acc-bootstrap-hir-contract.md) | ACC主体/能力清单与HIR阶段合同，archived | 已交付盘点，不等于自举实现 |
| [745](../../plans/archive/745-ac-source-checked-hir-native.md) | 独立Auto源码计算入口→native，archived | 已交付有限源码profile |
| [772（开发线原767）](../../plans/archive/772-ac-scalar-ops-short-circuit.md) | 精确标量运算、布尔短路、条件for及可信边界，archived 9/9 | 已交付，scalar Spec已沉淀 |
| [774（开发线原769）](../../plans/archive/774-ac-data-resource-contract.md) | 数据/资源共同契约与真实native可行性探针，drafting 0/8 | 先执行；决定/预算/oracle冻结并实际验证，独立复审合入后解锁770/771 |
| [770](../../plans/archive/770-ac-text-char-native.md) | str/Str/char首批原生编译，drafting 0/9 | 依赖774，可与771并行实施 |
| [771](../../plans/archive/771-ac-record-method-native.md) | 标量record、字段与首批方法，drafting 0/9 | 依赖774，可与770并行实施；本轮无文本字段 |

```mermaid
flowchart LR
    Scalar[772 标量已交付] --> Contract[774 共同契约与探针]
    Contract --> Text[770 文本与字符]
    Contract --> Record[771 标量record与方法]
    Text --> Mix[后续文本聚合组合]
    Record --> Mix
    Mix --> Data[后续List / enum / 模块等能力]
    Data --> Compiler[真实ACC组件与后端桥]
    Compiler --> Bootstrap[Gen1 / Gen2 / Gen3原生自举]
```

770/771的独立profile各自通过不代表文本record组合或完整lexer已实现。两树共享AST/typecheck/HIR/verify/native等热点；774冻结接口和合入安排，第二合入者同步新基线、重跑两条路线相关门禁并重新独立复审。无需先重构旧编译器全仓。

[773（开发线原768）](../../plans/archive/773-autodown-testcase-protocol-pilot.md)是独立AutoDown测试协议试点，当前execution_done；它不是本批AC扩面前置，不能据其状态放行774/770/771。计划状态是本次检查快照，后续以各自frontmatter/执行/复审收据为准；本次引用维护没有启动任何实施或关闭自举目标。

### 7.6 增量、插件与通用语法产品约束（2026-10-15）

用户要求增量编译、热重载和插件系统从AC/ACC设计起点融入，并将解析能力复用为以Atom/Batom配置语法的通用parser-generator产品，服务IDE/LSP与第三方语言symbol/signature扫描。对应 [Design 39](../39-ac-incremental-syntax-and-language-platform.md) 是新增目标态约束，不表示这些功能已经实现。

因此在真实ACC lexer/parser迁移前，先建立源码snapshot/edit/位置、容错CST、grammar/scanner代际、语义查询依赖和模块/接口身份合同；语法解析、Auto语义/Checked HIR、runtime重载分别验收。旧token/扫描/Pratt核心仍按743适配复用，输出与会话接口需按新边界设计。第三方源码结构扫描只产生候选元信息，需semantic/ABI验证后才可用于可执行互操作。

近期建议先规划“前端共享契约与全路径可行性探针”，enum/is与List语言扩面可共同规划；通用grammar引擎与ABI/热重载是独立实施线，不把整个产品塞入enum计划，也不等待完整生成器才继续自举。782按原合同复审修复，新约束不追加入其验收。§7.5保留历史快照，现状以各Plan/Spec与fresh收据为准。

同日进一步确认最终目标为亚秒级开发周转与低内存：AST/CST及语义/HIR/对象码统一进入跨工程全局cache，内存仅保存有界热点工作集，磁盘产物按块/查询读取；不得要求加载大型仓库全部依赖AST后才能服务。下一架构探针须同时验证分页语法树、已有AutoCache集成、共享session/多工程去重、配置隔离、崩溃/GC与端到端延迟；仅共享磁盘文件或仅报告parser耗时不足以验收。详见Design39 §5.1–5.5；具体实现与性能尚未交付。

[Design40](../40-compiler-artifacts-and-cache-lifecycle.md)进一步统一各阶段产物/Atom-Batom Schema/引用/存放和自动清理：required-data闭包、失效指纹、可重建边及来源分离；byte预算、lease、软保留、磁盘水位、物理回收/compaction与崩溃恢复形成一套生命周期。缓存健康不依赖用户逐工程clean，也不允许借GC删除源码/交付物/在途runtime代际。下一个架构合同计划以Design39+40共同取材，暂未分配编号。

### 7.7 C生态与未来Rust原生互操作（2026-10-15）

依据[A2C/ABI讨论](https://chatgpt.com/share/6ad0577b-5c00-83ea-9684-e64956b98cb1)，以 [Design41](../41-interface-metadata-and-foreign-abi.md)统一：AutoABI是独立原生模块契约，C/system ABI是具体目标外部接口，FFI含类型/所有权/错误/回调adapter，Module Runtime管理lease/状态/热重载。编译器内部Cranelift桥仍有独立协议，不能冒充公开模块ABI；普通native调用不强制Value/JSON/Atom序列化。

C headers/SDK/GIR等元信息与Rust构建的C兼容shim汇入共享Interface Metadata/Foreign IR；Native、VM、A2C/A2R各自lowering与验收。C互操作不要求Auto先转译为C，A2C仍服务MCU、特殊工具链/现有C工程及bootstrap；Rust原生私有ABI/fork-rustc不作为新主线前置，跨AutoABI域无显式已验证adapter则拒绝。

下一架构合同与Design39/40一起预留import provider、target layout/ABI及foreign元信息产物、缓存key/SDK依赖、加载代码/回调/foreign deleter的代际pin。可先用Auto双向C互调、SQLite外部宿主插件和小型Rust shim探针分阶段验证，未启动全部平台/库的工程；新能力不扩大782验收。use.c自动符号导入与package/header resolver是目标体验，语法与Schema另行冻结，既有auto-bindgen手工manifest面不冒称任意头文件摄入。

### 7.8 当前优先级与在途计划边界（2026-10-16）

类型/泛型/comptime/反射进入语言实现前，按Design42/43安排语义身份与推导合同、Typed MIR与ABC/AV全路径探针、AV驱动CTE/缓存回路、runtime描述保留等有界工作包。AutoDown接线与ABI探针仍优先可推进；不能等完整AC后再补类型/元信息架构，也不向787追加这些实现。

§7.4/7.5保留历史快照，不能据其旧drafting状态推断当前交付。新的优先安排：

1. 787继续当前语法/分页合同；其已executing且有在途改动，不临时添加ABI/native/完整TestCase框架。
2. 紧邻优先规划Design35 B阶段AutoDown AC source/HIR/native adapter，先引用现有fixtures/oracles，不大搬语料；syntax/增量观测以明确adapter能力表接入，保留Rust内省与既有CLI门禁。
3. 同时准备最小AutoABI合同和Auto↔Auto/C双向探针，公开互操作前冻结有限profile；不等完整AC，也不提前承诺完整C importer/Rust生态。
4. 首条新语法→真实原生集成按§3.1 G-01–05交付，从第一例消费统一cache/增量与新测试。之后enum/is、List/use等扩面逐项继承这些合同。

2026-10-16后续优先级更新：796继续共同语义/CT模型合同，797继续有限native ABI/C双向探针，不追加VM/MIR验收。下一步先规划“Typed MIR与ABC/AV执行合同及首个全路径探针”，纳入AC/AV/AJ共同服务、单元身份/profile/代际/可重定位产物合同和有限cache对照，但不实现完整AJ/发布器。再规划“AV驱动CTE查询与冻结缓存闭环”；native共享MIR接线与有限增量发布另立有界计划，完整分层AJ随后实施。ABC/AV设计可与796/797并行研究，最终真实绑定等其合入，不能复制权威模型。影响和依赖见[评估收据](../../plans/attachments/ac-foundations-20261016/ac-av-mir-impact.md)。

以上是工作包优先级，不分配新号、不启动代理或实施；787影响记录见docs/plans/attachments/787/strategy-impact-20261016.md，不替换其r1执行合同。

### 7.9 ACE慢测后续归属（2026-10-30，888 r3）

用户已授权将13项现有长原子测试归入ACE专属慢测组并交ACE路线处理：syntax 1、semantics 1、AV 4、native 3、CTE 4。它们来自787/796/798/817/818/828，不全是最近新增；保留原13叶及独立选择、启用、oracle、样本、真实规模与资源负控，不产生重复义务，不因归组自动免除既有门禁。随后用户授权正式立[893](../../plans/archive/893-ace-test-framework-and-slow-case-attribution.md)，drafting r2、0/12；按[共同测试设计](../testing/ace-test-framework-and-cost-attribution.md)复用888模块/执行/计量，交付ACE接入策略、成本关联与13例当前归因，不新造runner/裁判/meter。888负责归组/模块清单与证据交接；产品算法/协议修复据证据后继拆包，不预占后继号；暂关须用户逐项明确决定。清单及日志hash见[后续处理登记](../../plans/reports/ace-slow-tests-followup-2026-10-30.md)。本轮只规划。

历史单例85–526秒包括规模构建、512修订/GC或多轮正式周转/反控，不能据此推断单次编辑超门，也不是当前候选验收数据。888 r3现按模块单轮测墙钟/CPU/内存，取消全目录三轮/P90和新A/B/C矩阵前置；ACE能力原有正式样本、P90≤2000ms及资源/结构门保持。888其余模块采样结束前只读分析/写规划，之后逐文件交接S/N/H与测量窗口再实施，不改共享worker/helper/注册或启动重型验证干扰现场。

### 7.10 共享文本源码与资源收口（2026-10-30）

已正式立[894](../../plans/archive/894-shared-text-source-and-resource-closure.md)，drafting r1、0/13；[专题设计](shared-text-source-resource-contract.md)负责原T03=23/T05=42的共享source/HIR/MIR→AV/native、局部Str替换与真实函数级增量。当前已观察T03受限域明确拒绝、T05已有StrCmp分支，后者先跑原源再判需修或仅补证；本轮无执行结论。复用现有recipe/worker，不新增dispatcher/wire，版本变化的三消费者镜像及stale最小同步不能省。S与私有测试可独立，N必要owned-source桥和H微窗按真实owner交接，不等888/891/892整包；不改891 typed ABI，不扩892标量，AX-B以后消费正式资源事实。原source/expected/分母保持，旧T03限制断言迁移有批准扩面与历史依据，不冒M1全绿。本轮仅规划。

## 8. 官方参考与核查范围

2026-10-16补查：[Cranelift优化设置](https://docs.wasmtime.dev/api/src/cranelift_codegen_meta/shared/settings.rs.html)、[Context源码](https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/codegen/src/context.rs)，并只读本机Cargo registry的cranelift-codegen-0.126.2/src/context.rs确认锁定版本入口。未运行优化/LLVM基准，上游当前文档不是锁定版本性能收据。

2026-10-04 查阅官方公开文档/源码，尚未据此执行本项目性能或 ABI 对拍：
- [Cranelift 定位与平台](https://cranelift.dev/)
- [ObjectModule](https://docs.rs/cranelift-object/latest/cranelift_object/)、
  [COFF/unwind 源码](https://docs.rs/cranelift-object/latest/src/cranelift_object/backend.rs.html)
- [FunctionBuilder 的 SSA 构造接口](https://docs.rs/cranelift-frontend/latest/cranelift_frontend/struct.FunctionBuilder.html)
- [Zig 官方迁移说明](https://ziglang.org/news/migrating-from-github-to-codeberg/)、
  [代码生成入口](https://codeberg.org/ziglang/zig/src/branch/master/src/codegen.zig)、
  [0.15.1 历史发布说明](https://ziglang.org/download/0.15.1/release-notes.html)
- [QBE](https://c9x.me/compile/)、[QBE IL](https://c9x.me/compile/doc/il.html)
- [MIR](https://github.com/vnmakarov/mir)
- [LLVM 代码生成](https://llvm.org/docs/CodeGenerator.html)
- [regalloc2](https://docs.rs/regalloc2/latest/regalloc2/)、
  [object 写入接口](https://docs.rs/object/latest/object/write/)
