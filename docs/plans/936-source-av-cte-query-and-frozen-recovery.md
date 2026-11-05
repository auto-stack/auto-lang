---
plan_id: PLAN-936
status: executing
feature_name: SQ-C 普通源码真实AV CT查询与冻结回灌恢复闭环
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-06
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-cte/scalar-query-profile.md, docs/specs/auto-cte/frozen-cache.md, docs/specs/auto-semantics/source-query-session.md, docs/specs/auto-test-case/architecture.md]
new_spec_components: [docs/specs/auto-cte/source-query-integration.md]
touched_goals: [GOAL-002]
affects: [auto-cte, auto-semantics, auto-ac, auto-av, auto-a2x, auto-test-case]
depends_on: [PLAN-817, PLAN-934, PLAN-935, PLAN-929]
current_step: 11
total_steps: 14
---

# [PLAN-936] SQ-C 普通源码真实AV CT查询与冻结回灌恢复闭环

## 0. 变更摘要

在934源码query/935真实实例与929来源上，以有限`#{ expr }`自动产生CT查询，调真实AV执行checked callee/实例及其依赖，冻结成功结果入统一cache，再回灌普通source推导与Checked HIR并让四端真实观察。复用817真实服务/契约/资源机制，去掉新source链必须手写deck/placeholder/绑定位置的前置，不复制执行器或绕开依赖失效。

## 1. 目标

首例`fn sum(a int,b int) int {return a+b}`、`fn main() int {let n=#{sum(20,22)};return n}`，冷真实AV执行→I32(42)冻结→局部推导/常量HIR重验→AV/native/Rust/C各42；只改sum为a+b+1、调用点不改得到43。加入bool、嵌套CT DAG、935真实choose实例、有限source-meta→实际AV参数。成功hit/restart `ct_execute=0`，完整来源/依赖/消费重新准入；非读者与未改函数/实例真实P0。

非目标：`#if/#for/#is`、多语句CT、类型/声明生成、全语言反射/const泛型、非scalar运行值、捕获动态runtime局部、任意IO/env/时间/随机/FFI/JIT/发布器/旧产品替换。不把meta-only、796模型compute或新host折叠器当真实AV，不把旧deck证据冒新普通源已接线。

## 2. 架构方案

query核心定义无后端的执行端口（拟CtExecutor）；拟`experimental/ace/crates/source-cte/`从817提取真实AV/冻结与必要调度服务，依赖comptime-contract/core、source-query端口、shared lowering、av-core/artifacts，**不依赖ace-av前端会话或native**。consumer在编译宿主/worker组合根注入服务；A2X HIR/provider/目标后端仍不依赖MIR/AV/Cranelift。原comptime probe委托同服务，保closure/snapshot旧车道，不能复制cache/solver/AV或反向依赖comptime大包引环。

源AST按统一前门解析HashExpr及CT作用域，生成带来源/owner的query DAG；按需检查实际callee/实例执行闭包，构造checked最小调用入口，经共享MIR/ABC真实AV。结果只在成功身份绑定后产生executed envelope；回灌值→934事实/935实例继续推导→core bind/verify→四端消费。不要求整个模块已经final Checked才开始CT，不反复把表达式/用户模块格式化成源码再解析取结果。

## 3. 技术栈

817 `comptime/{driver,av_driver,store,observe}`、comptime-core/{schedule,dependencies,consumer}、comptime-contract、av-core执行/共享lowering；934查询端口、935实例、929来源/恢复、现worker/auto-lang-core TestCase桥与固定组。新服务/执行端口/source schema尚不存在，由T-02证明后实现。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户仅授权规划复核/直接修订，不施工、建树、运行测试或派发。初读main `f6b0f918d`，复核期间推进`0771a3b9e`：D928=`6b429502f`已交付（归档计数10/12是簿记残留，不重开）；D931=`84c894b92`已交付归档，Rust print轴为/2，D-931-1恢复首编辑仍由929持有、D-931-3 manifest独立来源审计仍归933；932已executing r2 0/12并有lang-932树，933draft r2 0/10。929executing r2 1/14，D928前置已满足但仍须原会话按用户挂起/恢复规则处理，本会话不代resume。下列f4062cca6/928在途/931draft等均为初订历史，现状态/窗口以本条与[本次复核](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)为准。新source-query/source-cte包仍不存在，不以本规划当实现。

用户本轮只授权正式规划，不施工/建树/构建测试/认证或派发。取材main `f4062cca6`。796/817均归档，历史delivery分别b76eca8d2/ec6a54c5d在当前HEAD非祖先；当前源码/Spec与[基线报告](reports/ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)hash定位，旧PASS不自动迁移。817确有真实AV成功/失败计数、执行envelope/A05统一store/依赖/consumer重验，不能重复造一个CT解释器。

真实差额：`CteDriver::query_step`接deck，consumer::backfill_and_verify会重compile consumer源，av_driver snapshot经源码wrapper暖编辑；comptime Cargo依赖AV/native，AV/native又依赖core，不能core或AV反依赖其整包。当前796 InstanceKey只是模型；935会供真实实例。旧Auto `#{expr}`语法已记入跟踪语言文档，但旧表达式位实际未完整编译期替换（auto-lang/comptime/overview/hash-syntax）；本包取其拼写，不假定旧运行行为是ACE CTE实现。相关[Design42 §10](../design/42-semantic-metadata-inference-and-comptime.md#10-普通源码sq真实实例与av-cte接线2026-11-04target-state)为本包范围/资源/依赖组成。

拟唯一树`D:/autostack/.wt/lang-936/auto-lang`/`plan-936-dev`（未建）。开工材料仅`D:/autostack/plan-artifacts/plan-936/`登记，外部consumer/target/log/包副本显式归scope或正常专属树；merge摘要入本计划后scope全删/全输出对账/guard清树。未指定模型人员，不新建聊天/agent。

## 5. 详细设计

### 5.1 先证明无环真实全链

T-01冻结comptime完整调用图/producer/旧envelope身份、真实fixture/原scale/所有source入口；T-02有界证明提取服务不依赖AV前端/native、旧probe可原样委托、新普通源码无需deck/人为constant锚：source HashExpr→最小checked callee/typed wrapper→真实AV→scalar frozen→owner事实/HIR→四端与恢复首编辑。若只能调用comptime大包造成依赖环、全源重parse或手工改HIR placeholder，先needs_replan，不保留“暂时”生产fallback。

源码新请求/producer/lane必须版本化，不冒旧`plan817/cte-probe`的执行身份。旧A05/envelope是否可由服务原样复用，按callee/deps/engine/caps/owner/source/mode与消费方逐字段准入；不兼容新namespace/key，保持旧codec/旧probe/模型CT拒伪vm约束。提取不能卸掉旧snapshot车道或原反控，也不能让CT服务依赖Rust/C发射器。旧SnapshotLane的FunctionIncrementalSession等前端适配可留在原probe之上，经中立checked-executable/执行端口调用共享服务；不得把该AV前端类型挪进最小服务从而恢复依赖环。原适配/新source lane共享实际执行准入/冻结/缓存，不要求机械搬整份av_driver.rs。

### 5.2 源表面、绑定与调度

显式能力profile`ac.source.cte.scalar/v1`（拟）启用单表达式`#{ expr }`，返回i32/bool，纯scalar函数/实际实例、控制流/递归按817支持面。HashExpr AST携带本体unit+relative node/source revision/期望类型/owner；callee名字由普通resolver绑定实际decl/instance，不按字符串定位let名或手填ct_ref→HIR索引。无cap保原拒绝，unsupported CT构造具名拒，不能跳过Hash token当普通运行时表达式。CT表达式不得读动态参数/可变runtime局部；literal/既已冻结CT值与支持纯callee合法；不承诺外层const功能或多语句块。

CT DAG复用817调度/manifest；普通纯函数的未执行if/短路分支trap保持潜伏，由checked AV控制流实际决定，不能在host/查询扫描时提前执行；相同完整key single-flight，前向合法依赖的解析按语义图决定，不能把旧“deck声明序”变新语言限制。普通函数递归走AV fuel，不等同query依赖环；互相需要CT值才可完成检查的A→B→A输出含闭合边调用链。类型/资源/取消/工具/协议/陈旧快照分别typed，从失败处保留事实，沿932/914有效策略verdict、坏事实ERROR规则，不读message归因。执行到trap/resource/cancel也计AV attempt/ct_execute真实事实，但只有成功值可发布executed冻结；失败不会缓存PASS或永久污染可修复会话。

meta源使用显式Record声明能力的有限query profile：允许`type Pair { x int; y int }`等声明和i32/bool函数，不允许record运行值/owned字段参与本CT执行。元信息从现有名义声明表来，不能为了得到FieldCount把全部record body/方法重新推导；新profile/caps由T-02与934/929前门冻结。

有限source meta按Design42 §10.3声明intrinsic `ct_field_count(TypeName)`/`ct_has_field(TypeName,"field")`，须`cte.meta.fields/v1`，只可在CT中读当前名义type声明。类型/字段串仅query selector、AV参数仍i32/bool。resolver绑定受保护intrinsic def（非猜拼写），该profile用户同名声明具名冲突，缺cap/缺record/字段串非字面量/错误位置的拒绝明确且配合法近邻。meta-only计meta_query、不能计vm_execute；`#{sum(ct_field_count(Pair),40)}`两字段42/三字段43，HasField缺→有真实负依赖。新intrinsic不是声称旧stdlib已有API，不将所有type/reflection/runtime值扩面。

### 5.3 真实AV与推导/HIR回灌

callee/实例及可执行依赖先得到shared Checked HIR，shared lowering→verify MIR/ABC→AV；绝不执行未checked AST，不以先编全module掩盖CT决定当前owner的类型。可用direct HIR wrapper或等价checked调用入口，不重parse未改用户body/整源来烘焙实参。旧817源码wrapper/probe保其兼容路径，新源lane必须有自己的禁止退化/真实producer证据。

冷sum(20,22)至少1次实际AV；相同CT子查询的重复callsite只按完整key去重；hit/跨进程恢复禁调execute反控。不能host折叠sum再把计数填1，不能写marking就mint executed。actual AV typed值严格对声明/期望验证（bool不当int任意转，错tag拒），替换结构化owner节点/语义事实后重新推导必要owner并verify HIR，不能让后端私自“信冻结页”补类型。A2R/C收到的同源最终HIR里已是具体常量/调用绑定；原CT call不得潜入运行时执行再冒编译期结果。

CT实例组合以`#{choose(40,42,false)}`=42、模板换body后40；调用点不改，真实实例/其helper body/caps等失效链贯通。bool CT用于if条件另设真假翻转oracle，检验tag/推导事实而非仅打印JSON。消费认证分轴：发射金样/manifest来源、真实四端行为、手写reference与CT执行事实，互不替代。

### 5.4 完整依赖、新鲜性与事务

执行语义key/manifest（与本次owner UseBinding分离）含实际callee logical/signature/有效checked body传递闭包、935template/substitution/具体instance和类型版本、参数、meta字段集/缺席探针/候选域、算法/profile/engine/lane/MIR/ABC/codec/producer、caps/target相关语义；当前source与owner事实由独立UseBinding绑定。调用点字节不变不是命中条件；改helper、名义字段序/新增字段、同名跨module/owner、权限/engine/producer任轴，真实读者失效而独立函数/CT结果可hit。沿817 ClosureBodies允许其声明的保守闭包，不把整个模块所有body当依赖；未访问meta域不受其集合指纹影响。

成功hit仍重验依赖/嵌入类型/来源及本次owner消费，不缓存consumer verdict。错frame/错地址/owner/key/foreign schema按原受控拒绝，不能一律miss；合法新依赖版本使旧语义条目不适用，重算走新key。version change不清库，保原old条目/lease与用户哨兵，内容寻址同key异payload受控冲突。

source/CST/decl/query实例/CT与目标root采用934/929同一candidate事务；CT成功中间CAS页不等于current，owner失败/取消/late不得安装candidate或发布半root，未root页预算/GC可回收。发布前检查required闭包活性，原source/旧程序/旧CT root与诊断版本绑定不变，后续修复可继续。recipe/溯源不强pin全部历史AV/AST，真正消费的冻结类型/值页为required，活执行旧代独立lease。

### 5.5 函数级增量与恢复首编辑

全链计数先于prepare/restore/AV调用安装，包括失败early-return attempt：source parse/project、query infer/instantiate、HIR/plan/bind、MIR/ABC/AV载入/执行、freeze/decode/persist、各target生产/I/O。只改helper body：helper及受影响CT/owner按依赖重做；不读helper body的普通caller/独立函数/无关实例P0。只改某CT实参：改变该query/owner，helper本体不重编；此前未请求过的callee/实例/新目标执行单元若因实际依赖变化首次需要，单列“新需求受影响生产”及其原因，不伪称未改函数P0、也不偷缩原固定负载P0门；resident AV wrapper若需新产须计为内部合成unit，不偷算到未改用户函数或藏在M。hit时ct_execute0，历史回退完整准入后复用真字节，不能假命中/默认0。

跨进程restore顺序：source/grammar/decl→query/instance→冻结envelope完整准入→必要Checked HIR深验→目标装配。无旧query/store PASS/指针/活AV序列化。decode与结构深验R非零，parse/solve/instantiate/lower/emit不可改记R；冻结命中不需要AV执行，不得先走source warmup/parse/lower再报告execute0/未改P0。若首编辑P0需要可复用执行单元，session root必须显式required已有checked/MIR/ABC页，在restore的R窗口decode/deep-verify并保留有界resident；这是真实R载入/字节，不是隐藏AV生产。首个只改实参的编辑复用这些居民helper，不再读写未改payload；新wrapper单列真实生产。缺必要复用页则该恢复车道具名拒绝，不能静默重编。

restore→立即首有效body/CT参数/helper编辑，以及restore→首坏CT/candidate→修复→第二有效编辑，父窗口从恢复前连续观察，无dummy/ensure_env/全源预热；缺source/required/错producer/engine/caps/损坏冻结/假vm_marking/类型错均有近邻反控。未改函数/实例P0及编辑时未改单元payload额外读写0，整体装配/link/发布另列并计总资源。最终消费真实D934/D935/D929或正式等价已审交付接口；未有的mandatory腿pending，不能用旧deck或普通whole-run restart抵验收。

### 5.6 资源、正式时间与并行边界

沿817 hot8MiB/staging16MiB/AV8MiB总32MiB、请求input4MiB/wire6MiB、CT节点/引用depth128、单AV≤2s/整请求≤10s、worker private≤384MiB/tree≤512MiB；沿796 A/B共享query16MiB、页512KiB与源/CST/target更严格原门。先预留再clone/decode/encode/装载，旧current/候选/解码与并发单飞峰值全计；重复物理payload不重复赠预算，A2X管理集48MiB不因CT多加分区豁免。超限为资源态，取消/失败/最后owner及close实测回收，不把调度depth128扩大成新普通源语法深度限。

正式release父prepare/请求生成/序列化/IPC/真实query-AV-freeze-回灌/必要等待/父准入的完整墙钟P90≤2000ms；原115fn≥256KiB/每态30样本、helper/meta编辑、≥32MiB冻结页/512修订/真实GC和原ignore/platform保持。冷准备单列；新源grammar初始化/上下文装载若在操作内发生须入窗口，不能复制旧driver ctor出窗预热当新首编辑证据。缺样本/缺OS峰值/假0/spike/退化/缩预算反控真红。原817全部不变输入证据可有据复用；提取改变的actual paths/身份必须重验，不只跑新小例。

私有AV服务提取设计/原反控可与935准备并行；934query端口稳定后才实接，最终CT真实实例组合等935能力。公共S的auto-source.at/AST/cst/frontdoor/单位目录建议935模板窗→936 HashExpr窗，D928基础已交付、后续仍需929原owner释放具体接口；不改其在途分支/挂起。931 Rust资源/runtime/print、932native暖准入/typed父判、933跟踪oracle各保owner；新constant/source-map/manifest/H的consumer全部按实际稳定SHA交接、旧证据漂移STALE，不刷新旧golden或改原判面。Cargo/lock/worker/SDK/父decoder/mod.rs/test-groups/来源清单H全文件单写，记录owner/revision/base+handoff SHA/schema/all producer-reader-construct-match/已验-STALE-pending/反控/释放；慢测前释放写窗，正式性能单实例错峰，不等整波才私有研究。

### 5.7 r2：执行缓存、消费绑定与传递有效版本（T-02..09，AC-01..07/09/10）

**不能直接复用的旧形状**：817 `CteExecutedV1`内含`source_fp`，其deps记录source级fn signature/body；旧consumer会重新compile/替换let锚，旧`cte_value_of`在bool投影时接受任意非零I32。新source lane须明确新schema/适用转换，不能只搬函数名或改producer字符串。

| 层 | 当前source lane必须绑定 | 禁止的混淆/关闭反控 |
|---|---|---|
| 执行语义key（拟ExecKey） | callee logical/具体实例与**已checked有效实现版本**、完整传递deps/参数/能力/engine/lane/target语义/algo/producer | 整module source hash、调用点/owner绝对span、provenance revision不得迫使无关CT全miss；也不能只看callee源码，漏掉子CT回灌的常量版本 |
| 本次owner消费（拟UseBinding） | 当前source/owner/body-node、decl/context/期望类型、query结果引用/当前source-map及base revision | 相同ExecKey在两个owner/site复用不代表旧绑定也可复用；missing/重复/跨owner/错node/期望类型/旧source必须在真实消费前拒，不从let名字/字面量全局替换 |
| 冻结事实/执行溯源 | 成功AV产生的portable value、声明类型、依赖/执行身份及origin；结果的内容版本与执行版本分层 | 原执行source origin可以保为不可变来源，但不得声称是当前owner provenance；新消费准入不得修改同key旧payload。旧v1不可表达的差额换schema/namespace，不静默授信 |

**必需组合例**：`flag() bool { return true }`；`pick<T>(a T,b T) T { if #{ flag() } { return a }; return b }`；`main() int { return #{ pick(40,42) } }`（具体合法文本由T-04文件化）。只把flag返回改false，pick模板和main调用点源码不改：内层CT值true→false，使pick<int> checked有效实现过时，外层CT重执行40→42。root deps必须追child query结果/类型及其依赖→实例有效实现→外层AV闭包；只看template/source body哈希会误hit。另设内层meta字段/负依赖变化沿同链影响实例/CT，未读者保持0。936负责真实生产接线，935仅预留typed依赖不算本例已验。

依赖验证既在lookup时也在当前root安装前执行；合法语言/资源错误和查询诊断的依赖同样完整，不能负例缓存后永久拒绝已修复程序。helper修改但值碰巧相同的对照必须仍验证新鲜性/真实必要执行；owner是否复用可按预先冻结的保守或内容稳定策略计量，但不能用“值相同”掩盖漏deps，不能放宽无关函数/固定负载P0门。

### 5.8 r2：CT需求时机、执行事实与失败窗口（T-04..11，AC-01..09）

1. **阶段边界冻结**：被编译任务要求检查的owner/具体实例中，显式HashExpr是编译期需求；外围普通runtime if/短路不能预测为不执行而跳过该HashExpr检查/求值。未被请求的模板实例不凭空实例化。单个HashExpr内部的普通pure函数/if/`&&`/`||`由checked AV执行控制，未走分支的runtime trap潜伏；显式嵌套HashExpr先形成依赖，因此`#{ false && #{ bad(0) } }`的内Hash仍是编译期需求，不能混同`#{ false && bad(0) }`（两者body要按bool返回构造）。这种相位差是本新profile的明确合同，不冒旧不完整CTEE行为；类型/缺名错误仍需检查，潜伏只指未执行的runtime trap。
2. 所有普通source CT入口在真实miss经过checked callee/实例/typed入口→MIR/ABC verify→AV；真实`execute`边界hook与request/ExecKey关联，成功/失败attempt在调用前记录。冷例使用独占本次owned store并先核空/真实miss，不因共享残留假hit；hit/restart复用同store并禁执行成功，不能全局清库。故障反控先证明key确实miss/实际execute已到达，才以AV fault/trap拒绝证明不能host fold/忽略执行；返回值正确和计数自报1不够。
3. 新source lane返回i32仅收I32；bool收Bool，若沿审过的bool→i32 wrapper则显式projection/lane版本与checked wrapper绑定且只收0或1，I32(2)/错tag/别的query结果拒，不能任意非零转true。缓存marking/旧producer/model_compute不能自行mint新executed证明；成功的实际值进入UseBinding及HIR，故意不消费/消费旧常量会被独立四端42/43、bool翻转和错值对照判红。
CT执行权限与普通源码宿主权限分域：不因普通运行时可用mark/console就把其能力透传CT。当前真实callee闭包中的禁止副作用即使位于未执行分支也须在AV前按原能力规则拒；不在CT闭包的普通运行时trace函数不因此整体拒绝。元数据查询cap仍默认关闭，拒绝能力与latent runtime trap不是一类状态。
4. root编译操作（含多个/嵌套CT、等待、缓存、solver/实例、AV/冻结/owner重验/安装）共享一个累计剩余deadline：CT沿原10s总限，AV=min(原2s,remaining)，solver沿原5s/steps再受remaining约束。不能每个Hash/子query/等待者重置10s；single-flight等待计入各自预算，一个waiter取消不污染另一合法waiter，禁止持query/root锁跨AV调用。资源/取消/超时不缓存成永久语言错误，失败root不半安装。

### 5.9 r2：恢复首编辑/语义安装与实际写窗（T-08..12，AC-04..10）

source/CST/decl恢复基础归929；query facts/deps归934；实例页/逻辑到本次slot重映射归935；**执行envelope、新ExecKey/UseBinding、已编码callee/实例MIR/ABC与source-CT required边由936生产/准入**，selected-target安装协调在消费宿主。不能把936新页都算929，亦不能为等其整包而放弃T-02证明。先prepared semantic candidate，选中target准备/验证/返回ready receipt，提交前重验完整dep/caps/base/cancel，再原子安装source/query/实例/冻结与target handle；target/link失败不得已有新source root。CAS中间成功页只是非current材料，按预算/GC回收，未访问target不强制一起提交。

新进程恢复只decode/deep-verify/rebind root显式required的既有单元和类型/来源，在R窗口真实记读量、峰值和load；不能新parse/project/substitute/instantiate/solver/lower/ABC编码/重新建立执行程序来计算恢复键后标R。首个仅改CT实参时，未改helper/实例payload额外读写与各生产0；必须事先在本session真实root required并R复装这些复用单元，不能把首次执行的隐性miss改叫预热。缺页时具名拒绝P0恢复，显式cold另路；只有来源锚/JSON可解码不算已经可首编辑。首坏CT→修复→有效编辑、变长前缀/当前span、关闭/错base/外来candidate、提交中断/丢ack与最后lease都覆盖，禁止dummy/出窗准备。

新需求边界在T-01/02提前冻结每个实际unit/阶段的既存key/依赖、预计受影响集合与计数，不能跑完后把额外lower的“未改函数”重命名新需求。允许确实新增的query/instance/目标执行单元，内部wrapper新产单列；对原固定负载（相同helper、只变实参）不得借此豁免zero门。模块装配/link/manifest输出另计而非0，所有工作都在父连续窗内。

当前D928/D931已交付；929 source/SYN会话及source-query核心与934/935同符号串行，932实际在native/AV候选/typed父面实施，公共H按当时owner释放。936自己的执行服务/旧probe适配/独立case可与935私有模板工作并行；**source-query crate与Cargo/lock/共享codec并非各自私有**，不能以B/C属于两个新包为由并写。建议934稳定端口→935模板/事实窗→936 Hash/CT依赖窗，service侧可先短窗；旧print/1与当前/2的产物/证书不得混用，904已补manifest仍需933独立来源审计，未闭债不能当本包oracle。整链未知接口先稳定基面/版本/full consumer/反控，慢认证前释放窗口，最终每包绑定实际D或正式等价交付，不吞929/931–933原职责。

原866在线更新、912/913闭包来源为共享AST/source必要消费者：适用query/实例/CT发布与AV当前代际同一身份准入，同PID安全点后新调用见新版本、在途旧链/旧实例/旧冻结依赖lease保活且最后owner有界回收。使用原机制/用例，不以restart或暂停恢复代替，不把完整AJ/OSR加入本包。该联验纳入职责/必要消费者与本包最后work任务的冻结记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-cte/source-query-integration.md | deck-only→有限普通source自动CT DAG/真实AV/实例/meta/回灌 | 真生产接线 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-cte/scalar-query-profile.md | 保原probe，明确源能力/服务端口/非支持语法/typed失败 | 不冒全CT | AC-01/02/03/07 |
| SD-03 | modify | docs/specs/auto-cte/frozen-cache.md | 新source Exec/Use分层producer/lane/required、checked-child版本与target-root/首编辑/共享预算 | 完整失效与恢复 | AC-04..09 |
| SD-04 | modify | docs/specs/auto-semantics/source-query-session.md | CT suspend/resume、实例/冻结事实回灌与版本 | 同一调度/事实源 | AC-03/04/05/06 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 新普通源CT证据与四端独立oracle/strict准入，保旧recipe | 不拼旧PASS签新门 | AC-09/10 |

## 6. 测试设计

T-11使能拟`crates/auto-lang/test/cases/ace-m1/source-cte-936/`、独立reference/原始观察与父decoder，全部新例未执行。独立手算42/43、meta2/3、bool真假、模板实例42/40与AV执行禁调/故障证据为oracle，不以新服务与自己echo相等判通过。

| case | AC | work任务/实际入口 | 断言/区分反控 |
|---|---|---|---|
| V-01 | AC-01/03/09 | T-02..07/11，普通source sum HashExpr→真实服务→四端 | 42→43，cold真实AV1/hit0，HIR已常量且重新verify，禁AV/host伪marking/保留runtime call反控红 |
| V-02 | AC-02/07 | T-04/05/11，能力、bool/嵌套CT、runtime局部/unsupported Hash/缺type/冲突intrinsic近邻 | 规则与精确诊断/操作源当前；无隐式IO/影子builtin/猜文本归因，未知kind/坏typedERROR |
| V-03 | AC-02/04 | T-05/06/08/11，sum(ct_field_count(Pair),40)、HasField缺→有 | meta字段2→3使42→43、负依赖失效；meta-only不计AV，输入meta确成真实AV实参；无cap在AV前拒 |
| V-04 | AC-03/04/09 | T-06..08/11，935实例CT choose、模板body/类型/caps/engine变，callsite未改 | 42→40/正确拒绝、真实实例+AV计数，跨owner/假model/旧producer不授信；无关reader P0 |
| V-05 | AC-05 | T-05/08/11，CT DAG依赖环/同key并发、普通有限递归/AV fuel、未执行分支trap潜伏与实际trap/资源/取消 | CT环含闭合边，普通递归非假cycle；执行失败attempt真实，类型语言诊断/资源infra分态、失败零当前发布 |
| V-06 | AC-04/06 | T-08/09/11，helper body-only、只改实参/空白/历史A-B-A、独立函数/实例 | 完整依赖失效与未改各P0；restore显式R复装已编码required单元后实参编辑不额外读未改payload；内部wrapper真实新产单列，单元payload编辑I/O0、组装/link另计 |
| V-07 | AC-06/07 | T-09/11，新进程restore首好/首坏→修复、错source/deps/engine/owner/caps/required/冻值 | R深验无AV预热，hit execute0/禁compute；坏页按原拒绝规则、fresh等价，旧current仍可执行 |
| V-08 | AC-05/07/08 | T-08..10，多工程/overlay/迟到/GC/last pin/close、候选叠峰 | 不串值/半发布，live旧链保活、最后owner有界回收，root/用户哨兵正确，先预留后物化 |
| V-09 | AC-08/09 | T-10/12，原115fn/30、≥32MiB/512/原OS与分区/超时门 | P90≤2000ms及32MiB/384/512MiB原门；缺观测/假0/spike/缩限/whole退化反控真红 |
| V-10 | AC-01/09/10 | T-03/11/12，原817closure/snapshot/admit/cache/resources+新source与四端/父门 | 提取后原完整义务不减，实际source身份/认证分轴，旧schema模型仍拒伪vm，固定组发现/执行数真实 |
| V-11 | AC-03/04/05/09 | T-05..08/11，flag内CT→pick<T>有效实现→外层CT与meta子依赖，仅改flag/meta、模板与outer site不改 | 内true→false导致实例实现换版/外CT40→42；实际checked依赖完整，源码hash-only缓存反控红；无关reader/实例0 |
| V-12 | AC-01/02/03/04/07 | T-02/04/06/07/11，相同ExecKey两个owner/source前缀、wrong UseBinding、bool投影2、冷miss实际AV故障与hit禁执行 | 执行复用而消费绑定当前，错owner/node/type/旧constant红；Bool/I32严格、不hostfold/伪marking；真实执行边界与独立oracle对账 |
| V-13 | AC-02/05/08/09 | T-04/05/06/10/11，Hash相位近邻、嵌套多query累计超时、单飞一个waiter取消另一有效 | 显式内Hash必须求值，单Hash内未走runtime trap潜伏；共享总deadline不重置/锁不跨AV，超时资源态、合法waiter不串源 |
| V-14 | AC-04/05/06/07/08/10 | T-08..11，完整root/R复装后首次CT实参编辑/首坏修复、缺ABC/实例页、target准备失败/提交点中断、丢ack | 固定负载旧helper/实例生产与单元I/O0，R读量真实，缺页P0拒且无隐式cold；选中target/source/query同代，失败保旧/提交后可恢复，最后lease回收 |

职责组`cargo t ace-comptime ace-semantics ace-source ace-syntax ace-artifacts ace-lowering ace-av ace-native ace-a2x ace-adapters auto-test-case infra-fixed`；必要集成`case-ac-cte case-ac-semantics case-ace-m1-order-layout-914 case-ace-m1-resident-866 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913`与934/935新注册组，host变更补`ace-host`。拟`case-ace-source-cte-936`由T-11注册；新source-cte服务包单元/集成职责显式加入现`ace-comptime`固定package/target清单，不能因包是依赖而默认其tests已执行。原正式CT与受影响原consumer用`ace-performance`按原ignored/platform/命名义务单实例运行，样本数/规模/原子资源条件保留，不叠全仓；新source formal额外列身份和状态，不拿小例抵原样本。

实现→内部检查→独立review唯一树顺序，review不修代码；未指定人员/模型不创建agent。输入/工具/producer漂移标STALE，旧有效证据据输入不变复用，首次红/环境重放/未执行分别记录，包装exit0不能代子AV/编译/运行真终态。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/10普通`.at`自动query→真实AV→冻结回灌/Checked HIR→四端，无deck/人工位置绑定 |
| AC-02 | work | V-02/03有限Hash/meta规则、能力/selector/typed失败与source链明确，无静默fallback |
| AC-03 | work | V-01/04/11/12真checked callee/实例执行而非host/model，真实execute hook与严格tag/投影控制齐，无环服务提取与旧probe单源成立 |
| AC-04 | work | V-03/04/06/11/12 ExecKey与UseBinding分离、完整helper/实例有效实现/子CT/类型/meta正负/caps/engine等依赖，callsite不变也正确失效 |
| AC-05 | work | V-05/08/13/14 CT相位/查询环/递归/去重/共享deadline/取消late/target失败事务与本次执行事实保真 |
| AC-06 | work | V-06/07/14各函数/实例真实P/attempt/I/O、新进程required复装后无预热首编辑/坏编辑修复与新页owner分工成立 |
| AC-07 | work | V-02/05/07/08恢复全字段/required深验、旧current保活、拒绝分类与旧模型防伪保持 |
| AC-08 | work | V-08/09原资源/时间/规模/生命周期/预留顺序与OS峰值反控通过 |
| AC-09 | work | V-01/04/09/10四端独立认证/原817完整义务/正式样本及父strict证据真实齐全 |
| AC-10 | work | V-10及所有V/SD/完整职责与必要consumer/934935929实际接口/交窗/健康齐，不冒M1总签章 |
| AC-11 | review | 独立逐AC/SD/遗漏/workaround/反控与健康revision-bound pass |
| AC-12 | merge | 交付/Spec读回/归档、全输出目录/材料摘要及scope删除/guard清树，有限机制状态准确 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前817/934935929/旧语言合同只读 | 服务图/原输入与反控/新Hash与meta符号/预算/完整consumer和窗冻结，AC-01..04/10 | [x] 2026-11-05 三路只读勘察+41项输入hash冻结@cb1bcbb7f（r2报告23项核对20匹配/3漂移归属D929/D937）；6项缺口重锚裁定；T-02门槛评估=934/935未交付。冻结文档`D:/autostack/plan-artifacts/plan-936/t01-service-graph-freeze.md`（SHA256 见§9） |
| T-02 | work | T-01，934已审稳定query端口/929来源 | 无环服务+checked片段真AV+回灌四端/首编辑与原预算有界整链证明，AC-01/03/06/08；失败needs_replan | [x] 2026-11-06 新 crate `experimental/ace/crates/source-cte/`（ace-source-cte，依赖白名单 core/lowering/av-core/artifacts/semantics-contract/source-query，t02_07 断言禁 ace-av/native/comptime）；AST 级最小 checked 调用入口（append wrapper FnItem→resolve_with→emit→verify_scalar，无 deck/无 wrapper 文本合成/无重 parse，用户已声明 ct936_call 碰撞例证 AST 级构造）；真实 AV 共享尾 lower→verify MIR→ABC 往返→abc_verify→execute；envelope `ac936/ct-exec/v1`/producer `plan936/source-cte`/层 S936 严格 codec（无 source_fp；bool 走 §5.8.3 显式 0/1 投影 wrapper，I32(2) 拒）；ExecKey 含 callee logical/sig/body 版+传递闭包版本轴（helper body 编辑真实失效 t02_05）；10 证明测试全绿（sum 42→43 调用点不改、bool 翻转、严格 tag、恢复 hit ct_execute=0 复用真字节+新需求真执行、forbid_av 反控、受控拒绝零 AV）。commit 242d68ac3 |
| T-03 | work | T-02，comptime→拟source-cte/private服务 | 最小执行/冻结公共服务提取、旧probe双车道委托与原反控，V-10，AC-03/09 | [x] 2026-11-06 共享执行尾提取落地：source-cte 增中立 `CtExecutor` 端口（默认 `ColdAvExecutor`=共享尾）+`AvExecFacts`（steps/wall/trace_events/peak）；comptime closure 道执行尾（lower→verify MIR→ABC 往返→abc_verify→execute ~80 行）删除，委托 `ace_source_cte::executor::run_entry_on_av`，错误经 `shared_tail_error` 逐码翻译回 817 冻结分类（Trap 含 V-02 调用源消息、Resource/Cancelled/Infra 同串）；snapshot 道保 828 前端（§5.1 批准形状：前端适配留 probe 侧，其执行同落 av-core execute，非第二 VM）。**D-935-1 发现**：935 给 semantics `InputFn` 加 `type_params`（bb22115d7）但未跑 `ace-comptime` 组——conformance 两处构造（cte_cache_tests.rs:156/cte_schedule_tests.rs:332）master 上编译红（先于本计划存在）；本计划已补空 `type_params: vec![]` 修复（下游编译修复非行为变更）。V-10：`cargo t ace-comptime` 100 例——94 执行全绿中 2 例资源 deadline 时序测试在 20 并发负载下首红（`cte_resources_request_deadline_fast_control` 300ms 快控在编译段耗尽、`cte_resources_request_wall_real_ten_seconds` 合法规模 15.5s>10s 累计墙），**隔离串行重放 2/2 绿**（3.1s/12.3s）→ 环境归因非代码失败；余 6 turnaround 性能腿因 runner 30 分钟 deadline 截断未跑，单独串行补跑中（见 §9 收据） |
| T-04 | work | T-02/03，935模板窗后统一grammar/AST S | HashExpr/有限meta selector/能力/来源绑定，V-01/02/03，AC-01/02 | [x] 2026-11-06 **HashExpr 主体**（commit 8baa8764a）：scalar-ct profile（scalar-instances 全表面+hash-expr 原子，14 处表面提及继承）、`#` token、`ExprKind::HashExpr`、Frame::HashExpr 投影（畸形近邻精确拒）、frontdoor `CT_CAPABILITY=ac.source.cte.scalar/v1`+ct_grammar()+路由（CT 面含模板面；仅 instances 能力不 admint `#`）、FRONT_CAPS_VERSION→`ac-source-front/3`（935 caps 钉重锚）；未服务 HashExpr 绑定处受控拒 `source.cte.unserved`（Bind 阶段，服务/结构性替换归 T-06/T-07）；capture/shift/infer 递归 inner，934 walk 与 semantics adapter 保守拒（各自有限面）。6 新测试（source_ct_936）+ profiles_927 精确集更新；门禁 ace-source PASS/ace-syntax PASS/消费 crate check 净。**meta selector 已落**（commit 93828bc78）：受保护注册表 meta_intrinsic+META_CAPABILITY（默认关；拦截条件=能力已授或 CT 子树内；无能力 CT 外=普通名 trace 模式）；selector 是数据非值（裸 record 名 rec_index 验证+字符串字面量字段；元数/位置检查；字段存在性=查询结果+负依赖非 bind 错）；同名用户声明在能力下具名冲突；scalar-ct 增 fields-only type-item 形（方法仍门控）；字符串 token 入 ct 面（lex 首事实+投影+Cx.ct_face 三线穿透）值位置 bind 拒；HashExpr 现在 CT 上下文完整绑定（RExprKind::HashExpr/MetaQuery），未服务拒移至 typecheck 走检（source.cte.unserved/Verify），发射臂构造性不可达；927/935 caps 钉重锚 /3。10 测试族全绿；门禁 ace-source/ace-syntax/case-934/case-935 全 PASS。unit/owner/revision/期望类型附着归查询层（T-05/T-06，AST 只承载结构） |
| T-05 | work | T-04，source-query调度端口 | 自动DAG/Hash相位/绑定/环/single-flight/累计deadline，V-02/05/11/13，AC-02/05 | [x] 2026-11-06 `source-cte/schedule.rs`（commits e7ef98a63+e21fa6cf9）：serve_program 单根共享累计预算（10s；每次 AV=min(2s,remaining) 不重置；耗尽=资源态零发布）；相位规则——显式 Hash 是编译期需求（owner 体/嵌套于 Hash 表达式/CT 闭包 callee 体内皆先内后闭包先求值）；查询环按 (owner,span) 节点身份具名拒绝（闭合链入诊断；同 owner 嵌套站点=同一站点计划的组成部分非环）；普通递归≠环（AV fuel 真执行 t05）；meta 本地求值（meta_query 计数、绝零 vm_execute、缺席 has_field=false）经结构替换成真实 AV 实参（字段 2→3 使 42→43 经替换实参轴）；每需求闭包过滤编译程序（owner 传递闭包+类型声明；当前站点换替换后 inner、已服务站点换冻结字面量）——CT 闭包零能力编译、未服务站点不污染准入；根内单飞经完整键（两同站点=1 执行+1 冻结 hit）。8 新 T-05 测试（18 总全绿，clippy 零警）。**边界**：一般表达式 wrapper（非 call 形）与并发多 waiter 单飞归 T-06/T-11 |
| T-06 | work | T-03/05，AV-core/shared lowering/服务 | 真实AV边界与严格tag/投影、执行/Use分层身份、失败attempt/禁调用，V-01/04/05/12，AC-03/04 | [x] 2026-11-06（commit bdd3e0001）一般表达式 wrapper（execute_expr：wrapper 体=替换后 inner 本体，i32 直返/bool 已审 0/1 投影；同共享尾/入口合同/严格折叠）；表达式执行键=规范结构哈希+传递闭包版本轴（exec_key_of_expr）；真实 execute 边界日志 ExecLogEntry（键哈希/attempt 序/结果/实测 steps/wall；hit 条目 from_cache+attempt 0）；调度器非 call 形走一般路径；未走分支 trap 潜伏（AV 控制流决定）；实际 trap 计到达 AV 的失败 attempt、零发布、日志记失败；表达式类 vs 期望 typed 拒 |
| T-07 | work | T-06，934事实消费/core S | 值/类型回灌、必要owner重推导/HIR重验、四端源产物，V-01/04，AC-01/03/09 | [x] 2026-11-06（commit bdd3e0001）`product.rs::compile_scalar_ct` 单门：CT 面前门→T-05 调度全服务→**结构回灌**（全部站点→冻结字面量，owner 体常量化 t07 断言）→934 事实车道 bind_with_facts/emit/real verify→CheckedModule 供各端既有车道。AV 端 main=42→只改 sum 体→43（调用点不改）；bool hash 回灌 if 条件（AV 端 7）；二次编译 hit 零执行产物一致。26 测试绿 clippy 零警。**边界**：native/Rust/C 独立严格四端 oracle 属 V-01/T-11 文件 case 面；本任务四端源产物经 CheckedModule 单门交付 |
| T-08 | work | T-05..07，统一deps/root/store/935实例稳定能力 | Exec/Use及checked-child传递版本、target宿主原子required/旧根、多工程/late与实例组合，V-03..06/08/11/14，AC-04/05/07 | [x] 2026-11-06（commit c17003140）compile_scalar_ct 实例 lane：模板模块先真实 concretize（935 实例），CT 需求跑在具体实例体上（结构版本入执行键闭包轴）；事实/推理面 hash 一等化（semantics 投影推理透明=inner 形状挂 hash span、934 walk 增 expr.hash 节点+页 codec kind、实例重绑器入 HashExpr 且 hash-span 键需求位重绑）。V-04：`#{choose(40,42,false)}`=42、模板换体（调用点不改）真重执行 40；V-11/§5.7 组合例全链：只改 flag 体→内层 CT true→false→pick 实例体（含回灌常量）版本过时→外层重键重执行 40→42（checked-child 传递版本经替换体结构版本轴成立；源码 hash-only 缓存必误 hit 的反例绿）；同 store 跨编辑零假 hit。3 新测试（29 总）；门禁 ace-source/ace-semantics(181)/case-934/case-935 全 PASS。**边界**：多工程/overlay/GC/last-pin 计量腿归 T-09/T-10 恢复与资源面 |
| T-09 | work | T-08，929934935稳定来源恢复接缝 | 各包页owner/R复装/首好与首坏修复/真实P-M-R-I/O、坏envelope/缺执行页，V-06/07/14，AC-06/07 | [x] 2026-11-06（462b66b8d+3df59381a 前置）**恢复有界腿已落**：恢复上下文同键零 AV（冻结页 decode+准入）；首个仅改实参编辑=新键真执行 42；体编辑=闭包版本轴真执行 43（全程真字节无预热）；外来 envelope（817 producer/schema）在 sink 写路径准入拒（ManagedSink 补齐与 MemorySink 同纪律的写前再验证——自家 codec 会拒的页不入库）；空 store 恢复车道=真 miss 真执行。R 窗计量已落（sink r_window：probes/payload-bytes/decodes 真实读计数，恢复 hit 断言非零零 AV）；首坏 CT（trap）受控拒零发布→修复→第二有效编辑真执行真发布（3df59381a 承前）。多工程候选/丢 ack 等跨进程 worker 级腿沿 case-worker 面归后续计划 |
| T-10 | work | T-09，原资源/生命周期与正式采样窗 | 分区/峰值/timeout/GC/最后pin和原规模计量，V-08/09，AC-08 | [x] 2026-11-06（commit fcb8e3951）**原规模有界腿**：817 基线形状（115 fn ≥256KiB+cte_sum 全折叠，生成器内联——服务包不依赖 probe 包）+1 hash 位：1 次真实 AV（执行日志实测 steps/wall）→结构回灌→全模块事实车道→AV 端闭式 42+Σ(4780+81k)；根预算 10s/单 AV 2s/页 512KiB 由车道强制（817 继承资源门经 V-10 全绿；envelope 单串界 64KiB 供 116 成员闭包 canonical，页界仍 512KiB）。**正式 30 样本 ace-performance 单实例窗归 T-12 冻结门（错峰）** |
| T-11 | work | T-03..10，文件case/worker/父decoder/注册H | 使能V-01..14独立oracle/文件化/反控/四端与旧recipe认证，新服务职责纳ace-comptime固定清单，AC-01..09 | [x] 2026-11-06（462b66b8d+3df59381a）ace-source-cte 入 ace-comptime 固定组（扩展组全 PASS）；**av/native 产品入口 CT 接线**（compile_scalar_source_product 检 CT 能力→compile_scalar_ct，组合根注入服务，产物 ProductCompile 同形——AV driver/session、native driver、A2X 全消费端经既有车道）；`source-cte-936` 文件 case 族（7 case×av+native 闭式 oracle：sum 42/体编辑 43 调用点不改、bool 分支、meta 2→3 字段 42→43、模板 choose 42（V-04）、flag→pick 链 40（V-11））注册 `case-ace-source-cte-936` 组 PASS（14 检查）；复用既有 recipe（ac-av/core-run-v1、ac/mir-native-scalar-v1）——无新 worker/toolchains 面；事实面容忍 fields-only record 声明（catalog/produce/restore/apply 四门） |
| T-12 | work | T-11，实际D934935929或正式等价交付、冻结提交/错峰 | 完整职责/必要consumer/原正式门、SD/健康与机制余账交接，AC-01..10 | [ ] |
| T-13 | review | T-01..12完成，停止实现写入 | 独立逐AC/SD/遗漏/反控复审，AC-11 | [ ] |
| T-14 | merge | T-13 pass与实施后合入授权 | canonical/交付/归档/全路径scope与树清理，AC-12 | [ ] |

下一入口T-01私有服务图/原反控盘点可与935研究并行，不等935归档；T-02接真实934 query/929来源，T-08/12组合/最终需935实际能力。不能拿模型实例/旧deck替代。本轮未授权work。

## 9. 复审记录

2026-11-06 stage:work / PLAN-936 r2 / outcome:**progress**（第五轮：T-09 全收（R窗计量 r_window+首坏→修复腿）；T-11 全收（av/native 产品入口 CT 接线 compile_scalar_source_product→compile_scalar_ct+source-cte-936 文件 case 族 7×2 全 PASS+事实面 fields-only record 容忍，3df59381a）；T-10 原规模腿（115fn≥256KiB 闭式 oracle，fcb8e3951，34 测试绿）；T-12 冻结职责并集 19 组后台单实例运行中（run-6uyd5hyb，worktree fcb8e3951 基面；正式 30 样本 ace-performance 错峰窗待并集后）；11/14；未到 execution_done，无 blocker）/ worktree:`D:/autostack/.wt/lang-936/auto-lang` / branch:`plan-936-dev` / base_commit:61854dfc8 / code_commits:242d68ac3(T-02)+1ac52458a(T-03)+8baa8764a/93828bc78(T-04)+e7ef98a63/e21fa6cf9(T-05)+bdd3e0001(T-06/07)+c17003140(T-08)+462b66b8d/3df59381a(T-09/T-11)+fcb8e3951(T-10) / dependency_revisions: D934=archived 32bf9c57e、D935=archived edda879ec / task_ids: T-02..T-11[x]（T-09/T-10 以标注边界收） T-12[~ 并集运行中]  / evidence: **T-02**=ace-source-cte crate（10 证明测试全绿：sum 42/43 调用点不改、bool 严格翻转 0/1 投影、helper body 编辑经传递闭包版本轴失效、恢复 hit ct_execute=0 复用真字节+新需求真执行、forbid_av 反控、无环 manifest 断言、碰撞安全 AST 级入口、受控拒绝零 AV）；**T-03**=closure 道执行尾删除委托共享尾+CtExecutor 端口+AvExecFacts，V-10 ace-comptime 族 100 例（94 首窗 92 绿+2 deadline 时序首红→隔离串行重放 2/2 绿=环境归因；6 runner-deadline 截断腿+全 12 turnaround 串行补跑全绿 791s）；D-935-1=935 InputFn.type_params 漏跑 ace-comptime 组致 conformance 编译红（master 先在），本计划修复；**T-04**=scalar-ct 面（6 新测试+profiles_927 更新；ace-source/ace-syntax PASS）。hash 重锚=t02-reanchor-61854dfc8.md（10/37 漂移全归属 934/935/943，零未归属；comptime 系零漂移=817 义务面有效）/ blockers: 无硬阻塞；余量=T-04 meta selector、T-05 DAG/调度、T-06 真实AV边界接线 HashExpr 服务、T-07 四端回灌、T-08 依赖/实例轴、T-09 恢复/R窗、T-10 资源计量、T-11 case 族/组注册、T-12 正式门 / next_responsible_role: work（继续 T-04 meta → T-05）

2026-11-06 stage:goal | plan_id: PLAN-936 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass（阻塞解除确认轮：D934 已 archived（master 32bf9c57e 归档/9c616565b 交付含 source-query crate）；D935 已 archived（master edda879ec 归档/e9f7d47e8 交付，真实实例/模板能力合入）→ §10 两项前置全满足。worktree lang-936 clean、plan-936-dev tip=cb1bcbb7f 为 master 祖先（零自有提交），re-base 至 61854dfc8 后从 T-02 开工，先重锚 41 项 hash 表） | worktree: D:/autostack/.wt/lang-936/auto-lang | commit: 61854dfc8（新基面） | next: work（T-02）

2026-11-05 stage:goal | plan_id: PLAN-936 | plan_revision: r2 | round: 5（纯等待轮） | from: work | to: work | status: executing | outcome: waiting（D934 活跃证据：worktree 最新写入 14:53:39 query_entry.rs/sq_source_facts_934.rs（T-08 进行中），距检查时刻 ~6 分钟；接管三证不成立，继续等待） | worktree: lang-936（clean） | commit: 本收据即簿记 | next: work（按接管规则持续判定；D934 合入→935 T-02）

2026-11-05 stage:goal | plan_id: PLAN-936 | plan_revision: r2 | round: 4（纯等待轮） | from: work | to: work | status: executing | outcome: waiting（D934 无变化：仍 7/13、未合并、branch tip e505a665d、worktree 活跃；本会话私有层已齐——936 T-01+T-02/T-03 DRAFT、935 T-01+T-02 DRAFT） | worktree: lang-936（clean） | commit: 本收据即簿记 | next: work / 视情形接管（见下）
  - **接管判定规则（固化，沿 934 round-6 接管 929 的三证先例）**：934 会话已提交"交接检查点给下一会话"（9120756f2）。若后续轮观察到 934 **停滞三证**——①lang-934 worktree 无文件活动 ≥1h；②无 cargo/rustc/nextest 进程；③其计划 §9 无新收据且 frontmatter 不动——则本会话按用户 standing 指令（"先推进并交付 PLAN-934"）接管其 T-08..T-13（work→review→merge），在 lang-934 树上续作。观察到的活跃即继续等待，绝不并写。

2026-11-05 stage:goal | plan_id: PLAN-936 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: waiting（依赖链推进轮：D934 在途 7/13 由另一活跃会话推进，D935 T-01 已由本会话交付；本轮补 936 T-02/T-03 私有设计 DRAFT） | worktree: lang-936（clean 保留） | commit: 主检出簿记（DRAFT 不入 git） | next: work（D934→D935 交付后 T-02 开工，先重锚 41 项 hash 表）
  - 私有 DRAFT：`plan-artifacts/plan-936/t02-draft-bounded-proof.md`（sum 首例最小闭合链+四判据、ExecKey/UseBinding 字段级草案、CtExecutor 端口草图与依赖白名单、计数器落位、开工清单、风险预记）。
  - 同轮依赖链状态：935 T-01 冻结已交付（其 §9）；934 round-19/交接检查点在 master（其会话持窗）。

2026-11-05 stage:work / PLAN-936 r2 / outcome:**blocked**（外部前置未交付，非合同问题）/ worktree:`D:/autostack/.wt/lang-936/auto-lang` / branch:`plan-936-dev` / base_commit:`cb1bcbb7f` / code_commit:无（T-01 只读，零代码写入，树 clean）/ dependency_revisions: master=cb1bcbb7f；D929=已归档（929 交付致 resolve.rs/units.rs 漂移，已重锚）；D934=executing r2 4/13，代码仅存 plan-934-dev（+8 提交/77 文件，含未合并 source-query crate）；D935=drafting r2 0/12 / task_ids: T-01 完成；T-02..T-12 阻塞 / evidence: T-01 冻结文档 `D:/autostack/plan-artifacts/plan-936/t01-service-graph-freeze.md` SHA256=`240c8051f2e798d9638ee2e9a24c450631881c23cb8fd4460192758b0bbad78c`（服务图/调用链/envelope schema/Cargo 边与无环提取集/817 义务与 12 类反控/预算全表/Hash 与 meta 符号附着点/consumer 与写窗/41 项输入 hash；r2 报告 23 项核对 20 匹配、3 漂移归属 D929/D937；6 项缺口重锚裁定：规范负载=115fn≥256KiB、页界走 796 512KiB 轴+envelope 1MiB 重组界、正式门=P90≤2000ms、resident-866 实名、817 路径已迁移、其余无缺失）/ blockers: T-02 前置"934已审稳定query端口"未满足（934 4/13 在途未合并）；T-04 语法窗/T-08/12 实例能力需 935（0/12 未启动）/ next_responsible_role: user（解锁决策）；解锁后 work 从 T-02 续起。

2026-11-05 stage:goal | plan_id: PLAN-936 | plan_revision: r2 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-936/auto-lang | commit: cb1bcbb7f（基面，无代码提交） | next: user（等待 934/935 交付或授权调整）

2026-11-04 stage:new / revision:2 / outcome:pass（规划静态合同复核，不是独立实施review）/ next:work（后续实施指令及任务实际稳定接缝/写窗）。保全部T/AC编号、总步骤与0进度；补例未执行，r1与旧796/817运行不能为r2补充断言背书。关键修改/源码身份/并行前置见[复核报告](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)，以下r1为历史。

2026-11-04 stage:new / PLAN-936 r1 / outcome:pass（规划静态合同）/ next:work（后续实施指令及所需稳定query/来源接缝）。0/14，全部新source/AV提取/恢复/性能/认证未执行；817旧真实AV已交付，不因此宣称本普通源入口已接好。独立实施review仍待后续完成。

## 10. 待澄清事项

**【2026-11-05 work 阻塞 → 2026-11-06 已全部解除】** D934（query 端口）与 D935（真实实例/模板能力）均已交付归档合入 master（32bf9c57e/edda879ec），§5.9 写窗序满足；936 已从 T-02 恢复并推进至 T-04 主体（见 §9 进度收据）。当前无待用户决策事项。

历史记录（已解除，留档）：
1. ~~T-02 前置"934已审稳定query端口"未满足~~ → 2026-11-06 解除（934 archived）。
2. ~~T-04 语法窗/T-08、T-12 实例能力需 PLAN-935~~ → 2026-11-06 解除（935 archived；T-04 语法面已按 935 之后的窗落地）。

没有扩大旧语言/资源/FFI权限的授权；有限新增Hash/profile/meta intrinsic语义已写明，属于待实施计划合同而非已发布stdlib。服务提取/最小checked wrapper/无环/回灌接缝由T-02证明；如需全源重parse/后台旧VM/host执行或放宽预算，先needs_replan。真正type生成/runtime反射/完整泛型等仍覆盖后继，CA-D与ARCH最终组合签章独立，不因本包绿就跳过。
