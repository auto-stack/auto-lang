### Plan 848 r3（2026-10-25，原生backlog验证器修复独立pass）

M848-NATIVE-BACKLOG CLOSED，绑定dd7076a7f/r3。合法clock自动暂停后探针误把未接受刺激用于动作/隐藏断言；修复仅原生验证状态机，实际OS Resume/恢复目标、未接受V重投/已接受不重放、原动作数量与phase deadline保持。独立display-gap文本/3D held/wrong-focus及强制重投已接受V变异均具区分性。生产clock/Host/Auto/128/32credits/CPU未改。[独立R4](attachments/848/review-r4-final/review.md)，reviewed9/10，当前冻结merge统一门仍pending，未合入。下方OPEN仅历史。

### Plan 848 r3（2026-10-25，merge原生门阻塞，未合入）

M848-NATIVE-BACKLOG OPEN（合入门阻塞，不是获准延期）：6bc5abc60/r3的producer/fresh consumer完整216各通过；原生用户脚本先后在Patrol phase5和搬迁例phase17记录auto-Pause tick backlog>4，随后输入/隐藏断言失败。单独原样Patrol通过；外部诱因未证实，不宣称848产品回归或全局原生绿。原时钟/credits/暂停及验收不改，reviewed9/10，T10剩余矩阵与收尾pending；未合入/沉淀/归档/清树。[收据及原始归因](attachments/848/merge-r1/receipt.md)。首次consumer链接栈配置遗漏已按既有32MiB配置定向及完整重放关闭，历史失败保留。

### Plan 848 r3（2026-10-25，独立final pass，合入门pending）

F-848-R1-01 CLOSED，绑定6bc5abc6080b2261a43f55a54f127cd779239edb/r3。用户授权的独立子代理/root/review_848_r3复审通过，18个定向测试含60预算子例、6Host及新选128节点混合斜向反例。原128节点/32credits/CPU合同保持，无新阻塞。reviewed9/10，T10统一合入门pending，未合入。[独立报告](attachments/848/review-r3-final/review.md)。下方待确认/OPEN仅为历史。

### Plan 848 r3（2026-10-25，work修复/内部复核通过，独立final pending）

F-848-R1-01 WORK-FIXED / 待独立确认：冻结6bc5abc6080b2261a43f55a54f127cd779239edb，先验证方案再修订r3实施；原R1输入/Host成功，60组128节点、32credits控制与6组Host oracle（含三轴正负/非等比例斜向、X/Y/Z标量写入）通过。shared按脏轴推导+受信紧凑生成区，原CPU/SDK/数值/单槽上限不变，新增4dirty标量后总60B；producer33/consumer12定向检查、4原生+2重开通过。无新增work gap，内部复核pass，当前会话含实现上下文，不冒称最终独立pass。Plan execution_done8/10，独立子代理授权待用户答复，完整统一门仍merge-owned pending，不合入。[工作收据](attachments/848/work-r4/readiness.json.txt)、[内部复核](attachments/848/review-r2-internal/review.md)。下方OPEN是R1历史。

### Plan 848（2026-10-25，R1 needs_fix：128节点预算覆盖缺口）

F-848-R1-01 P1 OPEN（合同内阻塞，非可延期债务）：r2/cef25b5c4，actual Session与正式Host的1group+127真实Cylinder、双group写/2cast+10getter+6AABB恰32credits，在晚Z轴miss或候选ordinal逐个更近时均触发100000累计步上限，Host Faulted/tick0；等深对照正常tick1。nearest剪枝令原均匀box测试通过，但不能证明全部既定128支持。T02/T05/T08重开，executing5/10；保留未变几何/版本/项目/资源证据，work同树修计算路径与永久反例，再受影响复审，完整回归仍merge-owned pending。不得以增预算/减节点/仅留易剪枝fixture关闭。[报告/重放/关闭范围](attachments/848/review-r1/review.md)。未合入或改产品。

### Plan 846（2026-10-25，merge:r1 根性能门阻断，未合入）

## D-846-INCR-PERF — AutoGraphics增量编译性能暂缓（用户授权，2026-10-25）

状态：后续性能议题，当前AutoGraphics交付非阻断。PLAN-846:r3用户明确只验增量编译功能正确性，功能完成、尤其整合真正ACE引擎后再定义性能优化验收。原签名重建P95 1249.3752/1107.9436ms与原1000ms门的真实失败保留于[846 merge:r3](attachments/846/merge-r3/receipt.md)，不宣称性能已达标。功能失效传播、未变单元缓存复用、实际链接/运行、数值/支持/回收及运行时安全预算仍必须通过。命名观测策略及规范增量见[846 r3](attachments/846/revision-r3/spec-delta.md)。后续应在真实ACE整合上下文另定性能基线，不沿用本次豁免当作长期性能承诺。


F-846-M1-ROOT-MN-P95 OPEN：实际SDK根接收44480完整t为879pass/1fail、4488未跑；MN signature-rebuild 30样本P95=1032.8293ms≥1000ms，归因待查，尚未证明846回归或预存。tv163及producer/consumer各178通过；剩余native完整矩阵pending。Plan保持reviewed10/11，WT恢复bf22并保留映射分支/SDK tag；未落地、沉淀、归档或清树。[阻断/恢复收据](attachments/846/merge-r1/receipt.md)。

### Plan 846（2026-10-25，R2 final pass；CLI finding CLOSED，merge门pending）

F-846-R1-01 CLOSED，绑定bf22db346/r2：缺值presence guard+永久CLI3测试（10真实子进程），R1原3命令及独立6兄弟/合法对照共9次复验通过；actual Host版本、345来源/consumer闭包齐。其余R1未变AC/Spec证据保留，reviewed10/11。完整root t+tv、driver178与原AGP/native分母/新空间查询完整矩阵仍merge-owned pending，尚未合入/交付。[R2受影响pass](attachments/846/review-r2/review.md)。下方R1仅为历史。

### Plan 846（2026-10-25，R1 final needs_fix：CLI版本选项缺值）

F-846-R1-01 P2 OPEN（合同内，非可选延期）：末尾`--behavior-api`缺值被当成未提供选项，实际`--inspect <合法scene> --behavior-api`返回0/closure_verified:true，违背§5.5孤立API选项usage error；有v2无--behavior正确2，无选项合法0。绑定655c366a3/r2，重开T03/T06/T08、executing6/11；SDK/空间查询/资源/已闭验收保留。仅修CLI验证与永久对照，刷新受影响来源再复审；完整门merge-owned pending。[复现与验收映射](attachments/846/review-r1/review.md)。

### Plan 839（2026-10-24，已交付archived10/10；P2 CLOSED）

当前独立final R2 pass，完整冻结59入口及delivery身份消费门通过，autographics-dev快进`a239653365a0c3451c85aa9921189fcf011bd8cf`；主分支规范/台账77与归档提交`c14fc97fe8c83a571d6cf7a249cba74a76e218c7`已核。fresh guard后唯一实施树/开发分支已移除，独立消费复现副本保留；batch not-due、搬迁重开smoke通过。无本Plan开放finding/gap/未决决策或待完成项，生产I2/I3维持原非目标。首轮旧Squash预算失败与同冻结完整重跑通过原样保留。[合入pass收据](attachments/839/merge-r1/receipt.md)。下方R1仅为历史。

### Plan 831（2026-10-25，R7 final pass；历史finding全闭，merge门pending）

[R7 pass与范围/路由](attachments/831/review-r7/review.md)绑定lang2a3e2c4d6/UI4250885/downc39ac7f9、Plan r3。R4/R5真实exit1路径控制和R6真实缺wrapper数字路径控制均符合同（共6分类对照），原5矩阵控制全过；F831-09-03 A CLOSED、全部work-owned finding已闭。7材料/4原始终源收据hash与canonical输入匹配；未变兼容/777/测试选择面/净迁移/其他验收证据按范围复用。r3经批准的jet/ark缺环境构建后续门控承接保持（不冒称构建成功）；V06e保持已闭。当前reviewed10/12，仍须T11 merge-owned裸cargo t与必要触面/UI适用门通过、实际落地/读回/ui-release.md，T12 Specs-ledger/索引/归档/guard清理闭环后才算交付；U831未成立、832不放行。未改产品实现/跑整档/合入/清树；当前审查会话上下文限制已记。以下R1..R6记录为历史，以本R7当前结论为准。

### Plan 831（2026-10-25，R6缺环境误红修复回归needs_fix）

[R6真实缺席入口/对照](attachments/831/review-r6/review.md)绑定langc79d34311/UI4250885/downc39ac7f9。R4/R5真实旧假绿均已闭、原5控制全符合同，V06e与其余已闭面保留。仅F831-09-03 A PARTIAL：新re.search仍扫整段error，真实缺wrapper/no build executed的合法r3延期因项目路径exited 1-r6-*被误当实际exit1而RED；同错误只去掉无关路径数字恢复env-gated。当前是允许延期被误红的修复回归，不再称旧真实失败误绿。只修producer原因结构与路径边界、保留真实失败红/获准缺环境绿，执行受影响反例和对照即可；T09重开/executing8/12，revision3/r3延期不变，其他族不重跑。未改产品/跑整档/合入/清树，merge-owned门pending、832等待U831。

### Plan 831（2026-10-25，R5仍仅F03-A分类不变量needs_fix）

[R5真实兄弟输入/对照](attachments/831/review-r5/review.md)绑定lang538e3b8fe/UI4250885/downc39ac7f9。R4真实not found旧例与原5控制均符合同，V06e及其他已闭面保留。仅F831-09-03 A PARTIAL/OPEN：所谓producer具名模板仍以sig in整段error搜索，合法目录Hvigor wrapper not found-r5-*使真实交付tool的已执行wrapper exit1失败report又被env-gated，去掉同错误中的无关路径字串即RED；是旧例已闭而分类不变量未闭，不新增验收/规模要求。下一work仅修原因/附带文本边界、未知错误红，保留r3环境延期；executing8/12、revision3不变。未改产品/跑整档/合入/清树，merge-owned门pending、832等待U831。

### Plan 831（2026-10-25，R4窄残项needs_fix，r3环境延期保留）

[R4真实入口与对照](attachments/831/review-r4/review.md)绑定lang3064b3465/UI4250885/downc39ac7f9。原R3五构建控制全过，endpoint-timeout正确红且独立TCP仍在，F831-09-03 B CLOSED；仅A分类PARTIAL/OPEN：BUILD_ENV_ABSENCE_MARKERS的not found对整个错误文本匹配，真实交付tool执行现有Ark wrapper（写marker后exit1），合法工程路径含not found时实际失败report却被V06c归为env-gated/ok=True；同失败仅移除错误文本里的无关目录字串即红。已核实际producer错误格式，模拟编译消息只作诊断，不将未确认生产路径当缺陷。仅修V06c环境归因不变量及该反例/正控/代表兄弟；r3缺环境延期继续有效、V06e及无关已闭族不重跑。T09重开/executing8/12、revision3不变；未改实现/跑整档/合入/清树，merge-owned门pending、832等待U831。

### Plan 831（2026-10-24，复审R3 needs_fix，Plan r3有界修订保留）

[R3受影响复验与控制](attachments/831/review-r3/review.md)绑定lang2d47e236e/UI4250885/downc39ac7f9。接受用户本轮确认的r3：jet/ark缺环境真实构建转后续环境门控，**该延期本身不再是831阻塞**；六profile生成/required三构建/vm运行及工具链在场真失败红仍保留。R2 progress-only反例现已拒绝，但F831-09-03仍P1 PARTIAL/OPEN：①V06c只聚合required，jet/ark真实编译错被忽略，合法全完成/获准缺环境绿、required/vue错红对照均过，jet/Kotlin与ark/ArkTS错仍假绿；②V06e把HTTP超时当关闭，同一真实listener取消前HTTP200、取消后仍TCP connect成功但HTTP无应答，case仍绿。仅修两个新判据触面，复用未变历史证据，T09重开/executing8/12、revision3不变；merge-owned门pending、未改实现/跑整档/合入/清树，832等待U831。

**经用户r3批准的后续承接项（非831当前阻塞）**：jet/ark在具备gradle/java/Android SDK与Hvigor/DevEco工具链的环境门控渠道完成真实构建验收。来源=Plan831 r3 §4/AC03/V03及handoff/freeze；当前缺环境实测记录保留，不声称构建成功、不退休后端。后续环境门控执行方负责补齐工具链/版本/输入与实测收据；尚无新Plan编号或远端CI入口，不虚构已创建任务/已运行渠道。工具链在场后的真实失败不得按本延期豁免。

### Plan 831（2026-10-24，T-10 R2 repair re-review needs_fix）

[R2定向复验与可重放控制](attachments/831/review-r2/review.md)绑定lang fe8f3090a/UI4250885/downc39ac7f9。F831-09-01 blueprint独立oracle5/5、F831-09-05 tracked dirty拒绝、F831-R1-01终态source/SD/hash刷新已闭，R1三个旧假绿反例均拒绝；原绿证据保留。仅F831-09-03仍P1 PARTIAL/OPEN：jet/ark约定环境面缺证不能转work complete；V06e把任意event当实际运行，合法Progress(resolve backend,1/3)+cancelled完整RunReport+真实descriptor但从未启动VM的真实framed载体仍判绿。是旧例闭合、完整运行接受不变量未闭，非新功能要求；下一work先交覆盖诊断与oracle调整，再做受影响正反控和真实链。T09重开、executing8/12；未改产品/跑整档/合入/清树，merge-owned门pending、832继续等待U831。当前会话独立性限制已记录。

### Plan 831（2026-10-24，T-10 R1 needs_fix）

[完整R1/原样验收器反例/AC-SD覆盖](attachments/831/review-r1/review.md)绑定lang1635835d3/UI464569d/downc39ac7f9。原示例扫描1/1、兼容实编24/24、777真实UI362文件strict0v5gf零警告通过；F02与主要布线修复保留。但F831-09-03(P1)REOPEN：jet/ark构建实际红且部分运行未验，V06e空闲不服务工具也判绿、V06d从未安装立即返回也判绿，不能证明VM/真实安装中取消与寿命；F831-09-05(P2)REOPEN：真Git tracked源码修改同HEAD仍admitted；F831-09-01(P2)原样例闭合但blueprint fallback精确oracle被contains字串弱化；F831-R1-01(P2)OPEN：最终source-audit/SD仍旧状态，三canonical Spec已有840更新需协调。均属合同内未闭，记账不豁免。T09重开、executing8/12，先同组work定向修复后更新终稿再审；merge-owned统一门仍pending，未改实现/跑整档/合入/清树，832仍等待U831。当前会话复审限制已记，不冒称新独立上下文pass。

### Plan 831（2026-10-24，T-09 work核查 needs_fix，非终审）

[最终来源核查及重放](attachments/831/handoff.md)绑定lang eaf1027e5 / UI 461c609 / down c39ac7f9。F831-09-01..06均OPEN、属于本Plan合同内阻塞，记账不构成延期或豁免：01示例/blueprint/宿主CLI根未迁（定向app_registry测试选1败1，实际0/预期≥34）；02兼容实编探针仍无条件pending；03全profile真实构建运行/安装失败恢复与寿命证据未齐；04UI t --lib漏integration、原档过滤/跨仓编排/777实际UI文本根未承接；05历史收据未完整绑定最终三仓来源；06目标无根LICENSE及编译告警需归因收口。841原文件全部迁入、零丢失的源码对账保留，不等于测试与发行闭环。T-09未完成、executing8/12；先同组work修复并定向验证再冻结，T-10独立终审及merge-owned统一门仍pending；832继续等待U831。未改实现/跑整档/合入/清树。

### Plan 839（2026-10-24，R1核验needs_fix历史；Starting取消一P2）

受审`e58828ca64b7e26ce984faf850fb3b1d0887517f`/r1；[完整报告及持久反例](attachments/839/review-r1/review.md)。F-839-R1-01/P2：Starting后失焦/resize-DPI/离窗/遮挡请求被Play::pause状态过滤吞掉，初始化后仍Running且errorNone，后续真实Auto tick均到2；同锁独立客户端3控制过、四边沿同一case失败。Running失焦、Starting Stop/真实owner释放、晚query失败整笔拒绝对照通过；35永久定向/native7/source及来源四反控/owner保留与重传101+101恢复0/check/fmt/diff过，无新增warnings。原合同AC04/05/06内必修，T04/06/07/08重开，executing4/10，T01/02/03/05成果保留；同树修齐后刷新source/SD/handoff并提交。当前为同实施会话internal核验，独立上下文final review仍pending，未据此授最终pass；原完整merge-owned冻结矩阵/实际落地沉淀归档guard仍pending。未修改实现/合入/清树。

### Plan 838（2026-10-24，已交付archived10/10；四finding CLOSED）

当前：独立R2 pass；完整冻结门/最终delivery身份门全过，autographics-dev已快进`37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`，主分支规范/台账75与归档提交`3cad6f34cd16ffce482c34737044102bba665a23`已核。fresh guard clean后实现树/分支已移除，3份独立consumer复现副本保留；batch not-due，无生产I2/I3冒认。无本Plan开放finding/gap/待完成项。[merge pass收据](attachments/838/merge-r1/receipt.md)。以下R1是历史记录：

受审 `8494e06d4056e7f4e0f6dd1925e128bb93a83ea6`、r1；[完整独立报告/可重放反例/冻结Spec](attachments/838/review-r1/review.md)。F-838-R1-01(P1)：Save As保留blob检查在路径规范化前且区分大小写，.GLB/ASSETS/含..三案均覆盖原GLB并Confirmed，原场景无法重开；小写精确路径拒绝正控保持。02(P1)：POSITION/NORMAL accessor语义类型未准入，NORMAL U8经生产Host导入在默认dev的gltf typed reader panic，POSITION U32被按f32 bits误读仍发布。03(P2)：mesh.weights morph声明被忽略；04(P2)：node matrix+TRS互斥声明被接收并静默丢弃TRS；二者实际Host均1asset/1object/1history。资产14/旧32/completion3/AGP1/source正反/owner20轮101+101恢复0均过；native首轮OS点击未入拖拽不冒认透明负控，第二轮完整7案正确oracle通过；check/fmt/diff过，SDK继承146warnings新增0。原合同内必修，不因记债豁免或删Spec降约；当前executing4/10，T03/06/07/08重开，T01/02/04/05不受影响成果保留。同树work一次修齐并补兄弟路径永久正反控，新提交再复审；最终pass后merge-owned完整冻结门/实际autographics-dev落地/规范ledger/归档guard/到期检查仍pending。未改实现/提交/整档/合入/清树，HEAD不变且实施树clean。

### Plan 816（2026-10-24，整体R10代码finding全闭；T08终稿待收口）

受审6d15d25a4a317bb028b160fa74be95ee90d9c788/r2；[完整R10/代码闭合与最终readiness边界](attachments/816/review-overall-r10/review.md)。F-816-B-R9-01/P1独立CLOSED：3原反例绿、真实cancel心跳冻结/组已空、组内handler42且撤回信号修复负控7真红，DirectChildOnly/无关进程保护通过；T02恢复。R8-01/04 reader/07安装已验范围保持，无新增实现finding。9原生定向/独立consumer5/实际服务尾部264ms关闭、Windows execution11+23/client17/CLI7与有效负控/777/fmt/构建/闭包通过。整体最终outcome仍needs_fix仅因既有T08当前身份/交接/验收矩阵与SD终稿未完成，非新增代码修复轮；executing6/14，T08收口后审终稿并按输入等价复用代码证据。最终pass后merge-owned门仍pending，未整档/合入/清树。以下R9的P1 OPEN为历史，当前已关闭。

### Plan 816（2026-10-24，整体R9 needs_fix；R8-01 CLOSED，Unix进程组P1必修）

受审04bb00a460d6c11f375ce9ccb1a95f900067c672/r2；[完整R9/原样Linux测试和真实组信号对照](attachments/816/review-overall-r9/review.md)。R8-01握手/失败清理CLOSED，原04 reader/超时和07安装范围保持；新增F-816-B-R9-01/P1：Unix kill_group与signal_group传正PID，只命中root，三现有树测试心跳仍进且owned后代S态存活。实际cancel后心跳0→300，外部仅向确认owned组发组信号后300→300，根因确定。不是本次diff引入，但为本PlanAC02/T02明确承诺，不能按预存债或平台门控豁免；T02重开，executing5/14，历史A/T12/T14发布事实保留。Windows7/fixture正反/777/fmt/构建/闭包过，不覆盖该Unix生产缺陷。需同树修组终止及SIGINT兄弟方向、永久正反控和实际consumer复验；T08终稿与最终pass后merge-owned门仍pending。以下R8为历史；未实施/整档/合入/清树。

### Plan 816（2026-10-24，整体R8 needs_fix；原04 CLOSED，新增fixture失败清理一P2）

受审abc911befad74bd85234946517f077b57caa1683/r2；[完整R8与可重放证据](attachments/816/review-overall-r8/review.md)。原04超时323ms/reader真停已闭，07保持CLOSED；新增F-816-B-R8-01/P2：永久测试assert失败不kill/wait receiver，Unix新fd-receiver在accept/recvmsg没有握手截止，原样根转交前失败后receiver仍活。真实两正控/实际1ms突变两红已独立复核，原工作NotFound双红排除；无人连接/静默连接实测阻塞。需work补所有失败出口与准备期真实截止/清理，不靠外层探针kill补绿。Windows6/原Unix正控/777/fmt/构建保持；executing6/14，T08终稿及最终pass后merge-owned门仍pending。以下R7为历史，当前以R8范围为准；未实施/整档/合入/清树。

### Plan 816（2026-10-24，整体B-R7 needs_fix；Unix保活已闭，04实际超时真停仍一P2，07保持CLOSED）

受审 `8c69caef752caf0f583aab5f5e0e4670eed7cf00`、r2；[完整R7/原样新driver与原生Linux超时反例](attachments/816/review-overall-r7/review.md)。R6 Unix字节/行泵保活及新告警已闭，两正常正控/根先退SCM_RIGHTS外部持写/Unix watch外部持写均过；Windows run/watch/真实CLI4/内容负控保持。但根实际运行至280ms超时时，run_bounded约288ms返回后仍有reader（1ms后仍在、100ms后自行退），error亦明示残活；剩余1ms停止预算不足50ms poll，RESIDUE保留归属不能算返回前真停。两次实际重放同机制失败，receiver真实持fd且观察时始终仍活，非要求清理外部进程。04沿用原P2，需同树work覆盖真实超时与共享失败出口、补永久回归；07/生产信号/服务/协议/seam及安装范围按未变输入保持。777 strict0v3g141边/fmt/diff/client闭包通过，新driver无新增warnings，基线告警单列。executing6/14，T08终稿仍待当期收据/SD收口，最终pass后merge-owned冻结门pending。以下R6为历史，以R7当前范围为准；未实施/整档/提交/合入/清树。

### Plan 816（2026-10-23，整体B-R6 needs_fix；04 Windows闭合、Unix所有权回归一P2，07保持CLOSED）

受审 `0e50bedd0e3775caf3be5ca172af2e16e125b718`、r2；[完整R6/原样新driver与原生Linux反例](attachments/816/review-overall-r6/review.md)。04 Windows watch实际外部持写5.5ms/OS线程回基线，run约218ms/输出保留/线程回基线，两个Windows范围CLOSED；Unix新字节/行泵只捕获fd而未持有r，函数返回drop读端，正常子进程code120/两流空、ready行通道Disconnected，原外部持写也120/输出丢失；同helper直接执行0/两流完整。无线程残留不能掩盖输入已提前关闭，04沿用同ID/P2 OPEN，新driver告警Win1/Unix4仍未清。真实CLI4、owned后代/旧SFC/20s截止负控、服务三读错出口/JSON保持，777 strict0v3g141边/fmt/diff/闭包过。07安装/来源、服务信号/协议等按未变输入具名保持；不重开既闭安装缺陷。executing6/14，同树work修Unix实际管道所有权及正常/超时/原持写永久回归、warnings；T08终稿仍需当期收据/SD收口，最终pass后merge-owned冻结门pending。以下R5残项为历史，以R6当前范围为准；未实施/整档/提交/合入/清树。

### Plan 816（2026-10-23，整体B-R5 needs_fix；07闭合，04一P2含watch/Unix两残项）

受审 `4b83659882901dae487d3c7e56a0bcbcbc11f1bf`、r2；[完整独立R5/OS快照与安装身份](attachments/816/review-overall-r5/review.md)。07当期独立安装/来源缺陷CLOSED：默认与显式SDK真实delivery均同源安装sdk、双资源verify过，5539源hash复核，当前Core真实构建/唯一hash/脚本过/缺UI具名失败、依赖树424包0禁项；当期产物由受审安装收据绑定，旧work hash仅历史，整份T08/最终SD仍需收口。04 Windows run外部持写原例已闭（约208ms/输出保留/reader回基线），但WatchGuard未接新机制，5.002s返回后reader32492仍活；Unix新模块cancel空函数+阻塞read，305ms返回后任务707仍活，释放外部写端才退。沿用04一个P2，不新增ID；false留在局部Vec字段不等于函数返回后保留归属，不能只写readers stopped。56当前定向测试过、9ignored不充数，777/fmt/diff/依赖围栏过；旧读错聚合/正常-holder/预算/JSON及不变信号/泵/协议范围保持。执行6/14，下一步同树work收口watch/Unix及新增测试warnings后提交复审。最终pass后merge-owned冻结门/Spec-ledger/合入归档guard pending；未实施/整档/提交/合入/清树。以下R4的07 OPEN为历史，当前闭合以本轮范围为准。

### Plan 816（2026-10-23，整体B-R4 needs_fix；04/07原例改善、各余一P2）

受审 `bb6b91ca163b2fba4fe25ad50b9bb9a2d1ef2c3e`、r2；[完整独立R4/OS线程与安装双向控制](attachments/816/review-overall-r4/review.md)。04原owned后代反例已闭（code=-1/部分输出保留/hb4→4/drains true）、watch原23s后代20.07s内收尾及内容负控保持；但新drain仍丢弃未完JoinHandle，测试自己的外部receiver真实持stdout时caller返回后reader线程37208仍活，释放外部写端才退，非要求误杀不归属进程。07服务SDK反例已闭：精确重建5539安装输入，生成path在安装sdk，真实check71.51s/随后locked过；但同安装默认gen --delivery-root的producer.source_root仍为实施树，唯一显式安装SDK env对照才正确，默认工具producer接线遗漏。沿用04/07两P2，不新增产品finding ID。已闭组合构建/读错聚合/原service-holder/JSON/Unix实际SIGINT及777保持；consumer30+真实CLI4过，0选择的api-gen滤串不充数。执行6/14、A/T03/T04历史保持；同树补两残项提交后受影响复审，最终pass后的merge-owned冻结门/Spec-ledger/合入归档guard pending。未实施/整档/提交/合入/清树；以下R3为历史，原例闭合范围以本轮为准。

### Plan 816（2026-10-23，整体B-R3 needs_fix；两项闭合、04/07两P2残留）

受审 `3068be74371628132b158219fab19b0ac49c283e`、r2；[完整独立R3/可重放证据](attachments/816/review-overall-r3/review.md)。B-R1-02 reader错误聚合CLOSED：READY前后及exit0/7真CLI三态具名，正常/owned-holder/非法预算保持。B-R2-01 Unix关闭CLOSED：原生Linux同源service tail子进程真收SIGINT/0.264s优雅退出，故意不退15.249s强杀兜底。仅B-R1-04/B-R1-07 PARTIAL/OPEN（原ID，不增新产品finding）：04原内容与低频噪声样例改善，但run_bounded200ms约274ms返回0空输出时后代心跳4→10、排水线程归属被丢，原样watch测试Guard.join仍拖至23秒；07双exe邻接脚本/Vue17文件/旧codec选项NOT sent零副作用过，但安装版Rust service生成依赖绝对指向实施树SDK，实际安装/资源闭包不在交付输入内。源码fresh构建不能替代安装输入fresh/locked；延期资源至R07未经人类合同修订授权。85当前定向测试通过，777 strict0v3g141边/fmt/diff/依赖围栏过；执行6/14、A/T03/T04历史保持。下一步同树修齐两残项提交受影响复审，最终pass后merge-owned冻结门/Spec-ledger/合入归档guard pending；未实施/整档/提交/合入/清树。以下R2四P2为历史，本轮已闭两项不再按OPEN处理。

### Plan 816（2026-10-23，整体B-R2 needs_fix；原4项闭合，4P2待处理）

受审 `bd50b0abaf61077e2f413d39b93766c8d9fb7ae6`、r2；[独立整体R2/可重放反例与闭合表](attachments/816/review-overall-r2/review.md)。B-R1-01(P1)/03/05/06 CLOSED：locked三包bins/真实tool consumer30、Linux后台配置、JSON new、777当前strict0v3g141边及审计正反控15通过。02 PARTIAL：原owned-holder悬挂关闭，但READY前读错/READY后读错+exit7仍吞reader错误；04 PARTIAL：新内容oracle对旧字节故障正确红，但噪声每行重置20s预算使22s后仍绿，run_bounded200ms遇持管道后代仍join3s。07原合同安装清单/fresh/full/Core/首次启动/升级失败余量OPEN，记账交裁定不授权延期或转移merge。新增B-R2-01/P2：Unix service独立owned组未接父Ctrl+C，15.27s后强杀；同handler继承组合法对照0.032s优雅退出，需保持树归属落实信号送达/排空。44当前定向测试过、fmt/diff/双依赖围栏过，不变输入的旧泵/客户端/设施/协议证据具名复用。executing6/14，A/T03/T04历史保持；同树补齐后新SHA复审。最终pass后merge-owned冻结门、Spec-ledger/合入归档guard pending；未实施/整档/提交/合入/清树。以下R1为历史，不表示本轮四已闭项仍开放。

### Plan 816（2026-10-23，整体R1 needs_fix；1P1+6P2 OPEN）

受审`75dc3c058faa99fe4a52284dc790797c404b7089`、r2；[整体独立报告/AC/原始反例](attachments/816/review-overall-r1/review.md)。B-R1-01工具漏新ExecutionSpec字段E0063不能fresh构建；02service rust根退出后stdout后代使join挂住（8秒心跳仍推，正常对照exit0）；03Unix后台serve原spec必遭Windows flag负控；04同字节watch测试对保留旧SFC的真实故障仍绿，deadline/失败收尾也未闭；05合法JSON new成功混入模板文本；06最终根Cargo变更使777再次stale（exit2/1v3g）；07原合同必需的独立发行/manifest/fresh-locked-重定位与实际Rust链未齐，不能延期R07或转交review来决定是否做。旧R2 Unix/泵早晚取消+残留重试/选项/签名/格式改善保留，74定向正控过不覆盖上述问题。executing6/14，T05..T09/当前T13未闭；A/T03/T04历史结论保留，但不授当前全消费或整Plan通过。先同树work修齐并完成必需验收，最终pass后merge-owned冻结门/Spec-ledger/合入归档guard仍pending。未改实现/提交/合入/规范ledger/清树。下方T05 R1/R2均为历史收据。

### Plan 816（2026-10-23，T05阶段R2 needs_fix；5P2+1P3 OPEN）

受审`3c09bdf5b59d3f1aa4103b50c433a1bad3a327d8`、r2；[独立R2/旧例闭合边界/确定性反例](attachments/816/review-t05-r2/review.md)。原9断言同源码全绿，但R2-01..06：Unix缺libc四E0433、真实重定向2文件却报告0/错误根、早取消后实际reclaim静默丢归属且线程仍活、旧v1 codec吞新选项且同describe放行产生错误副作用、三旧seam注入consumer各五E0050不能编译、两份新改测试格式失败。110不同定向测过/原9断言过不能覆盖这些不变量；tool_process9ignored未冒称通过。R1-02/InputAck、后端覆盖、旧cfg和落点旧例改善保留，完整寿命/报告/升级兼容仍需修。executing6/14，T05/T13未闭，T06不越门；A/T03/T04历史pass保留但新SHA消费证据必须重建。原合同内修复，不能记债豁免；T07的777/发行矩阵/T08规范和最终merge-owned冻结门仍pending。未改实现/提交/合入/规范ledger/归档清树，下方R1为历史。

### Plan 816（2026-10-23，T05阶段R1 needs_fix；5P2+1P3 OPEN）

受审`b6148dd90d716038395ab32d5ce8d1d926267dfe`、r2；[完整独立报告/输入/持久反例](attachments/816/review-t05-r1/review.md)。F-816-T05-R1-01..06：输出/交付根被忽略却成功、InputAck误作run终态、输入泵不可停且shutdown PipesHeld/线程4→5、-r覆盖丢失、strict/argv不跨进程、新feature门控告警。真实CLI落点与严格反例、合法协议交错/held stdin对照均绑定当前源和产物；68定向测试过不能覆盖这些缺口，独立9断言3控制过6功能红。原合同内必修，honest_notes不授权降约。executing6/14，T05未勾/T13当前重开，T06不得越门；A/T03/T04历史pass及A交付不撤销。777开工前绿色已过期，当前1 stale_baseline/3 grandfathered/0新禁边，既定T07重采样；T06全链/T07发行与JSON矩阵/T08规范、最终merge-owned冻结门仍pending。未改实施/提交/合入/规范ledger/归档清树。下方816历史保留，当前以本轮为准。

### Plan 828（2026-10-22，T-10最终PASS；reviewed10/11，T11待merge）

受审5e8e550fa2c68c0e4e7b44187877e8b46469c22f、r3，[最终PASS与等价证据](attachments/828/review-t10-r2-5e8e550fa/review.md)。T10-H01/P3 CLOSED：两文件与审核rustfmt期望逐字节一致；整份改动Rust格式/源码diffcheck exit0，新根scalar6/check-all-targets零告警；519共同输入仅两文件格式变化，47份原证据hash一致，三512/兼容/资源/关闭/正式双门与SD按等价明确复用。AC01..09最终PASS，无work-owned必需残项；状态reviewed10/11，T11/AC10 merge-owned裸t+tv/实际触面consumer/workspace、tf到期主检单实例、Spec-ledger沉淀/合入/归档guard pending；当前不是D828交付。历史战役丢诊断归因未确证限制保留。未改实现/canonical/ledger、未整档/合入/清树。下方828 needs_fix均为历史，不当作当前OPEN。

### Plan 828（2026-10-22，T-10整份验收needs_fix；仅格式收尾）

冻结a030427fe15dc043871d3253101f209653cabd9c、r3，[整份报告与核销](attachments/828/review-t10-a030427fe/review.md)。功能/信任/生命周期/存储/预算/正式双门与SD审查通过；三512 3/3(275.43s)、原close9、core/旧consumer/resources、ac-core111/scalar6本轮独立过；同冻结R5证据518共同输入/137运行物/92证据hash全等可复用。唯一新T10-H01/P3：828新改ac-syntax projection_engine.rs及scalar_projection_.rs未满足rustfmt，属于T09/T10必需健康收尾，不能记债后豁免。仅两文件格式化+scalar6/全改动Rust格式门后新冻结窄复审；纯格式等价可复用重负载，不重做已闭机制。executing8/11，T01..08完成，T09/T10未闭，T11 merge-owned整档/consumer/Spec-ledger/归档guard pending。历史战役丢失诊断不确证负载归因，记录保留。未改实现/canonical/ledger、未根整档/合入/清树。下方828记录均为历史，不再将其已闭finding当当前OPEN。

### Plan 830（2026-10-22，final review R5 PASS；R4-01/02 CLOSED，无开放阻塞finding）

受审`764e87559cff016d9e0d6b2141e5f1f9903398c5`、r3，[完整R5/原反例/新三态组合/新源真实遗漏负控/冻结输入](attachments/830/review-r5/review.md)。三态区分未路由/目标/已取消；原focus-exits6/6、扩展状态9/9，取消/正确revalidate先于pending渲染不再复活焦点，合法恢复/非法拒绝/动作不丢保持，R4-01 CLOSED。新分管线精确与总量守恒/内容精确活控通过；从当前提交重新冻结正式源码，真实持续漏plane记账或content汇总都正式gate红、exit1，合法23/23，R4-02 CLOSED。定向45、原R2四holdout、六surface真实OS/中文IME、829三案、实际重定位29字段本轮均过；check/fmt/release零warning。AC01..09通过，reviewed9/10、T09完成、T10 pending；历史needs_fix/OS失败保留但不再是当前OPEN。尚需merge-owned冻结全套AGP门/真实OS/829及consumer/负控资源门通过后合入autographics-dev、Spec-ledger沉淀/归档guard清树；不是交付完成，未改实现/规范ledger、未根整档或合入清树。已批准的error-scope/Null剪贴板/每帧重绘/未测GPU驻留和AutoShader/CAP/I1–I3边界保持。有效参数24B与实际quad传输160B在沉淀表述中区分。当前结论以本R5为准，下方历史保留。

### Plan 830（2026-10-22，final review R4 needs_fix；F-830-R4-01..02 OPEN）

受审`5cd03b861832b9de49e2f04637a7fce130b1e7dd`、r3，[完整R4/同字节原例/取消组合/真实计数故障/冻结输入](attachments/830/review-r4/review.md)。原R3组合3/3与R2四holdout全过，R3-02非法Surface拒绝与R3-03字段遗漏CLOSED；定向40、六surface真实OS/IME、独立重定位29字段过。829两游戏过，Iced一次OS点击超时后同commit重跑过，两份保留。01(P1)合批pending后取消或正确revalidate卸载再frame，router=None而真实双widget均focused（None被当允许全部，pending重新聚焦）；4合法对照过2出口失败。02(P2)新上传活控在真实省略plane计数或内容汇总时仍19/19 exit0（quad160B顶替plane128B，内容>=允许等于组合）；正式源码/断言同字节，真实写操作保留，故障trace/原始JSON持久化，非当前产品漏计/泄漏。第四次needs_fix，executing4/10、T01..04保持、T05..08重开；同树work补取消时序/恢复和有区分力的计量负控，新提交重新冻结oracle再复审。原合同内必修，记债不允许延期缩面；未改实现/规范ledger、未根整档/合入归档清树，最终review后merge-owned冻结门/合入autographics-dev/沉淀/guard仍pending。历史保留，当前以R4为准。

### Plan 830（2026-10-22，final review R3 needs_fix；F-830-R3-01..03 OPEN）

受审`5aec0a63429d0a4cd3253751b195e8f74ddc2976`、r3，[完整R3/原始报告/独立组合反例/脚本/冻结输入](attachments/830/review-r3/review.md)。R2原四项holdout同字节4/4通过，R2-01/03/04/05/06 CLOSED；定向34、六surface真实OS/IME、829三案、独立重定位29字段均过，新场景源实际9999故障8门红/exit1。仍有01(P1)同帧pending两次正常点击后router=B而真实双widget均focused、blur后A残留；02(P2)公开focus拒绝foreign Surface前先释放当前合法焦点，返回false仍污染状态；03(P2)原V09/R1-F09组合上传量与内存事实/不可用原因缺失（数量守恒已修，不称泄漏；R2审核未单列旧子项，本轮补正）。独立组合探针1合法对照过2失败。连续第三次needs_fix，executing4/10，T01..04保留、T05..08重开；同树work补合批时序/拒绝原子性/完整计量诊断与永久正反控后重新冻结复审。全部既有合同内必修，记债不授权延期/缩面。未改实现/规范ledger、未根整档、未合入归档清树；最终review后merge-owned冻结门/合入autographics-dev/沉淀/guard仍pending。下方历史保持，当前结论以本R3为准。

### Plan 830（2026-10-22，final review R2 needs_fix；F-830-R2-01..06 OPEN）

受审 `dbaea29e122d2fa2a18d18ce3f09444583098efd`、r3，[完整R2/原例重放/新反例/真实坐标故障/冻结SD](attachments/830/review-r2/review.md)。R1结构dirty/超限事务/host相机/递归遮挡/卫生已闭，旧11反例10过（唯一失败为已排除的未revalidate release协议边界）；定向29契约、六surface正控、真实中文IME、829两游戏/spike、本轮独立重定位29字段均过。但01(P1)Iced事件更新未失效Canvas消费者，子蓝父仍红；02(P1)切换两个真实文本框后旧框仍focused且blur未清；03(P2)实际caret未接入，固定锚点与整平面gate代替光标区域；04(P1)第二设备3D内容因None归属绕过入口验证；05(P2)resize旧目标drop漏计，live0但created2/dropped1（计量缺陷，不宣称泄漏）；06(P1)场景命中local实际变异[9999,9999]仍14/14，预设坐标注入绕过真实路由输出。4/4新增holdout红、合法对照过，故障trace/原始JSON持久化。F02/06/07/08/09/10原例关闭与不变量未闭分开记录。连续第二次final needs_fix，下次ready前work须按报告诊断补真实入口覆盖/闭合表；executing3/10，T01/T02/T03保留、T04..08重开。全部合同内必修，不因记债允许缩面/延期。最终merge-owned实验门/合入autographics-dev/沉淀/归档/guard pending，未改实施/规范ledger、未根整档或合入清树。下方为历史。

### Plan 830（2026-10-22，final review R1 needs_fix；F-830-R1-01..11 OPEN）

受审 `dd2788bcc16ce7cff247895346afbbf343352496`、r3，[完整独立审核/反例/原始报告/冻结SD](attachments/830/review-r1/review.md)。01(P1)结构dirty遗漏导致卸载残影/resize空白；02(P1)clean父面反复叠半透明逐帧变色；03(P1)GPU限额坏资源panic且graph已提交；04(P1)多父相机命中丢实际host而随机错映射；05(P2)嵌套非交互遮挡回落父内容；06(P1)真实宿主失焦/取消/键盘IME焦点路由未闭合；07(P1)IME锚点取BFS第一挂载而非活跃路径；08(P1)S01真实UI→3D→UI、S02 reset/文本与游戏焦点、S03共享Squash等合同场景缺失；09(P1)负控未真实注入对应故障、生命周期/资源计量与当前重定位证据未闭；10(P2)SD/交接陈述与代码/OS证据矛盾；11(P2)新增warnings与rustfmt失败。原135测试、六surface正控及真实OS中文IME、829两游戏/spike均过，但独立11项反例=1过10失败（含显式revalidate合法对照；未revalidate直接release仅为协议边界，不单列缺陷）。全部既有合同内必修，不因记债获准延期。T03..08重开，executing2/10，T01/T02保留；同树work一次修齐后新冻结复审。merge-owned实验门/合入autographics-dev/沉淀/归档/guard pending；未改实施、canonical Spec/ledger、未触根整档或合入清树。下方历史保持。

### Plan 828（2026-10-22，85cbebea4 T02生产前端专项核验PASS；第一批包级复审待T03/T04）

受审 `85cbebea47dd8c62baac38879fc386ca67fe0b22`，r3；[专项报告/生产六组合/原样回放/冻结指纹](attachments/828/review-t02-85cbebea4/review.md)。synth桩管线删除，生产session单声明投影→保留CheckedEnv候选文档事务，producer真单body窗绿、同窗额外执行旧管线红；miss校验/lower/admit与hit已验对象复用边界保持。独立6组合、contract21/t01r4、原boundary11/admission2/publisher3/transaction8/关闭9、session17/r1 86/core22、六source兼容目标、精确scalar_projection_6全过，五文件fmt/diff过。无新增阻塞finding，C01..C05/F05b已测闭合保持；初版探针收敛/语法假设及filter只选5的记录保留并纠正，不计实施问题/零选择PASS。完整link失败前来源/历史映射原子性仍C02-04/T05/06已登记必修，未因本次前端拒绝/guard控制通过解除；内存/GC/正式门仍原owner。保留检查期间主Plan新增三批节奏：本收据仅显式T02专项证据，继续T03→T04后第一批统一复审，不据此提前第二批。executing0/11、其他OPEN不变，不授reviewed/merge；后续包级/最终验收与merge-owned门/沉淀/归档/guard pending，未改实施/整档/启动其他任务/合入/清树。下方为历史。

### Plan 816（2026-10-22，T04窄面R2 phase pass；R1-01 CLOSED）

受审 `08195b67e8652029ac66ce2dc0c42dde844a0fbc`，[完整独立R2及原例回放](attachments/816/review-t04-r2/review.md)。F-816-T04-R1-01 CLOSED：同字节原独立宿主坏流/exit1/exit7均TeardownFailed，保留候选路径和实际原因，干净EOF成功；首候选收尾失败无后候选副作用。发现序/身份/版本/能力/缺件控制保持，原回放零失败；从提交源码显式构建fixture默认target/debug宿主后完整22/22，client15/15、client/fixture clippy零告警、fmt/check/777/闭包通过。全新私有包源与提交同字节、默认布局构建及回归通过；新增宿主固定fixture默认target路径，须保留先构建前置，不声称支持任意共享target或依旧副本作新证据。仅T04窄面代码/验收/规范phase pass，executing7/14，T04当期/本次复用T13完成，A/T03历史交付保持。完整UI/profile/bootstrap按M815-B门待做，最终B冻结裸t/实际触面/完整consumer-profile来源与Spec-ledger仍merge-owned pending；未整档/改实施/合入/归档/清树，当前T04状态以本R2为准。

### Plan 816（2026-10-21，T04窄能力R1 needs_fix；一项异常探测收尾传播缺口）

受审 `5d8f6ebab6a2a3a3a04ca881795e94b06791a798`，[独立T04-R1/真实回放](attachments/816/review-t04-r1/review.md)。F-816-T04-R1-01(P2) OPEN：discovery忽略shutdown的Err和Failure退出分类，合法describe后坏前缀/规范exit1/异常exit7均返回已发现Ok；相同宿主底层ToolClient分别报告StreamViolation(TooLarge)/Failure(1)/Failure(7)，干净EOF正控通过。应在discover真实入口传播具名收尾错误/退出码并补正反控，不用静默回落掩盖。env优先、debug→release、邻接安装/默认安装、缺件与身份/版本/能力拒绝、坏前候选不启动后候选等独立控制通过；原fixture21/21/client15/15未覆盖此新组合错误出口。check0error、client clippy0告警、两面fmt/777strict/verify-deps通过。仅本次T04窄面needs_fix，完整UI/profile/bootstrap等M815-B不作新缺项。T03-R4 pass与已发布M816-A保持，原client方法同构/T03实现输入零漂移。executing5/14，T04/当前复用T13重开，T01/T02/T03/T12/T14及历史A/T03pass保留；原树work一次修齐此项提交复审，不推进当前依赖门，最终B冻结门/完整链/Spec-ledger仍merge-owned pending。未整档/改实施/合入/规范ledger/归档/清树。

### Plan 828（2026-10-21，T-01/27aaa34d5受影响面PASS；C05闭合，可进入T02）

受审 `27aaa34d565b11facab37008d42c6e509cc3e13d`，r3；[PASS报告/原样重放/冻结指纹](attachments/828/review-t01-27aaa34d5/review.md)。**T01-C05/P2 CLOSED**：公开recheck成功发布推进generation，与candidate commit统一基面失效；prepare临时swap/失败recheck不发布。publisher3/3（旧失败变generation1/旧commit拒绝/新current保持），事务八行8/8；C01/C02/C03/C04/F05b已测闭合保持，无新增finding。contract18/t01r4、boundary11/admission2/关闭9、session17/r1 86/core22、ac-core六目标94全过，两文件fmt/diff过。当前D01..03接缝/候选最小前置可交work/T02生产session前端热路径换接，完整session原子同点发布/矩阵仍按T02/T06、新代码再复审。按用户保持executing0/11，不自动勾T01、不授整份reviewed或merge；F01/F02/09/10剩余/11剩余/12/14等既有OPEN与owner不变。整份最终验收/merge-owned冻结门/Spec-ledger/归档/guard pending；未整档/改实施/合入/清树。此前连续失败与诊断历史保留，下方均为历史。

### Plan 828（2026-10-21，T-01/0b612fda1受影响面needs_fix；原六失败全闭，仅C05发布代际遗漏）

### Plan 816（2026-10-21，T03-R4阶段pass；R3-01 CLOSED，无开放T03 finding）

受审 `5720a90594b8e5964b82c3772c2f52814597cbb9`，仅测试oracle/注释变化；[独立R4及证据](attachments/816/review-t03-r4/review.md)。F-816-T03-R3-01 CLOSED：从新提交原测试块独立重建晚复位变体，真实VM返回2时新标量断言红（left2/right1），正确入口同测试绿；三份测试块同hash、与R3旧块不同，未重跑冻结旧副本冒充修复。R1五项/R2两项保持CLOSED，依赖/配置/probes/生产实现与R3相同的指纹及前缀证明允许复用其已绿证据。runtime118过+4ignored、fmt/diffcheck过，777 strict重跑pass/0违规/3grandfathered/132边；clippy仍30唯一旧IO错误有既存同构归因、language改动零诊断。仅T03代码/验收/规范当期phase pass，executing6/14，T03/当前复用T13完成；已发布M816-A保持。下一步T04窄能力门已具备，完整UI正控/profile/bootstrap按M815-B门推进。最终B冻结裸t/实际触面档/完整consumer-profile来源与Spec-ledger仍merge-owned pending，当前未整档/合入/归档/清树，不等于整B或整Plan交付；历史needs_fix记录保留，当前T03状态以本R4为准。


受审 `0b612fda1ce004f0565a4f2e1057768f841a0f31`，r3；[报告/原八行适配/单失败及正控/冻结指纹](attachments/828/review-t01-0b612fda1/review.md)。C05候选所有权重设计有效，原八行同输入/同状态oracle适配8/8，上轮六失败全闭；C01/C02/C03/C04/F05b已测闭合保持。**原T01-C05/P2仅窄残项OPEN**：公开recheck_function成功替换当前条目未更新generation，只有candidate.commit更新，实际当前1→prepare2(base0)→公开recheck发布3(gen0)→旧候选commitOk覆盖成2。publisher3控制2pass1fail，新候选间stale拒绝/失败recheck不失效两合法对照过；不新增ID，不称session已发生污染。原boundary11/admission2/关闭9、contract16/t01r4、session17/r1 86/core22、ac-core六目标94全过，四文件fmt/diff过；保留旧API合法原地语义，修复需让所有真实发布统一参与基面失效，纯prepare临时swap/失败/取消不发布。只同树补此遗漏，勿重做已闭候选机制，三联动/八行通过后冻结复审，再T02全面换接。executing0/11、其他OPEN不变；最终验收/merge-owned门/Spec-ledger/归档/guard pending；未改实施/整档/合入/清树，记债不授权缩面延期。下方为历史。

### Plan 828（2026-10-21，T-01/de4cba702受影响面needs_fix；C04已闭、仅原C05残项）

### Plan 816（2026-10-21，T03-R3 needs_fix；两项产品修复与777通过，仅余argv回归oracle）

受审 `d93106fd20a10ca6e4a435b8edbbdfc1518e0e34`（实现 `2f2b54888`），[独立R3与持久回放](attachments/816/review-t03-r3/review.md)。R2-01双槽注入与R2-02入口argv复位产品反例CLOSED，R1五项保持，777 strict pass/0违规/3 grandfathered/132边。仅F-816-T03-R3-01(P2验证缺口) OPEN：新增永久测试执行返回argv长度标量，却按逗号片段数断言n=1，VM正确返回1与泄漏返回2均被接受；晚复位独立真实VM副本保留整个提交测试块不改、测试仍过，观察正确1/变体2且返回后全局表为空。当前生产实现已正确先清空，不表述为仍有产品泄漏；应断言VM标量值并用该负控验证漏检消除，修复后从新提交重生成变体核对测试块。runtime118过+4ignored、122名唯一、集成10/10、管理8/8、check/fmt/零UI闭包/类型单源保持；clippy30唯一旧IO错误有同构归因。executing4/14，T03/本次复用T13重开；已发布M816-A及T01/T02/T12/T14保持。原树仅修oracle与相应负控提交后受影响复审，T04暂不启动；最终B冻结门/完整链/Spec-ledger仍merge-owned pending。未改实施/整档/合入/规范ledger/归档/清树，当前T03状态以本R3为准。


受审 `de4cba7022a2dd9f2d03ee3dea91322a5088674c`，r3；[报告/八项真实事务控制/原样回放/冻结指纹](attachments/828/review-t01-de4cba702/review.md)。**T01-C04/P1 CLOSED**：五profile公开完整凭证受控拒缺body，额外五profile调用/未调用混合十组合拒绝，函数级脚手架合法保持。**T01-C05/P2 PARTIAL/OPEN**（不增ID）：prepare后置verify拒绝恢复有效、顺序取消/继续与closed过；但成功prepare返回当前env已换成候选、drop不恢复，commit只查closed不持候选/不核owner基面，restore只比名/槽。同名foreign restore实际接受并改另一环境9→旧1；双候选取消A再commitB返回Ok却实际1应3，先commitB后迟到取消A亦回1。独立事务8=2pass6fail；当前session未接新env，不报已发生session污染，原F05b foreign/late守卫不重开。原boundary11/contract13/t01r4/关闭9/session17+r1 86/core22及ac-core六目标94过，额外admission2过，五文件fmt/diff过；旧11的D03例只是旧原语示范，永久A/B实际顺序、foreign异名，绿色不覆盖隔离不变量。C01/C02/C03及F05b保持CLOSED，executing0/11、其他OPEN不变。连续失败先对账候选所有权与发布点/owner基面，下一轮同树仅修C05，候选隔离、commit真安装、drop/cancel不改当前，重验后冻结复审，通过后才T02全面换接；记债不授权缩面延期。最终验收/merge-owned门/沉淀/归档/guard pending；未修实施/整档/合入/清树。下方为历史。

### Plan 828（2026-10-21，T-01/23a6b35a0受影响面复审needs_fix，旧三项闭合）

### Plan 816（2026-10-21，T03-R2受影响复审needs_fix；原五例闭合，新增两项修复回归）

受审 `ae6eb14dc063a4896cfebfb40202536caf3ae848`，r2/T-03；[完整独立R2及持久回放](attachments/816/review-t03-r2/review.md)。R1-01..05五项原例CLOSED；新增F-816-T03-R2-01(P2)：compat ctor先占OnceLock，使调用方首次显式provider注入静默失败；相同consumer的runtime-only对照接受，compat真实链接进程拒绝。F-816-T03-R2-02(P2)：新语言adapter绕过历史run_file的参数复位，有效process.args源两入口真VM成功，native argv旧1/新2且前次sentinel泄漏；空参数正控保持，读取失败也旧清空/新残留。配套777 strict因auto-man/Cargo.toml锚定输入变更exit2/stale_baseline，须随修复处理，不复用旧绿；132声明边metadata一致、3既存逐边注记。runtime117过+4ignored、集成7/7、management8/8、121测试名唯一、类型单源与420包零UI闭包保持；check/fmt通过。clippy30唯一旧IO错误在与原基线同构两文件，证据归因保留。executing4/14，T03/当前复用T13重开；已发布M816-A与T01/T02/T12/T14、A历史pass保持。原树work一次修齐两项、777与定向控制提交后受影响复审，T04暂不推进；窄tool门不等M815-B，完整UI正控/profile/bootstrap等M815-B。未改实施/整档/合入/规范ledger/归档/清树；当前T03状态以本R2为准。


受审 `23a6b35a0e94a6c948230df94c0e686129c2e8ae`，r3；[报告/五profile公开准入反例/原样回放/冻结指纹](attachments/828/review-t01-23a6b35a0/review.md)。**T01-C01/C02/C03 CLOSED**：单body实产物1、同窗真旧synth使新生产门红、private_interfaces新增warning清除。**T01-C04/P1 OPEN**：五profile schema放宽body后公开binder产MAX哨兵，完整verify继续依赖旧binder拒绝前提而授信，真实native均越界panic；原descriptor同缺body文本拒绝。**T01-C05/P2 OPEN**：D03允许后置失败候选留当前env，严格隔离列为可选，候选机制/最小事务首例未定；原语已证recheck成功后verify拒绝不自动恢复，session尚未接新env，不报已发生session污染。contract6/收编4/关闭9/session17+r1 86/core22与六ac-core目标过，独立11=6pass5fail（同C04五profile），九文件fmt/diff过；原独立4原样3pass1fail残留旧弱谓词，以真实同窗生产门等价重验关闭C02。D02有界单元派发/真实跨修订身份及copy执行、递归/局部空间/完整trap span、真实坏绑定拒绝过，完整F02仍OPEN。executing0/11，F05b与既有来源闭合保持；其他OPEN不变。当前两项均既有合同内必修，不授权缩面/延期。连续两轮T01失败先对账共享授信边界/候选所有权根因与覆盖调整，同树work一批修齐再冻结复审；暂不T02全面换接。merge-owned冻结门/沉淀/归档/guard pending；未修实施/整档/合入/清树。下方为历史。

### Plan 816 T-03（2026-10-21，T-13受影响复审T03-R1，needs_fix；M816-A已交付保持）

受审 `1629aa60f0bc890c06a6a403aad5e6c07697c35c`，r2/T03；[五项复审/独立consumer及闭合oracle](attachments/816/review-t03-r1/review.md)。F-816-T03-R1-01..05(P2) OPEN：旧库UI入口新增手工install前提；runtime非UI VM服务回调UI provider、Rust workspace缺provider返回空值；旧auto_man::util六路径E0432；Git attached/detached判定反转；四条adapter旧回归被删且一条重复登记。真实consumer反例与已安装/合法API占口/clean-dirty/type identity等对照固定；两被删纯回归在无UI消费方2/2可过，旧测试迁移不能靠总数。runtime静态闭包420包禁项0/类型单源通过，三包check、runtime115次执行（114唯一通过+4ignored）、management8/8及777 strict132边0违规通过；clippy旧30唯一错误在完全同源旧模块，另有重复属性新告警归05，未声称全绿。T03与本次复用T13检查重开，executing4/14；原树修齐再审，当前不推进T04。窄T04只等T03+已发布窄tool，完整UI正控等M815-B，不误加整体815门。A十轮审查/实际发布与T14收据保持；未修实施/整档/合入/规范ledger/归档/清树，B最终冻结门和沉淀仍pending。

### Plan 828（2026-10-21，T-01/3e60af2c8受影响面复审needs_fix）

受审 `3e60af2c8aa2c287633f4700b377714e94c4ff4f`，r3；[复审报告/独立反例/原始输出/冻结指纹](attachments/828/review-t01-3e60af2c8/review.md)。**T01-C01/P2 OPEN**：stub_resolved先深clone拥有式body再清空，scoped emitter仍为全部函数生产body文档，真实三函数单body发射得到3个body，不能按接口目录遍历授PASS。**T01-C02/P2 OPEN**：真实旧synth管线计划2函数后原同gate仍true，固定receipt/局部env计数与gate(None,None)不足以发现额外全量工作。**T01-C03/P2 OPEN**：新CheckedEnv.sigs与私有FnSig可见性不一致，新增private_interfaces warning，须本轮处理。独立frontend4控制2pass2fail，合法冷编对拍/verify和非首位递归正控过；原contract5/5、r23a关闭原样9/9、六文件fmt/diff过。D02无merged方向/旧现状量化可保留，所选局部表/派发机制首例及D03候选环境事务责任仍PARTIAL；T01不通过，不直接进入T02全面换接。全部为在途必修项，登记不授权延期/降级；F05b与既有来源闭合不重开，其他F01/02/09/10其余/11其余/12/14 OPEN及executing0/11保持。下一步同树work修三finding并补前置证据，新冻结根受影响面复审；merge-owned门禁/沉淀/归档/guard仍pending。下方为历史。

### Plan 828（2026-10-21，r23a受影响面复审PASS；F05b-C01关闭拥有引用释放闭合）

受审 `47894c2e6747c8c06032be7a3d40bf00b90a204d`，r2；[r23a报告/Weak原例/所有权组合/冻结证据](attachments/828/review-r23a/review.md)。**F05b-C01/P2 CLOSED**：close清fns并替换轻量vacant根，无外部owner的根/HIR/code在close即死；旧close6/6（两失败转绿），新增3/3覆盖空根普通/分步执行与getters/幂等、外部history-hit候选处置后释放、最后PausedRun完成/实际取消后的Weak死亡。永久86/86、主17/17、core22/22、原62会话62/62和来源矩阵保持，fmt/diff过。F05b已测取消/终态/关闭拥有引用范围CLOSED，无新finding；四既有probe warning及非阻塞注释重复归F14。F01/02/09/10其余/11其余/12/14 OPEN不变，executing0/11，T06/T07未整体完成；继续同树work核心必需项。最终pass后的merge-owned冻结门及Spec/ledger/归档/guard仍pending；本轮未改实施/整档/合入/清树，下方为历史。

### Plan 828（2026-10-21，f1cd60778/F-05b受影响面复审，needs_fix；关闭拥有引用未释放）

受审 `f1cd60778f5438beecc3e35bb4b2afb56a9030fc`，r2；[r22a报告/关闭Weak反例/冻结证据](attachments/828/review-r22a/review.md)。r22a三函数24格矩阵完整收编PASS；取消不安装、兄弟候选提交/迟到取消、跨会话、全部终态守卫/幂等、退役history代码释放和真实PausedRun关闭后完成均PASS。**F05b-C01/P2 OPEN**：close清history/store却保留self.current与FnRecord当前HIR/code；无外部pin的root/HIR/code Weak在close后仍活，drop(session)才死，违反§5.4/V06关闭释放。独立close6控制4pass2fail（同一机制），永久85/85、主17/17、原62会话62/62、原相邻24格及历史来源矩阵过；fmt/diff过。F05b PARTIAL，T06/T07未整体完成；executing0/11。其他F01/02/09/10其余/11其余/12/14及既有来源闭合保持；不能用F10开放替代F05b关闭释放要求。原树work补close拥有引用释放、保活外部pin与PausedRun并收编Weak正反控后复审。最终pass后的merge-owned冻结门及Spec/ledger/归档/guard仍pending；未改实施/整档/合入/清树。

### Plan 828（2026-10-21，r21a受影响面复审PASS；函数/指令身份两项P2闭合）

受审 `3d14a15d613dbd79724c7dd0ac24917dc5169a28`，r2；[r21a报告/原样重放/相邻矩阵/冻结证据](attachments/828/review-r21a/review.md)。**F11-S01/function-id、F10-S02/instruction-id CLOSED**：来源证明同时绑定instr、func和映射后端点，选择与写回共用完整谓词；原binding6/6（四失败转绿），永久82/82，applicability5/5、generation8/8、sidecar6/6、62会话62/62、主17/17保持。新增三函数24格全过，含合法非零func1、错用同snapshot func0/MAX、重复instr及双轴损坏，primary/repair/无repair冷写回和公开执行/二次恢复。无新增finding，四既有probe warning归F14。仅本轮来源适用性/修复不变量关闭；F01/02/05b/09/10其余(A07/依赖/GC)/11其余(changed-only/manifest)/12/14仍OPEN，executing0/11；T02/T07未整体闭合，不授reviewed或交付。最终pass后的merge-owned裸t+tv/必要触面consumer/Spec/ledger/归档/guard仍pending；本轮未改实施/整档/合入/清树，下方r20a为历史。

### Plan 828（2026-10-21，r20a独立修复复审，needs_fix；来源身份/修复准入仍未闭）

受审 `e083f2a28ec4d8df05caf5013d502b0c16c0ce0d`，r2；[r20a报告/四失败反例/冻结证据](attachments/828/review-r20a/review.md)。r19a原applicability5/5（两失败转绿）、generation8/8、sidecar6/6、62会话62/62；永久76/76、主17/17、core22/22及798定向兼容全绿。**F11-S01/P2仍OPEN**：v2只比映射端点不比func，func0→u32::MAX实际准入且位移16格后返回旧23..28/funcMAX，应39..44/func0；坏primary继续遮蔽好repair。**F10-S02/P2仍OPEN**：忽略instr，[0,1,2]→[1,1,2]被弱谓词接受，后置verify回冷；已有好repair未被选、无repair回冷也未修复，第二次仍admitted0应1。新增6控制=2pass4fail，均真实store/执行输入及mutation断言；两项沿用既有完整不变量，不新增产品scope。probe四warning保留F14；其他F01/02/05b/09/10其余/11其余/12/14不变。保持executing0/11，T02/T07未闭；记债不批准延期/缩面。最终pass后的merge-owned裸t+tv/必要consumer触面/Spec/ledger/归档/guard仍pending；本轮未改实施/跑整档/合入/清树。

### Plan 816（2026-10-21，M816-A独立阶段复审R10，phase pass；当前finding全闭，合入门pending）

受审 `2551467776fea8844d43eed44dc3d0b62a7a2817`，r2/M816-A/T13；[独立R10/闭合矩阵及冻结证据](attachments/816/review-r10/review.md)。F-R3-02 CLOSED：写端准入移到所有reader/writer前，原四创建/复制故障入口具名且线程4→4、非owned持有者存活；F-R9-01 CLOSED：正常清槽后暂停组合等待实际结束并join，Done/无活writer/不提前返回。61不同现有Windows测试、全部旧共享回放/健康/闭包/777/CLI-Man及Linux净新增warning0通过。新增validate集测仅覆盖提前InvalidSpec拒绝，真实分配故障清理由持久R9四入口probe和顺序审查证明，不扩大负控声明。当前无开放finding，既往记录保留；仅A阶段pass、executing4/14，T01/T02/T12/T13完成。T14冻结裸t/最终触面/新包fixture闭包与关键回放证据、实际合入/SD01-A+SD05-A/受管ledger-index/来源读回仍pending，门通过前不得合入或称交付；无VM/trans/book/UI生成/AAVM触面，最终并集复核，tf主检出到期批量。本轮未改实施/整档/合入/规范ledger/归档/清树；保留原树继续B，当前状态以R10为准。

### Plan 816（2026-10-21，M816-A独立阶段复审R9，needs_fix；run拒绝收尾未闭，新增正常完成误判）

受审 `89b0b33ef71fd6c5a3f40fd8a7fc726063efcefa`，r2/M816-A/T13；[独立R9/两消费方真实失败入口与完成状态对照](attachments/816/review-r9/review.md)。F-R3-02(P2)仍OPEN：fallible凭证API和具名映射已验、client创建/复制失败线程4→4，但run两错误出口直接?绕过finish_early，错误返回后两reader存活（4→6），非owned持有者结束后才回4；两新增局部负控没到实际消费者/清理。新增F-R9-01(P2修复回归)：writer正常清槽后未返回窗口被is_finished+None检查误报缺凭证并带活writer返回；同一测试/透明调度hook当前失败、R8基线通过。60不同现有Windows测试、旧R1/R2/R5/R6/R7与终止失败回放全过，原取消时序/终止有界化/Linux健康保持，F-R7-01/F-R6-01/F-R5-01 CLOSED。executing1/14，T02/T12重开、T13未过、T01保持；原树一次修齐两出口与正常完成竞态、补真实负控后提交复审。SD05-A对齐、SD01-A全出口真停不能发布；T14冻结门/合入/Spec/ledger/来源pending。未改实施/整档/合入/归档/清树，当前以R9为准、历史保留。

### Plan 816（2026-10-21，M816-A独立阶段复审R8，needs_fix；仅余取消凭证初始化失败残项）

受审 `2d21cff8c3eb813fbce389d5449e0bb0311f1355`，r2/M816-A/T13；[独立R8/故障状态回放及闭合矩阵](attachments/816/review-r8/review.md)。F-R7-01 CLOSED（Linux净新增告警0）；F-R3-02有效凭证时序/客户端终止失败有界化子项CLOSED，仅取消凭证初始化/复制失败仍OPEN(P2)：.ok()/Clone空句柄静默丢能力，重发仍用同一凭证，“非承力冗余”说明不成立。TEMP生产lib原样前缀+追加测试、None/NULL两个生产失败结果边界注入，均3000ms PipesHeld且writer仍活；断言前等有限后代结束并join，未伪称OS实际复制失败。58现有Windows测试及R1/R2/R5/R6/R7旧回放全过；新终止失败注入0ms具名返回/真实收尸通过；健康/闭包/777/CLI-Man check通过。F-R6-01/F-R5-01保持CLOSED，无新增独立finding。executing1/14，T02/T12重开、T13未过、T01保持；原树只补凭证失败前具名拒绝或真实兜底及永久负控，提交后受影响复审。SD05-A对齐、SD01-A全路径真停不能发布；T14冻结门/实际合入/Spec/ledger/来源仍pending。未改实施/整档/合入/归档/清树，当前以R8为准，既往记录保留。

### Plan 816（2026-10-21，M816-A独立阶段复审R7，needs_fix；Linux编译已闭，取消检查后窗口未闭）

受审 `05d3940c6f57254bf74027079d4d00ac70ca8bac`，r2/M816-A/T13；[独立R7/透明调度插入及共享回放](attachments/816/review-r7/review.md)。F-R6-01 CLOSED（Linux两包+consumer check exit0），F-R5-01保持CLOSED。F-R3-02(P2)仍OPEN：旧R6入口前栅栏过，但内层stop检查后、真实write前暂停，取消槽关闭后放行，生产3s收尾后PipesHeld且writer仍活；client timeout/shutdown仍经terminate失败后无界wait，取消凭证失败静默降级未改（失败分支源码证据）。新增F-R7-01(P3)：Linux cancel局部变量/Worker.cancel字段两条净新增warning，三旧warning不计。58不同现有Windows测试及R1/R2/R5/R6旧输入全过；Windows健康/闭包/777与CLI-Man check过，不能代替新时序与净新增告警检查。executing1/14，T02/T12重开、T13未过、T01保持；原树修取消协调设计/失败路径与告警，补永久控制提交后再审。SD05-A对齐，SD01-A真停断言不能发布；T14冻结裸t/触面/新包fixture/实际合入/Spec/ledger/来源仍pending。未改实施/整档/合入/归档/清树；历史记录保留，当前以R7为准。

### Plan 816（2026-10-21，M816-A独立阶段复审R6，needs_fix；R5-01闭合，取消竞态与Unix编译未过）

受审 `a972286fd5f0aa05338ef4ca0c09c54a56de0027`，r2/M816-A/T13；[独立R6及确定性回放](attachments/816/review-r6/review.md)。F-R5-01 CLOSED：原stop读错/半帧三注入全过。F-R3-02(P2)仍OPEN：原writer已挂起取消场景通过，但取消先于真实write时凭证被消费关闭，随后WriteFile仍阻塞，生产3s收尾后PipesHeld且writer仍活；client早退与OwnedProcess Drop仍有未闭错误出口。新增F-R6-01(P1)：Windows专属类型无条件泄漏至Unix签名，真实Linux编译两E0433/exit101，上个6e03原源码同目标exit0；须两crate/consumer整体cfg隔离，不砍Unix支持。58个不同Windows定向测试及R1/R2/R5原回放均绿，不能代替新增取消顺序/跨目标负控；健康/闭包/777通过。executing1/14、T02/T12重开、T13未完成、T01保持；原树一次修齐/永久控制/提交后复审。SD05-A对齐，SD01-A真停/跨平台断言不能发布；冻结合入门/Spec/ledger/来源仍pending。未改实现/整档/合入/归档/清树；当前以R6为准，既往记录保留。

### Plan 816（2026-10-21，M816-A独立阶段复审R5，needs_fix；R4同提交pass已被新证据替代）

受审 `6e03f5f268ee0eb4a3c2b9f64d4924c226977b72`，r2/M816-A/T13；[独立R5/实机及确定性共享输入](attachments/816/review-r5/review.md)。F-R3-02重开(P2)：DirectChildOnly+非owned后代持stdin读端，run/client 250ms超时约3254/3272ms后返回PipesHeld，但own writer线程4→5，释放测试后代才回4；有界具名并不是真停，永久测试接受PipesHeld后再释放keeper的oracle不足。早退stop不drain/忽略join失败亦未闭。F-R5-01新增(P2)：R3 stop排空helpers返回()并吞read错，设施Stopped/stderr Ok、半帧仅Done丢TruncatedEof，三确定性注入全失败。两项均原合同内，不授权真停/错误语义降级。53个不同现有定向测试及R1五原例/R2四控制全部通过，旧常见路径修复保持；健康/闭包/777通过。T02/T12/T13重开、T01保留，executing1/14；原树work诊断两处false oracle/终态传播后修齐、永久回归、提交再审。SD05-A已对齐，SD01-A不能发布；merge-owned冻结门/实际合入/Spec/ledger/来源仍pending。未改实现/整档/合入/归档/清树；下方R4等均历史，以R5为当前结论。

### Plan 814（2026-10-21，R8/T-10 B最终复审就绪核查，needs_fix）

受审 `780415083650ddbfdd34587baf7e6e93af25e9ee`，r3、B work交接；[R8收据/限定核查与冻结输入](attachments/814/review-r8/review.md)。当前三项OPEN：F814-R8-01(P2) V-03仍明示Vue/Playwright双端对照未跑，却全勾T08并列“非本Plan面”，原B合同oracle未获准取消；02(P2) T09声称SD03全文但handoff§5只有提纲，缺可审定B正文及相对A已发布Spec的精确增量/hash/AC映射，仍指A的T14/T15；03(P3) 四包check过，但新runtime根lib.rs:66/72两未用再导出warning与新ui-runtime-consumer格式红未清。明确不把上层3条A旧warning/runtime全部警告判新增。提供的18选择清单经显式路径+R6身份证明+三个根模块源码pin，九档无遗漏/歧义，不报实际丢测；不认证工作脚本唯一叶名泛化，当前HEAD实际列表/完整B运行寿命仍待最终审查。因已知work-owned oracle未完成，本轮不展开完整最终验收，未整档/GUI/修实施/合入/规范ledger/归档。重开T07/T08/T09，保留T04/05/06实施及A已发布历史；原front10与实际12勾不一致，重开3后按真实任务数为executing9/15。next原树work一次补齐三项、提交并更新精确handoff，再进完整T10；记债不授权延期或缩面，最终pass后merge冻结门/来源知识与实际发布收据仍pending。A发布及下游已开放能力不因未合入B核查而撤销。

### Plan 816（2026-10-21，M816-A独立阶段复审R2，needs_fix；三项已闭三项未闭）

受审 `4372d56d90670210e90a5a7a97e672cec1867a3c`，r2/M816-A/T13；[完整R2/闭合表/原例及相邻组合](attachments/816/review-r2/review.md)。F-R1-01阻塞写、02 owned根退出/持管道后代、04 Windows挂起入Job时序独立CLOSED；原五反例5/5通过，双向大流量正控字节完整。F-R1-03(P2)仍OPEN：关闭阶段坏帧已被reader报Violation，却在join_all_bounded中丢弃并返回Success；05(P3)原格式/文档已清，但fixture新增:395 unused_mut；06(P2)正常helper注入过，生产超时拆除仍吞收尾失败、stderr仍吞读错，250ms超时实等6268ms报告elapsed250ms，返回后两own reader仍活（线程4→6，释放测试后代才回4）。属于原不变量残项/修复健康回归，无新增ID、不批准将真退出降低为有界等待；非owned后代继续遵守不清理规则。44个不同现有定向测试通过，不能替代上述两失败控制。SD05-A处置已对齐；executing1/14，重开T02/T12，T01保持；原树work诊断helper与实际收尾的覆盖断层，一次修齐03/05/06、永久控制及提交后复审。merge-owned冻结门/实际发布/Spec/ledger/来源仍pending；未改实现/整档/合入/归档/清树。R1历史保留，当前状态以R2为准。

### Plan 814 M814-A 发布遗留跟进（2026-10-20，T-15 收据）

master `f6e07acee` 发布后两项跟进：①交付投影不感知 .gitignore——master 树
ffi_dual `build_a2r/` 忽略态生成残留（含绝对 path 的生成 Cargo.toml）曾使
792 driver 首轮判红（外部 path 守卫），清理后 25/25 复绿；投影应在 staging
时跳过 ignore 态路径或在守卫前过滤生成物（owner=auto-man 交付域，B 阶段
或独立修复）。②777 master 侧基线重采样被并发会话脏文档阻塞（拒绝脏树
刷新），树静后补（strict 当前 2 violations=stale baseline，非真实违边）。

### Plan 816（2026-10-20，M816-A独立阶段审查R1，needs_fix）

受审 `6b4bf9c2fcf3e872b33f6cc8505cf9fa09aa550f`，r2/M816-A/T-13；[完整报告/冻结SD/可重放反例](attachments/816/review-r1/review.md)。当前六项OPEN：F-816-R1-01(P1) 250ms deadline不覆盖stdin/协议阻塞写，约1800ms断管才返回；02(P1) 根先退出后代持管道导致run/shutdown越过期限约1800ms仍Success；03(P2) 合法响应与坏前缀粘包误报Timeout，拆分正确StreamViolation；04(P1) Windows先spawn后assign Job有早派生逃树窗口（源码证明）；05(P3) 新client/fixture格式未过、execution文档3条新warning；06(P2) reader/read/join/cancel错误被吞（分支核查，未声称内核错误实测）。均为阶段合同内缺口，登记不授权延期/降级。29个不同原定向测试、真实consumer fresh/locked与闭包、777 strict、CLI/Man check及新包clippy通过，不覆盖五个新增失败观察；反例回放exit1含三个对照。executing1/14，保留T01、重开T02/T12，T13未过；原树work修齐并补永久控制后提交再审。B依赖/发行保持原门，SD05-A候选处置需对齐。merge-owned冻结裸t/触面/实际合入/Spec/ledger/来源仍pending；未改实现/整档/合入/归档/清树。

### Plan 828（2026-10-20，修复复审R2，就绪/原例与相邻控制，needs_fix）

受审 `791c62ab251b5d0f36d3d5610fade600760d2f52`，r2；[R2关闭表/11项重放/诊断与AC-V-SD](attachments/828/review-r2/review.md)。**R1-03/07/13 CLOSED**（递归与互递归/旧snapshot、恢复后未改代码Arc、脚本参数/workspace/零选择与错误传播）。**05/06/08/09/14 PARTIAL**：跨会话原例修好但取消/关闭未做；凭证自证原例修好但Scalar凭证仍获Core环境接受；trace启动修好但trace在未改callee时caller合法体编辑被拒；冷启4MiB原例修好但完整资源门未做；fmt修好但文档/SD和新增context_revision dead_code待清。**01/02/04/10/11/12仍OPEN**：真实函数前端/执行payload、未改trap位置、A07/实际依赖及GC、持久全unit重写、正式IPC/双线内存和真实反控缺口。永久修复6/6、原17/17绿；独立原7例6过1失败，新增4控制2过2失败（总11例8过3失败）；不以重复scope凑覆盖。work自己仍列in_progress/必需OPEN，故仅就绪/有界复验，不能授最终pass。executing0/11保持，下一次就绪须先补原树实现及诊断/覆盖调整；记债不批准延期。未改实施/跑整档/合入，merge-owned冻结t+tv/consumer与沉淀/归档/guard仍pending。下方R1为历史，已闭子项不复活。
### Plan 828（2026-10-20，独立送审就绪/有界反例复审R1，needs_fix）

受审 `940445bf57674780bc6ff6c68928c0c7148a0dbf`，r2；[R1报告/AC-V-SD/7项可重放反例](attachments/828/review-r1/review.md)。**F-828-R1-01..12(P1) OPEN**：热前端仍整Program resolve/typecheck/Atom中转；snapshot重建全体执行payload；递归体热更新被unit准入拒绝；未改函数trap来源陈旧；跨会话PreparedEdit错误安装；凭证未核真实环境；restore后未改代码身份切换；旧trace支持丢失；4MiB输入cap绕过；A07/实际依赖及GC释放证据缺失；持久热编辑重编解码/写入全部unit；正式worker/IPC/双线内存和真实反伪造门未交付。**R1-13(P2)/14(P2，fmt子项P3) OPEN**：脚本参数/非成员workspace/零选择保护缺陷，交接及SD与现状不符、新模块fmt红。已有828测试17/17及scalar差分6/6绿，独立7例7失败可复现；记债不批准延期/缩面，正式/内存不得转batch或后继。Plan回executing0/11，所有work任务完成断言重开，现有实现和正控历史保留；原树一次修齐后提交复审。未改实施/跑整档/合入/归档/清树；独立pass后的merge-owned裸t+tv、必要workspace/consumer门及Spec/ledger/guard仍pending。
### Plan 814（2026-10-20，R7/T-14按r3复审，M814-A phase pass；当前finding全闭）

受审 `256613936b6a5e9baea450e5c545899ee7a661dc`，revision3/M814-A；[R7阶段收据/AC-SD矩阵/独立重验及复用边界](attachments/814/review-r7/review.md)。R4-01在r3批准后的具名登记例外与端口归属下CLOSED，r2不通过保留为历史；当前R3-01..06/R4-01全部闭合。原树/依赖/配置未变，R6冻结34输入hash一致；复用strict proof九档逐ID及警告净新增0/减少7，不认证工作脚本唯一叶名泛化。独立重验显式host、直达Parser、frame显式覆盖/VM快照与autodown全操作/错误对照均exit0，无UI源码fresh+locked SDK5/5、compiler外部消费4/4。当前候选§8与SD已冻结绑定r3/256613936，SD03完整runtime留B。T14完成、executing5/15，只给A阶段pass；T15冻结裸t/触面tv·tu·tt及必要docs/schema/book、aavm触发面适用taa、实际A合入/规范ledger/index/版本来源consumer读回收据仍pending，tf主检出到期单实例。门红改代码则影响复审/新冻结门；当前pass不覆盖后续实现。未整档/改实施/合入/实际发布/归档/清树，A未交付，原树留B；下方814记录为历史，无新增阻断或批准延期/缩面。

### Plan 814（2026-10-20，R6三项未闭复验，needs_replan；仅余合同授权）

受审 `256613936b6a5e9baea450e5c545899ee7a661dc`，r2/M814-A；[R6报告/显式九档证明/警告复核/精确合同草案](attachments/814/review-r6/review.md)。R3-05 CLOSED：18件清单持久化，工作验证器原删除盲区已修；review另将四条实际迁移固定为显式映射、禁用叶名推断，九档旧ID全部核销且删除/同名替换负控检出，逐ID证据冻结。工作脚本的唯一叶名泛化不单独获“健全”认证；当前映射通过依据为独立strict proof。当前tu 869描述符/867唯一ID与原重复保持，taa编译/list6270与冻结新集合完全相同，未整档。R3-06 CLOSED：五包check过、完整路径/重复数对照原基线新增0/减少7，Drop两单测及跨crate四控制过、fmt/diff check过；不要求清仓旧warning、不延期B。R4-01事实纠正/提案准备CLOSED，但用户批准及正式合同生效仍OPEN：实现保留inventory和trait迁位、主计划仍r2，本次审核不代批。已备具名r3精确草案（上层启动注册轻表、只登记不执行UI业务/I/O；UiParseContext/UiGrammar入轻端口，SDK零UI与旧API/状态保持），未修改canonical/ledger或实际AC。executing1/15保持，T02/T03/T13未勾；用户决定→new正式修订→受影响验收，维持原合同则继续兼容方案，不推定砍API。阶段pass后的merge-owned冻结门/A实际发布/规范来源收据仍pending。未改实施/整档/合入/归档/清树，下方814旧记录为历史。

### Plan 814（2026-10-20，R5未闭问题与inventory合同对账，needs_fix）

受审 `16cc4f14a732f593244f4b60377dd6f1f01bef35`，r2/M814-A；[R5闭合表/合同逐项对账/验证器实测负控/冻结SD](attachments/814/review-r5/review.md)。R3-02身份升级/晚装配旧条目失效、R3-03 SDK实际输入428项禁止项0/无UI源码fresh+locked5/5均CLOSED；host/autodown行为保持CLOSED。R3-05别名包选择与三个代表项遗漏已修且3/3，完整映射证明余项仍OPEN：迁前原始清单随临时树删除，现存exact checker负控删除同名测试仍八档unmatched=0；不声称又发现实际漏测。R3-06 fmt/cfg子项CLOSED，原master基面新增≈14迁移warning未经批准转B，余项OPEN。R4-01(P2)仍OPEN：显式装配有进展，inventory直达兜底与未变r2 AC-02冲突；锁定inventory0.3.25实际用Windows启动构造器注册，提案“无静态构造器”须纠正；旧host行为关闭未授权机制。UiParseContext归属变更已有具名SD，但需正式合同/规范批准，不由复审默批或要求砍旧API。executing1/15，重开T02/T03/T13、保留T01与已修证据；原树work修健康/完整映射，保留语义变化则先new修订批准。记债不授权延期/缩面；阶段pass后merge-owned冻结门、A实际发布/规范ledger/来源收据仍pending。未整档/改实施/合入/归档/清树。下方814旧条目为历史，按本条认当前闭合状态。

### Plan 814（2026-10-20，R4重点复验六项，needs_fix）

受审 `6418f99efe0af2432a652212e98d39b973515cd9`，r2/M814-A；[R4闭合表/原例与相邻组合/独立源码消费](attachments/814/review-r4/review.md)。原host/cache/autodown三探针全过，无UI源码快照新锁+locked SDK5/5；R3-01/04原行为CLOSED，显式默认/快照及autodown往返/错误正控也通过。R3-02仍OPEN：None占位拒升级、晚装配保存误给旧条目改盖新身份；R3-03清单/源码执行子项CLOSED，SDK实际BUILD_INPUTS含UI/Man余项OPEN(P2)；R3-05的86名称与tt已修，但tb等原档选择漏项未闭；R3-06门面/cfg已修，新修复cache/macro片段fmt仍红。新增F814-R4-01(P2)：inventory隐式装配违反未变r2合同、trait迁位/缓存Spec子集未对账。Plan executing1/15，重开T02/T03/T13、保留T01与已修证据；原树work修齐具名边界提交再审。未整档/修实施/合入，后续阶段pass后的merge-owned冻结门/实际发布/规范/ledger收据仍pending。

### Plan 814（2026-10-20，M814-A独立阶段复审R3，needs_fix）

受审 `abe9c99002fe7bafb49a9f86c5867f68d796d65a`，r2；[R3报告/可回放反例与正控/冻结SD](attachments/814/review-r3/review.md)。SDK独立5/5、compiler独立4/4；早期WIP/空包/默认编译UI文法原例已修，不复活。当前 **F814-R3-01..04(P1) OPEN**：旧VM创建再导出入口不再自动装宿主；缓存load后才注入指纹，生成保存null失去升级失效保护；无UI源码快照仍需macros→旧包dev清单及optional文法清单；旧autodown特性build成功但native永久fallback。**R3-05(P2)/06(P3) OPEN**：tt/tb原inline选择漏SDK、新增门面fmt/迁移无效feature告警。均原A合同内回归/未闭不变量，记债不批准延期B。Plan executing1/15，重开T02/T03/T13、保留T01；原树work一次修齐并回放正反控/相邻组合后提交再审。未整档/修实施/合入；阶段pass后的merge-owned冻结门/实际发布/Spec子集/ledger收据仍pending，A不归档不清树。

### Plan 814（2026-10-20，修复复审R2就绪核查，needs_fix）

受审 `edb78b2efbdcadb1994d4065b9cb341b3d71497f`，r2；[R2报告/原始check/覆盖边界](attachments/814/review-r2/review.md)。独立locked/offline SDK+兼容包check退出0，R1-01不可编译子项CLOSED；**F814-R1-01整体仍OPEN**：auto-vm/frame provider、compiler实际迁移与A候选/consumer/发行未交付，frame无效feature门导致固定0且报unexpected_cfgs。**F814-R1-02仍OPEN**：UI文法normal边和默认装配未改，基线§10.3的解释不能覆盖r2明确禁止边。Plan executing1/15保持，T-01保留；原树work修齐并提交完整候选及根因/覆盖调整后复审。未整档/修实施/合入；后续独立pass后的merge-owned冻结门/沉淀/guard仍pending。

### Plan 814（2026-10-20，独立复审R1就绪核查，needs_fix）

受审 `03d983ae9a654d06963b2ad56a04a3adb3bf8e6e`，r2；[报告/AC矩阵/冻结SD](attachments/814/review-r1/review.md)。**F814-R1-01(P1) OPEN**：当前交接明确不可编译WIP，SDK根项级联未完、compiler/runtime只有文档骨架、auto-vm未切SDK、A候选及消费者/资源验证未交付。**F814-R1-02(P1) OPEN**：SDK无条件normal依赖auto-ui-dialect并直接调用完整UI grammar，违反AC-01/08。均属合同内未完成项，不因记债降格；无需重复整套最终验收。Plan保持executing1/15，T-01保留；原树work完成T-02/T-03/T-13及定向证据并提交后申请具名阶段复审，完整最终审查还需B原任务。未修改实施或跑Cargo/整档/实机/合入；后续pass后的merge-owned冻结门与沉淀/guard仍pending。

### Plan 815（2026-10-20，M815-A独立阶段复审R4，pass；当前finding全闭）

受审 `7045e20fa71ba437bc80075f17d9aace755f5abd`，revision2、scope=M815-A/T-13。[R4阶段报告/限定复审与复用清单](attachments/815/review-r4/review.md)。F-815-R3-01独立CLOSED：正确wire直接解码，两条helper构造路径合法控制须成功，六个负控须命中value/value.t并具名规则；14项wire测试及model all-targets零warning/fmt本轮通过。唯一变更wire_tests.rs，生产代码/依赖/配置/其余测试/Spec输入及真实tool二进制与R3相同，复用已核对的其余81项定向及进程/路径/深度/资源/777/闭包证据。R1六项和R2一项保持CLOSED，无新增或延期/缩面。Plan executing5/14，T01/02/03/12/13完成；仅A阶段pass，不代表整Plan reviewed或实际发布。T14 merge-owned冻结裸t/新包协议consumer门、实际合入/精确A规范/受管ledger/祖先与来源收据仍pending，未整档/合入/归档/清树。下方R1-R3为历史，当前无开放finding。

### Plan 815（2026-10-20，M815-A独立阶段复审R3，needs_fix；旧代码finding全闭）

受审 `bc61501c58d45dde28ee2c739425dbc3c911b38c`，revision2，scope=M815-A/T-13。[R3关闭表/原例与相邻控制](attachments/815/review-r3/review.md)。上轮四项F-R1-02/04/05和F-R2-01已独立CLOSED，F-R1-01/03/06保持；精确wire120/frame122、两reader、7组路径双cwd、三位置坏payload后流继续及诊断双侧准入均通过。唯一OPEN=F-815-R3-01(P2测试缺陷)：wire_tests.rs:469/477将helper已返回的完整wire再包入request，六个负控和raw负控均在unknown request key wire处提前失败，换成合法Value也满足任意Shape拒绝断言；须修fixture构造、同路径合法控制和目标分支断言。95个现有定向测试reported pass、健康/777/闭包通过，不能将这条伪覆盖当完成；无已闭实现问题复活。executing2/14，T01/02完成保留，T03/12只待该测试修复；实现/依赖/配置/输入未变时下轮可复用R3代码/规范证据并限定复审，不重启整档。阶段冻结门/SD/ledger/实际发布来源仍pending；未合入/归档/清树。下方R1/R2为历史，当前只此测试项开放，不授权将它转债延期。

### Plan 815（2026-10-20，M815-A独立阶段复审R2，needs_fix）

受审 `faea8c9d727df1ab2b16d45fed95c0f345c48180`，revision2，scope=M815-A/T-13。[R2关闭表/原例与相邻反控/冻结候选](attachments/815/review-r2/review.md)。F-R1-01/03/06独立CLOSED（诊断payload原例与非Unicode路径具名拒绝、数值/版本/真实分数窗口、fmt/零warning）；02/04/05原例修复但完整不变量仍OPEN：缺Value.t在assets/raw/node-props均panic exit101；Windows C:相对路径input/output/front/dep四臂仍依cwd；深度120限制未含完整协议封装且两reader未接线，wire119可编码/完整信封121写拒而真实读端接受。新增F-815-R2-01(P2修复回归)：重复type-mismatch前缀被trim全部删除而误接受，编码端又成功产出自身解码拒绝的未知field。当前四项均合同内P2，不以登记批准延期/缩面。87个不同现有定向测试、fmt/all-targets健康/777 strict及闭包通过，不能覆盖反例；executing1/14，重开T-01预算部分/T-03/T-12，保留T-02和已闭证据。原树work修齐、提交新SHA后复审；merge-owned冻结门/SD/ledger/实际发布及来源收据仍pending，未整档/合入/归档/清树。下方R1为历史，当前状态以R2关闭表为准。

### Plan 815（2026-10-20，M815-A独立阶段审查R1，needs_fix）

受审 `9f27d54f013119a93ca832a2c43d83413592cb69`，revision2、scope=M815-A/T-13，非整Plan final。[完整R1/冻结规范候选/共享反例](attachments/815/review-r1/review.md)。F-815-R1-01(P2)：wire丢失TypeMismatch.field、Windows不可表示路径被lossy改写；02(P2)：嵌套未知/缺失键仍成功，codec drift被吞；03(P2)：byte256→0、f64→f32溢出→inf→报告null、NaN编码成功但解码失败，版本诊断亦截断；04(P2)：input/output相对路径依tool cwd，普通文件工程根仍ok；05(P2)：同源writer写出的64层合法标签数组仅2233字节，reader默认recursion limit拒绝，实际tool exit2；06(P3)：新增测试两条warning、12个新增Rust文件fmt不通过。均为当前合同内OPEN项，登记不授权延期/缩面。现有76定向测试、777 strict/闭包/资源登记及流错误/输出超限对照通过，不能覆盖新反例；T-01（预算部分）/03/12重开，T-02完成保留，executing1/14。原树work修齐、提交后重放再审；B原依赖/能力仍pending，不把它们再造finding。merge-owned裸t/新包协议门、Spec/ledger/实际发布与来源收据仍pending；未整档/合入/归档/清树。

### Plan 796（2026-10-19，独立最终复审R16，pass；当前finding全闭）

受审 `52aad1fd205e05e28b8e7d0345906c8d1ad19f6a`，revision1；[完整R16/冻结旧键迁移及跨进程反控/AC-SD/最终状态](attachments/796/review-r16/review.md)。F-796-R14-01 CLOSED：FnSig新增hdr-sem语义轴，独立使用R14冻结旧键/同旧artifact保留→当前Miss→新键发布/HIT无Conflict→旧页仍在→独立子进程HIT；body-only/签名编辑/fresh对照正确。R13/R12/R11及更早已闭项保持，无新增阻断/延期/缩面。121个不同定向测试、acceptance13行、selftest12控制全通过；压力165.59s、库P95 6.94ms、完整IPC P95 296ms/30样本；fmt/all-targets check通过，计划命名空间零warning。Plan reviewed8/9，T01..08完成，T09 merge-owned冻结裸t/适用tv·tt/独立sema·788影响套件、tf主检出到期判定、SD/ledger/index/归档/合入/guard清理仍pending；未整档/合入，不宣称交付完成。下方796 R15/R14及更早needs_fix条目保留为历史，当前无开放代码finding；后续代码变化不自动继承pass。
### Plan 796（2026-10-19，R15就绪核查，needs_fix；仍为R14-01）

[核查及原例重放](attachments/796/review-r15/review.md)：原树HEAD仍 `1bab7e15ec7e3494df62a86f4695290bee3caa6d`，源码/脚本/依赖/Spec输入同R14，仅三份验收报告变化，无实际修复提交。重新编译原例仍复现保留合法旧FnSig页后读回拒绝、新正确页同键Conflict；fresh正控正常。F-796-R14-01(P2)保持OPEN，无新增finding；不重复整套最终验收。Plan executing2/9保持，原树work提交版本键隔离与旧页保留/重启反控后再复审。此前已闭项不复活，merge-owned门禁/沉淀/归档guard仍pending，未实施/合入。
### Plan 796（2026-10-19，独立最终复审R14，needs_fix）

受审 `1bab7e15ec7e3494df62a86f4695290bee3caa6d`，revision1；[完整报告/旧页版本切换反例/旧例闭合](attachments/796/review-r14/review.md)。R13索引失效/顶层FnMeta版本准入及body-only两项均独立CLOSED，R12/R11原例保持。新F-796-R14-01(P2，修复回归)：FnSig页头由sig+body改为sig，但查询键版本轴沿用原值；保留上轮合法页（同原artifact）时当前读回拒绝，再发布当前正确页同键Conflict，fresh store则正常。须加受影响页语义版本的键隔离，补保留旧页/新发布/重启及body-only/签名编辑控制；不得改Decl逻辑身份、783冲突策略或以清库替代。executing2/9，保留T01/02、重开T03..07；原树work修复提交后受影响面复审。原finding已闭不重标，当前仅R14-01开放；合同内回归不以debt批准延期/缩面。未改实施/合入，merge-owned冻结门禁/规范归档/guard仍pending，120个不同定向测试/acceptance13行通过，压力313.02s、库P95 10.15ms、完整编辑IPC P95 360ms/30样本，未覆盖该版本切换反例。
### Plan 796（2026-10-19，独立最终复审R13，needs_fix）

受审 `d67901b5bf3585ccd914ffbd90f2e1f1a28a9e2d`，revision1；[完整报告/真实索引编辑及FnSig页读回反例/旧例闭合](attachments/796/review-r13/review.md)。R12方法完整准入、7项warning、矩阵v2三项均独立CLOSED。新F-796-R13-01(P2)：ModuleIndex公开生产路径仅按函数候选名构键，函数签名编辑/新增record后仍HIT旧decl_versions/record_names，再正式发布Conflict，fresh store正确；实际发布字段未全部入键/版本/准入。R13-02(P2)：顶层FnMeta当前key/页头携真实旧签名decl.content仍HIT；签名不变仅body修改又在同key误拒绝（target_content错误含body）。D22其余页“全量等价”复核遗漏，非R12方法修复回归；须补各页payload→key/版本→准入/依赖及正反控映射。保留T01/02，重开T03..07，executing2/9；原树work修复提交后受影响面复审。旧已闭证据保持，当前开放仅R13两项；均合同内缺口，不以debt批准延期/缩面。未改实施/合入；merge-owned冻结门禁/规范归档/guard仍pending，118个不同定向测试/acceptance13行通过，压力337.37s、库P95 7.30ms、完整编辑IPC P95 297ms/30样本，不能覆盖当前新例。
### Plan 796（2026-10-19，独立最终复审R12，needs_fix）

受审 `bb10c99235367d79efbf20f4a2998528055f8537`，revision1；[完整报告/新准入矩阵/警告与schema审计/旧例闭合](attachments/796/review-r12/review.md)。R11方法身份碰撞已独立CLOSED。新F-796-R12-01(P2)：方法ref读侧只验kind/record短名，当前key/shape/group下foreign包、foreign模块、未声明方法和真实旧签名版本四模式仍HIT，错record owner则拒绝；须核验完整包/模块/owner/名字/当前签名版本。R12-02(P3)：计划新增缓存测试两文件7项未用/不可达warning（此前漏检），另依赖shim-metadata旧1警告不计。R12-03(P3)：当前CATEGORY_MATRIX的页载体schema仍v1，实际codec/键已v2，须同步并补一致性控制。117个不同定向测试/acceptance13行/selftest12控制通过，压力225.24s、库P95 7.04ms、完整编辑IPC P95 260ms/30样本，不能覆盖新反例。executing2/9，保留T01/T03，重开T02矩阵、T04及依赖T05..07；原树work修复提交后受影响面复审。旧R11/R10闭合保持，当前仅R12三项开放；合同内缺口不以debt批准延期/缩面。未改实施/合入，merge-owned冻结门禁/规范归档/guard仍pending。
### Plan 796（2026-10-19，独立最终复审R11，needs_fix）

受审 `074464f51f587058d5b3124e5d642ec441a912ae`，revision1；[完整报告/新方法owner反例/旧例闭合/AC-SD](attachments/796/review-r11/review.md)。R10完整窗口及raw准入两项均CLOSED；113项定向测试、acceptance13行与selftest12控制通过，库P95 7.39ms、完整编辑IPC P95 319ms/30样本。新增F-796-R11-01(P2)：发布方法FnMeta的DeclRef只含裸方法名，真实A.get/B.get完整ref相等，logical又与同模块顶层get相等。原合同名义身份覆盖遗漏，此前复审漏检，非计时修复回归；须保留方法owner路径并核对canonical/key/content/producer版本，补同名方法/顶层、重启/顺序、签名与body-only控制。Plan executing2/9，T03/04及依赖T05..07重开；保留R10通过证据按修复影响复用。此为当前合同内阻断，不以debt批准延期/缩面；原树work修复提交后复审。旧796条目为历史，当前开放仅R11-01。未改实施/合入，merge-owned冻结门禁/规范归档/guard仍pending。
### Plan 811（2026-10-19，独立复审R7，needs_fix）

受审 `04f7446fdc1d3aed9c44dba6efcc9f661ea6a31d`，revision1；[完整R7/44项绿与原负控调度反例](attachments/811/review-r7/review.md)。F811-R2-02旧正控造载余量CLOSED：有界补入、原门槛及从满态回收保留，独立44/44、正控3/3、test cfg零warning、fmt通过。当前仅新F811-R7-01(P2，测试修复回归)：新负控8项不能保证8槽未满，resize尚未取首项时可全入通道；仅在自有原单测子进程内延迟media-resize调度8s并恢复，任务正常Ready后在peak_backlog8<8错误断言红，exit101，已恢复并回收。不指控生产新故障；生产前缀/依赖/config等价，R6真实消费者与旧生命周期证据明确复用。须将负控预算降至严格小于容量（如7）或先受控确证首项已取走，不改已通过正控/生产实现。Plan executing5/11，保留T02..T06、重开T01/T07/T08；原树提交后受影响复审。最终pass后冻结裸t+tv/并集发行/知识归档guard仍pending；未整档/实施/合入，合同/revision不变。下方R6及更早为历史。

### Plan 811（2026-10-19，独立复审R6，needs_fix）

受审 `2ffb87755e482fb39f01123d28e3f82fa3e373c1`，revision1；[完整R6/初次红与三次单例绿/旧例闭合](attachments/811/review-r6/review.md)。F811-R4-01占用残留/下溢、F811-R5-01新增warning已CLOSED：原在途三例均backlog0/worker0/registry终态，core test cfg零warning；旧重复关闭/事件/准入/scan/Weak保持。当前仅F811-R2-02(P2)满通道fixture未闭：核心初跑42/43，唯一红peak_running=8，后续单例3/3绿；12次submit尝试只要求accepted≥8，入口容量8的合法调度不能保证所需running≥9/真实8槽满，须有界受控造载与明确前提诊断，不以重跑/降低阈值代替。初跑与本轮其他定向编译/小型探针有重叠，未机器独占；无证据认定生产shutdown失败。plan811 7/7、Man3/3、真实HTTP13/13、七窗口、777 strict/format通过。Plan executing5/11，保留T02..T06、重开T01/T07/T08；原树work仅修oracle后提交复审，不重开已闭实施。merge-owned冻结裸t+tv/并集发行/规范归档guard仍pending；未整档/实施/合入，未变合同或批准缩面。下方R5及更早为历史。

### Plan 811（2026-10-19，独立复审R5，needs_fix）

受审 `53f1111a2923a2bcca0096f973eb714a94147a06`，revision1；[完整R5/生产在途关闭反例/旧例闭合](attachments/811/review-r5/review.md)。R4-02重复shutdown完成等待、R3-02同步/async无timer与loaded回调换图已独立CLOSED，原身份/准入/scan/Weak例保持。F811-R4-01(P2)仅空闲下溢旧例闭：真实解码在途shutdown三次均backlog=1、workers=0、pending=0、registry已关；Stop后已接受Work未结算，预留量含阻塞send不能当8槽实际占用oracle。新F811-R5-01(P3)：新增测试unused Ordering导入；核心42/42但1条新增warning。plan811 7/7、Man3/3、真实HTTP13/13、七窗口、777 strict/format通过；work六收据当前SHA匹配，HTTP现存exe变hash已新跑重绑。Plan executing4/11，保留T02/04/05/06，重开T01/03/07/08；原树work修仪表/满通道oracle及warning后提交复审，不重开已闭事件。merge-owned冻结裸t+tv/并集发行/规范归档guard仍pending；本轮未整档/实施/合入，不批准延期或缩面。下方R4及更早为历史。

### Plan 811（2026-10-19，独立复审R4，needs_fix）

受审 `27d3387af988fd7f88b3cc392e043fb98bb3026e`，revision1；[完整R4/原例闭合/核心与七生产窗口反控](attachments/811/review-r4/review.md)。R3-01同URI多handler/换handler已闭，同步无timer与R2准入竞态/scan/Weak原例过；core40/40串行、两文件格式过。F811-R4-01(P2)：新resize_backlog在空闲shutdown后下溢panic，send后增/recv先增且Stop只扣的仪表不准确；R4-02(P2)：新swap门让二次shutdown0ms返回而registry未关闭，新ticket/状态转换仍受理。R3-02仍OPEN：async with-task无timer挂载Ready后loaded0，loaded回调换到Ready B后B0。SD05已在拟议delta保留809最新canonical；work原五JSON仍记4a，与“全绑27”自述不符需等价说明。Plan executing3/11，原树work修计数/完成状态/双入口及回调后调度，提交复审；不批准延期/缩面。未改实施或合入，merge-owned冻结t/tv/最终并集/知识归档guard仍pending。

### Plan 811（2026-10-19，独立复审R3，needs_fix）

受审 `4a1d8fcf3a72fcb6a524b642669ff0340dabcf5d`，revision1；[完整R3/生产Iced窗口/旧例闭合/准确helper反控](attachments/811/review-r3/review.md)。R2-01关闭竞态CLOSED，原顺序/scan/Weak正控过；core38/38、两修改文件格式过。F811-R3-01(P2)：新typed通知只按src去重，真实同URI不同handler loaded A0/B1、error A1/B0，换handler后B0；R3-02(P2)：新merged无独立媒体唤醒，无应用timer挂载Ready图片2s后loaded0，带timer不同URI正控A1/B1。R2-02仍OPEN：准确当前helper100ms及时红但线程/bytes仍活，running≥9仅可推至少6等待，不构成8槽满证明。新事件路径存在但完整合同未闭；VM/Vue/HTTP work收据commit仍33fc、merged4a，不能整体冒记本轮实跑。Plan executing3/11，原树work补身份/无timer/owner回收及准确阶段矩阵并提交复审；不批准延期/缩面。未改实施、整档或合入，merge-owned冻结t/tv/最终并集/知识归档guard仍pending。

### Plan 811（2026-10-19，独立复审R2，needs_fix）

受审 `33fc23208fb6e7b22f4b57b7101efaa553b4c8f1`，revision1；[完整R2/原例闭合/新交错/截止反控/收据核对](attachments/811/review-r2/review.md)。R1顺序关闭原例、三次扫描Drop后代次静止与格式项已通过；core34/34、十文件格式绿。F811-R2-01(P2)：open通过关闭检查后受控暂停，shutdown完成后恢复仍插入有效session、snapshot有效且close=true；R2-02(P2)：新scoped-thread截止超时panic仍join，100ms反控1500ms后才返回，实际满resize/在途oracle仍未建立。F811-R1-03仍OPEN：merged收据明确排除loaded/error且实际一直loading，静止期绑定等合同内缺口仍移交R04。四形态绿数不等于完整AC04；原收据/脚本已冻结，不冒记本轮UI/792实跑。Plan executing3/11，保留T02/04/05、重开T01/03/06/07/08；第二次final未过，下次work须先补诊断与覆盖/设计调整表，原树修复提交后复审。不批准延期/缩面，未改实施或合入；merge-owned冻结t/tv、最终并集/知识归档guard仍pending。

### Plan 811（2026-10-19，独立复审R1，就绪与生命周期核查，needs_fix）

受审 `0f2ab56a991e5c1ca7ac599ba2ca0bd68bf4f488`，revision1；[完整R1/反例/AC-SD映射](attachments/811/review-r1/review.md)。F811-R1-01(P1)：scan/open丢弃线程句柄，shutdown/drop只join三个解码/缩放线程，三次扫描在Drop后仍继续变更registry；R1-02(P2)：关闭后新票据仍能进入Ready、会话仍准入；R1-03(P2)：必需V05真实三形态/渲染后事件及生成媒体HTTP/流输出未完成，却移交R04；R1-04(P3)：六个新增/门面文件scoped格式红。core30/30过，旧票据关闭拒绝/普通Drop释放像素正控过，不能覆盖新例；已有792 uiminimal25项收据已保存，不冒称本轮实跑或媒体事件验收。均为合同内未闭，不批准延期/缩面。Plan executing3/11，保留T02/04/05、重开T01/03/06/07/08；原树work修复、补受控满resize/scan/open/实际消费者oracle与SD交接，提交后复审。未改实施、整档、合入；merge-owned冻结t/tv、最终并集/规范归档guard仍pending。

### Plan 796（2026-10-19，独立最终复审R10，needs_fix）

受审 `a0945653204cbd31622beebd68a61962cf8f2342`，revision1；[报告/旧例闭合/时间边界与raw变异反控制](attachments/796/review-r10/review.md)。R9返回record与长目标名页预算两项已关闭，49签名/7字段独立组合通过，T03/T04验收关闭。F-796-R10-01(P2)：新session计时在base64/JSON/hash之后开始、父侧准入之前结束，不能证明完整编辑端到端门；冷查询可单列，但不能扣当前编辑请求工作。R10-02(P2)：V07对缺失/null/0/负edit时间都PASS，实际1001ms反控FAIL；真实writer正值断言不替代独立raw准入。112项定向测试通过，压力237.30s、库P95 14.49ms/窄edit397ms/混cold总样本2055.39ms；不据混合total断言编辑必超1s。Plan executing4/9，保留T01..04、重开T05..07；原树补完整计时与raw合法性控制提交后复审。均合同内验收覆盖缺口，不批准延期/阈值缩减；merge-owned冻结门禁/规范/归档/guard仍pending，未改实施或合入。

### Plan 808（2026-10-19，独立最终复审R5，pass；旧findings全部关闭）

受审 `b561796b5750bd2799f835c36235de7efd36d7de`，revision1；[完整报告/闭合证据/AC/SD](attachments/808/review-r5/review.md)。R4-01复活路由、R4-02无淘汰旧值、R4-03三条warning、R4-04与807规范调和均CLOSED；R1/R2/R3原反例持续通过，无新增finding。历史下方808 needs_fix条目保留为原轮记录，不代表当前开放债。当前独立真实DLL24/24、core7/7、UI41/41、生成3/3、仅core fresh/locked4/4、完整792交付25/25（fresh182s/relocated170s及负控）、类型/格式/strict均过；包装源码字节与冻结源一致，起止输入稳定。Plan reviewed8/10，T01..08完成，T09/10 merge-owned pending：冻结合流t/tv、最终组合777/受影响792、Spec/ledger/index/归档/资源/guard清理。未合入，不宣称交付完成或R03整包结束。

### Plan 796（2026-10-19，独立最终复审R9，needs_fix）

受审 `f78491fbe47747264d414bfef23299c9a63a5a5a`，revision1；[完整报告/闭合与覆盖/独立反例](attachments/796/review-r9/review.md)。F-796-R9-01(P2)：方法返回record A改B，fn_sig_canonical仅编码r，热store仍record:A、新store为record:B。R9-02(P2)：前置页预算漏实际manifest目标函数名，600001字符名（source约600KB）零拒绝后codec页超限；未证实该例内存超8MiB，不沿用旧P1定性。原长let内存/页例与int/bool方法例已闭；三层/SCC/GC/CT/来源等继续正确，读回控制未越8MiB。完整定向实测/原始日志见报告。均合同内剩余字段覆盖缺口，不批准延期/缩面；executing2/9，重开T03..07，原树修复并补签名/最终页manifest逐字段映射与反控后复审。merge-owned冻结门禁/规范/归档/guard仍pending，未改实施/合入。

### Plan 796（2026-10-19，独立最终复审R8，needs_fix）

受审 `9b3ddec45701777094f3c3db63a5af45b4b85852`，revision1；[报告/AC-SD/原例与组合反例](attachments/796/review-r8/review.md)。F-796-R8-01(P1)：2000长名let查询新增Rust活堆9,263,561bytes、账本5,570,932零拒绝，超8MiB后才codec错误；原20k/30k短名前置拒绝已闭，预算完整不变式仍open。R8-02(P2)：已发布record方法签名不入record_canonical/KeyEnv，int→bool后旧store返回I32、新store返回Bool、有效版本却相同。原三层字段闭包/SCC与fmt/CloseHandle健康项已闭。统一106个不同定向测试通过，压力216.07s、真实1MiB IPC30样本P95 617.09ms；绿测不覆盖当前反例。均合同内缺口，不批准延期/缩面；Plan executing2/9，重开T03..07，原树修复提交并补全部发布facts/真实分配生命周期覆盖表后复审。merge-owned冻结门禁/Spec/ledger/归档/guard全部pending，未改实施/合入。

### Plan 808（2026-10-19，独立最终复审R4，needs_fix）

受审 `9f02a18dc8364cce12e1b62c84a670eb7d679bea`，revision1；[报告/AC/SD/实际反例与控制](attachments/808/review-r4/review.md)。F808-R4-01 P1：采样0后同VM重新安装A并真实spawn83×27，旧死项仍被按Arc移除，复活重加前owner=false/viewport0×0、VM free后A_live=1；移除前同VM与另一VM正控正常。F808-R4-02 P2：新进程retired FIFO限4096，无授权改变冻结旧值；一条真实94×34会话在第4096个受控其他缓存条目后global=None/未预读VM0×0，early/原service仍94×34。F808-R4-03 P3：修复测试:1702/:1754/:1798三条unused_mut。F808-R4-04 P2：当前canonical807已迁diff四入口，SD-03/残边表仍拟写未迁（并发输入更新，非终端运行回归）。均需收口，不批准延期/缩面。R3/R2/R1原反例全过；core7/真实DLL22/UI41/生成3/fresh独立4、类型/格式/strict均过仍不足。Plan executing1/10，保留T04，重开T01/02/03/05/06/07；新core retired契约/边界必须重核，原lease/OS释放正控保留。原树work更新诊断修复提交后再审；merge-owned t/tv、最终组合/792及知识归档/guard仍pending，未改实施/合入。

### Plan 809（2026-10-19，独立最终复审R10，pass，merge-owned pending）

受审390d02acc40b2141293730be7eee2b2f95113ec1，revision1；[R10完整报告/全部finding闭合/最终规范增量](attachments/809/review-r10.md)。最后F809-R2-06 CLOSED：正式driver24/24含fresh锁解析/真实构建、A→B无target重定位locked、负控及dev保持；24相对25仅少可选CLI编译记录。本轮A/B清除开发覆盖后离线locked重放均exit0，4654源文件/12项pin/相同独立锁/14本地metadata路径/资源正负控及main.rs身份独立核对通过。producer绑定当前clean SHA及根锁。R1..R9已登记finding全部CLOSED、无新阻断，旧红轮条目仅保留历史。reviewed9/11；AC01..05 pass，AC06及T10/11 pending。最终SD在Plan/review附件校准真实函数名/sidecar归属/trait兼容等事实，未改实现或发布canonical/ledger。merge须冻结并集跑裸t+必要包/实际触面档、最终777/792正式并集（主检出后续闭包已扩展，不能套旧收据）、规范/ledger/归档/接口SHA/guard清理及批量到期判定。本轮未整档、合入、归档或宣称交付。

### Plan 809（2026-10-19，独立复审R9，needs_fix）

受审 `390d02acc40b2141293730be7eee2b2f95113ec1`，revision1；[R9报告/普通环境原例及产物收据](attachments/809/review-r9.md)。F809-R8-01已CLOSED：INITIAL_CL_IN_PATH=False，原红例单独新进程1/1过，另一新进程原8项+产物控制9/9，Ninja真编译链接并运行main.exe exit0；无外层vcvars准备。生产源码/依赖/锁/测试档配置未变，限定复用此前规则/请求证据，fmt过，无新增代码finding。**唯一剩余F809-R2-06**：正式独立fresh构建、不带target重定位locked及受影响负控收据，最新work仍承认待执行；9项内容门不能替代。executing6/11，T07/08正式验收/证据收口仍待work，当前不具备最终pass；未整档/正式driver/改实现/合入，792原归档保持，merge-owned冻结门/最终并集/知识归档guard仍pending，不批准延期。

### Plan 809（2026-10-19，独立复审R8，needs_fix）

受审 `2568f92d21468294c21b95089d20bde93ac3c0dc`，revision1；[R8完整报告/红轮-环境正控/实际产物收据](attachments/809/review-r8.md)。原R2-04真实Ninja成功产物已CLOSED：完整MSVC环境下Pac/NinjaBuilder真编译链接、9728字节main.exe实际exit0，原8测试+独立产物控制9/9。新F809-R8-01(P2，测试自举回归)：helper敏感匹配PATH=漏掉实际Path=，SDK三键仍使len>=2/空PathBuf误判可用；普通环境原Ninja用例及产物控制均找不到cl而红（7/9），同一binary父侧大小写无关导入vcvars则9/9。需修键匹配与工具准入，非生产Ninja算法缺陷。R2-06正式独立交付fresh/重定位locked/受影响负控仍未补、用户已澄清待执行；9项内容门不可替代。T06效果验收完成，executing6/11，T07/08待work修新helper并补正式收据；无整档/正式driver/实现修改/合入，merge-owned冻结门/最终并集/知识归档guard仍pending，不批准延期。

### Plan 809（2026-10-19，独立复审R7，needs_fix）

受审 `dca409c94000011016be0fd8c5b5ae9a1baa7ff1`，revision1；[R7报告/内容门与真实交付门范围核对](attachments/809/review-r7.md)。R6-01 unused_mut已关闭，唯一代码差异删除mut、逐字等价/输入哈希及fmt通过；无新代码finding，R6原生产不变量限定复用。R2-04真实Ninja成功仍pending；R2-06仍partial/open：新增t06收据9/9是delivery内容单测，end_to_end走legacy入口、未跑Cargo，不能替代原合同正式CLI fresh build/重定位locked及对应负控；权威Spec和R2已明确两门分开，不是新要求。work“唯一余项Ninja”未授信，实际仍两项，须补绑定实际收据并校准handoff。executing5/11、T06/07/08待work；无本轮Cargo测试/整档/正式发行或实现修改/合入，未批准延期；792原归档保持，merge-owned冻结门/最终并集/知识归档guard仍pending。

### Plan 809（2026-10-19，独立复审R6，needs_fix）

受审 `23e94eb74666479789345c5c1b81f8ec6545824b`，revision1；[R6报告/闭合矩阵/原输入与收据核对](attachments/809/review-r6.md)。F809-R4-02/R5 sidecar完整不变量已CLOSED：对象原例旧新均复制文件、属性Node同名/异名head均保留模块+依赖；节点优先与缺源失败传播正负控一致。R2-04真实Ninja成功、R2-06受影响独立交付收据仍OPEN，本轮管理8/8打印成功pending，最新work仍记两项未补，未定位新证据/充分复用依据（不等同断言从未运行，792保持归档）。新F809-R6-01(P3)：rust_ui.rs:3289移除extend后遗留mut，普通库新增unused_mut，须按AC05清理。生成16/管理8/两文件fmt及原生产例+组合控制过；core46/model38/C输入编译按哈希/差异核对限定复用R5，不冒称本轮实跑。SD-02三形态行为冲突已闭，但整体仍needs_fix，executing5/11、T06/07/08待work补告警/两实际验收收据与交接后再审。未整档/正式发行/改实现/合入；merge-owned冻结门/最终并集/知识归档guard仍pending，登记不批准延期。

### Plan 809（2026-10-19，独立复审R5，needs_fix）

受审 `0fba0b604bd81076b8404ffbb9161432acfc0fe3`，revision1；[R5报告/原例/支持域对照/原始日志](attachments/809/review-r5.md)。R4-01编译及Block、R4-03非法C输入、R3-03本工程gen覆盖接线/R3-04死helper告警已闭。**F809-R4-02(P2)仍OPEN**：未改动R4对象输入旧生成有extra.rs、新没有；属性Node同名head正控保留模块/依赖，异名head原core识别但请求生成全漏。根因是Obj属性包临时root后又挂子节点、Node属性改按head选，未还原到core实际读取的根层；新分支仍缺真实产物断言。SD-02已改但“三形态消费”仍不实；R2-04真实Ninja成功及R2-06受影响独立交付收据仍承认pending，不批准延期。core46/model38/管理8（Ninja成功pending）/生成15及五文件fmt过；T03核心修复验证完成，executing5/11，T06/07/08继续work，原树补原例/属性组合产物oracle、根因及准确SD/绑定证据后再审。未整档/正式792/实施修改/合入；792本身已归档，不重开。merge-owned冻结门/并集/知识归档guard仍pending。

### Plan 809（2026-10-19，独立复审R4，needs_fix）

受审 `6a84a7e28f5579b7c188b54b5ca86a9ae295f3a9`，revision1；[R4完整报告/闭合矩阵/原输入与日志](attachments/809/review-r4.md)。F809-R4-01(P2)：core sidecar移除AutoStr导入，必需lib-test三处E0433，原R3-02 Block仍1模块变0；R4-02(P2)：原支持的Obj sidecar在request路径只还原raw_nodes，真实生成旧extra.rs存在、新静默丢失；R4-03(P2)：新Ninja C fixture字符串内实际LF，cl原输入exit2、转义正控exit0。节点来源反转/custom-front已闭，Ninja身份三态等旧例保持；R3-03有效后端gen入口、R3-04新unused、R2-04真实Ninja成功产物、R2-06准确SD/809受影响独立交付绑定证据继续开放。管理8/8（真Cargo/Vue、Ninja明确pending）、生成15/15、七文件fmt过，不抵消core编译红。Plan792已归档8/8、原正式driver25/25，本轮不重开；未定位新收据不等同断言未执行。登记不批准延期；executing4/11，原树work修复提交并校准oracle/SD/闭合表后再审。未改实现/全档/合入，merge-owned冻结门/并集/规范归档guard仍pending。

### Plan 808（2026-10-19，独立最终复审R3，needs_fix）

受审 `69337f5416e5c54fa12850274cab572b81e2b7ea`，revision1；[完整报告/AC/SD/受控组合反例](attachments/808/review-r3/review.md)。F808-R3-01 P1：剪枝mem::take清空列表，锁外live_count期间真实VM无法解析活owner，viewport80×24变0×0且free后A_live=1；恢复列表也未补做free，现有并发测试只要求至少一次查询成功，漏掉失败窗口。F808-R3-02 P2：per-VM retired只复制释放者/提前读取者，跨VM及直接core free的其他已绑定消费者在后续未知柄free清理后从91×31/92×32变0×0；提前读取正控保留值。均为原合同内缺口，不批准延期。R1旧例、R2四路重入8/8和同VM缓存原例闭合、R2新增warnings已清；core6/真实DLL20/UI41/生成3/fresh独立4、双包/no-ui check、fmt/strict均绿仍不足。Plan executing3/10，保留T02/03/04，重开T01/05/06/07；原树work更新并发调用窗口/共享消费者漏检诊断及oracle，修复提交后再审。merge-owned冻结t/tv、最终组合777/受影响792、规范归档/guard仍pending，未改实施/合入。

### Plan 809（2026-10-19，独立复审R3：R2闭合核查，needs_fix）

受审 `ef6f05cf439389d92ccbd9f1ca5ae4206abac962`，revision1；[完整闭合矩阵/原始反例](attachments/809/review-r3.md)。R2-01 P1身份矩阵CLOSED；Ark run/Vue dep二读/front/中文标题/节点原例、PATHEXT真实执行、786围栏原红均闭。完整不变量未全：F809-R3-01(P2) sidecarhelper未接线，request-only不复制、disk-only却复制，both正控正常；R3-02(P2) 迁入sidecar解析漏旧Value::Block，1模块变0；R3-03(P2) 实际源清单忽略custom/front，gen/成员run上下文仍默认；R3-04(P2) 原root_dir及新增死helper告警。R2-04完整三Builder矩阵仍部分，R2-06 SD/正式792收据自认未完，均不批准延期。core46/model38、管理8（真Cargo/Vue）、生成12+新增parity1及19文件fmt通过；未全档/正式发行全集或修改实现。T05复验完成，executing4/11，T03/06..08继续work；更新闭合表区分原例/完整不变量并提交后再审。最终review pass后的merge-owned冻结门、并集/规范归档guard清理仍pending。
### Plan 796（2026-10-19，独立最终复审 R7，needs_fix）

受审 `2e9955635814a6680ffc531d99aa5339629cb50c`，revision1；[完整报告/AC-SD/旧例闭合/独立反例](attachments/796/review-r7/review.md)。F-796-R7-01(P1)：A→B→C仅C编辑，A复用旧required闭包，单root A的GC后A Hit/当前B Miss；形成SCC后仍先读旧singleton。R7-02(P1)：20k局部声明真实查询新增Rust活堆峰值13.20MB、账本仅2.56MB且成功发布，30k达18.58MB、账本3.84MB零拒绝，最后codec超页；meter接线不能证明8/16MiB实际受管。R7-03(P3)：新建reflect/codec fmt仍红，CloseHandle新增声明冲突警告。均原合同内未闭，不批准延期/缩面。统一acceptance100个不同定向测试通过，压力214.08s完成，真实1MiB IPC30样本P95 665.44ms；R6 containment/stale/parse/必需测试身份原例已闭。Plan executing2/9、重开T03..07；下轮work须补有效依赖artifact/SCC与实际分配存活窗设计及组合反控，再提交复审。merge-owned冻结全档/规范沉淀/归档/guard仍pending；未改实施/合入。

### Plan 809（2026-10-18，独立最终复审R2，needs_fix）

受审 `3b733aa5d667f3fb14a0a2b87855de215797fde6`，revision1；[完整报告/闭合矩阵/原始生产反例](attachments/809/review-r2.md)。R2-01(P1)：Ninja按rename回查顶层对象覆盖先访问依赖副本，旧发override.c新发base.c；R2-02(P2)：Ark run不消费req、Jet/Ark build遗留、Vue主工程pac.at二读；R2-03(P2)：请求丢节点/sidecar/app(front)/调用上下文，双front与仅中文标题旧新输出不同；R2-04(P2)：npm.exe探测让可用工具验证提前return算绿，V05完整矩阵未闭；R2-05(P2)：786 source围栏1红与新增warning；R2-06(P2)：SD零遗留/无丢失失实，fresh/locked/792正式证据未全。均合同内OPEN，不批准延期/缩面。core45/model38/consumer各3/777严格/38文件fmt通过；管理7绿含Vue早退，独立真Vue成功及exit3控制通过；generation11绿1红（运行护栏过）。原R1四项仅部分闭合，实质修复仍有身份回归。executing3/11，重开T03/05..08；第二次final未过，下轮work须根因与完整不变量/oracle调整表后再审。无实现修改/全档/合入；merge-owned冻结门、最终并集/规范归档guard pending。
### Plan 808（2026-10-18，独立最终复审 R2，needs_fix）

受审 `69961e0465a65e1e3fec8607194fede9db739f28`，revision1；[完整报告/AC/SD/独立反例](attachments/808/review-r2/review.md)。F808-R2-01 P1：owner 查询/剪枝/槽替换在锁内调用 provider 或析构，四个可重入路径 3 秒不完成、四个直接正控 exit0；F808-R2-02 P2：实际 DLL + 真实 VM shim，free→replace 和 replace→free 后旧 viewport 从80×24/90×30变0×0，原service缓存仍在，违反已冻结旧值规则；F808-R2-03 P3：修复测试 :1005/:1006 新增 unused_mut 两条。均为合同内缺口，登记不批准延期。R1-01..04 原例独立重放已闭，R1-05原位置闭但health不闭；core6/plan808真实13/UI41/生成3/fresh独立4、双包/no-ui check、fmt/strict均绿仍不足以覆盖新反例。Plan executing3/10，保留T02/03/04、重开T01/05/06/07；第二次完整final未过，原树work须根因/覆盖或设计调整表后再送审。merge-owned冻结t/tv、最终组合777/受影响792、知识归档/guard仍pending，未改实施或合入。

### Plan 809（2026-10-18，独立最终复审R1就绪核查，needs_fix）

受审 `aa1d40e1d90b313b7dd6ba57b79123f5278ed90f`，revision1；[完整报告/AC映射/基线](attachments/809/review-r1.md)。F809-R1-01(P2)：真实三Builder受控工具driver/失败验收未实施，Ninja内存断言接受空输出；F809-R1-02(P2)：run等实际生成仍绕回旧读取器，R04延期无批准；F809-R1-03(P2)：Ninja依赖先建/rename去重与下载合并等纯规划规则仍在旧adapter；F809-R1-04(P2)：work-owned 777/792清单/独立消费证据与最终SD/接口交接未收口。均OPEN，登记不批准延期或缩面。按readiness规则退回executing4/11，重开T05/06/07，T08待完成；未运行Rust测试/完整终审，不授信实施绿数。原树work补齐提交后再审；最终pass后的merge-owned冻结裸t及触面档/并集验收/规范归档guard清理仍pending，未改实现或合入。
### Plan 796（2026-10-18，独立最终复审R6，needs_fix）

受审`97c6b95385b7654e990f4e58cb6c19e3a3dd1a96`；[R6报告/闭合矩阵/重放与诊断要求](attachments/796/review-r6/review.md)。R5范围和CT准入已闭，R4调用/来源/事实归属旧例关闭。R6-01..09：组/外部依赖版本漏失效(P1)、复用页漏project保活GC皆丢(P1)、子孙管道使60s期限61.579s仍PASS(P1)、未变报告STALE(P2)、真实预算未接线(P1)、IPC硬门非1MiB局部编辑(P1)、parse语言负例误归infra(P2)、跳过必需CT测试仍PASS(P2)、新增fmt失败(P3)。173项定向执行通过；V06运行400.814s未完成由复审停止自有进程，driver exit1，不计通过也不冒称断言失败。均为合同内未闭/修复回归，不批准延期；executing2/9重开T03..07。第二次完整final失败，下一work交接须根因与不变式覆盖/设计调整表。报告文件入场已脏且测试重写，三份冻结证据保留，Rust头未漂移。merge-owned全档/规范/归档/guard仍pending，未改实现或合入。

### Plan 796（2026-10-18，R5修复交接就绪复核，needs_fix）

受审 `acfad142ec5b9deb3046d4bacf2f0c21d06f0c53`；[R5报告/基线/交接缺口](attachments/796/review-r5/review.md)。本轮有实质修复，但F-796-R4-06自认部分仍未修却列13全闭，CT嵌套Unknown/结果准入仍OPEN；F07仅DAG保活，合法间接递归组发布被未批准列future，完整不变式未闭；F13闭合/规范声明仍矛盾。新增F-796-R5-01(P2)：81文件修复diff带入auto-cache/auto-atom/a2r-std/auto-bindgen/auto命令及无关示例等范围外排版/重排，须精确收窄或给必需映射，不据此断言功能回归。返回executing2/9，T03..07重开及依赖证据待验；未运行Rust测试或完整最终验收，其他R4修复尚未独立授信。均不批准延期，merge-owned门禁/规范沉淀/归档/guard仍pending，未改实现或合入。

### Plan 808（2026-10-18，独立最终复审 R1，needs_fix）

受审 `c346f107790f86b7d1d045331243b3ad7f6e2c7a`，revision1；[完整报告/AC/SD/可重放反例](attachments/808/review-r1/review.md)。F808-R1-01 P1：双活 service 都从句柄1起，provider切换后真实VM free旧柄误释放新服务；F808-R1-02 P1：两VmBridge组件构造漏terminal adapter，独立入口输入泵0/队列残1；F808-R1-03 P1：并发free同柄两次进入DLL，安全Send+Sync服务缺FFI生命周期保护；F808-R1-04 P1：独立service无Drop释放，实际DLL child在owner丢弃后仍活（显式free正控退出，探针仅回收自建PID）；F808-R1-05 P3：新增测试unused import/unused mut两警告。均为合同内开放项，登记不批准延期；原共享service不要求VM drop杀会话。现有core4/plan80810/UI41/生成3/consumer4/fresh独立4绿不覆盖反例；背压MiB语义核对旧完整函数一致，不列缺陷。Plan executing1/10，T01/02/03/05/06/07重开、T04保留；原树work修复提交后复审，merge-owned t/tv、最终组合/792及规范归档清理pending，未改实施或合入。

### Plan 796（2026-10-18，独立最终复审 R4，needs_fix）

受审 `cf736ee08922ea717278bbda41930607e08e58a4`；[完整报告/AC/SD/重放证据](attachments/796/review-r4/review.md)。D-788解除、原接线/文件缺失旧例关闭；首轮完整验收发现 F-796-R4-01..13：调用实参数量漏检(P1)、真实语言错误假绿(P1)、响应后deadline/回收与资源缺口(P1)、worker退出/版本/过期身份未验(P1)、旧source spans命中(P1)、CT嵌套Unknown/模型结果准入(P2)、record必要页图缺失并被GC回收(P1)、预算账本未接真路径(P1)、缓存事实归属漏验(P1)、实际adapter/IPC周转硬门证据缺失(P1)、零测试选择假绿(P2)、新增fmt/warnings(P3)、SD合同不一致(P2)。208项定向测试绿+1既有ignored不覆盖独立反例；均为原合同内开放项，登记不批准延期。Plan回executing 2/9，T03..07重开，工作原树修复后复审；merge-owned全档/规范沉淀/归档/guard仍pending，未改实现/合入。

### Plan 788（2026-10-17，独立最终复审 R10 —— pass，merge-owned pending）

审面 `0732d135295043b5adecbe145b534e78a6ab430c`，**reviewed 9/10**。[完整R10与闭合汇总](attachments/788/review-r10.md)、[实际入口反控](attachments/788/review-r10-r9-replay.log.txt)、[真实选择→执行正控](attachments/788/review-r10-live-tier.log.txt)。R7-01/R9完整ID残留已闭合：wrong-binary/wrong-module同名拒绝、精确命中过、短名兼容显式、root档位不回退。39selftest通过，真实2项/筛选1项小范围端到端通过。当前无Plan内未处理阻断finding；下方R1～R9条目为历史审查记录，不能读作当前未闭合状态。

Rust/锁/配置/工具/raw未变，绑定复用R8性能528/642/4ms与此前限定验收证据；历史红轮保留，不改记pass。仅code/acceptance/Spec审查通过：裸t、触面tv/tt及相关独立完整套件/closeout仍merge-owned pending，门禁失败须归因、改代码须受影响复审。未发布Specs/ledger，未合入/归档/清树，T-10待merge完成。

### Plan 806（2026-10-17，final review R1 就绪核查，blocked）

R4（2026-10-18）：**pass，reviewed 7/8**，受审 `50644ad43c89f8ada86214a5f4ac2a7998b8b494`，R1～R3 finding全部CLOSED、无新增阻断。[完整R4](attachments/806/review-r4.md)。probe成功/失败双路径合同与旧msg一致，推进/非推进失败及重复调用负控已收编，UI导入测试门控闭合；诊断调整表已独立核验。18语法三方（当前/旧臂/无UI）输出一致，light10/driver48/计划18/定向45/生成4、格式/check/轻闭包/777通过；新SHA正式792完整复验25/25（fresh127s/重定位locked116s）。

交付首轮V02一次早退保留红轮（原stderr缺失，不臆造根因）；原B原锁复放136s成功，另一次完整诊断复验25/25。两fresh锁仅redox_syscall0.9.5→0.9.4，未虚称全部外部依赖相同，完整锁和差异均在R4附件。无UI libtest仍有其他模块62基线错，但本计划新增诊断已归零，库无UI构建/解析过。下方红轮仅历史，不能当当前开放项；merge-owned裸t+tv/实际触面档、知识沉淀/归档/guard与批量到期判定仍pending，未合入交付。

R3（2026-10-18）：受审 `e2fc6ebf38e576e1cfb9486fd72cf92f47c5738a`，**needs_fix，executing 2/8**。[R3完整报告](attachments/806/review-r3.md)。R2三原finding已CLOSED：12原输入逐字恢复历史旧臂，权威闭包13/impact/SD05与format已修。新增F806-R3-01(P2，修复引入的接口合同失配)：peek非法字符字面量后失败已消费输入，重复返回None→Ident，原本后续应有的UnknownEscapeSequence消失，违反新API零副作用/幂等/后续记错承诺；不是把旧msg反例重新判红。F806-R3-02(P3建议)：新UI导入测试缺feature门控，给既有无UI libtest红面增加两处E0425；库无UI构建/解析本轮仍过，此项非单独阻断。

light10/driver48/plan806-14/定向45、旧12例、默认及无UI库、format/闭包/777均过；不能代替新增通用端口失败合同。保留T03/T04，原树修复其余；R2/R3两次实际final失败后，下次ready须补漏检诊断及oracle调整表。历史792-25/25未改记新SHA重编pass；无整档/实现修改/合入，merge-owned pending，登记不批准延期。

R2（2026-10-18）：R1的F806-R1-01已CLOSED（实施树/提交/轻包/收据齐备）；受审 `a20fcf6fe9885102d0059c04c9ac1e0551a3e3dc`，**needs_fix，executing 2/8**。[完整R2](attachments/806/review-r2.md)。F806-R2-01(P2)：新前瞻advance跳注释，原拒绝`value /*x*/ : str`现成功；restore不回滚pending_docs/errors，文档注释与同位置词法错误变两份，非仅时序差异。F806-R2-02(P2)：正式交付13受管crate已通过，但权威generated-ui-distribution仍12、impact/SD漏同步。F806-R2-03(P2)：三新增测试文件scoped format未过。保留T03/T04，重开T01/T02/T05/T06；登记不批准延期/文法改变。

轻测9+driver48+生产7+定向45+生成4均通过，独立生产use/VM handler状态1→2及导入坏注解注册完成，无UI构建/解析通过；777 strict0违规，正式792独立25/25（fresh123s/locked重定位113s）。不能覆盖上述反例。未改实现/合入，merge-owned门pending；下方R1仅保留历史，不再代表当前缺实施。

**F806-R1-01（P1，送审交接缺口，非批准延期）**：用户请求检查实施完成，但指定806树/分支和提交均未找到；轻方言crate/consumer缺失，Parser三算法与独立pilot仍在，附件仅只读预检。799/801现均归档，历史前置阻塞已解除。Plan `executing 0/8`，需定位原实施树/代码及work验收收据后复审；未将master旧实现定性成未知提交的回归。[R1报告](attachments/806/review-r1.md)。无Rust修改/测试/合入，merge-owned门禁pending。

### Plan 788（2026-10-17，独立最终复审 R9 —— needs_fix；R7-01全身份分支残留）

审面 `883878daa2346602be778b3a34e25fa0f67eb461`，**executing 7/10**，T-01～07保留，仅T-08重开。[完整R9](attachments/788/review-r9.md)、[实际tier入口反控](attachments/788/review-r9-binding-controls.log.txt)。R8真实选择接线已补齐；R7-01尚剩全ID查找失败后无条件短名回退，wrong-binary/wrong-module的同名测试仍family ok。局部限定完整ID精确匹配并补两项入口负控即可，不新增范围或整档负载。

selftest34绿，真实两项list元数据/既有run绑定正控通过。Rust/工具/配置/raw哈希未变，复用R8性能528/642/4ms及50修订证据。本轮无新Rust测试/整档/merge/归档，merge-owned pending，登记不批准延期。

### Plan 788（2026-10-17，独立最终复审 R8 —— needs_fix；R7-01实际接线残留）

审面 `a030015cb90ec357dad18939a39e7e0158f20e75`，**executing 7/10**，T-01～07保留，仅T-08重开。[完整R8](attachments/788/review-r8.md)、[实际调用反控](attachments/788/review-r8-selection-controls.log.txt)。R7合法筛选与run计数已修，31selftest绿；F788-R7-01的实际预期选中集合绑定未完成：closeout t/tv/tt仍RequiredIds=[]/MinPassed=1，参数级有名单缺项控制可拒，但空名单生产调用形状接受其他binary的单项真实日志且family ok。补真实调用选择/身份绑定，非新增需求、不重开R6已闭合原例。

Rust/测量工具/配置/输入未变；独立复算当前提交raw六场景各30全部ok，hot528ms/body642ms/syntax4ms，50修订正确。性能本轮未重跑，复用理由/哈希见baseline与audit；历史红轮保留，不作确定负载因果宣称。无整档/merge/归档，merge-owned pending，登记不批准延期。

### Plan 805（2026-10-17，独立 final review R1，needs_fix）

R2（2026-10-17）：受审 `589e3039962d0cc8f1cc473d8d8190d9f8083949`，**F-805-R1-01/02/03全部CLOSED，reviewed 7/8**。[R2完整报告](attachments/805/review-r2.md)。核心38+doc6、consumer5、parity/identity2（16原golden+顺序控制）、R1原始反例、scoped格式及闭包守卫、777 strict零违规均过；三核心改动恰为旧blob的rustfmt输出。完整locked metadata相等、上层源码/配置/依赖未漂移，复用R1 ninja/check/792闭包证据；SD语义与真实模块索引经独立重生成核实，无新增阻断。下方R1保留为历史；P805-D1上层旧加载器债务未随抽取修复。merge-owned冻结裸t+必要核心/Man门、规范/ledger/归档/guard清理pending，未合入交付。

受审 `66fc9d71001eab80dd33aacfe74102da565288a5`，**executing 3/8**；[完整报告](attachments/805/review-r1.md)、[顺序反例](attachments/805/review-r1-render-probe.rs)。F-805-R1-01（P2，health）：新增 parity 测试未使用 FlagMappings 导入实际告警，迁入三模块 scoped format 未过。F-805-R1-02（P2，AC04/05、SD01）：Spec“不递归插值”未如实描述顺序替换可继续展开先前插入值，正反方向及等长 key 反例已实跑；需纠正文案和区分性控制，禁止改旧算法迎合新承诺。F-805-R1-03（P3，建议随SD收口）：新索引将验收表误认模块清单，本项不单独决定needs_fix。

核心38+doc6、consumer4、parity/identity2（16 goldens）、ninja12 passed/1 manual ignored、check、777 strict零违规均通过；完整metadata证明无语言闭包及792漏包。保留T01/T03/T05，重开T02/T04/T06，原树修复提交后复审；登记不批准延期或缩面。P805-D1上层旧加载器空载仍为原已登记债务，805仅冻结不漂移；未修实施或合入，merge-owned裸t+核心/Man定向门/规范沉淀/归档/guard清理pending。

### Plan 801（2026-10-17，独立 final review R1，needs_fix）

R2（2026-10-17）：受审 `49512993baced60ca424af4fe49ecb88abc639a4`，**F-801-R1-01/02全部CLOSED，reviewed 7/8**。[R2报告](attachments/801/review-r2.md)。按签发作用域失效，原独立反例12/12+新增三层/兄弟调用组合12/12、consumer48/48通过；独立正式交付25/25（fresh210s、重定位locked222s及资源/缺输入/坏hash/失败保旧等正式负控）通过，SD-04权威闭包12成员及impact补齐。check/strict零违规/改动健康检查通过；无新增阻断项。下方R1记录保留为历史；merge-owned冻结裸t+tv及规范/ledger/归档/guard清理仍pending，未合入交付。

受审 `9f94b32ce60e2db2dfd5d70c67b8ff540a1e5533`，**executing 2/8**；[完整报告](attachments/801/review-r1.md)、[可复放反例](attachments/801/review-r1-snapshot.rs)。F-801-R1-01（P1，AC02/T03/04）：递归内层checkpoint复用外层已消费槽位，退出仅truncate长度水位，漏清内层凭据；Ident/token×Some/None/Err六例均错误接受过期restore并改游标，六个无hole对照正常。F-801-R1-02（P2，AC04/T05/06）：新增792必需闭包只验delivery9/9生成器单测，缺本版fresh/locked重定位与正式缺输入/坏hash验收；SD漏generated-ui-distribution权威闭包11→12同步。

轻consumer3/3、driver36/36、API4+doctest1、delivery9/9、check、闭包、777 strict零违规通过；不能覆盖上述缺口。两项是原合同内开放项，登记不批准延期或缩面。保留T01/02，重开T03..06，原树修复/补正式验收及SD、提交后复审；未修实施或合入，merge-owned裸t+tv/规范沉淀/归档/guard清理pending。

### Plan 788（2026-10-17，独立最终复审 R7 —— needs_fix；含复审计数建议纠正）

审面 `c4272101df190ad8bbc594d85f67f256864cb9e2`，**executing 6/10**，T-06读取问题已闭合；保留T-01/02/03/04/06/07，T-08与T-05待办。[完整R7](attachments/788/review-r7.md)、[真实default-filter结果](attachments/788/review-r7-nextest-default-filter.log.txt)、[包装层判定](attachments/788/review-r7-nextest-default-filter-parser.log.txt)。F788-R7-01：门禁误拒合法筛选排除，计数式run=passed+failed+skipped不符合nextest（run不含筛选排除项），与closeout的t/tv分档冲突。**R5/R6复审对该语义核查不足，现明确纠正；不将此归为R6原例漏修。** required未执行/失败/零测试仍须非零，修复不授权放行所有skip。

R6两项原例通过；本轮11项定向集成10绿1红（body-edit P95=1013ms，hot959ms/syntax3ms），selftest28绿，正常修订50/50产物核对。红轮完整保留；参考环境归因与有效证据待办，不直接定性确定性能回归，不追加挑绿重跑。无整档/merge/归档，merge-owned pending，登记不批准延期。

### Plan 788（2026-10-17，独立最终复审 R6 —— needs_fix）

审面 `3f86b8069dfd4c87380236df555fb02fc9b1c0f5`，**executing 5/10**，保留T-01/02/03/04/07，T-06/08修复、T-05性能红归因待办。[完整报告](attachments/788/review-r6.md)、[实际读错反例](attachments/788/review-r6-probes-results.json.txt)、[真实nextest成功误拒](attachments/788/review-r6-nextest-parser.log.txt)。F788-R6-01/02：syntax实际read_line仍unwrap（3次预热后非法UTF-8使raw仅header）；nextest真实`2 passed (2 slow)`成功摘要被门禁拒绝。均属既定合同内缺口，不批准延期或缩面。

R5三个原反例已修；本轮集成9绿1红，selftest26绿，真实nextest2绿，正常修订50/50产物核对。hot P95=1495ms红轮完整保留（body852ms/syntax2ms）；参考环境/竞争归因待办，不直接断言确定性能回归，不挑重跑绿轮。无整档/merge/归档，merge-owned pending。

### Plan 788（2026-10-17，独立最终复审 R5 —— needs_fix）

审面 `7d4040bf73e3cc72e332745fa109bdf188ffbd85`，**executing 5/10**，保留T-01/02/03/04/07，重开T-05/06/08。[完整报告](attachments/788/review-r5.md)、[实际入口反控](attachments/788/review-r5-probes-results.json.txt)、[nextest包装层传播](attachments/788/review-r5-gate-propagation.log.txt)。F788-R5-01～03：syntax验收自响应取期望hash（83条响应改错仍整体绿，生产runner同输入正确ERROR）；坏JSON在落盘前panic/修订reader解析错continue（实际raw仅header）；nextest不核MinPassed/零测试与守恒，实际Run-CargoFamily控制仍family ok。均为合同内缺口，不批准延期。

R4布尔/后缀、build stale、cap即时终止和旧parser负控及新增告警已修；正常修订50/50产物核对通过。本轮runner13、修复集成8、selftest22通过；hot P95=927ms/body-edit530ms/syntax2ms正常输入硬门通过。无整档/merge/归档，merge-owned pending；正常绿不能覆盖故障注入假绿与丢样本。

### Plan 788（2026-10-17，独立最终复审 R4 —— needs_fix）

审面 `8073d4cdcd35082916c1538147d73b5f09e105b2`，**executing 2/10**，保留T-01/T-07，重开T-02～06/T-08。[完整报告/修复要求](attachments/788/review-r4.md)、[组合反例](attachments/788/review-r4-probes-results.json.txt)、[50产物逐条审计](attachments/788/review-r4-publication-audit.json.txt)。F788-R4-01～06：修订流错误oracle的50个真实响应全部被记recon_ok=true；未变build误STALE；worker stderr超cap未即时停止；门禁错误状态/计数假绿并拒真实nextest成功；失败样本落盘出口仍缺、hot P95本轮2095ms（环境竞争待归因）；新增测试2告警。均为合同内问题，登记不批准延期或缩面。

R3 stdin/解码流/HIR部分产物与check/syntax旧stale例已闭合。本轮worker15+runner13绿，42case/账本通过；R3集成5绿1红，selftest16绿。无整档/完整acceptance/closeout/合并/归档，merge-owned pending；性能红轮保留，需修正确性后归因参考环境，不能挑选绿轮授pass。

### Plan 805（2026-10-17，work 阶段施工期发现登记）

- **P805-D1（P2，预存上游缺口，非 805 引入）**：`CompilerStore::load_from_am_config`
  （crates/auto-man/src/builder/ninja/compiler_store.rs:51）以
  `root.get_kid(&ValueKey::Str("compilers"))` 找 compilers 节点，但
  AutoConfig 解析器对文档形 `compilers { … }` kids 产**位置 Int 键**——
  字符串键查找恒 None，合法 am.at 文档形态下加载器静默空载
  （`compiler` 节点解析/坏 kind warn 跳过路径从未被触达）。PLAN-805
  V04 探针编写期发现（探针实测解析树 kids 键=Int(0)）；base
  `282e50053` 同行为，与抽取无涉（加载算法留上层合同，805 只冻结
  不改）。现行行为已冻结入 golden
  `crates/auto-man/tests/fixtures/plan805/compiler_store.txt`（合法
  文档→空 store Ok / 坏语法→Err / 缺文件→空 store Ok 三态）。修复
  建议：改按节点名迭代（`get_nodes("compilers")`，与 pac.rs port
  解析同型），另立小计划并补正常加载/坏 kind 双向真实用例。

### Plan 802（2026-10-17，work 阶段施工期发现登记）

- **P802-D1（P2，范围外候选）**：VM 单作用域局部变量索引 ≥16（第 17 个
  `let` 起）LOAD_LOCAL 读回垃圾值——纯 int、零 native 参与、无 provider
  路径即可复现（17 局部程序 `print(v16)`→`0`；`print(v15)` 正常）。PLAN-802
  V01 fixture 编写期发现（写族程序 17 局部时 `print(pa)` 得 Int 0），经
  最小探针定性为预存限制（疑 4-bit 局部槽编码上限），worktree 基线
  `1dd123361` 行为一致、与 802 迁移无涉（本计划不改 codegen 局部分配）。
  临时处置：fixture 测试程序拆两段（每段 ≤13 局部）。路由建议：后续
  codegen 局部槽编码专项计划核（LOAD_LOCAL 操作数宽度/寄存器分配上界）。
- V06 条件缺口（非债，环境条件）：live iced 窗口内 renderer 消费链
  （drain finals→operation::scroll_to→可见偏移≤2s 收敛/ScrollStateReader
  心跳）未跑实机；701 读=注册表投影单源契约已由真 controller 注册表
  headless 测试覆盖。补验条件：窗口会话 + `auto run -r vm` 编辑器示例。

### Plan 788（2026-10-17，独立最终复审 R3 —— needs_fix）

审面 `4fcc4bfd9dff06a84a81b62d57db6b4669dd4d08`，**executing 1/10**。保留T-01；T-02～08受影响重开，旧有正控证据保留。
[完整R3/修复要求与R2闭合表](attachments/788/review-r3.md)、[独立组合观察](attachments/788/review-r3-results.json.txt)、[R2原输入重放](attachments/788/review-r3-r2-replay-results.json.txt)。F788-R3-01～07：V13串行重播替代取消/错序发布；性能漏全路径/GC只3条/失败落盘不足；解码后流超cap与worker stderr超cap仍PASS、stdin100ms拖到约1267ms；合法HIR backend负例ERROR；syntax换worker仍FRESH/未变ac-check却STALE；required-ID仍子串；SD/冻结契约未更新。F788-R3-08新增告警。均为合同内问题，登记不批准延期或缩面。

本轮29项定向Rust+8项selftest绿，41-case语料通过；部分R2旧例确已闭合，但不等于全不变量。下次handoff先给调度/生命周期/失败产物/身份与逐族解析的覆盖设计调整；同树修复提交再审。无整档/merge/归档/清树，merge-owned pending。

### Plan 788（2026-10-16，最终审查入口 R2 —— needs_fix）

审面 `230825e78ca2b8eb1d278f2ed8be40941763050a`，**executing 1/10**（T-01保留，T-02～08受影响重开，历史正控不作废）。
[完整R2报告/修复与R1闭合表](attachments/788/review-r2.md)、[独立反例](attachments/788/review-r2-results.json.txt)。F788-R2-01～09：未完成syntax性能/修订流/p15·p16指定语义即execution_done；性能配方不含spawn/父侧发布且伪并发GC、body两内容热命中；流字段错配假绿；负例形状回归；父截止/硬cap仍缺；畸形响应PASS；syntax绕预算；验收器接受999ignored；真实工具身份被缩面。F788-R2-10：新增告警/TSV格式处理。全部合同内问题须work闭合，登记不批准延后或删减。

定向15项绿、当前runner build成功；13项独立观察验证上述关键反例。因入口已有明确未完work，本轮未耗完整最终审查/全档回归，不授pass。D-787已满足，merge-owned门禁全部pending；未改实施代码/merge/归档。

### Plan 787（2026-10-16，final review R8，pass）

受审 `d17b94fecbe61d4f830d6c7b9e318c74548a4f6c`；[R8报告/闭环/证据复用](attachments/787/review-r8/REVIEW.md)。F787-R7-01 CLOSED：单一codec kind表委派、SourcePage真实生产重开后按key/按id均可读；旧类型和未知字节拒绝面保持。独立原反例1、cache10、syntax admission18、compatibility3、probe lib14/admission7/store_boundary5均绿，check/fmt/diff-check通过。无新增finding，下方R1–R7开放状态保留历史。
状态 **reviewed 8/9**，AC01..07 pass，T09/AC08 pending；明确复用R7未受影响的runtime/旧反例/64MiB预算/lease证据（hash与等价路径说明见报告），并读回本提交work acceptance16族raw112日志（过滤重叠）。D-787-01/02既定范围外候选维持。
merge采用R8审核定稿SD（仅校准窗口已物化长度与进程驻留术语，预算/合同不变），完成冻结完整独立语法+783触面套件、Spec/索引/ledger、合流归档/guard清理后才交付；本review没有提交/发布/合并/清理，canonical未改。

### Plan 787（2026-10-16，final review R7，needs_fix）

受审 `8e9c12c99baccc1f3905c798a8ff454d774000aa`，审计基面回绑 `7c0caa18c`；[R7报告/复现/闭环](attachments/787/review-r7/REVIEW.md)。六代旧反例均绿（R1 7/R2 18/R3 8/R4 3/R5 4/R6 4，有重叠；R6缓存错产物已隔离重编并核用例名）；三份驻留/窗口前置限额/主线LEASE_SEQ原项通过，reachability5/5。acceptance含64MiB独立16族全绿/raw passed合计111次（过滤有重叠），预算上界断言通过，见报告。当前 **executing 5/9**，仅重开T06/07，保留T01..05。
F787-R7-01（P2）：SourcePage=10已注册frame codec，store数据库kind映射漏10，真实生产成功工件经公开按key/按id读取均bad kind；A01正控通过。CstView raw chunk源重建另一路仍可工作，不把本项推广成全部源码恢复失效。归787新增codec接入遗漏，须同树补齐/统一映射、真实生产→重开双入口永久oracle后复审，不批准延期；不是783租约修复回归。
check/fmt/diff-check绿；相对回绑基面生产源码零diff。最终pass前仍不合入；merge-owned冻结完整独立语法/783触面套件、知识/合流归档/guard清理pending，无根整档/docs_gen。未修实现或合入。

### Plan 787（2026-10-16，final review R6，needs_fix → 同日 work 修复轮全闭合）

受审 `462b27e5a011160f56b6944189708b110828b08b`；[R6报告/复现/闭环](attachments/787/review-r6/REVIEW.md)。R1 7/7、R2 18/18、R3 8/8、R4 3/3、R5 4/4均绿（有重叠），旧空args/大trivia例闭合；当前 **executing 5/9**，保留T01..05，重开T06/07，不批准延期。
F787-R6-01（P2）：source put同驻留三份只计两份，256KiB页计524288B/实际下界786480B；新计量64MiB work-owned必验被后置merge，旧matrix不能复用；未声称总预算实际超限。F787-R6-02（P2）：窗口先全物化后判2MiB，8MiB名称实际驻留8388623B才拒绝。
F787-R6-03（P1，上游783遗留）：lease id=毫秒+PID，连续申请即释放128次91次SQLite唯一键冲突；787实际syntax_cache_ 7/8通过、1红，单例重试绿。须由783依赖owner在合规worktree/计划内补唯一性后787绑定提交，不能跨计划私修或调用侧sleep遮红。
定向15族14绿/raw passed107次+failed1次（过滤重叠）；check/fmt/diff-check绿。readiness已知必验缺口故未重复长时64MiB整轮final；完整验收尚未通过。merge-owned正确范围=最终pass后完整独立语法/783触面+知识/合流归档/guard收尾，无根整档/docs_gen。未修实现或合入。
**修复轮闭环（work，同日）**：R6-01 ✅ put_peak=capacity+2×len+64（三份：聚合buf+payload克隆+探针framed）；64MiB work-owned 在修复提交上实测（acceptance 16族 raw 111 + 独立复跑：managed 8027KiB≤8MiB、OS各阶段采样峰值17176KiB≤64MiB、源重建阶段已入采样口径）。R6-02 ✅ 限额改 append 前判定（驻留上界=limit+一源页≤256KiB；8MiB名/默认2MiB实测2359296B恰为界内）+永久oracle `syntax_cache_range_query_window_limit_bounds_materialization`（512KiB名+64KiB限，自报物化≤320KiB）。R6-03 ✅ 按归属经独立 `fix-783-lease-id` worktree 修复（LEASE_SEQ 进程序列镜像 STAGING_SEQ；永久回归 `reachability_sequential_same_millisecond_leases_do_not_collide` 128连发全绿；probe 全套件绿）落 master `7c0caa18c`，787 merge 回绑 + 兼容审计基面重锚 7c0caa18c；787 侧无私修/无sleep。六代反例 R1-R6 全绿无空过。矩阵/SD 已按新口径更新；merge-owned 门禁清单已更正（无根整档/docs_gen）。

### Plan 787（2026-10-16，独立 final review R5，needs_fix）

受审 `d2b0ec99ba579547a1506a2babc40b232c3dcda3`；[R5报告/旧例闭环/新反例](attachments/787/review-r5/REVIEW.md)。R1 7/7、R2 18/18、R3 8/8、R4 3/3全部绿（集合重叠）；当前 **needs_fix / executing 5/9**，不批准延期，下方旧轮开放结论保留历史。
F787-R5-01（P1）：新子树准入误拒合法空args，f()恢复失败而f(1)正常；现有136文件/144声明中12次拒绝（7 valid/5 invalid）。F787-R5-02（P1）：合法2MiB+10前导空白使原query程序exit0但空name，进程内精确name=f。F787-R5-03（P2）：受管峰值仍漏返回树/源码页；65556B同型块可见漏树容器22728B及源payload至少65556B，未声称实际超限；OS期间采样改善认可。
acceptance含64MiB 16族全绿、raw passed合计106次（有重叠）。原R4-01/02/03旧例与旧SD文字/格式已闭，R4-04部分闭；本轮修复回归须按合同组合治理。保留T01..05，仅重开T06/07；原树修复永久oracle/计量/SD后再审，merge-owned完整独立套件/沉淀/合流归档/guard收尾pending。未改实现或合入。

### Plan 797（2026-10-20，收尾修复独立复核；PM-03日志入库残项已闭合，四项终态）

主HEAD `cb2194c24`；[本轮报告/真实读回/哈希与留存/具名GC复核](attachments/797/post-merge-recheck-20261020/report.md)。**PM-01 CLOSED**：真实loader exit0/v60/六区1102项，备份逐字对照仅12处u64修复，生产SHA b06ce5fb…前后未变。**PM-02 CLOSED**：R17 128份全入库、127项manifest逐SHA全配。**PM-04 CLOSED（原归档路径/当前状态）**：新导航与archived9/9当前结论已修，历史12引用=9个唯一已录丢失旧工件仍保留，不虚构恢复。**PM-03执行/配置已核验，证据留存残项已闭合**：当前代码/配置与delivery零diff，90项probe完整面及正式61行已绑定，独立重算P95=966/98ms；单项回收正控1/1、并发GC连续2次通过（受限环境os5访问拒绝另列，不充代码反例；历史SQLite锁失败保留，不绝对宣称背靠背根因）。~~仍有4份README引用`.log`被.gitignore:44排除未入库~~ **已闭合（同日补正提交）**：4份执行日志（parent-abi-family-serial/full-serial、auto-test-case、test-cases-tests）改名`.log.txt`入库并同步 README/归档计划 §9 引用，内容字节未变、门禁未重跑（输入未变）；目录内在盘/在库 diff 复核为零缺口，post-merge-recheck 报告目录一并入库。归档终态不回退、历史代码findings不复活。跨计划批量现已有a93e4b922续办收据（tt/tb全绿、tf3红单列known-reds/观察），不再沿用815旧未完成描述。

### Plan 797（2026-10-20，合入后收尾核查；4项已闭合，归档终态保持）

核查时点主HEAD `e918f91d5`，delivery `1799f86ca`为祖先，R17原范围50个非docs文件逐blob一致，canonical已落地、树/分支/组目录清理可见；旧代码findings不复活。[核查报告/真实loader正反控/等价与链接审计](attachments/797/post-merge-audit-20261020/report.md)。**同日收尾对账（HEAD `8d2fda1ee`，代码/依赖/验证配置与 delivery 零 diff）四项全闭**：**PM-01 CLOSED**——live 台账 `.autoos/specs.json` v60 四条 P797 的 12 处 ISO 时间戳→u64 epoch 1792485000（受管单写者手术+备份 .bak-v60-797pm01pre；前检全库非 u64 恰 12 且全在 P797），真实 SpecsStore::load 探针（SHA 555b1744…）复验 **exit0 v60/1102 项**，815 恢复与其余条目零改动（[受管收据](attachments/797/post-merge-audit-20261020/pm01-ledger-fix.json.txt)）。**PM-02 CLOSED**——review-r9..r17 全量+核查证据 703 文件入库（commit 401e9867c，纯文本普查无二进制）；r1/r4/r5/r6 及 r8 辅助工件为 2026-10-20 stash 清除事件已失（计划 §9 簿记在案），保留历史指向。**PM-03 CLOSED**——nextest 放置修正补充独立审查 ACCEPT（断言文件 reviewed→HEAD 零 diff、nextest-full.tf 不排除=批量可见、串行入口=verify 脚本 --test-threads=1 步，[审查](attachments/797/post-merge-audit-20261020/pm03-config-placement-review.md)）+ 冻结交付代码 closeout 全部面重跑收据：probe 套件 9 二进制全绿、正式双硬门串行 P95=966.0/98.0ms、父侧 abi_recipe 4/4（含被排除门本体）+ abi_ 全族 11/11、兼容 auto-test-case 87 + test_cases 26+1ig；脚本级两尝试分别败于计时边界抖动（1189.5ms，样本+并发快照保全）与 SQLite store-open 锁瞬态（该次套件内双门已过），归因在案未删样本（[实录](attachments/797/post-merge-audit-20261020/pm03-closeout-rerun/README.md)）。**PM-04 CLOSED**——归档计划 80 链接搬移改写+specs/plans.md 导航修正（commit 8d2fda1ee）+§10 当前结论改写为合入后状态。**仍开放（跨计划，非 797）**：815 触发批量回归——tf 已绿 6270/6270、tt 3 失败/1 超时待隔离归因、tb 中断未完、新合并窗口（815-A e609caf1a/797 1799f86ca 等）覆盖后写成功收据，归 /auto-plan:regress 主检出单实例（核查已更正收据">48h/814–817 均已合入"表述：815-A 实际落地、815%5=0 到期成立）；台账 tracked HEAD=v57/live v60 投影滞后为 813–817 合入期既有事实（live 真实读取已修复），提交策略未变。

### Plan 797（2026-10-20，R17完整独立复审，pass；当前finding全闭）

受审 `5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e`，revision1；[R17报告/原例与递归组合/冻结SD/AC](attachments/797/review-r17/review.md)。**F-R16-01 CLOSED**：固定扫描根+累计rel构造实际完整路径；原嵌套obj清理后仍hit/producer1，根/一层/三层控制同过，真头新增均miss/PE29=fresh29。R15及更早已闭项保持，无新阻断、缩面或延期。192不同定向最终pass+1既有ignored，210周转样本全ok；正式首轮P95=1010/118ms未过（并发编译可见、因果不作绝对归因），61行与日志保留；结束本轮其他测试后一次受控复测P95=657/106ms、父ABI11/11过，仍不宣称机器独占。**reviewed8/9**，T01–T08完成；T09 merge-owned冻结裸t/独立ABI完整套件与真实C三腿及正式双门/796·788受影响兼容、按diff触面档、tf主检出到期判定、SD/ledger/index/有限profile冻结/归档合入/guard清理仍pending；未整档/改实施/合入。下方R16及更早记录为历史，后续代码变化不自动继承pass。

### Plan 797（2026-10-20，R16 完整复审，needs_fix；旧项全部关闭）

受审 `cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`，revision1；[R16完整报告/AC映射/反例](attachments/797/review-r16/review.md)。R15的F-R4-07、R15-01、R15-02全部CLOSED，错误对象/裸源失败/升级Conflict已修。**唯一新增F-R16-01/P2修复回归**：递归full路径拼接双算prefix，已声明src/sub/sub/value.obj实际未被排除；同输入产物保留时hit/producer1，仅清理该临时obj后却miss/producer2、结果仍17。不是本轮错对象或实测超1s，也不要求所有无关文件变化必须hit；针对明确声明排除的产物，未批准延期。

**executing4/9**，T01–04完成；191定向测试pass+1既有ignored，210样本/正式61行保全，正式P95=722/87ms，内存/判定/错误传播保持关闭。原树work修唯一递归路径构造问题后再审。第十一次完整final未过；未改实现/整档/合入/canonical Spec/live ledger；merge-owned门禁/沉淀/归档/guard仍pending。以下为历史记录。

### Plan 797（2026-10-20，R16 修复轮完成，待 T-08 复审）

R16 复审（受审 `cf9e4c7cd`，needs_fix）唯一开放项 F-R16-01 已修复（修复提交 `5a54f588d`，同树）：清单完整路径改扫描根+累计 rel 锚定（原当前目录+前缀重复拼接使声明嵌套产物永不匹配，清理即误 miss）；反例为永久测试（声明嵌套产物删除后仍 hit、真实头变化仍失效）。probe 88/88、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R15 修复轮完成，待 T-08 复审）

R15 复审（受审 `e00133e62`，needs_fix）三项已全部修复（修复提交 `cf9e4c7cd`，同树）：F-R4-07 产物排除按完整路径+路径级 tmp- 前缀（同名异径头 17→29 反例；顺带修复字面/canonical 大小写漂移）；F-R15-01 裸源路径空 parent 回退 cwd（不再误拒，CWD 锁串行化两个改目录测试）；F-R15-02 信封版本入代际键（同材料旧 v3 永不占 v4 键，升级重编必发布）。probe 87/87、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R15 完整复审，needs_fix）

受审 `e00133e626a3d8a7857fcd0bff21c4b5b101658c`，revision1；[R15完整报告/AC映射/反例](attachments/797/review-r15/review.md)。R14三个旧输入已修，F-R9-01身份错用关闭；**F-R4-07 PARTIAL/OPEN/P1**：产物白名单仅按basename过滤，声明src/value.obj后，src/sub/value.obj合法文本头也被忽略，cache17/fresh29。**新增R15-01/P2修复回归**：直接Path("same.c")空parent导致canonicalize失败、producer0，fresh17；**新增R15-02/P1修复回归**：v4拒旧却不将版本纳入generation，合法旧v3与新v4域相同占同key，实际重编后Internal Conflict。必须分别核对实际产物路径、合法相对锚点和版本发布身份，登记不批准延期/缩面。

**executing4/9**，T01–04完成；188定向测试pass+1既有ignored，210周转样本/正式61行保全，正式P95=642/98ms。F10内存、F09计时/原始行/错误传播及其余旧具体项保持关闭；原树work修三项后再审。第十次完整final未过；未改实现/整档/合入/canonical Spec/live ledger，merge-owned门禁/沉淀/归档/guard仍pending。以下为历史记录。

### Plan 797（2026-10-20，R14 修复轮完成，待 T-08 复审）

R14 复审（受审 `e1088b0ad`，needs_fix）两项 P1 已全部修复（修复提交 `e00133e62`，同树）：F-R4-07 产物过滤精确化（调用方声明排除集+staged-link 命名空间，config.tmp 遮蔽 17→29 反例）+ 信封版本化 v4（旧窄域 v3 永不服务，合法旧形态信封反例）；F-R9-01 重开项以 canonicalize 目录锚点绑定实际解析位置与 cwd（相对字面量跨工程 17/29 反例+每工程 hit 控制）；配套修复 canonical 前缀混用导致的派生漏配。probe 86/86、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R14 完整复审，needs_fix）

受审 `e1088b0adf56906bee7c6ce854347830d429c66e`，revision1；[R14完整报告/AC映射/原始反例](attachments/797/review-r14/review.md)。父路径新建store例修复、1025目录include超过派生上限时不缓存分支通过。**F-R4-07 PARTIAL/OPEN**：实际include的config.tmp文本头因后缀被过滤，新增近端29仍cache17/fresh29；合法旧v3域（按R13算法/原generation恢复）未被升级失效，父路径遮蔽后仍hit17/fresh29。**F-R9-01/P1 REOPEN**：两个cwd使用相同相对src/same.c和include，共享store时B命中A对象17、fresh B29，原绝对路径控制仍通过。无新增ID/规模承诺，登记不批准原合同内延期/缩面；相对路径是原身份不变式遗漏，不声称本提交新引入。

**executing4/9**，T01–04完成；185定向测试pass+1既有ignored，210周转样本/正式61行保全，正式完成后P95=739/114ms。F-R4-10内存、F-R4-09计时/原始行/错误传播及其余旧项保持关闭；原树work修两个缓存不变式后再审。第九次完整final未过；未改实现/整档/合入/canonical Spec/live ledger，merge-owned门禁/沉淀/归档/guard仍pending。以下为历史记录。

### Plan 797（2026-10-20，R13 修复轮完成，待 T-08 复审）

R13 复审（受审 `ae51fea21`，needs_fix，F-R4-10 关闭、进度 4/9）唯一开放项 F-R4-07 已修复（修复提交 `e1088b0ad`，同树）：候选位置全集派生——对全部 (依赖, 解析目录, 链目录) 三元组记录 normalize(D2+rel(P,D1)) 父目录，覆盖父路径逃逸候选（R13 构造 17→29 反例+同输入 hit 控制为永久测试）；派生完备性=真实 spec 必在枚举内；上限 1024 超限保守不缓存；实测真实 consumer 成本可忽略。probe 83/83、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R13 完整复审，needs_fix）

受审 `ae51fea21171ef14ae0630498fab3203318edbce`，revision1；[R13完整报告/AC映射/原始反例](attachments/797/review-r13/review.md)。**F-R4-10 CLOSED**：五表+完整flatten路径成本，原1700名直链/共享DAG均超8MiB前拒、1300名终门拒、900名及小图合法控制通过；T-02闭合。**唯一OPEN/P1仍F-R4-07**：传递头引用 `../shared/dep.h`，新增更近文件不在source/-I/已成功解析依赖父目录的后代树里，仍hit PE17而fresh29。已知依赖目录递归不是全部候选搜索位置；无新增finding ID/规模承诺，登记不批准延期/缩面。

**executing4/9**，T01–04完成；184定向测试pass+1既有ignored，210周转样本全保留、正式完成后硬门P95=683/84ms，F-R4-09及其他旧finding维持关闭。原树work修缓存适用域后提交再审。第八次完整final未过；未改实现/整档/合入/canonical Spec/live ledger；merge-owned门禁/沉淀/归档/guard仍pending。以下为历史交接/审查。

### Plan 797（2026-10-20，R12 修复轮完成，待 T-08 复审）

R12 复审（受审 `634d4d0ab`，needs_fix）2 项组合残项已全部修复（修复提交 `ae51fea21`，同树）：F-R4-07 搜索域清单统一递归（传递头子路径遮蔽 17→29 反例；R11 顶层妥协的前提"命中回归"经实测推翻——依赖父目录集实为 cside 0.1ms）；F-R4-10 第 5 memo 记忆化最长 flatten 路径成本（共享子图完整展开入终门；1300 变体走查过终门拒、900 控制）。probe 82/82、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R12 完整复审，needs_fix）

受审 `634d4d0abed97846f2e1c212104eba3581b47ca8`，revision1；[R12完整报告/AC映射/原始反例](attachments/797/review-r12/review.md)。上轮3项 **F-R4-09 CLOSED**（发布完成后span/门禁消费及失败分类、写失败传播）；**F-R4-07/10 PARTIAL/OPEN**：07原传递头遮蔽已修，但common/outer.h→sub/dep.h新增common/sub/dep.h后仍cache PE17而fresh29；10原1700名直链已拒、小图控制正常，但同1024节点1700名共享后缀DAG接受并实际新增堆峰8,993,472B>8MiB，size memo命中截短链峰不能代表flatten路径。无新增finding ID/规模要求，登记不批准原合同内延期/缩面。

181定向测试pass+1既有ignored，正式完成后硬门P95 body-edit745/reject104ms；自身改动文件无warning、scoped格式/空白过。**executing3/9**，原树work修两项及完整组合oracle后再审。第七次完整final未过；未改实现/整档/合入/canonical Spec/live ledger；merge-owned冻结门禁、规范沉淀/归档/guard清理pending。下列为历史工作交接/审查。

### Plan 797（2026-10-20，R11 修复轮完成，待 T-08 复审）

R11 复审（受审 `26e307386`，needs_fix）3 项残项已全部修复（修复提交 `634d4d0ab`，同树）：F-R4-07 搜索域全集清单（传递头父目录顶层清单；17→29 遮蔽反例；工具链目录顶层化修复命中性能回归）；F-R4-10 两阶段峰值预算（flatten 阶段全量 memo+seen 显式入终门；1700 字节名链拒绝+800 控制）；F-R4-09 落盘完成后关闭 span（门禁消费 post-persist span；592/641/644 与 90/121ms 收据绑定）。probe 80/80、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R11 完整复审，needs_fix）

受审 `26e307386661498d821af7fffba3a243cc218d79`，revision1；[R11完整报告/AC映射/原始反例](attachments/797/review-r11/review.md)。上轮4项 **R10-01 CLOSED**；**R4-07/09/10 PARTIAL/OPEN**（无新增ID）：07旧嵌套/无扩展名修复，但common传递头新增近端dep.h后仍cache PE17而fresh29；10旧超限链和小图修复，但1700字节名1024节点合法链被接受，新增堆峰9,018,591B>8MiB（输入/栈不计）；09失败分类false/true和写失败ERROR均已闭合，既有发布计时残项P2仍在——span/request_ms在append前取值，未覆盖收据声称的落盘完成，不冒称已测超1s。

179定向测试pass+1既有ignored；自身改动文件无warning/scoped格式/空白过。**executing3/9**，T-01/T-03/T-04完成，原树work修三残项及完整不变式oracle后再审。第六次完整final未过，登记不批准合同内延期/缩面；未改实现/整档/合入/canonical Spec/live ledger，merge-owned冻结门禁/沉淀/归档/guard清理pending。下列为历史工作交接/审查，不能覆盖本轮独立结论。

### Plan 797（2026-10-20，R10 修复轮完成，待 T-08 复审）

R10 复审（受审 `3bcc41954`，needs_fix）4 项开放已全部修复（修复提交 `26e307386`，同树）：F-R4-07 递归全量负依赖清单（嵌套/无扩展名遮蔽反例）；F-R10-01 适用域整体入代际键（unused.h 合法重编与无 include 跨工程两死端反例变绿）；F-R4-09 原始行 ok=本请求判定结果+写失败传播（父侧负控：错期望→FAIL 行 ok=false 含失配文本）；F-R4-10 分配前增量预算记账（2000 字节名链超限前拒、单记录长名小图照常计算）。probe 78/78、父侧 13/13、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R10 完整复审，needs_fix）

受审 `3bcc4195431bf17f493ff0d40487b93c6bf8efe5`，revision1；[R10完整报告/AC映射/真实C与分配反例](attachments/797/review-r10/review.md)。上轮5项关闭2项：R9-01原身份错用、R4-12自身告警；**R4-07/09/10仍PARTIAL/OPEN**：嵌套/无扩展名include遮蔽仍cache17而fresh29；真实验收FAIL原始行却ok:true/error:null；原内存例拒绝和实际GC回收通过，但近邻合法链先超8MiB再拒、小图被最大深度估算误拒。**新增F-797-R10-01/P1修复回归**：完整v2适用域未进入generation key，新增无关头/无include源跨目录合法重编产生Internal Conflict。共4项合同内OPEN，登记不批准延期/缩面。

本轮173定向测试pass+1既有ignored，自身warning/scoped格式/空白过；实际错误PE/记录/超预算/合法重编失败不能由总数抵销。**executing3/9**，T-01/T-03/T-04完成，T-02/T-05/T-06/T-07仍OPEN，next work同树修4项与完整不变式oracle后再审。第五次完整final未过；未改实现/整档/合入/canonical Spec/ledger。merge-owned冻结门禁、规范沉淀/归档/guard仍pending。以下为历史交接/审查。

#### 前轮记录（历史）

R9 复审（受审 `34db55429`，needs_fix）5 项开放已全部修复（修复提交 `3bcc41954`，同树）：F-R9-01 缓存适用域信封 v2（源/include 目录·target·工具链·flags·负依赖清单逐项重验；跨工程错对象 PE17→29 与异工具链合法代反例+同输入复用控制）；F-R4-07 遮蔽三态（新增近端头 17→29 miss、新内容 31 miss、删除=环境复原正确复用历史代）；F-R4-09 计时边界到 typed 判定+原始行落盘（一请求一行、失败/infra 含、edit_ms 入样本、Job 进程+树峰值与 I/O；门 536/608/609 与 65/91ms 收据绑定）；F-R4-10 memo/索引/递归态入 8MiB 门（R9 长记录名链反例分配前拒绝）+ GC 收据入 meter 且真实回收有 rooted 存活反例；F-R4-12 自身告警清零。probe 73/73、父侧 12/12、verify closeout ALL PASS；merge-owned 门禁/沉淀/归档 guard 仍 pending。无新增债务延期项。

### Plan 797（2026-10-20，R9 完整复审，needs_fix）

受审 `34db55429c134fe0fd8cfaf502f606b956ef65c7`，revision1；[R9完整报告/AC映射/真实PE及实际分配反例](attachments/797/review-r9/review.md)。R4-02本轮CLOSED（descriptor/3多out/handle/callback/header独立oracle及record mut/out原生控制）。**仍OPEN四项**：R4-07/P1新近端header遮蔽未失效（cache PE17，fresh29）；R4-09/P1父IPC计时改善但仍排除最终typed判断/发布，31body全返后才落盘、失败/资源行不齐；R4-10/P1原长字段名例已拒，但合法1024节点链memo/index新增堆峰值10,298,591B>8MiB仍被接受，force-GC测试缺实际ran/deleted/live断言；R4-12/P2旧三warning清除，本轮新增sema.rs peak_job_memory_used dead_code与append_jsonl unused_mut仍未处理。**新增F-797-R9-01/P1修复回归**：C generation发现忽略源码目录及当前target/toolchain/flags，跨工程同名同内容不同头实际hit错对象（17应29），合法foreign-target/toolchain key也被接受。共5项合同内OPEN，登记不批准延期/缩面。

170定向测试pass+1既有ignored（root abi_包含2个已有FFI测试，非新覆盖）；实际错误PE与超预算独立反例不可由总数抵销。**executing3/9**，T-01/T-03保留、T-04完成，T-02/T-05/T-06/T-07仍OPEN，next work同树修5项/完整不变式oracle后再审。第四次完整final未过；未改实现/整档/合入/canonical Spec/live ledger，merge-owned冻结门禁、规范沉淀/归档/guard清理仍pending。旧R8工件丢失事件已知，本轮重新构造实跑并保存R9，不虚称复用丢失日志。以下R8及更早为历史。

#### R8 修复轮（历史交接）

R8 复审（受审 `bf2455758`，needs_fix）5 项开放 findings 已全部修复（修复提交 `34db55429`，同树 `D:/autostack/.wt/lang-797/auto-lang`）：F-R4-02 descriptor/3 如实化（ledger 三出参/two_step handle 出参/callback ctx/buf:i64 元素视图 + record mut/out 生产面 `p797_crec_mutate`/`p797_crec_make` 含字节级 untouched oracle 与期望翻转负控；oracle 改为 header 文本独立对照）；F-R4-07 `cl -showIncludes` 观测依赖闭包（本地+系统头）入同源多代闭包键、命中经公共 db 句柄逐一重验零编译调用（R8 三兄弟+宏 include 真编译 PE 17→29 永久反例）；F-R4-09 每请求完整父路径计时（edit 父侧应用，query 请求携带编辑后 deck；Job 对象峰值内存/I-O 入逐样本先落盘 JSONL；门禁 P95=663/101ms，verify closeout 收据绑定）；F-R4-10 物化表示精确字节暂存门（原 R5 长字段名输入 19,398,656B 下界在分配前拒绝）+ worker 预算前后双查 + 并发 GC 全链（该测试抓到并修复同 store 并发 worker scratch 碰撞真缺陷）；F-R4-12 本计划 3 条告警清零 + freeze/verify/SD/支持矩阵按 R8 事实重写。probe 68/68、父侧 abi 家族 12/12、verify closeout ALL PASS；merge-owned 门禁/合入/沉淀/归档 guard 仍 pending。无新增债务延期项。

**跨计划事件登记（2026-10-20，P1，待用户裁定）**：主检出在 PLAN-813 合并前后被 stash 化（`stash@{0}`=`c1c0fb06f`）且未跟踪复审工件目录被清——`docs/plans/attachments/{797,781,785,788,796,798…}/review-r*/` 中**未跟踪**的复审报告/基线/raw 辅助工件已从磁盘消失（tracked 文件在 stash 中保全）。797 的 review-r8/review.md 已按复审会话原文重建；r3–r7 及其他计划的丢失件未恢复，§9/§10 中相应链接为历史指向。各受影响计划复审前须核对其证据链完整性；后续建议复审工件一律入库（tracked）或集中保全，stash/重置操作前先清点未跟踪计划资产。

### Plan 797（2026-10-19，R6 就绪核查，维持 needs_fix）

R6：原树仍是R5受审 `5897eeb129e657a9d859de6d585434eec3f98f2a`、clean，无新提交/未提交修复；依赖/配置/锁/规范/二进制及R5证据hash全部一致，已知refs未见其他797 R5/R6修复。[R6报告/比对基线](attachments/797/review-r6/review.md)。维持 **executing 2/9**、R4七项与R5三项OPEN，原关闭五项维持；不重跑同一代码的测试，不计第三次完整final失败。已请求修复commit/树位置，实际交接（含根因/覆盖调整）到位后再审；merge-owned门禁/沉淀/归档guard仍pending。以下R5及更早为历史结论。


受审 `5897eeb129e657a9d859de6d585434eec3f98f2a`（revision1），原树clean；[R5报告/旧例闭环/独立组合/原始证据](attachments/797/review-r5/review.md)。当前 **executing 2/9**，保留T-01/T-03，T-02/T-04/T-05/T-06/T-07重开。R4原12项关闭5（04/05/06/08/11）；01/02/03/07/09/10/12仍PARTIAL/OPEN：ref未绑定实际同宽字段、生产descriptor仍丢view/ownership、资源组合与infra分类错、有效include漏闭包、父性能门跑A腿而非C消费复用、暂存payload误作内存/永久current roots、root健康/SD漂移。新增F-797-R5-01/P1：slot满失败create预先吃arena，释放全部合法handle后容量仍漏（C exit44）；R5-02/P1：代际16位/slot32位不一致，第65536代“成功”句柄立即失效（exit45）；R5-03/P1：真实worker读损坏ABI-L0块仍expect panic exit101。前两项为修复回归，第三项为新发现的原合同真实路径缺口，均OPEN。35个定向测试绿；真实反例含小图返回19,398,656B元数据越8MiB、4版本4个current roots、溢出后仍运行超过外部4s、cl有效include同源运行17→29而键不变、保留ref改字段仍admit。第二轮完整final失败，下次交接须先根因/覆盖调整和完整修复，不只报告绿总数。未改实现/合入/规范ledger，未整档；merge-owned冻结门/沉淀/归档guard仍pending。下方R4及更早为历史记录，登记不批准延期。


受审 `8679859e43f9eb14450cd9ca4225cda5dab42511`（revision1），D-796/D-788已交付，树clean；[R4完整报告/反例/哈希与原始结果](attachments/797/review-r4/review.md)。当前 **needs_fix / executing 2/9**，保留T-01/T-03，重开T-02/T-04/T-05/T-06/T-07。新增12项（8 P1/4 P2）：F-797-R4-01声明/同宽语义/record未贯穿且整数溢出panic；02公共profile/context/ownership/UTF-8等声明大于实证；03响应后deadline失效、迟到stderr超限/矛盾信封可PASS；04负索引可改写邻分配；05跨provider句柄碰撞；06destroy不回收arena；07C缓存漏header闭包；08hit仍隐藏emit/漏计；09body-edit采样大部分hot、未走正式父路径/原始样本不足；10预算/峰值/GC/lease未全路径接线（新增oversize测得约911MiB工作集、无隔离）；11混合recipe stale身份缓存顺序相关；12格式/新增测试告警和verify收口不实。均为既定合同内缺口，登记不批准延期。30个既有定向测试绿、独立正负控制区分成功；未跑整档、未修实现/合入/改canonical规范。F-R1-03保持CLOSED，F-R1-01/02仅原依赖/库/文件/锁/快照退出子项闭，完整验收仍PARTIAL/OPEN。原树修复/提交后再审，merge-owned冻结回归/完整ABI套件/沉淀/归档/guard清理全部pending。以下R3/R2/R1及工作轮文字为历史记录。


R3（2026-10-16）：实施 HEAD 仍 `635053f74aa49f89d74ffef6db9c70df7c363c06`、树 clean，较 R2 无任何新增实施提交。当前 master `f35c9aab437b564716988977027a00e36b16f9f7` 上 D-796/D-788 ancestry 均 exit1；T-06/T-07 未完成，维持 **needs_fix / executing 5/9**。F01 OPEN、F02 PARTIAL/OPEN、F03 CLOSED；锁及 Spec 草稿身份与 R2 一致，复用 R2 证据、不重复测试、不计为第三轮完整代码验收。详见 Plan §9 R3；依赖交付后原树完成 T-06/T-07，再最终验收，merge-owned 收尾仍 pending。

R2 独立就绪复核 `635053f74aa49f89d74ffef6db9c70df7c363c06`：**needs_fix / executing 5/9**，[R2报告](attachments/797/review-r2/review.md)。F-797-R1-03 独立确认 CLOSED（HEAD blob=实际锁，locked/offline metadata 在跳过本机 sccache 端口阻断后 exit0、120 packages）；F-797-R1-02 文件缺失子项关闭，T-07 终稿/V-01–11 仍 PARTIAL/OPEN；F-797-R1-01 OPEN，T-06 实际语义绑定/正式 AutoDown/性能复测尚未实施，D-796/D-788 未满足。当前 goals Spec 输入已变，终稿须重绑。没有 Rust/native 测试、没有完整 final pass；原树完成 T-06/T-07 后再审，merge-owned 门禁/收尾仍 pending。以下为 R1/工作轮历史。

受审 `defcc8eaa1fdcfa619feda99721d0b93c25035fb`；[报告](attachments/797/review-r1/review.md)。R1 后 work 修复轮 commit `635053f74`。当前 **executing 5/9**。**F-797-R1-03 已关闭**：Cargo.lock 强制入提交（blob `598833812a`，`--locked --offline` 全绿）。**F-797-R1-02 部分关闭**：verify/freeze/support-matrix/SD 草稿/并行审计已交，V-01–11 终收口与 hash 冻结仍待 T-06。**F-797-R1-01 仍打开（P1，AC-05/06，T-06）**：仍使用本地 semref snapshot，case 明示正式 AutoDown execution pending，D-796/D-788 尚未满足；真实合同/正式 runner/性能复测未完成。T-02–05 历史测试证据在锁一致后允许复用，但不得自动作为 final pass。

这些是原合同内完成性/提交完整性缺口，登记不批准延期。可独立项已补交；依赖交付后完成 T-06/T-07 终收口，再申请完整 final review。修复轮仅跑 probe 定向 `--locked --offline` 与 verify acceptance/closeout，无整档/合入；merge-owned 门禁与归档/清理仍 pending。

### Plan 794（2026-10-16，独立 final review R1，needs_fix）

R2（2026-10-16）复核 `d027e5f617a3d33bb8b1897779569020f886f0e2`：**F-794-R1-01/02/03全部CLOSED**，下方R1开放记录保留为历史。[R2报告](attachments/794/review-r2/review.md)。strict pass/0违例/2既有窄grandfather、定向9/9零本批文件告警；旧oracle反例现在正确拒绝，独立6控制覆盖pending多次查询与成功/失败出口，真实VM60/60保留旧57项；consumer5/5、源码/32轮ownership2/2重放通过；guard clean。生产代码/依赖/测试配置/Vue探针与CLI未变且hash一致，明确复用R1 Vue59/59。无新增finding，Plan=reviewed 7/8；merge-owned裸t+tv、规范/ledger/合流/归档/guard清理仍pending，未合入交付。

受审 `91974d8f8d826a9cdb3ea6a50038a518bc02c4e9`；[完整报告与反例](attachments/794/review-r1/review.md)。F-794-R1-01（P2，T05/06）：Cargo.toml改动令777 strict因stale_baseline退出2；须审查变化后恢复有效冻结基线，不放宽规则。F-794-R1-02（P2，T05）：原VM探针expect_reply只press一次后读缓存标签，确定性负控pressed=False+旧applied误绿；实机原跑56/57，换绑旧key粘住pending。重新执行只读查询的独立控制组57/57，不能替代修复指定oracle。F-794-R1-03（P3，T06）：新增plan794测试126行unused_mut，与零新增告警交接不符。均为合同内开放项，登记不批准延期。

定向9/9（含计划6例）、no-ui consumer5/5、独立八native源码/32轮heap与pool配平2/2、隔离Vue59/59通过；未发现滚动生产逻辑回归。Plan回executing 4/8，保留T01..04、重开T05/06，同树修复提交后复审。merge-owned裸t+tv与沉淀/归档/清理pending；guard实跑exit1，p656/p781生成node_modules含junction，须按实际target安全摘链接且guard clean前禁止移除工作树。未修实施代码/合入。

### Plan 796（2026-10-16，final 就绪核查 R1，needs_fix）

R3（2026-10-16）复核 `d85f89fc7584d40146d026e3c79fe0db18b2a78e`：新增验收脚本/boundary/spec-delta，F-796-R1-02 文件缺失子项关闭，完整收口仍 partial/OPEN（V-08、SD-07/08 pending）；F-796-R1-01 OPEN（D-788/T-06 未完成）。脚本 `$d788` 将无输出成功命令参与布尔判断导致误判，归入 F02 待收口修正。维持 needs_fix / executing 5/9；就绪核查而非完整代码验收，未运行 Rust 测试。详情见 Plan §9 R3。

R2（2026-10-16）复核 `5a8cd61ba8621abe4a755e2fa05c5d808b4afcde`：仅补交 contract-decision.md 的 called-sigs-fp 文档，未新增实现；F-796-R1-01/02 均 OPEN，D-788 未满足，验收脚本/Spec delta 仍缺。维持 needs_fix / executing 5/9；未进入完整验收、未运行测试。详见同一 Plan §9 的 R2 记录；不是第二次完整代码验收失败。

受审 `91b17357782c2ba2e93d93e973a2cccc82557470`；[就绪核查记录](796-ac-semantic-metadata-inference-probe.md#9-复审记录)。F-796-R1-01（P1，AC-05/T-06）：D-788 尚未满足，正式 AutoDown adapter/recipe/统一 runner 执行未完成，case 仍 not-executed。F-796-R1-02（P2，AC-06/T-07）：统一验收脚本、V-01–09 收口与 Spec delta 未交付。均为原合同内未完成项，登记不批准延期；保持 executing 5/9，先完成 work 再最终复审。本轮未进入完整代码验收，未复验前五项或运行测试，merge-owned 门禁及收尾全部 pending。

### Plan 788（2026-10-16，独立阶段审查 T-01～T-05 R1）

`54e6a46caf1badddaba898a471492ae52e7a1fbf`，**needs_fix / executing 1/10**。保留 T-01；重开 T-02～T-05。
[完整 findings 与修复要求](attachments/788/review-r1.md)、[独立结果](attachments/788/review-r1-results.json.txt)。F788-R1-01～10：消费者无界执行/ERROR 假绿、IPC 版本与结果/生命周期未核、manifest 越界/长度/cap 缺口、build/total 预算不执行、native JSON 非法且版本错误、stale 与缓存实际身份缺失、backend 分类及 trace/计数不实。F788-R1-11：新增 unused imports/自赋值告警。上述是本合同内未闭合项，债务登记不批准延期或缩面。

63 项定向测试通过，不能覆盖独立反例；原树 clean、未修实施代码、未跑整档/合入。T-06～T-08 与最终 review/merge 仍 pending；同树全部修复后提交再审。

### Plan 787（2026-10-16，独立 final review R4，needs_fix）

受审 `15cfeac977a3a87b35b18aa92e56bfe9097b7174`；[R4报告与全部反例](attachments/787/review-r4/REVIEW.md)。原 R1 7/7、R2 18/18、R3 8/8 重放均绿；当前判定 **needs_fix / executing 5/9**，下方 R1–R3 为历史，不批准延期。
F787-R4-01..03（P1）：同 snapshot fresh/增量树真实 A03 入库布局键冲突；有效 frame 的越源 child 经 decl_subtree 未拒绝；合法跨页前导空白令实际查询 bin origin 重定位下溢（exit101）。三个新独立集成反例全部失败，均含正控。
F787-R4-04（P2）：受管同时驻留/容量漏计与 OS 返回点采样不能证明全阶段峰值，未声称实测超限；认可 manifest 实读分列/查询增量口径。F787-R4-05（P2）：SD仍描述已删除的manifest revision和4096目录配置。F787-R4-06（P3）：新改面一处rustfmt多空行。
保留T01..05，仅重开T06/07；原树修复机制/永久集成oracle/计量证据/规范稿后提交再审。完整 acceptance（含64MiB）16族全绿、raw passed合计99次（有重叠），见报告；merge-owned完整独立套件/沉淀/合流归档/guard收尾pending。未修实现或合入。

### Plan 793 final review R2（2026-10-16，pass）

受审 `a8faf1e311debb96df0c3ed660b7d7ccbb0e23a4`；[R2报告](attachments/793/review-r2.md)。F-793-R1-01..04全部CLOSED：提交内golden与旧臂冻结件字节一致、38例重放通过；真实Type/E0106与Name/E0201及有效控制纳入互操作7/7；六文件fmt/diff-check绿；SD03补auto-lex归属同步、SD02路径修正。独立consumer5/5、strict零违规、闭包PASS；生产代码/依赖/测试配置未变，明确复用R1定向error_tests4/error_spans8证据。无新增finding，R1下方开放状态保留为历史。Plan=reviewed 7/8；merge-owned裸cargo t+tv、新crate/consumer、SD/ledger/归档/guard清理与792互链pending，尚未合入交付。

### Plan 793 final review R1（2026-10-16，needs_fix）

受审 `1bb5660b44d7835617fd88aa0dfbf7ee433b280e`；[R1报告](attachments/793/review-r1.md)。F-793-R1-01（P1）：V02 golden受`*.json`忽略、未入提交，本机38例绿依赖额外文件，干净检出缺oracle；F-793-R1-02（P2）：真实type/infer用例实际因`let x: int`落Syntax/E0099，未覆盖合同V03真实type阶段；F-793-R1-03（P3）：新error门面未过rustfmt；F-793-R1-04（P2）：SD漏同步auto-lex现行From/attach/Name残留归属，且SD02门面路径错。以上为合同内未闭项，不批准转为延期债。独立consumer各5绿，strict零违规，定向error_tests4/error_spans8绿，逐字搬迁仅别名及注释变化，未发现新增诊断行为回归。Plan回executing 2/8，保留T01/02、重开T03..06；原树修复提交再审。merge-owned裸t+tv、知识沉淀/归档/guard清理pending；未改实现/合入。

### Plan 795 final review R2（2026-10-16，pass）

受审 `edf982e52902e0b3b8d18c1f1366bab98dd1f161`；[R2报告](attachments/795/review-r2/review.md)。F-795-R1-01关闭：修复只更新777基线，独立strict=pass/0 violations/2 grandfathered，metadata45边零差异；规则与豁免未放宽。实现/依赖/测试配置/SD均未变，SHA256复核一致，明确复用R1行为38项+fresh4组与闭包/fmt证据。无新增finding；R1开放结论仅保留历史。Plan=reviewed 7/8；merge-owned裸cargo t+定向验收、规范/ledger/归档/guard清理及795批到期交接仍pending，未宣称交付完成。

### Plan 795 final review R1（2026-10-16，needs_fix）

受审 `11ddc162ecd839b98cee748e3d2c27cfd05df301`；[R1报告](attachments/795/review-r1/review.md)。F-795-R1-01（P2，AC-04/05，T-05/06）：777 strict 实跑 exit1，config_drift_since_baseline + stale_baseline 两红；新增两声明边合法、metadata45边零差异，但新crate/根members/auto-man依赖/域配置未重绑冻结基线。须原树审查并显式刷新、不放宽规则或grandfather，提交后strict复绿再复审；不能延期到merge。行为验收38项+fresh独立4组均pass，fmt/diff-check/闭包通过。Plan回executing 4/8，重开T-05/06；merge-owned整档与收尾pending。旧弱verify及Man未迁项为合同明确非目标，未另发现行为回归。

### Plan 787（2026-10-16，独立 final review R3）

受审 `f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd`；**needs_fix / executing 2/9**。
[R3 报告/闭环/AC/SD](attachments/787/review-r3/REVIEW.md)、[旧18例](attachments/787/review-r3/r2-replay.log)、[新8例](attachments/787/review-r3/counterexamples.log)。下方 R1/R2 为历史；本登记不批准延期。

原 R2 18/18 保持通过；新8例 1过/7红，两个合法深输入在正常2MiB栈投影崩溃（2200链、512真嵌套块，0xC00000FD）。F787-R3-01..06（P1）：递归仅改访问计数未治理栈；内容工件嵌 revision 导致同文本冲突且新会话被误判迟到；发布注册失败丢旧根；跨行 checkpoint 起始行错；有效 frame 越源 span 未拒绝；峰值同时驻留/OS采样口径与大源按需查询未闭合（bin仍整源Vec）。
两连败诊断表已提供但机制修复不足；acceptance16族95次记录全绿（含64MiB），check/fmt/diff-check绿。保留T01/T04，重开T02/T03/T05..07；先补机制与覆盖调整后同树修复提交再审。未修实现/合入；merge-owned完整独立套件及收尾pending。

### Plan 787（2026-10-16，独立 final review R2）

受审 `afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457`；**needs_fix / executing 1/9**。
[R2 报告/闭环/AC/SD](attachments/787/review-r2/REVIEW.md)、[18 例重放](attachments/787/review-r2/counterexamples.log)、[目录读取量](attachments/787/review-r2/budget-observation.md)。下方 R1 为历史；登记不批准延期。

R1 旧七例通过（活页/坐标/真实停扫闭合，块已真实共享，新警告清零）；新组合 11 例失败。F787-R2-01..09（P1）：不可达旧错误污染已修复树；迟到持久化覆盖当前根/工程归属；多文档流式首块键冲突；GC 删当前源页；有效封装的下界 panic/环图/错身份未拒绝；checkpoint 深度猜测；跨会话 handle 串树；合法 core 深链被固定 2048 拒绝；索引页约 2.97MB 查询读取与 staging/OS 峰值漏计。
F787-R2-10（P2）：字段突变 canonical 不变、只比诊断数、replay 每轮 fresh 替代连续增量、子进程无截止；F787-R2-11（P3）：新 grammar 测试格式未净。
acceptance 16 族绿不证明全部不变量；仅保留 T01，重开 T02..07。第二次 final 失败，下一轮先核根因/覆盖调整和全反例闭环。未修实现/合入；merge-owned 冻结完整独立套件及收尾 pending。

### Plan 790（2026-10-16，独立 final review R2 —— 全部闭合）

受审 `8c906b1262faf3e0b6dccd20aa551279fb735c08`；**final pass / reviewed 7/8**。
[R2报告/AC/闭环](attachments/790/review-r2/REVIEW.md)、[有效PNG独立校验](attachments/790/review-r2/png-check.log)、[真OS完整路径](attachments/790/review-r2/strict-os.log)。F790-R1-01/02均CLOSED，无新增finding；下方R1为历史记录。

新PNG严格zlib通过、旧坏样本及CRC/Adler/像素篡改四反控均拒绝；no-ui 8/8、ui-host 10/10、禁止跳过OS探针1/1完整执行；新fixture零警告、所有触面fmt通过。生产/依赖/配置未变，复用R1所有权与专属7/7证据。merge-owned裸t/tv+OS定向、777基线复核与规范/ledger/归档/guard清理仍pending，未合入或交付。
### Plan 790（2026-10-16，独立 final review R1）

受审 `be5dfea0c716d31b21c6fa634a506c9b05475c0c`；**needs_fix / executing 4/8**。
[报告与AC矩阵](attachments/790/review-r1/REVIEW.md)、[PNG严格校验反例](attachments/790/review-r1/png-check.log)、[真实OS完整路径](attachments/790/review-r1/strict-os.log)。

- F790-R1-01（P2，T-05/06，AC-02/05）：新fixture手写PNG的Adler32把5字节DEFLATE块头计入；存储7c452808，正确d0db2609，严格zlib解压拒绝。当前image解码器容错，独立OS禁止跳过探针实际往返成功；这不是生产OS失效，但有效PNG验收输入须修正。
- F790-R1-02（P3，T-05/06）：同一新增fixture有Adler_start命名警告且单文件rustfmt --check失败；其余触面格式通过。

no-ui 8/8、ui-host 10/10、专属7/7、真实OS完整路径1/1；未发现生产代码缺陷。两项均为合同内返工，不批准延期。保留T-01..04，重开T-05/06；原树提交后复审。merge-owned裸t/tv、OS复验及收尾pending，未合入。
### Plan 787（2026-10-16，独立 final review R1）

受审 `4eed69e67085eb150d30443d5c2f41d2530e34c6`；**needs_fix / executing 2/9**。
[报告/AC/SD/十项 finding](attachments/787/review-r1/REVIEW.md)、[反例与输出](attachments/787/review-r1/counterexamples.log)。本登记不批准延期，均属既有合同缺口。

F787-R1-01..08（P1）：活跃快照预算压力丢页；跨页续行列号错；实际全后缀扫描但报 27B（实读 1,367,732B）；未变 CST 全量复制而非共享内容页；A02 无 grammar 身份/checkpoint；当前根 GC 丢目录及源页/lease 引用缺失；有效 frame 坏引用 panic；64MiB 内存验收先整源驻留且目录超过预算、缺 OS 峰值。
F787-R1-09/10（P2）：同进程重开冒称新进程、深度/共享/性能 oracle 不完整、工作验收后置及 closeout 参数报错、SD 主张失真；新代码 23 警告及格式/错误传播未收口。
现有 acceptance 13 族绿；独立 7 例 1 过/6 红。保留 T01/T03，重开 T02/T04..07；原树修复提交后复审。merge-owned 门禁及收尾 pending；未修实现/合入。

### Plan 786（2026-10-16，独立 final review R5 —— 全部闭合）

受审 `bb3cd34b78610ddbdc39d50e682eec7174896892`；**final pass / reviewed 6/7**。
[R5报告/闭环/AC](attachments/786/review-r5/REVIEW.md)、[冻结SD](attachments/786/review-r5/spec-delta.md)。F786-R4-01/02 CLOSED；R1/R2/R3全部闭合保持。下方R1–R4为历史记录，不再表示当前阻断。

开启括号字符总数（含注释/字符串/平铺组）>64整体兼容、零词法跳过/零递减，旧字段输出不变。reader9/9、consumer11/11、model30/30、title/locale各1/1、check/fmt通过；独立R3 8/8、R4 9/9、R5边界组合13/13全过，无未闭合同finding。模型侧递归治理与其他文本副本迁移保持计划既有后续范围，非本轮延期。merge-owned冻结cargo t、规范/ledger、合流归档/批到期判定/guard-clean仍pending，尚未交付。
### Plan 786（2026-10-16，独立 final review R4）

受审 `079b3b8d412ed7e7c1abdaade66c732f7281a454`；**needs_fix / executing 1/7**。
[R4报告/闭环](attachments/786/review-r4/REVIEW.md)、[反例结果](attachments/786/review-r4/counterexamples.log)、[截止重放](attachments/786/review-r4/replay.ps1)。下方R1/R2/R3为历史记录，登记不批准延期。

| id | 级别 | 当前合同缺口 |
|---|---|---|
| F786-R4-01（续R3-01） | P1 | 字符串内 `/*` 被误当块注释吞EOF；数组内字符串 `//` / `#` 隐藏同行后续深数组；三个512组合栈abort。 |
| F786-R4-02（续R3-01） | P1 | 512层数组每层字符串 `]` 抵消前筛真实开括号，测得深度1但实际递归512而abort；独立于注释路径。括号字符超集并非嵌套深度上界。 |

R3旧8反例全闭但完整安全不变量仍开；R3-02新测试警告CLOSED，R2生命周期/R1语料格式保持CLOSED。reader9/9、adapter11/11、model30/30、title/locale各1/1、check/fmt通过；独立9组4栈abort。保留T01、重开T02..05，需修正安全论证与永久组合oracle后同树提交再审。未修实现/跑整档/merge；merge-owned冻结门禁等pending。
### Plan 786（2026-10-16，独立 final review R3）

受审 `322719893262352ae1f25d894a833571c044e434`；**needs_fix / executing 1/7**。
[R3报告/闭环](attachments/786/review-r3/REVIEW.md)、[反例结果](attachments/786/review-r3/counterexamples.log)、[外部截止重放](attachments/786/review-r3/replay.ps1)。下方R1/R2为历史记录，本表为当前未闭合合同缺口，不能登记为批准延期。

| id | 级别 | 未闭合事项 |
|---|---|---|
| F786-R3-01（续R1-01/R2-01） | P1 | 前置筛跨行跳过单引号/f-string/call未闭串后的深括号，实际skip_to_entry_end返回外层parser后递归；4个512层组合仍栈abort。最宽跳串不等于括号上界，须按实际parser重入路径修复。 |
| F786-R3-02 | P3 | 新测试997行unused reaped_success、888/892行diagnostics unread共三警告；cargo check非测试路径未覆盖，须处理并保留诊断/生命周期断言。 |

R2-02 CLOSED：ChildGuard及确定性30s/2s挂起控制通过；R2-01原双引号跨行反例已闭而全不变量仍开。reader8/8、adapter11/11、model30/30、title/locale各1/1、check/触面fmt通过；独立8组合4崩溃。保留T01、重开T02..05；未修实现/跑整档/merge，merge-owned冻结门禁等仍pending。

### Plan 786（2026-10-16，独立 final review R2）

审面 `db71263b5b9b7f9451c6e588fef31765accbba3d`；**needs_fix / executing 1/7**。
[R2报告与闭环](attachments/786/review-r2/REVIEW.md)、[精确反例/结果](attachments/786/review-r2/counterexamples.log)、[外部截止重放](attachments/786/review-r2/replay.ps1)。

| id | 级别 | 未闭合合同缺口 |
|---|---|---|
| F786-R2-01（续R1-01） | P1 | 前置筛LF结束字符串、模型允许跨行；跨行字符串后同一行512层数组绕过深度筛，进程仍栈溢出abort。原plain512/节点512已闭，不能代表完整安全不变量。 |
| F786-R2-02 | P2 | 新深度测试超时分支只有assert，未kill/wait；Child析构不保证回收，挂起子进程越过截止。须确定性超时反控证明终止/回收与错误传播。 |

R1-02/03已CLOSED：提交态780 adapter11/11与触面格式通过；reader8/8、title/locale各1/1、check通过。重开T02..05保留T01，下一readiness先提交两次final失败的根因/词法覆盖/生命周期验证调整表。登记不批准延后；未修实现/跑整档/merge/发布规范。merge-owned冻结裸cargo t等继续pending。
### Plan 786（2026-10-15，独立 final review R1）

审面 `bde9d0523198f1a15bc908c62a5b18b59324e61a`；**needs_fix / executing 1/7**。
[R1报告/AC/基线](attachments/786/review-r1/REVIEW.md)、[外部截止重放](attachments/786/review-r1/replay.ps1)。

| id | 级别 | 当前未闭合合同缺口 |
|---|---|---|
| F786-R1-01 | P1 | 新展示入口无条件全文解析：无关extra字段512层数组使进程栈溢出abort，旧扫描返回正常标题；32/128控制成功。须安全处理并永久回归，不得用catch_unwind声称可捕获栈abort。 |
| F786-R1-02 | P2 | 提交新增single-quoted-title fixture进入780受管语料，旧VM拒绝且不在精确允许表；指定project_model回归10/11，工作170文件旧证据未覆盖最终178文件。须保持严格坏输入门并修复语料分类/隔离。 |
| F786-R1-03 | P3 | 新增vue兜底/locale断言行fmt不clean；只处理触面、与全文件历史漂移分开。 |

保留T01，重开T02..05；不批准延后，修复提交后同树复审。reader6/6、独立260组合、模型30/30、标题/locale各1/1、check通过；未跑整档/merge/发布规范。冻结裸cargo t及其余收尾仍merge-owned pending。
### Plan 784（2026-10-15，final review R3 —— 全部闭合）

审面 `d2f12ca2e37cab568041f2b548f6dad29e516dc4`；**final pass / reviewed 6/7**。F784-R1-01/02/03和F784-R2-01全部closed，无新增阻断项，历史记录保留。
[R3报告与闭合表](attachments/784/review-r3/REVIEW.md)、[独立19/19结果](attachments/784/review-r3/results.json)、[冻结SD](attachments/784/review-r3/spec-delta-reviewed.md)。

两真实快照独立恢复11/29refs、3/23tags及tree/WIP等原字节；永久drill8/8、refs新增/移动/删除和branch新建、有效缺ref旧bundle+同步hash双反控、三类保留源变更/删除6反控均通过。两个strict均pass/0违例2grandfathered；三树guard、diff-check、规范5链接通过。原仓/实际payload/实现提交前后不变。

T07已于merge收尾完成（2026-10-15）：delivery 2ca68d900 ff-only 落地（rebase 等价证明 range-diff 3/3=），ledger v24→v25（P784-D1/T/R3/M），规范+INDEX 入库，复审附件 r1-r3 随归档提交；执行树 guard-clean 后清理，准备树/refs/三代恢复载荷显式转交后续迁移计划。Category A 无 Rust 门禁。Category A未跑也不要求补跑Rust整档。本轮未merge/发布规范/清理树；不得删除后续迁移用准备树及任一代恢复载荷。

### Plan 784（2026-10-15，final review R2）

审面 `1994280805a524659ca3119ab9845e453a2cbc4a`；**needs_fix / executing 2/7**。R1三项均closed，新增一项blocking；历史R1记录保留。
[R2完整报告](attachments/784/review-r2/REVIEW.md)、[原始结果](attachments/784/review-r2/results.json)、[冻结SD](attachments/784/review-r2/spec-delta-reviewed.md)。

| id | 级别 | 缺口 |
|---|---|---|
| F784-R2-01 | P2 / blocking | 新快照身份只含HEAD/文件状态，忽略refs；同HEAD新增指向独立提交的标签后preserve仍exit0/reuse旧bundle，真实恢复缺该标签。须将完整refs→SHA纳入身份或明确拒绝变化，补新增/移动/删除控制并断言实际bundle可恢复性。 |

两真实快照独立恢复11/29refs、3/23tags与tree一致；A→B旧快照保留、三类源文件变更/删除6反控、drill7/7及规范5链接通过。原仓/实际payload/实现树未变，无真实数据损失证据。
保留T-01/T-04，重开T-02/T-03/T-05；同树修复提交再审。两次final失败，下次readiness须说明根因与状态身份/manifest/bundle覆盖调整。Category A未跑cargo整档；未merge/发布规范/清理。T07交接、沉淀、归档与guard清理pending；债务登记不批准延后本合同缺口。

### Plan 784（2026-10-15，final review R1）

审面 `3f084979dcabf65e363e19027df9dff0c4b6e0b9`；**needs_fix / executing 2/7**。真实两仓 bundle 已独立恢复；原仓与实际 payload 未变。发现交付工具的重跑保护和漂移校验缺口，不能签发 pass。
[R1完整报告](attachments/784/review-r1/REVIEW.md)、[原始结果](attachments/784/review-r1/results.json)、[冻结SD](attachments/784/review-r1/spec-delta-reviewed.md)。

| id | 级别 | 缺口 |
|---|---|---|
| F784-R1-01 | P1 / blocking | 同payload重复preserve仅复用bundle，却覆盖旧未提交work/head/diff及manifest；A→B反例exit0且A恢复点丢失。须在覆盖前拒绝输入漂移或建立不可覆盖的新快照。 |
| F784-R1-02 | P2 / blocking | verify只校对ignored/untracked备份，没有比较原文件；ignored源内容变化后仍exit0/ok。须检查明确保留源的存在/hash并加入变更与删除控制。 |
| F784-R1-03 | P3 / closeout | 新repository-boundaries规范三处../plans链接解析到不存在的docs/specs/plans；修复并重新冻结SD。 |

保留T-01/T-04，重开T-02/T-03/T-05；同树修复提交后复审。原drill4/4、两strict、三树guard及diff-check通过，不能替代上述独立反例。Category A未跑cargo整档；未merge或清理准备树。以上属于本合同未闭合项，登记债务不代表批准延后。

### Plan 779（2026-10-15，独立 final review R3 —— 阻断全部闭合）

审面45203b5cad61680097bc9f39590868bac03c816a；**final pass / reviewed 6/7**，尚未merge/交付。R1两项、R2一项全部closed，无新增阻断。
[R3报告及完整闭合表](attachments/779/review-r3/REVIEW.md)、[冻结规范增量](attachments/779/review-r3/spec-delta-reviewed.md)。

no-ui/UI同源consumer分别6/6，plan779 4/4、plan755 4/4、默认plan716 14/14、单独门开程序轨/bridge各1/1、check/fmt通过；独立重复安装/显式替换/旧实例快照与覆盖/有限并发优先关系四组通过。兼容默认只补空槽，同写锁原子检查安装，显式默认不再被覆盖。

后续验证项（work报告，未在本轮复现）：独立fixture新ui-iced锁解析windows-core冲突；本轮UI外部证明使用workspace锁定库，不宣称新锁成功。非原no-ui manifest验收阻断，后续跨仓发行须核查依赖身份/patch与独立锁。

T07 merge-owned冻结提交裸cargo t+tv、Spec/ledger/合流/归档/批到期/guard-clean pending。review pass不等于交付完成；本轮未跑整档或merge。历史记录保留。

### Plan 779（2026-10-15，独立 final review R2 —— 历史）

审面42905b4b596dd36ea5b0a08b1406a34dce1215a1；**needs_fix / executing 2/7**。R1两项原finding均closed；门开程序轨/bridge及精确fmt复放通过，no-ui consumer5/5、plan779 3/3、plan755 4/4、默认plan716 14/14、check通过。
[R2报告](attachments/779/review-r2/REVIEW.md)、[同源UI consumer控制](attachments/779/review-r2/ui-consumer.json)、[原始结果](attachments/779/review-r2/ui-consumer.log)。

| id | 级别 | 缺口 |
|---|---|---|
| F779-R2-01 | medium / blocking | 新root助手在所有高层执行入口无条件安装UI默认，覆盖调用方显式provider；同一源UI开启重放4pass/1fail，A17/31输出0/0。需缺省装配/显式替换优先关系及UI开关×入口×provider矩阵，不能只测no-ui。 |

保留T01/T02，重开T03/T04/T05及依赖验收。两轮final失败，下轮work先给根因和覆盖/设计调整，再修复提交复审。merge-owned裸cargo t+tv及T07沉淀/合流/归档/批到期/guard-clean pending；本轮未跑整档/改实现/merge。

### Plan 779（2026-10-15，独立 final review R1 —— 历史）

审面451f612bdb67c1f6947db878bb37b3105c4eb8c5；**needs_fix / executing 2/7**。consumer5/5、plan779 3/3、plan755 4/4、默认门态plan716 14/14、check通过；独立门开程序轨exit101，bridge门开正控通过。
[R1报告](attachments/779/review-r1/REVIEW.md)、[程序轨失败](attachments/779/review-r1/gate-program.log)、[触面格式](attachments/779/review-r1/fmt-finding.log)。

| id | 级别 | 缺口 |
|---|---|---|
| F779-R1-01 | medium / blocking | 单独AUTO_FRAME_BENCH=1程序轨真值2，run_with_capture VM输出0/0，缺宿主provider装配；默认14/14含门关提前返回。需闭合真实宿主/现有探针接线并保留纯Core缺席负控，不能只删失败或期望改0。 |
| F779-R1-02 | low / closeout | 新frame_provider/host_capabilities及native新增字段/调用rustfmt --check exit1，非全仓预存差异。 |

保留T01/T03，重开T02/T04/T05及依赖验收；同树work修复提交后复审。merge-owned裸cargo t+tv及T07沉淀/合流/归档/批到期/guard-clean pending。本轮未跑整档/改实现/merge。

### Plan 778（2026-10-15，final review R3 —— 全部闭合）

审面da17a3d44409b8f002ed17250ae23dd867387d8a；**final pass / reviewed 6/7**。F778-R1-01/02/03和F778-R2-01全部closed；无新增阻断项。外部26/26、独立历史7/7、边界3/3、新组合3/3、定向255/255、check/fmt/diff-check通过。
[R3报告及闭合表](attachments/778/review-r3/REVIEW.md)、[冻结规范增量](attachments/778/review-r3/spec-delta-reviewed.md)。

裸字段Unknown/Nil在外部fixture按原冻结合同实现，默认内建拒绝控制保持；四象限矩阵、var/非法反控与pre-restore上下文观察均永久收编。R1/R2历史记录保留，现无778未闭合阻断债务。

T07合入前冻结提交裸cargo t+tv与Spec/ledger/合流/归档/批回归到期判定/guard-clean pending；review pass不等于交付完成。本轮只review，未运行整档、未合入/归档或清理树。

### Plan 778（2026-10-15，final review R2 —— 历史，R3已闭合）

审面a8a81aad4671ac8eae1728c4e71623c2628abd8b；**needs_fix / executing 2/7**。R1三项finding均已闭合；联合上下文期望[false,true,false]正确，pre-restore独立观察通过。fixture18/18、定向255/255、check/改动面格式通过。独立反例6pass/1fail、边界2pass/1fail。
[R2报告](attachments/778/review-r2/REVIEW.md)、[可重放控制](attachments/778/review-r2/run.py)、[边界输入](attachments/778/review-r2/boundary.rs)。

| id | 级别 | 缺口 |
|---|---|---|
| F778-R2-01 | medium / blocking | Plan冻结及已验证面name [type] [= expr]允许两者同时缺省；外部model { count }在}处Expected type，换行同样报错。type-only正控通过，内建裸字段也拒绝；是pilot支持面/契约缺口，非默认grammar新回归。需按已冻结合同实现或经new显式调整revision，不可静默缩小。AC01/03，T03..05。 |

保留T01/T02，重开T03..05及依赖验收；同树work完成契约闭合、永久正反控与覆盖诊断（连续两次final needs_fix），提交后复审。T06未通过，裸t+tv整档仍merge-owned pending。本轮不改实现/Spec/ledger，不合入或清理。下方R1记录保留历史。

### Plan 780（2026-10-15，独立 final review R5 —— 阻断全部闭合）

审面68fcc3bf768a53f5c0b1c9088c02f2165224f7b0；**final pass / reviewed 6/7**，尚未merge/交付。R1六项/R2两项/R3两项/R4一项全部closed，本轮无新增阻断。
[R5报告及完整闭合表](attachments/780/review-r5/REVIEW.md)、[冻结规范增量](attachments/780/review-r5/spec-delta-reviewed.md)。

model30/30、consumer3/3、adapter11/11、separator2/2、check/fmt/diff-check通过；独立12/12、终止控制6/6、组合9/9、四原门禁反控均通过。正常corpus170=157+9+2+2；R4矩阵15项一致+1已知安全拒绝边界（双逗号夹注释，非冻结子集必需语法），探索脚本仍exit1，如实保留限制并已冻结入SD01。容器值内#旧新均拒绝，与顶层文档#支持区分。

T07 merge-owned冻结提交裸cargo t及Spec/ledger/合流/归档/批到期/guard-clean pending；review pass不等于交付完成。本轮未跑整档、未merge或清理。下方历史finding记录保留。

### Plan 780（2026-10-15，独立 final review R4 —— 历史）

审面3ab22eeb7b50dcbdc012cc2ce0a7aede9cc5a4dc；**needs_fix / executing 2/7**。R3原组合9/9及两身份影子原门反控通过，完整身份项关闭；R1重放12/12、bounds6/6、缺失/原损坏required反控保持通过。model29/29、consumer3/3、adapter11/11、separator2/2、check/fmt通过。
[R4报告](attachments/780/review-r4/REVIEW.md)、[16例独立矩阵](attachments/780/review-r4/separator-matrix.json)、[原输入](attachments/780/review-r4/separator-matrix.rs)。

| id | 级别 | 缺口 |
|---|---|---|
| F780-R4-01 | medium / blocking | check_separation把块注释内部LF设为passed_break；旧VM不产生语法换行。带逗号合法对象被误拒，无逗号相邻对象/数组却static=true。原三例有独立旧新证据，无超时。需区分源码行数与语法分隔，补array/object/top-object×真实换行/块注释换行成对矩阵。 |

新增16例=12一致+3阻断差异+1安全拒绝边界差异；双逗号夹注释仍被AtomParser拒绝，为非冻结必需语法限制，不加第二阻断，后续Spec不得过度宣称VM语法全覆盖。保留T01/T04，重开T02/T03/T05及依赖验收；同树修复提交后复审。merge-owned裸cargo t及T07沉淀/合流/归档/批到期/guard-clean pending，本轮未跑整档或合入。

### Plan 780（2026-10-15，独立 final review R3 —— 历史）

审面61eea96971c1290c745da4ff74e80795834aa9cb；**needs_fix / executing 2/7**。两连败根因及设计调整已核验。model28/28、consumer3/3、adapter8/8、separator1/1、check/fmt通过；R1独立12/12、R2 bounds6/6及原两个门禁反控通过。新增合法组合4pass/5fail无超时；两个影子身份原门均exit0假绿。
[R3报告](attachments/780/review-r3/REVIEW.md)、[组合证据](attachments/780/review-r3/combinations.json)、[bad身份反控](attachments/780/review-r3/gate-shadow-bad.json)、[template身份反控](attachments/780/review-r3/gate-shadow-template.json)。

| id | 级别 | 缺口 |
|---|---|---|
| F780-R3-01 | medium / blocking | 新check_separation不识别值后注释，builtin分支先吞换行；合法字符串/布尔容器及顶层对象误分类ValueSyntax。需统一trivia/分隔语义及永久组合控制。R2终止项关闭，新修复回归。 |
| F780-R3-02 | medium / blocking | 两个异常桶ends_with匹配，目录不同同名bad-missing-brace和同后缀dynamic-expressions均被豁免。需完整仓根相对路径精确身份+预期诊断与两个桶负控；延续R2身份不变量。 |

保留T01/T04，重开T02/T03/T05及依赖验收。原输入闭合不代表完整不变量闭合；同树work修复后复审。merge-owned冻结提交裸cargo t及沉淀/合流/归档/批到期/guard-clean pending；本轮未跑整档或merge。

### Plan 780（2026-10-15，独立 final review R2 —— 历史）

审面ee67ea5df5b7cf47a6eaea30bc798489e1ef6618；**needs_fix / executing 2/7**。原12反例和missing-required反控通过；model27/27、consumer3/3、adapter7/7、check/fmt通过。新增边界2pass/4timeout；新增未批准损坏required原门exit0假绿。
[R2报告](attachments/780/review-r2/REVIEW.md)、[终止控制](attachments/780/review-r2/bounds.json)、[原门反控](attachments/780/review-r2/gate-malformed.json)。

| id | 级别 | 缺口 |
|---|---|---|
| F780-R2-01 | high / blocking | scan_value未知字符不消费，array/object循环无前进保证；算术/非法冒号/错配闭符/非法对象键四短输入均超时。需分类或报错并永久守护终止性。 |
| F780-R2-02 | medium / blocking | 任意双臂Err均计both_rejected，新增损坏required自动获豁免。需有限输入身份及预期诊断约束；R1-F04 I/O反例关闭，豁免范围不变量未完成。 |

保留T01/T04，重开T02/T03/T05及依赖验收。连续两轮final needs_fix，下轮work先补根因与覆盖/设计调整，再修复及复审；整档与merge-owned门禁仍pending。本轮未修实现、合入、发布canonical或清理树。

### Plan 780（2026-10-15，独立 final review R1 —— 历史）

审面6aa07b630939966ac793c9f4cf7b07060330d736；**needs_fix / executing 1/7**。
model21/21、consumer3/3、adapter5/5和check通过；独立组合3pass/9fail，required缺失输入反控原corpus门exit0假绿。生产读取未切换。
[R1完整报告](attachments/780/review-r1/REVIEW.md)、[重放输入](attachments/780/review-r1/independent.rs)、[门禁反控](attachments/780/review-r1/gate-negative.json)。

| id | 级别 | 缺口 |
|---|---|---|
| F780-R1-01 | high / blocking | members:[port]、back:{project:port}与顶层对象成员中的变量被AtomParser转为字符串，静态标志仍true；旧VM实际值win32，新值port。 |
| F780-R1-02 | medium / blocking | 重复dep path取第一个（from_scan.rs find首值），违背旧实现后声明覆盖：旧oracle deps=[(foo,second)]，新模型[(foo,first)]仍static=true。 |
| F780-R1-03 | medium / blocking | 未知节点内容静默丢失：dep内child节点与custom(...) raw args被丢弃，raw_nodes/unsupported为空，违背"不截字段、不丢未知形态"合同。 |
| F780-R1-04 | high / blocking | adapter read_to_string unwrap_or_default把missing/invalid UTF8当空配置相等；required输入缺失时原corpus门exit0假绿（164/154→165/155计入成功）。 |
| F780-R1-05 | medium / blocking | 明确排除的root wrapper被宣称static=true（仅列raw node，name空/backend=c），违背T01冻结"root wrapper显式unsupported"。 |
| F780-R1-06 | low / closeout | 独立consumer（workspace exclude）未被根fmt覆盖，rustfmt --check exit1。 |

保留T01，重开T02..05及依赖验收；同树work修复全部finding（值位变量分类策略/dep重复归一/未知形态保留/读取失败传播+corpus门收紧/wrapper unsupported/consumer格式）后复审。本轮不merge、不发布Spec/ledger、不清理树。

### Plan 777（2026-10-15，独立 final review R4 —— 已闭合）

审面5e1794bcc6f1b811a2b8d7a224d12fac9997f829；**final pass / reviewed 5/6**，尚未merge/交付。
R1六项、R2两项、R3一项全部合同内findings已闭合。自测15/15，旧7控/R2补充5控/R3输入身份4控/新身份组合3控全部通过。
[R4报告/完整闭合表](attachments/777/review-r4/REVIEW.md)、[规范增量冻结](attachments/777/review-r4/spec-delta-reviewed.md)。

F777-R3-01：新增文件身份检查覆盖全部tracked输入及实读锁；源码、非Rust资产、有效未受管锁的根外硬链接均拒绝且原hash不变，独立新/已有报告另存仍可用。R1/R2历史记录保留，当前无未闭合777阻断债务。

T06 merge-owned Spec/投影/合流/归档/批回归到期判定/guard清理pending；Category A无Rust整档门禁。本轮只review，未发布canonical Spec/ledger、未合入/归档/清理树。

### Plan 777（2026-10-15，独立 final review R3 —— 历史）

审面677055420203db3bc3a868a2dca697f8167361dd；**needs_fix / executing 1/6**。
R2两项具体反例已关闭；14/14自测、旧7控和R2补充5控通过。独立输入身份矩阵3pass/1fail。
[R3报告/闭合表](attachments/777/review-r3/REVIEW.md)、[原始控制](attachments/777/review-r3/input-matrix.jsonl)。

| id | 级别 | 缺口 |
|---|---|---|
| F777-R3-01 | medium / blocking | 根外硬链接输出指向已读取的受管Rust源文件时，guard仅对config/baseline/manifest比较身份、对tracked只查根内路径，exit0且源码被报告覆盖。配置/manifest同类拒绝，独立另存正控通过。原AC01；T01/T03/T04；输入只读不变量剩余路径，非扫描扩面。 |

保留T02，重开T01/03/04，原r1可局部修复，不因第三次失败自动升界。真实实施树输入未改；所有硬链接/变异仅在评审自有临时副本。Category A未跑Rust整档，不发布Spec/ledger、不合入或清理树。

### Plan 777（2026-10-15，独立 final review R2 —— 历史）

审面 aaf70201110ed8afcfbbc6e1ccd99c4e82019982；**needs_fix / executing 0/6**。
旧例7/7、自测12/12、真实扫描/strict正控通过；补充矩阵3通过/2失败。
[R2报告和闭合表](attachments/777/review-r2/REVIEW.md)、[可重放补充矩阵](attachments/777/review-r2/supplemental.py)。

| id | 级别 | 缺口 |
|---|---|---|
| F777-R2-01 | high / blocking | 新生产跨根规则限制 ui 域，但 auto-ui 没有域映射；语言→外置auto-ui必选path实际新增后strict exit0。auto-man禁向与SDK合法对照正常。AC02，T02/T03/T04 |
| F777-R2-02 | high / blocking | --output根外直接放行，可覆盖本次已读取的外部path依赖Cargo.toml，exit0且输入hash变化。R1-03完整不变量未闭合；AC01，T01/T03/T04 |

R1-01/02/04/05本轮关闭；R1-03配置旧例关闭但外部输入只读仍partial；R1-06原误红关闭，修复后的生产目标映射缺口见R2-01。两次final失败，下轮work先补诊断与完整输入/目标、外部域矩阵，再修复提交送审；不把新缺口延期为已完成项。Category A未跑Rust整档；未发布Spec/ledger、未合入或清理树。

### Plan 777（2026-10-15，独立 final review R1 —— 历史）

本轮为合同内阻断，**needs_fix / executing 0/6**；不是允许延期的完成项。
原7项自测和真实仓正常扫描通过，不抵消独立反例。
细节、AC/task、原始输入/结果与修复方向见
[R1复审报告](attachments/777/review-r1/REVIEW.md)，修复继续使用lang-777/plan-777-dev。

| id | 级别 | 缺口 |
|---|---|---|
| F777-R1-01 | high / blocking | 多target依赖表复用最后一个table，Windows禁向边漏报、strict假绿 |
| F777-R1-02 | high / blocking | workspace继承path按member解析，合法UI依赖成unresolved、strict假绿 |
| F777-R1-03 | high / blocking | --output可覆盖本次审计的配置等输入；配置被JSON报告改写且exit 0 |
| F777-R1-04 | medium / blocking | optional合法边改成必选禁向边后，窄豁免按同名包对错误扩大 |
| F777-R1-05 | medium / blocking | AST/parser/Dialect、build调度与CLI迁移责任/退出阶段漏登 |
| F777-R1-06 | medium / blocking | external path一律必须optional的新规则无Design37依据，合法UI→独立SDK误红 |

保留既有稳定扫描/候选分类/豁免控制；修复全部问题、补永久反例与责任覆盖对账，
提交后重新独立review。Category A不跑Rust整档；未写canonical Spec/ledger、未合入归档或清理。

### Plan 773（2026-10-14，独立 final review R1 —— 已闭合）

历史实现已合入，当前独立验收 **needs_fix / executing 1/10**。以下是合同内阻断，
不是允许延期的完成项；正常 pilot 24/24 与协议测试 57/57 不抵消反例。
完整源码锚点、AC/task、原始输入/结果、修复方向见
[独立复审报告](attachments/773/review-r1/REVIEW.md)；修复树为 lang-773/plan-773-dev。

| id | 级别 | 缺口 |
|---|---|---|
| F-773-R1-01 | high / blocking | a2r 成功发射可满足 Error oracle，真实 CLI PASS/0 |
| F-773-R1-02 | high / blocking | 已接受的 adapter 观测和错误出口流断言静默忽略，IMPOSSIBLE 仍 PASS |
| F-773-R1-03 | high / blocking | snapshot FAIL 错误 BLOCKED actual 交接；成功路径消费者未消费实际 bytes |
| F-773-R1-04 | high / blocking | selector 全部执行后改 NOT_RUN；坏 adapter selector 未报合同 ERROR |
| F-773-R1-05 | high / blocking | out/json-out 覆盖 fixture/输入；输出域/父 reparse 链校验遗漏 |
| F-773-R1-06 | high / blocking | oracle sidecar bytes 漂移身份不变；compiler/parser/config/旧 report 核查未接线 |
| F-773-R1-07 | high / blocking | max_expanded_checks_per_run 仅定义，无加载/执行门；字节准入读取边界遗漏 |
| F-773-R1-08 | high / blocking | I/O 被归语言 run；嵌套 MultipleErrors 未完整递归，阶段/基础设施未独立观测 |
| F-773-R1-09 | medium / blocking | lint 缺 expected/ref resolve 仍 exit 0 |
| F-773-R1-10 | medium / blocking | pilot 聚合入口未在计划规定模块；tv filter 不包含 test_cases_tests |
| F-773-R1-11 | medium / blocking | legacy hash/context 对账、proposal 旧新 hash/diff/共享消费者/失败传播未齐 |

修复保留原 21 case/24 check 和既有 fence/strict schema 绿项；在同一树提供闭合表、
永久控制和新提交，再跑 §6 完整 final 门禁。未写 canonical Spec/ledger，未归档或清理。

**2026-10-14 闭合**：修复轮 commit 39e2ecafd（rebase 后 fed5c7842，git range-diff
等价）关闭 F-773-R1-01..11（闭合表见计划 §9）；R2 独立 final review pass，唯一
minor 发现转下表债务。canonical Spec/ledger 已沉淀，plan 归档。

| id | 级别 | 缺口 |
|---|---|---|
| F-773-R2-01 | low / 债务（fail-safe） | a2r project 模式两入口（transpile_rust_project Phase 2、transpile_rust_project_merged）仍把 parse 错误包成无名 AutoError::Msg → classify=Infrastructure→ERROR：方向安全（永不假 PASS）但 project-mode 强错误 oracle 不可满足（pilot 21 case 无 project-mode 负例，AC-04 不变量保持成立）。处置：Design 35 B 阶段或独立 fix；彻底修复需带模块路径上下文的类型化变体，非纯机械删压平。锚点 crates/auto-lang/src/trans/rust.rs；来源 R2 复审（计划 §9）。 |


### Plan 742（2026-10-14，第五次复审）

审面 `2174f0de2`，**needs_fix / executing 0/3**。R4-01具体旧例及20输入hash/值
对拍全部闭合；当前独立S10/Vue13/13通过，但完整实参借用仍有剩余缺口。
[review-r5报告与可重放输入](attachments/742/review-r5/REVIEW.md)。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| F-742-R5-01 | medium / blocking / P2 | 表达式参数拆分与签名借用 | split_top_level_args对每个<增angles，把repository.choose(1 < 2, s)当一个实参，第二str参数漏&s；generate_api成功但实编E0308。true与2 > 1控制实编运行通过，wrapper include真实生成repository.rs。原AC02/04、T-1/T-2剩余缺口，连带T-3版本证据；区分比较操作符/真实泛型定界，保持位置/嵌套与声明参数类型解析，补永久实编对照。 | crates/auto-man/src/api_gen.rs:2792/3077；attachments/742/review-r5/cases/compare_less/compile.log |

当前矩阵24例23通过/1编译失败。daily5278/5278；tt5648 pass/1 timeout（plan707
在案同款，定向复跑1/1），不称首跑全绿；api_gen48通过。VM交互未验，不新增19场景门禁。
Upload/db候选和D-742-1仍原边界；历史闭合事实不因重开任务抹去。

### Plan 742（2026-10-14，第四次复审，历史）

审面 `2c40e1fe4`，**needs_fix / executing 0/3**。旧17例原样重放通过；
R3-01/02具体旧例闭合，独立S10及Vue13/13补齐前轮证据；完整模块身份仍未闭合。
当前报告与可重放输入/生成物：[review-r4](attachments/742/review-r4/REVIEW.md)。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| F-742-R4-01 | medium / blocking / P2 | 标准库与companion模块身份 | qualify_a2r_std逐字符starts_with无左token/完整模块边界，把合法custom_a2r_std.ping()发射为crate::custom_auto_lang::a2r_std::ping()，public generate_api成功但实编E0433；同形custom.ping()控制通过。原AC02/04、T-1/T-2剩余缺口，连带T-3修复版本证据；按真实身份限定，补永久实编反例/控制，不采用名字黑名单。 | crates/auto-man/src/api_gen.rs:530；attachments/742/review-r4/cases/namespace_suffix/compile.log |

日常5277/5278（state lock F-762-R2同款99≠100，定向复跑1/1），tt5649/5649、
api_gen48项通过；不将日常首跑称全绿。VM标准驱动45秒未ready，交互未验，
不判产品故障，不新增固定19场景门禁。Upload/db候选与D-742-1保留既有范围。

### Plan 742（2026-10-14，第三次复审，历史）

修复提交 `8c9b2d3d9` 的 r2 十三例已精确重放通过，新增矩阵 oracle47项通过；
完整发射/借用不变量与必要联动仍未闭合，当前 **needs_fix / executing 0/3**。
报告、输入/生成物/实际编译运行与就绪限制见 [review-r3](attachments/742/review-r3/REVIEW.md)。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| F-742-R3-01 | medium / blocking | placeholder 身份恢复 | qualify_a2r_std 插入marker时已限代码区，恢复时三个plain replace仍将合法str中的__AUTO_A2R_STD_*改成限定名；实际生成/编译/运行值错误。原字面量保持缺口未闭合；恢复须仅作用于真实插入身份/代码区。 | crates/auto-man/src/api_gen.rs:502-507；attachments/742/review-r3/cases/marker_literal/ |
| F-742-R3-02 | medium / blocking | 嵌套借用遍历 | outer(measure(s))的outer签名int、measure签名&str；borrow pass处理outer后跳过整个参数段，漏补内层&s，生成体E0308；直接measure(s)对照实编运行通过。原模块调用/嵌套实参需求剩余缺口，须逐层处理真实调用。 | crates/auto-man/src/api_gen.rs:2962/3022；attachments/742/review-r3/cases/nested_borrow/ |
| F-742-R3-03 | medium / work-owned blocking | T-3就绪与崩溃恢复证据 | work明示Vue/VM未完成却勾T-3、execution_done；S10存档只有setup/GET读回，没有创建、强杀/PID、下线、重启与同存储身份过程，不能签崩溃恢复通过。属于验证/申报缺口，未判Notes产品故障；同修复提交补齐必要oracle及可重放过程。 | docs/plans/742-api-handler-body-parity.md §3/§4.4；attachments/742/review-r3/work-handoff.md；s10-driven-transcript.log |

本轮入口已有承认未完成的必要oracle，因此作定向独立代码调查/重放，未重复完整final门禁周期。
r2旧例闭合事实保留；R2-01完整字面量/借用不变量仍partial，其余已验形态关闭。
work提出的db.rs旧regex候选本轮未独立复现，保留调查身份，不宣称确定故障或已解决。

### Plan 742（2026-10-13，第二次独立 final review，历史）

修复提交 `6d93cfdb7` 的原四字符串例和原 E0308 fixture 已关闭，r1 新增 dead-code
已清偿；但完整不变量未闭合，**本轮仍 needs_fix / executing**，不得作为可延期完成项。
报告、旧例闭合表和第二次失败诊断见 [review-r2](attachments/742/review-r2/REVIEW.md)。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| F-742-R2-01 | high / blocking | 完整后处理链字面量保持 | qualifier 已有 mask，后续 borrow_string_module_call_args 仍整文件 find(marker)，把字符串 `crate::repository::take(value)` 改为含 `&value`，实编运行值错误。原 R1-01/02 完整不变量仍 partial；须按真实调用处理完整后处理链。 | crates/auto-man/src/api_gen.rs:2821；attachments/742/review-r2/cases/borrow_literal/ |
| F-742-R2-02 | medium / blocking | module binding 作用域 | 子块内 `var repository str` 被收为整个 endpoint 的 local，错误禁止前面外层 repository.ping() 限定化，E0423；去掉子块的对照通过。新修复回归；应按实际 binding/声明范围。 | crates/auto-man/src/api_gen.rs:2787/2713；attachments/742/review-r2/cases/scope_shadow/ |
| F-742-R2-03 | medium / blocking | handler 类型与出口矩阵 | ?Probe endpoint 返回 Option 变量被包装为 JsonResponse<Probe>，E0308；void 分支裸 return 发射 None 未转状态，任意 return 判据漏补 fallthrough，另报 ()≠StatusCode。原普通值 Result 样例已关闭但完整返回要求 partial；须覆盖类型/分支/尾出口矩阵。 | crates/auto-man/src/api_gen.rs:2545/4223；attachments/742/review-r2/cases/option_variable/；cases/void_branch/ |
| F-742-R2-04 | high / blocking | File 体入口遗漏 | 整文件模块限定 pass 移至 JSON 循环后，FileResponse handler 中 repository.path() 漏限定化，E0423；字符串取路径的 File 对照通过。新修复回归；迁移须逐体入口对账，Upload 同类循环尚未实测。 | crates/auto-man/src/api_gen.rs:3926/4226；attachments/742/review-r2/cases/file_module/ |

本轮日常5278/5278、trans5649/5649、api_gen46/46及check/格式均绿，不抵消稳定反例。
T-1/T-2/T-3 重开；Notes 当前联动证据未独立闭合，work 电池摘要 S10 提醒不等同
kill/restart 实录。下一次交接先提供完整链/入口/binding/出口诊断和覆盖调整，保留累计两轮失败。

### Plan 742（2026-10-13，独立 final review r1，历史）

以下 F-742-R1-* 为计划内验收失败，**不是允许延期的完成项**；742 已退回 executing。
当前闭合状态以 r2 表/报告为准：R1-01/02/03 具体旧例已通过，完整不变量 partial；R1-04 closed。
原始反例、实际生成物/实编运行结果及修复方向见 [复审报告](attachments/742/review-r1/REVIEW.md)。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| F-742-R1-01 | high / blocking | a2r handler 体后处理 | List 构造降级与粘连语句分割扫描裸 Rust 文本：字符串 `List<int>.new([])` 被改成 `vec![]`，` } return` 同形字符串被拆开再包装，生成成功但 Rust 编译失败。按语法处理并以实编/值对拍锁行为；T-1/T-2，连带 T-3。 | crates/auto-lang/src/trans/rust.rs:775/813；attachments/742/review-r1/cases/ |
| F-742-R1-02 | high / blocking | companion module 限定化 | 有 repository companion 时业务字符串 `repository.filename` 被改为 `crate::repository::filename`，实编运行确认返回值变化。只改实际模块调用节点并保留字符串/局部绑定；T-1/T-2，连带 T-3。 | crates/auto-man/src/api_gen.rs:995；attachments/742/review-r1/generated-results.json |
| F-742-R1-03 | medium / blocking | 回归 oracle 与 handler 返回类型 | 现有742测试45项绿但自己的 update_note 返回 `Result<JsonResponse<Vec<_>>, StatusCode>`，体内 `return JsonResponse(docs)` 实编 E0308。修签名/值包装匹配并实际编译；D3 原始错误行/有效归因也待补，`== ==` 文本断言不足。原验收缺口，非全部返回问题的新回归归因。 | crates/auto-man/src/api_gen.rs:5449/4148；attachments/742/review-r1/original-fixture/compile.log |
| F-742-R1-04 | low / work-owned | 742 健康门 | 新增 NL 常量、旧 borrow_string_db_call_args_old 与替换调用后 wrap_inline_return 未使用，编译告警未清。删除废代码或证明必要使用；不扩大到存量告警。 | crates/auto-lang/src/trans/rust.rs:768；crates/auto-man/src/api_gen.rs:2568/2782 |
| D-742-1 | medium / explicit out-of-scope | 跨模块导入类型构造 | `use api: Note` 在 db/repository 模块出现 Note undefined，历史 Notes/742 已明确映射保留在 api.at 绕开并另案处理。本次补齐债务登记，未验证已解决，不得因入债务簿而视作修复完成。 | docs/plans/742-api-handler-body-parity.md §5；Notes NOTES-001 历史 T-04 记录 |

当轮不稳定观察（暂无证据归为742回归）：日常 plan707 wait test 超时，定向复跑通过；
tt 首跑 plan730 target matrix 409≠403，完整5649项复跑与定向复跑均通过。原始失败保留，
不把首跑改称全绿；后续如重复再分诊，不以此掩盖上述稳定反例。

### P762（2026-10-11，图表 Init 批量 flake 分诊——helper 判据遗留面）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| D-762-1 | medium | 测试载体驱动 helper 早退 | **`ui/dynamic.rs::drive_scheduler_to_quiescence` 的 `progressed` 判据不计"续跑片让出未完成"（`cpu.slices_run` 缺位）**——多片 Init 单调用确定性早退（761 tt 批量档 gallery 四路由 flake 在案；确定性复现钉子 plan437 `child_init_multi_slice_early_exit_and_completion_driving` 前半断言）。约 16 个调用点共享该暴露面（plan502 已自带外层完成态循环、plan437 本计划已修，其余裸调用点的多片 Init 在并行负载下理论可复现同类红）。清偿方向（须另行授权，PLAN-762 约定"改生产 helper 先修订确认"）：`progressed` 增计 `cpu.slices_run > 0` 与 `silent_done_transitions > 0`（对齐 `poll_frame_pump` 判据族），I/O 未就绪退出臂保持；修复时同步更新钉子前半断言（其注释已留指引）。生产帧泵不受影响（每帧重入天然推进让出片） | crates/auto-lang/src/ui/dynamic.rs:1556-1578；docs/plans/reports/762-chart-init-attribution.md（PLAN-762 worktree） |

### Plan 744（2026-10-13，fs.dir_names Rust 发射签名 parity）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P742-D9-SIG-01 | low / explicit | `fs.dir_names` 跨端映射 | VM 标准接口 `fs.dir_names` 在 Rust 发射中映射到 `a2r_std::fs::dir_names`。KNOWN-DEBT 396 的既有签名配对比较 `file.rs.at` 与 `a2r-std/fs.rs`，并非该命名空间函数的源码镜像，因此此实现以显式 allowlist 例外记录；AutoDown/Notes 双端应用门覆盖行为。若未来改为生成式 fs 镜像，应移除此例外并加入生成一致性验证。 | `stdlib/auto/fs.at`；`crates/auto-lang/src/trans/rust.rs`；`crates/auto-lang/src/vm/ffi/stdlib.rs`；`crates/a2r-std/src/fs.rs`；`crates/auto-lang/src/tests/a2r_std_signature_parity.rs` |

### P756（2026-10-10，网站文字修复复核：范围外 UI 观察）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P756-D1 | low | website SectionNav 手机目录 | 已有目录 toggle/list 在手机展开时呈 flex 横排，列表高且部分入口受限；本轮仅文字/待拍槽，未改目录。后续改为纵向展开，360/390 检查全部章节可达与标题遮挡 | website/.vitepress/theme/components/SectionNav.vue；reports/756-website-copy-review.md |
| P756-D2 | low | website v05 原图负载与阅读密度 | 已有桌面首图约5.2MB、发布页较长；本轮保留有效原图与内容。后续生成展示派生图并保留完整原图/来源，另复核手机滚动和章节密度 | website/public/desktop-showcase/02-desktop-dark.png；reports/756-website-copy-review.md |

### PG-MEM（2026-10-09，playground 全量示例走查实测登记——走查副产品）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~PG-MEM-1~~ 已销账（PLAN-746，2026-10-09 交付 95c093001：VM 协作式截止+timeout_secs 默认 10s，全量走查 +11MB/挂起清零） | high | auto-playground /api/run 执行管控 | **run 接口无超时/取消——挂起执行永不终止并持续吃内存**（2026-10-09 实测：1343 示例串行走查后服务端 16MB→1434MB，其中 ~740MB 来自 5 个挂起示例各跑满客户端 90s 超时仍在服务端继续；批量结束后挂起任务继续增长到 1.77GB+ 仍不收敛；早前 8 并发批 20GB+ 同因）。挂起源三类：①`print(i, end=" ")` kwargs 形态触发 VM 高速缓冲膨胀（~3MB/s，think-python ch07 块 01/02 两围栏 +210/+348MB）——独立 VM 缺陷；②书摘真死循环（`for true {}`）；③已 #[ignore] 的 plan231 无限循环语料。清偿方向：run_handler 加执行超时（返回错误+丢弃任务），根治需 VM 燃料/取消机制 | crates/auto-playground/src/routes/run.rs:26（spawn_blocking 无超时）；crates/auto-lang/src/lib.rs:471（join unwrap）；scratch/playground-check/results_mem.json（逐示例内存曲线） |
| ~~PG-MEM-2~~ 已销账（PLAN-746：内建族 kwargs/Bina(Asn) 形态编译期拒绝） | low | VM print kwargs 形态 | **`print(x, end=" ")` 不报错反而挂起并以 ~3MB/s 膨胀**（应为编译期拒绝或忽略 kwargs）——块 01 `for i in 0..3 { print(i, end=" ") }` 90s 内 +210MB；块 02 同形态 +348MB。与 PG-MEM-1 的取消机制正交，属 codegen/内建签名校验缺口 | scratch/playground-check/results_mem.json [903][904]；think-python ch07-iteration 块 01/02 |
| ~~PG-MEM-3~~ 已销账（PLAN-746 T-04 归因修正：基线缓涨系挂起旋转窗口误判，挂起源消除后净增 +11MB≈工作集涨落，非泄露） | low | auto-playground 单进程基线 | **逐运行基线缓涨 ~1MB/示例**（148 条 >1MB，多为快成功运行，如 ch14-classes-functions 块 03/04 各 +33/+26MB/0.02s）——Windows 工作集不归还 + 每运行全局状态（惰性 native 注册/类型缓存）叠加；相比 PG-MEM-1 次要，单独 profiling 再定责 | scratch/playground-check/results_mem.json |

### P726（2026-10-02，复审登记的预存发现——非本计划引入）


| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P726-R1 | medium | book_listing 再生成工作流 | **裸 `cargo tb` 经 `generate_book_expected` 无条件改写外部书仓全部 expected 文件**（fs::write 恒写，非仅漂移项）——①破坏性：书仓在途 WIP（如 PLAN-715 期 ch06 编辑）会被当前转译器输出覆盖；②竞态：与对拍族同池跨进程并行，expected.rs 读写互踩致红集非确定（同克隆实测 7 红/16 红两采样，2026-10-02）；③新交互：PLAN-726 T-03 脏态守卫落地后，generate 改写使书仓变 M 脏 → 后续对拍全 skip（消息可归因，非静默）。清偿方向：generate 收编为显式 env 门工作流（如 AUTO_BOOK_REGEN=1）或改仅写漂移项+先备份 | crates/auto-lang/src/tests/book_listing_tests.rs generate_book_expected（fs::write 无条件）；726 §8 T-03 回执预存发现① |
| P726-R2 | low | desktop_protocol FFI 卫生 | **stage3.rs 内同名 Win32 函数双声明签名不一致**——K32GetProcessMemoryInfo（mem_ffi，isize 句柄）与 EnumWindows（两处，签名不同）跨 extern 块重复声明且类型不匹配，编译器持续告警；同 crate 同符号两种签名理论 ABI 风险（现均为 8 字节句柄/x64 实际无害）。PLAN-726 已把自家 OpenProcess 声明对齐 isize 形态规避新增告警；存量两处清偿=统一收敛到单一 FFI 声明模块 | crates/auto-lang/src/ui/desktop_protocol/stage3.rs:146-152（mem_ffi）与 :5418（EnumWindows）；cargo check 告警实录（726 §8 T-05 预存发现②） |
| REG-4 | ~~medium~~ → 已清偿（待 auto-os 合流） | kitchen-sink/schema 金样跨仓脱同步 | **schema_drift_fence / queue_coverage_drift_fence / docs_gen kitchen_sink_page_in_sync 三红族（2026-10-02 批量回归新红）**——根因=跨仓 auto-os commit `73d02b5`（Plan 717 "show all component examples"，2026-10-01 05:05）手改 widgets-gallery/src/front/pages/kitchen-sink.at 未走 `KITCHEN_SINK_UPDATE=1` 再生：时间线 0d5f5bf 重采样（09-30 14:21）→ 09-30 批量回执（10-01 00:20）三绿 → 73d02b5 手改 5h 后破红；auto-lang 窗口 c19b5351a..4614bd9df 零触 docs/components 与 schema 源；非 PLAN-726（stash 对照 pristine@17292c07e 同红，先于其 diff）。**PLAN-759 T-01（2026-10-11）已清偿**：①根因深化——非仅金样手改，Plan 717 四表同步整体欠账（别名/覆盖登记/render 臂皆缺，schema_drift_fence 的 P1 红长期遮蔽 P0 累积漂移）；②aura.at 手工补 12 别名+render_support 补 carousel/command fallback 臂+element_coverage 登记（auto-lang 986cc9392）；③金样经 AUTO_OS_ROOT 锚定 auto-os 独立 worktree 再生并提交（auto-os plan-759-dev 分支 b5a2333）——**待合流**：合回 auto-os main 后主检出/CI 的 kitchen_sink 围栏方绿（worktree 侧兄弟解析已绿） | auto-os worktree .wt/lang-759/auto-os@b5a2333（待合流）；schema_drift_baseline 刷新说明见 PLAN-759 执行记录；docs/plans/.last-batch-regression.json 2026-10-02 回执 |

### P723（2026-10-01，28应用图文介绍素材核查）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P723-D1 | medium | AutoTerm 当前本地 VM 拍摄 | 以现有auto.exe（SHA-256 d23bd5338d27259807eb31f5c3eea7457fbafdf7a0453bcfa692b82db22129de）运行auto-term app源码普通副本，run -r vm --merged，设置实际AUTOTERM_ENGINE_DLL后App.Tick仍报future_all/race: empty or invalid future list，窗口未产生有效会话，空图拒收。当前723使用2026-09-22已留存Rust原生验证图（素材commit6ff3bad0ff0f780e81e7c9c13af2f0b3f8902040）并公开说明日期/形态，AC-03真图成立；未声称当前VM功能通过。后续匹配工具链/真实App.Tick修复后重拍，不在网站任务改终端实现 | docs/reports/p723-capture-catalog.json auto-term；website/apps/demos/terminal/index.md；website/scripts/p723-capture-vm.py |

### P722（2026-10-01，网站 Apps 介绍复审）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P722-D1 | low | 网站应用开发画廊 | 网站已打包 registry 使用旧 ID（如012-stopwatch/025-dashboard），与现势源012-clock/025-sys-monitor不同；不以页面可挂载推断操作/后台覆盖。后续从源再生成画廊并逐项核验，禁止手改bundle。当前新介绍为静态分类且明确运行条件，未影响722必需验收；28候选后续素材/验证是批准设计，不计未完成实现 | Plan722；docs/specs/website/design/application-demos.md §4；docs/reports/p722-source-baseline.json（722 worktree/branch） |

### P716（2026-10-01，供③ 确认复核件降级挂账——plan_revision 2 用户裁定）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P716-D1 | low | auto-edit 矩阵 供③ 多跑复核 | **大文件卡死回归「确认复核件」未完成——多跑×3 环境阻断，018 登记保持**——auto-edit `desktop_mcp.py` 矩阵 6 次启动实录：MCP 端口 TOCTOU 死占×3（9247=ui_desktop.exe PID 2824 [auto-musk 域在跑 UI] LISTENING+孤儿 auto.exe 占 9248/9249；`pick_free_port` connect_ex 探测对 LISTENING 态误判空闲）+boot 瞬态×1+**app 楔死×2**（主进程存活但事件循环楔死：X9-ALIVE 心跳 ~90s 停+SYN_SENT→8018 挂起+MCP 监听套接字消失——18 PASS 后断连）。根因假说=712 r2 T-16 桌面轨 AppTick 泵断链残余族（形态吻合**未证实**——同二进制 01:5x 空闲时段健康推进 T1→T13 20min+，楔死与机器态相关非确定性）。供③ 语义=稳定性确认件（018 v2205 单跑 T17.2/17.3/17.8 已全绿），非新能力；本件（PLAN-716）三组主体交付与其零耦合。**清偿路径**：712 T-16..T-19 修复交付后或共栖负载空闲窗口期，重跑矩阵 ×3（AUTOUI_MCP_PORT 绕开 924x 带）——T17 全绿→018 注记销账；若楔死复现→供③ 转「复现，原诉求恢复」（m1-supply §17 语义）。018 登记位未回写（不假销账） | 主检出 docs/plans/716-m4-upstream-batch.md §10 Q-5+T-12 行实录；auto-edit specs/auto-edit/tests/desktop_mcp.py pick_free_port:47；018-m4-perf-unblock-and-budget-gate.md:341 供③ 定性 |

### P706 r2 收口批（2026-09-30，work 阶段定谳的非本回归缺陷另立记账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P707-R1 | high | async http 直驱臂（e4） | **e4 `default_headers_reach_wire_on_plain_get` 确定性红（非环境红）**——`spawn_async_http_handle`/`spawn_async_http` 直驱三臂在独占测试进程内恒不到线：曾无限挂死整跑（2026-09-30 双 tf 现行实录，其一卡死 30+ 分钟），fix-test-stability 加 30s `recv_timeout` 定界后 30.0s 恒红实证。同机 plan705 e2e（`Http.post_json` 全链）与 plan707 SSE（真 TCP 分时帧）均绿 ⇒ 机器网络正常，**头号嫌疑=plan707 T-02 `submit_detached_client_job` 消息桥分离**（fire-and-forget 路径疑似不再派发直驱 job）。4f123a50e 曾把 `default_headers` 计入"预存红 4"以环境红定责——**误标**。修复归 plan707 域；修复前日常档带此红 + 30s 长尾在案（slow-timeout 120s 终止兜底，防挂死回归） | stdlib.rs e4 模块（default_headers 三臂）；4f123a50e 定责行；.config/nextest.toml slow-timeout |
| P706-D1 | medium | back-bridge call（merged 直调臂） | **probe_mtime merged 臂 release 载体 `done` 恒 false（挂起）为独立预存缺陷**——探针 widget 零 computed/memo，Init 弧 `probe_cases` back-bridge call 不完成；A/B 定谳：old-carrier c8f86ef92（pre-delta）release **同形红**（`done: false` 超时），修复 exe debug 全案绿 → release 特异的 back-bridge 调用面，非 706/plan047/705 delta 引入。清偿需 release 档 CALL/段驱动时序定位（debug canary 先爆掩蔽同窗） | auto-jade tests/probe_mtime.mjs；`p706-probe-mtime-rel.log` vs `p706-probe-mtime-oldrel.log` vs `p706-probe-mtime-dbg.log`（组目录 .wt/lang-706/） |
| P706-D2 | low | auto-jade e2e harness（消费方仓） | **vue 轨 Playwright e2e `快速打开` menubar 弹层点击预存 flaky**——元素反复 detach → click 重试至 240s 超时；双载体统计同 flaky（old c8f86ef92 2/4 pass / new 载体 1/4 pass，无区分度）⇒ 非本仓 delta 回归；gen/front/vue 为共享陈旧产物（前端非变量）。清偿归属 auto-jade harness 加固（force-click/动画等待/焦点稳定），不在 auto-lang | auto-jade e2e/matrix.spec.ts:632 waitButtonIn click；`e2e-old-run1..3.log`/`e2e-77d05-run*.log`（组目录） |
| P706-D3 | low | widgets-gallery VM 臂侧栏导航 | **sidebar 连按导航偶发失灵（双载体同）**——MCP press 后路由不切（old-carrier 2 连按后滞留 / new 载体同现），press 间页面重挂载期 MCP 响应可 >10s；四页收口经 press→fixture 双通道重试达成。机制未查（疑页面重建期 press 竞态），消费面仅探针脚本 idiomatic 影响向 | `.wt/lang-706/gallery-spot*.log`；widgets-gallery src/front/app.at routes |

| P706-D4 | medium | plan705 e2e 门禁（tf 并行池） | **plan705 e2e 探针裸 `#[test]` 入 tf 全量池→套件挂起风险**——`plan705_e2e_deadline_cancels_parked_and_reclaims` 绑定固定端口 18511/18512+全局 env 变更（AUTO_HTTP_REQUEST_TIMEOUT_MS），tf 并行 run 实测 >62min 挂起（隔离 <3min 绿）；其自家计划 §6 明示真 TCP e2e 应入 `test-http-e2e` 门/th 串行池。706 r2 复审按 `-E` 排除后收口（单测复证在案）。清偿=cfg 门收编 th 串行池（705 域治理，随 705 系后续档） | crates/auto-lang/src/tests/plan705_spike_tests.rs:711；.wt/lang-706/tf-review-full.log vs 隔离复跑记录 |

### P708（2026-09-29，实施方案审查：待核实风险）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P708-R1 | high | outlet memo 失效 | preview/nav 等宿主局部状态未明确纳入 outlet 产物的失效纪律；UI 标脏仍可能命中旧产物。本次仅静态检查，未实机复现。缺省扩大前须验证 memo 开/关行为一致；登记风险不表示验收通过 | aura_view_builder.rs:4387/5292；renderer.rs:17214；reports/708-design-review.md R708-05 |
| P708-R2 | high | UI 连续占用 | CPU handler/resume、computed、冷态模板构建与 layout/draw 仍可同步占用 UI；702 parked 与延迟 Init 不提供 CPU 时间片。10–30s 具体归因待 T-00 分段实测；不得以首帧截图代替加载期间交互验收 | vm/engine.rs:2331/2513；components/filetree.at:39；reports/708-design-review.md R708-01/02/07 |

### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P683-D1 | low | remote wire v2 | **mesh/canvas 原语 v2 词汇面外**（G2 词汇表未列；tiny_skia `layer.primitives` 位图降级承接未实施）——试点三例无 canvas 面，首个 canvas 应用上 remote 时需 Path 序列化或位图降级（T-01 勘定登记） | headless.rs lower_layers_v2 观测臂；设计档 rq-remote-renderer §4 |
| P683-D2 | low | remote 降格 | **Transform 旋转/剪切矩阵元不可达**——iced `Transformation` 未公开矩阵元（仅 scale_factor/translation），v2 降格只产轴对齐 [s,0,0,s,tx,ty]；旋转内容（canvas 旋转动画等）remote 面暂缺 | headless.rs lower_text_v2 近似臂观测 |
| P683-D3 | low | remote 图像 | **svg 图像（tiny_skia Image::Vector）降格省略**——v1/v2 词汇 src 均为位图通道；svg 组件经 lucide 栅格化外的直引形态需 src 词汇扩展或宿主侧 svg 栅格化 | headless.rs lower_image_v2 Vector 臂观测 |
| P683-D4 | low | daemon 重放 | **shadow blur 无 canvas 原语**——v2 wire 携带 ShadowSpec 全参，daemon 重放为偏移半透明垫层（无 blur 扩散；alpha 按有无 blur 稀释）——观感近似，004 卡面 shadow-sm 实测可接受 | broker_surface.rs paint_shadow_scrim |
| P683-D5 | low | remote 环境注记 | **已销号（PLAN-690 T-07 自愈臂落地）**——daemon `WindowResized` 0x0 守卫：零尺寸（最小化路径）不向 app 转发、基线尺寸不动（恢复期假空白根除），真实尺寸恢复 resize 自然同步；管道环 p690 测试③断言绿。原走查恢复臂诉求由守卫替代 | PLAN-690 commit（rqhost.rs 守卫 + p690 测试） |
| P683-D6 | low | remote IME | **大部销号（PLAN-690 G1）**——中文 IME 组合/候选定位实机已验：组合串+候选窗在 daemon 窗内按 App 焦点框定位可见（录证 `docs/plans/reports/p690-ime-walkthrough/`），enable 下行/定位/上屏通道全链在档。**残面**：物理 IME「组合→上屏」连续实机段（commit 串经子类桥入值）在多会话争焦窗下未录成（前台锁拒合成注入；FG-FAIL 留痕），桥→上行→入值段由管道环 p690 测试覆盖（同路径构造性等价），留一键复核（安静窗内 003 remote 走查脚本即验）。**694 T-04 对账注记（PLAN-697 AC-04）**：一键复核在 694 走查期再次因环境竞夺未录成（⛔ 阻塞在案，非状态翻转）；694 复审裁定管道环=协议级等价，残面降级为可选复核——行状态维持"大部销号+残面在案"不变 | PLAN-690 win_ime.rs 子类桥 + 走查录证；plan 690 待澄清②；694 T-04/复审对账（PLAN-697 刷新） |
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P684-D1 | low | 027 back API | **vue 轨写操作无后端端点**——rev7 只做了读面（fs_list/fs_home/fs_drives）；重命名/粘贴/新建/删除在 vue 轨统一守卫提示「写操作仅在 VM 轨可用」。后端加写端点（fs_rename/fs_mkdir/fs_delete/fs_copy 契约+impl）即可让 gallery 内嵌全功能，形态沿用 thin 契约纪律 | examples/ui/027-file-manager/src/back/api.at；src/front/app.at 写守卫四处 |
| P684-D2 | low | 027 媒体面 | **vue 轨图片缩略图缺失**——image.thumb 为 VM 轨媒体管线专属（256px rendition 渐进浮现），vue 轨条目 thumb_src 恒 ""，网格视图回落 FileIcon。补齐需后端缩略图端点或 media 路由跨轨化 | examples/ui/027-file-manager/src/front/app.at NavTo vue 分支 thumb_src:"" |
### P693（2026-09-23，Plan 693 native-exe-tri-mode work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P693-D1 | low | a2r 生成物存量 | **rust-workspace 既有 member 的生成 Cargo.toml 仍含 `ui-gpui = ["auto-lang/ui-gpui"]` 行**（PLAN-691 移除 feature 前生成）——存量 member 重复 resolve 即败（fresh 生成已被本计划生成器根修）；各 member 下次 `auto build -r rust` 重生成自然收敛，禁手工清（生成物 never-track） | rust_ui.rs generate_cargo_toml（已修）；主检出 examples/rust-workspace/011-calculator 等存量 |
| ~~P693-D2~~ | medium | desktop 接入跨仓配对 | **已销号（PLAN-698 T-04，2026-09-23）**——形态勘定=子进程（库形态被 winit Windows 单事件循环结构性否决，RecreationAttempt 实锤）；`spawn_desktop_daemon` 孵化 `auto rqhost --pipe autodesk-rqhost-desktop-<pid>` + AUTO_DESKTOP_ENDPOINT env 注册表发布（命名裁定定稿）；003 converter.exe 全环录证（adopt→开窗→v2 帧→EOF 回收→末窗自退，reports/p698-batch/T04-desktop-pairing.md）。SD-04 落 ui/overview.md；WM 深度集成仍归 auto-os 侧后续 | renderer.rs run_session Desktop 臂；rqhost.rs spawn_desktop_daemon；desktop.ps1 |
| P693-D3 | low | a2r 测试漂移 | **`merged_api_client_crud_fallback_for_uncovered_endpoints` 预存红勘定新增**（不在既有已知红清单）——测试期望旧 CRUD 兜底形（`static API_DATA` + `-> Vec<Value>`），实发为 PLAN-681 route A `api_impl` 伴随模块形；base rust_ui.rs 外科对照同败（与本计划 diff 零交集）。修 = 测试期望随 route A 形更新，归 PLAN-681 线或独立 L0 | rust_ui.rs:4813 测试 vs generate_merged_api_client route A 臂 |

### PLAN-698（2026-09-23，Plan 698 desktop-substrate-hardening-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P698-D1 | low | submenu 浮动式残余 | **像素级确认被 D5 screenshot 通道阻塞**——任意 menubar 开态 autoui_screenshot 10s 超时（基线零改动同现=通道级预存，与浮动/内联形态无关，判位留痕 T03 工件）；浮动面板截图验证待 D5 清偿后补一轮。交互细节：子面板开态点击=项动作+菜单关闭并存（shadcn 语义可接受但不精细）；hover 换子菜单归 P695-D1 键盘/hover 导航批 | popover.rs Panel::overlay；reports/p698-batch/T03-submenu-floating.md |
| P698-D2 | low | desktop 端点生命周期/接线 | **daemon 子进程与宿主无级联**（Windows Job 对象挂接缺——宿主强杀后 daemon 按末窗语义存活，零窗常驻）；注册表 launch 面尚未消费 AUTO_DESKTOP_ENDPOINT（env 发布就绪，launcher 自动带 `--desktop-endpoint` 拉起 a2r exe 归 auto-os shell 线）；虚拟窗内嵌/键面深集成归 P694-D2 | renderer.rs run_session Desktop 臂；desktop.ps1 |

## PLAN-692 债务候选（2026-09-23，W-2/W-1 走查批）

1. ~~**ui-cache 键缺生成器版本（infra，高优先）**~~：**已销号（PLAN-698 T-01，2026-09-23）**——生成器指纹进缓存键（build.rs hash src/ui_gen/** → AUTO_UI_GEN_FINGERPRINT；UICache.generator_fingerprint 不匹配整缓存失效重签，VERSION 1→2 迁移弃旧）；692 W-1 场景实机回归绿（改生成器→自动失效重生成，免手工清）。入库评估=生成数据注记落 SD-01（runtime/overview.md），入库策略不阻塞。（锚点：ui_cache.rs；build.rs）
2. **popover 选中后不自动关闭**：Combobox 选中候选项后 popover 保持打开（shadcn 官方为选中即关）。需 popover open 状态绑定 + CommandItem @select 联动关闭。
3. **VM(iced) 臂 command 家族未实现**：八 command 元素 `iced: none` 维持——VM 臂渲染降级。接线（含 `size` prop 的 iced scrollable 宽度）另立。
4. **reka sizes 初始测量 18px 下限（观察项）**：本仓环境下 ScrollArea thumb 长度常命中 reka getThumbSize 的 18px 下限（A/B 证实与实现无关；拖拽数学内部自洽，仅视觉比例偏小）。
5. **脚手架 h1-h6 基础 margin 内联隐患**：基础样式给 h1-h6 的 mt/mb 对"内联进 flex 行"的标题同类泄漏（PLAN-692 W-3 实证一处）；其余位置待观察，出现即补 m-0。

### P694（2026-09-23，Plan 694 desktop-dogfood work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P694-D1~~ | medium | 桌面宿主稳定性 | **已销号（PLAN-697，2026-09-23 外因结案）**——勘定归因档案 `docs/plans/reports/p697-stability/attribution.md`：机器级外因（内核 bugcheck 三日三蓝屏 0x9F/0x1A/0xA + GPU TDR 族；走查窗口 System Event 6008 = 09-23 10:59:12 非正常关机实锤），蓝屏瞬时吞进程 = 无 panic 无错误行无 WER 应用崩溃，与观测签名逐项吻合。嫌疑面 back-proxy lazy-start/media 供给/inproc panic 边界经 13 轮复现矩阵（基线窗口化×5+全屏×2+禁A×2+禁B×2+他App×2 全绿零死亡，嫌疑面点火实证）+ WER 零宿主崩溃记录三重排除。观测加固落地：ui_desktop 宿主 `[ui-desktop] start/alive/exit` 足迹桩 + panic 钩子（死亡判位表在档案；CLI 路径 X9 形制已在）；对照缝 AUTO_NO_BACK_PROXY/AUTO_NO_NATIVE_MEDIA（勘定器械，生产零影响）。残面=机器稳定性本身（GPU 驱动/内核，非本仓代码面） | 归因档案 p697-stability/；examples/ui_desktop.rs 足迹桩；back_provision.rs 对照缝 |
| P694-D2 | low | 桌面 shell 输入面（VM 轨） | **launcher 全量列表导航在 VM 轨不可用**——搜索框键入（物理字符派发为 `key_<char>` VM 事件，input widget 不收 CharTyped）、滚轮滚动、方向键（Named 键不路由至 bind 表）三路实测均不通（028-launcher overlay）；行击直点正常。本计划以 recent 存储夹具绕行完成发现面录证；shell 侧输入路由修复归 auto-os/shell 线（本计划非目标） | 028-launcher overlay；session.rs 键盘路由（key_<char> 派发臂）；walkthrough 走查留痕 docs/plans/reports/p694-dogfood/ |

### P695（2026-09-23，Plan 695 menubar-widgets-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P695-D1 | low | menubar 键盘面 | **方向键/Enter 键盘导航未实现**（T-08 时间盒裁定）——需 iced 焦点基建（逐项 focus 节点 + 键盘订阅 + 索引态进 MENUBAR_OPEN 注册表扩展），Esc 关闭已走 Popover 既有捕获（AC-04 最小集满足）。实施时对齐 shadcn：↑↓ 移项 / → 进 submenu / ← 回父级 / Enter 激活 | convert_menubar_component；menubar-snapshot.md PLAN-695 扩注 |
| ~~P695-D2~~ | medium | VM submenu 浮动式 | **已销号（PLAN-698 T-03，2026-09-23 翻案 T-11）**——真因勘定=iced 0.14 官方嵌套协议（overlay::Nested 递归 Overlay::overlay 钩子）未实现，嵌套 Popover overlay 永不注册；Panel::overlay 收集钩子落地（绝对坐标零平移）+ 视图臂浮动恢复（RightTop），内联降为 AUTO_MENU_SUBMENU_INLINE=1 降级路径。契约单测+快照+全程零死亡佐证；残余（像素截图确认被 D5 通道阻塞+交互细节）转 P698-D1 | popover.rs Panel::overlay；aura_view_builder 浮动臂；reports/p698-batch/T03-submenu-floating.md |
| P695-D3 | low | autoui-verifier × menubar | **MCP 合成指针事件 × hover-switch 干扰**——合成 press 的指针位移穿过被包裹 trigger 触发非预期 toggle，菜单开合状态在 MCP 驱动下呈非确定（真实鼠标语义不受影响）；autoui-verifier 对 menubar 的状态断言须容忍切换抖动或改坐标无关派发（menubar-snapshot.md 扩注已记） | aura_view_builder MouseArea 包裹臂；T-11 走查实录 |
| P695-D4 | low | gallery golden | **已销号（fold 门内完成，25cb53fa7）**——基线重基线落定（理由随提交：auto-os master 演进 + menubar.at 语料定稿），复跑对账绿（1108s exit 0） | crates/auto-lang/tests/fixtures/gallery_vue_golden.txt |
| P695-D5 | low | auto-edit 截图通道 | **auto-edit VM 臂截图与 1s 状态栏 Tick 争用（预存特征）**——无菜单态 screenshot 亦恒超时，AC-06 对照截图不可得；可读性由 token 断言链（bg-popover light=白/popover-foreground=墨）+ 快照结构验证 + probe_menu.py 承载。修复归 041/render 时序线 | tests/probe_menu.py（exit 0 两轮）；T-13 走查实录 |
| P695-D6 | low | gallery Vue 臂活体 | **widgets-gallery Vue 臂活体走查环境受阻**——worktree 无 front workspace/node_modules，`auto run`（vue）宿主窗不自动起 vite（3024 恒 000）。unblock：宿主窗交互启动 web 或主检出跑 692 同款全量再生成流程。SFC 发射面已由 p695 vue 单测 + gallery_properties schema 符合门 + fold 前 gallery_golden 确定性覆盖 | T-11 执行记录；.auto/ui-cache.json（合并后须再生） |

### P696（2026-09-23，Plan 696 HTTP runtime hardening 复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P696-D1~~ | medium | VM pubsub / publisher SSE | **已销号（PLAN-698 T-02，2026-09-23）**——bus.subscribe 转真运行面（案①进程内 iterator 注册表：EVENT_BUS broadcast::channel(256) 镜像生成 events.rs + 转发线程 + AsyncHttpStream 臂，696 回收语义沿用）；POST publisher 臂 publish_post_broadcast（Typing/New{Type} 对齐 api_gen，has_sse 门控经 record_api_return_type 新侧信道）；017 VM 臂全环录证（SSE 帧与 Axum 形态逐字节一致 + 带外 POST typing UI 渲染 typing 指示）。§8.1 落表+seam 注记划线 | stdlib.rs shim_bus_subscribe/EVENT_BUS；http_server.rs publish_post_broadcast；reports/p698-batch/T02-vm-bus-subscribe.md |
| P696-D2 | low | VM HTTP 生命周期 | **VM `#[api]` listener 尚无 graceful shutdown API**——本文 §7.3 只记录目标行为；本计划落实了 body/SSE producer 与断连连接任务的回收，不包含 Ctrl+C/SIGTERM 停止 accept、排空活跃连接的服务级接口。后续实现时需定义关闭信号、listener 停止和活跃连接排空契约。 | `docs/specs/stdlib/design/http-server.md` §7.3/§8.1；`crates/auto-lang/src/vm/ffi/http_server.rs` |

### P701（2026-09-25，Plan 701 供料包复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P701-D1 | medium | VM i64 算术面 | **.at 层 TAG_I64 操作数的算术解码错位**——`time.now_ms()/1000` 实测归 0（DIV 臂按 i32 解码 TAG_I64 nv；SUB 等同疑）；now_ms 全宽值在 nv 栈/unified print/桥面（nv_to_pub_value 已补臂）正确，唯 .at 内算术不可用。下游 bench「.at 内差值形」的前置清偿件（平台级，另件）；工作期探针 #[ignore] 留档 | tests/plan701_supply_probes.rs `vm_time_now_ms_agrees_with_now_sec_at_level`；specs/auto-lang/vm/design/time-natives.md 桥面节 |
| P701-D2 | low | trans JS 轨 | **JS 轨 print 模块内遮蔽（预存运行期边界）**——javascript.rs 维持裸 `console.log`（JS golden 字面断言，改道已回退 c9901f762）；store `console` 字段模块内在 JS 轨运行期同形遮蔽，无 TS2339 编译面。SD-03 已改口注记；如需清偿随 JS golden 重基线一并 | crates/auto-lang/src/trans/javascript.rs:427；specs/auto-lang/ui/design/vue-use-fn-emission.md strict-TS 节 |

### P702（2026-09-25，Plan 702 ui-handler-async work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P702-D1 | low | busy 标志 .at 查询面 | **`__busy_handlers` 镜像仅宿主读路径可查**——根态运行期追加字段不在声明字段表，MCP read_all_state 不枚举；.at 编译期 GET_FIELD 按静态 field_idx 对运行期字段不可达。".at 原生查询"（如 `.store.__busy_handlers.len()`）需合成期注入保留字段，等真实查询需求立项（重入忽略行为本身不依赖该面） | vm_bridge.rs sync_busy_flag；plan §T-04 口径偏差注 |
| P702-D2 | low | 释放档全量红 | **`cargo test --release -p auto-lang` 非维护门**——base 上即有 2 处 tests/ 编译 rot（本计划顺带修复：headless `()` Renderer 运行期门→编译期门、osconfig_integration photo_root 漏改）+ ~254 档位环境红（全部同名测在 tf debug 全档绿）。release 档红册清点/维护归属后续档位治理 | release_full3.log 分诊；tf_full2.log 5700/5710（10 红全预存在册） |
| P702-D3 | low | CPU-bound handler | **段驱动无抢占**——CPU-bound handler 仍占满一次 update 的 10M 步预算（WARN 后段中止），UI 冻结面在；裁定建议=ui_lint+BudgetExhausted 段切两步走（决策工件 docs/design/34 §Q2，待用户裁定后另立计划） | drive_handler_segment budget 臂；Design 34 |
| P702-D4 | low | musk 人在环走查 | **E7 原始事故端到端（PickFolder 原生对话框滞留>60s 后选择）留人工走查**——机制面已由探针 032 同驱动路径全链证实（park/交互/resume/落账 + 零忙等超时）；musk 腿需 auth/daemon 栈 + 人在环对话框操作。复跑手册 docs/plans/evidence/plan702/README.md | evidence/plan702/README.md musk 节 |

### CLEANUP（2026-09-27，主检出清理：fix-music-slim 弃置裁定登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| CL-1 | low | media scan 协议瘦身（P041-D6 残件） | **fix-music-slim 分支整支弃置（复核否决，非合并）**——c60eeaf56（瘦身：base 顶层一发+stream 相对路径，退役 url/audio_url/video_url/name/rel_dir）文本可合但有双臂硬伤：① rust 臂发射器 `auto-man/src/api_gen.rs:1789` 无 base/stream（相对 url 旧形），合入后 020 默认形态（`api:"rust"`）`media_base+stream` 拼空串播放死；② rel_dir 退役破 030-video-player（playlist.at:92 按 rel_dir 分组；共享发射器 back_proxy 服务画廊嵌入臂，"前端零消费实证"只覆盖 020）。其动机（to_value 大体转换上限疑虑）已由 PLAN-042 e7efcaa28 幂等根修+ScanProbe 回归钉实质消解。**复活前置件**（=一个 L1 计划）：双臂发射器同步出 base+stream（rust 臂 base=""同源相对；proxy 臂 base=绝对 origin+apps 前缀）+030 rel_dir 保留或迁移+两臂测试面+spec SD-02 契约行同步（overview.md:867/871）。弃置提交 SHA（gc 前可恢复）：c60eeaf56（瘦身本体）/f0bcd6e42（wip 调试桩+本机 media_root 路径，不具合入价值） | crates/auto-man/src/api_gen.rs:1789；crates/auto-lang/src/back_proxy.rs media_scan；examples/ui/030-video-player/src/front/playlist.at:92；docs/specs/auto-lang/ui/overview.md §SD-02 |


## 首次批量回归（/auto-plan:regress）新发现预存红登记（2026-09-30，PLAN-710 merge 触发首跑）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| REG-1 | low | book listing 档 | **book_ch06_09 / book_ch09_02 预存红**——首次 `cargo tb` 批量腿暴露；基线检出（c80887ab7，710 落地前）单跑复现同红=预存非 710 回归（golden 漂移候选，修复归 book 域另档） | .tb-full.log 首跑；fix-regress-attr-p710 基线复跑 |
| REG-2 | low | ffi_dual 批量档 | **ffi_dual_019_dep_layout_invariants 批量负载 flake**——tf 批量（5921 并行）红、pre/post-710 双基线单跑均绿（3.27s/2.97s）→ 并行负载态 flake 非 710 回归；复现窗口=全档并行期，后续批量跑观察复现率 | .tf-full.log 首跑；双基线隔离复跑在案 |
| REG-3 | low | app_registry 档（scan/launch） | **共享 target 撞名环境态红**——`convention_native_exe` 候选面含仓根共享 `target/<build>/<name>.exe`，examples/ui/041-auto-edit（name=auto-edit）与他会话在主检出 target/debug 建的 a2r 工作区产物 auto-edit.exe（PLAN-021/r3 系 corpus/复验构建，76MB@2026-09-30T20:16）撞名→exe 背书判定翻转→按 PLAN-694 语义退出 C 档策展钉面（scan 19 vs 20 红+launch 改走 exe 臂超时红）。归因实证：artifact 改名隔离后 26/26 全绿、复位后复现；非窗口内任何计划代码回归（7fcf913eb..HEAD 零提交触 app_registry/examples）。脆弱面=测试设计（共享 target 候选过宽+应用/示例同名），修复候选=共享 target 候选剔除或策展测试对 exe 背书噪音隔离——归 ui 域另档； artifact 在场期间批量跑按本条豁免 | 改名隔离复跑收据（2026-09-30 p714-regress，26/26）；convention_native_exe 源码面 app_registry.rs:162-200 |
| REG-1b | low | book listing 档 | **REG-1 同族欠账注记**——ch02_05/ch03_08/ch06_05/ch06_06/ch06_08 五枚与 REG-1（ch06_09/ch09_02）同签名（`aborting due to N previous errors` parse 形）同族：输入 main.at 均为 book 仓已提交态（WIP 仅 expected.rs 再生面）、auto-lang 窗口 7fcf913eb..HEAD 零 parser/lexer/依赖变更（Cargo.lock 零 diff）→ 解析行为不可能新变=covered_commit 时已在案，上批收据 known_reds_seen 欠录 5 枚（tb 腿红清单未全录——本批补全）。修复归 book 域另档同 REG-1 | .tb-full.log 七枚同签名核验；book 仓 git status（WIP=expected.rs only）+Cargo.lock 窗口零 diff 实证 |

## PLAN-715 批量回归登记（2026-09-30，收据 a984834a2）

- **并行负载序敏感 flake ×2（新登记）**：`ffi_dual_019_dep_layout_invariants`（tf 档）、`plan502_m3_layout_geometry_e2e`（tt/tb 档）——批量全档并行跑偶发红，隔离单测与整族隔离均稳定绿（ffi_dual 23/23×2 轮）；与 p053 族已登记的"并行负载序敏感——隔离可绿"同类。无语义归因（窗口内测试文件与被测模块无对应改动）。处置：批量档观察名单，若频率上升再立项查并发交互。
- **book_listing 外部书仓敏感性（登记观察）**：`tests/book_listing_tests::*`（tb 档，ch02_05/ch03_08/ch06_05/ch06_06/ch06_08/ch06_09/ch09_02）输入源为外部兄弟仓 `D:/autostack/book/rust/listings` 的实时内容——该书仓在途编辑（d7a71a7 重生成金样 + ch06 金样未提交 WIP）期间这批测试确定性红，随书仓落定自然收敛。本仓窗口对该路径零改动；book 仓编辑会话应在落定后复跑 `cargo tb book_listing` 确认。
- 既有基线沿用：712 §10② master 14 红族（p053×4/p054×2/plan606/desktop_protocol/iced renderer/e4/plan707/app_registry×2）本轮复现一致（native_gate 本轮未现=成员漂移）；a2vue_desktop 金样=4f123a50e 在案；plan484 streaming=711 §10 在案。

## 批量回归登记（2026-10-20，收据 covered 817，续 815 批）

- **state_file 并发对并行负载 flake ×2（新登记，观察名单）**：`state_file::tests::concurrent_atomic_writes_never_tear` / `lock_serializes_critical_sections`——tf 批量并行池红（0.207s/0.330s），solo 3 轮全绿（0.7-0.9s）；`state_file.rs` 在覆盖窗口 `2772c8012..5594158a4` 零提交（PLAN-044 远古模块，无计划归属）。与 REG-2/THR-D4（ffi_dual_019、plan502_m3）同类并行负载序敏感；处置=批量档观察名单，频率上升再立项。本批 ffi_dual_019 复现一次（同签名：池红 2.98s/solo 绿 3.56s，REG-2 在案）。
- **tt 首 fail-fast 截断注记**：nextest 现行为默认 fail-fast，`cargo tt` 首轮被已知 plan484 charts flake（812 收据 known_reds_seen 同族，solo 3× 绿）在 2237/5685 截断；完整轮须 `--no-fail-fast`（本批 tf/tt 完整轮均带）。后续批量跑建议直接带该旗。


### PLAN-721 批（2026-10-01，用户裁定「app 修改到此为止」余题登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P725-R1 | medium | 帧管线单帧单建的自动化回归守卫（PLAN-725 复审 P725-R1） | builds=1 断言与 MCP 快照等价对拍未自动化——现行证据=在仓双态谱 JSONL（56/56 typed 帧 builds=1）+可复跑脚本 ladder.py（AUTO_FRAME_BENCH 门控探针在库）；CI 时态守卫需 headless iced-session+MCP 测试基建（现仓无此 harness——musk/renderer 轨测试均不驱动 view() 全路径）。下游帧档（type_latency）为第二道语义守卫。 | docs/plans/evidence/725/T-00-survey.md §2；plan725 §9 复审记录 |
| P725-R2 | low | 键入载荷退役的计数断言面（PLAN-725 复审 P725-R2） | 1MB clone 计数断言未自动化——MCP 驱动走模型层直写不触发 on_change 闭包（实测约束，s1 探针全程 0）；谓词决策已单测（input_payload_consumed_contract）；真实键盘路径的诊断面=s1 分段探针（在库）。 | plan725 §9 复审记录；SD-01 §3 |
| P721-R1 | high | T-19 读侧机理（暂停回弹根因末环） | **PLAN-725 关联注记（2026-10-02）**：帧管线增量件 T-00④ 读侧一致性核查——MCP 驱动键入谱（[P725-FRAME] 分段，30 键×3 档）未现形相邻构建同 binding 读值漂移证据（编辑路径负载面）；机理定谳入口不变，仍在 T-19。增量缓存正确性纪律遵守=frozen⑤（本件零「跳过重解析」快道引入——T-03 memo 扩面裁定不扩）。 |
下行 epoch 单调门只挡「世代回退」子类；桌面实测回弹写来自 **OnTime 触发的全新构建烤回 paused=false**（handler 两次写均 false、同一 binding 相邻构建读值不稳定）——读侧世界分裂/陈旧 memo 回放机理待定谳。复现即现形：`AUTO_SCHED_DIAG=1` 桌面跑 030 点暂停看 `video_build` 行。 | evidence/721/apptick-verdict.md 结论二；desktop-721g.log |
| P721-R2 | medium | T-11 播控条布局塌缩 | col_right 438↔658 进程级翻转非确定性源未定谳（712-r2 登记态原样移交）。定谳入口=进程内双构建比对 + builder 侧 HashMap 遍历审计（style 层 Vec 面已初查非嫌疑）。 | 归档 712 §10①；layout_tests.rs p712 探针族 |
| P721-R3 | medium | 030 seek 派发被 arity 守卫跳过 | 用户实机取证（030-standalone4.log 07:15）：进度条点击/拖拽 → `[VM-ARITY] Controls.SeekFraction declares 1 param(s) but dispatch supplies 2 — skipping` ×6——seek 完全失效。疑拖拽事件路径多带一参（712 T-12 `$0` 剥离同域）。 | 030 controls.at SeekFraction 绑定；plan-576 D4 守卫 |
| P721-R4 | medium | 验收通道注入静默丢 | desktop-721i 实例 handler inject 排队后零派发（g/h 同代码正常）——registry_id 窗口反查在窗重用/多实例下疑 miss；inject 臂 `let _` 吞错无日志面。验收通道可靠性债。 | renderer.rs apply_desktop_injects Handler 臂 |
| P721-R5 | low | 独立窗 ok=true 自退（seek 后） | 030 独立轨在用户 seek 交互后 `vm interpreter returned ok=true` 优雅退出（无 wgpu 告警，非 712 T-07 press 死因；用户主动关窗不可排除）。复现则按 T-07 同窗定谳。 | 030-standalone4.log 尾段 |
| P721-R6 | medium | T-19 桌面活体 ×10 复验 | epoch 门落地后桌面实机暂停 ×10 无回弹未走查（实例 I 用户占用+R4 注入丢+712 T-07 三重阻塞）。移交用户日常观察；若仍回弹=R1 机理现形入口。 | evidence/721/apptick-verdict.md 未竟段 |

### PLAN-712 r3 批（2026-10-01，归档收据 eed6acf25——未竟/登记项集中挂账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P712-D1 | high | 桌面轨更新路径家族（PLAN-721 承接中） | **PLAN-725 家族注记（2026-10-02，worktree plan-725-dev）**：编辑路径帧管线分段谱定谳（ui::frame_segments——五段 S1..S4+builds/dirty，AUTO_FRAME_BENCH 同门）：S2 VM 段非瓶颈（0.07-0.33ms/帧）；S3 恒定 ~8ms；S4 曾随文档尺寸线性爆炸（root=iced scrollable 构造期 size_hint→content_height→fresh_fold_map 逐行物化，1MB 95ms/帧——已修，无折叠快道 O(1)）；Element 缓存 put-then-take 死缓存勘定（快道结构性不可能命中，死写移除）。双态谱+勘定=docs/plans/evidence/725/（下游 auto-edit M4 帧两行 FAIL 的上游清偿面）。 |
**推进中（2026-10-01，PLAN-721，worktree plan-721-dev @088b32aa6）**：①**T-16 结案**——r2「桌面内嵌轨 0 trace/Loading 卡死」定谳为伪象或已被 712-r3 治愈（sched_diag 四轴 trace 全绿：sub_parked/到达/恢复 50.9ms/置脏回填，018 内嵌实机复验）；②**T-19 帧级序列捕获**：暂停写全链落盘（mpv 真停+上行回灌）后 ~100-185ms，OnTime 触发的**全新视图构建烤回 paused=false**（同一 binding 相邻构建读值不稳定=读侧世界分裂/陈旧 memo 回放，机理开放；video_build trace 已就位）；③**已落部分修复**=下行世代单调门（VideoContractDown.epoch + contract apply 回退整包拒绝），mpv_contract 13/13 绿（712-r3 双锁零扰动），独立轨真实按钮暂停钉住 7s+ 零回弹；④**T-11 未动**（col_right 438↔658 定谳入口不变）。定谳文档 docs/plans/evidence/721/apptick-verdict.md（worktree 088b32aa6）。另：验收通道 handler inject 在 desktop-721i 实例静默丢（排队零派发，g/h 正常）——验收通道可靠性债，随 PLAN-721 待澄清②复查。 | 归档 plan docs/plans/archive/712-vm-desktop-defects.md §8.5.3/§9/§10①；PLAN-721 plan 文档 |
| P712-D2 | ~~medium~~ | 018 阅读链点验未完成 | **已清偿（2026-10-01，PLAN-721 承接批前置点验）**：独立 VM 单跑走查全绿——书架 3 本 → 点第 2 卡开对书（route /book/2，712-r3 Init 代际修复实机复证）→ 详情页标题/作者/「3 entries」+ 章节目录 3 章（884bcfa67 app 层在位）→ 点 Chapter 2 → route /book/2/chapter/2 + **`router.param("ch").to_int()` 堆化形态实测=2**（712-r3 第二层 materialize_obj_to_heap 修复实机复证）→ reading 页渲染章节标题/正文/「Chapter 2 of 3」/66.0% → Next → 第 3 章 100.0%（路由参数变化 Init 重跑在位）。载具=lang-721 worktree 二进制（master+T-1 门控插桩，行为面零差异）。截图 docs/plans/evidence/721/d2-reading-ch2.png。 | 018 book_detail.at（OpenChapter → /book/:id/chapter/:n）；reading.at:146 |
| P712-D3 | medium | ui_desktop 壁纸克隆 OOM（env 缺陷） | 桌面启动数秒~数分钟死于 `memory allocation of 2660706 bytes failed`——`load_image_bytes`（renderer.rs:6590）**每次视图重建裸 clone 壁纸字节**（purple.png 2.55MB），提交压力尖峰下 OOM 金丝雀。master 构建同崩（A/B desktop-master-ab.log）= 预存非 712 引入。修法候选：壁纸字节/解码结果缓存一次（挂 config 或 handle 缓存，Plan 650 缓存族同源）。本日 4 份崩溃日志存 worktree 组目录（desktop-r3*.log，未入库）。 | 归档 plan §10⑧；renderer.rs:6590/15354 |
| P712-D4 | low | release 档产物陈旧 | release 二进制 + `gen/front/vue/dist` 未随 r3 合并重建（PLAN-092 先例观察项）；桌面实机消费面=worktree debug 二进制形态。 | 归档 plan 合并收据 cleaned 段 |
| P712-D5 | low | 018 vue 轨 json builtin TS 报错 | `auto build`（vue 轨）`useBooksStore.ts` TS2552 `Cannot find name 'json'`——生成器未把 `json.from_value` 等 builtin 映射到 TS 侧（book_store.at:23 遗留；VM 轨不受影响）。018 vue 轨构建失败为存量，非 r3 引入。 | 018 book_store.at:23；ui_gen/vue.rs builtin 映射面 |
| P712-D6 | low | worktree 清理 pending + 僵壳 | `D:/autostack/.wt/lang-712b/auto-lang`（分支 plan-712-dev，已全部 landed @68000a5bd）待删——**阻塞=验收桌面正从该 target 运行（用户复验载具）**；用户复验完成后：杀进程 → `bash D:/autostack/wt-guard.sh` clean → git worktree remove + branch -d。另 lang-712 旧残壳（.tmp-712-app.log + 空目录链被僵进程 PID 35340 锁死）待**重启后**删组目录。 | 归档 plan 合并收据 cleaned 段；§10⑤ |
| P712-D7 | low | 语料过时注释 | viewport.at 头注「`flex-1` 与 `shrink-0` 在 VM 不生效」已不成立（Plan 370 Issue 1 起 flex-1→width=Fill 在位）——误导排查，待语料清理批顺带更正。 | 030 viewport.at:10-12；iced_adapter.rs:1158 |
## PLAN-718 网站执行登记（2026-10-01，T-07 视觉复核发现）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P718-D1 | low | website /playground hero | **Playground 页 hero 大标题 "Playground" 字形顶部被水平裁切**——1440 桌面 dark/light × EN/ZH 四变体一致复现；已核对 715 已入库截图（docs/reports/p715-website-ui/final-playground-en-1440-dark.png）同缺陷=既存，非 718 引入（718 未触 /playground 样式）。疑似 hero 标题 line-height/overflow 裁剪，一处 CSS 可修；不扩围，留独立修复 | docs/reports/p718-website-ui/VISUAL-REVIEW.md；docs/reports/p715-website-ui/final-playground-en-1440-dark.png 对照 |
| P718-D2 | low | website playground i18n | **en 版 /playground 语料树/按钮等组件文本为中文**（搜索标题或标签…/书籍示例/在 IDE 中打开）——auto-playground-vue 包组件级 i18n 未覆盖既存缺口；包源码不在网站计划可编辑范围 | docs/reports/p718-website-ui/VISUAL-REVIEW.md 次要观察；packages/auto-playground-vue |
| P718-D3 | low | website zh docs 侧栏翻译完备度 | **zh docs 侧栏残留英文条目**（Components/Script to ship/Tour/Generics 等）——ZH_TITLE_MAP 未覆盖的目录/文件名回退，内容翻译范畴非视觉缺陷 | docs/reports/p718-website-ui/VISUAL-REVIEW.md 观察项 3；website/scripts/prepare-content.js ZH_TITLE_MAP |

## PLAN-727 复审登记（2026-10-02，复审 @plan-727-dev 1fa1a49a4）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P727-F1 | medium | book 仓 listing 语法脱同步 | book `rust/listings` 的 is 臂仍用旧语法 `0 =>`（book_ch03_08/ch06_05/ch06_08/ch09_02 四例），现行解析器只接受 `0 ->`（语料 test/vm/04_control_flow/006_is_stmt 即新语法）——任何全新 worktree 的 `cargo tb book_ch*` 必红；主检出单跑通过属其 target 增量构建本地假象（`cargo clean -p auto-lang` 后复测仍过，其增量状态未深究；判定依据=基线提交 b34852532 全新 worktree 复现同红，probe worktree 已 guard+清理）。修复归属 book 仓（listing 再生成或语法升级），auto-lang 侧不动他仓。复审实现会话独立性局限已声明（结论由工件重建：HEAD scoped 复测绿 + 大宗门禁红逐名对照零新增）。 | docs/plans/reports/727-transfer-verification.md §3；727 §9 复审记录 |

## PLAN-728 执行登记（2026-10-02，work @plan-728-dev 3bc929ca0/c19fec79f）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P728-D1 | medium | 分页 rope 行查询的页内换行重扫 | `line()`/`line_start_byte()` 每次调用对其所在页做 O(页) 换行扫描（64KB 页 ~千行级重扫/次）——`find_next` 全文档扫描在 50MB 档实测 ~30s（debug）/answer 线 3ms（release 暖缓存后）。优化方向=页级换行偏移索引（PageDesc 增量数组）或行游标缓存；触发条件=下游 1GB E2E 消费件裁定优先级（头窗口内打字/跳转不受影响）。 | rope.rs q_line_start_byte Chunk 臂；SD-01 帧域协同节 |
| P728-D2 | low | 703 对齐行走字节基修正的回归面 | 顺手修复：`range_content_equal`/`collect_prune_spans` 的 leaf 比较从 str 切片改字节切片（对齐切割在对侧内容上可落字符中间位——纯 ASCII 语料 703 期从未踩中，分块树形态分歧后确定性触发 panic）。语义不变（字节精确比较），但 703 既有测试族覆盖的多为 ASCII/对齐友好语料——非 ASCII 分歧对拍面靠本件 dual_track/paged_snapshots 补位。 | rope.rs LeafView::bytes 注记；file_backing::tests |
| P728-D3 | low | >50MB 档交互面窗口化边界 | 头窗口（2MB）外交互体验边界如实登记：find 匹配落点重物化窗口后 published 光标行=窗口相对坐标（模型绑定下游需知悉）；fold 面在 >50MB 档返回空图（折叠发现=全文档扫描，护栏早退）；`code_editor_text` 全量读出在 >50MB 档仍 O(n) 物化（Plan 413 事件面消费——下游适配件改走 doc_snapshot）。三项全部=auto-edit 消费件范围（供料档 §10 回执预告位在列）。 | SD-01 双轨形态节/回执节；auto-edit de6cb95 |

## PLAN-729 执行登记（2026-10-02，work @plan-729-dev，R1 复审修复后）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P729-D1 | low | shutdown drain 专项 wire 演练 | 文件 body 在优雅关停排水窗内的收尾未做专项 wire 测试（scope 收口臂与 SSE FrameStream 共用路径已由共用代码覆盖；drain 窗语义沿 serve_with 现状）。补测需以 test_trigger_shutdown 驱动 + 慢 body fixture。 | http_transport.rs serve_file_seed/serve_with drain；729 lifecycle 报告 §3 |
| P729-D2 | low | legacy stdnet 文件 500 诊断 wire 档 | serve_blocking_stdnet 的文件端点 500 诊断为代码面实现（fn_is_api_file_return 门 + 诊断写回）；legacy 为非默认调用图（plan705 spike 锁单调用点），无独立 wire 档。补测需手工 VM 构建管线。 | http_server.rs legacy 分支；SD-01 §6 矩阵注 |
| P729-D3 | low | 端口可重绑专项 | e2e 服务器线程为 detached（既有基建约束，与 http_e2e 全族一致）；"结束可重新绑定端口"未做专项断言（每测试固定新端口规避）。 | plan729 e2e 基建注 |
| P729-D4 | low | 排队取消的直接观测 | 文件排队项无外部取消 API：客户端 socket 关闭经 scope 级联、或 30s 准备期限到期出队（两者均 e2e 覆盖）；"取消排队即移除"以间接形态保证（lifecycle §3.2 如实记录）。 | http_file_service.rs 排队臂 |
| P729-D5 | low | 权限拒绝 wire 级 | PermissionDenied→403 映射已对齐计划 §5.3（R1 G-10 修正）；Windows ACL 类权限场景的 wire 测试环境不稳（只读文件对读打开不生效等），未设 wire 断言——映射由单元分类与 SD-01 行承载。 | http_file_service.rs OpenError::Escape(permission) |


### P733（2026-10-02，VM fn 调用语义计划复审登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P733-R1 | ~~medium~~ → 已清偿 | 测试基础设施 | **plan502_m3_layout_geometry_e2e 并行负载 flake（双树同发预存）**——全量并行档下 `handler_FlowDiagram_Init` 被 CPU-slice Parked(CpuRunnable)（PLAN-711 片预算负载下耗尽→测试驱动轮次内未续完→几何断言落 x=108 旧值）；单跑恒绿（本树 4/4+master 3/3）、全量约 2/4~2/5 闪红（worktree 与 master 3053f1fdf 同发同错，非 PLAN-733 回归——其 diff 零触 diagram/layout/parking 面）。**PLAN-759 T-08（2026-10-11，d5a507db3）已清偿**：run_dir 单轮 drive 改带 10s 截止时间的完成态循环（Init demand/CPU 续体/I-O parked 收敛即出），几何断言原值不放宽；plan502 族 8/8 绿+m3 scoped 两轮绿 | crates/auto-lang/src/tests/plan502_diagram_tests.rs:284（td st/ck 中轴断言）；复审记录 F-1（master 全量对照 5 采样 2 红） |
| P733-R2 | ~~low~~ → 已清偿（③随 PLAN-759） | VM 视图求值器 | **musk_vm_track p053_1/p053_4/p053_6/p054 预存红族=fn 调用语义同族深层腿**（master 3053f1fdf 基线即红，PLAN-733 三腿修复前后零行为变化/同错）——computed 链式 helper 断言面（musk chats 消息列表 computed 形态）残腿指向子件 override 态 `.store.X` 解析链深层；PLAN-733 边界登记于 design/vm-fn-call-semantics.md 已知边界③。**PLAN-749（2026-10-10）甄别：与条件通道异根**；**PLAN-748 T-02 首腿（2026-10-10，2e2164b5e 待合）分解定音**：六红三根——①p053_1×2+p053_6=`.store.X` 泛名展平空快照拒展平（三套 .store 口径不一致）**已翻绿**（musk_vm_track 97/6→100/3）；②p054×2 图标子件渲染面**归 748 T-02 续腿已修**（2026-10-10，3ec6a4914：PLAN-625 T-09(b) `tag.contains("icon")` 粗闸误判 PascalCase lucide 名→占位卡；判别器改 lucide 名集成员资格 `is_icon_component_tag`，AppViewport ∉ 名集保持占位卡意图；musk_vm_track 102/103）；③p053_4 merged `#[api]` CALL_NAT 发射**异根**——**PLAN-759 T-03（2026-10-11，753733205）定谳并清偿**：PLAN-633 守卫语义（实现体在场=直调不发桩）为有意契约而非债务，旧断言陈旧；双臂重锁（直调落账+缺实现 3142→RuntimeError）绿 | crates/auto-lang/src/tests/musk_vm_track_tests.rs:563/628/675；docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md 已知边界节；PLAN-749 §9 T-00 定音报告 |

### P735 批量回归登记（2026-10-03，merge 后到期档 tf/tt/tb@68a6ce919）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P735-R1 | medium | http 上传服务（PLAN-730 域） | **plan730_commit_target_matrix 确定性红（scoped 同红 3/3：tf 全档+tf fail-fast 首跑+单测 scoped）**——30.1s 恒定时长=内部 lease/deadline watchdog 到期形态；本批窗口（6988b1d7f..HEAD）代码面仅 PLAN-735 帧通道四文件（ui::frame_bench/frame_segments/iced::frame_probe+renderer 观测订阅——与 http_upload_service 零交集，无因果路径），同晨批量回执（covered 6988b1d7f 含 730 全码）报零新增确定性红→**环境/时序敏感测试（730 自己的 slow-upload/lease watchdog 族先例 38s 跨期限）在当前机时下翻红**，非 735 回归。清偿归属=PLAN-730 域（其 734 会话在途）：deadline 断言对机时负载的鲁棒化或超时预算放大 | http_upload_service.rs plan730_commit_target_matrix（30s 恒时长）；本仓 .tmp-regress.log 三腿实录（收据 JSON new_reds） |

## PLAN-737 执行登记（2026-10-03，work @lang-737，跨仓缺陷单 jade PLAN-037 T-02）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P737-D1 | medium | MCP HTTP current-thread runtime 结构弱点 | MCP 服务器=`new_current_thread()` 单 tokio 线程；737 已把工具执行下沉 `spawn_blocking`（handler 同步阻塞曾冻结 accept/IO 线程=探针连发 ECONNRESET 放大器），但 `note_activity` 等仍在 runtime 线程持锁；残余偶发连接复位（jade 侧旧 binary 亦两见=非 737 引入；737 修复后全量探针 4 跑 1 见）。疑点=多 app 顺序 runs 的 keep-alive/端口生命周期；需服务器侧 stderr 捕获 + AUTO_SCHED_DIAG 定向排查（复位时 runtime 线程是否存活）。 | mcp_server.rs run()/mcp_http_handler；p737 回执 §5 |
| P737-D2 | low | autodown 编辑壳键域双形态面 | storage key 归一形（`__autodown_editor_*`）与裸键在 MCP 载荷/装配注册两侧并存，靠 `normalize_payload_key` 幂等归一桥接（737 探针 MISS 实录后补）。后续新增 keyed 面应统一走该归一口，避免再分叉。 | autodown_editor/core.rs normalize_payload_key；renderer.rs ade_link_dispatch_message |

## PLAN-739 执行登记（2026-10-03，work @lang-739，跨仓缺陷单 jade PLAN-037 F-037-R1）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P739-F1 | medium | auto-cache 测试目标仓内腐坏 | `cargo check -p auto-cache --tests` E0063：methods_pack.rs:697 测试构造 `ShimMethod` 缺 `trait_name` 字段（PLAN-596 T-03 给 shim-metadata/types.rs 加字段未同步该测试）。**日常档门禁只建 `-p auto-lang` 目标（cargo t 别名 --lib + 三 integration test），auto-cache 自身 lib 测试从不参与编译，腐坏不可见**——master@9fcbdb5de 实证同破，非 739 引入。清偿=补 `trait_name: None`（或 Default 展开）+ 评估是否需要把 workspace 兄弟 crate 的 lib-test 编译纳入某档抽样（防再腐坏）。 | crates/auto-cache/src/methods_pack.rs:697；crates/shim-metadata/src/types.rs:109 |

## PLAN-740 执行登记（2026-10-08，work @lang-740）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P740-D1 | medium | VM app 启动健壮性 | `.am/state.at`（app 侧持久化旁路文件，gitignored）被杀进程写半截时残留 NUL 填充 → 下次 boot lexer panic（parser.rs:404 `lexer should produce first token`）**进程起不来**（2026-10-08 实录，删文件即恢复）。容错面（解析失败→忽略重建）属 app 健壮性域，非 740 范围 | 041 例 `.am/state.at` 23×NUL 实录 + handoff-v3 §3；复现=ladder/复判跑中途 kill 后再 boot |
| P740-D2 | low | 执行环境 | 本机 2026-10-08 实测 app 进程偶发静默退出（无 panic 痕迹，rc=0/1 混合，改前改后二进制均现，手工短复现不复现；同机他方会话重负载嫌疑）——谱采集协议已按「失败档重跑」消化；复判件需同款重试纪律 | handoff-v3 §3；740 执行实录 3/12 跑触发 |

## PLAN-736 复审登记（2026-10-08，review pass @6156c3de3）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P736-R1 | low | auto-man 服务进程托管 | `serve_rust` 父进程不 health 轮询子进程、不消费 ready 身份体做核对（代码注释自标"T-05 接管"未落地；T-05 执行记录"父进程核对同源"为过述）。AC-02 安全结论由 734 strict 门 + 占口 bind 失败非零 + 503-not-ready 探针承载，不受影响；父侧身份体核对列为可选增强（防子进程打印与监听者错位类的纵深防御） | crates/auto-man/src/http_service.rs:245-257；复审记录 R1 |
| P736-R2 | low | http_transport stderr 日志 | 736 新增 eprintln 臂（413/超时等，http_transport.rs:416-432/476/506/556-559）打原始 method+path（可含 query 串）到 stderr——在 JSONL sink 不脱敏合同之外，但与 plan736 §5.4 精神不一致；后续统一脱敏或改道 sink | crates/auto-lang/src/vm/ffi/http_transport.rs 上述行 |
| P736-R3 | low | 生成轨观测/响应头一致性 | 生成轨无显式 "startup" JSONL 事件（仅 state:ready，VM 轨有显式 startup）；生成轨 429 用 lowercase `retry-after`（合法但两轨风格不一）；main.rs:2842 重复 `#[cfg(test)]` 属性（无害） | api_gen.rs SERVICE_RUN_LOOP 模板；crates/auto/src/main.rs:2842 |
| P736-R4 | medium | auto-man 测试目标腐坏（预存，非 736 引入） | `cargo test -p auto-man` 全 targets 编译失败：bin auto-man E0601（空 main）+ tests/plan734_commands_fixture.rs E0255/E0425/E0433（未生成 api 夹具）——master bef73f52f **逐项同破**；日常档门禁不建这些目标故不可见（同 P739-F1 模式）。清偿=退役/修复 bin main + 夹具改可编译桩或移出默认 targets | crates/auto-man/tests/plan734_commands_fixture.rs；master 对照同错（复审实测） |
| P736-R5 | low | 执行环境 | 698 域 SSE 订阅会话首播后 ~1-2s 自然收口（帧竞速观察项，连续流不受影响）；VM resources RSS 单调缓升（59.6MiB，远低于 512 阈值，6/10 样本）——批量回归档复核 | reports/736-http-load.md 观察节；736-http-proxy.md 支持面边界③ |

## PLAN-738 执行登记（2026-10-08，work @lang-738，T-04 裁决遗留 + T-05/T-06 实勘）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P738-D1 | medium | 生产 native 注册面 ID 相撞真冲突（T-04 遗留 b 分诊裁决） | 扫描面动态 id 相撞 13 组（id1607 char×conv、id9930-9933 http×transfer 等）冻结基线（t04 `id_alias_conflict_detection`）。T-06 分诊裁决：`auto stdlib inspect --check` 按模块前缀把冲突组归入受影响模块**如实上报非零**（check_http_reports_frozen_id_conflicts 冻结），不静默、不弱化；重编号=ABI 变更，归 D3b/后续计划 | validate.rs id_alias_conflict_groups；crates/auto/src/cmd_stdlib.rs check_core；plan738 27/27 基线 |
| P738-D2 | medium | 核心模块 id 面分裂（T-04 遗留 c 同族扩展） | json 实勘（T-06 探针）：生产绑定面=catalog canonical 名（`auto.json.get`@1906，register_std_shims 实绑，生产 json 可用）；校验器问的公共扫描名面（`auto.json.json_get`@99xx）无 shim 可调——诚实判 Unverified（check_json_scan_face_unverified_nonzero 冻结）。与 http 扫描前缀名（裁决①）、io 第四绑定面（VmModule 方法表，遗留 a）同族：**公共符号 canonical 面全量重写**为统一清偿路径（下轮） | vm/native_catalog.rs 19xx 段；validate.rs canonical_native_name；cmd_stdlib.rs 实勘注记 |

## PLAN-738 归档后补充复查 R11（2026-10-10）

原计划归档终态保持。下项为实际反例确证的后续必修候选，须另走修复流程；登记不代表批准延期。R9 两项修复已确证闭合，不撤销其闭合记录。

2026-10-10 闭合更新：P738-R11-01 已由 PLAN-751 `226ebfbbb` 修复，归档后独立补充复验原 R11 探针 1/1（1.55s），空文件一次绑定/再生成身份均 `811c9dc5` 且 fresh，读错误与恢复矩阵通过；正式生成服务完整链 1/1（361.01s，测试夹具仅适配缓存路径）。原反例登记保留历史；后续 HTTP 能力缺口见 P751-D1 / [751-review-r2.md](reports/751-review-r2.md)。

| ID | 级别 | 问题 | 证据与修复要求 |
|---|---|---|---|
| P738-R11-01 | medium / P2 | 空 Cargo.lock 生产/消费身份不一致 | 落地 `9ff34ae0d`，api_gen.rs:1076 成功读取空字节写 `811c9dc5`，rust_ui.rs:4030 归 absent；实际生成成功后门判 stale，absent 首次空文件出现也未绑定。仅 NotFound=absent，成功读取统一实际内容身份；补正式生成/消费空文件、漂移/删除及读错误恢复矩阵。AC-06/07，SD-05；[R11](reports/738-review-r11.md)，探针 0/1（1.40s） |

验证尾项：R9 HTTP 全档 45/102、57 项未跑，移交 regress 尚无覆盖此次落地的收据；完成完整串行 th 及逐名分诊后补收据，不外推全档通过。

## PLAN-738 补充复审 R9（2026-10-10，needs_fix @19973b089）——**已全部闭合 @41b4d4be5（R10 pass 核证）**

（历史登记保留）本节曾是本期未满足验收的**必修**；主链 re-merge 回归已修复后以下两项阻塞收口，现均已闭合：R9-01=gate/生成两端 NotFound-only absent + 其它读错误 fail-closed；R9-02=三文件四格式点收敛。证据：[738-review-r9.md](reports/738-review-r9.md) 头部闭合注记 + [738-review-r10.md](reports/738-review-r10.md)。证据与修复合同见 [738-review-r9.md](reports/738-review-r9.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R9-01 | medium | lock 读取失败 | 未绑定收据 absent + Cargo.lock 读取 PermissionDenied 被实际门误判 fresh；生成/复用两端只允许 NotFound=absent，其他错误传播/判陈旧，补 unbound/bound 读取失败反例，保留一次绑定语义 | rust_ui.rs:4025 / api_gen.rs:1071；AC-06/07/08，T-12/13/14 |
| P738-R9-02 | medium | 格式健康门 | 最终 14 文件默认与 skip_children 检查均 exit1；只处理三文件剩余布局并提交，绑定实际通过的新提交收据 | lib.rs:119 / renderer.rs:24761 / engine.rs:9840、9852；AC-08，T-14 |

## PLAN-738 补充复审 R5（2026-10-10，needs_fix @9c255993b）

本节为 revision 3 **未满足原验收的阻塞项**，不构成批准延期或 D3b 豁免。R4 pass 保留历史，本轮由实际反例回退 executing，详见 [738-review-r5.md](reports/738-review-r5.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R5-01 | high | 生成依赖失效 | 收据 lock=absent 无结束条件：首次 lock 物化和随后依赖版本变更均被实际新鲜度门接受。首次成功构建后绑定真实 lock/features，后续严格比较；缺失/读取失败闭合拒绝，补完整状态序列反例 | rust_ui.rs:4022-4042；api_gen.rs:1071-1089；R5 正式生成/门反例；AC-06/07，T-12 |
| P738-R5-02 | high | 六核心 receiver strict | 公开 JsonValue.len 的返回类型 int→str 漂移后，实际 CLI actual/check 仍 exit 0，proof 只有 parse。collector 不能对局部 receiver 直接跳过；补实际方法来源/producer/adapter 证明或拒绝，不能以 P738-D2 转 D3b | trans/rust.rs:5561-5586；json.at JsonValue.len；R5 CLI 反例；AC-03/07，T-11 |
| P738-R5-03 | medium | 真实消费者验收 | 三角测试把 CLI manifest clone 当生成消费者并丢弃 Rust 转译 Result，未覆盖普通/persistent VM 与 C。保留局部身份测试，另补真实 CLI/会话/产物/正式生成收据/ready 的共同身份与依赖变更对拍 | plan738_stdlib_assembly_tests.rs:1640-1702；AC-02/06/07，T-12 |

R6 复核（2026-10-10，@`68398d2d6`）：P738-R5-01/R5-03 **闭合确证**（一次绑定状态机+真实三目标三角/生成腿，[738-review-r6.md](reports/738-review-r6.md)）；P738-R5-02 **部分闭合**——len 形态闭合，但 receiver 公共方法面其余 11 方法（keys/is_null/as_int 等，含 is_null 可编译旁路）漂移仍 exit 0 无 proof，续期为 **P738-R6-01**（同锚点，修复要求见 R6 报告 §2.3）。

## PLAN-738 独立复审 R1（2026-10-09，needs_fix @e4425b673）

本节为本期**未满足验收的阻塞项**，不是批准延期。P738-D1/D2 的六核心部分由 R1/R2 收回，不能只登记为 D3b 后勾全。原计划已回 executing；详细修复要求、逐 AC、复现源码及门禁见 [独立复审报告](reports/738-independent-review.md)。

| id | 级别 | 领域 | 必要修复 | 锚点 |
|---|---|---|---|---|
| P738-R1 | high | 生产 strict/冲突 | 校验器未进入 compile/codegen/persistent/trans；双活 body resolve_uses 返回 Ok。生产闭包拒缺失/冲突/未核验/引用 unsupported，注册前防 ID 错绑；收紧 duplicate 测试的成功放行臂 | compile.rs:1245；plan738 测试:1466；review R1 |
| P738-R2 | high | 公共身份/独立签名 | tcp_listener_close 参数1改999仍 Supported+SignatureChecked。补六核心公共→真实callee映射及独立 producer 参数/返回/self/async/feature/stub 元数据；io第四面/json canonical 不得推到全库重写 | validate.rs:135/174；model.rs:128；review R2 |
| P738-R3 | high | epoch/来源缓存 | 同会话改层仍 old_layer；CWD B 同名模块仍用A缓存。先解析来源再查cache，按epoch/来源身份及递归依赖闭包失效 | compile.rs:1245/1284；module_cache.rs:348；review probe |
| P738-R4 | high | 装配指纹/ready | vm层改名rs层指纹不变；缺provider/host内容与环境features；receipt null+null 当fresh。绑定实际闭包与provider实现身份，依赖不可核验必须拒绝 | loader.rs:49；api_gen.rs:1039；rust_ui.rs:4015 |
| P738-R5 | high | 生产目标装配 | Rust/C实际trans无manifest，无共同AssemblyPlan；persistent仍单独根与后缀；Rust net发射不存在模块也Ok。真实消费者接线，共用来源与不可变manifest | lib.rs:5943/6020；autovm_persistent.rs:258；review R5 |
| P738-R6 | high | CLI严格语义 | rust net --check假pass；async公共parse失败假pass；jsonx actual检查无关六核心72违规。修target/env/实际闭包/parse/module归一及便携源ID | cmd_stdlib.rs:212/314；review CLI反例 |
| P738-R7 | medium | 全分母/环境矩阵 | 字段与读失败层缺条目、WalkDir错误被吞；VM矩阵用目标helper分母，Rust/C无两环境格，native绑定不能证明browser adapter | loader.rs:82/225；validate.rs:330；review R7 |
| P738-R8 | medium | 源错误定位 | 只有加目标文件名的Msg，没有source span/行列/双源位置；失败路径再次parse会写共享TypeStore | compile.rs:1650；review SOURCE_ERROR |
| P738-R9 | high / medium | 真入口证据/健康/SD | C支持样例实编与generated API→service→ready未执行，不能用C HTTP缺provider或普通Json.parse witness替代；最终SD未核验。新增model.rs rustfmt三处diff必修 | verification报告；model.rs:106/196/203；review R9 |

门禁完整重跑：t 14红均在已有批回执 known_reds；tt新增观测 back_provision红/dep_fields超时 scoped复跑均绿，归因未充分确认，暂不列本期确定性修复；th两back_proxy红见执行报告基线。本轮未新跑baseline worktree同命令，不过述“零新增红已双树证明”。

### R1 修复进展（2026-10-09，work needs_replan @4fbcfe968→4089b8418）

基础修复与剩余阻塞逐项对账见 [738-repair-round1.md](reports/738-repair-round1.md)；R1..R9 继续作为本期必要验收，**尚未整体清偿**。历史 P738-D1 的动态扫描 ID 冲突当前不再复现（分配避开 catalog 保留 ID，独立别名冲突检测为 0）；不同 callee 争同 ID 的实际生产门仍须补齐。P738-D2 的 json canonical/io VmModule 查询已接线，独立完整签名与最终活 callee snapshot 仍未闭合，不再采用“下轮公共面重写”作为当前延期许可。

本轮特定反例已修：双实现宽松放行、arity 数量自证、同会话源变更、clone 跨 root、深依赖缓存、层改名指纹不变、null/null fresh、Rust net CLI 假绿、actual jsonx 无关六核心假红、读失败丢层和二次 parse 猜源。生产 strict 门/真实 emitter/完整公共矩阵/C 与 generated-service 见证仍未完成。已证伪原 init 三件套等同最终 callee 的设计假设；TCP read 公开 buffer/单返回与 VM size/双输出的 adapter 合同待有界修订，不能伪造 SignatureChecked。计划保持 executing；具体次序和依据见修复报告，不降低验收。

另保留 HTTP 观察：full th 100/102；back_proxy routes 同 R1 断言，plan707 timed frames 在本轮两次定向复跑仍红（撤回新增 callee 排序/首个 producer 选择之后也红）。原 R1 log 该例通过；既有时序族 flake 记录不足以证明本次原因，未做新基线对照，不能宣称零新增红。最终 scoped 54/54 与三 crate check 通过；详细版本绑定见修复收据。

## P747-D1（2026-10-09，PLAN-747 复审登记）：无审计行的静默崩溃族未归因

WER 链中 09-12/09-22/09-29 的 AppCrash（c0000409/FastFail 0x7）当刻 `exit-audit.log`
无对应 panic 行——非 panic 路径，疑栈溢出（fastfail 且无消息）。PLAN-747 已消除
「caught panic → 打印失败 → panic-in-panic → abort」放大链（focus_pending 风暴根修 +
容错打印），但该静默族若无 panic 前导则不受此修覆盖。观察条件：再次出现 auto.exe
fastfail 崩溃且审计无行时，优先按栈溢出排查（重建符号 + RUST_MIN_STACK/主线程栈深，
嫌疑面：大树递归 Drop / vtree 递归转换）。证据锚：`AppCrash_auto.exe_*` 09-12/09-22×2/
09-29×2 五份 ReportArchive；PLAN-747 §需求分析的时间线对照。

## PLAN-748 执行登记（2026-10-10，work @lang-748）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P748-D1 | low | 测试基础设施 | **plan051_p2 模块级图标测试空洞绿**——`locate_corpus` 三候选路径（CARGO_MANIFEST_DIR 相对 / CWD 相对 / ../../ 相对）在 nextest 进程 CWD（crate 根）下全 miss `test/ui/plan051_p2_modules/pac.at`（语料实存仓根 test/ui/），`build_component_from_app` 返回 None → SKIPPED 分支静默 pass——该测试自落地起未真正跑过图标臂（PLAN-748 T-02 续腿定音插桩实证：全程零触 imported 分派臂）。清偿方向：locate_corpus 补仓根锚定（env!("CARGO_MANIFEST_DIR") 上溯两级）或断言 corpus 必达（miss 即 fail），防 SKIPPED 假绿 | crates/auto-lang/src/tests/plan051_p2_tests.rs:22（locate_corpus）/ :60（SKIPPED 分支） |

## PLAN-751 执行登记（2026-10-10，work @lang-751）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P751-D1 | high | HTTP e2e 挂死族 | **plan730 上传 e2e 整族（14 项）+ e2e_a_redirect_302 确定性挂死**——服务端 accept 后不响应（60s×2 read timeout 累计 120s 被 nextest TERMINATING）；隔离单跑确定性复现（vm_plain_endpoint_untouched / e2e_a_redirect_302_with_location 均 TIMEOUT 120.0s）；与负载无关（同条件 plan326 兄弟 39 项 0.6s 全绿）。红移窗口=PLAN-738 系列（plan734 时代 th 99/101 绿 `ae9b7a7e2` → 738 后转红；http_server.rs 最后触碰=R2 `bade1845c`，其后 `19973b089` 触及 http 族契约注册撤销语义；R2 收据自述 th 绿与本轮矛盾）。**根因 bisect + 修复走后续专项计划**；候选锚点 `dd9ffe35b`/`19973b089` | docs/plans/reports/751-th-full-receipt.md；crates/auto-lang/src/tests/plan730_http_upload_tests.rs:254（start_server）/ crates/auto-lang/src/vm/ffi/http_server.rs |
| P751-D2 | medium | auto-man（现 auto-ui-tools）日常档盲区 | **当前（2026-10-22 T09 R2独立复审）：shell_pack分项已CLOSED @0f2858e35；原生清单check/locked均0、freshness1/1，详[终审](attachments/815/review-t09-r2/review.md)。plan593 CSS值金样/merged_api仍为独立开放分项，plan593不是plan734排版族；下述三红与shell根因为历史记录。** **三红预存（T-04 迁移后随 rust_ui/vue 落 auto-ui-tools，2026-10-22 T-07 子批2 复核仍 3 红、stash 基线实证）**：`vue::plan593_index_css_golden_tests::index_css_values_match_p1_baseline`（CSS 金样漂移）、`rust_ui::tests::test_shell_pack_lib_freshness`、`rust_ui::tests::merged_api_client_crud_fallback_for_uncovered_endpoints`。**shell_pack freshness 根因已定位（PLAN-815 T-07 子批2）**；**T09 R1裁定该分项为PLAN-815合同内阻断P2 F-815-T09-01（实际生成消费107个E0433，非仅金样；详末尾记录），不得按预存债豁免；2026-10-22 T-09 R1修复轮已修复关闭@0f2858e35——壳层五处+清单模板canonical化+sanctioned regen，freshness红→绿、原生清单消费exit0，详[fixes](attachments/815/t09-fix/fixes.md)，余项=plan593/merged_api两分项**：`generate_shell_pack_lib_sources` 壳层仍发 legacy `auto_lang::ui` 前缀（rust_ui.rs 3771 一带），而入库 `crates/shell-pack/src/lib.rs` 已被 814 T-05 canonical 化（0 处 legacy）——814-B"模板发射方随策略同步"未完成的同族残项（wrap_example 臂已随 T-07 子批2 修复，shell-pack 再生器臂因 worktree 内 auto-os 镜像解析落主检出、无法在唯一树闭环验证而暂缓）；修复形态=壳层头/再导出 canonical 化后重跑 regen_shell_pack+提交。另有 `auto-api-gen api_gen::tests::test_plan734_tauri_fixture_matches_generator` 基线预存红（TauriGenerator 空白漂移 vs fixture `echo,]`，T-07 子批2 stash 实证非其引入）归 plan734/api-gen 家族。根因：日常档 `cargo t` 只 `-p auto-lang`，工具 crate 面长期脱离例行门禁——清偿方向：修复各红 + 将 scoped 面纳入复审触面清单 | crates/auto-ui-tools/src/rust_ui.rs（freshness/merged_api）、crates/auto-ui-tools/src/vue.rs（plan593 金样）、crates/auto-api-gen/src/api_gen.rs:7209、docs/plans/attachments/815/t07-acceptance/batch2.md §5 |

### PLAN-751 R2 补充分诊更新（2026-10-10）

- **P751-R2-01（high/P1，P751-D1 的实证入口）**：落地树 `d79282eb6` 的原上传与重定向程序，通过实际 `crate::run` 在启动前即返回 `STDASSEMBLY.SIGNATURE_UNVERIFIED`，分别缺 `http.upload_receive #9937` 与 `http.response_redirect #3108` 的独立 producer/adapter 契约（探针 0/1、0.10s；shim 实际存在）。后续需审计最终公共 HTTP provider/ABI 并补真实合同，保留严格门，再验 wire/完整 th；不是 751 lock 修改引入。
- **P751-R2-02（medium/P2，纠正 D1 归因边界）**：D1 上述「accept 后不响应、60s×2」只能保留为执行者假设，尚非确证机制；重定向实际配置 5s read timeout；原两 start_server 均丢弃 run Err。历史表为 plan730 13 timeout + interop fail，另 redirect timeout，不能把 interop 也计为 120s 挂死。需有界保留启动 Err/panic/readiness，再接通原 120 秒等待位置。last-touch 非根因证据；代码引入校验锚点 `d82eb02eb` 是新候选，尚未 good/bad 对拍，不签署精确归因。
- 原 102 项计数保留历史；本轮没有再跑完整 th，不外推全绿。详情与新证据：[751-review-r2.md](reports/751-review-r2.md)。738/751 归档终态保持，修复走后续专用 worktree。
- 原未修改的 `http_e2e_plan730_vm_plain_endpoint_untouched` 本轮独立复跑仍 timeout（120.046s，exit 100）；其长等待与有界诊断快返契约错误的差异尚未接通，不能直接等同。

### PLAN-755 闭合更新（2026-10-10，work @lang-755；**同日 merge 销案定稿**）

- **P751-D1 已销案（delivered @`e586b9113`，R1/R2/R3 三轮复审后合并）**：PLAN-755 已按专项修复并全档复验——①根因实证：契约错误（`SIGNATURE_UNVERIFIED` #9937/#3108，`crate::run` 32ms 快返）被夹具 `let _ =` 吞掉 + 本机拒绝连接 ~2.15s/次 × ready-poll 50 次 + 客户端 60 次重试爬行 = nextest 120s TERMINATING；"accept 后不响应/60s×2 read timeout"旧机制证伪（read 相位从未到达），见 [755-startup-diagnosis.md](reports/755-startup-diagnosis.md)。②修复：HTTP 公共面 41 项缺契约补齐 + 5 项 inventory 契约漂移覆盖 + 走查/merge 存活常驻守卫（[755-contract-inventory.md](reports/755-contract-inventory.md)）；夹具启动诊断有界化。③复验：串行全档 th **102 项 101 过/1 红（仅 back_proxy 在册基线红）/0 超时，186.4s**——14 timeout 清零、interop 转绿、86 绿零回归（[755-th-full-receipt.md](reports/755-th-full-receipt.md)）。bisect 候选锚点（dd9ffe35b/19973b089）随根因实证销案；规范增量 SD-01（公共面契约覆盖完整性）落 assembly-manifest.md。
- P751-D2（auto-man 盲区）不在 PLAN-755 范围，保持开放。

### PLAN-755 独立复审 R2（2026-10-10，needs_fix）

| ID | 级别 | 影响验收 | 证据与修复要求 |
|---|---|---|---|
| P755-R2-01 | medium/P2 | AC-01、T-02 | 三启动助手的 `Err(panic)` 直接 Debug 格式化 boxed Any，父线程消息只得 `Any { .. }`。实际 plan730 助手注入标记 panic，负向验收 0/1（2.153s），标记只在默认子线程 hook，未转发至启动失败载荷；三处提取 &str/String payload 并冻结真实助手反例。 |
| P755-R2-02 | medium/P2 | AC-06、T-06 | clean `4ce645516` 的四改动 Rust 文件 rustfmt check exit 1；此前“fmt 干净”与提交工件不符。格式修复提交后重新绑定门禁证据。 |
| P755-R2-03 | medium/P2，簿记已纠正 | AC-07、T-07 | SD-01 modify 已存在的 assembly-manifest.md，不能因新增节登记为新组件；本复审已修正 spec-impact 分类，规范语义不变，修后重审定稿。 |

契约守卫及上传/重定向原反例独立 2/2 + 2/2 通过，主要 HTTP 修复事实保留；整项退回 executing，P751-D1 保留闭合候选，尚不销案。详细证据：[755-review-r2.md](reports/755-review-r2.md)。

2026-10-11 落地后独立闭合更新（覆盖上述 R2 的历史待修路由）：P755-R2-01/02 已由 `5672e8e2d`（原 `e121debf8`，range-diff 全等）修复，三助手共享字符串 panic 转发核心；本会话在 clean `cee541da7` 隔离树复跑守卫 4/4、四文件格式 exit 0、完整串行 HTTP 102/101/1 在册 back_proxy 红/0 超时（合并守卫共 106 run，265.241s）。P755-R2-03 元数据纠正保留，755 archived 不回退。blueprints 68 文件恢复后逐文件 Git 哈希与 HEAD 一致；恢复前额外日常红不同于原 R2 实现缺陷。P751-D1 已销案结论得到串行复验支持；批量回归仍 DUE（旧收据覆盖 740），另按 regress 补执行。证据：[755-landed-independent-check.md](reports/755-landed-independent-check.md)。

### P753（2026-10-10，README 计划知识投影前置阻塞）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
**P753-D1 注记（2026-10-11）**：753/756 的投影阻塞已经 README §5 手工回退路径清偿（P753-1/2/3、P756-1/2/3 入账，两计划归档/清理完成，收据见各计划 recovery receipt）；债务剩余部分收窄为——store 本体无法加载现有 `.autoos/specs.json`（缺 document metadata，GET /api/specs 500）且无注册写入器可达（758 r2 同期收据复证 blocked by policy）。后续计划投影可继续走回退路径，store 修复为独立事项，本条保留至 store 恢复或裁定退役。

| P753-D1 | medium | runtime Specs store 兼容性 | 既有 `.autoos/specs.json` 无法由当前 AutoMusk store 加载；任务自有 loopback 服务 `GET /api/specs` 返回 500，已停止。原账本 hash 保持不变；不通过手改 JSON 或删除重建修复。README/语言文档可独立复验，但 store-mediated 投影及随后的计划归档/清理必须等待受支持的恢复路径。未将此项视为已完成。 | [验证报告](reports/753-readme-verification.md)；PLAN-753 §9；auto-musk backend/crates/musk/src/specs.rs |

### PLAN-754（2026-10-11，reviewed；低危外观瑕疵）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P754-R1-01 | low | playground-vue 深链 | 无效 `?note=<id>`（manifest 中不存在）时参数残留在 URL——`deeplinkTried` 仅在命中或无参时置位，`history.replaceState` 只在命中时执行。欢迎态不崩已由 e2e ③ 覆盖；后续顺手修：未命中分支同样置位+清参。 | PLAN-754 §9 复审记录；packages/auto-playground-vue/src/components/PlaygroundIDEApp.vue |

### P758 补充（2026-10-11）

- **P758-D1 / publication blocked**：AutoJade 当前规范与网站可合入；derived ledger 的 `P732-1` / `P722-1` 名称和九份规范投影（含 r3 宣传数字/桌面平台口径）须通过 store writer 更新。当前未注册写入工具，检查的既有服务端口无监听；临时 AutoMusk 服务启动/只读探测被自动审批以 `blocked by policy` 拒绝，未给进一步原因。未手改账本。保留 reviewed Plan 与工作树，待 writer 可用后只续做投影/归档/清理；证据与描述见 [p758 复核](../reports/p758-autojade-verification.md)、[待投影描述](../reports/p758-autojade/ledger-projection-pending.json)。延续 P753-D1 的收尾限制，不抹去旧账本内容。
- **P758-OBS1 / asset provenance**：用户给的 P4 tags 文件实际是其他面板，未冒充标签图；SHOT-09 恢复仍依合同留空。六张合格深色原图有完整字节/hash，缺深色专属工具链 manifest，因此只标用户提供准备态。补拍要求见 [截图指南](../reports/website-v05-capture-guide.md)。
- **P758-OBS2 / baseline gate**：注释改名无非注释差异；最后 Rust 日常档 19 红，其中 18 项在原注释基面复现，余一动画项在基线及最终隔离复验均通过；tu 的既有金样红亦在基线复现。不虚报全绿或纳入本次无关修复，逐项日志见 p758 复核附件。

- **P758-OBS3 / concurrent baseline**：同步 PLAN-754 后，静态预览 /playground 主动探测同源 /api/examples 得到 404，fallback 127.0.0.1:3030 未运行产生连接拒绝；原控制台检查隔离复现。两个宿主组件与当前 master 等同，本计划未改变它们或放宽检查。证据：[基面核对](../reports/p758-autojade/concurrent-baseline-check.json)。最终当前网站 61/63，搜索隔离复验通过，余一既有此项；最终 t 5213/5231、18 红（16 已对照、2 flake 隔离绿），tu 857/2 红（原金样 + env 路径用例隔离绿），完整记录在 p758 复核。

### PLAN-757（2026-10-11，r3 用户裁定留空合入；产品缺陷待修）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P757-D1 | high（用户指示需修改） | AutoEdit VM/iced 比较视图布局 | 双文件比较视图正文上方大片空白编辑区、差异挤在下半部（并排与内联均观察到；`h-0 w-0` 隐藏 code_editor 为排查线索，未确证根因）；真实画面不适合宣传（问题图 `docs/reports/p757-capture/media/p757-autoedit-diff-layout.png`，+16/-1 一个 hunk 被压缩）。功能本身已核查可用（保存后自动重比 +17/-1 两个 hunk 在案）。**SHOT-03 宣传图因此留空**，布局修复后按拍摄合同重拍（指南行"待布局修复后重拍"）。 | PLAN-757 §9/§10；[诊断原图](../reports/media/p757-autoedit-diff-layout.png)；[捕获目录](../reports/p757-autoedit-capture-catalog.json)；auto-edit specs/auto-edit/src/front/app.at 比较隐藏编辑器 |
| P757-D2 | high | VM 查找输入与焦点路由 | 原生键入（聚焦查找输入后输入 ws_root）同时落入查找框与活动内置 untitled main.at 正文（样例缓冲区未保存，真实源码未受写入）；合格搜索图改用应用 MCP FindInput handler 规避。需专门复现焦点/键盘路由，不能把 MCP 成功当原生链无缺陷。 | [捕获目录](../reports/p757-autoedit-capture-catalog.json)；vm2.stderr.log；PLAN-757 r2 findings |
| P757-D3 | low | VM 搜索响应 | 跨文件搜索扫描完整工程时同步阻塞原生窗口，完成后真实返回 ws_root=22、fn ws_root=2；未计精确时延不外推通用性能，拍摄须等待完成 | [搜索状态](../reports/p757-autoedit-search-state.txt)；AutoEdit fsys.at search_files_json |
| P757-D4 | low | website 首次搜索索引加载 | VitePress local search 首次加载索引长尾，并行负载下 e2e 偶发超时（新端口单 worker 复跑 19.6s 过） | 同上 |

### PLAN-759 清偿登记（2026-10-11，日常档 12 红族收口）

清偿成员（证据链见 docs/plans/759-daily-red-family-clearance.md 执行步骤）：
musk p053_4（P733-R2③）/ p748_t00 双臂（生而红 harness 边界）/ plan606 test_029（10.2s→0.18s）/
projector_counter（双形态重锁）/ desktop_bus_inbox + desktop_surface_merge（壁纸机器态隔离）/
schema 三围栏（REG-4 四表清偿）/ ash_stream_leak_probe（门禁分层，集成真跑绿）/ plan502_m3
（P733-R1 完成态驱动）。

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P759-D1 | low | iced 渲染配置 | **硬编码机器壁纸目录探测**——`wallpapers_dir_or_default`（renderer.rs）解析链末端探测 `D:\Down\stella-os\wallpapers`（PLAN-526 T19 机器便利默认），dev 便利进库代码路径：①机器相关行为（测试须显式隔离，规约已入 ui/overview.md PLAN-759 注记）；②他人机器不生效。清偿=挪 env/配置显式开启；涉及产品行为须用户裁定 | crates/auto-lang/src/ui/iced/renderer.rs `wallpapers_dir_or_default`（~:17019）；docs/specs/auto-lang/ui/overview.md PLAN-759 注记 |
| P759-D2 | low | schema 漂移围栏 | **schema_drift 的 rs 扫描器仅认单行 `elements.insert("`**——schema.rs 现存 209 个多行 insert 形态不可见（仅 15 单行被扫到），四表对账长期靠千行级 baseline 白名单吸收。PLAN-759 刷新 baseline 时实证（+249/-301）。清偿=扫描器多行化+随后全维度 baseline 重审裁剪（牵动数百条目，独立计划） | crates/auto-lang/tests/schema_drift.rs:62 `scan_insert_tags`；crates/auto-lang/tests/fixtures/schema_drift_baseline.txt（1075+ 行） |

### PLAN-759 落地后独立核查候选（2026-10-11）

以下候选登记于[落地后审核](reports/759-post-landing-review-20261011.md)（原判定 needs_fix）；D3/D4 已由 `fd55d2d42`（fix-759-review-p2 轻量 worktree 路径）清偿，并于2026-10-11完成独立复审（`35be7f1e0`，两项修正 pass，日常5236/5236、tv162/162、裸libtest三线程4轮均绿；tu唯一余红为既有desktop金样，完整证据见同报告末节），不改变759归档终态。

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| ~~P759-D3 / F-759-R1~~ 已清偿 `fd55d2d42` | ~~medium~~ | 生产渲染诊断 | **已清偿**：render_support 详情查询折叠 schema canonical（kebab/canonical 三围一致）；command/carousel 族按 view_builder 实际转换臂逐子件校准（command_input=partial 与 input 同口径、command_item=partial 仅 value 忽略、command_empty=fallback 恒渲染 Empty、command_group/shortcut=full、carousel_item/previous/next=full 等）；schema/aura.at 12 行同步；别名一致/ignored 不过报/实际 View 对照三回归测试入 render_support::tests | ui/render_support.rs（get_support+详情表+tests）；schema/aura.at backends.iced；报告F-759-R1 |
| ~~P759-D4 / F-759-R2~~ 已清偿 `fd55d2d42` | ~~medium~~ | 测试环境隔离 | **已清偿**：stdlib 新增 `HTTP_BASE_TEST_SERIALIZER`+`lock_http_base_for_test`（镜像 lock_storage_for_test 先例）与 `HttpBaseEnvGuard`（RAII panic-safe 恢复，含未设态）；plan606 photo 测试与 stdlib http_base_url_resolution 均改锁+守卫形态；panic 恢复回归测试入仓。全部原有断言（精确 URL/24 张恰一次/Cover/Contain/stub 收据）保留 | vm/ffi/stdlib.rs（锁+守卫+两测试）；tests/plan606_gallery_tests.rs；报告F-759-R2 |

### PLAN-761 根目录整理的范围外候选（2026-10-11）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P761-D1 | medium | CLI test filter | `auto test --filter` 在 `test_file` 执行全部测试之后才过滤显示；a2ts runner 内部已有 FAILED 仍可显示 0 passed/0 failed 并返回0，VM runner即使使用不匹配过滤器也继续执行到超时。原有实现，本次仅迁移runner；不能把过滤后的退出0当套件验证通过。应将过滤条件传入执行层并保留实际失败退出码 | crates/auto/src/main.rs Test 分支（test_file→遍历reports→filter顺序）；reports/761-root-directory-review.md |
| P761-D2 | low | 历史 Plan446 repro | 原字节保留的 b3check 语料在当前引擎报 `Init` 同时匹配 Theme/Modules（multi-store disambiguation）；新路径确已加载且没有SKIP，显式ignored旧复现仍不兼容当前严格规则。保留原始历史导出，不在目录美化中改其业务语义；后续若恢复常规使用，另做版本化修复夹具 | test/fixtures/plan446/b3check/auto；tests/plan446_j1_repro_tests.rs；reports/761-root-directory-review.md |


### PLAN-761 收尾批量回归新观察（2026-10-11）

- P761-R1（未销案）：tt的plan437 gallery_chart_components_render_geometry缺/bar-chart h6且Init仍CpuRunnable；同提交tf/tb通过。当前归因未确定，独立分诊[PLAN-762](762-chart-init-batch-flake.md)，不得仅凭偶发绿降为环境豁免；详见[批量报告](reports/761-batch-regression.md)。原目录整理已交付，未改变上述实现。


### PLAN-763 迁移基线对照登记（2026-10-11，taa 全档 5 红分诊收口）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P763-R1 | low | aavm compile 腿 + a2r goldens（master 预存） | **3 枚 AAVM 红为迁移前基线预存债，非 PLAN-763 目录归并致因**——基线 worktree（716ca62db=分支点）同滤串同红同错误形态实证：①`test_aavm2_compile_corpus`/`test_aavm2_compile_use_corpus` 双 E0433（生成 main.rs 引 `auto_lang::a2r_std::fs::read_text_range` 而临时工程 `[dependencies]` 空、harness 无内联 shim；血统=db4e5954b plan687 增补，2026-09-22）；②`test_aavm2_goldens_check` 参考执行 `STDASSEMBLY.INVALID_BYTECODE: 60`（字节码语义红，与路径无关）。清偿归属 aavm/a2r 域另档：compile 腿需给临时 Cargo.toml 接 auto-lang path 依赖或改发射面；goldens 需定位字节码 60 案。与 aavm 无实用面裁定一致（守护=CI+ta，本债不阻塞合入） | docs/reports/763-baseline-taa.{json,log}；763-final-taa.log；db4e5954b |
| P763-R2 | low | aavm 自举入口（master 预存） | **`auto <repo>/aavm.at` 单文件直跑 auto_link_E0401 `Undefined symbol: P.next in module typeinfo`**——master 主检出（迁移前源码+master 二进制）同命令逐字节同错实证=预存；与七删 CI 流最近五次全红的审计结论同向。迁移侧有效冒烟=命名空间 scratch（smoke-final aavm-namespace exit 0）；自举形态冒烟仍应走 aavm_build_smoke.sh（合并装载）而非单文件直跑 | docs/reports/763-smoke-recheck.json；763-ci-audit.md |

### PLAN-765 复审登记（2026-10-12）

| 编号 | 优先级 | 域 | 描述 | 锚点 |
|---|---|---|---|---|
| P765-D1 | low | VM 宿主内存 | idle 相位 ~1MB/min 线性缓漏未归因（1h 实测 +59.3MB/59.7min,AC-1 门 0.1MB/s 的 6× 余量内,线性不平台）。候选=每响应仍余 3 份全文瞬态(body/转义缓冲/VM Value+池)×2 拍/s 的堆记账漂移、消息批全窗重绘路径。后续杠杆:载荷池化复用/`unconditional-rendering` feature 退役评估(PLAN-708 基线启用项)。判定阈值:长寿实例(≥12h)私有内存超 ~1.2GB 再升级优先级 | docs/reports/765-t01-attribution-memos.md 后记⑥;soak tmp/soak-p765-1h.json(musk worktree,临时) |
| P765-D2 | info | 跨仓仪器 | ~~musk soak 仪器 slopeMBPerMin 单位 bug~~ **已修(musk plan-104-dev 9794aa1/0e06161,2026-10-12 核验)**;AC-01 已按真值重判 pass(idle 0.717MB/min 对门 8.4× 余量) | docs/reports/765-t01-attribution-memos.md 后记⑤;musk 104 evidence §T-03 |
| P765-D3 | info | 跨仓仪器 | bigpage-matrix.mjs 收尾击杀假象(try/finally 无 catch:click isError 抛错→finally taskkill→"VM EXITED code=1" 先于错误栈打印,被误读为进程自退;musk T-03 因此曾 blocked on 765-R2)。**765-R2 已定谳为假象非产品缺陷**(纪要后记⑦);修复归 musk 侧(click 补 value+收尾击杀标记,两行修,交接文案在案) | docs/reports/765-t01-attribution-memos.md 后记⑦ |

## v0.6-dev debt and review records

The following unique review and debt sections are carried forward from v0.6-dev. Their historical attachment paths and source commits remain unchanged; Plan 767 is now PLAN-772 in the current ledger.

## PLAN-772 独立复审（2026-10-12，revision 2，修复重审 pass）

以下为合同内阻塞缺陷与修复候选，必须在本计划修复后重新复审；登记不代表允许挂债收口或合入。受审提交 `033642a27d7404dee5797b195a9b7c680fe6ece5`，完整反例与任务路由见 [独立复审报告](attachments/767/review-codex-r2.md)。

**销号收据**：三项均已在 `4cc4645021f029459a3b870c34a23eb16bc4ce59` 修复。原 needs_fix 的独立复审者重放冻结 16 项、新增 48 项不变量检查及 scoped 门禁后确认 closed，无遗留阻塞。保留历史缺陷描述，当前结论见 [独立修复重审](attachments/767/review-codex-repair-4cc464502.md)。Plan 为 reviewed（8/9），T-09 尚未执行。

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P772-R1 | P1 / closed | HIR 身份 | Descriptor::domain 将未知三重身份降为 Core，程序化 bind/verify 与 writer 可接受未知 schema/revision/profile；三维分别复现，共六反例。精确注册身份随行并受控拒绝，不以 CLI registry 正确为豁免。 | experimental/ac-core/src/descriptor.rs:245、2111（受审树）；PLAN-772 T-02/06/07/08，AC-01/05；owner=767 work |
| P772-R2 | P1 / closed | HIR 类型可信门 | 签名 TypeUse 查表失败默认 I32，params/result TypeId(9999) 分别获 Checked 后 native 越界 panic；补所有签名引用验证和永久正反控制。旧默认逻辑不是本合同豁免理由。 | experimental/ac-core/src/verify.rs:226、255；native.rs:91（受审树）；PLAN-772 T-02/06/07/08，AC-04/05/06；owner=767 work |
| P772-R3 | P2 / closed | 设计收据 | design.md 的减法溢出公式写为 MIN-b/MAX-b，实际后端正确为 MIN+b/MAX+b；修收据并重绑证据，不照错误文档改正确实现。 | docs/reports/767-scalar-i32/design.md:112（受审树）；PLAN-772 T-01/07/08，AC-07；owner=767 work |
## PLAN-745 new 取号核查（2026-10-06）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P745-D1 | high | 中央取号 | scripts/new-plan.sh 的 `ls active-glob archive-glob \| grep -q .` 在 `set -o pipefail` 下，某目录无匹配导致 ls 非零，即使另一目录已有同号也漏检；本次 .next-id=744 与既有 744-vm-sqlite-module.md 冲突时误建了同号骨架。已撤回本次未跟踪骨架，再由中央脚本取得745，未覆盖旧744，计数现746。脚本还缺跨写者排他锁；修复应在独立worktree建立统一排他取号与active/archive重复控制，不在主检出顺手改代码。 | scripts/new-plan.sh:33；docs/plans/745-ac-source-checked-hir-native.md §4.3；owner=后续取号机制修复工作，未编号 |
## PLAN-741 复审批（2026-10-04，plan-741-dev @ 4dc4da636 同基线对照裁定——零新增债务，仅归档对照证据）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P741-R1 | info | 日常档红对照 | **PLAN-741 复审 cargo t 26 红裁定为非本回归**——worktree 基线 951b6c70f 上 `cargo t --no-fail-fast` = 4945 绿/26 红/1533 skip；同基线 scoped 对照（主检出 0d7c7f024，crates 与基线一致；PLAN-741 零 crates diff）：14 例同红（musk p053×3/p054×2、plan707 sse、plan502_m3、plan498_area、plan484_024、ffi_dual_017、dep_parity_017、projector_counter、plan606 gallery_029——**全部落入 THR-D3 既有清偿族**）+ 12 例 scoped 串行绿（并行负载 flake，与 THR-D4 同模式）+ 4 例 filter 未命中（schema_drift/docs_gen fence、ash_leak_probe 环境族）。无新增挂账项；本行为跨仓跟进类红的又一同基线实证样点 | PLAN-741 §9 复审记录；docs/reports/741-ac-hir-native/verification.md §4.1（入库后可解析）；THR-D3/THR-D4 |
## 执行吞吐治理第二批挂账（2026-10-02，PLAN-726 立项分批裁定：第一批四项见 726，余量在此）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| THR-D1 | medium | 构建面 | **crate 拆分前置测量**——单 crate 76.6 万行/1,650 文件（2026-10-02 实测），Rust crate 粒度增量下任何改动触发整 crate 重编+测试二进制重链；先测量冷 worktree check/链接时间占比（样点：sccache 79% 命中下 `cargo check -p auto-lang` 57s、lang-724 target 17G、non-cacheable 801 次），数据支撑再决定 workspace member 拆分立项——不测不做 | .cargo/config.toml jobs=12；sccache --show-stats；2026-10-02 会话实测样点 |
| THR-D2 | medium | 构建面 | **共享 target/链接缓存评估**——REG-3 已实证共享 target 候选撞名红（auto-edit.exe 撞名翻转策展测试），评估带产物命名空间隔离的跨 worktree 共享方案；未评估不实施 | 本文件 REG-3；clean-autolang-targets.cmd |
| THR-D3 | medium | 验证信号 | **master 14 红族清偿**（musk p053×4/p054×2/plan606/projector/plan707/e4→PLAN-726 承接/app_registry×2/desktop_protocol/iced renderer/a2vue 金样/plan484 streaming）——跨仓跨域，逐族立项；目标=验证信号零噪音、复审零定案税 | 711 §10② 红册；PLAN-715 批收据基线 |
| THR-D4 | low | flake 族 | **ffi_dual_019 / plan502_m3 并行负载序敏感 flake**——批量档观察名单积累复现率，频率上升再立项查并发交互 | 本文件 REG-2；PLAN-715 登记 |
| THR-D5 | low | release 档 | **release 档红册治理**（~254 环境红+tests/ 编译 rot 残余）——非维护门但偶发消费（P712-D4 陈旧产物观察） | P702-D2；release_full3.log 分诊 |
| THR-D6 | low | 走查基建 | **实机走查脚本沉淀**——主检出 `.tmp-vm-*` 一次性脚本族（20+ 件未跟踪）收编入 `.agents/skills/autoui-verifier/scripts/` 标准件（AGENTS.md 既有要求），消除每次走查重写成本 | 主检出 .tmp-vm-*；autoui-verifier 技能 |
| THR-D7 | low | 术语残件 | **725 计划落定后其文档内残存 10 处「谳」→「案」清理**（2026-10-02 出清批遗留——当时该文件由并行起草会话持有未动） | docs/plans/725-typed-frame-incremental-pipeline.md |

### P723（2026-10-01，28应用图文介绍素材核查）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P722（2026-10-01，网站 Apps 介绍复审）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P716（2026-10-01，供③ 确认复核件降级挂账——plan_revision 2 用户裁定）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P706 r2 收口批（2026-09-30，work 阶段定案的非本回归缺陷另立记账）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P707-R1~~ | high | async http 直驱臂（e4） | **已清偿（2026-10-02，PLAN-724 T-02）**——真因勘定=**测试侧违反 managed 提交协议**（非派发缺陷、非 detached 桥分离）：e4 三臂直接 `spawn_async_http_handle`/`spawn_async_http` 提交 managed job（999_001/999_101/999_102）却未先 `register_live_op`，707 `submit_client_job` 的 cancel-before-install 闭合（still-live 复查）在 spawn→登记窗口内发现令牌缺席 → 立即 abort 刚提交的任务 → 请求恒不到线 → 30s `recv_timeout` 恒红。生产 shim 全部先 register（stdlib.rs 5367/5390/5413/5434 等在案）。修复=测试改走生产协议：register→submit→wire 断言→消费配对（plain 臂增验 managed 完成落表 `Structured{status:200}`）+`cancel_live_op` 幂等清理，并钉住未登记反例（abort 句柄 submit 返回前回收 + 无令牌永无可消费结果）。取消守卫零改动；plan712（2/2）与 plan707_cancel（5/5）复证绿；修复后单跑 0.46s。公开入口覆盖保持：直驱两点即公开 native shim 的唯一 send 汇聚处，默认头/query/body wire 断言原样 | stdlib.rs e4 模块（default_headers 三臂+未登记反例臂）；`cargo t default_headers_reach_wire_on_plain_get` 绿日志（work 阶段在案） |


### P708（2026-09-29，实施方案审查：待核实风险）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|

### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
### P693（2026-09-23，Plan 693 native-exe-tri-mode work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P693-D2~~ | medium | desktop 接入跨仓配对 | **已销号（PLAN-698 T-04，2026-09-23）**——形态勘定=子进程（库形态被 winit Windows 单事件循环结构性否决，RecreationAttempt 实锤）；`spawn_desktop_daemon` 孵化 `auto rqhost --pipe autodesk-rqhost-desktop-<pid>` + AUTO_DESKTOP_ENDPOINT env 注册表发布（命名裁定定稿）；003 converter.exe 全环录证（adopt→开窗→v2 帧→EOF 回收→末窗自退，reports/p698-batch/T04-desktop-pairing.md）。SD-04 落 ui/overview.md；WM 深度集成仍归 auto-os 侧后续 | renderer.rs run_session Desktop 臂；rqhost.rs spawn_desktop_daemon；desktop.ps1 |

### PLAN-698（2026-09-23，Plan 698 desktop-substrate-hardening-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
## PLAN-741 交付后质量复审（2026-10-04）

复审基线c1ac219e7，结论needs_fix。~~均未清偿~~ **已全部清偿（2026-10-05，PLAN-741 r2 Phase 2）**：
用户裁定不另开合同，激活741以plan_revision 2追加Phase 2修复；独立代理复审pass
（44/44测试、六反例全翻转），交付合入v0.6-dev@52c67d3df。清偿证据：
[741归档](741-ac-hir-native-core.md) §9 Phase 2记录；
原始报告：[复审报告](../reports/741-quality-review-20261004/REVIEW.md)。

| id | 级别 | 问题与实际证据 | 修复方向 |
|---|---|---|---|
| P741-QA-01 | P1 | bool icmp结果i8与local/ABI i32冲突；check绿而build panic101或后端拒绝 | 统一bool表示并补绑定/参数/返回原生测试 |；**已清偿**（r2 T-09 icmp_bool 唯一出口uextend归一+bool原生正例（bool-if exit3/bool-abi exit5））
| P741-QA-02 | P1 | 按eval_args索引校验而非bindings映射；异类型合法调用被拒、非法调用获Checked | 按映射检查类型，保留求值顺序 |；**已清偿**（r2 T-10 bindings映射制类型核对+双射校验（binding-swap原生exit9/swap类型不配check拒绝））
| P741-QA-03 | P1 | 两函数可共享异签名body且check绿；native参数索引panic101 | function/body双向owner及唯一归属检查 |；**已清偿**（r2 T-11 function↔body双向归属（shared-body-owner/same-sig/stolen全verify.owner-mismatch拒绝））
| P741-QA-04 | P1 | receipt写失败exit1，exe已更新、obj仍旧 | 制品准备/发布事务与失败回滚，补发布失败测试 |；**已清偿**（r2 T-12 暂存-备份-发布-回滚事务（发布失败exe/obj哈希不变+旧exe实跑；回滚测试还实抓并修复同stem备份名碰撞））
| P741-QA-05 | P2 | README多传ac-probe且示例无d_entry；真实exit2/entry.not-found，归档链接陈旧 | 统一命令/命名、可运行fixture和链接，复制运行验证 |；**已清偿**（r2 T-13 README真实bin名/带入口fixture/归档链接/schema笔误（三命令逐字复制运行全过））
| P741-QA-06 | P2 | rustc/reg发现直接output无deadline；执行器先等退出再排pipe | 有截止的统一执行器与并发输出读取 |；**已清偿**（r2 T-14 全子进程硬截止+并发管道读（1s截止杀挂起helper、>126KB输出不阻塞单测））
| P741-QA-07 | P3 | tests/common/mod.rs descriptor未使用warning | 清理并覆盖all-targets健康检查 |；**已清偿**（r2 T-15 未用导入清除+verify脚本all-targets零warning门）
## PLAN-741 r2 合入后复核（2026-10-05）

基线965b368a2，needs_fix；旧四项P1常规反例已确认修复，44/44原型测试通过。
~~以下为新增/扩展边界，未清偿~~ **已全部清偿（2026-10-05，PLAN-741 r3 Phase 3）**：
独立代理复审pass（AC-16..20全过、原型52项+60s证据测试、watchdog 60.05s准时拒绝、
cargo t双侧同基线归因零新增），交付合入v0.6-dev@a0b3750c4。清偿证据：
[741计划](741-ac-hir-native-core.md) §9 Phase 3记录；
原报告：[r2复核报告](../reports/741-quality-review-20261005/REVIEW.md)。

| id | 级别 | 观察 | 修复方向 |
|---|---|---|---|
| P741-R2-QA-01 | P1 | 入口块自环check栈溢出0xC00000FD；断开双块循环check0 | 先做完整块图无环/可达性验证，失败后阻止递归走查 |；**已清偿**（r3 T-19 块包含图结构门（显式栈DFS全可达+入边不变量、结构失败阻止走查；复审代理自构3块环/else回边反例全拒、两报告输入定位拒绝））
| P741-R2-QA-02 | P2 | 子进程退出后reader join不受截止约束；持pipe后代反例114.963s仍未返回 | 进程与输出收集共享deadline，补继承pipe反例 |；**已清偿**（r3 T-20 单一单调截止覆盖等待+输出收集（Job Object KILL_ON_JOB_CLOSE 管控子树、reader全出口回收；watchdog实测60.05s准时拒绝 vs r2实测114.96s挂起；read_to_end+lossy修截断成功））
| P741-R2-QA-03 | P2 | 收据暂存写失败遗留已链接.tmp exe；旧exe/obj保持不变 | stage所有权/统一清理覆盖准备/备份/发布失败 |；**已清偿**（r3 T-21 publish暂存所有权全失败出口回收（含已链接exe）+link.restore恢复诊断+占位目录保护；复审helper实测staged_exe_left=false）
| P741-R2-QA-04 | P3 | archive内frontmatter仍reviewed，与merge receipt的archived不一致 | 修正簿记，保留历史与revision |；**已清偿**（r3 T-22 状态/指针簿记（复审方激活+本轮核对一致；最终归档由本merge完成））

2026-10-05 用户明确再激活：[PLAN-741 Phase 3](741-ac-hir-native-core.md)，revision 3、executing。
上述 P741-R2-QA-01..04 由 T-19..T-24 / AC-16..20 跟踪；本次仅合同修订，
未经修复及独立复审不销账；旧 P741-QA-01..07 已修证据与历史记录保留。
## PLAN-743 合入后独立复核（2026-10-05）

基线 fa3abe476，needs_fix；用户授权激活 [PLAN-743 Phase 2](743-acc-bootstrap-hir-contract.md)，
r2/executing。18/18 既有测试通过、旧 F-01/F-03 已修，以下新问题待 T-08..14 / AC-09..14 闭环。
证据：[独立报告](../reports/743-quality-review-20261005/REVIEW.md)，本次不预销账。

| id | 级别 | 观察 | 修复方向 |
|---|---|---|---|
| P743-QA-01 | P2 | 跨行字符串里的 pretend() 被扫为调用；MD-506 认可换行恢复 | 字面量整体屏蔽/行号回归，复核观察和结论 |
| P743-QA-02 | P2 | Meter.len/new、真实 CG.new/Ar.new 被标 native-runtime | 接收者/owner 证据优先，未知保守分类 |
| P743-QA-03 | P2 | 无绑定/缺证据/空决定被 CHECK-OK | provided schema/证据绑定校验+完成态严格门 |
| P743-QA-04 | P2 | 必经路径常量溢出允许变编译期错误，observable/运行 trap 改变 | 保留 runtime trap，澄清编译期语境和 effect 术语 |
| P743-QA-05 | P2 | A2 源码误称 existing，候选①要 A8、候选②缺 A9 前置 | 分 HIR/源码状态，语料与无环能力前置对齐 |
| P743-QA-06 | P2（新鲜度） | 当前8决定/9绑定 stale，漂移拒绝正确 | 逐条语义重核后校准，不 blind hash update/exempt |
| P743-QA-07 | P3 | T-05/T-06 未勾与6/6/已修收据矛盾 | 历史勾选校正，新 Phase 待办/最终归档一致 |
## PLAN-743 r2 合入后复审 / Phase 3（2026-10-05）

基线 5983f8aeedeae2dc5769f9a0820eec4b722d4c6d，needs_fix；[报告](../reports/743-r2-quality-review-20261005/REVIEW.md)。
上节 r1 反例修复已核验（41/41、真实47决定新鲜）；该历史待办描述由本节取代，剩余保证由以下新 ID 跟踪，不把新增缺口抹成已闭环。
用户授权激活 [743 Phase 3](743-acc-bootstrap-hir-contract.md)，r3/executing，T-15..21/AC-15..20；仅合同，本轮不销账。

| ID | 级别 | 观察 | 修复 |
|---|---|---|---|
| P743-R2-QA-01 | P2 | 本地同名 IO/print 仍 native，跨 owner/无 owner 方法升级为 self | 当前 owner/冲突证据优先，unknown 保守 |
| P743-R2-QA-02 | P2 | evidence 未在同条绑定，改变内容严格门仍绿（当前真实决定无此缺口） | 同条证据 hash 覆盖和变化拒绝 |
| P743-R2-QA-03 | P2 | resolved family 删除 decision 仍闭环绿 | 适用决定引用与 open 去向真实校验 |
| P743-R2-QA-04 | P2 | kind/conclusion/bound/evidence 错误类型抛 traceback | 字段/元素类型先验、定位受控拒绝 |
| P743-R2-QA-05 | P2 | canonical R3 旧摘要与新正文冲突，报告 CLI 仍 ac-probe | 新 SD-07 同步摘要/正文，SD-06 工具政策，真实命令对账 |
| P743-R2-QA-06 | P3 | archived 13/14/T-14未勾、模块executing/r2、归档两链接失效/ledger旧指针 | 历史据实勾选，新Phase独立待办，merge核对终态/索引/链接 |
## PLAN-743 r3合入后复审 / Phase4（2026-10-05）

基线8e8f6f19b96516fdbc9d4c893297fa1757a7aad4，needs_fix；[新报告](../reports/743-r3-quality-review-20261005/REVIEW.md)。
上一轮未绑定证据、本地遮蔽/跨owner与canonical摘要/链接等反例已修；63/63和当前真实47决定绿，剩余保证由以下新ID跟踪。
用户授权激活[743 Phase4](743-acc-bootstrap-hir-contract.md)，r4/executing，T-22..27/AC-21..24；本轮仅合同，不修工具，不预销账。

| ID | 级别 | 观察 | 修复 |
|---|---|---|---|
| P743-R3-QA-01 | P2 | 被类型检查淘汰的决定仍入引用索引，evidence=17在族consumer处TypeError | 只消费验证记录/提前受控拒绝，引用组合负例 |
| P743-R3-QA-02 | P2 | subject/hash覆盖可代替evidence覆盖，严格门假绿（真实七族证据完整） | evidence相关性独立且必需，移除OR旁路 |
| P743-R3-QA-03 | P2 | 导入IO/List/File/process和Meter参数IO被标native | qualified宿主准入包含导入/局部绑定冲突，未知保守 |
| P743-R3-QA-04 | P3 | archived20/21、T-21未标完成、P743-3活动指针失效 | 历史补据实完成，新Phase另列；final所有指针/计数统一验收 |
## PLAN-741 r3合入后复审 / Phase4（2026-10-05）

本节新增发现不撤销前两轮已修的具体证据，也不沿用r3“全出口已回收”覆盖新反例。
[报告](../reports/741-r3-quality-review-20261005/REVIEW.md)基线f6a945db8，needs_fix；
依用户授权激活[741 Phase4](archive/741-ac-hir-native-core.md)，r4/executing，T-25..30 / AC-21..24。
~~本轮只复审/合同，不实施、不预销账~~ **已全部清偿（2026-10-05，PLAN-741 r4 Phase 4）**：独立代理复审pass（AC-21..24全过、death-watch反例5/5零泄漏、locked-cleanup全要素、60s生产路径准时拒绝、worktree红集⊂基线零新增），交付合入v0.6-dev@8fdae8c11；canonical SD-07/08已沉淀、live ledger P741-6已投影。清偿证据：[741计划](741-ac-hir-native-core.md) §9 Phase 4记录；原报告：[r3复核报告](../reports/741-r3-quality-review-20261005/REVIEW.md)。

| ID | 级别 | 观察 | 修复/跟踪 |
|---|---|---|---|
| P741-R3-QA-01 | P2 | 公开run_exe约60.106s返回link.deadline后自有后代仍活；时序相关，另次正确回收。先spawn再约束、约束失败静默None/detach有缺口 | 受控启动及失败安全收口/reader回收，T-26，AC-17/21 |；**已清偿**（r4 T-26 受控启动（CREATE_SUSPENDED→入 Job→ToolHelp 恢复，约束前窗口消除）+约束失败受控拒绝（安全收口零泄漏）+reader 全出口回收；death-watch 反例 5/5 零泄漏、public-60s 60.05s 准时拒绝且后代回收）
| P741-R3-QA-02 | P2 | 真实共享锁使owned暂存exe不能remove；收据准备失败只报link.receipt，未列该残留/清理错误，旧三件套不变 | 清理错误/自有路径/准确事务状态汇总，T-27，AC-18/22 |；**已清偿**（r4 T-27 清理错误全量汇总（link.cleanup：原失败+提交状态+残留自有路径+实际 OS 错误；目录=用户占位不删不计）；复审 locked-cleanup 实测 staged_exe_left=true+全保留标志、库内共享锁测试锁定）
| P741-R3-QA-03 | P3 | archived23/24、T-08漏勾与r1/r3收据矛盾 | 根据收据补T-08，重开T-20..24验收，新六任务；最终完成ID计数/链接断言，T-28，AC-23 |；**已清偿**（r4 T-28/merge T-08 据收据补勾、重开 T-20..24 依 r4 任务关闭、最终计数 30/30 与归档断言一致（本轮 merge 收口））

root门禁观察（非新增AC原型缺陷）：daily4971全跑4935通过/35失败/1超时；
超时ffi_dual_018单独2.532s通过。旧THR-D3/D4及阶段报告包含对应族，但本轮未逐个完成35红对照，
不得写成required daily gate绿或“新基线零新增已证实”；正式失败清单已入报告，T-29负责修复后复验/归因。
## PLAN-741 r4合入后复审 / r5 Phase5（2026-10-05，未修复）

绑定HEAD7b9c6948c；[独立报告](../reports/741-r4-quality-review-20261005/REVIEW.md)、[同ID修复合同](741-ac-hir-native-core.md)。r1..r4实施/原债项销账为历史，不删除；本轮54测试/一键13步/公开60s回收通过。

- P741-R4-QA-01 / P2：link_object_staged非零及执行器错误出口吞暂存exe清理失败；真实共享锁OS error32、残留不入诊断，旧成功三件套不变。T32/AC25，SD09兑现SD08，不降为informational。
- P741-R4-QA-02 / P3：archive内4个历史报告链接用active相对路径而失效；T33/AC26，必须最终真实归档门。

尚未实施产品修复。r5 executing/22-of-35，只写证据/合同/导航；canonical/live ledger留pass后merge。P741-3..6旧archive指针激活后暂缺，最终必须全部恢复；不占新号、不碰其它计划工作树。
## PLAN-743 r4合入后复审 → r5 Phase5（复审2026-10-05；2026-10-06补齐落地，未修复）

绑定HEAD c865adf66a75099df07113b053eb9d19e92986a3；[独立报告](../reports/743-r4-quality-review-20261005/REVIEW.md)、[执行合同](743-acc-bootstrap-hir-contract.md)。

- P743-R4-QA-01 / P2：mut/方法及裸调用参数遮蔽漏判，误升级native；T30/AC26。
- P743-R4-QA-02 / P2：合法JSON manifest容器形状错误traceback；T31/AC27。
- P743-R4-QA-03 / P2：非法语义/stale/重复先插入记录仍进入消费者；CLI正确拒绝，无假绿；T29/AC25。
- P743-R4-QA-04 / P3：归档6个报告链接失效，r4断言未检查；T32..33/AC28。

仅复审/合同激活，产品未修复，不宣称销账；83测试、fresh47、旧27/27/ledger真实通过。SD09/canonical/derived ledger留r5独立pass后的merge；激活导致P743-3..6旧archive指针暂缺，最终归档必须全部恢复。

2026-10-06补齐激活文档，当前22/33；原83测试/47fresh为复审基线历史证据，主线另有5条AutoAC Spec过期绑定待T28/T30逐条重审。现有741 r5合同/债项原字节保留，未实施743修复。b'\r\n'  - **已清偿**（r5 T-32 两出口经共享 discard_own_staged + with_staged_cleanup 汇总（link.cleanup：原失败+残留完整路径+OS 错误+NOT committed+恢复办法）；复审 nonzero/timeout 实测 path/cleanup_in_diagnostic=true、os error 32、旧三件套不变、released_cleanup_ok=true）b'\r\n'  - **已清偿**（r5 T-33 计划 7 处链接改 ../../reports（归档终态可解析）+ final_assertions.py 三态门；复审逐条验证 5 目标自 archive 父目录可解析（覆盖原 4 断链超集）；本轮 merge 归档后以 archive 模式复跑 35/35 全勾门收口）
## PLAN-741 r5合入后复审 → r6 Phase6（2026-10-06，待修）

基线6baed9bba84016dd8610221a9a0358195444385e；needs_fix仅P3验收/证据，r5核心P2修复已通过真实当前库nonzero/60s共享锁及58测试/一键13；不重新挂旧核心缺陷。
- P741-R5-QA-01/P3：archive current34/total35但35唯一任务已完成，最终断言不检查metadata，错误/缺数字仍通过。T36/AC28。
- P741-R5-QA-02/P3：active只检查模拟archive，实际父目录7断链仍通过；当前真实archive链接已正确。T36/AC29。
- P741-R5-QA-03/P3：旧反例任意取第一个缓存rlib可选旧库；r5当前摘要54/lib14过期（实际58/lib15）。T37/AC30。明确当前Cargo artifact重放已证实产品修好，旧库结果不得冒充回归。
详见 ../reports/741-r5-quality-review-20261006/REVIEW.md。同741激活r6 current26/total38，未实施修复/未销账；仅最终断言/文档证据范围。canonical/live ledger保持原状态，P741-3..7暂态archive指针留最终归档恢复。
## PLAN-743 r5合入后独立复审 → r6 Phase6（2026-10-06，激活时未修复；现已清偿，见后续复审）

基线9111c21e53ccd9a81554bfd4764c540fdfdcad21；needs_fix，旧110测试/旧四类反例/47fresh/确定性/归档33门实际通过。
- P743-R5-QA-01/P2：额外绑定文件删除后全局stale但记录仍入可信索引；存在/删除/恢复三段式CLI0/1/0，中间族消费者仍21。T35/AC29。
- P743-R5-QA-02/P2：字段类型错误continue绕过去重，同ID坏记录前后两种顺序均保留合法同ID记录供族消费21；字段合法duplicate对照为0。T35/AC29。
- P743-R5-QA-03/P2：已观察普通参数与local type同名，bare仍type-construction、qualified仍type-qualified，违反SD09绑定在类型升级前优先。T36/AC30。
前两CLI受控非零/no traceback，不误报假CHECK-OK；当前真实人工层未因合成反例被宣称失效。全部复现/修复/SD10见../reports/743-r5-quality-review-20261006/REVIEW.md。同ID激活r6 current26/total38，未实施/未销账/未改canonical或live ledger；P743-3..7 archive暂态留最终merge恢复。741/742/ABI及所有历史pass/merge保留。b'\r\n'  - **已清偿**（r6 T-36 final_assertions v2（frontmatter 必需字段/数值解析、current_step==完成唯一数、total_steps==唯一总数合同派生无硬编码、archived=全完成+archive 位置、同 ID 冲突失败）；上线即抓到执行侧 current_step=35≠34 计数错；受控 fixture 8/8 真实退出码）b'\r\n'  - **已清偿**（r6 T-36 active 模式双链接验证（真实父目录 active 形 + 显式转换模拟归档副本，两行输出、任一失败即 exit 1，模拟不冒充实际 active）；复审确认 fixture 复刻缺陷形态正确非零；本轮归档后 archive 模式复跑收口）b'\r\n'  - **已清偿**（r6 T-37 采纳复审 exact-artifact 脚本（Cargo JSON 按 package/target/profile 唯一绑定库+CLI，无 rlib glob，Cargo 输出与哈希落盘）+ 全量重放；计数更正 54→58（lib15+43）带 provenance 入 r5 verification.md 与归档计划）
## PLAN-741 r6合入后复审 → r7 Phase7（2026-10-06，未修复）

基线d1adc03be39d2897f5dfd7090e0909095cf24465；[报告](../reports/741-r6-quality-review-20261006/REVIEW.md)、[同ID合同](741-ac-hir-native-core.md)。核心同源无新故障，17/17独立validator控制通过，不重新打开原型/Rust任务。

| ID | 级别 | 真实缺口 | 修复与验收 |
|---|---|---|---|
| P741-R6-QA-01 | P3 | fixture固定active文件，最终归档后FileNotFoundError；只修输入后T35冲突mutation为零次替换，反例exit0 | T39/AC31，active/archive来源、动态实际翻转、预期诊断与mutation核对 |
| P741-R6-QA-02 | P3 | 两模块741导航仍executing且指向缺失active文件；final_assertions只验Plan/ledger而遗漏README/导航，仍ALL-PASS | T40/41、AC32，最终门明确覆盖README/两模块/全部指针，实际归档后遗漏导航更新负控制非零 |

同741激活r7，current36/total41；未实施/未销账。只重开T36/T38，保留历史pass/merge；不改canonical行为/live ledger，P741-3..8暂态留最终归档恢复。
## PLAN-743 r6合入后独立复审清偿记录（2026-10-06）

reviewed_commit=8b17e4de8851cdf37ccdce76a82f78422cf8bb15，outcome=pass；[独立报告](../reports/743-r6-quality-review-20261006/REVIEW.md)、[归档计划](archive/743-acc-bootstrap-hir-contract.md)。原r5反例/激活时待修文字为历史，本段为三个743债项的当前终态；不修改741或其它计划债项。

| ID | 当前状态 | 独立验证 |
|---|---|---|
| P743-R5-QA-01 | 已清偿（r6 T35） | 绑定文件存在/删除/恢复CLI0/1/0，consumer21/0/21，缺失文件不再入可信消费者 |
| P743-R5-QA-02 | 已清偿（r6 T35） | 错类型/合法duplicate双顺序及缺字段双顺序整组consumer0；错误ID容器类型受控拒绝无traceback |
| P743-R5-QA-03 | 已清偿（r6 T36） | 原参数/local type同名bare/qualified保持unknown；7组普通/mut方法、let/var/use控制通过，无冲突正例维持 |

128/128、真实strict47/146/7、确定性/manual原字节/三方一致及actual archive38/38/all P743指针通过。保持743 archived/r6，不新增修复phase；审查treeguard clean后已清理，Category A未跑Cargo/native/docs_gen。；**已清偿**（r7 T-39 fixture_tests.py 生命周期重写（计划源自动检测 active/git 历史重放归档态；冲突变换按真实行状态翻转恰一行+断言实际修改；状态变换通配 set_fm；全部变异经 assert_modified）；两矩阵 21 用例 + 全 PASS（含 archive 态 git 历史重放与全完成合成控制））；**已清偿**（r7 T-40 final_assertions v3 扩到 prototype README 741 链接与 auto-hir/auto-ac 741 导航行（状态生命周期一致+行号定位+--repo-root 沙盒负控制）；本轮归档后 archive 模式复跑收口）
## PLAN-741 r7合入后复审 → r8 Phase8（2026-10-06，未修复）

基线18beff64b4ceefa1e7f4176296f5e37f77dd6da1；[报告](../reports/741-r7-quality-review-20261006/REVIEW.md)、[同ID合同](741-ac-hir-native-core.md)。实际归档41/41与导航正确，核心同源；原缺文件/零操作已修，不重新挂旧缺陷。

| ID | 级别 | 当前缺口 | 修复与验收 |
|---|---|---|---|
| P741-R7-QA-01 | P3 | 两个导航控制未实际运行，总数固定+2；归档7报9、双源17报21且记录称已验证 | T42/AC33，纳入实际控制列表、失败传播、按执行记录计数，旧日志保留注明过期 |
| P741-R7-QA-02 | P3 | 必需README Plan引用缺失、README/模块/P741 review指向其它存在的归档Plan均仍ALL-PASS | T43/AC34，必需引用存在及规范唯一741目标严格比较，active暂态限定规范archive/741 |

同741激活r8 current38/total44，只重开T39..41，新增T42..44；未实施/未销账，不改Rust/行为Spec/live ledger，P741-3..9归档暂态由最终merge恢复。742/743/ABI及历史收据保留。；**已清偿**（r8 T-42 两矩阵导航控制以 add_external 进入统一 cases 真实执行（各带沙盒 repo root）+ 总数从执行记录派生（固定 +2 删除）；控制性确认 forgotten-nav-update 期望改 0 → fixture 整体非零（失败传播）；--source auto 实测 25 用例全 PASS）；**已清偿**（r8 T-43 final_assertions canonical 目标身份检查（README 必须存在 741 计划引用；README/两模块导航/全部 P741-* review 指针解析后必须恰等于 docs/plans[/archive]/741-ac-hir-native-core.md，错目标失败含 expected/got 与 entry ID；激活期 canonical_archive 尚不存在=显式列暂态）；4 个 identity 沙盒负例 + 反向验证旧版门假通过全部实证）
## PLAN-745 r1 最终独立复审（2026-10-06，待修）

reviewed_commit=b3cd8171966281a42f96187780342ce0ff51c5fb；needs_fix；[最终报告](attachments/745/review-r1/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。同一实现树保留，执行中1/9；本轮仅复审/证据/簿记，没有修复、合入或规范沉淀。原92绿/74例/两脚本通过为真实局部证据，不能覆盖本轮反例。

| ID | 级别 | 合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R1-QA-01 | P2 | 非括号RHS/callee跨行仍check0 | T02/05、AC02；换行边界正反控制 |
| P745-R1-QA-02 | P2 | 用户ac_start/ExitProcess过Checked却native符号冲突 | T03/04/05、AC02/05/06；隔离native内部/用户名字空间，不静默保留源码名 |
| P745-R1-QA-03 | P2 | 作用域逃逸指return，双分支终结后语句指前面的if | T03/05、AC04；精准出错token/span，纠正N09宽oracle |
| P745-R1-QA-04 | P2 | Auto关键字当名字；!与转义str误分类 | T02/05、AC02/04；保留关键词和识别但排除的诊断 |
| P745-R1-QA-05 | P2 | source测试无截止output/先wait后read堵管道；合法20K trace控制93ms，错误顺序2s仍阻塞 | T05/07、AC05/06/08；有限截止、并发采集、后代/全部出口回收 |
| P745-R1-QA-06 | P2 | HIR trap结构断言、实际Bundle删let/错type/越域变异、source SENTINEL永久门缺失 | T03/05/07、AC01/03/05/06；真实变异/不变量oracle/失败传播；独立source三件保留已通过，不报告产品破坏 |
| P745-R1-QA-07 | P3 | inventory --output误作文件，误称manifest未跟踪而推merge重扫 | T06/07、AC07；正确strict47已过，纠正证据/策略，不自动重hash |
| P745-R1-QA-08 | P3 | source/HIR各复制Checked后构建管线，未兑现共用合同 | T04/05、AC06；同一构建入口，两模式回归 |

这些项未清偿；登记不等于批准延后。SD01/03及盘点策略待修后重新冻结，完整宽度/char保持open；canonical/live ledger保持当前状态。root门禁完整daily29红/tv4红另有原始清单，生产源/config零745 diff且均在历史红集并集；未称门禁绿色或严格同源逐项归因完成。
## PLAN-745 r1 第二轮最终独立复审（2026-10-06，待修）

reviewed_commit=91b37560cffda75cf07cef8620cf524f5d6a3c7c；[第二轮报告](attachments/745/review-r2/REVIEW.md)、[同ID计划](archive/745-ac-source-checked-hir-native.md)。needs_fix/executing/1-of-9；同树保留，未实施本轮修复、合入或规范沉淀。

R1-QA02（native符号）、QA04（关键词/排除）、QA07（盘点误归因）、QA08（共用构建）**已清偿**；QA01/03的具体原反例已修，但新增回归见下；QA05的大输出/普通父进程截止、QA06的trap结构/删let/错type/部分SENTINEL已修，剩余合同项按本轮ID追踪。旧历史段落保留，不撤销具体pass，也不以旧待修文字误称产品毫无修复。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P745-R2-QA-01 | P2 | 合法多行call/括号闭合后的同行+表达式误拒 | T02/05、AC02/05；括号上下文和真实语句换行边界成对控制，保留C01/C03拒绝 |
| P745-R2-QA-02 | P2 | Rust助手500ms截止实际5016ms仍成功；PS1s实际5044ms；无效UTF8输出被吞 | T05/07、AC05/06/08；执行/采集同deadline，受控子树及全出口回收，字节排空/真实错误传播，两助手均修 |
| P745-R2-QA-03 | P2 | 两Stray分支被提升有效Div，后续死块遮住break/continue目标错误 | T03/05、AC02/04；组合保留Stray诊断可达性，固定实际冒犯token |
| P745-R2-QA-04 | P2 | Bundle第三变异只验异type；publish控制先删旧receipt后仅比较两文件 | T03/05/07、AC03/06；同type越域真实变异、保留三件旧文件的实际publish失败/恢复控制；独立verify.uninitialized-read已证实，不报告产品verifier失效 |
| P745-R2-QA-05 | P2 | 4122字节/4096未闭合括号使编译器栈溢出exit -1073741571 | T02/03/05、AC02/04；深畸形输入受控diagnostic/exit1/无panic，浅合法控制；不偷缩profile或只扩大栈 |

本轮99测试/两脚本/strict47均通过，但不覆盖这些反例；root日常33红/tv4红实录保留、hover隔离1/1通过，未声称严格同源逐项零新增或根门禁绿色。五项未清偿，登记不等于批准延后；SD01/03完整叙述待修后重新冻结，canonical/live ledger保持当前状态。
## PLAN-745 r1 第三轮最终独立复审（2026-10-06，待修）

reviewed_commit=7437eb137a789259f01446da7a170e74031ff7f2；needs_fix；[第三轮报告](attachments/745/review-r3/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原树保留，executing 2/9；没有实施修复/合入/规范沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R3-QA-01 | P2 | Rust 505ms 超时返回仍留后代和读线程；PS5.1 415ms 静默窗假成功，后代/输出未收完 | T05/07、AC05/06/08；受控启动子树、实际EOF、统一截止、所有出口回收/错误传播，持管道/延迟写/超时/约束失败控制 |
| P745-R3-QA-02 | P2 | NEST_LIMIT 把合法224字节99层括号当syntax错误；32794字节合法加法链仍栈溢出 | T02/03/05/06/07、AC02/03/07/08；覆盖实际树及后续处理递归，合法与不闭合边界控制；不得静默缩小r1支持集 |

R2-QA01 换行、QA03 Stray 组合、QA04 Bundle同类型越域/三真文件publish oracle已清偿。R2-QA02 字节读取/有界返回局部修复保留，整体转R3-QA01；R2-QA05原不闭合E02已受控，新健壮性/边界问题转R3-QA02。R1其它清偿保持。

本轮104 passed/1 ignored、741 13步/745 12步74例、独立74/74与strict47通过；根日常27失败+1超时，tv4失败未称绿色，新增dep018超时隔离1/1通过。初始PS5.1哈希失败是复审.NET驱动继承PS7模块路径的环境注入，已通过单变量正反控制定位并按计划原命令重跑成功，不记为实施缺陷。
## PLAN-745 r1 第四轮最终独立复审（2026-10-07，待修）

reviewed_commit=dc5e95be757dba647ef758aa283e7e2788015747；needs_fix；[第四轮报告](attachments/745/review-r4/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。同树保留，executing 3/9；本轮没有实施修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R4-QA-01 | P2 | 新PS C#执行器约束失败只杀空Job，挂起子进程仍存活；真实ReadFile错误6被当EOF返回Code0/空输出 | T05/07、AC05/06/08；未入Job进程直接终止+wait，读取/等待等错误传播，有效及真实失败控制、无幸存/无截断成功 |
| P745-R4-QA-02 | P2 | NEST_LIMIT改语法计数仍未批准支持边界；200层完整合法括号426字节被syntax拒绝 | T02/05/06/07、AC02/07/08；保留r1支持或显式new修订边界/诊断/提案/正反验收，不以扩阈值替代合同义务 |

R3-QA01的Rust生命周期与PS静默窗口原反例已清偿；R3-QA02的合法16384链崩溃、99/100层括号误拒已清偿；未完成的边界子项和新PS错误出口按上表追踪。原C/D诊断、符号、换行、Stray和既有oracle没有退化。

本轮112 passed/1 ignored、741 13步/745 12步74例、独立74/74、strict47通过；新增native复杂求值24/baab、trap70空stderr、长链退出300通过。根日常35失败/tv4失败未称绿色；唯一新红名plan643隔离1/1通过。真实Win32失败控制保留原cleanup/result路径，不用全绿banner替代失败传播检查。
## PLAN-745 r1 第五轮最终独立复审（2026-10-07，待修）

reviewed_commit=15fd9263f6f4c400a6ba1d8ec5cb2cae44354d41；needs_fix；[第五轮报告](attachments/745/review-r5/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，executing3/9；没有实施修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R5-QA-01 | P2 | 连续括号塌缩未覆盖1+(1+(…))；4096层完整16410字节与未闭合12314字节均stack overflow，5s仍未返回 | T02/05/07，AC02/04/08；实际非连续及组合路径受控，合法source→Checked通过、畸形诊断无panic，保留优先级/效果/span |
| P745-R5-QA-02 | P2 | 调用/块护栏1000仍未批准入r1/SD；函数体占一层，999可native/1000完整语法已syntax拒绝 | T02/05/06/07，AC02/07/08；保留支持或显式new修订计数/各入口/资源诊断/边界及正反验收，不仅扩大阈值 |

R4-QA01未入Job挂起子泄漏、ReadFile错误6假成功与wrapper传播已清偿（实际native失败控制无幸存）；R4-QA02原200层连续括号误拒已清偿，连续100000层通过，剩余边界转R5-QA02。历轮其它清偿保持；新发现不覆盖掉已经通过的控制。

本轮113 passed/1 ignored、741与745均13步、独立74/74、strict47通过；native300、24/baab、trap70空stderr、混合括号111/5、999call/if通过。root日常33失败/tv4失败未称绿色；唯一新增plan707_cancel隔离1/1通过。完整反例/原始stderr/边界两侧与冻结delta在第五轮报告附件，登记不代表批准延后。
## PLAN-745 r1 第六轮最终独立复审（2026-10-07，待修）

reviewed_commit=522b98a4c27d2064049e188d386f6958721b3a90；needs_fix；[第六轮报告](attachments/745/review-r6/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，executing3/9；没有产品修复、合入或canonical/live ledger沉淀。

| ID | 级别 | 当前合同内缺口 | 修复/验收 |
|---|---|---|---|
| P745-R6-QA-01 | P2 | 迭代表达式解析器逗号分支未先归约Bin：f(1+2,3*4)、named/异构bool/嵌套/trace均syntax误拒；同源H09由native111退化为拒绝 | T02/05/07，AC02/05/08；按Call/Paren边界归约当前实参，保留mark/命名映射/求值顺序，补正反永久回归 |
| P745-R6-QA-02 | P2 | 终止分支仅归约前检查顶帧，Bin遮住未闭合Call/Paren，非法源码check/build0；f(1+2缺右括号被丢调用而native3，真SENTINEL三件被覆盖 | T02/05/07，AC02/04/06/08；所有出口归约后检查未闭合帧，准确诊断，无Checked/发布，保留三件 |
| P745-R5-QA-02（块项） | P2 | 调用子项已清偿；块护栏1000仍未批准入r1/SD，999可native/1000完整if已拒，实施侧明确未完成 | T02/05/06/07，AC02/07/08；继续满足原合同，或显式new修订计数/入口/资源诊断/正反验收后获批 |

R5-QA01原4096层非连续括号崩溃清偿；R5-QA02调用边界清偿（20000调用check0、4096混合native4097、未闭合受控），剩余块子项沿用原ID。历轮执行器/来源/Bundle/旧事务清偿保持；额外实参括号只是定位正控制，不能视为批准workaround。

本轮113 passed/1 ignored、741与745均13步、独立74/74、strict47通过；Rust正常与PS9/9真实失败出口无幸存。root日常37失败全部历史共现、tv4失败未称绿色。完整输入/实际PE输出/SENTINEL前后指纹/冻结delta见第六轮附件，登记不代表批准延后。
## PLAN-745 r1 第七轮最终独立复审（2026-10-08，清偿）

reviewed_commit=bb8cb4b4330d505059ea20e7bfe561a93c1051f3；**pass**；[第七轮报告](attachments/745/review-r7/REVIEW.md)、[计划](archive/745-ac-source-checked-hir-native.md)。原同树保留，reviewed 8/9；没有合入、归档或canonical/live ledger沉淀。

| 原ID | 处置 | 独立证据 |
|---|---|---|
| P745-R6-CLOSE-01 | 已清偿 | positional/named二元实参42、嵌套192、异构bool5、trace23/ba、H09恢复111；括号内非法逗号仍拒 |
| P745-R6-CLOSE-02 | 已清偿 | return/let/if/while缺右括号均text/source.syntax准确拒绝；实际M21 check1/build1，exe/obj/receipt三件旧SHA逐件相同 |
| P745-R5-QA-02（剩余块项） | 已清偿，原finding整体关闭 | 无固定嵌套计数护栏；1000if/1500if-else/2048while真实PE、20000if到Checked、深未闭合受控；新增N01..24块流/作用域/循环/trace矩阵全过 |

历轮其它清偿保持；本轮无未清偿合同内finding。114 passed/1 ignored、741/745各13步、独立74/74、strict47新鲜；真实Bundle及执行器失败控制保持。root日常30失败全部历史共现、tv4历史semver失败，不称全绿或严格同源码main对照；生产路径无745 diff。冻结delta与终审HEAD/依赖/Specs/配置无漂移，证据见第七轮附件。T09真实closeout与批回归到期判断仍待执行，既有P745-D1中央取号脚本债务不在本计划修复范围。
## PLAN-745 合入批量回归观察（2026-10-08）

745 的独立源码 profile 合同已通过；以下属于根测试/外部输入的观察，未静默缩减验收或把失败报成全绿。完整原始日志与逐名归因见[合入报告](attachments/745/merge/MERGE.md)。根生产源、配置及依赖清单在落地前后无745变更。

| ID | 观察 | 后续处理边界 |
|---|---|---|
| P745-MERGE-ENV-01 | temp_plan009_t8_transpile_at_app 硬编码外仓 auto-term 输入路径不存在，tt失败 | 后续独立计划修测试环境合同；本次不创建或写外仓路径 |
| P745-MERGE-VERIFY-01 | tb 的55个book金样差异，且未ignore的generate_book_expected在同批成功写外仓金样；并发验证oracle会变化 | 后续独立计划隔离生成与只读验证；无外仓运行前快照，保留现存修改，不重生成/回滚来制造绿色，不等同旧七个parse失败 |

stage3_memory_baseline_n1_3_5 与 plan707_client_sse_poll_close_error_surface 批量失败后，单实例、单job各1/1通过；按负载时序观察记录。tf/tt/tb分别31/34/76失败均保留原始证据，新增未匹配名没有自动归为历史红。
## PLAN-774 状态核查（2026-10-13，规划交接漂移）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P774-D1 | medium / closed | 计划交接与取材 | 原共同契约769改为774后，活跃执行指令/意向工作树仍用769，标量前置链接仍指向不存在的旧767文件；770/771仍引用已不存在的769共同契约文件，消费者auto/lib路径亦已迁移。启动实施前校正活跃编号/依赖/取材，历史附件保留来源；这不是已完成实现的缺陷或豁免。774实际drafting 0/8，探针/报告/脚本/共同Spec均未产出。 | docs/plans/774-ac-data-resource-contract.md §4/8、770/771 §0；[状态核查](attachments/774/status-audit-20261013.md)；owner=774 plan handoff |

P774-D1销号（2026-10-13）：770/771确认未改号；774/770/771活跃依赖、执行指令、工作树入口、真实消费者路径及战略/模块导航已同步。历史附件保留来源，当前指纹已刷新；仅引用缺陷关闭，774仍drafting 0/8。证据见[引用更新收据](attachments/774/reference-update-20261013.md)。

## PLAN-770 r2 第二轮最终复审（2026-10-14，待修）

reviewed_commit=`5061c20a9e63b79da0d9bb505f35c40cfb549484`；outcome=needs_fix；
[第二轮报告与重放集](attachments/770/review-r2/REVIEW.md)、[计划](770-ac-text-char-native.md)。
下表为合同内缺口，登记不批准豁免/延期；执行器 gate 13/13 与 ac-core 200 passed/1 ignored、
runtime 20 passed 无法覆盖这些独立 source/HIR/Bundle 反例。原树保留，executing 1/9；
未修改实施源码、合入、归档或 canonical/live ledger。上轮 F-01 marks oracle 已关闭。

| ID | 级别 | 当前缺口 | 修复/验收归属 |
|---|---|---|---|
| P770-R2-QA-01 | P1 blocking | Text Bundle.domain 改 Core/Scalar 后 verify_text 仍发 Checked，native panic | T-02、AC-01/04；精确域前门拒绝，零 native 调用 |
| P770-R2-QA-02 | P1 blocking | 删除 runtime requires 后无 capability、库无 sidecar 仍真实 HIR build0/PE3 | T-02/05、AC-01/04/06；实际运行时需求与 capability/真实库合同闭环 |
| P770-R2-QA-03 | P1 blocking | return/if move 重复释放；短路跳过 move 漏释放 | T-02/05、AC-03/05；完整控制流唯一清理、路径反控 |
| P770-R2-QA-04 | P1 blocking | 默认 view Str 参数可伪转移为拥有返回，caller 双释放 | T-02/04/05、AC-03/04；名义类型与持有权分离 |
| P770-R2-QA-05 | P1 blocking | owned Call/literal 临时借用漏释放；借用 call result 先 FREE 后消费，合法输入 PE70/-7 | T-02/04/05、AC-03/04/05；完整来源与语句尾临时寿命、有效返回控制 |
| P770-R2-QA-06 | P2 blocking | 同源 mut+view、mut 参数借用返回均放行 | T-02/04、AC-03/04；verify.mut-conflict/borrow-escape 前门 |
| P770-R2-QA-07 | P2 blocking | 循环内每轮 fresh owned 首次 move 也 use-after-move 误拒 | T-02/04、AC-03/05；区分 fresh/循环外来源，正反及退出矩阵 |
| P770-R2-QA-08 | P2 blocking | 独立 runtime workspace Cargo.lock 被 ignore、未提交 | T-03/06、AC-06；锁入受审提交，干净输入 --locked 与 fresh gate |
| P770-R2-CLOSE-01 | P2 docs correction | SD len_bytes 单位、marks.obj 共链边界、SD-02 现行来源文字错误 | T-07；按修复行为与现行 canonical 改 SD，禁止缩围 |

本轮为第二次 final needs_fix；第三轮入场前须按 review/work 技能写 diagnosis/coverage/design
调整及 finding closure 表，重放本轮冻结 source/HIR/Bundle 与合法控制，再提交并完整 gate。

## PLAN-774 r3 第一轮独立最终复审（2026-10-13，待修）

reviewed_commit=`f1a26e1be51de2dbb0c0d983b5ec346a86475aed`，outcome=`needs_fix`；原唯一树保留，executing 1/8。详见[完整复审报告](attachments/774/review-r1/REVIEW.md)、[计划](774-ac-data-resource-contract.md)。以下均为合同内缺口，登记不代表批准延后或缩减验收。

| ID | 级别 | 缺口 | 修复/验收 |
|---|---|---|---|
| P774-R1-01 | P1 | 64槽资源账本满后静默丢length；65对象全部释放真实PE exit0但live_bytes=1，预算65再次申请65误拒；预算0隐式当unlimited | T03/04/06，AC03/04；完整准确记账、live_bytes不变量、释放整额回收；裁定预算0语义，补64/65及0控制 |
| P774-R1-02 | P1 | ReturnViewParam不查来源/状态；已free局部可标成参数返回；M4兄弟实际只返回i32；field/container拒绝码仅列名，move参数要求无入口证据 | T02/03/04/06，AC01/03；来源/访问模式的真实模型前门、pointer/length借用返回与调用者使用、逐语境正反/合法兄弟PE |
| P774-R1-03 | P2 | 宣称LIFO但clone/branch按FIFO free；临时/嵌套break/continue/参数转移边界和拥有资源下部分申请失败缺区分断言 | T03/04/06，AC03；有ID/顺序trace的清理oracle、代表性全路径及clone内容/存活控制 |
| P774-R1-04 | P1 | 参数Str统一move与既有双侧默认view冲突；770/771 profile和字符转换未同步；跨行/转义决定及nested record布局缺项；R2失败被泛化为方案不可能 | T02/05，AC01/04；核查定义处访问模式/源拼写，唯一身份及下游修订；冻结literal和递归布局规则；限定候选结论，不自改既有承诺 |
| P774-R1-05 | P2 | frees计调用而非实际释放、忽略HeapFree结果；双free负控读取已释放header，不能保证确定性 | T02/03/04/06，AC03；有效账本先验、成功/尝试/非法分离、OS结果检查；保留成功释放口径，补可靠负控 |
| P774-R1-06 | P2 | 多行warning检测漏报；shim build120s/未locked；V06超时前驱未观测；收据丢case原文/产品与合同hash；SD误称List已冻结、空Goal未解释 | T04/05/06，AC05；真实正反门禁/60s/locked/持久结果与hash，可观察前驱；纠正SD和边界说明 |

原门禁774 6步/probe39、ac-core145 passed/既有1 ignored、741 13步/745 74例/772 45例全部通过；独立7控制3通过4失败，输入和实际输出见报告附件。合法UTF-8边界、多view/mut控制及平面ABI通过面保留；本轮未修实现、合入或发布canonical/live ledger。

## PLAN-774 r3 第二次交付就绪核查（2026-10-13，待修）

reviewed_commit=`773f9a09aeb874845dc0a0cf44d89a07e3b3a87f`；needs_fix，原树保留，executing 1/8。已知work-owned未完事项使本轮不进入完整最终复审；报告见[review-r2](attachments/774/review-r2/REVIEW.md)。保留第一轮发现和旧样例清偿，不把明示遗留当豁免。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P774-R2-01 | P1 | incoming mutable参数明示未建模；新ParamBorrowed→View→EndView直接panic，2个参数return正反控制通过 | 延续R1-02，T02/03/04/06、AC01/03；来源×模式×区域结束矩阵、合法兄弟PE、非法受控拒绝 |
| P774-R2-02 | P1 | 770/771仍r2旧身份/转换拼写，改齐被推到下次revision；跨行和常驻字面量返回被新附录削减，未见变更授权 | 延续R1-04，T02/05、AC01/04；先实际同步下游，保留合法支持或显式new/revision，冻结表/SD一致 |
| P774-R2-03 | P2 | 新ledger HeapAlloc增长失败及user分配后回滚无注入/执行oracle，metadata请求计数口径未明 | 延续R1-01/05，T03/04/06、AC03；首次/扩表失败+已有owner+回滚，准确保留/释放/计数，不称已实测泄漏 |
| P774-R2-04 | P2 | work收据HEAD仍f1a26e1be，verification §8 raw-134941实际不存在（目录raw-134911）；无blocker/全关闭声明与模型/下游遗留冲突 | 延续R1-06，T06、AC05；区分历史与提交后证据、实际目录/源hash/SD冻结，已知work缺口完成后再execution_done |

本轮原7控制7/7通过，原4失败样例已清偿；来源组合3控制2通过/1panic；warning正反实际通过。work原始55/55核对后复制，未冒称本轮重跑全套或最终pass。两次未通过交付的诊断/覆盖调整表见报告；合入、canonical/live ledger与清树均未执行。

## PLAN-774 第三轮独立最终复审（2026-10-13，待修）

reviewed_commit=`b3bab4278c9f065a5609a57d02d1f2552f640eab`，needs_fix；原树保留，executing 1/8。详见[第三轮报告](attachments/774/review-r3/REVIEW.md)。前轮7/3控制均通过，来源EndView panic、literal恢复、下游同步、释放口径及提交后收据已实际清偿；不重复打开其旧样例。

| ID | 级别 | 缺口 / 建议 | 修复/验收 |
|---|---|---|---|
| P774-R3-01 | P2 | i64预算以as u32保存，4GiB/4GiB+1下仅申请2 bytes误拒；u32最大预算对照通过 | T02/03/04/06，AC03/04；数值域/表示/checked比较一致、不得静默换预算；两个失败控制及既有0/负值/释放额度控制重放 |
| P774-R3-02 | P2 | V05-15在扩表失败后先free，重试len15→16不触发grow，绿色未证明声称的恢复后扩表 | T03/04/06，AC03；保持16owner重试或另加真扩表恢复永久oracle、内容与peak断言；独立真实17项恢复本轮已通过，非实测泄漏 |
| P774-R3-D1 | P3 / nonblocking | SD仍标39与§3.8增补、非法矩阵写15实际16；SD引用整个当前§3/接口可解析有效§3.9，不能因同hash单独判delta失效 | 随预算修复更新当前引用/统计；canonical仅沉淀有效规则，不搬历史作废条款；非独立阻断项 |

本轮重跑774 7步/61 tests、ac-core145 passed/既有1 ignored、741 13步/745 74例/772 45例全绿；旧反例7/7和3/3通过。新增4个真实PE控制2通过2预算失败；完整输入/输出/重放/冻结SD/hash在报告附件。只有前两项阻止验收，登记不授权延后；未实施修复、合入/归档或canonical/live ledger沉淀。

## PLAN-776 债务候选登记（2026-10-13，T-05 执行发现）

| ID | 级别 | 问题 | 证据与修复要求 |
|---|---|---|---|
| P776-D1 | medium | handler 任务栈残留对象 RC 未清 | `remove_task`（engine.rs:845）只移任务与 sqlite 句柄，不清任务栈槽 rc——handler 局部堆对象在任务结束时残留引用（VM 既有模式，PLAN-776 T-05 的凭据 live 计数使其可见；T-05 已修**返回值槽**双点：sync `HandlerEnd::Value`/parked `HResume::Value` 的 `rc_release(nv)`）。通用面=任务栈逐槽释放或 drop 钩子，属 RC 系统改造（PLAN-776 §1 非目标）。1000 闭环凭据面已回基线（776-return-resource-verification.md §4.1） | crates/auto-lang/src/vm/engine.rs:845；crates/auto-lang/src/vm/ffi/http_server.rs（HandlerEnd::Value/HResume::Value 双点） |
| P776-D2 | low | 点链 `http.response.redirect` CALL_SPEC 形态静默 None | 主线预存（plan352 注释在案：triple-dot native 需 CALL_SPEC 链式分派）；真实 wire 契约=下划线 `http.response_redirect`（CALL_NAT，e2e_a_redirect 同款）。PLAN-776 未扩面（资源臂不影响该形态的既有行为） | crates/auto-lang/src/tests/plan352_tests.rs:215；vm/native.rs 契约注册 |
| P776-D3 | medium | 缓存 Internal Future 的 `.await` 组合 sync-await 帧缺陷（归 705 轨） | PLAN-776 R3 分诊（F776-R3-03，2026-10-14）：全局缓存 `~{...}` future 再 `.await`（含跨 handler 重放）→ 500 `execution terminated unexpectedly`、重放恒 0。机制：`execute_future_body` 复用当前帧（只存取 ip 不保 bp/sp）——body RET 弹掉 handler 帧，handler 自身 RET 落 bp==0 → Terminated；结果槽随帧错位，future 被写 Ready(Int(0))。**归属 705 状态机**（Plan 776 §5.2「Future 的展开/等待仍由 705 状态机负责」）：数据体 `~{42}` 同形失败（无资源参与）、776 diff 零触该机制、失败在返回编组之前（base 5e21561d2 即有，engine diff 仅 rc_release 不在数据体失败路径）。776 边界不变量（有界 500/重放数据面/凭据回收）由 `http_e2e_plan776_r3_cached_future_await_triage` 钉住——**705 修复 sync-await 后该测试断言翻转，移交 705 轨**，不得在 776 内静默改判 | 复现：plan776 测试 `http_e2e_r3` 模块（提交 73d4f0ff3）；机理：vm/engine.rs `execute_future_body`（帧复用）+ RET `bp==0` 短路；分诊记录：776-return-resource-verification.md §4.5 |
| P776-D4 | high | rapid SSE 早断连 churn 下 HTTP 服务级死锁（归 plan707 轨） | PLAN-776 R7 分诊（F776-R7-03，2026-10-14）：SSE 流首帧后立即弃连的快速序列（harness read_wire 形态；≥2-10 连发）后服务整体停摆——新连接 accept 后零响应、`/ping` 类普通路由也无响应（服务级死锁，非单流泵死）；观测形态两相：零字节或「200+chunked 头后零 chunk」（R7 报告的 S3 有头无帧）。**base `5e21561d2`（776 之前，老查表 SSE）同款实验 2/6 复现同形**（判别 worktree：同端点 ×10 早断连，探测=None）——预存缺陷，归属 SSE 服务/断连清理/调度域（plan707/hyper serve 族），非 776 diff 引入。776 侧：S3 oracle 带 15s 有界+探测诊断（`http_e2e_plan776_r7_s3_internal_stream_frames_bounded`）；marshal future 驱动的 task try_lock 已加有界重试（防静默跳过）。**修复归 plan707 轨**（断连清理/泵调度机制），776 不扩面接管 | 证据：plan776 R7 修复轮节（判别实验记录）；oracle 失败信息含探测结果 |
| P744-D1 | medium | 纯 VM 后端轨（`--server vm`）三连缺陷：fs 奇偶分裂 + service 链接缺 Type.method 形 + handler 上下文 fs.exists 疑似误报 | PLAN-744 r2 重绑定时发现（2026-10-14，auto-notes main@1391bd7 探测）：①`fs.copy_recursive` 两轨语义分裂——`stdlib/auto/fs.at:58` 声明 `void`+`#[vm]`（失败抛 RuntimeError），a2r 轨返回 bool；依赖 bool 语义的 .at（notes repository.at `fs_copy`）在 VM 后端轨 500（legacy 升级门）。②`auto service --server vm` 链接失败 `Undefined symbol: SqliteDb.exec in module p2probe`——service 每-模块解析缺 Type.method→模块函数映射；run 模式扁平化有（同代码 `db.exec` 建表成功佐证）。③后端 VM handler 上下文 `fs.exists` 疑似对不存在路径误报 true（fresh 目录 meta 恒缺 `upgraded` 行+copy 500 佐证；独立脚本 VM 探针 exists 语义正确——需源级插桩隔离根因）。三项均非 sqlite 模块缺陷（744 的 VM shims 在 run 模式实证执行）；修复属 fs 模块对齐 + VM 符号解析，影响纯 VM 后端轨可用性 | 复现与证据：docs/plans/attachments/744/r2-evidence/（vmserver-probe-boot-tail.log + service-vm-link-failure.log + README）；stdlib/auto/fs.at:58；crates/auto-lang/src/vm/native.rs:12025-12050（copy_recursive raise 点）；notes 仓 src/back/repository.at fs_copy/upgrade_if_legacy |

## PLAN-774 第四轮独立最终复审（2026-10-13，待修）

reviewed_commit=`c5e631bd43d1855ba76e144dfac31be3da487bd1`，needs_fix，原树保留，executing1/8。详见[第四轮报告](attachments/774/review-r4/REVIEW.md)。R3-01旧setter截断、R3-02恢复扩表与D1整理已清偿；历轮通过反例不重新打开。

| ID | 级别 | 单一当前阻断 | 修复/验收 |
|---|---|---|---|
| P774-R4-01 | P2 | 预算已u64，live_bytes/peak仍u32并saturating_add；累计越界后少记，下一比较允许超预算请求。受控host状态probe：expected4294967298、actual4294967295、extra_2_admitted=true | T02/03/04/06、AC03/04；累计表示/保存/加减/比较/观测一致准确，或明确受控表示边界；补跨u32累计、恰等/超一、释放后剩余与再申请额度oracle |

本轮774 7步/63 tests、ac-core145 passed/既有1 ignored、741 13步/745 74例/772 45例均通过；r1/r2/r3控制7/7、3/3、4/4通过。新失败证据是原shim函数体的受控记账状态probe，只申请小块物理内存并注入逻辑账本，**不是4GiB真实native申请实测**；原源码/构建适配/输入/输出/可再生重放已冻结，见报告。未修实现、合入/归档或发布canonical/live ledger；此登记不授权延后预算不变量。

## PLAN-776 第一轮独立最终复审（2026-10-14，needs_fix）

审查提交 `b4183e761199642aa341cfecd6491c1821c9e2e4`；Plan revision=1，executing，T-01..08重开。详见 [R1报告](reports/776-return-resource-review-r1.md) 和可重放附件。本表全部是原合同内必须修复项，登记不授权延后或降低验收。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R1-01 | P1 | 普通0与活跃iterator同号变SSE；旧Response表与五个服务端sugar producer仍裸id | AC01/02/03/06，T01/02/03/07；全部egress来源迁移、实际backing id碰撞wire、不保留猜测臂 |
| F776-R1-02 | P1 | 新dispatch无条件rc_release把同值普通i32扣到全局凭据RC，跨宽限窗误回收 | AC01/02/05/06，T02/03/05/07；按实际slot stake/identity释放，数字碰撞后drain再重放 |
| F776-R1-03 | P1 | 两VM同号凭据编码相同，A值在B取出B响应 | AC05，T01/02/07；可验证owner/session/代次，双方同时有同号资源反例 |
| F776-R1-04 | P1 | 真Internal async返回/构造后异常，11请求drain后各残留11凭据 | AC03/04/06，T04/05/07；完整HTTP资源终态释放与Future结果持有清理；P776-D1非目标不能豁免此scope内缺口 |
| F776-R1-05 | P2 | 用户alias和空Response声明仍分类ExplicitResponse，来源不是解析绑定 | AC07/08，T01/06/07；共享decl/import/alias身份、三消费者对照 |
| F776-R1-06 | P2 | 新307凭据交真实status shim读到0，旧表控制307 | AC02/07，T02/06/07；producer→访问器/setter新载荷适配与独立契约 |
| F776-R1-07 | P2 | heap id 2^32仍Ok，u32截断后凭据失效/可能取同号项 | AC05，T01/02/07；实际载荷边界前拒绝、边界与活跃同号控制 |
| F776-R1-08 | P2 | derive Clone不加存活计数、Drop无条件减，clone/drop使仍存原件漏计 | AC06/09，T02/05/07/08；Clone/Drop配平或禁clone，owner表和RC佐证oracle |
| F776-R1-09 | P2 | 无await用例冒充三Future形态、生成lib绿冒充真wire；吞startup、固定端口；新增warnings/mod.rs格式红；SD失配 | AC03/04/07/08/09，T01/04/06/07/08；先补实现与真实矩阵、再更新收据/SD和健康检查 |

本轮专项25/25、语料162/162、755守卫4/4、auto-man专项1/1、api_gen46/46通过；完整t=5294/5295、th=108/110，chart/upload/SSE scoped重跑绿保留为间歇红待归因。四个P1反例及五个P2均见冻结输入/输出；没有修实现、合入、归档或写canonical/live ledger。

## PLAN-774 第五轮独立最终复审（2026-10-14，needs_fix）

reviewed_commit=`8e4fb77289ad95eacfa2538f33dd460da2d707cf`，原树executing1/8；详见[第五轮报告](attachments/774/review-r5/REVIEW.md)。R4-01的累计保存/比较/释放恢复已清偿，r4状态控制和永久探针均通过；观测同步链还有下述单一阻断，历轮通过面保留。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P774-R5-01 | P2 | SUMMARY按attempts/allocs/frees/live/live_bytes/peak排列，summary_of末项仍u32并as u32；合法peak=4294967298被读成2，文档重复错误混合类型；永久状态探针未验证输出→解析 | D06、V08、AC01/03/04/05，T02..06依赖链；按字段统一类型（live对象数u32、live_bytes/peak u64），同步decisions/interface/probe-cases/verification/SD，永久合法上界/跨界观测oracle；重放review-r5/replay.ps1不得改expected为截断值 |

本轮774 8步（63探针+状态1/1）、ac-core145 passed/既有1 ignored、741 13步/745 74例/772 45例全绿；旧r1/r2/r3/r4控制7/7、3/3、4/4、1/1通过。新解析控制1通过1失败（exit101），是合法协议输入对真实已提交parser的检查，**不是4GiB真实native申请实测**。未修实现/合入/归档或更新canonical/live ledger；登记不授权延后现行观测合同。

## PLAN-774 第六轮独立最终复审（2026-10-14，pass／闭合）

reviewed_commit=`c713aaafb5dfe73fd587a2e52f53f83c3f40355d`；原树clean，Plan reviewed7/8、T07完成、T08待授权与merge-owned收口。详见[第六轮报告](attachments/774/review-r6/REVIEW.md)。

P774-R5-01已清偿：summary_of按字段返回(u32,u32,u32,u32,u64,u64)，峰值不窄化；永久V08-7上界/跨界、r5原解析控制2/2通过，decisions §3.12明确勘误及interface/probe-cases/verification同步。新增原shim formatter→实际stderr→原parser三组合控制3/3，区分live对象数/live_bytes/peak，并验证全释放后仍保留宽峰值。该观测链连同r4累计保存/比较/释放恢复闭合，未降低expected或变更支持范围。

R1六项、R2四项、R3两项、R4一项、R5一项共14历史阻断全部闭合，R3-D1整理完成；历史ID/失败证据保留，以上待修条目为历史记录，不再是774当前阻断。五轮重放7/7、3/3、4/4、1/1、2/2全绿；本轮64 probe tests+状态1/1、ac-core145 passed/既有1 ignored、741/745/772旧门禁全绿。无新增债务/未授权延后；全语言/生产ABI/GC/List/ACC自举仍是明确非目标。review未合入/归档/发布canonical/live ledger；T08收口仍pending，不借复审pass预勾。

## PLAN-776 第十轮独立最终复审（2026-10-14，needs_fix）

**R14 闭合补注（2026-10-15，pass）**：冻结提交`e65c01012807ca1fa42235ba2971d245d57b5738`独占格释放前实际比对expect快照，F776-R13-01警告已消除；F776-R11-01关闭保持。776专项66/66（34.328s，0timeout），format/diff-check通过，无新增警告；生产/矩阵验收pass。status=reviewed8/8，merge-owned裸t/tv/HTTP与生成验证、Spec沉淀/归档/wt-guard及到期batch tf仍pending，未合入或交付。原P776-D1..D4分诊保持。详见 [R14报告](reports/776-return-resource-review-r14.md)。以下保留历史记录。


**R13 补注（2026-10-14）**：`367f4b288b515d49938a0b5f3bf47cf9de6f9b33` 已补齐剩余8格与最终回收断言，F776-R11-01关闭；生产/15格矩阵pass保留。新增 **F776-R13-01 / P3**：测试exclusive helper的expect预期参数未使用，实际产生unused variable编译警告；按Mandatory Independent Review Health Check仅重开T08，executing7/8。建议释放前使用expect比对快照（不碰回收后id），提交并定向复审。测试1/1、format/diff-check通过，后置merge门禁pending，不授权延期。详见 [R13报告](reports/776-return-resource-review-r13.md)。


**R12 补注（2026-10-14）**：`d0fa9e5e2d0b50924698a01c591541f6e8744c8c` 已补真实五setter调用及RC2共享/mutation断言，定向测试1/1通过；F776-R11-01部分关闭。四个非status setter仍缺R11已要求的RC1独占/错误receiver共8格，shared settle最后释放后无回收断言。延续同一P2 finding，不新增功能范围、不授权延期；生产修复pass不变。T07/T08重开、executing6/8；后置merge门禁pending。详见 [R12报告](reports/776-return-resource-review-r12.md)。


**R11 闭合补注**：`38ad38abaf7d0fc40adacbb775d3f0258aaa53a7` 五 setter 生产修复通过原 shared Response UAF 重放及15格独立共享/独占/错误 receiver 矩阵，F776-R10-01 运行时缺陷关闭。尚有 **F776-R11-01 / P2**：`plan776_r10_setter_single_release_shared` 声明的五项函数指针表从未调用，实际各段只测 status，另外四项仍占位参数；须在同树提交实际五 setter 回归矩阵、更正覆盖声明（T07/T08，AC02/06回归证据及AC09），不是可延期债务。专项66/66、协议守卫126/126通过，未发现新运行时红；status=executing6/8，后置merge-owned门禁pending。详见 [R11报告](reports/776-return-resource-review-r11.md)。以下保留 R10 历史证据。

reviewed_commit=`dd41d684dc2d8c1321034ed56d4a8cb38eab2894`，revision1，实施树clean。
详见[R10报告](reports/776-return-resource-review-r10.md)及review-r10反例/重放/log/hash。
R9两项原例与record兄弟路径已闭合；前轮遗漏的共享Response setter为原合同内P1，登记不授权延后。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R10-01 | P1 | status/header/text/html/bytes仍显式rc_release+StakeGuard Drop双消费；shared status RC2→1，释放返回槽后0销毁globals并触发UAF | AC02/06/07/08/09、T01/02/05/07/08：全setter恰一份消费/返回owned份额、RC1/RC2+、丢弃/保存/串联、错误早退、跨reap共享replay；更正SD/decision协议 |

本轮专项65/65、协议守卫针对性126/126、format干净；iterator共享、Future record/赋值/object/array/closure控制通过，shared-response新反例失败。整档后置merge-owned pending。status退executing3/8，T03/T04/T06及历史已证子项保留；未修实现/合入/归档/发布canonical或ledger。

## PLAN-776 第九轮独立最终复审（2026-10-14，needs_fix）

reviewed_commit=`5b8bffdef3889008d1cc84d85cfda35b01c1afa3`，revision1，实施树clean。
详见[R9报告](reports/776-return-resource-review-r9.md)及review-r9/source/重放/log/hash。
R8旧例闭合，但完整不变量失败；以下为原合同内必须修项，登记不授权延后/缩减。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R9-01 | P1 | iterator共享源RC2经显式release+不retain的StakeGuard Drop减两份到0，globals仍持有而reap后消失 | AC02/05/06/07/08/09、T01/02/05/07/08：每个owned槽只消费一份，独占/共享/早退/派生/next+terminal矩阵；保留清零和旧净零控制 |
| F776-R9-02 | P1 | typed Box.f Future<int>构造/取出仍丢tag，HTTP返回JSON1而非Future终值42 | AC02/03/08/09、T01/02/03/04/07/08：CREATE_OBJ及ObjectData/GenericInstance字段读补完整身份，所有Value往返入口闭合表，普通字段对照 |

本轮专项63/63、触面format干净；数组/数字/closure代表例与独占/slot旧例通过，shared与typed-record新反例失败。整档按后置裁定未跑，合入门禁merge-owned pending。status退executing1/8，T06及历史已证子项保留；未修实现/合入/归档/发布canonical或ledger。

## PLAN-776 第八轮独立最终复审（2026-10-14，needs_fix）

reviewed_commit=`5d4d29ab0fac4cb2b18f7a720d55eb5a18a02213`，revision1，实施树clean。
详见[R8报告](reports/776-return-resource-review-r8.md)及review-r8/source/重放/log/hash。
以下为原合同内P1必须关闭项，登记不授权延后/缩减；旧R7功能例和类型来源进展保留。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R8-01 | P1 | TAG_FUTURE经Value转普通Int；~int函数普通496仍驱动成42，数组取真实Future则JSON496 | AC02/03/05/08/09、T01/02/03/04/07/08：所有合法载体保逐值来源，撤raw位/声明猜测；同时覆盖~T声明真碰撞数字与容器/捕获真实Future |
| F776-R8-02 | P1 | pop_iterator_receiver rawpop无清零/消费源凭据stake，guard解包iterator id；源消费后RC1，输出释放reap仍存活 | AC02/05/06/07/08/09、T01/02/05/07/08：原始nv读后释放+slot清零，错误早退与派生输出份额正确，next/全adapter/terminal owned矩阵；更正交接与SD清零声明 |

本轮专项60/61（/s0有头无帧，非前条误定位的/s3），该例单跑1/1绿；P776-D4分诊进展保留，不冒61全绿。auto-val NanoValue16/16、触面format干净。整档t/tv/th按2026-10-14后置裁定未跑，merge-owned pending；status退executing1/8，T06及历史已证子项保留。未修实现/合入/归档/发布canonical与ledger。

## PLAN-776 第七轮独立最终复审（2026-10-14，needs_fix）

reviewed_commit=`5fda97cebd952397b41af80f7ea5f076534223f9`，revision1，原实施树clean。
详见[R7报告](reports/776-return-resource-review-r7.md)与review-r7实际source/重放/log/hash。
原R6示例关闭不等于完整不变量闭合，登记不授权延后/缩减验收。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R7-01 | P1 | fn Debug串Return(AsyncBlock全函数登记；if false里返回Future使普通496被驱动成42，helper实际Future仍变JSON496 | AC02/03/04/05、T01/03/04/07/08：当前终值真实来源，覆盖同函数不同分支/同值数字/helper正例，不再以fn标记冒血缘 |
| F776-R7-02 | P1 | generator无条件产凭据，但enumerate仍pop_i32；同源raw控制=新句柄1，credential=-1 | AC02/07/08、T01/02/07/08：全部native消费者统一凭据解包/owner/ownership及派生身份，保留非HTTP旧调用控制 |
| F776-R7-03 | P2 | plan776两轮/s3 SSE200头后30s无帧；单跑绿，未证明端口/harness归因 | AC03/06/09、T03/04/07/08：归因真实调度/生命周期/锁/读协议，保留完整红收据；本轮未证明新回归，不冒称必为环境 |
| F776-R7-04 | P3 | alias目标改完整形态后bare_type_name无调用，新增dead_code | AC09、T06/08：清旧helper与失效注释；诊断DBG无断言测试建议改oracle或移证据，不计覆盖证明 |

本轮t5308/5308、tv162/162、生成JSON scaffold/bus SSE1/1；专项首次20pass/1fail/37未跑，完整复跑57/58，失败精确单跑1/1绿；th见R7最终报告。旧int/helper/alias反例已闭合，新增Future与native反例失败。status退executing1/8，保留历史通过子项；未修实现/合入/归档/发布canonical或ledger。

## PLAN-776 第六轮独立最终复审（2026-10-14，needs_fix）

reviewed_commit=`5bdca42e49483a79c8a0b69c4758b24986b8238b`；revision1、executing1/8；原树clean。
详见[R6报告](reports/776-return-resource-review-r6.md)及review-r6/source/重放/log/hash。以下均为原合同内必须关闭项，登记不授权延后/缩减。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R6-01 | P1 | CREATE_GENERATOR包装仍看所在fn声明；int端点/local及int helper产出真实流仍JSON0 | AC02/03/08、T01/02/03/07/08：producer逐值来源跨旧int/helper，保留非HTTP与同值普通数控制 |
| F776-R6-02 | P2 | Download=Future<FileResponse>抹为alias_of Future；合法九alias链因新增深度8静默Json | AC07/08、T01/06/07/08：保留完整alias目标/包装，visited循环检查，不引入未批准合法链深度限制，三消费者一致 |
| F776-R6-03 | P1 | stream声明+活跃fid1时return普通496竟驱动~{42}，wire变42；int声明对照正确496 | AC05/02/03/04/09、T03/04/07/08：Future真实来源守卫，声明/bit/表不能授身份，真实Future正例与活跃同值数字负例 |
| F776-R6-04 | P2 | File/Upload生成wire2/2补齐，但JSON/bus SSE仍仅生成级验证，AC08缺必需oracle | AC08/09、T07/08：可运行fixture/实编/真TCP；依赖前置未满足则显式记录或经用户批准改合同，不冒final pass |

旧R5流声明/限定名原例闭合，新增warnings关闭；本轮54/54、生成2/2，但新行为反例失败。t/tv/th未再跑：交接明确必需oracle未完成，按就绪审计针对性返回work；不拿work总数冒充final验收。未修实现/合入/归档/发布canonical与ledger。

## PLAN-776 第五轮独立最终复审（2026-10-14，needs_fix；推翻R4 pass）

审查提交 `73d4f0ff3a89f507c4192e38304354688ea00b52`，Plan revision=1；原实施树clean。
详见 [R5报告](reports/776-return-resource-review-r5.md) 与 attachments/776/review-r5/实际探针/重放/log/hash。
下列为原合同内必须关闭项，登记不授权延后或范围缩减。旧修复/历史证据保留。

| ID | 级别 | 缺口 | 修复/验收 |
|---|---|---|---|
| F776-R5-01 | P1 | 同一generator直接return为SSE；存local/helper return为JSON0，Internal body return为JSON数字496 | AC02/03/08，T01/02/03/07/08：producer来源身份跨local/helper/Future，保留普通同值数字负控制，不恢复声明/表授身份；扩组合永久oracle |
| F776-R5-02 | P2 | 用户同名遮蔽http.FileResponse限定名；Download=http.FileResponse及alias链均误判Json | AC07/08，T01/06/07/08：按实际绑定解析限定名/alias链/包装与循环，三消费者同源；保留用户同名Json |
| F776-R5-03 | P2 | AC08生成真TCP仍partial却final pass；未见用户批准移除该必需oracle | AC08/09，T07/08：补现行支持面实际生成/实编/wire；确受742阻塞则明确前置或经用户批准修改合同，不能以预存归因冒充验收 |
| F776-R5-04 | P2 | 新增fn_is_generator/handler_declares_stream_return/handler_is_generator/declared_return死代码、codegen block内doc注释warnings及旧表/声明门注释残留 | AC09，T06/08：清触面新增warnings/失效侧道与注释；不扩全仓预存warning整改 |

本轮plan77649/49、t5304/5304、tv162/162、th124/125唯一corpora在册红；api_gen45/1failed/1ignored，742全名定向1/1绿（共享fixture敏感）。基线稳定但新行为反例失败，status退executing2/8；未修实现/合入/归档/沉淀Specs或live ledger。

## PLAN-776 第二轮独立最终复审（2026-10-14，needs_fix）

审查提交 `6494092de6c33875b5e99f81491bb25815852b38`，修复基线b4183e761，Plan revision=1、executing0/8。详见 [R2报告](reports/776-return-resource-review-r2.md) 和冻结source/原始输出。已通过的R1原例不抹去；本表全部为原合同内当前必须关闭项，登记不授权范围缩减/延后。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| F776-R2-01 | P1 | 声明~Iter后普通0仍因活跃iterator表命中变SSE | AC01/02/03，T01/02/03/07；按值producer来源，声明/表不能授身份，保留真实generator正例 |
| F776-R2-02 | P1 | 7位owner按创建次数重复；A=1与129同号，A值在129取129响应 | AC05，T01/02/07；owner/session不复用、耗尽受控拒绝，少量live+历史VM与nv/VmRef控制 |
| F776-R2-03 | P1 | status/header/text/html setter drop stake后raw push，返回RC0；跨窗UAF、owner panic/wire503 | AC02/05/06，T02/05/06/07；owned返回stake转移/重建，全部setter串联/保存/丢弃/跨窗 |
| F776-R2-04 | P1 | remove_task只扫顶层credential；Box.response持有资源的11异常请求drain后live11 | AC02/04/06，T02/04/05/07；实际task-owned容器/捕获子路径收口、保留escaped/shared正控制 |
| F776-R2-05 | P1 | 真http.get上游201/upstream，服务返回200 JSON裸id2；client被悄悄排除身份边界 | AC02/03/06/08，T01/02/03/07；遵守Plan显式client egress范围，按值来源+同值数据控制+生命周期 |
| F776-R2-06 | P2 | response_bytes仍pop_i64+旧表；有效字节列表控制shim Ok但snapshot空、返回失去资源身份 | AC02/07，T01/02/06/07；bytes载荷/owner/ownership接线，语言字节控制和二进制roundtrip |
| F776-R2-07 | P2 | alias只加入空名集，Download=FileResponse及http.FileResponse分类Json | AC07/08，T01/06/07；实际decl/import/alias target来源，stdlib正例/用户同名负例三消费者一致 |
| F776-R2-08 | P2 | External用例仍未await、无完整矩阵/取消资源oracle；生成wire失败；mod.rs fmt红；SD存在未批准缩减 | AC03/04/07/08/09，T01/04/06/07/08；先实现完整不变量和实际oracle/生成wire，再刷新收据/SD |

R1-F02、F07 local边界、F08关闭；直接Internal/Err资源、用户Response=int/空声明、status即时读取、未声明int碰撞旧例绿。但R1-F01/03/04/05/06/09完整不变量仍未闭合，setter引入回归。专项38/38、语料162/162、755 4/4、api_gen46/46绿；完整t5299/5300、th116passed/1failed/1timeout，原始门禁不改写成全绿。没有修实现、合入、归档或canonical/ledger沉淀。

连续第二次最终复审needs_fix。R3入场前work必须交付剩余不变量/漏测原因/路径状态与设计或oracle调整/当前结果的诊断与闭合表；不通过只重放旧例、internal pass或改revision重置失败轮次。

### PLAN-744 R1 完成情况核查（2026-10-14，needs_fix）

以下为计划内未完成项，登记不等于批准延期或允许按完成归档。详见
[R1 核查报告](reports/744-completion-review-r1.md)。P744-D1 的 service/fs 缺陷另行保留。

| ID | 级别 | 验收/任务 | 当前缺口与处理 |
|---|---|---|---|
| F744-R1-01 | medium | AC-01/T-01 | 测试设计承诺 VM 嵌套堆列表逐格断言，实际 roundtrip 仅断言 `shim_sqlite_query_impl` 返回的 Rust `Vec<Vec<String>>`，未覆盖 `shim_sqlite_query` 的字符串池/inner heap/outer VmRef/栈路径。需补真正经过 shim 或 VM 字节码的多行多列 oracle；当前未证明运行错误，不可宣称完整验证。 |
| F744-R1-02 | medium | AC-01/T-01 | scoped sqlite 2 passed 不等于既有 VM 无回归；缺 744 绑定提交/依赖/配置的门禁收据或明确复用依据。补测试提交后在恢复 worktree 跑仓规 `cargo t`+`cargo tv`，逐项归因真实红；本次纯核查未跑这些门禁。 |
| F744-R1-03 | low | review/merge 收尾 | 入场无规范增量/影响元数据解释/最终 review pass。已在 Plan 提出 SD-01/02 与影响路径，仍待最终复审冻结和 merge 沉淀；a2r-std Spec 已有 VM-local/task 寿命约定，不能退回初版进程级连接注册表。 |

## THR-D5 批量回归 2026-10-14（收据 766→774 窗口）

- **REG-2 升格确认（ffi_dual_019 满池竞态）**：本收据窗口内 tf 满池连三轮红
  （6287/6288×3）、定向 1/1 绿（3.5s）——连同前两份收据累计五轮同形复发。
  维持"并行负载序敏感 flake、非语义回归"判定（REG-2 原始双基线实证），
  观察名单不动；若后续收据再发（≥7 轮）应立项查 ffi_dual 测试体的并发
  交互（锁序/临时目录竞争面）。
- **新登记 REG-5（P775 aavm 黄金红，确定性）**：`cargo taa` 两红
  `tests::aavm2_a2r::test_aavm2_goldens_check`（`rust reference run:
  STDASSEMBLY.INVALID_BYTECODE: 60`，定向复跑仍红）与
  `tests::aavm_runner_tests::test_aavm2_compile_corpus`。窗口
  （3ecd375d6..14e387537）内 stdlib_assembly 面唯一改动 = PLAN-775
  b1230e767 新增 `json.encode` 泛型 adapter 规则
  （stdlib_assembly/host.rs ADAPTER_RULES + verify_adapted_reference
  泛型放宽）——黄金语料的 json 调用经新规则发射后 a2r 参考运行报
  bytecode 60 无效。归因 PLAN-775（置信高：唯一触面改动；未做提交级
  二分）。处置：aavm/aa2r 面按 2026-10-02 豁免裁定短期不跑，本红不阻塞
  交付；恢复运行时需先修 adapter 规则的黄金兼容（或按退役裁定一并处置）。
  与 PLAN-744 F744-R1-01/02 同页登记。

## PLAN-770 第四轮最终独立复审（2026-10-14，needs_fix）

受审提交 `92c14fe6cbf1761521fb059bf41f667abeacc214`、revision2；入场/结束原树 clean。详见 [R4报告](attachments/770/review-r4/REVIEW.md) 与冻结 source/HIR/API/本次 raw。R3五项原例闭合有效；以下为原合同内必须关闭项，登记不授权延后/缩围。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P770-R4-01 | P1 | 内层 owner 释放后 view 仍 Checked，两腿读取已释放字节并 RT-FAIL -7/70 | AC03/04/05，T02/04/05/06；块出口生命周期与 backing 验证闭环，静态/存活 owner 正控制 |
| P770-R4-02 | P2 | 短路 RHS 拥有临时语句尾 FREE 的 SSA 值不支配 join，合法输入双腿 backend.lowering | AC02/03/05，T05/06/07；路径相关清理、跳过无分配/无FREE，执行唯一FREE且语句尾/4096不退化 |
| P770-R4-03 | P2 | 终止 return 臂 dead/backing 合入 continuation，合法后续读取误拒 | AC03/04/05，T02/04/06；真实 incoming 状态合流，嵌套/return/break/continue覆盖 |
| P770-R4-04 | P2 | 分支自身 owned 局部被误算跨臂清理义务，动态单/双臂合法误拒 | AC02/03/04/05，T02/04/06；入口 owner 与子块 own_lets 区分，子块出口收口 |
| P770-R4-05 | P2 | wrapper 两个不同 view 形参实际来自同一 owner，内部 mut/view 仍放行 | AC03/04，T02/04/06；借用使用约束跨调用边界，保持独立双来源合法控制 |

本次完整13/13门禁、ac-core227 passed/1 ignored、runtime20 passed、零warning/fmt clean、741/745/767绿；第一次 receipt missing 原文保留，零代码变化复跑绿，根因未作证明。新15source+15HIR及真实Bundle API、历史r2 17+19/r3 5+7原字节本次重放均持久化。状态退executing2/9，保留T01/T03；未修实现、合入、归档或发布canonical/live ledger。下一轮先核验漏测诊断与组合oracle/模型调整。

**r4 返工处置（2026-10-14，work @42b0ba7da+@1f8027500）**：P770-R4-01..05 全部修复并永久化（tests/text_review_r4.rs 17 测试，含 mark 先于 FREE/循环出口 backing/两层 wrapper 传播三个组合 oracle）；r2 18+r3 9 历史重放保持绿，ac-core 244 passed+1 ignored、runtime 20/20、gate 双收据 13/13（gate-20261014-171125/171634）。诊断表与闭合表见计划 §9.11。本节五项待第五轮 fresh review 核验后随计划归档一并清账；未合入。

**第五轮 final review（2026-10-14，needs_fix @1f8027500）**：上述旧例 fresh 重放闭合有效；完整路径不变量仍未闭合，新增两项原合同内阻断如下。报告与可直接重放原始输入见 [review-r5/REVIEW.md](attachments/770/review-r5/REVIEW.md)。fresh gate 13/13、244+20 测试绿不能豁免；登记仅追踪，不授权延后或缩围。T01/T03保留，其余 work-owned 受影响任务重开，2/9，未合入。

| ID | 级别 | 缺口 | owner / 必须闭合的不变量 |
|---|---|---|---|
| P770-R5-01 | P1 blocking | Logical 在 LHS 前记录 RHS 临时边界，跳过侧把已分配 LHS 指针也替成 null；&&/||/嵌套正常返回漏 FREE | T05/06/07，AC03/05；LHS 已执行临时与 RHS 条件临时分开，语句尾唯一释放、mark顺序/4096保持 |
| P770-R5-02 | P1 blocking | 嵌套 break/continue 在 if join 被整体丢弃，loop 没有目标出口状态；Bundle/双腿放行 FREE 后借用读，70/-7 或偶然3 | T02/04/05/06/07，AC03/04/05；正常 continuation 与终止目标状态分开传递/合流，前门拒绝悬空借用，保留存活 owner/return 正控 |

**r5 返工处置（2026-10-14，work @ebd0d05ee+@a14f1470b）**：P770-R5-01/02 全部修复并永久化（tests/text_review_r5.rs 11 测试，含 mark 先于 FREE 与双臂 break/continue 组合 oracle）；r2 18+r3 9+r4 17 历史重放保持绿，ac-core 255 passed+1 ignored、runtime 20/20、gate 双收据 13/13（gate-20261014-175221/175631）。诊断表与闭合表见计划 §9.13。本节两项待第六轮 fresh review 核验后随计划归档一并清账；未合入。

## PLAN-770 第六轮最终独立复审（2026-10-14，needs_fix）

受审提交 `a14f1470b1d5e1174653e8d499091147b4257fe6`、revision2、实施树始终clean；R5原例已闭合，完整不变量未闭合。报告/可重放source/HIR/API/raw见 [R6报告](attachments/770/review-r6/REVIEW.md)。13/13门禁、255+20测试绿不免除此处缺口；主线771已先合入，独立旧树绿不构成联合验收。

| ID | 级别 | 原合同内缺口 | owner / 纠正义务 |
|---|---|---|---|
| P770-R6-01 | P1 blocking | continue/自然back-edge只并入after-loop，无重入读验证；下一轮读前轮已释放view，Bundle与双腿放行、PE70/-7 | AC03/04/05，T02/04/05/06/07；重入状态/来源安全闭合，fresh局部身份与外层存活owner/静态view/4096正控保持 |
| P770-R6-02 | P2 blocking | false不可达臂无条件发布loop_exits，合法空串在循环后被owned-scope-escape误拒；修复回归 | AC03/04/05，T02/04/06/07；目标出口可达性随状态传播，静态合法性检查保持，不取消真实退出义务 |
| P770-R6-03 | P2 blocking（依赖漂移） | 771先合入后未整合，text gate不跑record；SD现行before仍仅core/scalar | AC06，T02/04/05/06/07；同树整合并按774重放两profile实际门禁，更新SD/最终收据，重新独立review |

**r6 返工处置（2026-10-14，work @7e0b9f712+@643a1b3eb+@e01997dc5）**：P770-R6-01/02/03 全部修复并永久化（tests/text_review_r6.rs 9 测试，含重入后 fresh let 陈旧代际组合 oracle）；master（771）merge 进同一实施树并调和共享热点，联合门禁 14/14 双收据（gate-20261014-193430/194300，record gate 实跑）+record gate 独立 24/24+整合套件 321+1；SD-02/04 before/after 已按合 771 后 canonical 刷新。诊断表与闭合表见计划 §9.15。本节三项待第七轮 fresh review 核验后随计划归档一并清账；未合入。

T01/T03保留历史已完成，其他受影响work任务重开，executing2/9，未合入。登记仅追踪，不能延后为merge-owned债务。R5-01原例+本轮嵌套短路控制通过，R5-02仅post-loop原例闭合；旧发现不抹去，完整路径清偿待新模型证据。

## PLAN-770 第七轮最终独立复审（2026-10-14，needs_fix）

受审HEAD `e01997dc593c1d20b3ccab045b33b167eedf4a23`、revision2，已在同树整合771；R6三项原例/本轮受测控制闭合，联合14/14、record24/24(23 cases)、321+20测试绿。详见 [R7报告](attachments/770/review-r7/REVIEW.md)。新修复回归如下；登记仅追踪，不允许延后为merge债务。

| ID | 级别 | 原合同内缺口 | owner / 纠正义务 |
|---|---|---|---|
| P770-R7-01 | P2 blocking | 直接HIR只有break、无自然/continue回边的循环仍把body_st送回header重走，第一条读取被ghost iteration污染而误拒owned-scope-escape | AC02/03/04/05，T02/06/07；区分自然/continue/break/return出口，只真实回边进fixpoint；m01/m02真实Bundle与source控制、原UAF/fresh-view/4096正反控保持 |

**r7 返工处置（2026-10-14，work @314cbbeb6）**：P770-R7-01 修复并永久化（tests/text_review_r7.rs 10 测试，m01/m02/p01/p02 冻结 atom 字节一致；终止/回边矩阵含 return-only 控制）；联合门禁 14/14 双收据（gate-20261014-202410/203110）、整合套件 331+1。诊断与闭合表见计划 §9.17。待第八轮 fresh review 核验后随计划归档一并清账；未合入。

仅T02/T06/T07重开，其他work任务保持已完成，executing4/9，未合入。m01已清理frontend包装的孤立块/条件表达式且重映射引用，最终拒绝仅资源码；m02同图存活owner合法PE0，c05源码+包装HIR合法PE0。不能限制直接HIR只能使用frontend for包装绕开该契约。

## PLAN-770 第八轮最终独立复审（2026-10-14，needs_fix）

受审HEAD `c717555679451b4c109d308d740777e19d7ce2af`、revision2、同树clean。R7直接break原例已闭合，完整真实退出不变量尚未闭合；详见 [R8报告](attachments/770/review-r8/REVIEW.md)。串行联合14/14、record24/24(23 cases)、331+20测试绿不豁免以下原合同内缺口。

| ID | 级别 | 原合同内缺口 | owner / 纠正义务 |
|---|---|---|---|
| P770-R8-01 | P2 blocking | block_terminates仅认两臂语法终止、不采用const_truth；恒真then break/return、恒假else break仍伪造回边，合法首读source/HIR/Bundle误拒owned-scope-escape | AC02/03/04/05，T02/06/07；可达终止与自然出口分类一致，恒定/嵌套/真实back-edge矩阵永久化，保持原直接HIR和UAF/fresh-view控制 |

**r8 返工处置（2026-10-14，work @7468c553e+@645054332）**：P770-R8-01 修复并永久化（tests/text_review_r8.rs 8 测试，含嵌套恒定条件组合矩阵）；联合门禁 14/14 双收据（gate-20261014-230536/231354）、整合套件 339+1。诊断与闭合表见计划 §9.19。待第九轮 fresh review 核验后随计划归档一并清账；未合入。

仅T02/T06/T07重开，executing4/9，未合入；登记仅追踪，不批准缩围/延后。首跑门禁因审查者并行CLI重放造成exe占用os error5，完整原文保留，停止重放后无代码变化串行重跑全绿；不列为实现发现。

## PLAN-770 第九轮最终独立复审（2026-10-15，needs_fix）

受审HEAD `6450543321bdff0e595c0eb61007fd5bfaadf40b`、revision2、同树clean；R8-01常量终止缺口已验证闭合。串行联合14/14、record24/24(23 cases)、339+20测试绿；另有原合同内遗留预扫描缺口，详见 [R9报告](attachments/770/review-r9/REVIEW.md)。

| ID | 级别 | 原合同内缺口 | owner / 纠正义务 |
|---|---|---|---|
| P770-R9-01 | P2 blocking | for true中直接return consume(x.move)，loop外存活x实际只移交一次；进入body前却被block_definite_moves先判死，source/HIR/Bundle首move误拒use-after-move | AC02/03/04/05，T02/06/07；真实入口/回边/目标退出责任代替一律预杀，确定单次移交合法、真实重复拒，caller/callee唯一清理与fresh-view/常量终止/4096保持 |

**r9 返工处置（2026-10-15，work @e60b6eacc+@7a73cf0b6）**：P770-R9-01 修复并永久化（tests/text_review_r9.rs 11 测试，含恒定 break 终止 move 组合矩阵——顺带抓出并修复循环后责任撤销缺失）；联合门禁 14/14 双收据（gate-20261015-003622/004319）、整合套件 350+1。诊断与闭合表见计划 §9.21。待第十轮 fresh review 核验后随计划归档一并清账；未合入。

c05将构造放至loop内、c06去掉loop均真实PE3，唯一LIT/len/FREE；p04真正重复move仍正确拒，不能通过放行重复移动修p03。仅T02/T06/T07重开，executing4/9，未合入。登记仅追踪，不授权缩围/延后；该项非const-aware helper回归，不撤销R8闭合证据。

## PLAN-770 第十轮最终独立复审（2026-10-15，needs_fix）

受审提交 `7a73cf0b68431073181937e3a6a384ca772101a9`，revision2；[R10报告](attachments/770/review-r10/REVIEW.md)。R9-01首次move旧例已闭合，以下为r9退出责任修复新增回归；登记不授权延后/缩围。

| Finding | 级别 | 缺口 | 修复责任 |
|---|---|---|---|
| P770-R10-01 | P1 blocking | AwaitLoop以dead并集撤销entry-owner caller清理，未核验零迭代/移交前break与移交后break的责任一致性；两个source/HIR/真实Bundle通过，正常PE0只LIT无FREE | PLAN-770 T02/06/07、AC03/04/05；同树work核验真实after-loop incoming责任，保留单次move合法/真实重入拒绝，永久化冻结正反控、提交后重绑证据 |

**r10 返工处置（2026-10-15，work @9033cdb39+@afe45e2d6）**：P770-R10-01 修复并永久化（tests/text_review_r10.rs 11 测试，每个合法 run 附 alloc=free 通用断言）；联合门禁 14/14 双收据（gate-20261015-012600/013331）、整合套件 361+1。诊断与闭合表见计划 §9.23。待第十一轮 fresh review 核验后随计划归档一并清账；未合入。

原合同V04 alloc=free/774D04每分配恰一个责任方；R9纠正要求已明确混合出口责任。if-join现有一致性检查可受控拒绝，不要求额外动态清理。联合14/14、350+1/runtime20与record24/23均通过仍不覆盖该缺口。T02/T06/T07重开，executing4/9；未改实现/合入/发布canonical。

## PLAN-770 第十一轮最终独立复审闭合（2026-10-15，pass）

受审提交 `afe45e2d65332b0cd448f3a2b5edf562903296b6`，revision2；[R11报告](attachments/770/review-r11/REVIEW.md)。**P770-R10-01 closed**：两个冻结漏释放反例source/HIR/真实Bundle受控拒绝ownership-required；其余caller/callee合法控制保留唯一FREE。

89 source+95 HIR历史原字节全重放/hash对账，唯一4项行为变化为R10反例两腿恢复正确拒绝；R2..R10受测阻断集闭合。新9 source+11 HIR嵌套/多owner/终止/Drop组合正反控精确trace通过；联合14/14，core361+1/runtime20，record24/24（23 cases），无新增阻断项。既有历史needs_fix记录保留；本段是当前闭合收据。

Plan **reviewed8/9**，T09实际合入/规范沉淀/导航/ledger/index/归档/批回归到期/guard收尾pending；未修改实现或合入。基线/依赖变化须重绑复审。

## PLAN-770 第三轮最终独立复审（2026-10-14，needs_fix）

受审提交 `41291aa10d22eb2e9a263e7aa574361fb55eafd1`、revision2，原树clean；详见 [R3报告](attachments/770/review-r3/REVIEW.md)。r2原例17 source+19 HIR已恢复预期，旧证据不抹去；full-invariant仍有以下合同内阻断，登记不授权延后/缩围。

| ID | 级别 | 当前缺口 | 修复/验收 |
|---|---|---|---|
| P770-R3-01 | P1 | 嵌套跳过move漏FREE、条件Drop双FREE、false臂move后合法读取误拒 | AC03/04/05，T02/04/05/06；按可达出口真实责任join，move/Drop统一，嵌套/终止/fall-through正反控 |
| P770-R3-02 | P2 | 普通借入参数同源mut/view前门漏拒 | AC03/04，T02/04/06；五类来源身份与派生view闭环，保持独立参数合法控制 |
| P770-R3-03 | P2 | 临时FREE早于同完整语句后续mark，违背774 D04 | AC03/05、SD，T05/06/07；完整语句尾寿命且4096帧有界；不能自行改expected/SD |
| P770-R3-04 | P1 | 纯清理义务runtime派生漏检；无requires/capability/sidecar仍build0 | AC01/04/06，T02/05/06；操作/调用/清理统一实际需求并接三门 |
| P770-R3-05 | P1 | 新runtime需求扫描先于结构验证，Eq.lhs=u32::MAX触发索引panic | AC04，T02/06；非法Bundle受控Err、不得panic，确认变异+合法反控永久化 |

本轮完整13/13门禁、ac-core218 passed/1 ignored、runtime20 passed、零warning/fmt clean、741/745/767均绿；五项有新增source/HIR或程序化API反例与raw输出。状态退executing2/9，保留T01/T03；未修代码、合入、归档或canonical/live ledger沉淀。

### Plan 771（2026-10-14，独立 final review R1，needs_fix）

合同内阻断，非允许延期债务；计划回到 executing 1/9，T-02..07 重开。
受审提交 ecd9530d0086d0bdae35ef09cf229f2fb6f91468（lang-771 / plan-771-dev）。
[报告与永久反例](attachments/771/review-r1/REVIEW.md)含原始输入、结果、AC/task、源码定位与重放命令。

| id | 级别 | 缺口 / 修复归属 |
|---|---|---|
| F-771-R1-01 | P1 / blocking | nested record 自赋值与 p=swap(p) copy-elision/out-slot 覆盖存活输入，两 PE exit12；T-04/05/06/07，证明无重叠或用临时再提交 |
| F-771-R1-02 | P1 / blocking | 位置构造 Q(7) 未实现却在计划/SD 声称支持；T-03/05/06/07，实现 declared-order 构造及源码/HIR/PE 证据 |
| F-771-R1-03 | P1 / blocking | Call binding.arg 越界实际 Bundle 变异在 verify.rs:901 panic；T-02/06/07，前置 bounds 与受控诊断 |
| F-771-R1-04 | P2 / blocking | HIR copy 首个失败分支与成功同 exit1，故坏断言仍通过 native test/gate；T-05/06/07，区分退出码与故意失败控制 |

18步门禁全绿（187/187、record_native 9/9、旧741/745/767）不能抵消反例。
修复后同树提交、重放和独立复审；未发布 canonical/ledger、未合入归档清理。

### Plan 771（2026-10-14，独立 final review R2，needs_fix）

修复提交86f3a91a0cabfafede9b9b69579d368bdf3df522；原F-771-R1-02/03/04闭合，
F-R1-01原例闭合但全值语义不变量尚未闭合。22步门禁与199项测试全过不能抵消反例。

| id | 级别 | 缺口 / 修复归属 |
|---|---|---|
| F-771-R2-01 | P1 / blocking | record constructor前一个record值仅保存指针，后一个字段mut调用改源对象后才复制；命名/位置两PE exit11，显式副本control exit0。T04..07；需按源码序捕获record值，区分value与borrow语境，源/直接HIR永久反例与覆盖调整。 |

[报告/冻结输入/hash/重放](attachments/771/review-r2/REVIEW.md)。计划executing3/9，T04..07重开；
两次final non-pass后下次交接先提供不变量诊断与覆盖调整。canonical/ledger未发布，未合入归档清理。

### Plan 771（2026-10-14，独立 final review R3，pass）

绑定e0ad328d80bfe00ca7563174b3352edeb8e6347b / revision2，reviewed8/9。
F-771-R1-01..04与F-771-R2-01全部闭合：旧反例重放/双向源码序值捕获/HIR故意失败控制/借位冲突/新组合PE均通过。
24步门禁/202项测试/15项native/旧741/745/767全绿。[报告与冻结证据](attachments/771/review-r3/REVIEW.md)。

| id | 级别 | 残余项 / owner |
|---|---|---|
| F-771-R3-01 | low / nonblocking 文档债务 | verification.md §4仍留旧187/9数量与耗时，而当前概述202/15；§4c声称旧行已移除。实际复审收据准确，不阻断实现验收。merge closeout修正当前验证表。 |

本pass不覆盖将来的整合代码；770若先合须新基线联跑并fresh review。未合入/发布canonical/归档/清理。

### Plan 781（2026-10-15，阶段 C 实施期发现，work 侧登记）

#### r3 独立复审（2026-10-15）


**2026-10-16 review-r11最新覆盖：final pass**。R08-C根路径遗漏经独立原反例确认关闭，R08-A/B及原短pane全部保持通过；R01..R08/H-781-01验收缺口全部关闭。当前基线lang **3dca8a39c62596b93de9785b9ac940f904983b32** + Reader **7bb1869aef771e8d02b953f25598361a0516b14d**；Rust定向130/130、签名配对1/1、原生锚点57/57、通道9/9通过。复用未变触面r8/r10证据（含Vue59、Reader66、MD1346章/TXT冷启动1347章/W-03），逐项依据见[最终报告](reports/781-final-review-r11-20261016.md)。C01/C02/C03/E02等既有非阻断边界/观察不冒称解决。**reviewed≠已交付**：merge-owned t/tv/tu/tt/th与Reader门禁pending，未合入/发布规范/归档。下方open/needs_fix均为历史状态，不覆盖本轮版本绑定的关闭结论。


**2026-10-16 review-r10最新覆盖**：R08上轮A/B两遗漏独立重放关闭，但 **R08-C P1 / open，T-19/T-23**：新增`!node.path.is_empty()`排除合法根path=[]，根级无controller pane虽已发布vnode键仍漏链，第6行y333.6越过128高viewport却隐式visible=true；显式根pane、可见首行和仅加controller对照正确，稳定帧复查仍失败。须区分合法根身份与不可用路径；非新增布局能力。Rust13/13、原生57/57、通道9/9通过；本轮MD1346章、TXT冷启动1347章、W-03第900章损坏检出/恢复全部通过，不能豁免根pane假阳性。R05/R06/R07保持关闭，详见[review-r10报告](reports/781-final-review-r10-20261016.md)。下方为各轮历史。


**2026-10-16 review-r9最新覆盖**：F-781-R08保持open（P1总体，T-19/T-23），原短pane/V-10/V-13旧例已闭但完整归属未闭。A/P2：tool_visibility只接受非空结构链，有效空链又几何回退，导致真实可见的非pane兄弟col被误裁掉。B/P1：结构函数只接controller键，跳过无controller内pane，外有controller+内无controller的第6行越过内裁剪仍假visible；显式内pane正确clipped_out。真实窗口初始/2.2秒后稳定查询均复现，探针exit1、实例已回收；Rust12项和原通道三例通过不能豁免消费端缺口。R05/R06/R07保持关闭，见[review-r9报告](reports/781-final-review-r9-20261016.md)。下方状态为历史。


**2026-10-16 review-r8最新覆盖**：R07输入引用泄漏经独立原反例确认关闭，R05/R06/H-781-01保持关闭。新增 **F-781-R08 P1 / open，T-19/T-23，AC-09**：resolve_pane_chain用目标与viewport正面积相交作为归属条件，导致被裁掉的子节点丢失pane链并只按窗口判visible；短pane底177，第6行target y363.8仍在窗口内，隐式visible=true，显式ag-pane正确clipped_out。原V-10/V-13稳定红；须分离归属与裁剪，保留D01多pane/标准controller控制，不以改驱动接受no_pane消红。Rust50/50、Reader66/66、VM57/57、Vue59/59和TXT1347章全扫通过不豁免该假阳性。见[review-r8报告](reports/781-final-review-r8-20261016.md)。下方为各轮历史状态。


**2026-10-16 review-r7最新覆盖**：R05数字准入关闭（0.0/-0.5拒绝，Reader66/66）；R06/H-781-01保持关闭。新增**F-781-R07 P2 / open，T-25/T-23**：shim_type_of使用pop_nv消费参数却未释放输入引用份额，100次调用并清栈+reap_all后残留100堆对象或100动态字符串，正常JSON调用对照回到基线；终态断言exit101。须按VM shim所有权约定消费/释放输入并补对象与字符串回归；非E02推测。见[review-r7报告](reports/781-final-review-r7-20261016.md)。下方状态为各轮历史记录。


**2026-10-16 review-r6最新覆盖**：R05旧-0.5、缺字段及字符串数字反例已修；H-781-01关闭；R06保持关闭。R05尚有P2类型合同缺口（T-25/AC-19，连带T-23）：浮点0.0两种字符串均为"0"，num_domain_ok仍接受，与本轮“仅非负VM整数、全部浮点拒绝”声明不一致。独立parse/serialize确认仍为浮点；未证明越界，数值零本身合法，问题限于类型准入/规范一致性。修复或显式论证合同语义修订后再审；见[review-r6报告](reports/781-final-review-r6-20261016.md)。下方各轮状态为历史记录。

**2026-10-16 review-r5当前状态**（覆盖下方实施交接中的全量关闭声明）：R06独立重启与短排水控制通过，关闭；R05缺字段旧例已闭，但负小数字节长度仍假成功，保持open。详见[本轮报告](reports/781-final-review-r5-20261016.md)。

| id | 当前状态 | 具体缺口 |
|---|---|---|
| F-781-R05 | P2 / reopened，T-25 | `> -1`不能证明非负整数；真实索引仅将首章body_len=19改为-0.5，load_meta仍接受，章读返回原标题、空body、word_count=18；正常/第二章/恢复控制均有证据。应验证完整数字消费域，不作单例修补。 |
| H-781-01 | P3 / 健康收口，T-23 | 本轮重启测试plan781_budget_http_tests.rs:449的dir、:481的dir2产生两个新增unused警告；可改underscore前缀以保留TempDir寿命，不能直接用`_`提前drop。 |

**再次复审状态覆盖（2026-10-15，review-r4）**：R04原CPU反例和R05原畸形JSON反例独立重放通过；下方“已修复”为实施交接记录，R05完整缺字段不变量仍open，新增R06修复回归。报告：[781-final-review-r4-20261015.md](reports/781-final-review-r4-20261015.md)。

| id | 当前级别/状态 | 缺口 / 归属 |
|---|---|---|
| F-781-R05 | P2 / reopened | Reader load_meta的typed数字绑定不构成存在性检查；只删除真实meta第一章body_len，仍接受且返回原标题、空body、word_count=18。原解析崩溃修复有效，不能扩大为全部缺字段拒绝。T-25/AC-19。 |
| F-781-R06 | P2 / new修复回归 | SHUTDOWN_DRAIN_CUTOFF是永久进程全局值；同进程关停后新VM/server的ping正常200，但同一100000次有限循环请求从200变为503 request cancelled，受前实例过期截止污染。T-22/AC-11/12/14/17。 |

报告：[781-final-review-r3-20261015.md](reports/781-final-review-r3-20261015.md)。以下为当前合同内阻断项，记债不代表验收完成。

| id | 级别 | 缺口 / 归属 |
|---|---|---|
| F-781-R04 | ~~P1 / reopened，T-22~~ **已修复（2026-10-15 修复轮，lang c3b09e233）** | parked三路测试通过，但连续CPU请求在200ms期限0.219s返回503后，5.973s仍写出late文件（30s期限同计算200正对照）。LegacyHttp循环未响应scope取消；不能用每次心跳park的fixture证明真实导入无迟到副作用。AC-12/14/17。**修复**：引擎驱动环协作式取消（50k 步安全边界探测绑定 scope 终止/全局排水截止）+ 取消 Err→503 映射 + owner 侧幂等 scope 收口 + 桥竞速 try_recv；CPU/park 两类四+三永久测试（th 9/9），反例 replay 绿。 |
| F-781-R05 | ~~P2 / new，T-25~~ **已修复（2026-10-15 修复轮，E01 根修 lang 33a9a6a94 + reader 97ceba9）** | Reader load_meta首尾形状门禁漏内部畸形JSON；`{"version":1,"source_sha256":"x","total_chapters":1,"chapters":[BAD]}`触发raise/catch后的调用帧毒化，chapter_json报str+NoneType（CLI exit1），未诚实返回索引缺失。F-781-E01底层债不能豁免AC-19。**修复**：走 E01 引擎根修（下条）+ load_meta 条目级五字段探测（缺任一→受控"索引缺失"，原生速）；反例回放绿（corrupt→受控/restore→恢复/exit 0）；错类型标量=动态类型边界成文（SD-11）。 |

| id | 级别 | 缺口 / 归属 |
|---|---|---|
| F-781-C01 | P2 / 新计划候选 | VM HTTP 段派发泵串行——单段无 park 的长忙 handler（10MiB 导入 70-230s）占住泵，共居 split-VM 前端的内部 HTTP（Init `/api/books`）饿死（实测 2/2 负载态 + 1/3 安静态）。781-budget-decision r3（D-1 预见的公平性证据坐实）；修复面 = 段泵并发化或 HTTP CpuSlice 化——D-1 冻结为 v1 非目标，需新计划。 |
| F-781-C02 | P3 / 路由 READER-001 | Reader 前端把 Init 内部调用失败当致命 → 进程退出 → 进行中导入连接中断（consumer 韧性：Init 重试/降级不自杀）。 |
| F-781-C03 | P3 / 预存（非 781 引入） | th 档 `http_e2e_back_proxy_real_routes_corpora_data_face` 在 776/770/771 合入后的 master 基线即红（`:id` 参数模板转发）——stash 基线同红实证；归属近次合入面，781 记录在案未修。 |
| F-781-E01 | ~~P2 / 待用户定路由（新计划候选）~~ **已修复（2026-10-15，781 R05 修复轮，lang 33a9a6a94）** | **VM 跨帧 raise 毒化**（781 阶段 E T-25 实施期发现）：被调函数内部 raise（即使被 catch）返回后，调用方帧的参数/局部槽位损坏为 None——最小复现 `fn maybe_raise(){try{json.parse("{bad")}catch{}}; fn caller(n int){maybe_raise(); return "n="+n.to_string()}` 崩 str+NoneType。**根因**：intercept_error 跨帧弹帧条件 `old_bp >= handler.bp` 把 try 帧自身弹掉（CallFrame.old_bp 全仓语义=帧自身 bp，五压帧点均在 bp 赋值后捕获）→ 同帧 catch 后 RET 弹空 → current_fn_n_args 不恢复 → 调用方 LOAD_LOCAL 参数臂（bp-n_args+param_idx-1）走 null 守卫。**修复**：严格 `>` + 最外被弹帧 old_* 恢复 n_args/n_locals/num_locals；永久回归 `plan781_e01_cross_frame_raise_catch_restores_caller_frame`（同帧 native raise/跨两层 opcode raise/catch 带参）。books_json 等既有 try/catch 消费面同步解雷；tv 163/163 + 帧/catch 族 34/34。 |
| F-781-E02 | P3 / 观察登记（未根因） | **循环上下文 find/池槽混叠**（781 r4 修复轮 t07 S11 实施期发现）：while 循环内 `json.parse`→标题变量→`find(needle)` 断言形态中，标题变量槽被观测读作 check 消息字符串内容（消息自嵌套）；顶层同形断言（S4/S9）稳定通过，最小复现不稳定（直接/嵌套 fn 调用 + 循环各形单独均复现不出）。S11 已改全串相等断言绕开（reader dcd6ca4）；E01 修复（33a9a6a94）不覆盖此面。疑似字符串池/槽分配在循环域的别名缺陷——需独立根因调查。 |

### Plan 782（2026-10-15，T-03 探针发现的基线预存缺陷，work 侧登记）

非 782 引入（master 合入基线即有），782 组合程序已规避，不转交本计划修复；与 §10.6 对应。

| id | 级别 | 缺口 / 归属 |
|---|---|---|
| D-771-A | P3 / 771 交付面 | 命名构造字段值位不支持二元运算表达式：`Pair { left: 1 + 2, ... }` 在纯 record 域报 `source.syntax: expected operator`（771 交付即如此）。需独立修复并回归 771 面；782 组合程序以 let 提升规避。 |
| D-770-B | P3 / 770 前门缺口 | text 域 assign 位 `.move`（`o = x.move`）产生 `source.internal`（adapter 发射 text descriptor 不接受的 `expr.move`）而非受控拒绝。770 该位本就不支持 move 标记，缺前门拒绝；782 组合域 assign 位 move 走 descriptor 接受的包装表示不受影响。 |


### Plan 782 final R1（2026-10-15；合同内阻断，owner=782 work）

受审 `b025a85aa38197700763b944d1b7fec2ecbc4ad0`；证据、原文与合法对照见 [R1 报告](attachments/782/review-r1/REVIEW.md)。下列登记不构成 scope 外延后批准；计划已退回 executing=2/9。

| Finding | 严重度 / AC | 缺口与关闭条件 |
|---|---|---|
| F-782-R1-01 | P1 / AC02/03 | 嵌套资源 FieldGet 在 let/return/ctor 隐式位复制，实际 FREE-INVALID。补完整拥有来源门和 clone/copyable 对照，前门拒绝隐式共享 owner。 |
| F-782-R1-02 | P1 / AC03 | `a = consume(a.move)` 释放同语句已移出的旧 buffer，读取返回值 trap70；按 RHS move 后责任生成 replace_frees，保全返回 owner。 |
| F-782-R1-03 | P1 / AC04 | pending_aggs 未随短路路径合流，循环 take→skip 重复 free；聚合临时按实际创建路径清理并补动态循环控制。 |
| F-782-R1-04 | P2 / AC04 | 文本与聚合局部/临时分开清理，跨类型创建顺序 LIFO 失效；补统一顺序及两个方向精确 trace。 |
| F-782-R1-05 | P1 / AC03/05 | 实际 Bundle Move.inner=u32::MAX 在 verify.rs:936 数组越界 panic；语义前结构界检并补永久 mutation；纠正现有 V08 取证映射。 |
| F-782-R1-06 | P2 / AC04/06 | mandatory V07 未接实际组合故障注入；Rust 模拟 P6/计划形状/runtime 自测不足。核对 status helper/fatal 实际路径，补第N申请/账本/预算/零长 oracle，并纠正已完成声明。 |

### Plan 782 final R2（2026-10-15；合同内阻断，owner=782 work）

受审 `a52a54472728755027078fe712a1de6d2002f905`；[R2报告与可重放反例](attachments/782/review-r2/REVIEW.md)。R1 原文已闭合，F03完整不变式仍未闭合；以下登记不构成延期批准。

| ID | 等级/验收 | 缺口 |
|---|---|---|
| F-782-R2-01 | P1 / AC04 | 外层 LogicalRhs 将内层 Agg.present 覆盖1；嵌套 &&/|| take→skip loop FREE-INVALID；须保留组合存在性并补真实内层跳过控制。 |
| F-782-R2-02 | P1 / AC03/04 | move 整根后仅字段赋值却恢复整个根 live；Str 字段悬空读取 trap70、嵌套record重复free；须拒绝部分重建，同时保持完整根重建/存活字段替换合法。 |

### Plan 782 final R3 闭合（2026-10-15）

受审 `17508b703c84e6f24496ede7b908621a1c05f365`；[R3完整报告](attachments/782/review-r3/REVIEW.md)。F-782-R1-01..06、F-782-R2-01/02 **全部关闭**；保留上方历史记录。90定向测试、22原文控制、9真实API控制与15新增组合控制复验通过；嵌套聚合存在标记与join后回滚、三字段终型×两语句期moved根均有实际oracle。代码/验收/Spec final pass，merge-owned联合收口/归档/清理仍pending。D-771-A/B/C既有独立登记不受此闭合影响。

### Plan 783 探针边界与后续阶段候选（2026-10-15；work 阶段登记，非验收缺口）

以下为 PLAN-783 探针的**已声明边界**与生产化差距（feasibility.md §4 / perf-report.md / spec-delta SD-04），供后续计划取材；不构成本计划的延期批准。

| ID | 等级 | 内容 |
|---|---|---|
| D-783-A | P2 | GC 与查询并发的尾延迟可见（R1 修复后 30 样本：P99 667ms vs 同进程基线 509ms；GC 删除已实证 77 块/22 趟，归因含 liveness 互锁排队+删除 I/O 交错）。生产化需 GC 限速/有界批次让路（Design40 §7.2），探针未实现调度。 |
| D-783-B | P2 | 对象命中路径由 rust-lld 子进程链接支配（~210ms）：增量链接/对象缓存直出 PE 的增量发布（A10 拆阶段）未做。 |
| D-783-C | P1（生产差距） | auto-cache `BlobStore::put` 固定 tmp 名：结构性风险（R1 F-08 反例）；探针以 per-chunk LockFileEx 有界等待（10s）串行化规避并实测重叠双进程同键行为——探针证据不构成生产 BlobStore 原生并发安全的证明；生产修复/上移未做（SD-04 差距清单）。 |
| D-783-D | P2 | 无 pack/compaction：空间归还以 blocks 层字节为证（恢复趟已清 trash，R1 F-06 修复）；SQLite 主文件不收缩（freelist 复用，高水位为声明行为）；NTFS 簇级 slack 未计量（已声明局限）。 |
| D-783-E | P2 | A03 布局人口为确定性造数：证明分页/预算/共享机制，不冒充通用 CST；真实 grammar-driven CST 归 Design39 线后续计划。 |

### Plan 783 final R1 阻断（2026-10-15）

受审 `62d93dae82546e5685f164382345ad4050690fa2`，结论 **needs_fix**；[完整报告与复现](attachments/783/review-r1.md)。以下均为既有 r1 合同内缺口，不能以债务登记替代修复。T-01..07 重开；历史工作收据保留，非当前通过证明。

| ID | 等级 | 未闭合事项 |
|---|---|---|
| F-783-R1-01 | P1 | GC 快照后新 lease+hard root 取得成功，但仍删除产物；实测 lease_live=true/readable=false。 |
| F-783-R1-02 | P1 | perf --root 无 marker 检查即递归删除目录；受控外部哨兵丢失。 |
| F-783-R1-03 | P1 | GC 留 deleted chunk 行，重发布 INSERT OR IGNORE 不复活；put成功/get miss。 |
| F-783-R1-04 | P1 | CAS 读只验自校验不验请求 ChunkId；合法异内容替换被接受。 |
| F-783-R1-05 | P1 | LayoutView 绕过框架/身份门；损坏页返回值，非法偏移触发 panic。 |
| F-783-R1-06 | P2 | conflict 的无 artifact chunk 行永不入回收；recover 不处理 trash。D-783-D 的 recover 释放说法待修正。 |
| F-783-R1-07 | P1 | churn 仅7种源码，gc_runs 是调用次数；perf 未证实际删除，资源计量及尾延迟归因不成立。D-783-A 删除 I/O 归因待重证。 |
| F-783-R1-08 | P2 | 同 chunk 竞争立即 Internal 失败，未有串行/重试协议；所谓跨进程并发测试实际顺序执行。D-783-C 探针串行零复现依据待修正。 |
| F-783-R1-09 | P1 | 聚合器接纳 ignored/无摘要；acceptance 为66 passed+1 ignored，非76/0，perf 合同无env直接返回。 |
| F-783-R1-10 | P2 | probe cargo fmt --check失败；依赖shim warning属未改基线。 |
| F-783-R1-11 | P2 | 诊断render使用空path/source，丢当前工程来源绑定。 |

### Plan 783 R1 修复轮（2026-10-15；work 同树修复,受审基线待 R2 重绑）

**最新R8（2026-10-16，文档受影响复审）**：受审 `46dc5f442851bbcc6d7e7c647d21b0966f1f5a64`，**pass**；[R8报告](attachments/783/review-r8.md)。F-783-R7-01关闭：首行恢复、六场景/GC计数/正文与冻结JSON完整读回及独立逐值校验一致。代码/依赖/脚本/JSON未变，复用R7的93+1白名单验收；未重跑512/30。Plan reviewed8/9，merge收尾的可复用门禁收据核验、文档落地、归档/guard清理pending。下方R7及此前状态为历史快照；D-783-A..E探针外后续候选保留。

**最新R7（2026-10-16，落地后受影响复审）**：受审 `c00f77b526be85e758afaf2187f316eaa01559d8`，**needs_fix**；[R7报告](attachments/783/review-r7.md)。RuntimeInjection适配与18族93项验收通过，R6历史闭合保留。仅F-783-R7-01(P2，AC-05/06，T-06/07)：closeout再生人读报告表头串入旧表格，正文/GC计数与冻结JSON混轮（GC-P99旧774vs当前503等）。需原树docs-only纠错后文档复审，不要求为文案重跑512/30。已落地事实保留，归档清理pending；executing5/9。下方R6及更早状态为历史快照，非当前最终结论。

**最新R6（2026-10-16）**：受审 `aa8c666caa793891c9085efa3f8675ef8f6fe7d1`，final **pass**；[R6复审报告](attachments/783/review-r6.md)。R5三项全部关闭：失败/清理受阻残留消耗配额及恢复后再写控制通过；人读报告与JSON/预留指标一致；新增warning清零（shim旧基线单列）。18族93 passed+1白名单ignored，fmt/diff绿。R1–R5下方开放状态为历史快照，不再表示当前缺口；D-783-A..E实验外后续候选保留。Plan reviewed 8/9，merge-owned完整closeout512+30及知识沉淀/归档合入/guard清理pending，未宣称交付收尾完成。

**最新R5（2026-10-16）**：受审 `4ca6f3a0f6c2415d693a887a6e19b3a760fc128d`，final **needs_fix**；[R5报告/逐项证据](attachments/783/review-r5.md)。91项正常验收通过；R4孤儿回收与报告发布失败传播关闭，普通并发预算原例关闭但完整失败出口未闭。以下仍属合同内修复事项，非延期债务；下方“最新R4/R3”等为历史快照。

| ID | 等级 | R5当前缺口 |
|---|---|---|
| F-783-R5-01 | P1 | CAS rename重试耗尽留下3MiB.tmp，后续预算仅扫staging漏计；失败→再写实测总临时9,437,328>8MiB。F-R4-02完整不变式仍未闭。 |
| F-783-R5-02 | P2 | perf-report GC表格342/509/544与当前JSON227/382/419不符；restart比较方向及预留峰值/实际staging语义未同步。 |
| F-783-R5-03 | P2 | 本轮新增gc.rs:307 unused total_candidates与review_r4_.rs:17 unused Write，health未过。 |

**最新R4（2026-10-16）**：受审 `bb78975db1feff8a5b624e2be7dbcd24739beb16`，final **needs_fix**；[R4报告/闭合表/反例](attachments/783/review-r4.md)。R3主要原例已闭、acceptance88+1白名单ignored绿、无探针新warning；下方R1/R2/R3为历史记录。以下为既有合同内剩余缺口，不能以债务登记替代修复。

| ID | 等级 | R4当前缺口 |
|---|---|---|
| F-783-R4-01 | P1 | GC known集合包含全部状态行，遗漏deleted行+重现CAS及live无artifact块；前者显式recover也不收，后者GC不收。无pin却持续超low。 |
| F-783-R4-02 | P1 | 独占锁仅保staging，CAS.tmp副本未预留；2×3MiB在os32退避期间额外temp合计12,583,104>8MiB。PEAK还把拒绝申请9,437,328称作实际6,291,552驻留。 |
| F-783-R4-03 | P1 | 报告校验成功后Copy-Item目标被占用，IOException未进入failures，closeout最终exit0/PASS且新证据未发布。 |

**最新R3（2026-10-15）**：受审 `e04de7b960e2a6c13ea39a16f42532c23a325ee4`，final **needs_fix**；[R3完整报告/闭合表/反例](attachments/783/review-r3.md)。下方R1/R2表为历史记录：R2原文反例与84项正常验收本轮通过，但完整不变式未闭。以下为合同内修复事项，不构成延期或预算变更批准。

| ID | 等级 | R3当前缺口 |
|---|---|---|
| F-783-R3-01 | P1 | shared liveness不互斥预算预留；4×3MiB实际staging12,583,104>8MiB，4成功且peak仅报3,145,776。 |
| F-783-R3-02 | P1 | GC用DB live行替代物理blocks，crash orphan漏计；真实total3,235,888>low524,288却backoff=None。 |
| F-783-R3-03 | P1 | recover有namespace门但GC没有；staging根junction下旧目录被递归删除，外部自建哨兵丢失。 |
| F-783-R3-04 | P2 | closeout正确exit1拒绝非法perf，但仍无条件覆盖旧报告；失败保留/校验后发布未闭。 |
| F-783-R3-05 | P2 | SD仍写DB不收缩/旧发布协议，perf-report仍旧22趟77块与旧延迟，未跟新JSON及代码同步；缺完整before/after规范增量。 |
| F-783-R3-06 | P2 | 新layout:569、node:193两处unused_mut；非shim基线，health gate未过。 |

**R2 裁定：下文为 work 历史声明，不能作为11项完整闭合证明。** 受审 `90a35f6841f37b40df668275c4ccc338b2233630`，final **needs_fix**；[R2报告/逐项闭合表/原始反例](attachments/783/review-r2.md)。R1 F-04/10/11关闭，其它旧例修复保留；以下合同内缺口不得降级为未来债务。

| ID | 等级 | 当前未闭合事项 |
|---|---|---|
| F-783-R2-01 | P1 | CAS投放先于shared liveness；recover(false)可删在途复用块，随后put成功/get miss。 |
| F-783-R2-02 | P1 | B header常量13/实际15，13/14/21/22字节记录仍panic；A偏移2^48截断后被接受。 |
| F-783-R2-03 | P1 | 同进程staging仅毫秒+PID，4线程成功发布后3份不可读；acceptance及定向重放reader/GC均os32失败。 |
| F-783-R2-04 | P1 | 新trash恢复只检子项reparse，不检namespace根junction；外部自建哨兵被删除。 |
| F-783-R2-05 | P1 | 512总量低水位断言降为高水位+手动全扫；原r1未获修订。快控final4,350,603>low600,000仍判收敛；峰值/资源/overlap证据待补。D-783-A/D不能据此豁免合同。 |
| F-783-R2-06 | P1 | closeout显式512仅验exit：0例churn与无写入perf模拟仍返回PASS，普通acceptance硬化不能代替merge载荷校验。 |

R1 全部 11 项 finding 已修复并以 `tests/review_r1_.rs` 12 项永久回归钉面(逐条镜像反例):F-01 liveness 互锁(发布/注册共享锁 vs GC/恢复独占锁+注册时存活校验);F-02 perf 改为独占 marker 子目录、绝不删给定目录;F-03 chunk 状态 upsert 回 live+级联删行;F-04 读取时 CAS 地址复验(HIR/object 双消费者);F-05 布局读取路径完整准入(框架+地址+字段界,未知布局/短记录/越界偏移受控拒绝);F-06 恢复覆盖已提交孤儿行/trash 残留/tombstone 级联;F-07 churn 512 真异版本+判别 oracle(越限/真删/收敛),perf GC 场景 22 趟/77 块真实删除/0 失败;F-08 有界等待(10s)+失败清理 staging+真实重叠双进程同键竞争;F-09 聚合器清单化(在场/最小通过数/ignored 白名单+已提交 perf 合同实证校验);F-10 cargo fmt 全绿;F-11 诊断绑定当前工程源路径。下列候选条目已按修复后事实制正。闭合由 R2 独立复审确认,本节不构成最终通过声明。

### Plan 785（2026-10-16，T-01/T-05 实施期发现的基线预存缺陷，work 侧登记）

非 785 引入（master@340ab95fd 基线即有）；785 已按登记边界规避（fixture UI 形态改走 workspace 锁定 rlib 重放、触面文件单独 rustfmt），不转交本计划修复。

| id | 级别 | 缺口 / 归属 |
|---|---|---|
| F-785-P01 | P3 / 预存（779 交付面腐烂） | `{ui}` 与 `{ui, ui-clipboard}`（无 iced）feature 组合在当前 master 均不可建（8 错：`ui::iced/session/dynamic` 缺失 + `log::set_boxed_logger` feature 缺）——779 fixture 文档化的 UI 复跑命令 `--features auto-lang/ui` 已烂（可建 UI 形态仅剩 ui-iced 全家桶）。另：779/785 fixture 独立 workspace 解析新 UI 锁时 wgpu-hal 版本漂移编译失败（779 R3 同款），UI 外部证明只能走 workspace 锁定 rlib 直编（覆盖边界=锁身份随主 workspace，不构成新 UI 锁发行验证）。归属：feature 图整备/跨仓发行依赖身份验证，需独立计划。 |
| F-785-P02 | P3 / 预存（仓级 fmt 卫生） | `crates/auto-lang/examples/ui_container.rs:195` 存在未闭合双引号（E0765）使全仓 `cargo fmt -p auto-lang` 直接失败；且仓内大量文件存在 fmt 漂移（本会话 rustfmt 递归波及 ~48 文件后已全部回滚，仅保留 785 触面文件格式化）。归属：独立 fmt 整备清理（修复坏 example + 全仓一次性格式化落单独提交）。 |

### Plan 785 final review R1（2026-10-16，needs_fix）

受审 64c72d20c2e081b563760c1c4fe38761ccc6d00b；以下为合同内返工项，不批准延期。[完整报告](attachments/785/review-r1/review.md)。

| id | 级别 | 缺口 / 归属 |
|---|---|---|
| F-785-R1-01 | P2 | 新 adapter 测试及 bridge 构造测试与 plan785 默认槽测试未共锁；6项 libtest 并行第2次重放在 :55 “缺席=无 provider”失败。须统一相关默认状态测试的隔离并保留恢复，不能仅依赖 nextest 进程隔离。 |
| F-785-R1-02 | P2 | fixture tests/clipboard_vm.rs:498 新 unused shim_clipboard_text 导入，no-ui/UI 编译均警告；删除并验证两形态。 |

### Plan 785 final review R2（2026-10-16，needs_fix）

受审0ac6e2ff5ced743974ce274bb0fa8cf424c14453；[R2完整报告及闭合表](attachments/785/review-r2/review.md)。F-785-R1-02已闭合；F-785-R1-01原6项反例30/30通过，但完整不变式仍未闭：普通plan320 VM高层测试经run_with_capture→installer间接补默认槽，不受新增共享锁约束。20项轻量组合第14轮复现plan785缺席断言:55失败（19/20）。本条为原合同内剩余缺口，不批准延期；T-05/06重开。下次work交接应附连续两轮失败的诊断与完整隔离/永久组合反例调整，不能只扩展具名provider测试锁域。

### Plan 785 final review R3（2026-10-16，pass）

受审c49ca102abb198006ef1737a87522e37292a2c99；[R3报告](attachments/785/review-r3/review.md)。F-785-R1-01完整关闭：共享libtest缺席锚改构造式，公开入口全局缺席合同仍由独立进程no-ui fixture验证；R2混合反例21项×30轮全绿。F-785-R1-02保持关闭。此前R1/R2条目保留为历史，均非当前未解债务。预存F-785-P01/P02及新独立UI锁边界仍保留；merge-owned裸cargo t+tv及收尾尚pending，不宣称已交付。


### Plan 789 final review R1（2026-10-16，needs_fix）

受审 `1c916823376f748cd04a2f736e86f9fbe7f10624`；[完整报告](attachments/789/review-r1/review.md)。以下为原合同内缺口，必须在同一实施树修复后复审，不批准延期；T-05/06重开。

| id | 级别 | 缺口 / 修复要求 |
|---|---|---|
| F-789-R1-01 | P1 | V02测试读取的 `crates/auto-lex/tests/fixtures/v02-golden.json` 被`.gitignore:161 *.json`忽略，HEAD无该blob。本机绿依赖忽略资产，干净检出失败；将旧臂冻结件字节副本纳入提交并验证hash及提交内可读性。 |
| F-789-R1-02 | P2 | 合同T05严格边界门实际fail：config_drift_since_baseline + stale_baseline；不能用observe零违例/metadata一致替代strict。核对现有基线后按777规范刷新，严格正控及禁向负控重放，不加宽豁免。 |
| F-789-R1-03 | P2 | 新lex-consumer在正常cargo test中出现unused imports(LexError/LexerError)及dead_code(tokenize_all/tokenize_all_rest)共3警告；按测试职责隔离并零新增警告复验。 |

修复落点（2026-10-16 work 轮，待 R2 复核闭合判定）：F-01/F-03 提交 `7a76aef57`（golden `git add -f` 入库 blob `a09fa2c`、期望未重生、SHA256 与冻结件一致；consumer 测试代码收 `#[cfg(test)]`，复验零警告 4/4）；F-02 提交 `f3726816d`（锚定路径 7d51845ff..刷新点逐项归因：根 manifest=785+789、auto-lang manifest=781+789、boundaries 配置与 auto-lex manifest=纯 789，均为已复审合入的合法增量；clean 树 `--refresh-baseline` 后 strict 正控 pass 0 violations、负控 `--self-test` 15/15，证据 attachments/789/review-r2/）。

### Plan 791 final review R1（2026-10-16，needs_fix）

受审 `0cd3e3e9f7e448fc95a1ef61b13761ac02d7e65e`；[R1报告](attachments/791/review-r1/review.md)。定向47项及外部consumer4项通过、生产真实委托core与净删成立。以下为合同内返工，不能作为批准延期：F-791-R1-01(P2，AC-02/05，T-01/05/06)：V02/V03没有非空lib_files/lib_paths、use库或缺失库加载顶层错误的旧/新对照；F-791-R1-02(P2，AC-05，T-05/06)：新common helper产生6条dead_code warning、两个crate fmt失败及Cargo.toml EOF空行diff-check失败。计划回executing 3/8，merge-owned门禁pending。

预存后续债候选（本次提取不要求修行为）：D-791-01 AutoFile/AutoCode变量不可见及lib_files作用域错配（库项仍须F-01补实测）；D-791-02 shared-interpreter渲染不消费fstr_note/lib_paths（template.rs仍TODO，具体use行为待F-01冻结）；D-791-03 auto-man exporter/builder与asset模板仍未迁离语言栈。既有autogen_tests.md bool期望已在测试冻结提交对齐true，不再列为开放实现缺陷。这些记录不覆盖或豁免上述两项返工。


F-789-P01 / P3（预存资源缺口，非789新回归）：扩大widget定向筛选遇到`plan051_ext_widget_markdown_renders_from_vm_adapter`于:56因`corpus app.at`缺失失败（123pass/1fail/1ignored）；`test/ui/plan051_ext_widget/src/front/app.at`在受审树不存在、基面e4ac72a62的git tree也无该blob，测试源码本次未变，未进入词法/解析。本计划Parser/桥面定向验收已通过；后续资源补齐/测试基建负责，不能宣称宽筛选全绿。见789 R1报告及原始targeted日志。


### Plan 789 final review R2（2026-10-16，pass）

受审`f3726816db18d38996e3f80ae496d33719edc5e5`；[R2完整报告](attachments/789/review-r2/review.md)。F-789-R1-01/02/03全部关闭：golden受管blob与旧臂冻结件SHA256一致且34例重放通过；严格边界门pass零违例、metadata41边一致及15/15正负控；consumer4/4零警告。R1条目保留历史，不再为开放实现缺口。F-789-P01旧plan051资源缺口继续保留。Plan=reviewed 7/8；merge-owned裸cargo t+tv及规范沉淀/归档清理仍pending，未宣称交付完成。

### Plan 791 final review R2（2026-10-16，needs_fix）

受审 `b54cc4a2a1b95631c9e4726985615d66ba116c7e`；[R2报告](attachments/791/review-r2/review.md)。R1两项均闭合：新增库路径三臂+缺库哨兵覆盖，50项定向/consumer4项通过，fmt/diff-check通过、无新增crate警告。新增F-791-R2-01(P2，AC-04/05，T-05/06)：R1修复删除auto-gen/Cargo.toml末尾空行后未刷新777基线，strict实跑exit2（唯一stale_baseline，grandfathered2）。须原树核对锚定diff并按规约显式刷新、重跑strict后提交，禁止以闭包/observe替代。合同内返工不批准延期；executing 4/8，merge-owned门禁pending。D-791-01/02库预存行为已由本轮fixture补实测，其未来行为修复债与本次元数据缺口分列。

### Plan 787 复审 R1 修复轮（2026-10-16，findings 全闭合送审）

受审修复提交 `afbacfbcc644d246a2bf3a0ae6185bbd9e6ca457`（R1 报告
attachments/787/review-r1/）。F787-R1-01..10 全部闭合：反例重放 7/7（修复前
1/6）、acceptance 16 族 90 项 ALL PASS、783 触面 --lib/admission_/dep_audit_/
identity_/concurrency_/review_r* 全绿。修复轮提取的预存后续债候选（本次不
要求修行为，均非阻断送审）：

- D-787-01 probe put 提交成本随块数超线性（staging 预算锁每次全目录扫描
  CAS tmp）：64MiB 负载 4700 次 put 致构建 204.7s 主要耗于此。783 既定边界
  （GC/提交协议冻结），787 已以源页 256KiB 聚合缓解（4096→256 put）；
  未来大负载语法产品化前需 783 侧批量提交/增量扫描方案。
- D-787-02 debug 构建下增量编辑时延不优于 fresh（227ms vs 200ms，264KiB）：
  增量收益本轮以重扫字节/重解析声明数/内容块复用率为证据面，时延优势
  非主张；release 档复测留待后续语言扩面决策（matrix.md §2/§5 如实声明）。

### Plan 791 final review R3（2026-10-16，pass）

受审 `561a61fc694f6f1a0a493300bf6385c8e11f96dd`；[R3报告](attachments/791/review-r3/review.md)。F-791-R2-01关闭：基线唯一变化为base_revision重绑，41声明边/规则与2项grandfather完全不变，新冻结头strict独立实跑pass/0 violations/2 grandfathered。R1两项保持关闭；实现/fixtures/测试配置/依赖锁未变，明确复用R2独立50项+consumer4项及health证据。此前R1/R2开放结论均为历史，当前无合同内未解finding；D-791-01..03预存行为/未迁债仍保留。计划reviewed 7/8；merge-owned裸cargo t+定向全测及规范/ledger/归档/guard清理pending，未宣称交付完成。

### Plan 787 复审 R2 修复轮（2026-10-16，findings 全闭合送审）

受审修复提交 `f7ea3a9e93bb76e0187c10fa280f032baa0fbcbd`（R2 报告
attachments/787/review-r2/）。F787-R2-01..11 全部闭合：R2 反例 18/18（修复前
7/11）、R1 反例 7/7 保持、acceptance 16 族 95 项 ALL PASS。两连败诊断已入
计划 §9（根因=修复未到不变量级+oracle 强度不足）。预存后续债候选更新：
D-787-01（probe put 提交成本超线性）维持——R2 轮目录页加密到 4096 条/页后
put 次数进一步上升（64MiB 构建 366s，仍为 probe 边界）；D-787-02（debug 档
增量时延非主张面）维持。

### Plan 792 final review R1（2026-10-16，needs_fix）

受审 `8a83e1f39383b43c01b5b159dacef8d2a89fd7c2`；[独立报告/反例/AC/SD冻结](attachments/792/review-r1.md)。F-792-R1-01（P2）：资源校验只在driver比较原fixture，清单hash改零仍True，真实locked构建缺图仍exit0；F-792-R1-02（P1）：再交付准备失败前删除旧app/锁/资源，CLI红时旧产物已丢；F-792-R1-03（P2）：sidecar可成功发行主检出绝对path，破坏独立来源/重定位；F-792-R1-04（P3）：新增rust_ui两处fmt差异。均为合同内开放项，登记不批准延期；Plan回executing 2/8，保留T01/02，重开T03..06。独立fresh build161s、relocated locked131s、delivery单测5/5；首次driver12/14因review scratch在仓内使V04条件不符，按合同仓外独立补跑V04两项全过。新反例仍红；merge-owned全档/规范/ledger/归档/guard均pending，未改实现/合入。

### Plan 787 复审 R3 修复轮（2026-10-16，findings 全闭合送审）

受审修复提交 `15cfeac97`（R3 报告 attachments/787/review-r3/）。F787-R3-01..06
全部闭合：R3 反例 8/8（修复前 1/7）、R2 18/18 + R1 7/7 保持、acceptance
16 族 98 项 ALL PASS。三连败诊断已按轮留档计划 §9（R3 根因=R2 调整仍旧例
级）。D-787-01（probe put 成本超线性）维持——目录页 2048 条后 64MiB 构建
548s；D-787-02 维持。

### Plan 792 final review R2（2026-10-16，needs_fix）

受审 `477e498199ed6302a1e9724e89cbb8d2625bb65b`；[R2报告/闭合矩阵/反例/SD](attachments/792/review-r2.md)。R1-F01正式资源校验与F04格式CLOSED；F02/03原反例已修，完整不变量仍失败：F-792-R2-01（P1，修复回归）固定同级 .<name>.delivery-staging/.delivery-old 无归属检查即递归删除，两个外来sentinel丢失、CLI均exit0；F-792-R2-02（P2，边界未闭）Windows根相对 /autostack/... 绕过两层is_absolute/深度检查，成功发行指向主检出的path；metadata仍是子串包含。资源独立7控、delivery7/7通过，旧包预检失败/生成失败/换出失败均保留。Plan=executing 2/8，T03..06重开；第二次final失败，下一次work交接须根因与覆盖/设计调整表。根Cargo.lock实际gitignore，历史“已提交锁”叙述纠正并冻结实际锁快照/hash。两项为合同内开放项，登记不批准延期。merge-owned全档及收尾pending，未改实现/合入。

### Plan 792 final review R3（2026-10-16，needs_fix）

受审 `dc645528f506f7887be8e4428ade53486cd9b777`；[R3报告/闭合/正式反例](attachments/792/review-r3.md)。根因/覆盖表已核，R2十组旧例按预期通过，R1资源/格式保持CLOSED；完整约束仍失败：F-792-R3-01（P1）备份只看通用manifest存在，CLI成功却删掉另一应用unrelated_app的合法交付包；F-792-R3-02（P2）边界只匹配固定文本path空格/双引号，现行sidecar原始TOML表及源manifest的紧凑单引号path均成功发行，真实cargo metadata确认本地源在包外。两项均延续原合同约束，登记不批准延期或缩小要求。driver23/23（CLI1+验收22）、delivery8/8、资源7控；fresh真实build117s、重定位locked121s。Plan回executing 2/8，T03..06重开；下轮覆盖表须治理操作归属与TOML语义，未修实现/合入，merge-owned全档及收尾pending。

### Plan 792 final review R4（2026-10-16，needs_fix）

受审 `33d45f70879ed5066ffbe9198ba59c83a5fac0f1`；[R4报告/旧例闭合/新增GC反例](attachments/792/review-r4.md)。R3他应用交付与两TOML入口均CLOSED，旧13条语义观察与资源7控按预期；新增F-792-R4-01（P1，修复回归）：生成前GC只看换出标记+清单，未证明已有新包成功替代。正式旧包处于换出后、换入前状态（根缺失），重试先GC删唯一旧备份，再因缺sidecar模块exit1，最终新旧包均不存在。不是新增自动恢复要求，而是本次失败不得主动销毁唯一已有旧包。delivery8/8、fmt/diff-check绿；正常独立构建收尾证据见报告。Plan回executing 2/8，T03..06重开；后续覆盖表需区分事务标记存在与成功提交后可回收。合同内开放项不批准延期；未改实施/合入，merge-owned全档/规范/ledger/归档/guard均pending。

### Plan 792 final review R5（2026-10-16，pass）

受审 `41d95ea95d6d8209d2373336e758729826bc1fbf`；[R5报告/全部闭环/AC/SD](attachments/792/review-r5.md)。R4-F01 CLOSED：正式失败重试保留唯一旧备份/载荷/清单/标记，随后成功重试才回收；R1..R3保持闭合，无新增finding。delivery9/9、独立资源7控、全部先前反例及生命周期控制按预期，fmt/diff-check绿。正常新生成包4621构建输入与R4字节完全一致，依赖/工具链/driver/配置未变，明确复用R4 fresh171s/relocated locked150s/driver25绿的证据；未声称重新跑完整构建。SD LF冻结patch对当前main dry-run通过；并行794 UI新事实/795 Man规范须增量保留。Plan=reviewed 7/8，merge-owned集成冻结裸t/合同触面tu及实际VM触面tv、受集成闭包变化影响的consumer检查、规范/ledger/索引/归档/guard清理pending。下方各轮开放状态为历史；尚未合入/交付。

### Plan 798 readiness / 定向复审 R1（2026-10-16，needs_replan）

受审 `ce860fb244597d7929b5bc911880326cb945fa78`；[报告/反例/AC/SD快照](attachments/798/review-r1/review.md)。F798-R1-01..07开放：T06..08尚未实施、796/788未满足合入门；全表合计65536改每函数65536未升revision（须回规划）；周转gate-ms=0反控body-edit832.4154ms仍exit0；编辑30样本仅1次重算、restart未换进程、GC串行；512churn替换未命中实际零编辑，内存/正确压力store水位证据缺失；MIR缓存仅定义未接线，mir_build计数掩盖整模块lowering；独立锁未跟踪且新增警告/fmt红。重开T01/T04/T05，executing 2/10；保留T02/03原work标记但未授独立验收pass。已知未完成项使本轮止于readiness与定向核查，完整语义/信任矩阵pending。均为合同内开放项，登记不批准延期；先显式修订资源合同，再原树work闭合并补齐实际绑定/runner/证据后送审。未改实现/合入，merge-owned全档和收尾pending。

### Plan 798 readiness / 定向复审 R2（2026-10-16，needs_fix）

受审 `31628c5d7e11e92d92e0d931f89700d8a34eba91`，plan r2；[报告/证据](attachments/798/review-r2.md)。R1预算revision、锁入库及worker阈值旧例有实证进展（0ms反控worker exit5）；R1-F01持续开放，T06..08仍未完成，binding子进程exit3。R1-F03/F05未全闭：F798-R2-01 P2正式脚本在写summary后又用未赋值benchOut覆盖，独立反控29bytes→0；F798-R2-02 P1私有内存仅bench前后端点差，暂存计数在closure重分配前记录，不能证明OS峰值/8+16+8分区上界（未声称实测已超限）。正规化丢失T01/04/05行首的任务表、重开T05，executing4/10；其余work声明保留但未授完整独立pass。已知未交付使本轮仍止于readiness/定向检查，完整最终验收pending；原树先闭合所有finding及必验项。登记不批准延期，未改实施/合入；merge-owned全档与收尾pending。

### Plan 798 R2 修复轮新债务候选：已着陆 scalar 前端瞬态内存峰值（2026-10-16，F798-R2-02 归因；2026-10-20 R12 精确归属更新）

PLAN-798 T-05 峰值采样（worktree 修复轮）实测：`auto_ac_prototype::source::
compile_scalar_source` 在 267 KiB 冻结参考负载上的**单相瞬态峰值 ≈180-211 MB
（peak working set / PrivateUsage）**，稳定复现（隔离归因测量 211.8 MB、
request 子进程 211.6 MB、bench worker 全程 215.7 MB，端点差值仅 ~7 MB）。
后果：PLAN-798 §3 "worker private-bytes 相对空载增量目标≤128 MiB" 在冻结
负载上未达——探针自身 MIR/ABC/AV/cache 各层合计个位数 MB，目标 miss ~98%
由该前端相位贡献。ac-core 为 798 的只读面（并行合同），本计划内不可修。
归属建议：ac-core 前端 owner（PLAN-741/745/767 面）在后续计划中对
compile_scalar_source 的瞬态分配做剖析与收敛（或由 798 后续修订在依赖合入
后重定目标语义）。证据：docs/reports/plan798/turnaround.md（r2.1 内存章节）
+ turnaround-summary.json（frontend_peak_bytes/target_met 字段）+
av_resources_peak_sampler_distinguishes_transient_spike（采样器区分性反控）。

**2026-10-20 R12-04 精确归属（T-01 义务交付，分阶段测量，常驻工具
`experimental/ac-av-probe/probe/src/bin/mem_stage_probe.rs`，release 独立进程
2ms 采样）**：冻结 273 575 B 负载上，parse+resolve+typecheck（公共 seam 逐段、
值全持有）合计 ≤ ~20 MiB；Emitter 渲染 atom_text 4 719 087 B；
**`descriptor::bind(scalar_descriptor, atom_text)` 单独运行峰值
166 649 856 B ≈ 158.9 MiB——即 ac-core 公开缝 `compile_parts_with` 强制的
"源码→atom 渲染→atom 整体重解析+重绑定"架构内部、单一公共 API 调用内即超
128 MiB**；compile_parts_with 整体 209 444 864 B、完整 driver 请求 ≈207 MiB。
线性缩放（~765 B 峰值/源字节）：49/103/167/209 MiB @ 60/136/197/274 KiB。
798 侧（探针管线全层）个位数 MB，无授权内收敛路径；**2026-10-20 用户裁定方案 C 已落地**：PLAN-798 r5 §5.9 双线预算（A 前端瞬态≤224MiB ac-core owner / B 探针残差≤128MiB，均实测均硬门；T-11 @`06114f431`，当前证据 A=202.1MiB ✓/B≈3.5MiB ✓）——本条目转为改进机会记录：A 线收紧（ac-core 收敛 atom 渲染→重绑定架构）仍属 ac-core owner，非 798 阻塞项。

- 2026-10-16 PLAN-800 review R1：F800-R1-01（SD声称未实现的找包错误映射）、F800-R1-02（新核心格式门失败）为本合同待修候选，**不作可延期债务豁免**；见 [审核收据](attachments/800/review-r1/REVIEW.md)。当前 executing，T06/07待修复复审。

- 2026-10-17 PLAN-800 review R2：F800-R1-01/02 已在 c50e7a1c670c695766dfd81dcd572b446a99a31c 修复并独立验证 closed；[R2收据](attachments/800/review-r2/REVIEW.md)。无遗留延期债，状态 reviewed，merge-owned门仍 pending。

- 2026-10-17 PLAN-799 review R1：F799-R1-01（P2，新增ast::AutoResult破坏旧ast::*+error::*消费编译）为合同内待修候选，**不可延期豁免**；[独立反例与复审](attachments/799/review-r1/REVIEW.md)。状态executing，T02/03/05/06重开，原树修复提交后复审；整档/合入收尾pending。

- 2026-10-17 PLAN-802 review R1：F802-R1-01（P2，AC-03/V06 live iced 滚动消费与≤2s收敛必验缺口）为合同内开放项，不作可延期债务豁免；见 [R1报告](attachments/802/review-r1.md)。executing 4/8，T05/06重开；原树 work 补齐再送审，完整复审及 merge-owned 门禁 pending。

- 2026-10-17 PLAN-803 work T-06：Man 模板资源迁移登记的三项**预存遗留**（随资源层抽取如实入账，未在本轮修复——行为改善须另立计划，见 auto-man-assets/project.md 冻结语义节）：①`Templates::copy` 先创建/截断目标文件再 eval_template 的破坏性失败窗口（eval 错误会留下空/半截文件——基线实测 13 模板全部渲染成功，窗口当前不可由内嵌模板触发，但 panic 链逐字保留）；②模板求值旧 VM 插值怪癖（`app("${name}")`→`app("")`、多行整文件插值等，五族输出已按基线 `1dd123361a` 冻结在 tests/plan803_asset_facade_tests.rs）；③GHS exporter app 级 program.gpj mold 解析级求值失败（`Expected term, got Dep`，flipped 代码只含模板文本与数据无关、模板字节 hash 证明与旧臂相同 ⇒ 预存旧臂行为，default.gpj 生成正常，panic 边界已冻结）。收据：docs/plans/attachments/803/t01-oldarm-probe.txt + t01-newarm-replay-identical.txt；实现 904b5e5e0。

- 2026-10-17 PLAN-799 review R2：F799-R1-01已在4194fccc98f5c945288cceb030afd283726e1a8e修复并独立重放closed；新F799-R2-01（P2，auto-ast根模块导入顺序触发rustfmt --check失败）为强制健康门内待修项，不作延期豁免。ui-dialect-consumer锁定zvariant5.15.1镜像缺失为独立环境阻断（在线/离线均101），未误判代码回归；792交付独立25/25全绿。[R2报告](attachments/799/review-r2/REVIEW.md)。计划executing3/8，原树定向修复后复审，merge-owned门pending。


## PLAN-803 final review R1（2026-10-17，三项已由 R2 验证关闭；以下为历史 needs_fix）

受审 `bbad82a9f9a5b667d33fec28dcf680f6d95a1e7e`，revision=1；[独立报告与反例](attachments/803/review-r1/review.md)。F803-R1-01：三个 RustEmbed::iter 门面 opaque 返回破坏旧 Filenames 具体类型调用，旧正控过、新反例 E0308；F803-R1-02：新增测试 rustfmt 与 diff EOF 检查红；F803-R1-03：纯资源无 IO/内嵌说明缺 debug 读源码与 release 内嵌边界，实际解包 consumer debug 移开资源失败、release 成功，须如实说明而非追加未经批准的跨模式保证。T-02/03/04/06 重开，executing 2/8；T-01/05 完成证据保留。合同/健康/SD 缺口不能以记债代替关闭。原树 work 修复提交后复审，merge-owned 全档、Spec/ledger/归档/清理 pending；未合入。

- 2026-10-17 PLAN-799 review R3：F799-R1-01/F799-R2-01均closed；8069076b7c1b2dcba43bebb794ba9f901be1b6d2根入口格式门及永久回归独立通过，外部UI方言同锁文件26/26通过，R2环境oracle缺口清除，无新增延期债。[R3报告](attachments/799/review-r3/REVIEW.md)。final pass、reviewed7/8；冻结t+tv+tt+tu及合流/Spec-ledger/归档/guard仍merge-owned pending，未合入。


## PLAN-803 final review R2（2026-10-17，已由 R3 验证关闭；以下为历史 needs_fix）

受审 `a9b401421d205fdfea5c9c8b66d195c8589771e5`，revision=1；[报告](attachments/803/review-r2/review.md)。R1三项API/格式/构建模式说明均独立验证关闭，资源6/6、门面9/9、consumer4/4、旧asset1/1、adapter11/11与模式/缺资源反控通过。新 F803-R2-01：asset.rs:80/133/192具体Filenames修复引入三条refining_impl_trait_reachable警告（28→31），健康门未闭；保持具体API类型，以最小范围显式允许有意细化及理由，勿改回opaque/全仓关警告。仅T-03重开，executing5/8；修复提交后复审受影响警告/API正控。merge-owned整档与沉淀/归档/清理pending；记债不批准延期，未合入。


### PLAN-803 R3 闭合收据（2026-10-17）

F803-R2-01于`b455b4ef99210ab7d40e95c4b9e8340ad79e3340`独立验证关闭：三个方法级有意细化声明保留具体API，auto-man警告31→28回预存基线，无新增警告；标准门面9/9与格式检查通过。[R3报告](attachments/803/review-r3/review.md)。R1三项亦已闭合，无开放复审finding；原非目标遗留债仍留存。Plan reviewed7/8；merge-owned冻结门/沉淀/归档/清理pending，未合入。

- 2026-10-17 PLAN-799交付后核查：代码映射/祖先关系/工作树清理确认；F799-CLOSEOUT-01（P2，旧活动draft删除仅为未提交D，HEAD仍双计划）及F799-CLOSEOUT-02（P2，baseline/driver收据被*.json/*.log忽略而未入库）待收尾补正，不重开archived终态、不作延期豁免。[核查报告](attachments/799/post-merge-audit.md)。未改代码/合入/重跑主检出测试。

- 2026-10-17 PLAN-802 review R2：F802-R1-01 partial仍开放；F802-R2-01（P2，V06验收driver错误clamp/Reader失败可exit0）及F802-R2-02（P2，新增Rust格式红）为合同内待修，不作延期豁免。独立consumer7/7、定向27/27绿；见 [R2报告与反控](attachments/802/review-r2.md)。executing1/8、T02..06重开；第二轮non-pass，原树work需诊断/负控/全不变量闭合表再送审，merge门与收尾pending。

- 2026-10-17 PLAN-799交付后续查：a9a1e3943补正经独立git对象核对，F799-CLOSEOUT-01/02 closed（唯一archive终态、基线/driver等已持久入库，25/25及冻结SD hash一致）；新增F799-CLOSEOUT-03 P3仅为归档后7个Markdown相对链接缺../，另R3说明引用无输出format日志与check-6后缀小错，不推翻代码验收或重开终态。[续查报告](attachments/799/post-merge-recheck.md)。未运行主检出测试或修改实现。

- 2026-10-17 PLAN-802 review R3：F802-R2-02格式问题CLOSED，旧错误clamp/缺Reader负控已正确拒绝；F802-R1-01/R2-01完整时序证据仍partial。F802-R3-01（P2，修复oracle放宽≤2s门限，独立2.5s延迟+多拍捕获仍PASS）为合同内待修，不作延期豁免。见 [R3报告/计时反例](attachments/802/review-r3.md)。consumer7/7、定向27/27、fmt绿；executing4/8、T05/06重开，原树work继续；merge-owned门与收尾pending。

- 2026-10-17 PLAN-802 review R4：F802-R3-01仍OPEN（P2，同一≤2s验收缺口）：新零查询耗时2.5s负控已拒，但Reader与press各0.25s时，实际2.2s迟到/2.5s首见仍全部PASS。测量不确定区间跨阈值不得直接认证。见 [R4反例/报告](attachments/802/review-r4.md)。原提交/依赖未变，复用R3定向与格式证据；executing4/8，T05/06重开；合同内待修不作延期豁免，merge-owned门和收尾pending。

### Plan 804 final review R1（2026-10-17，needs_fix）

受审 `315ed5b5f325f8de12f7970981739cdb28ffdbd7`，plan revision 1；[独立报告/证据/AC/SD](attachments/804/review-r1/review.md)。F804-R1-01（P2）无父窗oracle在窗口销毁后读owner=0可误通过，且真save取消未验；独立Win32 parented反控证明旧oracle仍通过。F804-R1-02（P2）五次真DoDragDrop只直接消费DONE队列，没有实际on_dnd_finished送达发起App证据。F804-R1-03（P2）792复制整个auto-lang源码，host_adapters/装配等构建输入实际变化，不能以facade签名不变跳过V06 fresh/locked/重定位/保护控制重放。F804-R1-04（P3）新增fmt/warnings及native_dnd专用STA/受理即返旧说明未纠正。均为合同内开放项，登记不批准延期；未发现这些迁移路径的功能回退不等于必验已闭合。no-ui7/7、provider6、真dialog4/4、dialog-only1/1、真dnd五会话1/1、既有组件18/18、兄弟23/23、adapter8/8、事件目标helper1/1与双check过；owner误通过/实际事件链/792来源边界仍须修复补验。Plan回executing 2/8，T03..06重开，T01/02保留。merge-owned裸t/tv、合流/777/受影响consumer、规范/ledger/归档/guard pending；未修改实施/合入。

- 2026-10-17 PLAN-802 review R5 PASS：F802-R1-01/R2-01/R3-01已闭合，R2-02格式保持CLOSED。原R4输入独立重放有效对照通过、2.2s+查询耗时拒绝，六态自检通过；实机Reader1.713s/端点1.797s均在2s内。见 [R5报告/冻结SD](attachments/802/review-r5.md)。无新增合同内延期债；P802-D1预存局部槽债维持。reviewed7/8，merge-owned冻结t/tv与规范/ledger/归档/guard仍pending，尚未合入。

### Plan 804 final review R2（2026-10-17，needs_fix）

受审 `12e8936281620f660883a30e90beaa846e61af0e`，revision1；[报告/旧例闭合/反控/SD](attachments/804/review-r2/review.md)。R1-F01真owner/双取消与R1-F03交付重放CLOSED：独立dialog4/4、dialog-only1/1双腿、792正式driver25/25（fresh200s/重定位locked215s，producer clean@本树HEAD、根锁/source digest绑定）。R1-F02/F04完整约束仍有缺口：F804-R2-01 P2手工调用锚定/队列/目标/注入，绕过实际宿主dispatch/订阅/事件消费；F804-R2-02 P2修复回归，新增App测试exit0后左键仍按下，250ms后仍未释放，独立子进程反控已复现并恢复输入；F804-R2-03 P3新增renderer测试三组warning未清。no-ui7/7、定向56/56、双check0；五会话首次红、清理测试自身残留后同代码4.31s绿，不能把残留统一归用户按键。Plan回executing4/8，保留T01..04、重开T05/06；第二次final non-pass，下轮交接须先有生产消息入口/驱动join与键态收尾/changed-line warning审计的诊断与覆盖调整表。均为合同内开放项，不批准延期；merge-owned冻结t/tv与合流/777/规范/ledger/归档/guard pending，未改实施/合入。

### Plan 804 R3 readiness与定向复审（2026-10-17，needs_fix）

受审 `21d5f9c018cf64b1c6e7439075819871db7df17a`，revision1；[报告/旧例闭合/反控/SD](attachments/804/review-r3/review.md)。两轮失败诊断表已核，真实宿主方向修正、R2新增warning已清；五会话当前提交1/1（9.75s），退出立即/+250ms left/right/ESC均松开。仍有F804-R3-01 P2：work承认最终形态没有当前完整实机通过，不能用早期18.35s覆盖后改触发/收尾；正常全链本轮因预存Return按下未启动，不记成代码失败。F804-R3-02 P2新回归：新Child在窗口未出现/前台失败/finish断言等出口无回收守卫；受控初始化停滞两次复现测试25s超时exit101后宿主仍活，复审只清理本次跟踪的宿主且不发输入。均为合同内未解项，不批准延期；原树先修child全出口终止/wait并补成功/失败/退出输入反控。Plan回executing4/8，T05/06重开，T01..04保留；前两轮失败计数不重置，本轮止于readiness/定向核查。未改实施或合入；merge-owned整档/合流/规范/ledger/归档/guard pending。

### Plan 804 final review R4（2026-10-18，pass）

受审`1ca36a674e9199985f8bc804d1ac0f61d1bcaa97`，revision1；[独立报告/AC/闭合/SD](attachments/804/review-r4/review.md)。R3-01 CLOSED：当前重新编译、真实生产宿主链独立test exit0/5.15s，started=true、finish=copy且生产CALL_HANDLER达InitiatorApp、干扰方无marker；R3-02 CLOSED：原始外部初始化停滞反例exit101+25s诊断后宿主零遗留，NO_DRAG实际started=true但finish缺失的交付失败同样exit101后零遗留，两个反控均由测试ChildGuard自行kill+wait、外层回收数0。正常/交付失败退出立即/+250ms输入全清，R1/R2既有问题保持关闭，无新增finding。锁/config/manifest及生产/fixture/交付生成器和driver未变，逐路径核对后复用R2独立历史证据，不把历史bundle digest冒充当前整包。Plan reviewed7/8；最终合流冻结t/tv、共享并集/777/受影响consumer及实际producer清单、规范/ledger/归档/guard均merge-owned pending；未改实施/合入或跑整档。此前各轮开放状态为历史，当前合同内findings均已闭合。


### Plan 807（2026-10-18，独立 final review R1，needs_fix）

受审 `1f5e0514181d9c108809487fedd2f1ac27ca40cf`，[报告/AC/反例与修复要求](attachments/807/review-r1/REVIEW.md)。当前 **executing 3/10**，保留 T-02/03/04，重开 T-01/05/06/07；本登记不批准延期。
F807-R1-01（P2）全局默认槽空态假设使 703+807 同进程组合 13绿/1红；F807-R1-02（P2）新身份测试缺 feature 门控，no-feature 新增4处 E0433；F807-R1-03（P2）冻结的 a2r 四条 off panic 不实，旧/新 window-off 均无导出（E0425），需真实签名矩阵/收据对账，非本轮新运行回归；F807-R1-04（P3）触面fmt、新rustdoc私有链接警告及handoff引用/计数未净。
core65、consumer3、相关41、独立core反例5/VM-on3/off组合1、777 strict及自测15、792 driver25均通过（集合重叠）。新1GB谱结构3,033,703B≤12MiB/RSS16.5MiB已实际绑定；无资源未测延期。原树修完整finding集合并提交后复审；merge-owned裸t/tv/tu、必要资源影响面、Spec/ledger/归档/guard收尾仍pending，未修实施或合入。


### Plan 807（2026-10-18，独立 final review R2，needs_fix）

受审`1eb8434f50e104f593914701f30318b9af255a6e`；[R2报告/闭环/反例及下一轮诊断要求](attachments/807/review-r2/REVIEW.md)。当前 **executing 5/10**，保留T-01..05，重开T-06/07；不批准延期。
R1-02/03已闭；R1-01旧单线程15/15闭合，但F807-R2-01（P2）共享默认写者仍污染真实reader：完整703+807并行首跑13/2、另3新进程全exit101，703已有文档却返回“编辑器不存在”，继而poison registry锁；pair-only20/20不能代表完整隔离。R1-04卫生大部已闭，但F807-R2-02（P3）handoff摘要误拼且当前源包身份未正确重绑（新包7文件/229812B/6b7380e5…）。
core65/consumer3/顺序15/fmt/doc/777均绿，1GB结构3,033,703B/RSS16.5MiB本轮重验绑定；关门本模块4编译错误已消除（62旧族另列）。连续第二次final失败，原树修复前先提交根因/覆盖调整，保留实值oracle及完整并发反例；merge-owned整档/Spec/ledger/归档/guard仍pending，未修实施或合入。


### Plan 807（2026-10-19，独立 final review R3，全部关闭）

受审`4e3bde4d6f837821c15e668688bce4dd7c8e84a9`；[R3报告/AC/闭环与证据复用边界](attachments/807/review-r3/REVIEW.md)。**final pass / reviewed 8/10**；R1/R2 findings全部CLOSED，无新增阻塞。
R2-01默认槽写者已隔离进子进程，父进程真实reader无共享污染窗口；703+807在1/8/16线程各2次15过/2子体ignore，更宽组合56过/4显式ignore，实际两体各1/1及helper断言失败传播正反控通过。R2-02当前源包身份7文件/229812B/6b7380e5…已正确绑定，历史与当前证据已区分。唯一改动为测试编排，其余代码/依赖/配置/Spec指纹未变，资源/发行/VM等具体证据按报告有界复用。
T-08完成；merge-owned裸t/tv/tu、必要资源影响面、master共享片段/777、Spec/ledger/归档/wt-guard仍pending，未合入或交付。下方/上方旧轮OPEN记录仅为历史，当前以本轮关闭结论为准。


### PLAN-810（2026-10-19，独立 final review R1，needs_fix）

受审 `ccd024d7208caa14fb2f71d71b8d256d2ba74f82`，revision 1；[完整报告/AC/反例及修复要求](attachments/810/review-r1/REVIEW.md)。当前 **executing 4/11**；重开 T-02/06/07/08，保留单源迁出与接线成果；本登记不批准延期。

F810-R1-01（P2）成功投机快照残留：默认 store/pub style、公开 widget 门面在 1/8/64 项后仍持 1/8/64 槽，builtin widget 和 body 回退对照为 0；F810-R1-02（P2）777 strict 实红 stale_baseline（Cargo.toml），handoff 绿不成立；F810-R1-03（P2）style None/session/store 护栏测试空壳及 V03/V04/V05 工作期 oracle/矩阵缺口；F810-R1-04（P2）scene 两 push 重同步误用 stream 一次推进原语，prev.pos 旧 6/新 0；F810-R1-05（P2）SD 中 computed/完整迁出状态、inner wrapper、端口归属等冲突；F810-R1-06（P3）新文件 fmt 实红及新增 core_parser warning。

本轮 check/build、plan810 13、轻 consumer locked 7、806 18、408 20、051 25 通过（集合不宣称无重叠）；051 初次直接执行 cwd 错误导致语料找不到，按 Cargo 包 cwd 复跑全绿，未归为产品红。未跑整档或修实施，未合入/归档。原树完成全部修复与定向证据后提交复审；merge-owned 裸 t/tv/tu、必要触面、Spec/ledger/归档/guard 仍 pending。

### PLAN-810（2026-10-19，独立 final review R2，needs_fix）

受审 `9226c8af9365389342bbc63d9c91e9ff0b6fee42`，revision1；[R2报告/闭环/第二次失败诊断要求](attachments/810/review-r2/REVIEW.md)。**executing 5/11**，T-02及已有迁出接线通过；T-06/07/08未闭。R1-01/02/04/06 CLOSED，R1-03/05 PARTIAL；上方R1 OPEN为历史，不批准延期。

F810-R2-01（P2）同实例错误oracle仍未齐：fresh Parser成功不能证明失败scope；独立控制中合法handler后arg不可见，缺body错误后arg仍可见（on旗标已恢复false，旧源码同样先`?`后exit_scope，按原合同具名保留而非顺手修复）。新承认fragment EOF缺原基面隔离证明；本轮当前输入`view fn F(x: int) { if x {`在5秒截止未返回，子进程已终止/回收，不冒称旧提交或固定崩溃码。F810-R2-02（P2）792现存25/25属于旧源包，包内无raw_cursor_release/resync，旧/新hash不同，须在最终修复源码重验适用发行门。F810-R2-03（P2）parser-api仍同段声称完整Parser逃逸口“仍使用/已删除”，旧失败边界/基线总结未调和。

新测20/20、轻consumer locked7/7、80618、7784、40820、05125、inline_lambda9（含VM及四生成器）、Rust counter1、CSS7、strict777与self-test15/15、触面fmt2021均过；独立R1快照/prev原例已全部关闭。未修实施或合入/归档，未跑整档。原树work先完成报告的具名根因/覆盖调整表，再修完R2-01..03提交复审；merge-owned裸t/tv/tu及后续沉淀/guard仍pending。


### PLAN-810（2026-10-19，独立 final review R3，仅新增两处告警未闭）

受审 `e49af4d0d253db1ed1a406a2e168f515d98eda1e`，revision1；[R3报告/闭环与证据适用范围](attachments/810/review-r3/REVIEW.md)。**R2-01..03 CLOSED，R1剩余全闭；仅F810-R3-01（P3）OPEN**：新同实例双测737/763行`let mut bridge`产生unused_mut，AC-06要求新增warning清零。当前 **executing 7/11**，只重开T-06；原树移除两处多余mut并提交后定向复验，其余成果保留。

本轮22passed/1显式ignore、fmt2021通过；同实例状态/旧边界、新792生产源25/25适用性和SD单一事实已核实。七crate817个src文件仅cfg(test)排除的新测试文件不同，816个产品输入逐字节相同，发行证据限定普通fresh/locked构建复用；EOF直接测试子进程正常正控1测、零用例负控被拒、准确异常输入5秒截止后已终止/回收。产品源/锁/配置与R2未变，其余777/轻包/VM及四生成器证据有界复用。未修实施、未合入；merge-owned裸t/tv/tu及后续沉淀/guard仍pending。

**P810-R3-D01 / 非阻塞工具复用候选**：一次性`plan810_baseline_identity.py`的类型标注正则会抹掉`TokenKind::Dot =>`枚举项，不能无条件作为未来迁移的等价性validator。本轮未靠该宽松比较替代验收：独立完整函数边界+保留`::`枚举项的比较再次证实10函数同构，并实际检出10/10 guard/call/case变异，当前EOF基面证据已补强。后续复用该工具前应收紧正则并自带变异负控；并非延期当前未满足的行为要求，无当前UI运行代码变更请求。

### PLAN-810（2026-10-19，独立 final review R4，pass）

受审 `6b4963de3ffd99c7f68205fe7e9ffd675e812f14`，revision1；[R4报告](attachments/810/review-r4/REVIEW.md)。**F810-R3-01 CLOSED：两处unused_mut已去除，受影响同实例双测2/2、目标新增诊断0、格式通过。** 全部契约内findings已闭，无新增阻塞项；既有产品代码、锁/配置/输入及Spec提案不变，R3及更早证据按报告有界复用。状态 **reviewed 9/11**，T-10/11仍pending；merge须在最终冻结提交跑裸t/tv/tu并完成沉淀/ledger/归档/guard，尚未合入。P810-R3-D01仍为上条所述非阻塞工具复用候选，不将其伪称已修复，也不以记债替代当前验收。

### PLAN-812 merge 环境风险候选（2026-10-19）

- **P812-D1 / low / 测试基础设施**：fixture 子 Cargo 未隔离父 `CARGO_TARGET_DIR`，但产物按各自 `target/release` 查找；本次缓存复用环境导致 FFI oracle 已编译而找不到文件、构建锁争用和超时。属于非标准构建环境兼容性候选，不修改本次产品修复；标准目录、清除本次无效 `.parity_cache_ok`、预热 AC worker 后相关定向复验通过。后续宜隔离子构建目标目录并校验 marker 对应产物。证据：docs/plans/attachments/812/verification.md；首轮 2 FAIL/13 TIMEOUT 的 raw log 保存在 recovery 收据目录。
- **P812-D2 / resolved / Windows 审计测试 ABI**：PLAN-813 R9 的 `cargo t`、`cargo tb` 与两种隔离 feature 配置均复现 P765 子进程退出异常。入口标记确认 IAT 追踪成功补丁 2 个槽并进入 `traced_exit_process`；根因是 `RtlCaptureStackBackTrace` FFI 漏掉第 4 个可选 `BackTraceHash` 指针，x64 调用属于未定义行为。PLAN-813 Phase 2 补齐四参数签名、匹配 `u16` 返回类型，并在两处调用传 `NULL`；`test-book` 与日常 `ui-iced,test-cases` 配置隔离回归各 1/1 通过，`cargo check -p auto-lang` 通过。完整 `cargo t` / `cargo tv` / `cargo tb` 尚待 R10 复审通过后在最终冻结提交重跑，不以定向通过代替 merge gate。详见 PLAN-813 §9 T-10 与 `attachments/813/p765-ffi-fix-r13.log.txt`。

### PLAN-813 merge gate 候选（2026-10-19）

- **P813-D1 / resolved / Rust 书示例与对拍（PLAN-819 Phase 1）**：PLAN-813 首轮 `cargo tb` 的 `book_ch02_05` 与 `book_ch03_08` 曾因 Auto 源误用 `=>` 而失败。用户明确要求将修复并入 PLAN-819 新 phase，不另立计划；PLAN-819:r5 已修复并合入 Book master `d05218b`。在 PLAN-813 R9 最终组合、`AUTO_BOOK_LISTINGS` 指向隔离 Book worktree 的 `cargo tb` 中，两例分别通过。R9 完整档曾被 P812-D2 的 P765 IAT 测试阻断；该 ABI finding 已由 PLAN-813 Phase 2 修复，这不改变 P813-D1 两个原始失败已关闭的结果。证据：PLAN-813 §9 R9 门禁收据与 `attachments/813/cargo-tb-r9.log.txt`。

### PLAN-819 merge gate residual (2026-10-19)

- **P819-D1 / resolved / Rust 书 ch06 语法与金样对拍（由 PLAN-819:r5 交付关闭）**：PLAN-819 r2 `cargo tb` fail-fast 报 ch06 `book_ch06_01/02/03/04/05/06/08/09/10` 红；r4 干净树定向复现确认 05/06/08/09 为作者 Auto 源误用 `=>`（parser 要求 `->`），修正源后这些 golden 仍需同步当前生成输出（旧标准库导入、类型宽度及借用/分支形状变化）；01/02/03/04/10 为枚举 derive 新增 `Hash` 导致的 golden 漂移，11 输出将 `x.unwrap_or(0)` 改为 `x`。用户于 2026-10-19 授权将问题纳入 PLAN-819 Phase 2，不另立计划。Phase 2 已修复并通过 10 个 ch06 定向对拍，revision 4 最终复审 pass；主书仓 9 个未提交 golden WIP 保留，其中 ch06 六项与计划实际输出哈希一致。revision 5 冻结组合的 `cargo tb` 已通过（5312 passed / 1547 skipped / 1 slow）；Book 已在 `d05218b` fast-forward；PLAN-819 已归档至 `docs/plans/archive/819-rust-book-listing-syntax-fixes.md`，本项关闭。证据见归档计划 §9。
- **P819-D2 / resolved / Rust 书 ch08/ch09 对拍与 Auto 语法（由 PLAN-819:r5 交付关闭）**：revision 4 冻结组合 `cargo tb` exit 100（1150 passed / 4 failed / 1547 skipped，fail-fast），失败为 `book_ch08_01/02/04` 三个旧 golden（实际输出含当前 `a2r_std::value_len` 及新容器索引/克隆形状）和 `book_ch09_02`（作者 Auto 源及 EN/ZH 正文将模式分支写成 Rust `=>`，Auto 要求 `->`）。真实定向复现确认 ch09 源修正后还需把旧 `Option<i32>`/冗余导入 golden 同步为实际 `Option<i64>` 输出。主书仓 ch08 三份未提交 golden WIP 哈希均与隔离实产完全一致且原文件未触碰；门禁只在隔离 PLAN819 书树生成过副作用，已清理。按用户指示，不另立计划，直接纳入 PLAN-819 Phase 3；已提交 ch09 源/EN/ZH 修正与四份实产 golden，四个精确测试均 1/1 pass、无 SKIP，书树 clean。revision 5 独立最终复审 pass，冻结组合 `cargo tb` exit 0（5312 passed / 1547 skipped / 1 slow）；Book 已 fast-forward；PLAN-819 已归档，本项关闭。PLAN-813 R9 的两个早期 Rust listing 目标亦在最终组合通过；P812-D2 曾阻断 R9 门禁，现由 PLAN-813 Phase 2 修复，最终三项门禁待 R10 复审后重跑，见 PLAN-813 §9。证据与门禁日志见归档计划 §9。

### Plan 798 final review R3（2026-10-19，needs_fix）

受审 `b9682967fb177b065ea76e5240048e3321aef8ef`，plan r2；[报告/AC/V/SD/全部独立反例](attachments/798/review-r3/review.md)。D796/D788已满足，旧依赖阻断关闭。开放F798-R3-01..11：owner/越界类型/未知能力及import/Decoded版本信任门漏检；真实manifest profile/sem/签名/logical及当前能力未验、796 ref只分别自检未关联；位置漂移ABC命中丢trap span；根帧预算及始终0的槽高水位/平坦表达式前门预算；父runner对exit9、额外帧和错source_hash均误绿，65.189s响应突破60s会话；正式窗口扣掉父call内IPC，重建冻结worker后30样本编辑P95仍1084.5ms红；156KB wire/24581条目使verifier增量54,239,232bytes后才拒绝，32MiB受管守卫未落实；T08有限代际/native准入仅文本无模型；新增warning/README diff-check和既有语义corpus缺oracle兼容红；native helper仍走无限capture兼容API/事后预算。常规定向35+资源7+合同20+正式正负3通过不覆盖这些不变量，兼容15过1缺件红。全部合同内开放项，登记不批准延期或降格；源自前端的128MiB目标未达归因不替代32MiB硬预算。Plan executing1/10，保留T01最小vertical，重开T02..08；root整档/merge-owned收尾未执行，未改实施或合入。下一轮须不变量闭合与oracle覆盖调整，不只单例补丁。

### Plan 798 修复复审 R7（2026-10-20，needs_fix）

受审`36108624a432d50df65bda0725e66a205f56950b`，plan r2；[报告/旧8项闭合边界/全部反例](attachments/798/review-r7/review.md)。当前F798-R7-01..05：native开始尚未过期时仍不给link剩余截止（本次自身伪钟前耗59秒后helper完整执行12.15秒，非实测71秒）；bool关键字/命名实参冒号仍重置128层计数；可解码空sidecar仍恢复为无来源trap；82字节COFF声明raw data在u32::MAX仍获准；合法if/else双return被lower.unwired-block拒绝，独立native与expected均1、落空对照AV=1。前4项延续R6完整不变量；第5项为原支持面新发现，非本轮修复回归。错键/错型参数/原解码放大/警告与缺oracle行为已修；core20+指定probe57+正式协议4+兼容16+合同20均过，不证明完整矩阵。原预算未批准降格，contract-decision r4(c)不能代替修复；新基线30样本V11本轮pending，不拿旧提交性能授pass。Plan executing3/10，保留T01/03/06、重开T02/04/05/07/08；原树work修复提交。未改实施/规范/ledger或合入；merge-owned全档/沉淀/guard仍pending。R7源码/锁/raw须保留，旧附件缺失不能用work转述当独立证据。

### Plan 798 修复复审 R8（2026-10-20，needs_fix）

受审实际`f9d9821577eefd0ccfb74530b7d1077923ea9c33`，plan r2；[报告/旧5项闭合范围/全部源码与raw](attachments/798/review-r8/review.md)。当前F798-R8-01..03：前门全局操作计数误拒合法128/129/200个浅层命名实参（Checked HIR→真实MIR/ABC/AV均=1）；sidecar只覆盖Bin，MIN一元NegI32在空映射下仍恢复为无来源int-overflow@；新native监督器之前的首次worker构建仍Command.output无deadline/caps/收容，r4(c)旧预算降格文字也未实际撤销。三项分别延续前门、来源、全程有界不变量，不要求新支持面。旧在途linker反例现1013ms即Budget/已回收、双return两端=1、深链/二元空sidecar/COFF节窗口原例均已修。core20+指定probe62+正式协议4+兼容16+合同20均过，fmt/diff-check0；当前提交完整V11本轮pending。Plan executing3/10，保留T01/03/06；原树work修完三项提交，并保留复现资料。未改实施/规范/ledger或合入；merge-owned冻结门禁/沉淀/guard仍pending。

### Plan 798 修复复审 R9（2026-10-20，needs_fix）

受审`e00340842fdca15417c3195041ca909b1fcb8bb1`，plan r2；[报告/闭合范围/源码及原始输出](attachments/798/review-r9/review.md)。当前F798-R9-01/02（均P2，合同内必修）：闭括号时construction全局相加，64条独立`let vI=(1+1)`误拒为129层，同一Checked HIR→MIR→ABC→AV正确=2；新首次准备器不读stdout、stderr超帽不传播失败、仅kill父进程——2MiB stderr仍Ok、128KiB stdout堵塞误超时、1.5s截止返回后后代heartbeat继续15→19，review已清理本次辅助子树。原宽实参例、NegI32诚实来源/空映射拒绝、旧r4(c)降格文字已验证修复；R8-02关闭，T04恢复。core20+probe70+正式协议4+兼容16+contract20均通过，两package fmt/diff-check0；完整新基线V11未跑，不能用work旧摘要授pass。保留T01/03/04/06，T02/05/07/08待完成，executing4/10；原树work需补最大路径组合及双流/超帽/持管后代实际回收oracle与诊断。登记不授权延期/缩围；未改实施/发布规范/ledger/合入。最终pass后的merge-owned冻结门禁/沉淀/guard清理仍pending。

### Plan 798 修复复审 R10（2026-10-20，needs_fix）

受审`f24bd46a6f84286c30c90b156d9b4e7d72ad1358`，plan r2；[报告/精确程序/源码与raw](attachments/798/review-r10/review.md)。当前F798-R10-01 P2开放：不同兄弟实参分别有60层括号与70项加法链，前门把独立最大值相加为131层误拒；源码nesting62，直接Checked HIR→MIR→ABC→AV=1/143步，交换表达式=70/143步。两轴单独均通过；真正同路径60层调用+70项链仍正确拒绝。属于原最大构造路径不变量未闭，非新增规模/语法要求。原连续浅语句16/63/64/65/100均已通过；**R9-02关闭**：双流帽、排空、超时整树清理与非零退出/缺产物均独立通过（后代heartbeat15→15且OS进程不存在）。core20+probe72+正式协议4+兼容16+contract20=132定向测试通过，fmt/diff-check0；完整新基线V11未跑，不据work旧摘要授最终pass。保留T01/03/04/06，T02/05/08待修、T07完整验收pending，executing4/10；原树work补sibling路径关联及异路径/同路径/顺序/语句组合oracle与多轮诊断后提交再审。登记不授权延期/缩围；未改实施/发布规范/ledger/合入，merge-owned门禁和收尾pending。

### Plan 798 修复定向复审及r3 readiness R11（2026-10-20，needs_fix）

受审`5def3af48689487a212b518df276e737cb62d8bd`，当前plan r3；[报告/精确输入/HIR层数/源码与raw](attachments/798/review-r11/review.md)。F798-R11-01 P2：R10修复使同一个命名实参内部嵌套调用与外层左结合链不再组合，真实Checked HIR最长131层仍前门放行且Driver返回70；位置实参同131层却resource。原7组异路径误拒及5档连续浅语句已闭，不等于完整预算成立。F798-R11-02 P2为r3已列明未完成的V-14缺口：单函数完整module含未执行直接自调用、另一分支42；编码前is_unit=false且公开手工DecodedAbc授信后执行42/6步，decode后变true导致mir.unit-callee，实际Driver fresh同拒。非这两文件的新回归，r3要求未被r2 work93测覆盖。三个精确定向测试各1/1、fmt/diff-check0；因readiness未满足未跑完整最终门或整档。T01/03/04维持r3重开、T07/08待完成，重开T02/05，仅T06完成，executing1/10。原树work需补每sibling完整组合路径及kind/版本/缓存/多入口oracle；登记不授权延期/缩围。R9-02关闭记录保留；未改实施/发布canonical规范/ledger/合入，最终pass后merge门禁和收尾pending。

### Plan 798 修复复审及r4 readiness R12（2026-10-20，needs_fix）

受审`bf7e44f8ef18d9f6b9c03467986f5b91cadc58c3`，plan r4；[报告/反例/命令/当前资源数据与raw](attachments/798/review-r12/review.md)。开放四项P2：F798-R12-01真实Checked HIR仅71层的尾操作数深调用被前门估成192层误拒（直接管线=70/262步）；R12-02 raw unit仍获executable凭证并直接执行（零参=7/2步、带参=42/1步，未走闭包装配/完整entry门）；R12-03新增kind位而wire/schema/producer/sem/key未隔离旧语义，旧无kind单元raw误作module且真实store hit1/miss0/emitted0成功7；R12-04冻结273575bytes当前release请求值正确2665，但private-bytes空载761856/峰值211918848/增量211156992（约201.4MiB）>128MiB，r4明确历史前端归因/债务不是豁免。资源若只读ac-core不可行按T01给owner阻断及有界修订回new，不扩大上限/扣前端/抢写依赖。R11原131层漏准入和module42往返具体例已验证修复；四个精确定向测试各1/1、fmt/diff-check0，不证明完整K14/资源。仅T06完成，executing1/10；未跑完整最终门或整档，未改实施/发布canonical规范/ledger/合入。登记不授权延期/缩围；最终pass后merge门禁/归档/guard仍pending。

## D-815-L：派生账本无法按store schema加载（2026-10-20）

PLAN-815 T-14收尾发现：v56备份已有1060条历史item与六section不满足store schema，v57四条815投影亦不完整。JSON/ID校验不等于store成功。原件/备份保留；修后815脚本在schema前检拒写。记录恢复责任在PLAN-815 r3/M815-L（T-15 work→T-16独立审查→T-17受管恢复及815 upsert→关闭T-14），不新增编号、不回滚已合入A代码；R4仅未变代码阶段pass保持。审计与原件SHA、恢复方案见`docs/plans/attachments/815/t14-closeout/`。未完成前不宣称完整阶段发布，不归档不清树。

## D-815-L1：恢复工具失败路径尚未通过审查（2026-10-20）

T-16/L1 needs_fix，feacf79e8：六项F-815-L1-01..06开放，涉及CLI假成功、并发写入/回滚丢失外部内容、原值/ID转换与删除保护、schema负控、双checkpoint/version及命令冻结SHA。均为r3合同内阻断，不能以既有账本缺陷或记录为债务豁免。原候选可被真实store加载/往返的正控不等于安全发布。T-15重开，T-17与T-14继续pending；原树修复并新SHA复审，生产账本未改。反例及完整证据：docs/plans/attachments/815/review-l1/review.md。

## D-815-L2：恢复复审剩余失败不变量（2026-10-20）

T-16/L2 needs_fix @1082bde52：L1-01/04/06已独立闭合，双checkpoint57→58→59/1098项真实往返通过；L1-02部分写失败留下本运行257字节残留、L1-03存在性/legacy删除/section声明保护仍开；L1-05代码闭但当前收据旧版本策略残留P3；新增L2-01最终SHA已不符仍成功退出。T-15重开，production v57原件未动；这些是r3合同内阻断，不能以debt豁免。原树按两轮未闭诊断修复后新SHA复审；证据 docs/plans/attachments/815/review-l2/review.md。

### Plan 798 修复复审及r5 readiness R13（2026-10-20，needs_fix）

受审`06114f4316468030f1b160bb028046b9e5beab71`，plan r5；[报告/原反例闭合/新prefix输入/双线raw与身份](attachments/798/review-r13/review.md)。R12四项在当前范围关闭：71层尾表达式合法、unit不可执行、wire2隔离旧1、内存按用户批准r5方案C双线门（A224/B128）本轮release六场景×1实测A201.6MiB/B3.6MiB且四态门控通过；历史r4单线失败不改写。新F798-R13-01 P2：前缀unary在callee标识符operand时被消费，随后遇'('撤回名字把prefix一起丢掉；20个-/!加110层调用真实HIR131/132仍放行并返回1，同语义仅加括号则正确resource。116/24层合法控制通过。53项不同定向测试绿、fmt/diff-check0，不覆盖此新组合或完整最终验收。T03/04/06/11保留，T02重开，T01/05/07/08完整K14/正式30样本readiness原已pending，executing4/11。原树work补prefix绑定整个调用的状态与-/!/分组/普通operand区别性oracle后提交再审；未修实施/发布canonical规范/ledger/合入，最终pass后merge门禁和收尾pending。登记不授权延期/缩围。

## D-815-L3：恢复审查缺口已闭，实际恢复待T-17（2026-10-20）

T-16/L3阶段pass @0a65ee2a7：D-815-L1/L2涉及的CLI/schema/并发/部分写/前像/终门/收据finding均独立关闭，无新增。真实CLI57→58→59/1098项往返、A/B六种写失败与持续存储失败的准确残留/完整备份已验证。获准身份与映射边界见 docs/plans/attachments/815/review-l3/review.md。该结论只清偿审查缺口，**D-815-L的生产账本恢复尚未完成**：live仍v57原件未动，T-17真实恢复/upsert/sidecar与版本/读回收据及T-14终验仍pending；不得以review pass宣称阶段发布完成。

### Plan 798 修复重放及 readiness R14（2026-10-20，needs_fix）

受审 `c8c598f0aa67f79d8b083346b5c552a655353d1d`，plan r5；[报告/冻结身份/反例/正式门/raw审计](attachments/798/review-r14/review.md)。R13-01 已独立关闭（131/132层门前拒绝、116/24合法控制执行），新增 self-call 正式 runner 末步 hit1/emitted0/execute1/value42，正确性4/4。正式30样本门一次运行 edit P95=1195.8ms/sig540.3ms、exit101；共享CPU启动62%/期间43%且有并行构建，未重跑A/B，不断言无码回归，不反复取绿。新 F798-R14-01 P2 为合同内验收证据缺口：提交两份180行raw无双线内存字段，旧summary不能对应r5；pass raw重算edit P95=897.3352ms（仍过，摘要888.8ms不符）。需找回同轮真实BenchSummary/退出码/raw哈希和提交身份并更正摘要，无法恢复才受控重测；不得手填或混轮。T05重开、T07/08仍开放，保留T01/02/03/04/06/11，executing6/11；登记不构成豁免。未修改实施或canonical规范/ledger、合入或清树，最终pass后的merge门禁与收尾仍pending。


### Plan 798 独立最终验收 R15（2026-10-20，needs_fix）

受审 `79b31f5dcb99b2b95e1c79a5ae58299c0139f8f6`，r5；[完整报告/AC-V-K14-SD/原始反例与输入](attachments/798/review-r15/review.md)。R14-01 CLOSED：同轮180raw/summary/hash齐全，edit P95 414.7482ms、A201.3MiB/B4.2MiB、正式30样本保存日志948.9ms PASS；此前关闭原例保留。新增五项P2合同内遗漏：F798-R15-01 A08/A09缓存命中不对当前grants复核，caps=[]仍Success7+MarkA；02 单元信封接完整module载荷，可信producer-kind冲突仍执行；03 Driver空可解码sidecar被当bound，A08/A09命中trap span=None（正常重推导正控23..28）；04 manifest required边0且生产无lease，只有manifest lease移除scratch项目根后GC删除3块，后续恢复unit missing；05 正式runner locate_worker构建准备器stdout不排空/无Job，原函数提取体300ms加速反控超时后自建后代仍STILL_ACTIVE（已定点清理）。新增P3 F06 core/src/abc.rs:1063 unused import（实验验收实际warning，非根既有566条）。实验101/正式正确性4/共享probe兼容16通过不覆盖新失败；原树work同批修复并补入口/状态闭合矩阵再提交送审。T01/03/04/05/07/08重开，保留T02/06/11，executing3/11；登记不授权延期/缩面，不要求825/826未来优化。未改实施或canonical Spec/live ledger、未跑根整档/合入；最终pass后的merge-owned冻结门禁/沉淀/归档/guard仍pending。


### Plan 798 r7 最终复审 R19收口（2026-10-20）

R15六项、R16多单元lease及R18位置修订manifest/sidecar GC不变量已关闭；当前83086e76f，cache20/resources17/formal7、同轮180计量/双线内存及原独立反例均通过。R17 warm-only时延阻断按用户r7交付裁定关闭（历史失败保留），完整函数增量归优先828；不能把它写成旧失败测试已改绿或新hot架构已实现。新正确性组合反例、修复与覆盖见[诊断](attachments/798/review-r18/diagnosis.md)，最终逐AC/V/K14/SD见[R19](attachments/798/review-r19/review.md)。旧“needs_fix”段落是历史，不代表当前仍开放；merge-owned整档与沉淀/归档/清理仍pending，未借债务登记豁免验收。

## D-815-T04-R1：生成主体提前定向复审六项开放（2026-10-21）

受审 90aed7fc7、r3；[报告/反例/冻结输入](attachments/815/review-t04-r1/review.md)。F-815-T04-R1-01..06：P1 发行包漏 auto-project-model/auto-toolchain-core 与 compiler 单测缺 tempfile；P2 旧公开路径断裂、资产迁移未同步脚本/锚/围栏/viewer lock、plan734 金样格式化新红；P3 根 lib 格式。真实交付生成后 metadata 即失败，诊断副本逐补两来源复绿；旧路径消费失败/新路径正控过；Man seam+811 5/5、777 strict与独立窄工具闭包/4进程通过不覆盖上述失败。登记为合同内未完成项，不授权延期/缩面或恢复重复资产。T-04 重开，executing9/17；原树一次修齐、提交后再审；T-05 等 M814-B 的门保留，历史 A/L 发布保留；未执行整档、改实施或 canonical/live ledger、合入、归档、清树。

### D-815-T04-R1 的 R2 关闭更新（2026-10-21）

受审 dbd295f05/r3；[R2 报告](attachments/815/review-t04-r2/review.md)。R1-01..05 已独立关闭：实际 source 闭包/缺两项负控、compiler 测试依赖、旧公开路径、资产消费和 plan734 金样均通过；本轮51条定向Rust与17布局通过，792 work 冷构建证据经内容身份核准复用。唯一 OPEN 为 R1-06(P3)：原空行已清，但新增根转发行 lib.rs:3479 仍使 rustfmt --check exit1。仅修格式并提交可再次定向核验，其他输入不变无需重跑五项/792冷构建。旧六项全开段落仅历史；本轮无新增功能 finding、不授权延期卫生验收；T-04保持未完成，executing9/17，未整档/合入/归档/清树或写canonical/live ledger。

## D-829-R1：AutoGraphics 原型独立最终复审未通过（2026-10-21）

PLAN-829 r1 @ `f7ea95f3b71459914cc081d70dd62123c1303a6d`，final `needs_fix`；[完整报告/逐AC/SD/反例及raw](attachments/829/review-r1/review.md)。F829-R1-01..11开放：P1为LIVE跳跃键未接、Iced最终合成不画仍PASS；P2涉及按重绘计时、Squash原点瞄准/缺detector及朝向动画、缓存旧key跨代际绑定/64轮释放后allocator live3→67、生命周期20目标故意泄漏仍报恢复且after采样实际在负载前、source/像素oracle缺口、metrics及证据指纹、r1禁复制patch与实际vendor冲突、3D未接共用scene/身份/delta。P3为4条测试warning/格式及提案事实矛盾。独立74测试与6真实probe通过不覆盖这些反例；30hash匹配。均为合同内未完成项，登记不授权延期/缩围/批准vendor；原树work修复并提交后复审。T01/02/04..09重开，仅T03保留，executing1/11，T10/11待完成；未修实施/跑根整档/发布canonical或live ledger/合入/归档/清树，最终pass后的merge-owned门禁及迁移交接仍pending。

### Plan 814 B 最终复审 R9（2026-10-21，needs_fix）

受审6063b7a94、r3；[报告/反例/冻结SD/Spec/输入身份](attachments/814/review-r9/review.md)。R8三项原例独立关闭，新F814-R9-01 P2 比较位强制as_str导致bool/int/反向比较组件E0277；02 P2 旧ui::style::IcedStyle及iced_adapter公开路径丢失；03 P2 新runtime声明的headless形态check失败（Iced适配器/桌面引用未按feature守门，不能只补首错）；04 P3 rust生成器新增格式失败。SDK隔离consumer5、新runtime5、API4、canonical1、provider43/runtime44、双端counter三状态、777与九档无丢ID通过不抵消失败；792旧实际收据24/24限定复用。T04/T05重开，T08/T09开放，T07复勾，executing8/15；原树work同批修齐、更新SD与边界矩阵后提交复审。A发布及既闭R5/R6/R7按边界保留。本登记是合同内未完成项，不授权延期/缩面；最终pass之后merge冻结门/沉淀/交付仍pending，未改实施/canonical/liveledger、合入/归档/清树。

### D-815-T04-R1 的 R3 关闭更新（2026-10-21）

受审 d0b7e6ebd/r3；[R3报告](attachments/815/review-t04-r3/review.md)。唯一剩余R1-06格式不变量独立CLOSED，原01..05保持关闭，无新增finding；单文件/净交付格式/diff-check通过，其他输入与R2证据逐hash未变，按批准范围复用。旧needs_fix段落为历史；T-04提前定向pass、executing10/17，不等于完整B/T-09终审或交付完成，外部门/merge-owned冻结门仍pending，唯一树保留。


### Plan 814 B 最终复审 R10（2026-10-21，needs_fix）

受审236827f1c、r3；[报告/稳定反例/冻结SD与身份](attachments/814/review-r10/review.md)。R9-02/03/04关闭；R9-01仅已知形状旧例闭，未知形状/Value prop负数与浮点五组合仍编译失败。新增F814-R10-01 P2 替换浮层扫描遗失后继position覆盖，absolute relative/fixed两组合误移浮层；02 P2 777 strict因runtime清单log std变化基线过期实际exit2（非新增禁向边）；03 P3 scroll_state_reader遗留serde两导入新warning。四包/六形态、旧路径/比较正控、runtime consumer5/provider43/定向65及t/tu无丢ID通过不抵消失败；其他证据按未变边界复用。executing8/15，T04/T05/T08/T09开放，T07与A发布/既闭R5/R6/R7保留。原树work修齐、同步SD与覆盖后提交，再采777基线送审；登记不授权延期或缩面，最终pass后的merge冻结门/沉淀/交付仍pending，未改实施/canonical/live ledger、合入/归档/清树。


### Plan 814 B 最终复审 R11（2026-10-21，needs_fix）

受审49206c699、r3；[报告/稳定类型组合/冻结SD与身份](attachments/814/review-r11/review.md)。R10-01浮层八组合0差异、02真实777 strict绿、03serde新warning清偿均闭；R9-01原五例虽编译全绿，分组浮点及声明int/bool/float操作数/Value prop七组合仍E0277/E0308，类型不变量继续开放。新F814-R11-01 P3 数字扫描if条件多余括号新增compiler warning，不能与runtime减少相抵掩盖。四包/两feature/正控/65定向通过；其他SDK/provider/选择/双端/发行证据限定复用。T04复勾，executing9/15；T05/T08/T09仍开放，原树work修齐类型环境与透明分组覆盖及告警、同步SD再提交。登记不授权延期或削减支持；A发布与既闭R5/R6/R7保留，最终pass后merge冻结门及实际B交付仍pending，未改实施/canonical/liveledger、合入/归档/清树。


### Plan 814 B 最终复审 R12（2026-10-21，needs_fix）

受审1bbf98620、r3；[报告/36组真实编译与冻结证据](attachments/814/review-r12/review.md)。R11-01新增告警清偿CLOSED；R9-01原27组全绿，但字段自身分组仍误进truthy，声明float算式左位仍按尾token发射f64（右位正确f32），新增9组合1过8个E0308/E0277，原类型保持finding继续OPEN/P2。四包/fmt/777 strict绿，overlay八组合0差异；其余未变证据限定复用。T04保持；重开T05/T08/T09，executing9/15，原树work修齐比较节点/分组/已有表达式类型，不以登记授权延期或削减能力。A发布与既闭R5/R6/R7保持，最终pass后merge冻结门与B实际发布仍pending；未改实施/canonical/liveledger、未整档/合入/归档/清树。


### Plan 814 B 最终复审 R13（2026-10-21，needs_fix）

受审feae28e0f、r3；[报告/57组真实编译与冻结证据](attachments/814/review-r13/review.md)。R12八败与原40检查全绿；替换解析器仍有<=/>=后向消费漏等号回归（loop/prop/分组六例）、bool取反操作数类型丢失（三例），合计57编译48过9败，R9-01仍OPEN/P2，无新增独立finding。已知shape同语法正控均绿，既有类型组合能力不得削减。四包/fmt/777 strict/overlay绿，其他不变证据限定复用。发行Spec新增M816-A节已对账无冲突，后续B增补须保留当前章节。重开T05/T08/T09，executing9/15，原树work收齐比较符与已支持表达式类型传播；登记不授权延期。A发布与既闭项保持，最终pass后merge冻结门与B实际发布pending；未改实施/canonical/liveledger、未整档/合入/归档/清树。


### Plan 814 B 最终复审 R14（2026-10-21，needs_fix）

受审42a4ef792、r3；[报告/96编译/六符双向24组与冻结证据](attachments/814/review-r14/review.md)。R13九败全部关闭、原64检查全绿，独立六比较符×两方向×loop/prop24/24通过。R9-01仍OPEN/P2：比较子表达式本身的bool结果未传播，`p.dirty == (.limit > 3)`及反向/prop/取反/逻辑组合五例E0277；同语法已知shape正控绿，96编译91过5败。四包/fmt/777/overlay绿；未变证据限定复用，R13及更早闭合项不回滚。T04保持、重开T05/T08/T09，executing9/15，原树work修齐既有bool比较节点递归传播；登记不授权延期或削减能力。Spec与R13当前版本对账保持，未跟踪复审证据按用户及仓规保留。A发布保持、最终pass后merge冻结门与B实际发布pending；未改实施/canonical/liveledger，未整档/合入/归档/清树。


### Plan 814 B 最终复审 R15（2026-10-21，pass）

受审6f1cac8cb、r3；[报告/全部受验编译与探索诊断/冻结身份](attachments/814/review-r15/review.md)。R9-01 CLOSED，原R14五败全绿、原99检查全过、六符双向loop/prop24矩阵保持；新增比较子树递归正控通过，Plan受验编译112/112。既闭R11/R10及更早边界保持；四包/fmt/777/overlay绿，其余不变证据限定复用。T10完成，reviewed13/15；T11/T12的master并集复审、冻结裸t+tv+tu+tt及必要触面、实际B合入/受管沉淀发布/归档清理仍pending，未交付冒称。

**D814-R15-01（非阻断债务候选，独立于本Plan类型修复）**：探索性escaped-quote条件三个编译均E0765，known/unknown shape同败；AURA expr_to_string把已解码引号直接拼回字符串。extract.rs与A实际发布f6e07acee整文件blob相同c0f7ab6f1e076308fecce29d143a9ec700025a90，原件/输入/真实失败日志已冻结；未跑旧A完整二进制。归因属另一条未改的字面量再编码能力，本Plan分包保持/合法下形条件类型修复不包含修复全部既有编码缺陷，不能将失败正控当成类型不变量反例或声称该能力已绿。后续调查上游转义并同时核查下游扫描器转义感知，改码后受影响内容另行验证；不授权削减语言输入，也不将任何已约定未完成义务转为债务。原始总矩阵115编译112过3探索败如实保留。

## D-829-R4：原型修复后完整不变量仍未闭合（2026-10-21）

PLAN-829 r1 @ `077de3b6697c69053c24f4067bf73188ca90ab62`，final R4 `needs_fix`；[报告/输入/同源正反控/截图及冻结SD](attachments/829/review-r4/review.md)。F829-R4-01..05：P2 死亡后首个敌人误投影成玩家（canonical1372、retry图可见但门绿）、Space死后无法retry、刚性旋转90°同碰撞几何产生不同死亡结果、snapshot漏新rotation；P1 全透明最终Iced合成仍以draw计数验绿（独立标题正控/反控均exit0、反控计数1→19→22）。90测试/36hash及6正式probe绿不覆盖这些反例。R1-F05/F07/F11的原例关闭不等于完整不变量关闭，source内容金帧实际冻结了死后错误角色；R2/R3历史PASS/替代oracle决定不能使合同内缺口成为非阻断债务。当前重开T01/03/06..10，保留T02/04/05，executing3/11；原树修复并补类型删除/联合方向/输入状态/透明合成诊断后提交复审。登记不授权延期、削减验收或自动批准新oracle；既闭原例与R2接线/consumer修复保留。未改实施/canonical/live ledger、未跑根整档/合入/归档/清树，最终pass后的merge-owned冻结门禁及迁移放行仍pending。

## D-829-R6：源坐标、合成观察与纯提交交付仍未闭合（2026-10-21）

PLAN-829 r1 @ `b84c6d91e83a00d4101e6e42403820127516c061`，final R6 `needs_fix`；[完整报告/原Godot source方法/像素/交付反例及冻结输入](attachments/829/review-r6/review.md)。F829-R6-01 P2 原Main.gd/Mob.gd在Godot4.4.1四次真实生成y0，AGP y1，误把父路径y1作为实际传入局部坐标；02 P2 当前截图分析器仍接受25,600个3D越界padding像素和80px整体视图错位，透明原例则已判红；03 P1 必需golden_frames/manifest.json被*.json忽略未入HEAD，纯git archive交付的committed读取器compare=None，审查fixture补本地ignored文件才Some(0,0,true)。97测试与本地3D probe绿不覆盖源正确性和提交闭包。R4前四项/3563px局部诚实重冻独立关闭保留；当前Iced live90s截止无报告，不虚构环境或产品归因，修复后须正反控。均为既有合同内缺口，登记不授权延期或改oracle自证；保留T02/03/04，T01/05..10开放，executing3/11，原树work补E16三维度诊断、修复完整提交后复审。未改实施/canonical/liveledger、未根整档/合入/归档/清树，merge-owned冻结门与迁移放行仍pending。
## D-815-T05-R1：真实工具生命周期定向复审七项开放（2026-10-21）

Plan 815 r3 @ `31a25f5cf0e40c42617140768cd2488df7b55257`，T-05 phase needs_fix；[报告/原始帧/真实状态反例/冻结SD与身份](attachments/815/review-t05-r1/review.md)。F01..07：P1 为 owned 子孙在 Cancelled+最终结果后仍活/TCP可连、合法大 stdin 卡住控制循环、真实 Iced 取消及 EOF 后不退出；P2 为 worker 输出 fatal 降成 exit0、Rust 取消终态误报 Failed、777 新依赖后基线过期、独立消费者旧能力断言两红。51 定向通过、格式通过不覆盖这些反例；独立消费实为2/4，777 strict fail。首个未到达 run 的缺字段探针已纠正并保留，受控安装/cargo 仅作生命周期负控，不冒充真实 UI 成功。T-05 重开、executing10/17；原树修齐、补真实 Launched/交互/取消/EOF 正控及验收输入并提交复审。此登记是合同内未完成项，不授权延期、缩面或将源码边界注记当许可。A/L 发布及 T-04 历史已闭范围保持；未改实施/canonical/liveledger，未整档/合入/归档/清树，T-09 与 merge-owned 冻结门仍 pending。

## D-829-R8：合成门每视图位置/底边与当前source oracle收口（2026-10-21）

PLAN-829 r1 @ `814f9bf01aa14eadcd9e851bc4cce7ac8caa360b`，final R8 needs_fix；[完整报告/真实图像正反控/可复跑探针/冻结SD与输入](attachments/829/review-r8/review.md)。F829-R8-01 P2：正式analyzer拒原R6反例但仅3D下移80px、底边156800天空色像素越界及正确裁剪的仅3D错位均`position_ok/clip_ok/ok=true`，位置仅盯2D顶行、clip漏底部外围，R6-02完整不变量仍partial。F829-R8-02 P3：当前sample-manifest/result仍写生成y1、feasibility仍death1402，与真实源fixture/driver y0及当前trace death2703冲突；实现/trace修复保留。R6-03纯提交manifest交付闭合，独立git archive→离线build→双游戏headless exit0/bit-exact；Godot原脚本四次y0、R4前四原例、45定向测试通过；required清单实际19项，不沿用R7收据24计数。T02/03/04/06完成，executing4/11；原树work修齐完整门与oracle消费者事实、补E17诊断后提交复审。登记不授权延期/缩面/重冻腐坏画面；未改实施或canonical/live ledger、未根整档/合入/归档/清树，最终pass后merge冻结门与迁移放行pending。

### D-815-T05-R1 的 R2 部分关闭与相邻不变量更新（2026-10-21）

Plan815 r3 @54aadefad（修复5c4dc2773），phase needs_fix；[R2报告/真实反例/冻结SD与身份](attachments/815/review-t05-r2/review.md)。原F02/05/06/07独立CLOSED；F01 owned子树原例已闭，非owned持管道时join超时静默detach、自有reader跨终态仍活，剩余P2；F03原已呈现cancel闭但启动清零吞本次cancel/EOF，四例复现，P1继续开放；F04 EOF后exit1闭，但worker fatal输入仍开放不退出，P2继续开放。新R2-01 P1串行第二个VM请求RecreationAttempt/exit101；R2-02 P3新增launched未读告警+consumer格式红；R2-03 P2为R1已指出的真实Launched/交互quit成功验收缺口编号化，不能以T-06注记延后。777 strict0/3、独立consumer5/5/闭包及生成/兼容正控通过；首次scoped就绪失败与DirectChildOnly LEAK负控原样披露。当前六项均合同内未完成，不授权缩面/延期或转非阻断债务；原唯一树一次修齐并提交复审，T-05未勾、executing10/17，A/L与T-04历史保持。新runtime公开API实际影响补登记SD-07候选/metadata，无新AC或语义降级；未改实施/canonical/live ledger、未整档/合入/归档/清树，完整B/merge-owned门pending。

### D-815-T05-R1 的 R3 五项关闭与三项未闭（2026-10-22）

Plan815 r3 @64ca649ff，T-05 phase needs_fix；[R3报告/原例与新反例/SDK ABI真实来源/MCP State与冻结SD](attachments/815/review-t05-r3/review.md)。R2六项中R1-F03/04与R2-01/02/03独立CLOSED，R1-F01旧reader残活已闭但stop误标自然EOF导致收尾错误漏报继续OPEN/P2；新R3-01 P1为VM隔离只传entry丢父进程外部后端/模块映射，真实SDK ABI装载成功后子进程Undefined greeting，本地同源正控State/快照有预期标记；R3-02 P2为初始化期有效InputAck q被run_session快照吸收，两个有效案例均被忽略，已呈现q正控completed。87定向Rust/777strict0v3g/独立consumer5/闭包/格式通过不覆盖这些反例；初版双方nil及未拿到Ack的探针原件保留且不计缺陷，最终使用合法.Init调用/有效Ack重验。原树一次修齐当前三项并提交复审，T-05未勾、executing10/17；登记不批准缩面/延期，A/L与T-04及已闭范围保持。未改实施/canonical/live ledger，未整档/合入/归档/清树，完整B/T-09及merge-owned冻结收口门仍pending。

## D-829-R10：受影响复审的两轴/内容裁剪、真实resize与截图完成（2026-10-22）

PLAN-829 r1 @ `1fa149873a578dcff44537eb383ee7ef05be6dc8`，affected final R10 needs_fix；[报告/原例关闭/独立截图探针/两次真实窗口/冻结身份](attachments/829/review-r10/review.md)。R8-02 CLOSED，当前文档local y0/death2703对齐，旧例历史有日期。R8-01三原图关闭但不变量OPEN/P2：两视图×左右80px正确裁剪四例仍全绿，25600个实际角色RGB(86,2,10)或地面RGB224越界像素全绿；只观察y extent/clear及sky颜色仍漏。F829-R10-01 P2修复回归：真实resize3D被约束为313高，新门按320误拒（旧/当前真实图片新门红、814旧函数绿，synthetic正控共享错误假设）。F829-R10-02 P2新观察既有路径：两次clean-HEAD真实window case均正常结束exit1，initial截图未收便finalize；第一次也缺resized，第二次采到resize且误拒；phase未绑定CompositeShot完成/窗口id就绪，未插桩具体callback遗漏调度点，不编造环境/GPU原因。48定向/fmt/check--tests/repair diff过，非完整107/10矩阵；R8源高度/提交闭包保持关闭。T01..06完成、executing6/11，T07..10开放；原树work修齐后新SHA再审，登记不授权延期/缩面/重冻腐坏画面。未改实施/canonical/liveledger、未根整档/合入/归档/清树；最终pass后merge冻结门与迁移放行pending。

## D-828-B1：第一批 A07 恢复/真实依赖/新增关闭拥有者缺口（2026-10-22）

PLAN-828 r3，第一批 phase needs_fix；原批6f482efaf，实际同树d93a9e373（后批叠加，运行证据仅绑定实际HEAD），[完整报告/12个独立控制/冻结身份与GLM任务单](attachments/828/review-batch1-d93a9e373/review.md)。B1-C01 P1：错误owner/参数编号等verifier拒的HIR穿过不完整canonical恢复，resident/history复用坏对象；C02 P2：实际span未核；C03 P2：C01-03真实body-fact查询/失效缺项；C04 P2：合法混合intrinsic编辑工件的局部callee编号用冷表误解，实际依赖存成mark_a而非mark_b，恢复仅采纳2/3；C05 P2：新增cold_module深副本在close后仍被session拥有。永久定向与既有边界/事务/关闭控制绿，独立artifact10中5败、合法roundtrip2中1败；ABC结果/trace仍正确，不编造执行错误或内存定量。原闭合不变量按未变/已测范围保持，新owner回归不能由旧Weak门豁免。均为合同内未完成项，登记不授权延期、缩面或债务化放行；原树work一批修齐，executing0/11保持，后批/最终/merge-owned冻结门仍pending。未改实施/canonical/liveledger，未整档/合入/归档/清树。


### D-815 的 T-05 R4 / T-06 / T-08 定向复审（2026-10-22）

Plan815 r3 @02401f77c（T-05修复50a743b7f、T-06 347d09d28），phase needs_fix；[完整报告/十项关闭oracle/真实SDK后端与MCP状态/有效正反控及冻结SD](attachments/815/review-t05-r4/review.md)。R3外部模块、初始化q及Windows持管道错误分类原例关闭；R1-01非Windows阻塞read_until/截止后无界join仍OPEN/P2，R1-07独立consumer能力/不服务负控随本轮扩展过期重开/P2，R2-02新增两warning/冻结格式红重开/P3。新R4-01..07：P1串行VM沿用前一外部后端、新Jet构建子进程绕过owned设施取消/EOF不收口；P2缺后端DLL仍completed、三生成profile输出根不存在、主pac第二真源、exit7仍completed、Multi漏后续目标却回报全部。根定向74/74(1既有负控LEAK)+Man6/6不覆盖这些反例，独立consumer实际3/5。T-08只有索引/源路径缺真实指纹/SD正文，准备稿partial；期间新增9ca1130f9仅T-07测试/fixture及格式WIP，生产零diff，行为证据限原冻结、不授予后加提交pass。Plan保持executing10/17，原树一次修齐十项/补真实矩阵与规范后再审；A/L及T-04与未变关闭范围保留，不授权延期/缩面/转非阻断债务。未改实施/canonical/liveledger、未整档/合入/归档/清树；T-07/09与最终pass后merge冻结/实际发布收口pending。

## D-829-R12：完整背景补集裁剪与冻结live收据（2026-10-22）

PLAN-829 r1 @ `1ff4ee67e06197180d4ab691c2d5764bdb6c5a61`，affected final R12 needs_fix；[报告/完整区域矩阵/旧例闭合/窗口状态与指纹](attachments/829/review-r12/review.md)。R8/R10旧反例全拒、真实resize回归CLOSED，53定向/fmt/check--tests/diff-check通过。F829-R8-01仍OPEN/P2：控件下方/2D上方gap31360px、上侧外围2480px、下角3200px（各sky/model/ground）及顶padding39200个原图实际RGB(24,38,52)，10个腐坏输入仍全绿；另9个已观测区域反例正确拒。ctrl_bot..v2_top漏段、frac内部列和padding视图y范围留角区、纯顶padding误归signature-only控制带，不能以修好旧例宣称完整mask闭合。R10-02代码屏障/就绪单测已验证，完整新SHA live闭合partial：R11两报告实际1fa-dirty且相对目录链接缺；本轮独立debug/fresh release各90秒仅留initial三图无报告，已定点回收自建树、未继续透明负控，不猜阻塞根因/不假称通过。原源高度/文档/纯提交交付关闭保持；executing6/11，T01..06完成/T07..10开放。work补完整背景mask并更正、补绑定live证据后新SHA再审；登记不授权延期/缩面/重冻腐坏图。未改实施/canonical/liveledger、未根整档/合入/归档/清树，最终pass后merge冻结门与迁移放行pending。

## D-828-B1-R2：原例全绿后内容身份与依赖归属仍未闭合（2026-10-22）

PLAN-828 r3，phase R2 needs_fix，冻结14c929a5d（基面d93a9e373、同树clean）；[R2报告/可重放反例/修复任务单及冻结输入](attachments/828/review-b1-r2-14c929a5d/review.md)。原12反例/合法控制全部通过，真实verify、实际span、v2跨表重绑及cold_bundle关闭释放成果保持；C03真实query已建但精确失效仍OPEN。B1-R2-01 P1：内容canonical漏Place.local，合法x=3→y=3增量3/冷编1、旧HIR/code被复用，同根合法但不同写目标工件过verify后误采纳；属上轮漏测既有要求，不是本修复引入。B1-C03 P2残项：consumer名级可覆盖/删除reads使重读刷新旧凭据、B remint清掉历史A依赖，旧对误hit；需每产物依赖冻结归属。B1-R2-02 P2：新query_body_fact漏closed守卫，close后Some且重建依赖/计数0→1。b1 4/session18/contract22/t01r4/r1 86/close6+3/session组合6全绿不覆盖这些反例；新adjacent5中4败、semantic2中1败，初版entry错误前提已校正并保留raw。先修机制与oracle再一批修齐，executing0/11保持；登记不授权缩面/延期/债务化放行，原已闭项不回滚。未改实施/canonical/liveledger，未整档/合入/归档/清树，后批/最终及merge-owned冻结门pending。


### D-815 的 T-05 R5 / T-06 / T-08 定向复审（2026-10-22）

Plan815 r3 @10e4e1cc1（code8a08e84fe），phase needs_fix；[报告/四项关闭oracle/真实ABI与MCP/捕获及PM/旧API正反控/冻结SD](attachments/815/review-t05-r5/review.md)。旧十项七闭：consumer5/5、修复卫生、串行上下文、缺DLL错误、目录、多目标、协议失败原例；三项仍partial/open：R1-01 P2 Unix新poll/有界join代码已补但cfg测试未执行，R4-04 P2包管理器同wire pkg=npm却按磁盘选npm/pnpm，R4-05 P1新ADB capture不检查cancel/stop/deadline（14s无终态、EOF后仍活，私有非owned holder释放后又报completed）。新R5-01 P2严格失败逻辑改变旧公开非托管Jetbuild返回，Ark/VSCode同因需对账。94定向Rust通过不覆盖上述失败/未验平台；14冻结Rust格式通过，bin ab88ccb4与交接一致，原T-08身份/正文缺口闭而保证与实现仍冲突。计划executing10/17，原树修齐四项/补平台与状态字段旧新入口矩阵后再审；A/L与T-04及未变关闭保持，不授权削面/延期或自动改变旧合同。未改实施/canonical/liveledger、未整档/合入/归档/清树，T-07/09与最终pass后merge冻结/实际发布收口pending。

## D-829-R14：冻结动画时钟的截止失效与旧live绑定（2026-10-22）

PLAN-829 r1 @ `36f520c104ad99e2eaed2e22a5a3f0bb93195391`，affected final R14 needs_fix；[报告/同源Tick重放/当前真实正负控/冻结身份](attachments/829/review-r14/review.md)。F829-R8-01完整裁剪CLOSED，19/19反例拒；旧几何/真实resize保持闭合；当前36独立release正控0、透明负控1且三时刻齐全。新F829-R14-01 P2：phase4冻结GRAPHICS.tick，phase5 age取同tick致180截止永不可达，原函数1000 Tick仍phase5/无失败，active对照181超时、实际CompositeShot可到phase6；boot缺窗口id1000 Tick亦无界。新F829-R14-02 P3：R13/E21标1ff的report正文含仅36新增f_top等字段，旧SHA绑定不成立，需dated更正；本轮36正确收据已另采，不推断build.rs/复制根因（两toy Git探索均正确dirty，不列产品bug）。57定向/fmt/check--tests/diff-check过，replacement失败对应旧句实际已改不列finding；executing6/11，T01..06完成/T07..10开放。原树修齐deadline clock/readiness与绑定叙述、保留冻结帧/几何/source正反控后新SHA再审，登记不授权延期/缩面。未改实施/canonical/liveledger、未根整档/合入/归档/清树；最终pass后merge冻结门及迁移放行pending。

## D-829-R16：时钟修复已验，缺窗口ID失败终止与result绑定更正仍未闭（2026-10-22）

PLAN-829 r1 @ `ae40d6052cdb7716c7cabe9e3f7e9edb1fe92d74`，affected final R16 needs_fix；[报告/逐字原函数/原生失败与实际正负控](attachments/829/review-r16/review.md)。F829-R14-01原冻结计时CLOSED：三个截图等待均181Tick具名超时，冻结graphics保持120/合法延迟回调到6；新window-id等待302Tick到99。但该ID缺失终态写报告/verdict1后TaskNone，finished后600Tick仍零退出动作，已知ID sibling有close；扣留Resize事件的同状态原生副本实际超时后Surface::present/Parent device is lost panic、exit101，无正常返回（不称20秒挂死、不推断GPU根因），故R14-01/P2 PARTIAL/OPEN。R14-02/P3 E21已正确CORRECTION，但result.md:113–118仍认证1ff旧live，PARTIAL/OPEN。当前ae40原CLI独立release正控0、transparent负控1且三时刻全绿/全红，57定向/19外围反例及全部旧几何正反控、fmt/check过；首轮探针绑定/编译错误保留但不算产品证据，校正后重验。executing6/11，T01..06保持/T07..10开放；原树补正常错误退出和result dated更正，覆盖到实际run返回后提交受影响复审，不授权延期缩面。未改实施/canonical/liveledger、未根整档/合入/归档/清树；最终pass后merge-owned冻结AGP门/真实consumer及迁移放行仍pending。

## D-828-B1-R3：原三项闭合，can3仍漏一个语义绑定标记（2026-10-22）

PLAN-828 r3，phase R3 needs_fix（仅余1项P2），冻结599cc15af、基面14c929a5d、同树clean；[报告/独立mutation与GLM窄修复单](attachments/828/review-b1-r3-599cc15af/review.md)。R2 P1写目标结果错误、C03历史/重读依赖残项、关闭查询守卫独立7/7 CLOSED；原12控制、b1 9/session18/contract22/t01r4/r1 86/cache20/close6+3/session组合6全绿保持；新增取消候选/迟到依赖正控通过。B1-R3-01 P2：can3未编码现有CallBinding.moved语义字段，false→true使HIR PartialEq不等但canonical相同，真实verify接受、restore3/3采纳、resident标记偏离冷源；实际失败exit101保存。未声称新i32输出错误，不新增Str/.move支持要求；当前内容身份/恢复合同缺口不能债务化放行。原树窄修复并补永久断言、冻结复审，executing0/11保持；不撤销已闭机制/后批提交。未改实施/canonical/liveledger，未整档/合入/归档/清树；后批/最终及merge-owned门pending。

## D-828-B1-R4：第一批窄修复通过与历史finding关闭（2026-10-22）

PLAN-828 r3，phase R4 PASS，冻结a66d9d9e5、基面599cc15af，同树clean；[收据/原反例重放/输入与身份](attachments/828/review-b1-r4-a66d9d9e5/review.md)。B1-R3-01/P2 CLOSED：can4将moved纳入内容身份，原合法mutation不再被当同内容采纳（2/3、resident=false），overflow显式渲染与旧行键隔离保持。R3原2、R2原7、首轮原12控制全绿，b1 11/session18/contract22/t01r4/r1 86/cache20通过；前两轮原B1-C01..05/R2-01/02在已声明标量/query接缝范围闭合，无新增阻断。这里只关闭第一批已审缺陷，不冲销全Plan原F09/10/11/12/14等后续责任；executing0/11保持。下一角色review第二批T05+T06完整受影响面，后批/最终/merge-owned冻结门pending；无整份reviewed/merge许可，未改实施/canonical/liveledger，未整档/合入/归档/清树。

## D-829-R18：剩余R14两项独立关闭，PASS / reviewed（2026-10-22）

PLAN-829 r1 @ `ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`，affected final R18 PASS；[报告/原函数与原生正常失败退出/当前真实正负控](attachments/829/review-r18/review.md)。F829-R14-01 CLOSED：missing-ID终态从TaskNone改iced::exit，单测/同源控制器有退出任务，原生扣留Resize反例具名超时→失败report/verdict1→native_loop_returned→exit1，非panic101/kill；已知ID正常close、冻结截图181Tick超时/合法延迟回调保持。R14-02 CLOSED：result与E21一致dated HISTORY/CORRECTED，旧1ff raw非commit-bound，真实36/ae40及当前ddb各自记录有效。58定向/格式/check通过；原CLI正控0三时刻全绿、透明负控重跑1三时刻全红；首轮click未观察到触发901Tick截止、后图缺失，保留raw且不作完整负控，外部原因未定。analyzer逐字未变13414字符、旧输入hash未漂移，19外围/旧几何与真实resize限定复用，无新增阻断或缩面。T01..10完成、reviewed10/11，T11仍pending；旧D-R14/R16为已关闭历史，既有非阻塞文案R5-03在沉淀时写清skipped-draw计数与透明内容门分工。未改实施/canonical/liveledger、未根整档或全AGP/合入/归档/清树；最终冻结AGP门/十probe/真实consumer、迁移写面交接放行与沉淀归档guard收尾尚未完成，reviewed不等于交付。


### D-815 的 T-05 R6 / T-06 / T-08 定向复审（2026-10-22）

Plan815 r3 @bb4f0a1ca（codec6fdaaaf1），phase needs_fix；[报告/当前两项/原例关闭/真实Unix与分类及环境对照/冻结SD](attachments/815/review-t05-r6/review.md)。R5四项三CLOSED：Unix独立冻结源码--locked --offline4/4、PM单源原例、旧公开Jet/Ark返回语义。R4-05取消/EOF原P1例已闭（290ms cancelled、holder释放前EOF0），剩余捕获超时/截断被Jet归false后completed/error=None继续OPEN降P2。新增R6-01 P2新PM矩阵在合法bun回退环境误判来源，exit101；真实工具无pkg/disk npm正确选bun，不能改产品正确规则让坏oracle过关。Windows98不同定向用例继承环境通过+Unix4/4不覆盖两失败输入；格式/指纹通过、tree clean。T-08旧缺口有进展但失败保证仍不符，T-07完整发行/资源/Linux覆盖及T-09后续；executing10/17，原树修齐两项后再审。A/L与T-04及未变关闭保留，不授权缩面/延期/自动接受坏oracle；未改实施/canonical/live ledger、未整档/合入/归档/清树，最终pass后merge冻结/真实发布pending。

## D-828-B2-R1：T-05/T-06第二批复审四项未闭（2026-10-22）

PLAN-828 r3，phase needs_fix，冻结plan-828-dev@a66d9d9e54e4db1de90edc078ed7d23d161e00d5、原批168b6b565/基面6f482efaf，同树clean；[报告与原反例](attachments/828/review-batch2-a66d9d9e5/review.md)、[同批修复单](attachments/828/review-batch2-a66d9d9e5/glm-next-round.md)。OPEN：B2-C01/P1同名异owner误绑定执行43；B2-C02/P1实际未闭合但ABC合法的import在MarkA后才Infra；B2-C03/P2真实编辑的code/maps配对重排后来源64..69误为33..33；B2-C04/P2六步程序连续16次Some(1)仍不进展。属于既定C02/V03/V04要求，登记不等于准许延期或完成。实际commit linkErr完整旧状态保持/good随后可提交、真实payload身份与副本反控、异构六排列值、旧链自递归互递归trace、关闭原6+3通过；第一批已闭不重开。下一责任work同树四项同批修复并永久回归，冻结后复审；r3/executing0/11，后批/整份最终/merge-owned冻结门pending，无merge许可。

D-828-B2-R1最终身份补记：测试期a66树clean；出收据后检测到并发session.rs新增Weak观测API与未跟踪T07测试，最终freeze明确FAIL（matches_baseline=false），HEAD/依赖未变。未修改/撤销该WIP；四项needs_fix只绑定已测冻结a66，不为当前WIP或T07授结论。见同目录concurrent-wip.diff、status、final-state与final-freeze；新冻结须处置/包含全部实际修改再复审。


### D-815 T-06～T-08提前定向R1（2026-10-22，与T-05同轮修复）

Plan815 r3 @bb4f0a1ca（codec6fdaaaf1），用户明确请求扩查，phase needs_fix；[完整报告/六项正反控与关闭oracle/待办及冻结SD](attachments/815/review-t06-t08-r1/review.md)。新T06-01/02(P2)：VM未消费有效非默认front声明、Ark run仅手工准备却completed。新T07-01/02/03(P2)：确定install7时dist成功测试仍PASS、大量正常stderr未排水导致验收阻塞、纯数据闭包门漏判实际UI文法/禁入lang-infra实现；04(P3)fixture格式与新告警未清。纯协议5/5只覆盖现有断言，缺四种服务端回执及深度负控；777现况strict唯一stale_baseline，792/完整fresh/locked/资源/字段矩阵承认未做，不能当新缺陷反复计数或提前勾选。T08指纹正确而真实运行/字段等保证仍冲突，partial；与T05 R6两项关联统一work修复，期间jet/pkg/proc_host并行WIP未获本轮pass。结论限原冻结/相关未变源与c95748c8实际bin，原R6/历史已闭范围保留；executing10/17。登记不授权缩面/延期/坏oracle通过，未改实施/canonical/live ledger、未整档/合入/归档/清树，最终完整review与merge冻结/发布仍pending。

## D-829-MERGE：PLAN-829 收口（delivered to autographics-dev；2026-10-22 用户裁定）

PLAN-829 r1 终收据 pass：R18 PASS 基线 `ddbde3e62857` 上 merge 门禁全绿（117/117+十探针 10/10+consumer 重定位 mismatches=[]，证据 attachments/829/merge-t11/gates.md）；**用户裁定不合 master**——代码落 `autographics-dev` 长期集成分支（merge `ab56c32ec`，AGP 子树与复审提交逐字节一致；830+ 沿用），master 侧仅知识文档（docs/specs/auto-graphics/ + ledger v65）。**遗留（非阻断，随 AG 路线后继计划）**：① `autographics-dev` 合回 master 的时机=814/815/816 系列（及后续）迁移全部完成后裁定（I2 迁移收据同时是裁定前提）；② Godot 角色级引擎画面对比 deferred（R2/R3 裁定，headless 导入限制，golden 以独立 Python 参考+常量+官方引擎轨迹语义为替代 oracle）；③ iced 窗口用例运行手册：同标题僵尸窗口会毒化 FindWindow，重跑前预检/taskkill 残留 graphics-probe/iced_spike 进程（T-11 门禁已内置预检）。计划已归档 docs/plans/archive/829-*。

## D-828-B2-R2：原三项关闭、来源残项与T07两项未闭（2026-10-22）

PLAN-828 r3，phase needs_fix，冻结cfa67eeceb4555e55b978354d3fd7ab508e80664，基面a66d9d9e5；[报告/实际原反例/三个残项控制](attachments/828/review-batch2-r2-cfa67eece/review.md)、[同批修复指示](attachments/828/review-batch2-r2-cfa67eece/glm-next-round.md)。B2-C01/P1、C02/P1、C04/P2本轮受测边界CLOSED；C03/P2原例绿但不同真实函数共享生产槽导致映射覆盖，仍OPEN。新增T07-R2-01/P2 manifest重复掩盖缺函数/未知格式仍matched；T07-R2-02/P2 manifest无required引用，独立soft根退役后实际GC删必需工件，warm0应3。独立projection失败1、T07失败3归2finding；其他原link8/execution4/permanent7目标/close9/core22/lifetime2过。T07内存Weak与计数盘点已测面通过不代表完整F09或存储GC收敛；两512/正式双线/workerIPC等最终门pending。末检发现并发T08 WIP，后跑cache-tier及组件重编不冒作精确cfa证据；未覆盖WIP，新冻结须纳入实际全部修改。登记不准延期或完成，下一责任work同树三项一批修齐；r3/executing0/11，未merge。


### D-815 T-05 R7 / T-06～T-08提前R2（2026-10-22）

冻结e3fdcdc76/code d039f2399 needs_fix：[完整报告与复放](attachments/815/review-t05-r7-t678-r2/review.md)。八历史finding四CLOSED（T05捕获失败/合法bun原例、T07闭包门/卫生），四PARTIAL（Rust front全链接线、Ark JSON5启动ID、dist引用oracle、lifecycle关闭有界），新增F-815-T05-R7-01(P2)新矩阵受legacy OnceLock缓存干扰——Windows/Linux pkg模块均7/8，单测合法bun通过不能外推模块通过。原取消/VM三目录真实呈现与777本分支strict0v/3g已验，不重复重开；当前五项均属合同内缺口，交work修齐，不能降成非阻断债务。完整T07发行/资源/字段/协议矩阵与T08/T09仍pending；Plan executing10/17，最终pass后merge-owned冻结门/实际发布待办。

## D-828-B2-R3：上轮三项关闭，新增manifest身份升级回归（2026-10-22）

PLAN-828 r3，phase needs_fix，冻结3090f2326f3595a69132c80bee3a2891c5f41b76/基面cfa67eece、直接修复基面8be89bf5d已含T08脚手架，入口结束clean/518输入一致；[报告与反例](attachments/828/review-batch2-r3-3090f2326/review.md)、[窄修复指示](attachments/828/review-batch2-r3-3090f2326/glm-next-round.md)。B2-C03/P2、T07-R2-01/P2、T07-R2-02/P2本轮原不变量及区分性GC/帧映射/格式集合控件CLOSED，前序已闭保持。新增OPEN T07-R3-01/P2：manifest required边改变artifact_id却沿用旧format1/bodyfp键；合法旧空边行阻断重开/partial冷自愈，完整旧ABC/sidecarwarm3后同内容编辑persist_error1，新缓存正控正常。迁移控件1pass4fail归一个修复回归，不吞Conflict/覆盖旧行/停持久化掩盖；下一work只修身份兼容，保留已闭机制，一次冻结复审。其他八永久目标/原控件/core/cache/组件全过；T08正式/双线内存/512/完整AC与SD/merge-owned门pending，r3/executing0/11，无merge许可；登记不是延期授权。


### D-815 T-05 R8 / T-06～T-08提前R3（2026-10-22）

冻结c050e718e/code e351cb2a needs_fix：[报告及复放](attachments/815/review-t05-r8-t678-r3/review.md)。前轮五项四CLOSED（Rust front全链、Ark生成ID、lifecycle关闭有界、PM缓存矩阵Win/Linux9/9），仅F-815-T07-R1-01(P2)继续PARTIAL：入口只引用存在CSS、完全不加载JS也通过成功产物oracle；旧缺文件/无引用负控已闭，不能把该残留降为非阻断债务。真实lifecycle3/3、777本分支0v/3g保持；完整T07发行/资源/字段/协议面及T08/T09/merge冻结门仍pending，Plan executing10/17。

## D-828-B2-R4：T07-R3-01原例关闭、不变量仍部分开放（2026-10-22）

PLAN-828 r3，phase needs_fix，冻结b846f45dc10abb580136ff97f0562db7c1f62e41/基面3090f2326，入口结束clean/519输入一致；[报告与有效正反控](attachments/828/review-batch2-r4-b846f45dc/review.md)、[下一窄修复单](attachments/828/review-batch2-r4-b846f45dc/glm-next-round.md)。旧format1→required format2迁移原五控制与永久mig6已过；T07-R3-01/P2完整不变量仍PARTIAL/OPEN，不新增ID：同format2下缺primary、合法位置变化、cold重建primary新ID，body逐字节相同而required目标不同，manifest仍占同format2/bodyfp键，Conflict阻断自愈；同缺块未移动源码正控成功。实际primary重建已证，错误repair键前提单列审核日志。下一work只修实际body+规范化refs的稳定身份，不再仅提高固定版本号；旧行不覆写/不吞Conflict/新根成功再退旧根。其余已闭及永久定向保持，r3/executing0/11，T08正式/双线/512/T09/整份最终/merge-owned门pending，无merge许可；登记不准延期或当完成。


### D-815 dist终项关闭（2026-10-22，受影响范围pass）

冻结c203d9642/test f8c01f76：[独立终核](attachments/815/review-t07-r1-final/review.md)。F-815-T07-R1-01/P2 CLOSED，五控valid/nested通过、缺文件/无引用/CSS-only拒绝，真实lifecycle3/3；前轮Rust front/Ark ID/close界限/PM缓存四项按未变输入继承，联合修复finding已全闭。本项pass不豁免完整T07发行/资源/字段/协议/Linux待办，不代表T08/T09或整Plan通过；Plan executing10/17，merge冻结门/真实发布仍pending。

## D-828-B2-R5：T07-R3-01完整身份不变量关闭（2026-10-22）

PLAN-828 r3，phase PASS，冻结cccb09b1f2b77e14cb410811eb4493247e2ed787/基面b846f45dc，入口结束clean/519输入一致；[R5报告与真实publisher/GC来源/原反例](attachments/828/review-batch2-r5-cccb09b1f/review.md)。T07-R3-01/P2 CLOSED：同规范化required集合既指纹入key又发布，原migration5＋缺块重建2全过；实际publisher3证顺序/重复幂等、目标/RefKind区分、缺目标保旧根；GC精确来源链1证body相同新root/旧row字节不覆盖、退soft根后warm3且trap func0/30..35同冷编。永久mig9与其他受影响定向面全绿，无新增阻断，前序已闭保持。这里只关闭已审finding，不冲销T07完整512/存储预算、F09双线/T08正式workerIPC与性能/T09/整份最终/merge-owned责任；r3/executing0/11、无merge许可。下一work按原Plan进入T08正式测量，见同目录next-step.md。


### D-815 T-07子批1定向通过（2026-10-22）

冻结f6fd0ce7c/code661c0573：[报告](attachments/815/review-t07-batch1-r1/review.md)。四服务端回执/帧122与123边界/real_tree Linux不可移植三条既有待办已独立核验：fixture7/7、原始帧两读入口正反控、Win proc_host9/9、Linux10/10+慢例1/1；无新增finding，旧关闭保持。仅子批1 pass；子批2发行(792/fresh/locked/失败恢复/消费)、资源许可/唯一源、完整字段兼容矩阵尚待work，777 master并集归T10。Plan executing10/17，完整T07/T08/T09与merge冻结发布均pending。

## D-828-T09：shim-metadata既有dead-code告警（2026-10-22登记）

PLAN-828 T-09 告警清理时发现：`crates/shim-metadata`（PLAN-596 面）bin 目标存在 4 个 dead-code 告警（`classify_all_third_party_mono`/`fingerprint`/`fingerprint_parts`/`SelfHandle`）与 98 处既有 rustfmt 漂移，均为 master 既有债、828 分支零触及（本批仅按 R3 复审点名修复 classify.rs:213 的 match 臂 `///` 文档注释告警）。归 shim owner 处置，不属 828 验收面。


### D-815 T-07子批2 R1：三个完整性/审计缺口（2026-10-22）

冻结503c9f67f/code e22fc7592 needs_fix：[独立报告/原始收据](attachments/815/review-t07-batch2-r1/review.md)。work两个实缺陷修复已核验，独立原792 Rust app25/25及Vue字段/删pac、真实lifecycle4/4通过。新增三P2：F-815-T07-B2-R1-01新tool发行/独立消费未被旧app driver覆盖；02资源审计误删实际runtime Inter/OFL并错述源资产随包路径；03完整逐profile字段/适用边界/生产oracle仍缺，Vue静态正例不替代。均属原T07验收范围，不可作为非阻断债务；旧closed与子批1pass保留。T07/T08未勾，Plan executing10/17，777 master并集/T09/merge冻结发布仍pending。


### D-828 第三批复审：五组原合同缺口（2026-10-22）

冻结8a18db450 needs_fix：B3-C01/P2合法A07拒绝恢复后的HIR root泄漏；B3-C02/P1正式judge未绑定当前revision/准确安装目标；B3-C03/P1实际执行副本未触发正式结构门；B3-C04/P2计时提前结束与必需observations/raw缺失；B3-C05/P2后继API/Spec生产路径文字及计数不实。均为T07/08/09原合同阻塞项，不是延期许可；已闭前两批保持。原512正控及30轮数值/预算绿不覆盖这些反例。[独立报告与一次性修复任务单](attachments/828/review-batch3-8a18db450/review.md)。额外性能扩样/专用漂亮报告可后继，原§5.5观测及成功失败raw仍required；最终与merge-owned门均pending。


### D815-T07-B3-R1 — 原字段矩阵P2未闭（2026-10-22）

PLAN-815 / AC02/04 / T07/T08 / reviewed9aee8659d：子批3定向复审needs_fix，原B2-R1-01新tool发行和B2-R1-02资源事实已CLOSED（本轮独立33/33、资源377文件全等）；仅原B2-R1-03 PARTIAL。field-profile-matrix.md将Rust窗口/标题、VM runtime.api、Vue default_classes误列为无生成/运行消费，dep_roots还漏Rust；真实包工具两组窗口/标题生成物与默认class开关对照证实字段生效。回work修矩阵事实/适用列/证据映射，不改变产品来迁就错误文档；合同内验收缺口不能降为非阻断债务。保留全部有效已闭证据，不重跑整档；T08最终身份/SD、T09整体与T10 merge门另待。详[报告与共享回放](attachments/815/review-t07-batch3-r1/review.md)。Plan executing10/17、未合入/归档/清树。


### D-828 第三批R2：原五项收敛为C03/C04两组残项（2026-10-22）

冻结f4d7a9da6 needs_fix：B3-C01/C02/C05原具体错误CLOSED；B3-C03/P1仍缺与真实AV消费绑定的身份/代际/内容证据（run内实际副本、缺main观测误绿；正式worker合法当前内容命中误红），B3-C04/P2仍在失败assert后才落盘且未补阶段wall/IO。是原合同残项与修复回归，不是新增可延期债。原六控制/两root、独立512和正式30轮数值/预算均过，不覆盖上述反例；前两批/F05b保持。[R2报告/两组关闭条件](attachments/828/review-batch3-r2-f4d7a9da6/review.md)。下一责任work同树一批修齐再冻结复审，T10及merge-owned门pending。


### D815-T07-B3-R2 — 原矩阵验收缺口 CLOSED（2026-10-22）

对D815-T07-B3-R1及原B2-R1-03的闭环：冻结afd1a66bf定向复审pass，Rust窗口/标题、VM api、Vue默认class与Rust dep根的实际消费分类已更正，主题/VSCode/sidecar载荷及余携带行映射重核可接受；两枚新增永久钉独立各1/1，原33/33发行/377文件及实际效果对照按未变源/依赖/配置复用。原三P2现均CLOSED，无新增finding；不是把合同缺口降级为债务。T08实际产物身份/SD定稿和T09整计划终审、后续merge门仍pending；不提前勾T08或标全计划reviewed。详[复审报告](attachments/815/review-t07-batch3-r2/review.md)。


### D815-T05-T08-closeout — 联合任务收口通过（2026-10-22）

冻结a67f114fd，T05～08整任务联合复审pass并四项勾选，Plan executing14/17；无新增finding，当期修复均已闭合。T08实际release身份与四组协议/资源probe通过，归档收据与此前快照同字节，旧独立证据未随scratch删除而丢失。原已披露shell_pack/plan734及全库诊断不冒称已清，T09按原合同作整体裁定；777 master并集/冻结整档/实际合入和Spec ledger归档等仍归T10/T11，尚未正式发布M815-B。详[联合报告](attachments/815/review-t05-t08-closeout-r1/review.md)。


### D815-T09-R1 — shell公开再生器合同内阻断（2026-10-22）

冻结a67f114fd，T09独立final review needs_fix：**F-815-T09-01/P2，AC01/04/05，T06/T07/T08重开，Plan executing11/17**。原P751-D2的shell freshness分项经新实证为真实产物问题：公开generate_shell_pack_lib_sources使用当前canonical组件却发仅auto-lang的Cargo，实际原样消费107个E0433；入库canonical库check0、同私有生成源只加两SDK直依赖check0；源/manifest调整仅私有诊断，未修实现。五件声明冻结可重放，无需写auto-os或建第二实现树。原app/dev canonical修复、33/33、377文件与生命周期通过保留；本次不把这些原例重开。plan734 exact fixture仍红，但归一排版相同且上游814提交2aa0529cae引入，仅非阻断卫生债，不能把其报绿或类推shell可忽略。修F01后刷新T08冻结身份并受影响复审；T10/T11收尾门仍pending。详[终审及正反控](attachments/815/review-t09-r1/review.md)。


### D-828 第三批R3：C03关闭，仅C04两处窄残项（2026-10-22）

冻结d5a4f3147 needs_fix：C03/P1已独立关闭，其他旧闭合保持；仅C04/P2剩ready前EOF/坏JSON由`?`提前Err无可保存report，以及store_write_bytes对HIR载荷重复加数（fresh四Ok实际903B、上报1155B，多252B）。原错值失败落盘与阶段wall已交付，不再重开。两处都是原收据/真实IO合同的残项，非延期许可；同树窄修、一批冻结后C04窄复审，通过才T10。[R3报告/两处关闭oracle](attachments/828/review-batch3-r3-d5a4f3147/review.md)。整份及merge-owned门pending。


### D815-T09-R2 — F01独立闭环，Plan reviewed（2026-10-22）

冻结ca9e89b44/code0f2858e35，T09 final pass：F-815-T09-01 CLOSED，无新增finding。五件同hash冻结输入真实生成的库/原生Cargo与入库逐字节等，未补依赖的消费check和locked均0，freshness原样1/1，入库check0；新release a73b117f…及四组消费4/4。T06～08复勾、T09完成，execution_done14/17→reviewed15/17；T10/T11仍pending、未合入/正式发布/归档清树。旧债归类更正：plan593不属于plan734，未沿用work的错误归类；plan593/merged_api独立分项继续留账，不能报作已清偿或全库全绿。详[最终报告及merge-owned待办](attachments/815/review-t09-r2/review.md)。


### D-828 C04窄复审：只剩早退丢已接受字节（2026-10-22）

冻结d16af9cb3 needs_fix：握手收据与成功路径903B原例已闭，C03/其他旧闭合保持；仅C04/P2接受计量失败出口未闭。后续HIR/sidecar put失败时，前面真实Ok的173/425B还在局部变量，随Err丢弃，上报0；首写失败0和manifest失败497正控保持。是前轮“相关失败出口不丢已发生计量”原条件残项，不是延期许可或新增功能。[单项窄复审报告/4出口原始控制](attachments/828/review-c04-r4-d16af9cb3/review.md)。只修共因计量落点，冻结再单项复审，PASS后T10；整份及merge-owned门pending。


### SDK-833-MATH-CONVERSION（候选，非833合同缺口）

- 来源：PLAN-833 r2 T-02 外部公开SDK探针，固定SDK `9576b4dad5ce`。`Math.sqrt` 返回double；探针中的 `as float` 未得到期望f32数值/标签。原始对照/失败与原因在 `attachments/833/work-r2/initial-isolation-probe.rs.txt`、`isolate.txt`、`sdk-driver-decision.md`，正式回执按主Plan §9引用。
- 833的冻结归一化改用有限Auto f32算术并验证初速度位型与完整原golden，无语言/VM修改、无Rust玩法替代或容差放宽。833不声称支持通用double→float/native数学修复。
- 归属：language-sdk / VM owner 后继核实并修复通用转换；非本次输出/玩法未完成项。当前所用标量/容器、边界与数值分母均有实际验收。

### PLAN-833 R1 — 新增文字叠层的归属与上传计量缺口（2026-10-22，R2已关闭）

- 受审r2/`b136129fed9a018f07ea49445868066449eaf5e9`，独立final **needs_fix**；计划回executing 6/10，重开T-07及依赖证据T-08。登记仅用于追踪，属于原AC-06/07合同缺口，不能降格为后继债务或验收豁免。
- **F-833-R1-01/P1**：AGP `SurfaceContent::device_ordinal()`在quad/text不同时用text ordinal覆盖quad；foreign quad+local text初次准入/可变替换后逐帧校验均漏拒绝。矩阵实际GPU路径panic；独立replacement可直接frame=Ok。应逐资源证明一致，在GPU工作之前拒绝并保last-good；补双向foreign/raw与合法控制。
- **F-833-R1-02/P2**：新TextOverlay的camera 64B与glyph 48B/个未进入content/runtime真实上传计量。五glyph正常绘制额外304B，报表仍与无文字同为1 rewrite/64B。应在真实write_buffer点计数、纳入各级汇总并用省略文字计量反控验证，更新受影响V-07与交接/SD。
- [完整报告、原始正反控和可重放输入](attachments/833/review-r1/review.md)。现有16项验收+57项指定兼容独立通过，不能抵消这两项；work在原树一批修复并冻结后再复审，完整merge-owned门仍pending。


### D-828 C04单项复审PASS：已接受字节早退丢失关闭（2026-10-22）

冻结a030427fe：B3-C04/P2 CLOSED，无新增finding；四个真实逐写故障0/173/425/497全部准确，903/no-store与握手/C03代表控保持。第三批已登记finding全闭，旧C04残项记录转历史，不再作为当前OPEN。下一责任review/T10整份最终验收；当前phase PASS不能合入。[PASS收据及原始证据](attachments/828/review-c04-r5-a030427fe/review.md)。历史一次战役失败具体诊断缺失，负载归因未独立确认，T10如实保留证据限制并逐次留存后续raw；整份/merge-owned门仍pending。


PLAN-833 R1两项finding修复进展（非独立复审结论）：F-833-R1-01/02已在6c47b79b1修复，冻结HEAD77302d855；双向设备/raw/可变替换及实际文字计量回归、反控与57兼容通过。实施侧无剩余合同缺口，待独立复审确认；证据见[work-r3 readiness](attachments/833/work-r3/readiness.md)。原R1条目保留审计历史，非延期许可。

PLAN-833 final R2独立结论：**F-833-R1-01/P1与F-833-R1-02/P2 CLOSED**，受审`77302d8552f074ebedd53e757acfcf3db0bde230`、r2，未新增finding。相同原反例逐hash重放：双向foreign/替换提前拒绝；5glyph=3/368B、增量2/304B。6图形与57兼容独立通过，正式计量oracle/负控和完整设备矩阵已核。计划reviewed9/10；旧OPEN/work待审措辞仅为历史，完整merge-owned门与T-10仍pending，未合入/归档/清树。[R2完整报告](attachments/833/review-r2/review.md)。SDK-MATH-CONVERSION独立上游候选保持。

### PLAN-817 final review R1 (2026-10-23)
F-817-R1-01..05 为合同内阻断项，非可延期债务：多callee错误执行、正式编辑路径不保留session、性能路径/负载不符、空样本门禁假绿、Meta→AV组合缺失。证据及关闭oracle：[R1](attachments/817/review-r1/review.md)。返回work，不得以登记债务代替完成。

PLAN-817 R2（2026-10-23）：F-817-R2-01 cold snapshot首bool被误报类型错（真实worker复现）；R2-02未提交/漂移/规范和健康交接未完；R1周转与V-12覆盖仍未完整关闭。均为合同内阻断，不能延期债务化。见[复核](attachments/817/review-r2/review.md)。

PLAN-817 R3（9fe341e61，2026-10-23）合同内阻断F-817-R3-01..04：正式报告638MiB超worker门仍绿、驻留时间实测失败且gate漏判、端到端窗口/负载遗漏、benchmark语义oracle与退化失败传播不完整。冷bool已修。详见[审查与关闭oracle](attachments/817/review-r3/review.md)，不得延期债务化。


### PLAN-836 final R1 — 原生验收未完成成功退出（2026-10-23）

冻结87fb0b051、r1，独立final needs_fix。F-836-R1-01/P2（AC-05/07，V-09/10，T-07/T-08）：`--interaction-check --frames 1`及两种负控均exit0、passed=false、0records；after-run保护仅覆盖旧window-probe。原验证合同内阻断，不能登记后延期豁免。22定向+4独立反例、物理拖动/Undo和新进程reopen保持通过；自动原生普通正门本轮有额外桌面输入干扰，原始失败如实保留，未确诊为产品回归。计划回executing6/10，原树修复并提交/刷新交接后受影响复审，merge整档仍pending。[R1报告、稳定反例和关闭oracle](attachments/836/review-r1/review.md)。


PLAN-836 final R2（2026-10-23）：F-836-R1-01/P2 **CLOSED**，冻结5f3e218ba、r1，无新增finding。原3截断与legacy/观察/启动失败永久控fresh通过，完整native正门phase27/6records通过，原wrong/drop负控、部分phase3截断、组合截断、保存后新进程重开均符合退出/report/内容断言；R1库证据仅在未变输入范围复用。共同完成性守卫先于report，原错误保留，未以降低取消规则过门。Plan reviewed9/10；旧OPEN/needs_fix为历史，完整merge-owned门及T-10仍pending，未合入/归档/清树。[独立R2报告及证据](attachments/836/review-r2/review.md)。

PLAN-817 R4（f2221f50d）：F-817-R4-01..04合同内阻断：worker/tree误采父进程、暖IPC时间不含准备/父判、oracle漏HIR/来源/逐步核查、owner-only删除合法native入口（真实case-serve复现）。驻留P95与退化反控本轮已过。详见[审查](attachments/817/review-r4/review.md)，不可延期债务化。


### PLAN-837 final R1 — 层级原生门缺少移动后的合成像素断言（2026-10-23）

冻结`abe682c5a1144f7e9b72f4ffeb95ece19dd02841`、r1，独立final **needs_fix**。**F-837-R1-01/P2**（AC-05/06/08，V-10，T-07及依赖T-08）：native hierarchy在移动后只核world/pick/GPU命令并记录hash，未将实际终点像素与初始叶色、原位置与背景比较；同错图像的新进程reopen比对可过。审查专用副本不改实施源码，将真实GPU圆形变透明并断言其合成像素已为背景，原探针+完整原聚合脚本仍phase16/204frames/exit0/passed=true，配对reopen也过。普通未变异产品画面与38项定向/check/fmt/source均通过，不能抵扣必需oracle缺口。关闭须让同一门实际断言终点叶色/原位置背景，真实透明/错位绘制反控触发report/exit/script失败，保留原正/错父/漏释放/partial/reopen，并刷新身份/文档/SD-05交接。此为原合同内阻断项，登记不是可延期债务或验收豁免。Plan回executing6/10，保留T-01..06，原树修复提交后受影响复审；完整merge-owned门仍pending。[独立R1、可重放输入及raw证据](attachments/837/review-r1/review.md)。

PLAN-817 R5（eb8f9bc03）：F-817-R5-01..03合同内阻断：tree及spawn内存采样漏、Meta性能负载擅缩、结果oracle/变异证据不完整。owner-native入口与暖IPC计时旧问题已修。见[R5](attachments/817/review-r5/review.md)；不可延期债务化。


PLAN-837 work repair — F-837-R1-01（2026-10-23）：**WORK-FIXED / 待独立复审**，修复2ffc92d34143bc97a98f2684406160b183fd2ff3。真实top终点/原位置像素覆盖7状态，原R1透明GPU反例与永久透明/残影绘制故障均native phase9/report false/非零并传到aggregate；正常/旧负控/重开通过。最初像素主张已纠正，source247及独立consumer与定向检查完成，未扩SDK/AGP/库/产品面。旧R1 needs_fix保留历史，work自查不是独立pass，不作债务延期。计划execution_done8/10，原树等待受影响复审，完整merge-owned门pending。[修复闭环及证据](attachments/837/work-repair-r1/readiness.md)。


PLAN-837 final R2（2026-10-23）：**F-837-R1-01/P2 CLOSED，无新增finding**，冻结`2ffc92d34143bc97a98f2684406160b183fd2ff3`、r1。原R1反例仅重绑HEAD guard后独立重放，真实透明终点与残影GPU永久控也重跑，均phase9实际像素拒绝/native1/reportfalse/aggregate1；正常7状态phase16/204frames、旧parent/release/partial负控与保存后新进程精确重建通过。定向3项/check/fmt/source247/SDK680/closure479/17locals/29edges通过；191项库/测试/样本/AGP输入相对R1未变，原38项独立证据按指纹复用，不复用旧像素主张。当前Plan reviewed9/10，旧OPEN/WORK-FIXED/needs_fix仅为历史；完整merge-owned门和T-10实际合入/知识/归档/清理仍pending，未合入/清树。[R2独立结论、关闭证据与冻结Spec](attachments/837/review-r2/review.md)。

### PLAN-837 F-837-R1-01 — CLOSED / delivered

2026-10-23：独立final R2在2ffc92d34关闭原合成像素门漏检，无新增阻断；冻结完整driver80/AGP162、16官方case、七状态像素、真实透明/残影GPU反控及全新consumer门通过。源码已落autographics-dev cf2e99f50，规范/ledger74及归档、guard清理完成，archived10/10。先前WORK-FIXED待复审记录作为历史保留；本项无后续债务。见[合入收据](attachments/837/merge-r1/receipt.md)。

PLAN-817 R6（c7ec7c076）：R5三P1已关闭；新增原合同资源阻断F-817-R6-01（合法32query=22.2s仍成功，累计10s无门）、R6-02（编码/恢复/IPC解码先分配后预留）。见[R6反例和关闭oracle](attachments/817/review-r6/review.md)，不能债务化延期。

PLAN-817 R7（d640fbce4，2026-10-24）：R6-01原累计deadline反例已闭合；F-817-R7-01仍阻断R6-02完整恢复预算：预留1MiB实际loader返回2MiB（生产接口真存取反例），driver仅在分配后拒绝。见[R7](attachments/817/review-r7/review.md)，不可延期债务化。

PLAN-816 B-R9（2026-10-24）：exec_lifecycle **Linux 面预存红 ×3**（非 B-R9 diff 引入——F-816-B-R8-01 修复使该文件首次在 Linux 可编译实跑而暴露；pre-fix 代码 + 仅一行编译修复复现同样失败，[attrib2 收据](attachments/816/repair-r9/attrib2-result.json)）：early_spawned_descendant_belongs_to_owned_tree / root_exit_terminates_pipe_holding_descendants / cancel_tree_scope_kills_grandchild。历史收据只有 `cargo check`（不含 --tests），T-02 的 Unix 进程组树终止语义（pre_exec setpgid + killpg）从未在 Linux 实测过；Windows 面（Job 对象）恒绿。属 T-02 Unix 树终止真实行为的验证债，处置（修复 Unix 语义/平台门控/声明边界）待裁定；同文件预存 E0308 编译错（creation_flags_are_windows_only，T-06 引入）已在本轮顺带修复。 **R9审查更正**：已独立确认真实Unix树终止实现缺陷F-816-B-R9-01/P1，属于本PlanAC02/T02必修；原“待裁定/预存债”仅保留为工作归因历史，不作为延期或平台豁免授权，当前以整体R9报告为准。 **整体R10独立闭合**：F-816-B-R9-01已在6d15d25a4验证CLOSED，真实Unix树/信号/消费者正反控通过；当前仅T08既有终稿收口待完成，详整体R10报告。

PLAN-817 R8（2552597ff，2026-10-24）：R7完整恢复不变量尚未关闭。R8-01：SQL小长度、损坏CAS文件2MiB时allocator实测读分配2MiB，reserve仅1072B；R8-02：probe吞SQL错误为clean miss。见[R8](attachments/817/review-r8/review.md)，不可延期债务化。

PLAN-816 B-R10 更正（2026-10-24）：上条"exec_lifecycle Linux 面 3 预存红"经 R9 整体复审升级为 **F-816-B-R9-01/P1 必修**（非可豁免债——Unix 树终止属本 Plan AC-02 合同范围）。已在 6d15d25a4 修复：unix_pg signal_group/kill_group 改负 PGID 组信号；Linux 全 lifecycle 26/26（三红测转绿）+组送达永久回归。原条目所记"预存债待裁定"由本修复收口，待 T-09/T-13 复审确认。

PLAN-817 R9（cb0ba5231，2026-10-24）：R8 SQL错误和2MiB旧损坏反例关闭；R9-01仍阻断恢复预算：take限制len不限制Vec capacity，近界1025B文件reserve1072却allocator实际2048，正常frame1024+payload512共存1536亦超1072。见[R9](attachments/817/review-r9/review.md)，不可延期债务化。

PLAN-817 R10（e55b59cae，2026-10-24）：R9精确容量读取/分账预留实质闭合、20项定向绿；原AC-05编解码不变量仍有合同内阻断：R10-01/P1，301146B正确CAS坏信封恢复先留602292B、实际Vec单次3145728B（901146B例达12582912B）而managed_target_met=true；R10-02/P2，正常AV envelope编码预留1163B却实际capacity2048B；R10-03/P2，永久allocator窗口未重置峰值、结束前改基线且8192B容差放过旧错误算法。独立真实driver/allocator重放见[R10](attachments/817/review-r10/review.md)。三项均不可延期债务化；保持executing6/11，回work同树修复。

PLAN-817 R11（b29cb2348，2026-10-24）：R10-02编码容量、R10-03观测器及原空串数组反例已关闭，22项定向绿。R10-01完整解码预算仍有合同内阻断F-817-R11-01/P1：132166B正确CAS坏信封的审计Hot额度268780B，带单次转义的长字符串经scratch与owned String共存，decode峰值393995B；真实driver仍managed_met=true，plain配对正控界内。该例证明decode/Hot预留不足，未声称默认32MiB总量实测超限；应显式覆盖scratch容量/归属/寿命，不能假定复用另一分区frame额度。见[R11证据和关闭oracle](attachments/817/review-r11/review.md)。不可延期债务化；保持executing6/11，同树work修复。

PLAN-816 T-10（2026-10-24）：**master 预存编译破损（cargo test 全目标面）**——`cargo t`/`tf`（nextest 档）结构性不触发、日常门不受影响，但裸 `cargo test`/`cargo check --tests`（全目标）红：①`crates/auto-gen/tests/common/mod.rs:19-20` 引用 `auto_lang::atom::Atom`，而 auto-gen 依赖自 814 T-06（9c7e44855）已切 auto-lang-core、无 auto-lang 依赖（E0433，master 主检出同红，非 816 引入）；②`crates/auto-cache/src/methods_pack.rs:697` ShimMethod 初始化缺 `trait_name`（shim-metadata 结构演进未同步消费者，E0063，master 同红）。两处均为计划合入面外的预存债，修复归后续 fix 计划裁定。

PLAN-818 D828刷新轮（c4819bf58，2026-10-24）：**ac-syntax workspace 跨兄弟消费缺陷（路由 ac-syntax/828 owner）**——828 为 auto-av-probe 新增 `ac-syntax-core-adapter` 路径依赖（该 crate 及 runtime/grammar/artifacts 全部用 `.workspace=true` 继承）后，`experimental/` 内任何兄弟 workspace 经 path 链消费 auto-av-probe 时，cargo 将这些成员的继承解析到 auto-lang 根（差分实测：根临时补 edition 后错误推进到 publish；树外 /tmp workspace 消费同链正常）而根从未定义 workspace.package——跨 workspace path 依赖链在 828 布局下不可构建。PLAN-818 以语义等价修复临时解锁（四成员清单改字面量，值与 ac-syntax [workspace.package] 逐字一致；ac-syntax 自建 `cargo test --workspace` 39/39 全绿零回归）。路由事项：owner 侧要么正式接受字面量形态，要么以不破坏兄弟消费的方式恢复继承（如 [patch] 或 workspace 重组）；在 owner 裁定前该四文件为 818 分支携带的跨面修复，818 合入时随计划一起合入。

PLAN-817 R12（4309f49de，2026-10-24）：F-817-R11-01已关闭，3项受影响定向及独立三档escaped/plain/其他String字段反控通过，T-05完成。剩余F-817-R12-01/P2仅为T-08/T-11交付覆盖缺口：acceptance-matrix.md只有6行（空行+R7..R11历史），缺既定V-01..V-12/AC/task/证据映射。补齐索引优先复用已有证据，无需重跑全局/资源重族。详见[R12](attachments/817/review-r12/review.md)。executing7/11，统一门禁仍merge-owned pending，不能以债务化方式跳过收口。

PLAN-817 R13（97203795b，2026-10-24）：无新运行时缺陷；覆盖矩阵结构/V-12四子落点已恢复，脚本6/7/18列表选择和六组路由/失败传播正反控通过。T-08仍有两项P2收口：F-817-R13-01，closeout将已认可的783追加API误判WRITE VIOLATION（真实exit1）；F-817-R13-02，V-08测试名缺cte前缀、V-09性能摘要与冻结raw不符。见[R13](attachments/817/review-r13/review.md)，仅需只读审计边界正反控及文档证据比对，无需全局测试。T-08重开，保留T-11完成，executing8/11，不能以登记债务代替收口。

PLAN-817 R14（c224a4b0b，2026-10-24）：F-817-R13-01/02均已独立关闭。真实closeout exit0；783单文件+120/-0与授权diff SHA匹配，六项输入边界控制通过；82入口可解析、冻结性能摘要一致。无新增finding，final pass，reviewed10/11，仅T-10开放。统一回归仍merge-owned pending，门禁通过前不得合入/宣称交付完成。见[R14](attachments/817/review-r14/review.md)，本轮无Rust测试/全局回归。


### PLAN-840 review R1（2026-10-24；合约内阻断，非获准延后）

- F-840-R1-01：callback Unit检查未覆盖初始化/一般表达式内return，实际非Unit回调被准入并发布；T03/T05待修。
- F-840-R1-02：immutable常量引用及纯常量FStr/backtick被误拒，违背不缩原支持面的合同；T03待修。
- F-840-R1-03：Spec proposal SD01/02/04与批准Plan稳定编号/目标映射不一致；T08待修。
- 以上均不能通过记债判定完成。当前executing7/12，完整合入门禁仍待最终review pass后执行。复现/关闭oracle/验收映射见[PLAN-840 review R1](attachments/840/review-r1/review.md)。


PLAN-840 R1修复（2026-10-24；work已关闭，复审待定）：F-840-R1-01/02/03修复提交`1f19395254d7e676bd188e43e3d8f734d57aaa12`；共享AST返回策略/合法常量准入/稳定Spec目标映射和原反例/相邻控制均通过。原R1记录保留为旧提交的缺陷事实；当前execution_done10/12，需受影响独立复审后才可执行完整合入门禁，并非已交付。详[修复关闭证据](attachments/840/work-r3-repair/closure.md)。


PLAN-840 R2受影响复审（2026-10-24）：在冻结提交`1f19395254d7e676bd188e43e3d8f734d57aaa12`核验F-840-R1-01/02/03全部关闭，未新增合同内阻塞；原10case、行为12、实际Host/GPU修源恢复1、独立相邻8和Spec5ID/8目标通过。原支持面不减，当前reviewed11/12；完整合入门禁仍merge-owned pending。当前实施会话无独立上下文隔离，限制及未变证据复用边界见[复审报告](attachments/840/review-r2/review.md)。

### PLAN-818 R1 (2026-10-24): needs_fix
F-818-R1-01..04 OPEN，均为合同内必修：增量计时漏编译、历史命中全模块回退、正式recipe未接增量会话、fmt失败。见 [复审证据](attachments/818/review-r1/review.md)。不得记债降约或合入；T08/T11重开，7/11。


PLAN-840 merge（2026-10-24，archived11/12）：R1三finding已复审关闭；冻结1f的完整63入口、root5332/163、producer与fresh consumer各13原生场景及原有16AGP/旧游戏/创作/资源/层级/Play/owner/counter+content全分母通过。源码已快进autographics-dev@8b62db5cb；master仅SDK四文件精确补丁0676dc133，规范/ledger78→79已读回。归档知识提交/fresh guard清理/840到期批量回归仍pending，暂非最终merge pass。AGP2x原输入辅助失败以PID绑定DPIaware真实OS输入夹具完成相同原断言，未改AGP/压断言/缩面；后续自动点击辅助可单独改善，非本计划生产缺陷豁免。失败原记录、真实反控、最终产物重冻回放见[合入收据](attachments/840/merge-r1/receipt.md)。

### PLAN-820 R1 — 2026-10-24，契约内缺口，禁止记债豁免
F820-R1-01/02/03（P1）：全链增量前端重编与退化gate缺失、worker瞬态峰值oracle缺失、同源码native对照被降级。基线 a33538088cd4ce309176d217361edc68a1d2ced8，needs_fix，executing 3/9。详见 [复审报告](attachments/820/review-r1.md)。回原树work修复/必要时显式回规划，merge-owned门禁pending。

### PLAN-818 R2 (2026-10-24): needs_fix
R1历史命中/正式接线/格式已验修复；新F-818-R2-01 trace链接及字节、R2-02语法回退丢前序编辑（202→102）、R2-03新增警告OPEN；原计时仍缺父侧比较边界。均为合同内必修，不降约。见[证据与复现](attachments/818/review-r2/review.md)，T08/T11重开7/11。


### PLAN-840 r8：批量回归修复关闭（2026-10-24）

原840内追加Phase2/3/4，保留全部原支持面：CTE可选feature注册、真实计量边界/运行池前提、测试载体Init及RQHost真实发布、full/t3 FFI共享fixture分组、Windows延迟删除瞬态锁错误有界重试，以及SSE/嵌套generator任务表guard重入死锁均已修复并保留反例。原批量文件锁日志缺errno，不将所有旧失败强归为错误5；新增真实Windows错误5反例及完整池复验关闭已确认机制，生产诊断保留最近系统错误。冻结ba6479da5完整t5345/tv163/tt5717/tb5316通过；落地53298cd27主检出完整tf6291/tt5717/tb5316全绿，失败与诊断历史均保留于840附件。D-762-1生产UI helper未修改、债务未被关闭；同计划修复仅将相关测试改为等待真实完成态。无新增未处理finding，不以隔离绿豁免完整档。

### PLAN-818 R3 (2026-10-24): needs_fix
R2 trace/prefix/warnings独立验闭；新R3-01 rounds deadline阻塞write/wait旁路、R3-02读入后验cap/无界队列/stderr超限未传播OPEN，原R1-01计时独立比较边界partial。合同内必修，不能记债合入。见[完整证据](attachments/818/review-r3/review.md)。T08/T11重开7/11。

### PLAN-820 R2 — 2026-10-24，needs_fix 5/9
受审 baf4d7d49492444cfd8063495a0bbd26af1b1d86。R1-02/03独立关闭，R1-01仍partial（真实事件计量缺口）；F820-R2-01 P1失败编辑已安装坏snapshot；F820-R2-02 P2 required skip脚本假绿；本计划新增Local0编译warning待清。见 [报告/复现](attachments/820/review-r2/review.md)。契约内问题，登记不授权延期；两轮non-pass后work先补根因与覆盖调整表。未合入，merge-owned门禁pending。

### PLAN-820 R3 — 2026-10-24 needs_fix 4/9
受审e8497f2f8d275873f55b4d7c0e9dd6ac1b75e3bf；R2-01失败安装/health独立关闭，R1-01真实操作反控与R2-02聚合必需ID仍partial。新增F820-R3-01 P1布局推导每根重置预算，2100层字段访问至少2206050而verify接受；V04 overflow漏例的不可达依据也被63类型真实反例推翻（实现拒绝正确）。详见 [报告/复现](attachments/820/review-r3/review.md)。契约内问题不授权延期，原树work修复并提交再审；merge-owned门禁pending。

### PLAN-831 r3 — jet/ark 真实构建面转后续环境门控渠道（2026-10-24，用户裁定承接项）
F831-09-03 环境面处置：jet（gradle/java/ANDROID_HOME 全缺席→gradlew 9009）与 ark（hvigorw/DEVECO_SDK_HOME 缺席→无构建执行）的真实构建验证，经用户三选项裁决"正式修订验收合同"（2026-10-24，T-10 R2 后）自 PLAN-831 AC-03/V-03 移出（经用户批准的非阻塞延期，review-r3 确认不再构成本 Plan 阻塞；在场执行后的真实编译/协议失败仍传播红）。承接条件：适用工具链到场（jet=JDK+gradle+Android SDK；ark=DevEco/hvigorw）时以新计划补真实构建实测；PLAN-831 内 V06c 仍五 profile 全尝试逐项记录（缺环境记【环境门控·转后续】，工具链在场失败仍红）。生成面（jet 24/ark 25 文件）与 vue/rust/vscode 构建、vm 运行均已实测绿，不受本项影响。

### PLAN-820 R4 — 2026-10-24 needs_fix 6/9（仅T06）
受审a789a2adc1afdab97025a0c294bb6121c97aa797。共享布局预算、真实overflow回归和真实操作反控已独立关闭；R2-02仍partial：listing和执行同时缺必需项可绿；新增R4-01 P2资源测试新增代码格式失败。见 [报告/控制](attachments/820/review-r4/review.md)。契约内缺口不授权延期，保留已闭证据，merge-owned门禁pending。

### PLAN-818 R4 (2026-10-24): final review pass
R3-01/02及R1-01比较残余独立验闭，R1/R2旧finding均无开放项。reviewed10/11@fec6b0c415fb1d384db1217057430b591ce01e43；[R4证据与复用边界](attachments/818/review-r4/review.md)。signature全请求性能已有同路径绿/负载红历史，merge参考机closeout仍须门控，不能放宽或略过。裸t/触面与独立workspace等merge-owned门pending，未合入/交付。此前R1–R3为历史。

### PLAN-845 final R1 — F-845-R1-01（2026-10-24，needs_fix）

P2原合同缺陷候选，非可延期债务：受审75277a7b7b8d1b43f840c8993721c02833e5657a/r2。已有保存路径的场景绑定全新未保存源码后Run，实际SourceSnapshot/Last Run/app_start错误误标尚不存在的scripts/<hash>.at，违反§5.2明确memory/hash来源；三个合法来源控制通过，反例实际执行至Auto API错误后在来源断言exit101。[完整finding与最小重放](attachments/845/review-r1/review.md)。T-06/T-08重开、executing6/10；AC03/06 fail、AC08 partial，其他pass保留。修复须提交及受影响独立复审；登记不授权缩面/延期，完整门仍merge-owned pending，未合入/归档/清树。

### PLAN-820 R5 — 2026-10-24 needs_fix 6/9（仅T09）
受审affb907bd8657d32a027453268cb209450b6c73b；R4两个P2独立关闭。新增F820-R5-01 P1：apply_caps新增trace授权未同步驻留CheckedEnv，后续合法mark_a热编辑undefined，而同源码同授权冷启动返回1。见 [报告/组合反例](attachments/820/review-r5/review.md)。原合同AC09缺口不授权延期；T06保持完成，merge-owned门禁pending。

### PLAN-845 R2 — F-845-R1-01 独立关闭（2026-10-24）

冻结e518f5fc778cfa6827f7de9eb99b4c0910ef294d/r2，新独立上下文复审pass。原反例与三控制、永久来源生命周期2/2、独立SaveAs/坏Verify/坏Open/解绑Undo兄弟状态通过；SourceSnapshot/Last Run/真实Auto回调错误均正确标记memory/hash或当前root已确认来源。F01关闭，无延期或合同降级；reviewed9/10，T10及全部merge-owned完整门pending，未合入/交付。[独立R2证据](attachments/845/review-r2/review.md)。保留R1历史及其他计划记录。

### PLAN-827 final R1 — 2026-10-25 needs_fix
F-827-R1-01..03 为原合同验收缺口，不可延期债务：汇总门漏资源/忽略case可绿，V13缺实际退化注入及未改对象代际/IO观测，G0仅数量摘要缺逐项库存。受审bd396d732/r3；T01/T06/T09重开，executing4/9。完整反例/最小修复重放见 [R1报告](attachments/827/review-r1/review.md)。统一门禁仍merge-owned pending；未合入。

### PLAN-820 R6 — 2026-10-25 needs_fix5/9（T02/T06）
受审7c93283423f23b52020e7e98c7bd235eeb9bd51a；R5授权环境同步独立关闭；V08有效hash/信封坏内层真实恢复拒且dispatch0。新增F820-R6-01 P1：正式data recipe只传Option<i32>，合法bool success/null丢值，宽标量typed fixture位型父侧比较/wrong-bits门未交付，原contract-decision§10/V10不可降级。见 [报告/原始请求](attachments/820/review-r6/review.md)。登记不授权延期，保留T09完成/既有关闭证据，merge-owned门禁pending。


### PLAN-845 merge — F-845-M1-04（2026-10-25，阻断 / needs_fix）

冻结da43267d4dffe9064d9d95585b7b56416b730d5e，consumer-user Move实际native phase6要求Single Step恰好一tick，但暂停trace54仍54（应55），Paused/error null；生产端同源完整Move通过。尚不能区分原生点击未接收与consumer运行时分发问题，不当作已知flake/可延期债务，不放宽原断言。共享play::Probe phases5/6→native click/post→Host play-step→PlaySession single_step，并影响后续user/project native。原始[失败报告](attachments/845/merge-r1/consumer-user/move/report.json.txt)、[诊断/剩余门](attachments/845/merge-r1/failure-m1-04.json.txt)、[收据](attachments/845/merge-r1/receipt.md)。

producer/consumer完整driver各155、AGP各163和已绿原生门保留身份；消费端user余12/两端project各15与实际UI输出搬迁各2/完整81-call审核未执行。前三次自动修复已独立关闭，默认三轮cap到达；无第四次源修复、未合入/沉淀/归档/清理，保留同一845 clean树。需明确click接收/pending/tick过渡、永久反例控制、同树修复提交与受影响独立复审后补所需完整门。登记不授权合同缩面或豁免。

### PLAN-827 final R2 — 2026-10-25 needs_fix
受审4531ece17/r3；R1-02核心检测缺口定向关闭，R1-01/03仍开（nextest范围外跳过误拒/必需成员未绑定；stdlib及源码身份库存仍缺），新增R2-01三处unused_mut。T01/T06/T09重开executing4/9。见 [R2报告](attachments/827/review-r2/review.md)。原合同缺口不可延期，统一门禁merge-owned pending，未合入。


### PLAN-845 R6 — F-845-M1-04 独立关闭（2026-10-25）

冻结944c1c551b524d7d8c7f69519e465831417d21a8，新独立上下文R6 pass。实际正常/36帧延迟Step均phase27；丢点击在原360帧phase6截止拒绝，重复实际+2拒绝；真实Step矩形down/up与36GPU检查保留。独立Patrol120帧绿、361帧超预算拒绝、原交互/丢释放反控保留；实际cursor裁切与任务栏foreign PID护栏均准确拒绝。仅原生probe工作区放置/每点可达性和严格tick+1有界确认，不改Host/VM/正常App/取消/期限，不重发Step。原历史54→54缺事件记录故不虚构唯一根因；两夹具不变量与可区分反控独立闭合。用户上限6，计数4/6；此前blocking历史保留。完整83项merge门仍pending，无合入/交付。[R6](attachments/845/review-r6-step/review.md)、[第4轮工作/边界](attachments/845/work-r4-step/readiness.md)。


### F-845-M1-05 — closed (Plan845 r2, automatic repair5/6)

Merge on944c1c55 exposed the original native authoring probe silently discarding scroll errors after real node selection. The legacy click path lacked the common physical-pointer/setup contract. Frozen fix ae1a87e15b857329efc1110beb9c76e9c0299baa unifies legacy OS inputs and propagates all five scroll errors without changing product semantics, original target/GPU/Save/Undo/Redo/resize assertions or1400-frame budget. Permanent actual native failure case distinguishes discarded Result (controlled mutation red) from immediate error (restored green); full accepted control phase9/280/accepted3, wrong/partial remain negative. Fresh-context independent R7 pass: [receipt](attachments/845/review-r7-legacy/review.md). This is a closed gate finding, not deferred debt; complete merge/landing remain separately pending until verified.


### F-845-M1-06 — closed (Plan845 r2, repair6/6)

Merge onae1a87e15 exposed hierarchy phase0 scroll before initial focus/pointer preparation; shared assets path had the same ordering. Actual native point hit Terminal PID1932 before run() readiness was reachable. Fix231957020cc855485adfc4b15be75457a94ef2d8 moves initial readiness ahead of auto-scroll in both probes, preserving original240 budget, pixel/state/Save/cancel and PID guards. Two permanent actual-native delayed acceptance/deadline tests reject old-order mutation (both frame12 foreign-hit), restored green; affected hierarchy4/pixel-faults2/assets7 pass. Fresh-context [R8](attachments/845/review-r8-preparation/review.md) independently closes it, no deferred debt. Complete merge/landing remain separate required checkpoints.


### B845-FEATURE-01 — CLOSED / delivered PLAN-847 (2026-10-26)

845 post-cleanup main batch at c9ab21ea69ff70ed8f53bbbef87f9a89ff26b891: tf/tb fail before tests with unresolved optional auto_test_case; tt completes green. Introduced by 818 T-02 commit5bebb0ad02c9807f157968bc6b34bf59f7431d66: inserting ac_mn_probe_tests between the existing cfg and probe_adapter_tests moved the old module's feature gate onto the new one. Both attributes now protect the new module; the old module is unconditional. This is a main Rust regression, independent of 845's autographics-dev source. [Raw three-leg receipt](attachments/845/batch-r1/receipt.json.txt), [delivered847](archive/847-restore-probe-test-feature-boundary.md). No inline source fix, threshold/selection waiver or unmerge. Complete coverage remains anchored at840; due batch retry required after847.


Closure847: independently reviewed60f0bb033; frozen t3181/3181 and main feature-off tb3109/3109 pass, optional/default-off and2177IDs unchanged. [Delivery](attachments/847/merge-r1/receipt.md). Existing847 blocker closed; last complete global green remains840.

### B847-FULL-OWNER-01 / B847-TT-STATE-02 — CLOSED / delivered PLAN-850 (2026-10-26)

Main2ff1286c5 full batch: tf cannot select external auto-ui packages after831 abd34a355 (zero tests); tt3547pass/5fail/1timeout includes4Windows1224 report-output writes, stdout-cap classified wall, FFI018timeout; tb3109pass and prior frozen t3181pass remain valid. Cause beyond observed output/selection failures is not invented. [Full triage](attachments/847/batch-r1/triage.json.txt), [fix-forward850](archive/850-restore-cross-workspace-full-regression.md). Preserve all IDs/limits/inputs, no global green anchor advance; product repair beyond orchestration needs its own concrete contract.

PLAN-850 final closure: independent final pass at7750f8d33, UI fixture-root repair b4fe000; actual frozen main batch-r4 language3218/UI3073/tt3553/tb3109 all passed. Per-feature full/required identity guards and producer-only738/796/818/827 report preservation verified. No test/profile/input/threshold reduction or performance waiver. Earlier failed r1/r2/r3 receipts retained; r2 signature timing cause is not asserted from concurrent-process presence. [Final evidence](attachments/850/batch-r4/receipt.json.txt), [archive](archive/850-restore-cross-workspace-full-regression.md).


### PLAN-845 final closeout — closed findings / optional fixture observation

R8 closes all implementation findings within6/6 repairs; 83 effective calls and both driver158/0/0 pass, source landed autographics-dev, canonical/ledger/archive/guard cleanup verified. Initial Squash negative stopped atphase2 before focus oracle and is explicitly invalid evidence, preserved raw; bounded same-binary original-assertion replay reachesphase9 and rejects expected focus fault. Exact historic timing cause remains unproven; optional fixture reliability observation, no product-contract waiver or pending845 fix. [Classification](attachments/845/merge-r1/game-control-replay/oracle-classification.md). Batch feature regression above stays open under847.

### PLAN-820 R7 — 2026-10-25 needs_fix5/9
受审278b1dce0e94cbe7e5e68a921e5ee569abc88314。typed通道及7反控/corpus正例绿；新增R7-01 P1旧core空i32值附data typed从Fail变Pass（真实parent负控），R7-02 P2冻结f32 NaN被1.0替代（原覆盖余量）。见 [报告/复现](attachments/820/review-r7/review.md)。契约内问题不授权延期，T02/T06回work，其他旧finding关闭保留，merge-owned门禁pending。

### PLAN-827 final R3 — 2026-10-25 needs_fix
受审25c2ac9fb/r3；R1-03库存修复与R2-01警告已独立关闭；R1-01残余P2：必需ID子串匹配，被加_disabled后缀改名测试冒充，AST原函数三对照复现。仅T06重开executing6/9，其余完成保留。见 [R3报告与重放](attachments/827/review-r3/review.md)。剩余原合同缺口不可延期，最终验收及merge-owned门禁pending。

### PLAN-827 final R4 — 2026-10-25 needs_fix
438a700d0/r3：R1-01精确ID残余已独立关闭；续做初审发现F-827-R4-01 P1，必需native工具缺失时AP01/AP23对照测试仍pass/exit0（非本次修复回归）。实际AC741_RUST_LLD不存在文件反例两条复现，恢复工具正控两条通过。T03/T04/T06重开executing4/9。见 [R4报告/最小重放](attachments/827/review-r4/review.md)。原合同缺口不可延期，未合入，剩余最终审查与merge-owned门禁pending。

### PLAN-827 final R5 — 2026-10-25 needs_fix
受审ef924df27/r3，R4缺工具误绿已关闭（实际脚本负控exit1/正常两scope通过）。F-827-R5-01为原合同未闭包的验收覆盖绑定：V06/V08/V09/V13剩余准入/授权/边界故障清理/依赖寿命raw，完整矩阵见 [R5报告](attachments/827/review-r5/review.md)。允许复用未变的D798/D828有限证据，不要求全档重跑。T04/T05/T06/T09重开executing3/9，其他通过保留。不可延期抵扣原合同，最终验收及merge-owned门禁pending。

### PLAN-820 R8 — 2026-10-25 needs_fix6/9（仅T06）
受审7308acad627df367c7a79e43212aec4f9078b790。R7两finding独立关闭；新增R8-01 P2结果协议shape漏检：bool/2、bool/00及有匹配exit时weird/bad都经真实parent报Pass，见 [报告/7组合复现](attachments/820/review-r8/review.md)。T02保留完成，旧关闭不重开，原树work修oracle无关的统一typed准入。既有性能红环境归因未重试也未独立豁免；merge-owned门禁pending。

### PLAN-827 R6 readiness — 2026-10-25 needs_fix
9f7b5dd6e/r3：F-R5-01仍partial，送审矩阵明确承认V09第n次分配故障注入无缝/留后续owner，未获合同缩面批准，不可凭显式缺口登记判完成。依skill就绪门未跑完整复审/测试。仅T04/T06重开executing5/9，其余成果保留。见 [R6记录与冻结矩阵](attachments/827/review-r6/review.md)。先work补控制或new批准合同修订，统一门仍merge-owned pending。

### PLAN-820 R9 — 2026-10-25 needs_fix6/9（仅T06）
受审8e0e0ef337991c1e1726847b8c6c70aaccf26a6c。R8原七组合独立关闭；新R9-01 P1规范i32校验误拒零本身：真实record源码返回0、期望exit0被判协议Error，而-1/1/42通过。见 [真实guest矩阵/报告](attachments/820/review-r9/review.md)。原树work修单字符0合法边界，旧关闭保留，不授权延期；merge-owned门禁pending。

### PLAN-827 R6 work repair — 2026-10-25 frozen for independent T07
4a069ce7f已补齐原V09 nth帧槽位分配失败/真实释放反控（n1/2/3失败/n4正控/零预算attempt0/暂停恢复及取消drop清理）；变异绕过触发exit101、恢复后定向测试通过。原R6 admitted gap的work闭包见 [冻结收据](attachments/827/work-r6/readiness.md)。work7/7、execution_done7/9；当前聊天已实施，独立T07未执行，不能据此宣布final pass/交付。统一门merge-owned pending。

### PLAN-827 final R7 — 2026-10-25 needs_fix5/9
4a069ce7ffeaeb1cdd7491bab65e7838dc3f09ea/r3：R6 nth帧槽位分配/实际释放不变量独立关闭（33定向测试含原文件/cache/增量反控/executor全部通过）。F-827-R7-01 P2为V13 raw的30行未改函数before身份全null及失败前未持久化；F-827-R7-02 P1为资源脚本未构建/未绑定当前release worker，当前旧exe仅辅助实物证据，旧性能raw不能认证R6。仅T06/T09重开，其他完成保留。两项为原合同证据缺口，不授权延期；最小闭包与完整AC/SD审查边界见 [R7报告](attachments/827/review-r7/review.md)。未跑全性能矩阵/整档，统一门merge-owned pending。

### PLAN-820 R10 — 2026-10-25 needs_fix6/9（仅T05）
受审403eb57d0f1b20cb5600d620edc0e1ac465876cd；R9零边界及所有旧finding关闭。V12精确cap、17位型call/store/restore与数据域剩余证据通过；新增F820-R10-01 P1：已交付817的av_driver snapshot/closure两处Value匹配E0004，CTE锁漏新依赖边导致--locked失败。见 [报告/构建原始输出/锁差异](attachments/820/review-r10/review.md)。用户本聊天已明确豁免本轮旧标量1064.5ms性能项（r7 EX820-R10-PERF），该豁免不覆盖编译/锁闭包。仅回work修T05，其他已闭数据矩阵无需重跑；merge-owned门禁pending。

### PLAN-820 R11 — 2026-10-25 final pass（reviewed8/9）
受审18d031102551bc259f7d1307309ffe0ac4eab6d1/r7。F820-R10-01独立关闭：冻结lock完整metadata通过，CTE snapshot/closure及类型边界定向6/6通过；所有旧finding关闭，无活动work-owned缺项。AC/SD收口及证据复用边界见[R11报告](attachments/820/review-r11/review.md)，规范增量已冻结hash。用户批准EX820-R10-PERF仅本轮旧标量1064.5ms性能项，原红值与长期门保持；merge-owned裸cargo t+tv+独立workspace/新基面消费者检查及Spec/归档/清理仍pending，未合入或宣称交付。

### PLAN-831 合并门新可见预存红（2026-10-25，归因在案非本迁移引入）
UI 仓新 t 档（PLAN-831 T-09 创建）首次选中 auto-ui-tools lib 测试面，暴露两个在 lang master 迁移前副本即同样失败的预存潜在红（旧 lang t 档从未选中该面）：rust_ui::tests::merged_api_client_crud_fallback_for_uncovered_endpoints（a2r 生成物形态断言漂移）与 vue::plan593_index_css_golden_tests::index_css_values_match_p1_baseline（金样漂移）。归属证据：lang master 同测试同败（merge 收据）。待后续计划修复或再基线。

### PLAN-827 final R8 — 2026-10-25 needs_fix6/9（仅T06）
ebced8cd5e5f26e609062acdcb08808452af5e20/r3：R7两finding独立关闭，T09恢复pass，R6分配/清理闭包保留；新F-827-R8-01 P1原资源证据残余：当前绑定worker虽正确，但实际bench summary/exit未持久化，raw180行和binding无前端/总峰值、残差硬门字段；仅交接数字不能补峰值真值。见 [R8字段审计与最小补证](attachments/827/review-r8/review.md)。优先恢复现有stdout；无源则修留档后仅补受影响资源测量，不重放无关全验收/性能矩阵。不可由merge统一门豁免；未合入。

## PLAN-820 合入适配（2026-10-25，merge c4afe1281）

- **818 mn 工作区编译适配待 owner 复核**：820 扩展共享 auto-av-core
  （Value/MirType/MirConst/ScalarShape/MirInstr 宽变体）后，818 在途工作区
  `experimental/ac-mir-native` 六处穷尽匹配 E0004。按 F820-R10-01 同型做
  最小受控拒适配（emit.rs 宽/record 参数 Unsupported；fingerprint.rs 形状
  avdata 标签/常量防御渲染/三指令纯格式渲染；lower.rs 宽常量与
  FieldGet/FieldSet/Aggregate 受控 lower 拒）+CTE/mn 双工作区锁补
  windows-sys 0.59.0 实际边。mn 标量语义零变化（语料不触新臂）；master
  日常档 ac_mn 族保持绿。**818 owner 在其合并/下次修复轮复核受控拒绝
  语义并按需替换为真实支持**。
- **818 resources_tier 采样器计数 flaky**：mn 工作区
  mir_native_resources_transient_spike_sampler_and_raw_gate 在并行负载下
  "sampler actually polled (4)" 偶红、隔离绿（主检出同绿）——818 既有
  并行敏感测试，非 820 引入。

### PLAN-827 final R9 — 2026-10-25 needs_fix6/9（仅T06）
cce6139e87629edbb251ded6a714a2c0dd5bdb66/r3：F-R8-01缺峰值/summary主体已关闭，103747直接捕获数据及历史summary原console均独立核实；同finding降为P2有限残余：gate0实际validator接受gate1000+无关fresh失败exit5，实际writer非空-Args输出空串（真实artifact同样）。见 [R9实际函数反例与最小闭包](attachments/827/review-r9/review.md)。仅T06重开；当前性能raw/summary保留可复用，不要求脚本判定/参数修复再跑性能矩阵。R6/R7/T09通过保留，统一门merge-owned pending，未合入。

### PLAN-832 v0.7 环境门控承接（2026-10-25，用户批准延期）

- **jet/ark 真实构建**：r3 用户裁定“暂时不测，等v0.7再考虑”。本 Plan 仅验收六 profile 真实生成、vue/rust/vscode 真实构建和 VM 真实运行；jet/ark 构建转 v0.7 环境门控渠道，未实测不得宣称通过，工具链在场且失败仍须非零。承接时需补 Gradle/Java/ANDROID_HOME 与 hvigorw/DEVECO 的实际构建及产物检查，不扩大其他延期。

### PLAN-827 final R10 — 2026-10-25 reviewed8/9（全部原finding关闭）
9ae7a7a3b6fa118cd94d6ea053ecc7da6d6bab96/r3：F-R8-01有限残余独立关闭，原AST反例false、writer参数逐字保留，8同机制控制+6真实/回收工件判断全符；原103747空args不改写，sidecar保留源码导出/归一化溯源性质。T07 final pass，AC01..07/09完成审查；AC08/T08和冻结merge-owned门仍pending。完整 [R10报告/冻结SD/hash/证据复用边界](attachments/827/review-r10/review.md)。无未解决in-contract finding，不等于已合入/交付；未跑整档/性能矩阵，未改canonical/ledger或清树。

### PLAN-832 final R1 — 2026-10-25 needs_fix（合同内OPEN，非批准延期）

r3 / lang `cd603e418accde3bb85327cc1b6affd49b2ce93f` + Man `356f2c8186d07bc18567b18179c3cad7bda2d483`：[完整审查/可重放反例/AC-SD矩阵](attachments/832/review-r1/review.md)。F832-R1-01(P1)：Core关闭UI特性仍要求UI邻仓manifest，安装无条件读取UI/down，四仓clone测试不能证明无UI独立来源；02(P2)：Windows OpenProcess AccessDenied被判死亡，真实PID4/error5导致合法staging误删，活控/死亡控均已验证；03(P2)：Man ti和跨仓CI未接UI tool/lang te-child owner，清环境真实生成测试1败1，显式固定tool后1/1；04(P2)：目标README/CLAUDE仍指导旧am/根cargo install（virtual manifest exit101），旧release.ps1与fixed sources/profile/恢复入口未承接。全部为原AC内必修，记债不构成延期；jet/ark r3用户批准转v0.7保持有效。185检出hash、Core/full校验、实际脚本、runtime2/2、777 strict0v4gf、29refs/23tags恢复通过事实保留。T05/T07/T08/T09重开，executing5/12；同树work修齐定向反例和合法对照、新SHA/SD/handoff后复审，统一门仍merge-owned pending。未改实现/跑整档/合入/发布Spec-ledger/归档清树。

## PLAN-821 T-09 发现的 782 源域缝（2026-10-25，work 记录、owner=782/ac-core）

- **多参 `.move` 调用标记关联错位**：`fn bump(move u Note, k int)` + 调用
  `bump(a.move, 1)` → `verify.ownership-required: argument is marked .move but
  parameter 1 does not declare move`（标记被关联到参数 1）；`.mut` 双参
  （`bump(mut t Tok, d int)`+`bump(a.mut, 5)`）与静态方法形
  （`Tok.new(fresh.move, 9)`）均正常。实测于 plan-821 T-09（owned 增量签名
  变更形状选型时绕开）；782 owner 复核多参 move 标记的参数关联。
- **借入 record 参数的 owned 调用侧降层未落地**：hir_adapter 的
  `ExprKind::BorrowView` owned 臂仍为 `lower.borrow` 占位拒（原注释"land with
  PLAN-821 T-03"，T-03 实测未触达）；782 源域支持借入 record 参数
  （`fn bump(u Note) str { return u.body }` 编译过 verify），但 owned MIR 冻结
  类型表无 record-view 值类型可表达调用侧不 kill 的借入传参——MIR CALL 的
  owned 实参统一 move-in kill。属 MIR 表达力缺口非降层遗漏；如需支持须扩
  record-view 类型（回 new 显式 revision）。T-09 测试以冻结论域内形状（param
  Note→Str 重定型）覆盖同一接口失效轴，并留此债。

### PLAN-832 final R2 — 2026-10-25 needs_fix（F01/F02关闭；F03/F04有限残余）

r3 / lang `87f82d637ecd586c17a01c1c975e1456bc346991` + Man `ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f`：[完整R2/复现/关闭oracle与覆盖诊断](attachments/832/review-r2/review.md)。F832-R1-01独立Core来源/锁/640安装/真实管理语言和F832-R1-02实际AccessDenied/live/dead/unknown保护均独立CLOSED。仅F03(P2) PARTIAL/OPEN：Cargo父测试先解析再给CLI注入tool，原例1/1绿；直接生产CLI清CARGO_MANIFEST_DIR/AUTO后仍exit1只尝试Man邻接target；合法工程仅加tool/manifest/UI根各exit0产App.vue，安装邻接与显式缺tool正反控成立。F04(P2) PARTIAL/OPEN：旧am/虚拟manifest说明及release退役已修，新文档从真实升级backup移回后verify因额外.man-backup.marker返回2，移走唯一标记其余字节不动即640/exit0，备份数据无丢失。原CI/helper定向、Core和owner有效证据保持，文档与默认发现不变量未闭合；记债不批准延期，jet/ark r3裁定继续有效。只重开T08/T09、executing7/12；这是第二次final needs_fix，work下一handoff前须记录真实CLI子环境与真实backup往返的覆盖诊断/oracle调整，提交后仅复审受影响面。未改实现/跑整档/合入/发布Spec-ledger/归档清树，merge-owned统一门仍pending。

## R849-NATIVE-CLOCK — spatial native probe clock-pause recovery

PLAN-849 merge observed one consumer dash3d-short reopen auto-Pause (tick backlog>4) after8ticks; phase4 waits for80 inputs without resuming clock pauses and timed out. Same frozen binary full14+4 isolated retry passed, no geometry/CPU fault or input interference shown. Preserve original clock guard; candidate follow-up should explicitly resume legitimate clock pauses through OS controls within unchanged deadline and retain counterexamples. [Failure and retry](attachments/849/merge-r1/sweep-clock-diagnostic.json.txt). This is verifier scheduling stability debt, not a waived gate.


## PLAN-841 R5 — F841-R5-01 (OPEN, P2, acceptance blocker)

The accepted compound props `[[.source]]` and `[{inner: .source}]` keep a borrowed nested VmRef without queue/replayed-container ownership. Real parser/lowering/view build accepts both and queues Child Init; replacing the parent's source field then forces collection of the still-referenced object. Direct-array control passes. This is not accepted deferred debt or approval to narrow nested input support. [Finding and replay](attachments/841/review-r5/finding-F841-R5-01.md); [review](attachments/841/review-r5/review.md). Original G04 graph/direct-ref examples now pass, but shared lifetime invariant remains partial. Plan executing6/12; work must fix full producer/container lifetime then obtain affected review. Unified gates remain merge-owned pending.

### B855-FEATURE-01 — AV corpus optional dependency boundary (2026-10-27)

Main batch7f966bfa3 language listing failed with24 E0432/E0433 in821/827 test modules;822 insertion displaced existing cfg. Zero tests ran; UI/tt/tb not reached. Fix-forward [PLAN-859](859-restore-av-corpus-feature-boundaries.md); evidence [build](attachments/855/batch-r1/language-build.log.txt). Last complete green remains850; retry required. No855 graphics source attribution or rollback.
