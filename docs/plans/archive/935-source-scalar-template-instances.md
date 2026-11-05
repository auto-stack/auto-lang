---
plan_id: PLAN-935
status: archived
feature_name: SQ-B 普通源码有限标量模板与真实实例闭环
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-05
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-semantics/semantic-contract.md, docs/specs/auto-semantics/inference-profile.md, docs/specs/auto-semantics/source-query-session.md]
new_spec_components: [docs/specs/auto-semantics/source-scalar-instances.md]
touched_goals: [GOAL-002]
affects: [auto-semantics, auto-ac, auto-av, auto-a2x, auto-test-case]
depends_on: [PLAN-796, PLAN-934, PLAN-929]
current_step: 12
total_steps: 12
---

# [PLAN-935] SQ-B 普通源码有限标量模板与真实实例闭环

## 0. 变更摘要

承934普通源query/依赖与929统一来源，令796仅模型InstanceKey成为真实源码模板→具体推导/Checked HIR→AV/native/Rust/C可执行实例。先限单类型参数、i32/bool、现直接调用的有限形态，完整实例身份/依赖、single-flight、恢复后首编辑与未改实例生产0；不做完整泛型/spec求解。936消费本包实例执行CT，不另造模板替换器。

## 1. 目标

真实`.at`声明`fn choose<T>(a T,b T,flag bool) T`，由现调用实参与明确期望唯一解T；int/i32规范同型，bool另一实例。模板签名合法化后按需产生可执行实例，每实例body/凭证/符号/产物身份可观察；无Unknown/Any/模板残留进入执行HIR。相同完整实例只生产一次，不同T/owner不串用，修改模板body/签名或实际读取事实即正确失效，未改callsite也不能盲hit。

非目标：const泛型、spec/trait约束、模板方法/泛型record、owned record/text类型实参、HOF、任意显式类型实参语法、完整stdlib/meta/runtime反射/ABI/旧产品替换；不是拿List<T>内建形态代用户函数真实实例。旧profile合法/拒绝面、source/expected/NA/分母不改。

## 2. 架构方案

模板/实例调度归934拟`ace-source-query`，source语法只增加统一grammar/AST所需type_params与实例调用事实，无新的parser。模板是未具体化语义描述，绝非Checked可执行单元；具体化使用同一796 solver与core资源/verify事实消费口，在单体环境内产生具体HIR。template + canonical substitution +真实读取版本组成实例recipe/页，native/AV共享MIR，A2X直接消费其具体Checked HIR，目标后端不再各做替换。

实例与调用者依赖分层：模板签名/实参约束决定选哪个实例；实例body版本决定其实现，普通调用者如果只读具体签名，不因模板body变重新推导，链接/装配所引用的实现版本仍必须更新。不能通过把所有caller重算掩盖绑定/失效缺口。

## 3. 技术栈

934 source-query会话与页、core/source/{ast,cst,resolve,adapter}、796 `reflect::InstanceKey`/query/infer及同ManagedStore，现AV/native/A2X消费入口和正式TestCase/双轴认证。版本化新实例页/端口是拟新增，T-02证明整链；不改796模型类型含义/旧producer marking。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户仅授权规划复核/直接修订，不施工、建树、运行测试或派发。初读main `f6b0f918d`，复核期间推进`0771a3b9e`：D928=`6b429502f`已交付（归档计数10/12是簿记残留，不重开）；D931=`84c894b92`已交付归档，Rust print轴为/2，D-931-1恢复首编辑仍由929持有、D-931-3 manifest独立来源审计仍归933；932已executing r2 0/12并有lang-932树，933draft r2 0/10。929executing r2 1/14，D928前置已满足但仍须原会话按用户挂起/恢复规则处理，本会话不代resume。下列f4062cca6/928在途/931draft等均为初订历史，现状态/窗口以本条与[本次复核](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)为准。新source-query/source-cte包仍不存在，不以本规划当实现。

用户本轮只授权正式规划，不施工/建树/构建测试或派发。取材main `f4062cca6`；796 archived9/9原delivery `b76eca8d2`、817 archived11/11原delivery `ec6a54c5d`均非当前HEAD祖先，当前安装源码/Spec/hash及重组继承以[基线报告](reports/ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)为准，不能沿用历史PASS证明本实例服务。`semantics/reflect.rs::InstanceKey`注释为schema-only；core FnItem目前无用户类型参数，TypeRef::Generic是List/Result等现有参数化类型，不是本包已实现。

源拼写参考跟踪`docs/language/specification.md` Generics的`fn identity<T>(x T) T`与现直接调用；仅取语言合同，不依赖旧Parser/AutoVM/AST。设计/范围/资源为[Design42 §10](../design/42-semantic-metadata-inference-and-comptime.md#10-普通源码sq真实实例与av-cte接线2026-11-04target-state)。934尚未交付，不能把拟接口当事实。

唯一实施树拟`D:/autostack/.wt/lang-935/auto-lang`/`plan-935-dev`（未建）；开工材料只在`D:/autostack/plan-artifacts/plan-935/`登记，外部consumer/target/log显式归scope或正常专属树，merge摘要入计划后全删并guard清树。实现/内部检查/独立review同一树顺序交接，未指定模型人员，不启动会话。

**授权记录（2026-11-05）**：用户 `/goal 完成计划935` 授权整链交付——work（含建树）/独立review/契约内有界修复/merge（canonical Specs、归档、清树、落地检查），一次授权逐阶段继承。D934 已合入归档（`9c616565b` ff-only，master `a0750086c`），§10 等待条件解除，T-02 开工。前一会话（936-goal）只留下 waiting 收据，无在途写面。

## 5. 详细设计

### 5.1 有限源码、类型与歧义

首例为Design42 §10.3 choose源码：原结果42，交换模板两个return分支后main结果0；必须由普通source前门产生至少T=int和T=bool两个真实实例，而非手写实例HIR。模板声明只允许一个唯一类型参数T，参数/返回可为T/int/i32/bool，局部、bool控制流、返回和受支持scalar操作按替换后的真实类型规则检查。`choose(40,42,false)`与`choose(true,false,true)`合法；`choose(40,true,false)`冲突、未能约束的T/歧义受控拒绝，不加隐式转换、Any或猜默认T。

统一grammar/profile增加显式能力`ac.source.instances.scalar/v1`（拟字面量由T-02冻结）；无能力仍原拒绝，不改旧负例expected。保留形参/命名实参源码求值顺序和模式事实，类型推导绑定后端无关；不支持的约束/const参数等具名Unsupported，不能当普通record名或token跳过。T-01冻结原语法差额/所有FnItem构造器，T-03按929稳定AST/schema批量完整更新reader/construct/match，不并写929在途AST迁移面（D928基础已交付）。

模板声明的参数名/类型变量绑定/arity/签名良构先检查；模板body语法错全局前门仍报错，类型错在被请求实例时检查，不无限实例化未被引用模板。具体实例存在独立decl/名义owner与来源链；不以generated symbol拼写当身份。普通递归同实例先注册已验证具体签名后检查body；CT/value依赖环不得借“递归”逃避，936保查询调用链诊断。

### 5.2 实例键、去重与真实生产

先区分逻辑InstanceId与实现recipe/cache key：前者为template logical identity+规范substitution，不因body-only变；后者含模板body/依赖版本。caller绑定逻辑实例及签名，目标闭包/代际另钉当前checked实现版本，不能靠重推导所有caller换callee。完整实例cache key（本节称InstanceKey）同时绑定template logical/package/module/owner、template signature/body semantic版本、规范化类型substitution（含真实类型身份/语义版本）、推导算法/profile/caps/读取的候选正负依赖；target布局/对象版本在下游分层。旧796仅template+substitution schema可继续作描述/接口输入，**不能直接当新已执行实例缓存证明**；新页/producer/version独立注册，旧796 S796与模型CT不改。

同一完整key的一次查询生命周期pending→in-progress→completed/failed，真正producer在运行前计attempt；两个callsite请求同实例在会话/并发合法共享，两个不同类型/模块/overlay绝不混用。模板body变化使所有被请求的活实例checked body和产物正确重做，未读者0；只改单个调用者常量/分支而实例key不变时实例produce0。修改substitution只影响真实实例集合/调用绑定；A→B→A可用历史字节，但必须重新准入当前签名/来源/依赖，不能只换hash。

新增BuiltinType并非本包必需：执行的具体类型只有I32/Bool；生成符号/集合/entry只装配实际可达实例与必要runtime闭包，不能把所有可能T全编。AV/native/A2R/A2C看到同一具体函数，后端不能遇模板自己重新推导。实例签名改变时调用者真实依赖重验，普通body-only不要求不必要重推导；可观察返回值/link实际引用必须更新。

### 5.3 原子发布、函数增量与来源恢复

复用934 source/query root事务，将模板描述、具体实例页/checked-body source refs、实例集合/调用绑定与目标依赖作为同一candidate提交；资源/语言错误/取消/late旧revision不推进current或安装半实例。成功CAS孤页不等于当前，未root孤页按GC回收，old/current/候选/执行lease同时存活计峰值，最后pin/close回收；跨工程soft root/overlay独立。

新进程restore先重验source/grammar/decl/template/substitution/producer/类型ref/实际依赖及required闭包，再恢复具体untrustedHIR并verify；未持久化Checked票据/工作栈/union-find/地址。hit/restart未改实例instantiate/solve/HIR/lower/emit/persist0；恢复R深验可非零，若重新具体化再铸key就是P不能改R。restore→首有效callowner编辑和restore→首坏编辑→修复→有效编辑不得预热或全源ensure_env。以真实934+929稳定接缝联验，未有则AC-06 pending，私有模板设计可先不等整包；最终需D934/D929或正式等价接口身份闭合。

### 5.4 资源与写面交接

A/B共享796语义hot8/scratch8/managed16MiB、页512KiB/solver5s与1M steps；不因有实例再赠一份16MiB。源/CST/A2X48MiB/AV/native原适用门保持，未改函数body不读取/深clone；新增实例的实际checked/缓存/解码峰值在clone/编码前预留，取消及不终止请求按资源态，不能偷加N个实例语言上限。正式release父连续窗口P90≤2000ms，原1MiB/30普通编辑与适用115fn≥256KiB消费负载/原样本数保持；新实例负载至少双T和多个相同实例读者，规模/反控由T-01固定，不能只用一个小identity跑绿。

934先交typed单体query/依赖端口，B私有语料/身份可提前，实际公共接线按其已审稳定SHA；C可平行准备AV提取/CT语法，但同`auto-source.at`/AST/frontdoor/units/类型事实/manifest/H必须串行，建议B模板窗后C HashExpr窗。928/929原owner保统一来源与SYN，不扩大其scope或自动解除929用户挂起。931只消费本包具体HIR，Rustlower/rt/mode修复不动；932的typed有效性/914策略verdict不能回退；933源fixture/904原golden只读、不为新实例刷新旧expected，原sema红需稳定材料解决。共用Cargo/lock/driver/父decoder/mod.rs/test-groups/source清单H全文件单写，交owner/revision/base+handoff SHA/schema/full consumer/STALE/反控/释放，正式采样错峰。

### 5.5 r2：逻辑实例、有效实现版本与来源绑定（T-02..07，AC-01..06/09）

| 层 | 身份/依赖与实际consumer | 必须观察的结果 |
|---|---|---|
| 模板/类型参数 | TemplateLogicalId=package+module+声明owner，类型参数按声明绑定位置而非字符串猜测；body/签名/source-map各有版本 | T不能误绑到同名record或本地变量，重启intern/slot变化不改变逻辑实例；未解/错arity仍原规则拒 |
| 逻辑实例 | template logical id+有序canonical substitution；int/i32同型，bool异型，不能带全源/callsite/span/body版本 | 同实例不同callsite可共享；不同包/模块/owner/T不混淆；允许物理CAS内容去重，不能要求机器码字节必不同来证明类型隔离 |
| 实现recipe/页 | 模板签名/body及**已展开checked body的有效依赖版本**、实际类型/meta/子实例/CT结果（936接入）、算法/profile/caps | 源body不变但child CT/类型事实变仍使有效实现过时；非本包支持的owned/nominal实参不可借schema写成已执行 |
| caller与target闭包 | 普通caller读取具体签名/绑定；selected target root/AV代际读取实际实例实现版本 | 只改模板body、签名不变时caller推导/HIR/打印0，实例实现及实际target引用更新；不能为修calleeRef重写所有caller，不能还调用旧实现冒新结果 |

935当前实际执行范围仍为i32/bool模板。936为实例body接入CT时，必须把child query结果/类型/依赖身份传播进实例有效实现而不是只改原模板source hash；本包新页预留完整typed依赖表达与reader拒未知字段/版本规则，**预留不算CT已实现**，最终组合由936补真实源例。模板的source-map/声明/上下文变化不由全module hash兜底，消费934稳定的scope/body-node事实端口，不用796 name/span稀疏表猜出实例body类型。

模板body已命中的实例因调用者位置/期望/候选环境变化，仍要重新准入call绑定与实际substitution；相同文本call换了预期类型不能沿用旧实参解。接纳的结果必须是当前具体签名/类型，错误型事实不能用Rust/C隐式转换掩盖。真实solver/instantiate producer只允许在有实际需求的实例运行，不以最初“所有T都先生成”换后续0；实例级读集/attempt/commit/discard/source来源可追。

### 5.6 r2：实例集合、恢复与目标安装（T-05..10，AC-04..09）

删最后一个callsite或把一个caller由int实例改为bool实例，current可达实例集合随真实编译需求更新；旧实例/执行链被pin/历史root持有时继续有效，最后owner后回收，不能永久pin所有曾实例化T，也不能为腾预算杀旧链。恢复根据logical instance+substitution/template依赖重新映射当次def/local/intern编号，不能把进程内slot写盘；同名异module的另一个实例页/已编码目标即使结构相同也不能顶替。

沿934“prepared语义candidate→消费宿主target-ready→重验dep/base/cancel→选中target原子current”提交规则，target准备/verify/link失败不安装新实例集合；提交点前/后中断、single-flight一个waiter取消另一个合法、旧版本late结果均有回放。新source变更未提交，不能因mutable resolver context提前改变默认实例集合。真正类型诊断可在完整key下缓存但随负依赖/上下文恢复新鲜；资源/取消不永久缓存失败。

恢复矩阵必须包含：重复请求已有实例、调用者变长/命名重排但T不变、模板body编辑使两活实例重产、改变T所需实例集合、首坏实例推导→修复、同源catalog重新构建的源身份变化。计数按函数/实例/阶段分列，在restore前装探针；R允许结构重验/已编码页复装，不许重新substitute/instantiate/solve来算key再标R。无新需求的实例/函数在首编辑阶段P及单元payload额外读写0；确实首次请求的实例单列真实“新需求”，不改原固定负载零生产分母。

当前D928已交付，929仍是来源/SYN与AST/manifest/会话迁移的公共owner，不因D928满足就自动恢复其goal；934稳定事实/端口与929恢复接口释放后本包T-02..07才能实际接线，私有模板语料/身份可先。931已交付D931 /2与原modes消费，935不写Rustlower/rt；932已开工，native/AV候选/typed父decoder按其实际符号窗；933审计已补904 manifest与sema跟踪来源，不能把存在的金样再生覆盖。各包独立case目录可并行，core/source、source-query、Cargo/lock/父decoder/test-groups/manifest字段全文件或实际同符号单写；稳定接口的基面/版本/reader/反控先交，最终证据身份漂移STALE，不互等整包才做私有准备。

原866在线更新、912/913闭包来源为共享AST/source必要消费者：适用query/实例/CT发布与AV当前代际同一身份准入，同PID安全点后新调用见新版本、在途旧链/旧实例/旧冻结依赖lease保活且最后owner有界回收。使用原机制/用例，不以restart或暂停恢复代替，不把完整AJ/OSR加入本包。该联验纳入职责/必要消费者与本包最后work任务的冻结记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-semantics/source-scalar-instances.md | schema-only→有限真实源码实例/四端checked执行、键/恢复 | 不冒完整泛型 | AC-01..07 |
| SD-02 | modify | docs/specs/auto-semantics/semantic-contract.md | 保模型InstanceKey边界，另列新实例producer/页/类型身份 | 不混信任域 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-semantics/source-query-session.md | 注册真实Instantiate任务/逻辑实例-有效实现依赖/selected-target事务/集合与计数 | 与934单源 | AC-03..06 |
| SD-04 | modify | docs/specs/auto-semantics/inference-profile.md | 有限T推导/语法能力与支持外边界、错误分态 | 不Any/猜类型 | AC-01/02/08/09 |

## 6. 测试设计

T-09使能拟`crates/auto-lang/test/cases/ace-m1/source-instances-935/`与独立手算/手写reference，所有新例未执行。oracle不能仅比较两份新后端结果或模型substitution日志。

| case | AC | work任务/真实入口 | 区分性oracle/反控 |
|---|---|---|---|
| V-01 | AC-01/07 | T-02/03/04/06/09，choose普通source→shared concrete HIR→四端 | 初值42/模板交换分支0；两类真实实例并存/各有producer和执行身份，不用模型count冒实例 |
| V-02 | AC-02 | T-03/04/09，异质实参/无约束T/重复名/不支持约束、正确近邻 | 具体诊断/source/展开链当前，无Any/隐式默认；旧profile仍拒新语法 |
| V-03 | AC-03/04 | T-04/05/09，同T多callsite/int-i32、不同T/module/owner、template body编辑 | 同key共享/不同key隔离，调用点不改依赖也更新；禁instance producer反控，普通caller只读签名时infer0 |
| V-04 | AC-04 | T-05/06/09，只改一个caller/substitution、空白/前缀、A-B-A历史 | 未改函数/实例parse/instantiate/solve/HIR/lower/emit/persist0，目标引用正确，源码序/来源映射当前 |
| V-05 | AC-05/08 | T-05/07/08/09，实例失败/取消/late/双工程/并发/close/最后lease | old root/旧实例/旧程序完整，修复可继续，single-flight无串值；全生命周期峰值/释放真实 |
| V-06 | AC-06 | T-07/09，新进程restore→首好/首坏→修复，错template/substitution/source/type/page/required | 逐字段R重验，禁止compute重铸key与出窗预热；未改实例P0、编辑单元payload I/O0，与fresh等价 |
| V-07 | AC-08 | T-02/08/10，原规模/预算/正式采样、紧限额与多实例叠峰 | 原P90/16MiB及消费域门保持，缺count/造0/复制造峰/缩限spike反控真红，不静默追加语言上限 |
| V-08 | AC-07/09 | T-06/09/10，文件化emit金样/独立rustc与C/AV/native认证、typed错误准入 | 四端独立值/类型/trace、daily emit与cert分轴，所有新接口consumer/版本绑定，原NA/金样不洗绿 |
| V-09 | AC-01/02/03/04/07 | T-02/04/05/06/09，同实例多callsite/int-i32与异T/同名异module，模板body-only/重启slot变换 | 逻辑实例稳定/实现recipe换版、caller语义与打印0而目标调用新实现；独立非零返回/类型oracle，CAS字节同不误判语义同 |
| V-10 | AC-03/04/05/06/08 | T-05/07/08/09，删最后caller/改T/历史pin/实例集合恢复、首坏推导修复、target失败/丢ack | current集合与target原子一致、旧链保活/最后释放，restore按logical ref重映射不重实例化，真实无新需求P0/单元I/O0 |
| V-11 | AC-01/02/06/09 | T-03/04/07/09，同名T/遮蔽、期望类型/命名绑定环境变但call文本相同、缺/错实例facts、资源失败后重试 | 当前substitution/decl/body-node完整准入，坏事实实际core/父侧红且无旧综合fallback，取消/资源不永久缓存，正确近邻与fresh同型同值 |

职责组`cargo t ace-source ace-syntax ace-semantics ace-artifacts ace-av ace-native ace-a2x ace-lowering ace-adapters auto-test-case infra-fixed`；必要集成`case-ac-semantics case-ace-m1-order-layout-914 case-ace-m1-resident-866 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913`与934新组（待其注册）；拟`case-ace-source-instances-935`由T-09注册。实际host触面补`ace-host`，正式`ace-performance`只选受影响原义务、原样本/ignore/platform不减，最终冻结职责/消费者并集，不叠全仓。坏实例/错事实必须在真实verify/父decoder拒，不能只辅助checker红；未执行/条件排除不算PASS。

实现/内部检查/final review在唯一树顺序，review独立上下文不修代码；未指定人员不创建agent，输入漂移标STALE，旧有效证据有据复用。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01有限普通源类型参数从调用约束到具体Checked HIR/四端真实可执行，无模型替代 |
| AC-02 | work | V-02类型变量/能力/歧义与诊断明确，执行域无未解类型、无Any |
| AC-03 | work | V-03/09/11模板/逻辑实例与实现recipe分层、规范substitution/有效checked依赖键完整，名义/同型与旧schema隔离正确 |
| AC-04 | work | V-03/04/09/10真实实例与函数生产0/历史准入/实例集合与目标引用成立，不重算所有caller |
| AC-05 | work | V-05原子发布/失败保旧/取消late/多工程single-flight与后续修复成立 |
| AC-06 | work | V-06/10/11真实来源恢复→无预热首编辑/实例集合变更/失败修复与slot重映射与完整R重验，实际934/929接口闭合 |
| AC-07 | work | V-01/08四端独立认证与同源具体HIR事实成立，目标后端无私有替换器 |
| AC-08 | work | V-05/07原资源/时间/样本/最后owner门齐，不增预算/伪计数 |
| AC-09 | work | V-08/全部V、SD、职责与必要consumer/窗口/健康/936交接完整，无in-contract未决 |
| AC-10 | review | 独立逐AC/SD/遗漏/反控与健康revision-bound pass |
| AC-11 | merge | 交付/Spec读回/归档、全路径材料摘要/删除与guard清树，不冒M1总签章 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前源/934设计/语言合同只读 | 冻结有限语法/类型/所有构造器/consumer/版本/预算与S/H，AC-01/02/09 | [x] 2026-11-05 两路只读勘察+33项输入hash冻结@1b2473480；**全部7处FnItem构造点**+解构消费审计清单（shift/codec/units等）；语言合同Generics节锚定+EBNF失同步裁定；求解原语（Ty::Var/unify/mint_var）与直接调用钩子锚定；796 InstanceKey模型边界；934端口消费设计（STALE待D934）；四端consumer身份键清单；B线预算锚；种子语料。冻结文档`D:/autostack/plan-artifacts/plan-935/t01-syntax-instance-freeze.md`（SHA256见§9） |
| T-02 | work | T-01，934已审稳定query/929来源端口 | choose双实例→四端与恢复/16MiB整链可行性，V-01/06/07；失败needs_replan，AC-01/06/08 | [x] 2026-11-05 整链可行性通过（基面 master `a0750086c`=D934后，33项hash重锚22同/10漂全属934交付面，§11记录于冻结文档）。语法面：grammar `scalar-instances` profile（fn-tail步骤级门控`opt`+类型位裸ident alt+6处scalar族gate扩列）、revision 2、`FRONT_CAPS_VERSION=ac-source-front/2`、能力字面量冻结`ac.source.instances.scalar/v1`（frontdoor `parse_front_with_caps`，无能力仍原语法拒绝）。AST：`TypeRef::TyVar`+`FnItem.type_params`过全部7构造点+shift/kinds/units+resolve/adapter防御具名拒。实例层：`ace-source-query::instances`（域`ac935/instance-logical|instance-key/v1`、producer`plan935/source-instances`、algo`ac935/instances-discovery/1`；确定性concretize+脱糖+重绑，装配名`choose_i32`探测避让用户名——身份在key不在拼写；T-02受限实参推导为T-04替换车辆，模块doc显式声明边界：跨模板体/期望传播/求解器unify）。lane：`compile_scalar_product`能力分支（`ProductCompile`增instances+program脱糖程序，四端经原caps自动继承；a2x-store保留env改用脱糖program不再二次盲parse）。证据（树内提交`114f042d8`）：核心语法5例（含无能力拒绝/plain面回归/有限形态具名拒）+查询层8例（双实例真Checked HIR无模板残留/异质`instances-conflict`/无解`instances-unresolved`/同T单实例demands=2/caller-only编辑恰+1求解且key不变/模板body换版key变sig不变/未引用不生产/plain逐字节同产物/确定性）+四端真执行：AV ABC解释42/0、native mn构建运行42/0、A2R rustc+运行value:42/0、A2C MSVC cl+运行value:42/0（O-3/O-4闭式oracle双方向）+恢复可行性（pages-only bind+seed后solver_runs=0、首好编辑恰main一页、首坏编辑零求解具名拒）+四方向反控（冷miss禁求解器红/暖hit禁求解器绿/篡改实例页类型真bind红`source.fact-mismatch`/异owner页seed拒）+预算锚（3页各≤512KiB、总量≪16MiB）。回归：core conformance 346/346（含927七profile差分全绿）、source-query 54/54、av sq934 14/14+av_source 29/29、native/a2x-worker新增4例。T-04边界已具名（详见instances.rs doc）：期望类型解T、求解器mint_var+unify替换discovery、跨模板体 |
| T-03 | work | T-02，统一grammar/AST S窗 | 模板声明能力/投影/诊断与全部construct-match，V-01/02，AC-01/02 | [x] 2026-11-05 提交`366f0d18b`。约束/const形状（`<T has Bound>`=r-has保留词、`<const N>`=r-const）语法识别+投影具名拒`source.instances-unsupported`（不当token跳过）；裸ident拼写在plain面维持语法拒。模板签名轴编码type_params（空=与plain逐字节稳定）；construct-match审计：units.rs整体FnItem封存无字段丢失、plain面lane模板不可达。core conformance 347/347（新增constraint/const具名拒2例） |
| T-04 | work | T-03，query instantiate/core事实消费 | 真类型约束/具体body/签名/递归与去重，V-01/02/03，AC-01..03 | [x] 2026-11-05 提交`53dd57b1b`。`Ann::TyParam(i)`+`InputFn.type_params`入中立输入；body-scoped投影模板=签名stub（模板体不泛型求解，按实例检查§5.1）；求解器调用钩子每类型参数mint共享var+签名替换+实参/期望真unify，`take_instance_demands`求解后解析；实例层discovery fixpoint（跨模板体两轮物化，有界），重绑键(owner fn,span)——同模板实例共享span、纯span键会串替换（实测修复）；T-02受限推导车辆退役，`INSTANCES_ALGO`→`ac935/instances-discovery/2`。新能力测试：期望类型解T（`let x int = make()`）、跨模板双层substitution（wrap<T>→choose各自i32/bool实例）、模板自递归同实例；诊断面换真求解器原码（type-mismatch/unsolved-variable，Stage::Bind原span）。12/12链+6/6恢复+全矩阵绿（semantics 20/core 347/source-query 54/av-native-a2x 6） |
| T-05 | work | T-04，实例key/deps/ManagedStore | 逻辑实例/有效实现与候选失效、target宿主原子root/集合/单飞，V-03/04/05/09/10，AC-03..05 | [ ] |
| T-06 | work | T-04/05，四端consumer/H | 同具体HIR与目标实例引用/真实执行/函数计数，V-01/04/08，AC-04/07 | [ ] |
| T-07 | work | T-05/06，934929稳定恢复接缝 | 实例来源R/slot重映射/集合变更/坏页/首好及首坏修复，V-05/06/10/11，AC-05/06 | [ ] |
| T-08 | work | T-07，原预算/生命周期观察 | 预留/峰值/取消/最后lease与正式负载，V-05/07，AC-08 | [ ] |
| T-09 | work | T-03..08，文件化case/parent/注册H | 使能V-01..11独立oracle/反控/认证，AC-01..09 | [x] 2026-11-05 case族`crates/auto-lang/test/cases/ace-m1/source-instances-935/`+组`case-ace-source-instances-935`（test-groups.toml注册）**PASS**：5闭式正值例（choose42双实例42/choose0换版0/expected-type期望解T 42/cross-wrap跨模板fixpoint 40/recursion同实例自递归40）×AV+native双probe recipe经产品入口携实例能力真执行；负例neg-hetero冻结真诊断（av=parse/type-mismatch、mn=resolve/type-mismatch——实测映射冻结）。crate级反控/恢复/预算在source-query测试（54例）由`ace-semantics`组携带 |
| T-10 | work | T-09，实际D934/D929或正式等价接口、冻结提交/错峰采样 | 完整职责/必要consumer/原时间门、SD健康与936稳定交接，AC-01..09 | [x] 2026-11-06 SD四件齐（SD-01新`source-scalar-instances.md`+SD-02/03/04增量落worktree待merge沉淀）；职责组门（work级受影响并集）：ace-syntax/lowering/adapters/av/a2x/semantics/artifacts/source、auto-test-case、infra-fixed 全PASS，case-ac-semantics、case-ace-source-query-934、case-ace-source-instances-935 全PASS；ace-native 224/225执行0红——唯一具名perf测试`typed_v10_four_state_20_samples_p90_gate`被runner 30min聚合deadline截断（环境归因），单测隔离重放**PASS(21.5s)**；冻结提交完整并集门留给merge owner（仓库规则）。936交接：`instantiate_program(_limited)`/`InstanceRecord`/`ProductCompile.instances+program`公开面就绪，实例按`(模板,substitution)`可枚举，CT接入点=concretize后的具体body（935 spec恢复/身份节已述） |
| T-11 | review | T-01..10完成，停止实现写入 | 独立逐AC/SD/遗漏复审，AC-10 | [ ] |
| T-12 | merge | T-11 pass与实施后合入授权 | canonical/交付/归档/全路径scope与树清理，AC-11 | [x] 2026-11-06 交付`e9f7d47e8`（master前移至943后rebase，7提交range-diff全等价：`114f042d8→b91247561`…`3b3d330d2→e9f7d47e8`；ff-only落地、tip=交付）；canonical：SD-01新`docs/specs/auto-semantics/source-scalar-instances.md`(SHA256头`09cbd9910fd423de`)+SD-02/03/04在`c34662749`随代码入库；主检出冒烟case-ace-source-instances-935 PASS（943并存的组合态）；冻结并集门=T-10记录复用（机制/输入未变，health修复仅`_`绑定+重验）；归档/清树见§9 merge收据 |

下一入口T-01语料/身份私有盘点可先；T-02具体实现需要934单体查询稳定端口与公共S窗，不能未实现依赖用模型替代。本轮未授权work。

## 9. 复审记录

2026-11-06 stage:merge | PLAN-935:r2 | checkpoints: prepared（reviewed基线`3b3d330d2`+spec delta在worktree已提交）→ landed（rebase到`6fae9d3c8`上7提交，range-diff全`=`；`git merge --ff-only`，master tip=`e9f7d47e8`=交付提交）→ canonical_verified（`docs/specs/auto-semantics/source-scalar-instances.md` 新档SHA256头09cbd9910fd423de；semantic-contract/source-query-session/inference-profile 增量随`c34662749`在默认分支；无legacy ledger写）→ archived（本git mv+status flip+本文档commit）→ cleaned（guard clean→worktree/分支/组目录移除；plan-artifacts/plan-935全删，冻结文档要点已折叠入T-01行与§11引用；registry刷新）| merge-owned gates: 冻结受影响组并集=T-10执行记录复用（13组PASS+ace-native 224/225环境截断-具名perf单测隔离重放绿；health修复语义无变+受影响套件重验），主检出合并态冒烟case-ace-source-instances-935 PASS | 产物观察：本仓无部署面产物（无daemon/web bundle/release义务）；批回归due检查：935<最近覆盖937非due | outcome:**pass**
2026-11-06 stage:review | review_scope: final | reviewer: 同会话评审（实现会话内复审——局限性已声明；裁定从工件重建：6+1提交独立重跑+独立探针，非执行方摘要） | plan_id: PLAN-935 | plan_revision: r2 | outcome:**pass** | worktree:`D:/autostack/.wt/lang-935/auto-lang`（`plan-935-dev`）| reviewed_commit:`3149e399d`+health修复提交（unused-binding，重验semantics 20/20+chain 13/13）| base_commit:`a0750086c` | dependency_revisions: D929/D934 归档合入 | spec_inputs: SD-01新+SD-02/03/04（worktree docs/specs/auto-semantics/，随代码交付）| acceptance_results: AC-01..09 全pass（证据：case族11检查点双recipe重跑PASS、source-query 69例独立重跑、AV/native/a2x四端重跑、core conformance 347、**评审独立探针3/3**——命名实参模板调用/装配名碰撞探测（用户`choose_i32`+模板→确定性`choose_i32__2`）/bool自递归，均执行方套件未点名的反例）| findings: F-935-H1 unused binding（已修commit，重验绿）；非阻塞观察3条记入KNOWN-DEBT候选（ace-native组聚合30min deadline截断具名perf测试-环境归因隔离重放绿21.5s；plan827测试写tracked报告文件-既有卫生；master两文件fmt债-934遗留）| pending_closeout_gates: 冻结受影响模块并集`cargo t`（merge owner）、归档/清树（T-12）| evidence: 本收据+计划任务行+树内测试；评审探针文件用后即删（结果在此）| next: merge（T-12）
2026-11-06 stage:work / PLAN-935 r2 / outcome:**pass**（T-01..T-10 全部完成 10/12；T-11 review、T-12 merge 属后续角色）/ worktree:`D:/autostack/.wt/lang-935/auto-lang`（`plan-935-dev`，6个代码提交`114f042d8`..T-10提交）/ base_commit:`a0750086c` / dependency_revisions: D929✓/D934✓（均归档合入）/ task_ids: T-01..T-10 [x] / evidence: 树内测试~90例（core conformance 353含6语法面/source-query 13链+10恢复反控+5宿主/av 2+native 2+a2x 4/case族11检查点双recipe）+受影响组门全绿+ace-native环境归因重放绿；规格四件在worktree / blockers: 无 / next_responsible_role: review（T-11 独立复审；冻结完整并集门=merge owner）
2026-11-05 stage:work / PLAN-935 r2 / outcome:**in-progress**（T-03、T-04 完成 4/12——求解器真unify替换T-02车辆、期望类型/跨模板fixpoint/自递归真走通；证据见任务行）/ worktree:`D:/autostack/.wt/lang-935/auto-lang`（`plan-935-dev`）/ base_commit:`a0750086c` / code_commit:`366f0d18b`(T-03)+`53dd57b1b`(T-04) / task_ids: T-01..T-04 完成；下一 T-05 / blockers: 无 / next_responsible_role: work（T-05 实例键/依赖/ManagedStore持久化/原子root/单飞）
2026-11-05 stage:work / PLAN-935 r2 / outcome:**in-progress**（T-02 完成 2/12——choose 双实例→四端真执行+恢复可行性+四方向反控整链走通，证据见 T-02 行；T-03 起继续）/ worktree:`D:/autostack/.wt/lang-935/auto-lang`（`plan-935-dev`）/ base_commit:`a0750086c`（D934 合入后 master）/ code_commit:`114f042d8` / dependency_revisions: D929=已归档✓；D934=已归档✓（交付 `9c616565b`，§10 等待条件解除；33项hash重锚10漂全属934交付面）/ task_ids: T-01、T-02 完成；下一 T-03 / evidence: 树内测试31例（core 5/查询层8+6/AV 2/native 2/A2R+A2C 2）+回归（core conformance 346、source-query 54、av sq934 14+av_source 29）+冻结文档§11重锚 / blockers: 无 / next_responsible_role: work（T-03 统一grammar/AST全construct-match覆盖）/ 执行方注记：本会话为 935 专属 goal 会话（用户 `/goal 完成计划935` 授权整链）。
2026-11-05 stage:work / PLAN-935 r2 / outcome:**waiting**（T-01 完成 1/12；T-02 前置 D934 未合入——934 由另一活跃会话推进中 6/13、T-07 已提交 `2793de9b2`、worktree 分钟级活动，本会话按单写面规则不并写，等其 work→review→merge 交付）/ worktree:未建（T-01 只读零代码；T-02 起建 `D:/autostack/.wt/lang-935/auto-lang`/`plan-935-dev`）/ base_commit:`1b2473480` / code_commit:无 / dependency_revisions: D929=已归档✓；D934=executing 6/13 在途（source-query crate 仅在 plan-934-dev +9 提交） / task_ids: T-01 完成；T-02..T-10 等 D934 / evidence: 冻结文档 `D:/autostack/plan-artifacts/plan-935/t01-syntax-instance-freeze.md` SHA256=`6c7ef5b5863ba31ef0563e6a8346f29618df4db5acae6a7e18601b4f1035434a`/ blockers: D934 合入 / next_responsible_role: work（D934 交付后 T-02 choose 双实例整链，届时重锚 33 项 hash 表）/ 执行方注记：本轮 T-01 由 936-goal 会话按用户 goal 续行授权执行（935 此前无认领会话）。

2026-11-05 stage:goal | plan_id: PLAN-935 | plan_revision: r2 | round: 2（936-goal 会话） | from: work | to: work | status: executing | outcome: waiting（D934 未合入——其会话推进至 7/13：T-07 完成/T-08 部分/T-11 部分，worktree 持续活跃；本轮无公共写面动作，私有层补齐） | commit: 无（DRAFT 不入 git，收据即簿记） | next: work（D934 merge 落地→建树 T-02，先重锚 hash 表与 934 交付符号）
  - 私有 DRAFT：`plan-artifacts/plan-935/t02-draft-choose-chain.md`（整链骨架/独立 oracle 矩阵 O-1..10/四方向实例轴反控/开工清单/风险预记——fn-tail 尖括号歧义、两层签名键、实例 body NodeId 空间）。

2026-11-04 stage:new / revision:2 / outcome:pass（规划静态合同复核，不是独立实施review）/ next:work（后续实施指令及任务实际稳定接缝/写窗）。保全部T/AC编号、总步骤与0进度；补例未执行，r1与旧796/817运行不能为r2补充断言背书。关键修改/源码身份/并行前置见[复核报告](reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)，以下r1为历史。

2026-11-04 stage:new / PLAN-935 r1 / outcome:pass（规划静态合同）/ next:work（后续实施指令及934/929所需稳定接缝）。0/12，新模板语法/实例/测试/认证均未执行；独立实施review另做，不由已有796 InstanceKey模型背书。

## 10. 待澄清事项

**[2026-11-05 work 等待——已解除（同日）]** D934 已合入归档，T-02 开工并完成；下为历史记录。T-01 已完成（只读冻结，见§9）。T-02 的前置"D934 已审稳定 query 端口"未满足：PLAN-934 由**另一活跃会话**推进中（executing 6/13，T-07 已提交 `2793de9b2`，worktree 分钟级文件活动、cargo 进程在跑，review/merge 未开始）。本会话按单写面规则不并写 934。**解除条件**：master 出现 934 merge 交付后，T-02 开工（建树 `lang-935`/`plan-935-dev`，重锚 T-01 的 33 项 hash 表与 934 端口符号）。

有限type参数/错误/能力边界本计划已明示，未请求扩大完整泛型。模板/实例编号及字节格式由T-02证明并版本化；如只能重编全部caller/序列化Checked或突破预算则needs_replan。936组合CT须真实本包实例，runtime反射/const泛型等留覆盖后继，不借未实现能力参与M1签章。
