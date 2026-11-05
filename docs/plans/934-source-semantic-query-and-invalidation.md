---
plan_id: PLAN-934
status: executing
feature_name: SQ-A 普通源码语义查询与依赖失效闭环
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-05
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-semantics/inference-profile.md, docs/specs/auto-semantics/query-cache.md, docs/specs/auto-ac/function-incremental-source.md]
new_spec_components: [docs/specs/auto-semantics/source-query-session.md]
touched_goals: [GOAL-002]
affects: [auto-semantics, auto-cte, auto-ac, auto-av, auto-a2x, auto-test-case]
depends_on: [PLAN-796, PLAN-927, PLAN-928, PLAN-929]
current_step: 4
total_steps: 13
---

# [PLAN-934] SQ-A 普通源码语义查询与依赖失效闭环

## 0. 变更摘要

把796真实有限语义查询从对照探针接入普通Core/Scalar `.at`编译/编辑/恢复，查询事实真实决定共享Checked HIR并供AV/native/A2R/A2C消费。以单体输入、正负依赖、来源分离、统一ManagedStore事务与严格准入保未改函数真实生产0；不先做全源旧推导再假写缓存命中。本包为935真实实例、936普通源码CT提供无环端口；本身不签ARCH-02/04全闭或M1。

## 1. 目标

有限入口完整覆盖当前Core/Scalar支持的i32/bool普通函数/局部/控制流/直接调用与命名绑定；FnSig、InferBody及源声明RecordFields/ListCandidates可查询。原资源/闭包profile继续原权威路径并显式登记未迁范围，不以unsupported静默绕过本包承诺的scalar子集。查询结果为类型事实生产源，core仍独立bind/verify事实与资源计划；不复制求解器、不降Any、不输出可反序列化Checked票据。

非目标：不实现用户模板/CT语法（935/936）、完整重载/spec/HM/跨模块导入/runtime反射/公共ABI、旧产品替换；不改931Rust临时值或932拒绝判面。不修改旧796 S796页含义，新增源query/schema走新版本域。

## 2. 架构方案

拟`experimental/ace/crates/source-query/`位于core与consumer之间，依赖core公开AST/事实消费/verify、796中立求解器与contract、统一artifacts；无AV/native/目标发射依赖。AV/native/A2xSession消费其Checked单元，不能core→source-query或source-query→消费它的AV形成环。若复用sema-store使依赖/单体懒输入不合适，保其已验证codec/查询/预算机制，以中立库差额拆出，不能另建DB/第二算法。

source/CST/decl目录由929提供；保留声明摘要、每body不可变共享输入/依赖，命中时不物化未改AST。新source-sq页（名称拟SSQ1，T-02冻结）分开语义内容与当前source-map，原796全模块src-fp/整Program投影不能直接充新暖P0。共享端口允许后续Instance/CtSuspend节点，但本包不以假实现关闭它们。

## 3. 技术栈

core/source/{adapter,resolve,typecheck,cst}、semantics/{adapter,input,infer,query,reflect}、semantics-contract、artifacts ManagedStore；现AV/native/A2xSession/语言auto-lang-core TestCase桥与固定组。具体SourceTypeFacts/恢复页为新接口，T-02冻结结构与所有producer-reader-construct-match。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户仅授权规划复核/直接修订，不施工、建树、运行测试或派发。初读main `f6b0f918d`，复核期间推进`0771a3b9e`：D928=`6b429502f`已交付（归档计数10/12是簿记残留，不重开）；D931=`84c894b92`已交付归档，Rust print轴为/2，D-931-1恢复首编辑仍由929持有、D-931-3 manifest独立来源审计仍归933；932已executing r2 0/12并有lang-932树，933draft r2 0/10。929executing r2 1/14，D928前置已满足但仍须原会话按用户挂起/恢复规则处理，本会话不代resume。下列f4062cca6/928在途/931draft等均为初订历史，现状态/窗口以本条与[本次复核](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)为准。新source-query/source-cte包仍不存在，不以本规划当实现。

本轮仅授权正式规划/设计，不施工、不建实施树、不运行构建测试/认证、不启动或联系其他会话；未指定实施人员/模型。全部新V例/路径是拟新增，由§8任务使能，不是已执行证据。用户原批准、NA/accepted-diff/source/expected/分母不变；新增有限覆盖以独立case/profile登记，不批量刷新原golden。

796归档9/9、原delivery=`b76eca8d2`；817归档r3 11/11、原delivery=`ec6a54c5d`。两历史SHA本次在当前HEAD均非祖先，不能拿历史PASS直接为重组后的新接线背书；当前安装在experimental/ace/crates/semantics*与comptime*，权威为当前Spec/实际代码与[基线报告](reports/ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)的hash/版本。归档不重开；T-01核当前继承与原反控，实际触面在本包重验。933接手原sema ignored附件，不从该目录读取新证据。

共同设计为[Design42 §10](../design/42-semantic-metadata-inference-and-comptime.md#10-普通源码sq真实实例与av-cte接线2026-11-04target-state)，共享查询身份/所有权/状态/预算/并行边界均为本包合同组成部分。SourceQuerySession、source-query/CT服务与schema为拟接口，必须由有界T-02证明；现有函数/文件是取材，不能假装新API已存在。

调查基面`f4062cca6`；928头9/12在途、9291/14按用户裁定等D928，930头draft但已存在lang-930树，931–933draft r2，不改变其进度。`core/Cargo.toml`目前无semantics/query依赖；`compile_program_parts/prepare_candidate_document`是实际单体接缝；`adapter::project`会遍历所有AST，InferSession读SemaInput，旧InferBody键含src-fp。上述是源码调查，未新复跑。

实施拟唯一树`D:/autostack/.wt/lang-934/auto-lang`，分支`plan-934-dev`（未建）；开工材料仅`D:/autostack/plan-artifacts/plan-934/`登记owner/用途/在途，普通构建在专属树，外置target/consumer/log明确归scope，merge摘要折本计划后全删并guard清树。纯规划无材料scope。

**实施授权（2026-11-05 /goal "完成计划934"）**：授权整链 work→review→有界修复→merge（auto-plan-goal 范式），含建树。分层按用户目标语执行：私有查询工作（T-01盘点/冻结 + 单体facts/读集/键/独立用例/legacy切断方案的DRAFT准备）可先行不等929整包；T-02整链与T-03..08公共接线、恢复验收须消费真实D929或正式等价的已审稳定接口，不把在途分支当D。主检出未提交非文档项均属其他会话（examples/ui/015-notes、test/fixtures/reader-import-budget、scripts/tools/native-fixture/Cargo.lock，929计划§4已具名上报），不入本包、待其owner路由。

## 5. 详细设计

### 5.1 有界起步与真实事实消费

T-01冻结929稳定API/调用清单（含core scalar完整/片段与AV/native/A2X source入口）、796实际profile差额和当前符号版本；T-02在一个普通`.at`上完成源→单体query→事实驱动HIR→AV/native/Rust/C的整链可行性，测试只变leaf body，main只读其签名而不再推导。正常编译入口必须使用新服务，不可另造只在专用query recipe执行的路径冒接线。

有限profile的Ann/模式/算子/命名实参差额必须逐项检查。事实以body表达式/绑定的稳定本体身份关联type和source位置；core消费并验事实覆盖/归属/类型，错事实/漏绑定/旧body拒绝。query miss调真实796 solver（以原算法扩必要adapter而非复制）；core不能在后台再跑全body推导来产生最终类型。solver/emit真实producer反控必须证明命中禁调，不能靠结果碰巧相同。原parse/type诊断stage/code/span/message/顺序若新接线与合同有差异先有界裁定，不改原错误expected凑绿。

### 5.2 键、失效与类型/meta依赖

分离logical(package/module/owner)与signature/body/类型shape/source-map版本。FnSig不读callee body；InferBody只读本体、期望类型、实际callee签名、名字/候选负依赖与配置；RecordFields读有序字段与名义ref闭包/SCC，不把短intern id写盘。各查询key含profile/algo/schema/producer/能力及真实target语义；目标物理layout不污染无关逻辑query键，后端/ABI身份由其下游绑定。

主例`leaf(x int){return x+1}`、`main(){return leaf(41)}`，oracle42；只改leaf为+2得到43，main infer/emit P0，链接/真实运行可非零。另有调用者源码不变但callee签名/参数mode/field shape或候选新增导致正确重验/诊断；无关函数零生产。合法前向递归先用已验证显式签名，不把普通递归当query环。真实不满足的名字→新增定义、字段缺席→出现、名义同形异owner、SCC拓扑与字段序均有正负反控；不以“模块source hash变”代具体依赖。源位置漂移刷新映射/诊断，不重新solve所有未改body。

### 5.3 事务、函数增量与恢复

query pending/in-progress/completed/failed绑定请求revision/cancel；临时工作态、late结果与current不可变事实分开。发布先验证所有required页/嵌入ref/来源/版本，再原子切root；取消/预算/verify错/迟到保持旧程序与source/target关联可用，下一修复成功。single-flight按完整key，两个不同overlay/owner不串值；相同key异payload必须拒并定位，不清库绕过。

新来源恢复依赖929或等价已正式释放的source+CST/声明上下文接口，本包补query事实页/typed准入，不重做929源恢复。R深验当前source/grammar/owner/schema/decl/正负依赖/required闭包后恢复单体事实与IR，缓存不授信；无parse/solve/emit先算key。restore→立即body编辑，及restore→首坏编辑→修复→第二有效编辑，计数器在恢复前安装，无dummy/ensure_env/遍历body预热。未改parse/project/infer/typecheck/HIR/plan/emit/lower/persist与编辑时未改单元payload额外读写0；实际R/decode/M目录与整文件组装/link另计，总墙钟不遗漏。错误源/外包/旧算法/页类型错/缺required/损坏payload受控拒（按原更严格规则，不一律miss），显式cold另走。

### 5.4 资源/时间/交接

A与B共享796语义hot8+scratch8/managed16MiB、单页512KiB、solver5s/1M steps原门；新持久输入与同时存活candidate/旧链/AST/decoder先计真实峰值后物化。源/CST/AV/native/A2X更严格适用预算同时满足，尤其A2X原管理集48MiB包含本次新增保留物，不能16MiB额外赠送。原1MiB/30编辑与≥32MiB页、512修订/GC、4096深依赖链原控制不减；正式release父prepare→query/IPC/产物可用/admit全窗口P90≤2000ms，fresh冷准备分列，样本未执行/缺counter/伪0/越界真红。

D928提供已交付AST/grammar基线，929拥有后续typed来源与恢复公共S面，本包私有query/测试可先准备；T-02整链、T-03..08公共实现需其已审稳定接口和逐符号窗口，最终恢复/正式源认证须真实D929或正式等价交付，不能在途分支当D。931负责Rust资源/print版本，只读其稳定事实；932掌AV/native candidate/typed父判，沿914有效拒绝verdict/坏事实ERROR；933先交跟踪sema fixture/904oracle，本包原sema组不得依赖ignored796附件/豁免已知红。H全文件单写：Cargo/lock、worker/SDK/父decoder/mod.rs/test-groups/来源表，按Design42 §10.7记录全consumer与SHA/STALE；慢测错峰，不新增等待931–933整波的语义依赖。

### 5.5 r2：事实端口、逻辑身份与提交前置（T-02..09，AC-01..06/08/09）

**观察到的缺口**：`adapter.rs::compile_program_observed`目前先`resolve_with`再`typecheck_with`，resolver已经产生`VT/ResolvedExpr`；只屏蔽typecheck或在发射前挂一个query日志不足以换语义权威。796 `InferFacts`以变量name及(start,end)稀疏表出结果，`adapter::project_arg`把命名实参标unsupported。934必须补真正单体事实消费端口/必要neutral输入模型，不能将这三项作为现成能力。

| 必须冻结的规则 | 真实入口/关闭反控 |
|---|---|
| 生产权威切断 | T-02冻结core内“只做名字/作用域/签名绑定”的可复用部分、旧类型综合/推导部分及新事实驱动的Resolved/Emitter入口。ACE正式Core/Scalar source入口（AV session/worker、native source worker、A2xSession与其worker的cold/warm/restore）须默认走本新服务；旧低层compile API可作有边界的兼容/测试参照，不能作为正式consumer隐藏fallback。不是仅为新TestCase选择的额外query recipe。cold miss必须调用真实query solver；新profile legacy类型综合attempt=0，bind/IR结构verify仍执行且真实计量。真实worker入口禁止绕回旧resolver综合、typecheck或重新生成整源事实。 |
| 完整事实身份 | 拟事实索引至少为module logical id+owner/body semantic版本+scope/local声明或表达式的body内结构ID；绑定/调用callee身份、形参映射/期望类型及依赖context版本明确。name、绝对span、intern slot不得单独作键；缺/重复/跨scope/错callee/旧body/不完整覆盖在使用前拒，不能“查不到就infer”。结构ID映射同样校验body与声明，不靠自报hash。持久事实用portable类型/名义ref，不能直接serialize带进程内slot的VT/LocalRef/ResolvedExpr；消费时按当前声明环境重映射且深验。 |
| 有效性与消费可观察 | 每个实际type/绑定消费处给生产侧可核fact-id/type-use记录（拟接口由T-02定），对照独立HIR断言。miss时屏蔽796 solver应失败；给合法事实时屏蔽旧类型综合仍成功；删除/调换同名遮蔽local事实、替换成同签名异callee事实应在core准入红，不能只让附加checker报红、主编译仍绿。hit时禁所有solver producer仍成功。四个方向一起证明，不能只加query报错开关。 |
| 命名实参/内建事实 | 真实沿`E01-ace.at`的`sub(b:mark_b(),a:mark_a())`：value=7且trace=b,a；保source eval顺序，再按param绑定，不能按形参顺序提前求值。796中立输入需携带source-order和绑定map；mark等原能力intrinsic按实际def/caps签名纳入查询，不退回旧推导或假普通名字。 |
| 语义身份与来源 | 928 `UnitCatalog::build`的`module=content_sha(src)`及instance/mint/current span属于目录来源/授权身份，不是持久LogicalModuleId。新语义模块/声明按package+真实module/owner稳定命名，目录key经显式绑定表映射；不可直接把source FNV/Rc地址/ordinal当语义身份。SealedFn载荷是mint span，当前span由目录包装映射，旧pin不变；不全量改写共享AST/span来刷新来源。 |

源query语义key不得含整个模块source hash或无关body/candidate域；source revision/目录instance只做本次请求授权与provenance。失效集合由实际正负reads+声明事件重验而来；928 `DeclChanged.dependents`是语法名字候选，不是真语义dep图。缺/错/省略deps会错误命中，非法fake依赖也不能随payload自声明合法；T-05对实际solver读集、缓存manifest和当前源绑定逐轴对账。负依赖scope/候选粒度先声明，不能靠全module候选版本让无关函数全部重产。

query服务无backend依赖，故**原子选中target的安装点在消费宿主**：prepare source/query candidate→准备该target产物/ready receipt→重验base revision、完整dep versions、caps/cancel→一次切换绑定source/语义root/target handle的current记录。纯FnSig/RecordFields/只读语义query只返回绑定本次revision的view，不要求构建target、不安装执行current；不可把这种view/命中报告冒可执行替换成功。编译目标操作才需要上述target-ready协调。query中间CAS页可留作非current不可变材料，target失败时不得先安装新source/query root；其他target可保独立旧版本，不新增四端必须同时提交的大事务。提交前故障保旧；提交点后丢ack通过原版本/请求身份恢复已提交状态，不伪称回滚。T-06/08按现ManagedStore/AV/A2X设施做最小接缝，不另造总事务框架。提交前dep变更须失效/重试，不能只校验base源码不变。

single-flight共享计算与各waiter的current资格分开：一个waiter取消/过期不能让另一合法waiter串用其source快照或无故失去结果；没有合法owner时停止/回收，任何迟到结果不安装到已变current。执行端口/未来CT suspend不能持有全局query锁/源root锁等待，真正环与等待超时具名，必要等待计父窗口。资源/取消不持久成语言诊断；真正诊断的精确source-map/绑定上下文当前且deps仍完整。

### 5.6 r2：恢复路径分工与当前窗口（T-07..11，AC-04..09）

来源/SYN生产接缝由929完成；934负责额外的query facts/绑定图/solver语义版本/读集页required边及其R准入，935/936各补实例/CT页，不能仅写“等929”而无人提供新页。T-02/07冻结一份实际schema/reader矩阵：哪些是929 source/CST/decl字段，哪些是本包facts/deps/context字段，哪个是消费宿主的target-ready/root记录。新进程只从明确API的restore进入，无`[run]` whole path代替。

R读/解码新facts/依赖图先按长度、元素/字符串/映射数量预留，错引用/重复节点/跨body/坏UTF-8边界在类型消费前拒；预留覆盖decode暂存/clone与同时存活候选，不能先分配大Vec再报资源。R可decode/deep-verify已有声明/IR/事实和结构，不能调用类型综合/solver、重新构造语义body再称verify；所有相关producer在restore前装探针，失败attempt与discarded物化也记录。缺必要页拒绝该P0恢复车道，显式cold另计；合法恢复后的首编辑不得再发现必需上下文缺失而隐式cold。提供原root未编辑恢复、变长前缀/位置漂移、首错误/修复、发布中断/丢ack与closed/stale/foreign对照，不能只测恢复前已经有活env的会话。

当前D928已可读取，不再等待其整包；929仍拥有source/env/manifest/AV-native-A2X会话公共S，原会话恢复/释放以其规则为准。932实际占native/AV候选/typed父判相关面，本包在这些文件不并写；931交付后按D931消费Rust /2/key/mode，旧/1到/2迁移miss单列且不称稳定hit。904 manifest已经随931补入，但D-931-3独立来源未闭，933负责审计/最小纠正；不能继续把它们说成缺文件，也不能据其存在当独立oracle。934私有query可先实施，T-07/08与最终恢复/消费者验收等真实稳定接口，不给933/932整包加无关前置。

原866在线更新、912/913闭包来源为共享AST/source必要消费者：适用query/实例/CT发布与AV当前代际同一身份准入，同PID安全点后新调用见新版本、在途旧链/旧实例/旧冻结依赖lease保活且最后owner有界回收。使用原机制/用例，不以restart或暂停恢复代替，不把完整AJ/OSR加入本包。该联验纳入职责/必要消费者与本包最后work任务的冻结记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-semantics/source-query-session.md | 对照/整输入→有限普通源真实query事实、依赖、会话/恢复 | 生产接线 | AC-01/02/03/04/05 |
| SD-02 | modify | docs/specs/auto-semantics/inference-profile.md | 明确有限普通源consumer及原探针/其他profile边界 | 不冒全求解器 | AC-01/08 |
| SD-03 | modify | docs/specs/auto-semantics/query-cache.md | 新source命名空间/逻辑与catalog身份分离/完整准入/target宿主事务/预算 | 不改旧S796字节语义 | AC-02..07 |
| SD-04 | modify | docs/specs/auto-ac/function-incremental-source.md | source query接缝与真实计数/首编辑联验 | 函数零生产 | AC-04/05/08/09 |

## 6. 测试设计

拟新`crates/auto-lang/test/cases/ace-m1/source-query-934/`（T-09使能）；独立闭式值/手写reference及现前端冻结诊断为oracle，不能拿新solver与新backend互相同意充独立事实。

| case | AC | work任务/真实输入与入口 | 可区分的关闭oracle/反控 |
|---|---|---|---|
| V-01 | AC-01/08 | T-02/04/08/09，普通源Core/Scalar→query→Checked HIR→四端 | leaf42→43、完整i32/bool/命名实参矩阵；禁旧推导/实际query故障反控能使入口红，不是旁挂日志 |
| V-02 | AC-02/03 | T-03/05/09，只改callee sig/mode/type字段/候选，caller源码不改 | 真实读者失效、无关P0；正/负依赖/SCC/名义异owner，source相同不是hit理由 |
| V-03 | AC-03/04 | T-04/05/06/09，leaf body-only、空白/前缀、前向/递归、无变化/历史A-B-A | 签名读者infer0，当前span正确，真实body producer禁调反控和payload计量，不遍历全部body |
| V-04 | AC-05 | T-07/09，新进程restore→首好/首坏→修复→好 | R深验/首编辑无预热、未改P0与单元I/O0，fresh等价；旧source/schema/owner/required损坏反控 |
| V-05 | AC-05/06 | T-06/07/09，候选取消/资源/verify失败、迟到、跨overlay、同key并发 | 旧root/执行结果不变，typed事实/诊断当前，single-flight只共享同key，修复可继续 |
| V-06 | AC-07 | T-02/10，原1MiB/30、16MiB/页/深链/512churn与multiowner/close | 连续P90原门/峰值/GC原规模保持，缺样本/造0/缩限spike必红，不用小例取代 |
| V-07 | AC-02/05/09 | T-03/07/09，S796旧页/错new schema/错solver/位置过时/缺field | 新域隔离、旧span不得授信，bad facts在真实core/父侧拒；原796控制仍可运行 |
| V-08 | AC-08/09 | T-08..11，AV/native/Rust/C原适用source与932/933consumer | 真实编译运行/值与emit-cert分轴；原来源/预算/typed判面不退化，不改原golden/分母 |
| V-09 | AC-01/02/08 | T-02/03/04/08/09，真实source worker的cold/hit，solver/legacy综合切断+同类型错scope/callee事实注入 | miss禁真实solver红；合法事实禁legacy综合仍绿；缺/错事实由实际core拒且不走fallback；hit双禁solver绿，独立HIR/type-use证明消费 |
| V-10 | AC-02/03/04/05 | T-03/05/07/09，整源prefix漂移/同名遮蔽、重启catalog新instance/module哈希、原pin | LogicalModule/Decl稳定、来源授权不串，当前诊断span正确且旧pin不变；不可直接用928 module FNV/绝对span或全源key使全模块miss |
| V-11 | AC-03/05/06/09 | T-05..09，纯meta query无target构建对照、source未变但dep中途更新、target准备失败/提交点故障/丢ack、一个waiter取消另一个有效 | 提交前完整版本重验/旧source+query+target一致，提交后可幂等恢复；无全局锁跨CT等待/错误共享取消；缺页P0拒绝而非warmup |

现职责组`cargo t ace-source ace-syntax ace-semantics ace-artifacts ace-av ace-native ace-a2x ace-lowering ace-adapters auto-test-case infra-fixed`；必要集成`case-ac-semantics case-ac-cte case-ace-m1-order-layout-914 case-ace-m1-resident-866 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913`及T-01冻结的受影响原source cases，触实际host补`ace-host`。拟新固定组`case-ace-source-query-934`由T-09注册；新source-query包的单元/集成职责显式纳入现`ace-semantics`固定package/target清单，不能只注册文件case却漏跑新crate测试。不建第二调度器。最终冻结并集与受影响`ace-performance`原sema/消费样本错峰，原ignored/platform保留，未执行不PASS；不叠全仓轮次。

实现→内部检查→独立final review在唯一实施树顺序交接，未指定模型/人员，不创建agent/chat。review不修代码；输入漂移标STALE，只复用身份未变证据；原红/环境重放分记。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | V-01/09有限普通源query是实际类型事实生产源，完整scope/node事实消费与legacy综合切断反控齐，四端消费同一Checked结果，无后台旧全推导/Any |
| AC-02 | work | V-02/07/09/10身份/完整键/嵌入ref/源坐标与目录module来源分离及旧页隔离可核 |
| AC-03 | work | V-02/03实际正负依赖失效、前向递归/名义/SCC与未读者复用成立 |
| AC-04 | work | V-03真实函数P/attempt/单元I/O与旧caller零生产成立，不隐藏全源投影 |
| AC-05 | work | V-04/05/07/10/11来源恢复深验与新query页owner明确、无预热首编辑/坏编辑修复及拒绝规则经真实929稳定接缝成立 |
| AC-06 | work | V-05/11提交前dep重验、selected-target原子安装、并发/取消迟到/失败保旧/最后owner回收正确，无半发布 |
| AC-07 | work | V-06原资源/样本/时间门与峰值反控通过，未增加预算 |
| AC-08 | work | V-01/08适用四端真实独立认证/共享HIR与其他profile边界正确 |
| AC-09 | work | V-07/08固定职责与必要消费者/反控/SD/交窗/健康齐；796817当前继承准确、933原红未豁免 |
| AC-10 | review | 独立逐AC/SD/遗漏/workaround/健康复审revision-bound pass |
| AC-11 | merge | 实际交付/Spec读回/归档、全输出路径摘要与scope删除/guard清树，不冒M1全签章 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出与验证/AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前基线/原796817/929933只读输入 | 全调用/差额/实际版本/预算与S/H窗口冻结，AC-01..03/09 | [x] 2026-11-05：五要素冻结于 master `f38e50758`，主件 `plan-artifacts/plan-934/t01-freeze-inventory.md`。两报告 hash 表全量重算：仅 2 漂移均出自 D932 merge `da4c98b51`（resolve.rs/test-groups，漂移面已核不伤本包缝）；**932/933 已归档**（相对 r2 快照推进——typed 判面/933 材料现为 master 稳定）；929 executing 10/14（代码 `bf8c189c0`，T-11 在途，非D）。PR-01/02 三缺口 master 现核仍开口（adapter.rs:795 resolve先于typecheck；codec.rs:256 InferFacts name+span稀疏；adapter.rs:363 命名实参Unsupported）；929 消费接口具名清单（decl_env/from_restored/fragment桥/profile_grammar/syn decoder）登记待 D929 交付 SHA |
| T-02 | work | T-01及source/cst/声明稳定接口 | 无环+单体事实消费+恢复首操作/预算的有界整链可行性，失败needs_replan；V-01/04/06，AC-01/05/07 | [x] 2026-11-05：**有界整链通过，不 needs_replan**。代码 `d3226ff69`（crate）+ `bba86bb4b`（事实消费端口/av 用例）；接口冻结件 `plan-artifacts/plan-934/t02-interface-freeze.md`（端口/身份轴/legacy切断设计/恢复交接/预算）。证据：源→每 body 查询（复用 796 solver，他函数仅签名 stub）→事实安装→`emit_resolved_parts`→`verify_scalar`→MIR/ABC→**真实执行 42**，改 leaf body→**43 且 caller 暖命中**（solver 恰 +1）；`body_work` 计量 facts 路径 `typecheck=0`、legacy 对照 1/函数；四方向：冷 miss+solver 关→红、暖 hit+solver 关→绿、篡改事实类型→真实 HIR 校验红、异 owner 页→准入拒。验证：`cargo test -p ace-av --test conformance sq934` 8/8；`cargo test -p ace-source-query --lib` 13/13；`cargo t ace-source ace-semantics` PASS（run-w0t3vybv，冻结提交）。**拆分归属**：legacy 综合切断+命名实参输入=T-04；恢复页/首编辑腿=T-07；预算/峰值正式采样=T-10；四端全消费=T-08；独立 V 用例族/固定组=T-09 |
| T-03 | work | T-02，拟source-query及semantics adapter必要窗 | 单体懒输入/签名目录/新schema命名空间，V-02/07，AC-02/04 | [x] 2026-11-05：`cc9f9b255`。①**单体懒输入**：`ace_semantics::adapter::project_body_scoped(program,target,…)` —— 目标 body 物化，其余函数仅签名 stub（空 body），**不再 clone 整 Program**，无关 body 的越界构造根本不投影（sq14 直接断言 stub 空/翻转 target 生效/Text 模块的 str body 不被投影）。②**签名目录**：`SignatureCatalog`（name→sig 页、logical↔name、排序候选集+指纹），纯声明面——只改 body 时目录内**所有版本与候选指纹不变**（sq15）；session `fn_sig` 与失效判定改读目录（sq17 一致）。③**命名空间准入**：`validate_against`/`check_envelope` 先验 producer/schema/algo/profile，旧 `ac796` schema 永不命中（sq16：旧 schema/异 producer/错 algo/错 profile 全拒，且 `SQ_SCHEMA != SEMA_PAGE_SCHEMA_V2`）。验证：crate 17/17；av `sq934` 8/8 复跑；`cargo t ace-semantics` PASS（run-bu0_9jsy，冻结提交）。V-02 轴（callee 签名漂移/负依赖/候选集）与 V-07（旧页隔离）在本层成立，正式 V-case 族归 T-09 |
| T-04 | work | T-03，core事实消费/solver S | 真query事实消费→HIR/legacy综合切断、命名输入差额，V-01/03/09，AC-01/04 | [x] 2026-11-05：**命名实参**（`bac17cb48`：`InputArg{name,value}` 保 source-order、按名绑参/未知/重复/元数/类型诊断、adapter 投影、canonical 含绑定名；`pick(b: true, a: 7)`=7，位置写法准入红）+ **legacy 综合切断**（`a676fce5c`：core `TypeSupply` 端口与 `resolve_with_supply`——每个 scalar 表达式/`let` 的 VT 由事实页安装、core 的规则降为校验（`source.fact-mismatch`/`source.fact-missing` 在 bind 阶段红，HIR 之前）；`type_synth` 计量：事实路径 `synth=0 / fact_checks>0 / fact_missing=0`，legacy 对照 `synth>0`；source-query `BodyTypeSupply`+`bind_with_facts` 统一查页/准入/绑定/签名安装）。验证：crate 20/20、av `sq934` 10/10（42/43/命名 7 + 坏事实 bind 红 + cold/warm 反控）、`cargo t ace-source ace-semantics` PASS（run-dzy4abt8，冻结提交） |
| T-05 | work | T-03/04，query deps/meta目录 | 完整键/正负/SCC/来源刷新，V-02/03，AC-02/03 | [ ] |
| T-06 | work | T-05，ManagedStore required/root/会话 | source-query candidate/target宿主安装两阶段、dep重验/取消/single-flight，V-05/11，AC-06 | [ ] |
| T-07 | work | T-05/06，929已审稳定来源恢复接口 | 929-source与934-facts页分工、深验/首好与首坏编辑修复/历史/坏页，V-04/07/10/11，AC-05 | [ ] |
| T-08 | work | T-04..07，AV/native/A2X/worker H短窗 | 真实普通源consumer接入与四端认证，V-01/08，AC-08 | [ ] |
| T-09 | work | T-03..08，case/parent/注册H、933稳定材料 | 使能V-01..11独立oracle/负控，新包职责纳现ace-semantics固定清单，AC-01..06/09 | [ ] |
| T-10 | work | T-09，冻结提交/正式采样单实例 | 原资源/时间/GC样本与完整职责/必要消费者，AC-01..08 | [ ] |
| T-11 | work | T-10，真实D929或正式等价接缝 | SD/依赖继承/窗口与余账/健康冻结，AC-09；不能带in-contract pending送审 | [ ] |
| T-12 | review | T-01..11完成，停止实现写入 | 独立逐AC/SD/遗漏/反控复审，AC-10 | [ ] |
| T-13 | merge | T-12 pass与实施后合入授权 | canonical/交付/归档/全路径材料与树清理，AC-11 | [ ] |

下一入口T-01私有盘点不等929整包；T-02及S实现须其已审稳定接缝，最终交付不能缺首编辑腿。frontmatter依赖不是要求所有私有研究先等整波。本轮未授权work。

**T-02 已完成（2026-11-05）**：D929 归档（merge `29a7d7007`）后门槛解除，建树 `D:/autostack/.wt/lang-934/auto-lang`（`plan-934-dev`，基面 `cc7b2fce2`）。`d3226ff69` 新 crate + `bba86bb4b` 事实消费端口：普通 scalar 源经 query 事实驱动 HIR 后**真实执行 42**，body 编辑 43 且 caller 暖命中，四方向反控齐；接口冻结见 `plan-artifacts/plan-934/t02-interface-freeze.md`。

**T-03 已完成（2026-11-05，`cc9f9b255`）**：单体懒输入（`project_body_scoped`：只物化目标 body，其余仅签名 stub，不再 clone 整 Program）、签名目录（纯声明面，body 编辑零版本变化）、页信封准入（producer/schema/algo/profile 先验，旧 `ac796` 页永不命中）。

**T-04 已完成（2026-11-05，`bac17cb48` + `a676fce5c`）**：命名实参按名绑定（保源求值序），以及 **legacy 综合切断**——`resolve_with_supply` 下 HIR 类型全部来自事实页，core 规则降为校验，`type_synth` 计量 `synth=0`（legacy 对照 >0），坏事实在 bind 阶段红。下一入口 T-05。

## 9. 复审记录

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 11 | from: work | to: work | status: executing | outcome: in-progress（**T-04 完成**：命名实参 + legacy 综合切断，事实成为 bind 期类型权威；T-05 下一入口） | worktree: `D:/autostack/.wt/lang-934/auto-lang`（`plan-934-dev`） | commit: `bac17cb48`（命名实参）→ `a676fce5c`（综合切断）；主检出簿记 | next: work（T-05 完整键/正负/SCC/来源刷新 → V-02/03）
  - 综合切断证据：`resolve_with_supply` 下每个 scalar 表达式/`let` 的 VT 来自事实页（`FnResolver::node_ty`/`binding_ty`），core 自身规则仅作校验；`type_synth` 窗口内 `leaf`/`main` 均 `synth=0`、`fact_checks>0`、`fact_missing=0`；同窗口 legacy `compile_scalar_source` `synth>0`（对照）。坏事实（binary 事实改成 bool）在 **bind 阶段** 以 `source.fact-mismatch` 红；部分页 `add_body` 准入拒（sq20）。
  - 命名实参证据：semantics 20/20（bind-by-name 正例 + 位置写法类型错对照 + 混合 + 三诊断 + 按绑定参数类型检查）；av `pick(b: true, a: 7)`=7；source-query sq18 事实 clean 且两种拼写 body version 不同。
  - 验证：`cargo test -p ace-av --test conformance sq934` **10/10**；`cargo test -p ace-source-query --lib` **20/20**；`cargo test -p ace-semantics --lib` **20/20**；`cargo t ace-source ace-semantics` **PASS**（`target/test-groups/run-dzy4abt8/`，退出 0，冻结提交单实例；legacy 路径行为未变）。
  - 说明：`resolve_with`/`compile_*` 旧入口保持 legacy 行为，仅作兼容/差分参照；事实路径已在 av 消费链上真实运行。T-04 行 AC-01/04 的"事实权威 + 综合切断 + 命名输入"三项均具观测。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: in-progress（T-04 部分：命名实参输入整链闭合；legacy 综合切断未开始，T-04 仍 `[ ]`） | worktree: `D:/autostack/.wt/lang-934/auto-lang`（`plan-934-dev`） | commit: `bac17cb48`；主检出簿记 | next: work（T-04 余项：`resolve_with_supply` + 事实权威 VT 安装 + synth 计量反控；随后 T-05）
  - 命名实参（V-01 差额面）：中立输入 `InputArg{name,value}` + `Call` 保 source-order；solver 按名绑参（未知参数/重复绑定/元数/按绑定参数的精确类型检查均出定型诊断，`infer.rs` 5 例）；adapter 不再把 named-arg-call 塌成 `Unsupported`；canonical 内容与预算计数含绑定名（同名异绑=不同语义内容）。
  - 证据：`cargo test -p ace-semantics --lib` **20/20**（含 bind-by-name 正例与**同操作数位置写法=类型错**的对照、混合写法、admission 三诊断、按绑定参数类型检查）；`cargo test -p ace-av --test conformance sq934` **10/10**（`pick(b: true, a: 7)` 真实执行 **7**；同操作数位置写法在事实准入处红）；`cargo test -p ace-source-query --lib` **18/18**（sq18：命名实参事实 clean + 位置写法带 type-mismatch 诊断 + 两种拼写 body version 必不同）。
  - `cargo t ace-semantics` **PASS**（`target/test-groups/run-_hu1zuzy/`，退出 0，冻结提交单实例）。
  - 未完成（T-04 仍开口）：legacy 综合切断——`resolve_with_supply` + 每个 scalar VT 由事实安装、规则仅作校验、`synth` 计量（legacy>0 / supply=0）与"坏事实 bind 阶段红"反控；随后 T-05 读集三方对账。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: in-progress（T-03 完成：单体懒输入/签名目录/新命名空间准入；T-04 下一入口） | worktree: `D:/autostack/.wt/lang-934/auto-lang`（`plan-934-dev`） | commit: `cc9f9b255`；主检出簿记 | next: work（T-04：真 query 事实消费→HIR/legacy 综合切断 + 命名实参输入）
  - 证据：`cargo test -p ace-source-query --lib` **17/17**（新增 sq14 懒输入：目标 body 物化/其余 stub/翻转 target/Text 模块 str body 不投影；sq15 目录纯声明面：body 编辑零版本变化、签名编辑只变该条；sq16 命名空间准入：旧 `ac796` schema、异 producer、错 algo、错 profile 全拒；sq17 session 与目录一致）。`cargo test -p ace-av --test conformance sq934` **8/8** 复跑（T-02 链在 T-03 重构后仍 42/43 且反控保持）。`cargo t ace-semantics` **PASS**（`target/test-groups/run-bu0_9jsy/`，退出 0，冻结提交单实例）。
  - 变更面：`ace-semantics/adapter.rs` 仅新增 `project_body_scoped`/`project_fn_scoped`（`project` 原样保留，796 探针不受影响）；source-query 新增 `catalog.rs`，`produce.rs` 改用它（删除整 Program clone 的 `stub_out_other_bodies`），`facts.rs` 增加页信封，`session.rs` 失效判定改读目录，`apply.rs` 签名页先验信封。
  - 未闭环（按任务表归位）：T-04 legacy 综合切断+命名实参、T-07 恢复页/首编辑腿、T-08 四端消费、T-09 独立 V 用例族/固定组、T-10 预算/峰值正式采样、T-05 读集三方对账细化、T-06 托管存储事务。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: in-progress（T-02 完成：事实驱动整链真实执行 42/43 + 四方向反控；T-03 下一入口） | worktree: `D:/autostack/.wt/lang-934/auto-lang`（`plan-934-dev`） | commit: `d3226ff69`（crate）→ `bba86bb4b`（端口+av 用例）；主检出簿记 | next: work（T-03 单体懒输入/签名目录/新 schema 命名空间 → V-02/07）
  - 证据：`cargo test -p ace-av --test conformance sq934` **8/8 PASS**（真实执行：`leaf(41)`=42；只改 leaf body `+1→+2`=43 且 main 命中、solver_runs 恰 3；冷 miss+solver 关闭→红；暖 hit+solver 关闭→绿；篡改 leaf 二元节点事实为 bool→`verify_scalar` 红；叶事实贴到 main→准入拒；空 caps 同值）。`cargo test -p ace-source-query --lib` **13/13**（含 sq12 事实驱动 emit+verify、sq13 异 owner 拒）。`cargo t ace-source ace-semantics` **PASS**（`target/test-groups/run-w0t3vybv/`，退出 0，冻结提交后单实例运行）。方向：`cargo tree -p ace-source-query --depth 1` 无 av/native/a2x。
  - 计量：facts 路径 `body_work` 每函数 `typecheck=0`、`resolve/plan≥1`；legacy `compile_scalar_source` 对照 `typecheck=1`。即 **typecheck 阶段已切断**；resolve 内综合仍在（T-04 关闭，接口冻结件 §4 已列 4 条关闭面）。
  - 冻结件：`plan-artifacts/plan-934/t02-interface-freeze.md`（端口/依赖方向/应用面边界/身份轴/legacy 切断设计/恢复交接设计/预算）。
  - 未闭环（按任务表归位，不阻塞 T-02 判定）：T-03 签名目录+V-02/07 用例、T-04 legacy 综合切断+命名实参、T-07 恢复页/首编辑腿、T-08 四端消费、T-09 独立用例族/固定组、T-10 预算/峰值正式采样。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: in-progress（D929 门槛已解除，T-02 开工：新 crate + 结构身份/事实页/失效会话落地 11/11 绿；core 事实消费端口未开始） | worktree: `D:/autostack/.wt/lang-934/auto-lang`（分支 `plan-934-dev`，基面 master `cc7b2fce2`=含 D929 归档） | commit: `d3226ff69`（crate）+ 主检出簿记 | next: work（T-02 core 事实消费端口/legacy 综合切断/四方向反对称→V-01 四端值 42）
  - 证据：`cargo test -p ace-source-query --lib` 11/11 PASS（sq01 单体事实完整/结构 id 对齐；sq02 绑定按声明节点；sq03 只改 callee body → caller 命中、body 重解（solver_runs 3）；sq04 callee 签名变 → 真实读者失效并带诊断；sq05 前缀漂移命中且 span 变化；sq06 负依赖名字出现即失效重解；sq07 旧版本/缺覆盖/错 kind/重复/带诊断页全部拒绝；sq08 solver 关闭时冷 miss 红、暖 hit 绿；sq09 有限面外受控拒绝；sq10 签名查询不触 body/solver；sq11 走查确定性与值敏感版本）。`cargo t ace-semantics` PASS（组清单已加 `-p ace-source-query`，`results.json` 在树内 `target/test-groups/run-g16r_x83/`，忽略产物不入库）。
  - 方向核验：`cargo tree -p ace-source-query --depth 1` = ace-core/ace-semantics/ace-semantics-contract/blake3，无 av/native/a2x/lowering/abi。
  - 未完成（T-02 仍 `[ ]`）：core 侧 `ResolvedProgram` 类型事实消费与 legacy 综合切断端口、四方向反对称反控、恢复首操作/预算观察窗、AV/native/Rust/C 至少一端真实值 42。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 6 | from: user | to: work | status: executing | outcome: in-progress（**解除门槛行动**：按用户指令接管停滞的 929——恢复/重派其 T-11..T-14，以产出 D929） | worktree: 934 未建；929 树 `lang-929` @ `bf8c189c0` 由本会话接管执行 | commit: master `3651146f9`→`a9a485a3a`（929 接管收据） | next: work（929 T-11 并集后台运行中→T-12/T-13/T-14 → D929 落地后回本计划建树 T-02）
  - 停滞三证：无 cargo/rustc/nextest 进程；worktree 04:03 起零文件活动（≈5h40m）；stale 锁 pid3376 已死。授权链：用户本轮指令 + 929 round-11 已录用户令（T-07..T-14 全链）。
  - 现场处置：Cargo.lock 非法自依赖（ace-core→ace-core）还原 HEAD；已完成的正式样本（incremental-raw run 1793822173-pid25788，30 样本 pass p90 92.9ms）pid 快照移 plan-artifacts/plan-929/，tracked 刷新随 T-11 冻结提交；stale 单实例锁清除；T-07 漏勾补记。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 5 | from: user | to: user | status: executing | outcome: **blocked**（3 纯等待轮上限：D929 未落地——master 无 929 merge、archive 无 929、929 仍 10/14 @ `bf8c189c0`、WIP 冻结 03:56→09:39） | worktree: 未建 | commit: master `df0f6c57e` | next: user（解除条件与恢复入口见 §10 首条；检查点全保留）
  - 交付状态快照：T-01 1/13 完成；私有层五要素 DRAFT 齐（plan-artifacts/plan-934/）；T-02..T-13 待 D929。无半成品代码/树/scope 泄漏。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 4 | from: user | to: user | status: executing | outcome: waiting（纯等待轮 2/3：D929 仍未落地——master 无 929 merge、archive 无 929、929 仍 10/14 @ `bf8c189c0`；WIP 仍冻结于 03:56，现 09:39 ≈5h43m 无变化） | worktree: 未建 | commit: master `efd8a25c1`（无变化） | next: user（下一轮若仍未落地按上限转 blocked；解除条件：master 出现 929 merge 交付+archive/929-*.md 后 /goal resume）

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 3 | from: user | to: user | status: executing | outcome: waiting（纯等待轮 1/3：D929 未落地——master 无 929 merge、archive 无 929、929 仍 executing 10/14 代码 `bf8c189c0`） | worktree: 未建 | commit: master `918ab7a24`（无变化） | next: user（私有层五要素已齐、无授权内工作可做；D929 落地即自动恢复建树 T-02）
  - 观察：929 worktree WIP（incremental-raw.json/Cargo.lock）自 03:56 起 ~5h40m 字节级无变化（现 09:37）——可能为静机 T-11 长采样，也可能 929 会话已停滞；按合同本会话不代 resume，连续 3 个纯等待轮后将转 blocked 并具名解除条件。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: in-progress（私有层补齐：oracle 语料 DRAFT；D929 复查未落地——929 仍 10/14、非 D，master 无 929 merge/归档） | worktree: 未建 | commit: master `ec292f41f`（无代码写面，本轮零提交需求，收据即簿记） | next: work（D929 落地→建树 T-02；私有层五要素已齐，后续无 D929 前仅剩等待复查轮）
  - 新增 DRAFT：`plan-artifacts/plan-934/t09-draft-oracle-corpus.md`（主例族 42/43、命名实参五例含 E01 事实提取 mark_a=1/mark_b=2→value=7/trace=b,a、i32/bool 矩阵、失效/依赖族、for-in 承诺面裁定注记——§1 承诺含控制流，796 adapter 现 Unsupported 须 T-02 扩真实现而非静默绕过）。

- stage: goal | plan_id: PLAN-934 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: in-progress（T-01 完成 1/13 + 私有 DRAFT 层交付；T-02 前置 929 稳定接口未满足——929 executing 10/14、T-11 正式采样在途，非 D） | worktree: 未建（T-01/私有层无代码写面；T-02 起建 `lang-934`） | commit: master `f38e50758`（规划波次簿记+授权记录）→ `94cc4235b`（T-01） | next: work（D929 落地→建树 T-02 整链；期间仅私有 DRAFT，不触 929 S 面/core）
  - T-01 证据：`plan-artifacts/plan-934/t01-freeze-inventory.md`（两报告 hash 表全量重算仅 2 漂移=D932 `da4c98b51`；932/933 已归档刷新；PR-01/02 三缺口 master 现核仍开口；预算/版本/S-H 窗冻结；929 消费接口具名清单）。材料 scope 已登记 `.retention/policy.json`。
  - 私有 DRAFT 层（T-02/05/09 输入，均标注 DRAFT 待整链终冻）：`t02-draft-facts-port-and-session.md`（职责三分切断+SourceQuerySession 骨架+四方向反对称+新命名域草案+legacy 切断面清单）、`t05-draft-keys-deps-invalidation.md`（键分层/读集/失效矩阵/事务与恢复页大纲，[D929] 项具名）、`t09-draft-v-case-skeletons.md`（V-01..11 骨架+分组运行）。
  - 授权记录（§4 新增）：2026-11-05 /goal 整链授权；私有先行/公共接线消费 D929 稳定接口的分层按用户目标语执行。

2026-11-04 stage:new / revision:2 / outcome:pass（规划静态合同复核，不是独立实施review）/ next:work（后续实施指令及任务实际稳定接缝/写窗）。保全部T/AC编号、总步骤与0进度；补例未执行，r1与旧796/817运行不能为r2补充断言背书。关键修改/源码身份/并行前置见[复核报告](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)，以下r1为历史。

2026-11-04 stage:new / PLAN-934 r1 / outcome:pass（规划静态合同）/ next:work（后续明确实施指令及T-02所需稳定写面就绪）。全部0/13，新机制/例/认证未执行；D796/817历史非祖先事实已记录，不冒新baseline PASS。正式最终review仍待实施完成。

## 10. 待澄清事项

**[2026-11-05 13:10 已解除]** 上一轮的 D929 门控已闭合：`docs/plans/archive/929-unified-source-entry-incremental-retirement.md` 存在（`status: archived`），master `cc7b2fce2` 含 merge 交付 `29a7d7007`；本计划已建树并按 T-02 开工（见 §9 round-7 收据）。原条目保留为历史：

**[2026-11-05 09:39 blocked] D929 门控等待（3 纯等待轮上限触发）**：T-02（首个代码任务）按合同须消费 PLAN-929 已审稳定接口（source/CST/decl：`decl_env.rs`、`UnitCatalog::from_restored`、fragment 桥、统一会话）。现况：929 executing 10/14（代码 `bf8c189c0`），T-11 正式采样在途，其 worktree WIP 自 03:56 起 ≈5h44m 无字节变化（可能静机长采样或会话停滞）；本会话按合同不代 resume。**解除条件**：①master 出现 PLAN-929 merge 交付 + `docs/plans/archive/929-*.md` 后 `/goal resume` 本计划（恢复入口：读 D929 交付 SHA → 建树 `lang-934` → T-02）；或 ②若 929 会话确已停滞，用户先恢复/重派 929 的 T-11..T-14。全部检查点已保留可续（T-01 已入库、私有层 DRAFT 在 plan-artifacts、无 worktree 需清理、主检出仅计划簿记）。无需放宽语言/预算/旧批准。

（原）无需放宽语言/预算/旧批准。单体事实应用API、页格式与复用sema-store方式由T-02有界证明；若必须后台全源check或突破原预算，先needs_replan，不能换名M/R。935936机制、CA-D与最终ARCH签章仍各有owner；本包不因只支持有限类型而豁免真实依赖/恢复。
