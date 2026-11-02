# auto-av 常驻函数安装与事务快照（PLAN-828）

> **PLAN-858 开发期周转口径**：ACE 有效样本按升序取第 `N-floor(N/10)` 项（从 1 开始）为 P90，硬门为 P90≤2000ms；正式样本数不减少，10 项最多 1 项超过 2000ms，等值通过。缺失、非法或错误结果不得计入绿；保留各路径原有零值准入、真实窗口、结构与资源断言。P95/P99 作为诊断与历史数据保留；架构基本成形后以实测另定长期目标。详见 [TestCase v2 周转口径](../auto-test-case/profile-v2-native.md)。


> Status: active（实现=`core/src/{function_code,program_snapshot,av}.rs` +
> `probe/src/session.rs`）。

## 单元派发（无 merged 载荷）

- `ProgramSnapshot::link_with_maps`：单一 owner 上下文门（schema+module 全一致）+
  实际调用 import 闭包门（每个被执行的 Call 必须有链接期绑定）+ `fn:<name>` 签名
  等值绑定表（scratch interned 类型表仅作内容等值校验）。**不复制未改 blocks/
  instrs/slots/spans**——执行视图是单元本身（`UnitView`/`ModuleView`）。
- 三公开入口（`execute_program`/`execute_program_steps`/`resume_program`）共用同一
  单元派发；嵌套/递归沿捕获 snapshot 选 callee；分段 resume 预算（段内增量比较，
  累计硬限额跨段）。
- 来源投影：payload span 经帧执行槽投影映射（payload func 号不参与键控）；强制
  复制未改执行 payload 且实际使用时结构门必红（副本负控）。

## 运行中安装（PLAN-866 常驻在线）

- 运行中安装的准入点 = guest 调用边界安全点（见
  [resident-online-reload](resident-online-reload.md)）：armed 段在下一
  Call/CallClosure 派发前停车，commit 在该点发生，随后以新快照解析恢复——
  在途调用链留旧代码至返回，安全点后新派发调用取新 revision。暂停/恢复
  （两次顶层调用之间）语义不变，仍由本规范上文声明。
- 在途被编辑函数（changed 函数在帧栈上）在安全点**延迟安装**：不 commit、
  不 swap、候选留队可观察，新 body 只对下次顶层运行可见——本规范"新顶层
  调用取新 snapshot"规则的常驻形态推论。
- 迟到/外来候选在安全点由本规范的既有守卫（base_revision/session_id）拒绝，
  运行不受扰；安装失败与 guest 失败分列。

## 安装事务与寿命

- prepare 仅构造候选（PreparedEdit 自持候选 HIR/code/来源/绑定）；成功 commit 在
  同一发布点原子更新 root/source/revision/env 槽位；失败/取消/迟到/foreign 不覆盖
  当前可用版（session_id/base_revision 守卫；内部 FnCandidate 另核 env_id/base_generation）。
- 新顶层调用取新 snapshot；在途调用链（PausedRun/pin）固定旧 snapshot 至退出；
  旧代码在 snapshot/frame/lease 引用期间保活，最后 owner 释放即回收（Weak oracle）；
  close 幂等、释放 env/store 拥有引用，外 pin 存活、vacant 执行受控 Infra。
- 每函数内容历史有界（HISTORY_CAP=32，真实淘汰；驻留当前条目永不逐出）。

## PLAN-817：snapshot 车道消费（exec snapshot，追加）

817 消费面接入 828 session：deck `exec snapshot` 车道——warm-up 一次装载全部单元+
link ProgramSnapshot，逐调用实参经 wrapper 增量编辑（尾部锚点对齐），ProgramSnapshot
单元视图执行。车道为缓存身份轴（engine.exec 入 envelope key——closure/snapshot
永不共享条目）；增量纯度（未改函数零重编译/代际不变、warm 为唯一整模块事件）由
lane 退化计数器与 828 计数器双重观测（resident-edit rewarms≠0 即红）；冷启动首调用
即 bool callee 属正常支持面；新能力为追加车道，默认 closure 车道保持，不退回整模块
热重编/装载。消费契约详文见
[auto-cte/scalar-query-profile.md](../auto-cte/scalar-query-profile.md)。
## PLAN-818：mn 消费者的增量继承（SD-06）

- 818 消费 828 函数 IR/代码复用模型：未改函数不重新 lower/codegen（驻留工作
  集+store 持久层）、内容命中=历史单元复用（cv→fp 映射、代际不跳）、每编辑
  恰一单元重发——**新增能力不退回整模块热重编/装载**（退化仅记录性：注入缝/
  异常触发的全模块回退=结构门非零）。
- 用户明确的顺序与增量机制保留：编辑序即提交序（前缀 exactly-once 保留，
  语法回退从 session 当前源重建）；代际/身份变更只落在被编辑单元。
- mn 消费门对每个单元重验证（verify_mir）后安装；emission 失败永不提交
  revision——受检安装/依赖/寿命语义与上节安装事务一致。

## data profile 接入（PLAN-820）

data profile 接入 828 公共函数级增量接口：`lower_function_data_unit`（record
域单函数 AVAD unit）→ `FunctionCode::admit_unit` → `ProgramSnapshot::link`
（fn: 导入解析+签名检查+owner 一致性）→ unit view 执行。**全链增量**：

- 纯 body 编辑走 828 单体事务：字节级分段 diff 命中唯一被改函数（否则显式
  non-fast 拒绝）、等长注释掩码的单函数解析文档（解析面=1）、保留 CheckedEnv
  的 recheck（resolve+typecheck 仅单 body，签名漂移受控拒）+
  emit_function_body（收据=1 真 body/n−1 scaffold）+ body_work 生产窗（未改
  函数零 body 工作）；未改函数 HIR Arc 与 FunctionCode 指针/代际不变、生产
  阶段/IO=0；结构门内建于 edit_body 生产路径（实现退化即使编辑本身红）。真实
  生产事件在调用点计量（verify_mir/decode/verify_abc/admit 逐 def），HIR 身份
  绑定生产 bundle Arc（非消费者克隆）；失败/陷阱编辑零安装（build-aside +
  运行先于发布）。
- 类型/签名/声明集变化走依赖事实计算的精确失效：record mention 嵌套闭包+
  跨 bundle 签名形状等价——签名内容变化→全部 unit 重 lower（unit import 表
  嵌入每个 def 的签名），body 未变函数保留 HIR Arc；helper body 变化不重
  lower 调用者（快照按槽位绑定调用）。
- caps 变化在同一成功发布点同步三面：全 unit 重 lower/admit（HIR 身份
  不动）+ 驻留前端环境按新授权重建（grant 后使用新能力的热编辑可解析、
  被用撤销受控拒且旧值全保、成功撤销后无权热编辑被拒）。
- 跨修订签名失配=link 拒且前快照存活；强制 whole 前端与"只保 hash 换对象"
  双退化反控均使同一结构门非零。

V-13：incremental 族 9/9 + 828 标量套件绿=共享基线。


## PLAN-827 源码编辑消费 D828 会话

PLAN-827 的源码编辑消费已交付 D828 会话。签名/依赖/profile/配置/授权不变的
纯 body 编辑保持函数 HIR/MIR/代码对象与代际不变；未改函数生产阶段与工件 IO
保持零。稳定源基线 273575B/115 函数与至少 30 个不同修订保持独立值、精确
安装目标、结构身份观测与 P90≤2000ms。仅时间绿不能开脱整前端/重装载退化；等
内容强制换对象必须使身份检测器红。源签名或未授权编辑是保留当前修订的记录
性拒绝；helper body 变更不移动仅接口依赖的调用者身份。

暂停链在 resume 期间 pin 原快照并拥有槽位缓冲；完成/取消/drop 释放这些
引用。prepare 永不发布，cancel/drop 保留当前状态，close 释放拥有的条目。
新帧槽位 reserve 失败使用同一生产 dispatch 缝并保留这些所有权规则。持久化
raw 观测必须包含实际基线与当前身份、可归因于受审输入/运行时，并在失败
verdict 下留存。cold/整源/重启路径分开报告；有限源语法拒绝不退回整编译。


## PLAN-821：owned 双通道生产安装会话

PLAN-821 交付 owned 生产安装通道 `owned_session`（828 接口
lower→admit_unit→link→execute 之上）：

- **双通道按编辑真实形状路由**（会话自身结构性证明）：纯单 body 编辑（恰一
  个自由函数声明被触及、全部编辑严格在其 span 内、零 `type` 声明触及）走
  D828 单函数常驻前端——被编函数切片一次 fn-item parse +
  `prepare_candidate_document` 对保留 CheckedEnv 的 re-check/单 body 发射
  （BodyEmitReceipt 生产者证明：real_body_plans=1、其余 body-less scaffold）
  +单单元 lower/verify/encode/admit；**未改单元 parse/recheck/emit 生产
  工作=0**。接口/上下文编辑（type-context 变化、签名漂移被 recheck 接口门
  拒、多声明/声明集变化）走整模块通道——全单元重前端+重产（正确失效语义，
  非静默降级）。
- **scaffold 模式事实**：单函数文档的 body-less scaffold 无参数模式表，
  t_MutRet 包装 ABI 所需 callee mut-record 参数位由会话保留接口事实
  （sig_mode_overrides，冷/整轮捕获）注入 probe 侧 lower；文档自带 body
  恒优先。
- **候选事务化**：前端→失效→逐单元生产→整集受检 LINK→entry RUN 全部成功
  后原子发布（EnvStep 分段：单 body 提交 FnCandidate::commit 一次、整轮
  重建环境，均在发布点应用）；任何失败保 source/revision/identity/
  residents/production/可运行性。**保留声明坐标随发布原子平移**：单 body
  发布携带 (decl_idx, old_hi, delta)——编辑函数 hi 平移、其后全部声明整体
  平移；连续变长编辑恒以当前源坐标路由单 body 道，拒绝编辑不平移；整轮
  路径重建全部坐标。
- **计量**：heap_estimate 计 RecordView 载荷与 FieldMove 路径（解码卫哨
  镜像）——同时存活载荷进保守上界，超 hot 预算输入不估算绿。Production
  六计数（parse/recheck/emit/lower/encode/admit 逐单元）+整模块前端计数；
  **incremental_gate 通道感知**：单 body 编辑整模块前端必须零增、前端三元组
  仅期望集推进；整模块编辑恰 +1 前端、全单元前端三元组推进、backend 仅失效
  集；居民 Arc 身份+mint 代际双保与成员关系（forced whole、对象换不换代际、
  漏失效、静默移除、虚报期望集都红）。

证据：av_owned_821_incremental 7 例（main-only 零未改前端工作、虚报整集
反控、连续变长 1234→8/107→108、失败候选三保）。


## PLAN-822：collections 更新会话（SD-06）

沿 828 增量模型的真实 collections 消费者 `collections_session`：单元仅经真门
（verify→encode→abc-admit→unit-admit，可选 store 持久）入会话；修订仅重建
被提交函数，未改单元由会话携带**同一 Arc 对象/代际不变**、生产阶段与产物
IO=0；`fn:<name>` 经 link 内容等值签名绑定（容器型接口变化结构性拒）。

- **build-aside 原子提交**：候选先 link+执行成功才安装；拒绝/执行失败候选
  不覆盖当前状态，prior snapshot 保持可用。
- **内容指纹域结构 gate**：同内容换对象（强制全量重建形态）unchanged_degraded
  非零 → gate FAIL；诚实修订 pass。
- **授权预检（R18 完整闭包）**：输入 required capabilities 与当前 host
  grants 分离；每次修订对**完整候选闭包**（submitted+carried）逐单元核对能
  力对当前 grants——缺口在任何生产/link/执行/安装前拒（零生产可观察）；
  替换单元以自身声明为准；已发布 snapshot 保留其准入时点授权事实（不回溯）。
- 依赖变化控制：标量签名/容器型接口 layout（link 拒）/内部 layout（精确
  单元重建）/helper body 四轴正反控齐备（V-13 子case 3）。

## PLAN-824：动态 profile 的增量消费者

`DynamicUpdateSession` 消费既有受检单元、link 和快照接口；body-only 编辑、签名/依赖/profile/授权不变时保留未修改函数身份及其生产/IO 零重做。类型、布局或授权变化按当前绑定重验或拒绝，不迁移旧活值。该 typed profile 不证明新增源码或 native Dyn 支持。

## PLAN-823：closures 更新会话与 env 契约

`ClosuresUpdateSession`（822 同型真门会话）消费 828 单元接口：闭包 body 自带
`env` 声明（`MirFunction.env`/`MirImport.env`），`CREATE_CLOSURE.callee` 走
`CalleeRef`（单元经 import 绑定闭包体），captures 逐位等于被调声明；link 期
`fn:` 绑定做 env 等价检查——声明漂移精确拒绝且会话可恢复，不用全模块退化。
body-only 修订未改单元 Arc 同对象/同代际、生产阶段与产物 IO=0；强制退化
（unchanged_degraded）必红。V-13 四子case（body-only 身份/IO、env 漂移精确
失效、强制退化非零、单元独立验证）与 512 真实修订长会话（结构绿 + P90≤2s
r9 口径：10 样本至少 9 次 ≤2000ms，全部正确性门必过）为常驻回归面。cache
抽取/合并透传 env 声明（含 CalleeRef 重写携带被调声明）。

## PLAN-825：驻留索引身份与精确失效

未改 code `Arc` 跨修订保留驻留索引身份与代际（链接时按 Arc 身份复用，同内容新对象不命中）；被改函数仅重建自身表，旧快照经 Arc 保活旧代码/旧表。索引字节并入共享执行 8 MiB 分区账本（跨代单账本），表构建/分配失败先于该函数可分派。编辑会话计数按函数归因，未改函数 build 恒 0 为常驻 oracle（详见 [loaded-program-runtime](loaded-program-runtime.md)）。

## PLAN-826：WireCodec 增量会话

- 828 增量接口（warmup/restore/persist/edit）带显式 wire 策略：`warmup_with_codec`/`restore_with_codec`/`warmup_persisted_with_codec`；默认 legacy 行为逐字节不变。
- compact 行落 codec 维度命名空间：同策略温重启 `restore_units_admitted`=全量，跨代冷回退确定性 miss。
- 未改函数零生产工作/对象/代际恒等在 compact wire 下保持（V-12 四子例实测：builds=1/reuses=2、六阶段计数全零、签名变更精确失效、强制整备无法冒领 reuse）。

## 闭包单函数生产安装（PLAN-912）

`ClosuresFunctionSession` 在 823 会话之上承载闭包源码的**单函数**生产：冷路径整模块
truth 构建驻留前端（目录/创建点事实/受检 registry），暖路径按编辑形状路由（lambda 单
body 快路径 / owner body 复用路由 / 具名非快路径整模块），每候选最多提交被改单元；
未改单元由会话携带同 `Arc` 对象与代际，全阶段生产/工件 IO=0（`WarmGateReport` 结构门 +
P/M/R attempt 轴，whole 轴暖路径必须 0；同 wire 重提交被 823 结构门拒）。候选
`commit_candidate` 唯一安装（stale/foreign 拒）；registry overlay 仅在安装成功后落位。
4MiB 源前门与驻留前端池 ≤16MiB 在会话层执行；A→B `AcceptedSourceState` 只读导出见
[闭包单函数前端生产](../auto-ac/closures-function-production.md)。R3（新进程恢复前置
lower/encode）由 PLAN-913 关闭，见下节。

## 闭包来源绑定恢复安装（PLAN-913）

- 恢复入口不复用 `apply_revision` 的生产面，而是 `restore_install`：接收**已由
  调用方验证路径** admit 的单元（`AdmittedUnit`：code/wire/body_fp/artifact），
  与 `apply_revision` 共用同一安装事务（授权预检 → link → index_build 记录 →
  execute → history/hot/roots → 原子提交），因此恢复不能绕过 823 结构门或原子
  安装；恢复期 `verified/encoded` 恒 0、`admitted` 为单元数。
- 恢复会话用 `new(None, entry)`：无 store root，`close` 不触 store（只读正例）。
  会话不取读侧租约，故"最后 owner 释放"语义不适用于恢复路径；发布期租约是
  OS-backed，进程亡即释放，不遗留永久 pin。
- 恢复后的下一次单 body 编辑仍由 912 的暖路径完成（未改单元同 `Arc`/代际、全阶段
  生产与工件 IO=0）；恢复上下文对需要保留 body 的 owner 路由返回具名非快
  `RestoredContext`，调用方走整模块 truth 并如实计 whole 轴。
- 预算/轴：P 轴正例 0；R 轴按单元记 `Stage::RestartDeepVerify`；manifest/声明
  上下文/单元 wire 全部有界读（超预算读前拒）。详见
  [闭包源码来源绑定恢复](closures-source-recovery.md)。

## 方法单体前端与会话（PLAN-881）

- 方法为 data/owned 会话一级单元公民：outline 方法条目 + 类型壳/方法轴分离
  （方法体编辑不触发整 type）；owner 基单元键 `m.<Rec>.<name>` 贯穿
  居民集/依赖事实/签名事实/计数；bind 计量按 owner 限定键确定性重映射。
- 方法 body 编辑走真单体前端：等长遮罩文档解析保绝对字节坐标（编辑点后
  声明按 delta 平移；owner 类型壳字段真实保留），recheck 按保留条目
  receiver/owner 取 `{Owner}::{name}` sigs 键，候选事务原子发布；owned 会话
  decl_facts 不再因方法关闭快道，identity/body_fp/def_ids/模式事实纳入
  `Method`。
- body-less 方法 scaffold：schema method 声明 body 可选（binder sentinel；
  整模块 verify 仍要求实体）——828 D-01 约定扩展至方法。Static 方法 AST
  span 自 `static` 关键字起（遮罩不悬空）。
- 方法快照寿命=pin 语义（旧 snapshot Arc 仍执行旧码）；866 在线机制不退化
  （14/14）；不冒领复合方法在线 HMR（866 域）。


## PLAN-889：集合source候选的native消费者

`ac-mn::CollectionsIncrementalSession`复用同一CheckedEnv/FnCandidate，纯body走UTF8 fragment、
真实单体resolve/typecheck/plan/bind/shared lower/verify/unit生产；未改AST/MIR/COFF对象身份和0 body工作
独立受检。新intern池与候选共同提交，history可保COFF并重mint目标MIR/坐标；声明/config改变显式失效。
失败/取消/stale/foreign/坏链接保source/env/目录/root/manifest与旧产品，prepared commit重核runtime/config。
冷态有界共用scaffold与仅省全scalar恒空ownership侧表不跳过实际验证。

持久source+mint/current坐标与每单元MIR/COFF/runtime required闭包在重启深验后才link/run，
不恢复trusted凭据。该native静态产物消费不提供活native帧HMR，也不改变旧AV集合source会话的
whole前端差额。预算、性能/512口径及实际限制见
[native collections](../auto-mir/native-collections-variants.md)。

## PLAN-911：拒绝的 typed 只读出口（SD-06）

- `EditOutcome` 新增 `rejection: Option<EditRejection{kind: RefusalKind, phase: Option<RejectPhase>,
  typed_diagnostics: Vec<Diagnostic>, base_revision, changed_fn}>`：在**同一拒绝站点**随本次
  `EditOutcome` 返回（`prepare_edit` 全部 11 站点、`commit`/`cancel_prepared` 的 session/candidate 拒绝、
  env install 拒绝），`typed_diagnostics` 是编译器实际返回的 `Vec<Diagnostic>` 原序克隆（不去重）；
  `diagnostics`（rendered）与 `non_fast_path_reason` 字符串**不变**，仍是同决定的展示投影。
- 不改 prepare/commit 决策、安装、捕获、资源生命周期；不从 session "last errors" 迟读；不 pin AST/HIR/
  code/env。`rejected()` 保留给无 kind 的旧构造（data/resident 会话），scalar 会话全部站点 typed。
- W-S2 真实计量：`frontend_body_attempt:{fn}` 在 `prepare_candidate_document` **调用前**、
  `verify_hir_attempt:{fn}` 在 `verify_function` 调用前 bump（原成功后计数保留为 success 账）；
  `structural_deltas` 的 unaffected 口径不受影响（attempt 键归 changed fn）。
- 909 交付基面上单写；全部 `EditOutcome` 构造/消费者（formal_828/formal_worker/av-worker/resident/
  av_resident tests/native incremental）编译验证，`ace-av`、`case-ace-m1-shared-mutable-909` 组回归通过。
