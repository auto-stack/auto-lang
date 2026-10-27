---
plan_id: PLAN-823
status: executing
feature_name: AV类型化闭包环境与捕获生命周期
plan_revision: 10
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-27
current_step: 9
total_steps: 11
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
  - docs/specs/auto-ac/artifact-contract.md
new_spec_components:
  - docs/specs/auto-av/closure-environments.md
  - docs/specs/auto-mir/closure-capture-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-823] AV类型化闭包环境与捕获生命周期

## 0. 变更摘要

2026-10-27 用户要求解决已交付 824 与本计划的编号和接口冲突。r10 保留 824 的 AVAD v4/profile 5、Dyn tag 16 和指令 0x2F–0x31；将尚未合入的 823 闭包身份改为 **AVAD v5/profile 6、Closure tag 17、指令 0x32–0x34**，同时要求两种 profile 的真实首例、恢复、身份交叉拒绝及直接/嵌套闭包 pack/freeze 拒绝与 824 合法值正控。旧 v4/id5/tag16 及 0x2F–0x31 为整合前历史，不能作为现行身份。r9 延迟、其余功能/结构/资源验收和 11 项任务保持不变；现行进度以最新 work 记录为准。

2026-10-27 用户调整增量编译延迟目标：本计划实际增量编译按 **10 次样本、至少 9 次不超过 2000ms（P90 ≤ 2s）** 验收；每次正确性和结构门仍必过，功能 churn/GC 与资源预算不削减。r9 覆盖本计划中与此冲突的历史 P95/样本数表述；用户确认计划 823 的 798 兼容门也采用新口径；798 旧合同和过去测量作为历史保留。GC 发现已删除块及已提交事务元数据随编辑数增长，受影响 work 包含共享 artifact-store GC 的终态元数据清理、真实恢复/重新发布与长期空间水位对照；不得以历史根数量有界代替磁盘水位证据。

本次r7仅新增**T-11统一能力验证阶段 / AC-11 / V-15**，消费852已交付底座；原T-01/T-10完成项、私有模块证据及其他原验收/预算保留，任务总数10→11，current_step不增加。范围与无环交接见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。当前已实施，旧稿“仅起草/尚未创建”措辞属于历史，实际进度以§8和§9为准。

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付typed owned环境、逃逸捕获与frame/env身份。已完成模块切片不等于正式profile交付；本轮修订不启动或代替work。

## 1. 目标

在822的类型化资源和集合上加入typed closure环境、受检间接调用、move/显式clone逃逸捕获、静态可证非逃逸的同步借用捕获；验证创建者返回后合法owned闭包仍可调用，环境/载荷清理责任准确。

成功标志：真实受检MIR→ABC→AV在新profile下执行，正例独立oracle一致、坏输入在消费前拒绝、资源/版本/恢复可验证，旧core-i32及实际已合入的817/818标量路径兼容。

非目标：共享可变捕获/RC闭包、async/生成器/任务调度、跨线程Send/Sync、用户函数类型推导全套、源码闭包语法扩面、持久闭包/native ABI/JIT/OSR。

范围只在auto-lang仓的共享实验核心、独立fixtures/recipe/本计划脚本/证据；根workspace成员/产品manifest、旧VM、UI/Man/SDK及他方在途实现不改；仅允许本recipe在当前真实owner处追加最小注册/父侧判定薄接线，并按实际根Rust触面验证。共享注册位置服从814–816实际迁移后的owner，不复制协议/执行器。

## 2. 架构方案

```text
支持的已有Checked HIR / 明确未信任typed fixture
  → av.closures/1 MIR + verify
  → ABC encode → decode/verify → LoadedModule
  → AV typed access / layout / lifecycle → typed result/diagnostic
             ↕ 783 managed store（代码与portable载荷分域）
   新worker恢复重验 + AutoDown父侧独立oracle
```

执行profile拟为 `av.closures/1`，名字/编号由T-01确认不冲突后冻结；不复用core-i32授信。纯语义TypeRef与AV LayoutRef、native target ABI分离；不再创建VM私有推导器或第二MIR。

## 3. 技术栈

Rust、D依赖交付的 `experimental/ac-av-probe/` 独立workspace（已观察package为auto-av-core/auto-av-probe，work须重核），783 ManagedStore、796语义事实、现有probe/run/contained worker。

影响入口：共享MIR/verifier/ABC/AV/place/ownership/layout（D-822）；拟新增 `core/src/closures.rs`、环境描述/构造与call指令、typed fixtures/AutoDown recipe；只读旧AutoVM闭包反例。

拟新增 `scripts/verify-av-closures-823.ps1`，本计划测试前缀 `av_closures_823`、`test/cases/av-data/823/`、独立recipe `av-data-823`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

2026-10-26用户明确授权修订852及823～826，先交付测试底座，再由在途计划加入本范围验证phase。本次r6→r7只修订合同，不启动实现/会话、不改822或实施树，不追加语言/native/AJ能力。新增验收属于用户明确授权，原已完成任务和证据按其实际范围保留；新AC-11尚无执行证据。

2026-10-26用户授权审计/修订823–826并行边界，本轮仅改计划与战略，不启动实现或其他会话。取材初始821 archived r6、822 executing 6/9；期间822 R1 needs_fix重开至4/9，两次均非D822（他方进度原样保留）；实际输入/代码接缝与指纹见[依赖审计](reports/ace-parallel-dependency-audit-2026-10-26.md)及[inputs](attachments/ace-parallel-20261026/inputs.json)。旧“整计划串行/仅只读准备”排期按下文两阶段合同更新，原目标/非目标/预算和已有进度保留。

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-822**=上游独立最终review、冻结门禁、合入、Spec/merge收据齐全，不消费WIP。保留D822：完整首例包含List<record>捕获，需真实822类型/集合/资源合同；不等824整计划。 T-01只读准备可先；T-10在D821/D828及私有写面登记后可做真实独立模块开发，不占公共核心窗口。实际全路径构建T-02起遵守本阶段D门/窗口与原授权。具体规则见§5并行合同。

拟唯一实施树 `D:/autostack/.wt/lang-823/auto-lang`，分支 `plan-823-dev`；本次修订不创建/切换实施树，现有工作依§8完成项及§9收据。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

## 5. 详细设计

### 5.1 当前执行合同

1. ClosureRef绑定具体函数/签名/profile/代码代际与typed env；CreateEnv完整初始化后才发布，CallClosure核验签名和store。旧Fn签名槽不能自动接任意闭包。
2. 初版闭包是owned值：move转移、显式clone按环境字段深clone，按逻辑view/mut签名调用；无自动共享RC/Copy。调用期间保持env有效，不在执行时释放其载荷。
3. 逃逸捕获仅move或显式clone，环境自身拥有资源；创建者返回不影响合法env。借用捕获只允许静态可证非逃逸同步调用，不可返回/入字段/入List/跨未知effect或挂起。
4. 嵌套捕获经过真实父env位置传递责任，不能重用创建者bp/slot解释；frame实例UID、env generation和store均有边界。同bp复用不授权旧借用。
5. 捕获源与构造顺序按输入语义各执行一次，失败创建回滚部分env且未提交源保留；返回和调用失败沿821全出口清理。
6. borrow分析采用保守显式允许集，未知callee/effect拒绝；共享/weak循环与async全部unsupported，不用模拟park/resume声称已支持真实调度器。

### 5.2 有界首例与资源政策

T-01冻结支持/拒绝、类型/指令/wire版本、生命周期、输入前门及计量决策，不在下游实现中临时猜规则。T-02必须完成：typed fixture经真实verify/ABC/AV创建捕获owned Str及List<record>的闭包，返回创建者后在新调用帧执行、改变可变环境字段，再drop闭包。与固定结果/环境事件对照，首例cache保存代码而非活环境并重启重建执行。

首例不可行则记录实际输入、拒绝/资源原因和合同冲突，回new显式revision；不能改用旧VM、弱化verifier、去掉oracle或只留纯模型。未知语法/接口的fixture是未信任typed输入，真verify授信后执行，不把未落地接口描述当证据。

沿D依赖已冻结输入≤4MiB、module/每函数表≤65536、验证≤200万步/5s、执行≤100万步/1024帧/2s的**适用上限交集**；旧profile更小cap仍生效，单位/计步在T-01核对，不按本段放宽旧合同。新增类型描述、路径、图、construct/drop/freeze必须申请预算并使用有界显式遍历。值数据合计≤8MiB（槽+聚合+buffer+环境+Dyn），受管总32MiB（hot8/staging16/execution8），不是各类各领8MiB；allocator/库暂存按真实峰值报告。新worker private-bytes相对空载增量沿798上限128MiB，包含本次allocator/解码/临时对象；父侧采样瞬态峰值，与自有计数共同判定。native工具进程另列归因，不从worker实际峰值扣掉前端。

这些是拟议实验资源政策，非用户给定或永久语言限制；T-01如不能成立须带边界输入回规划，不能在work直接提高。OOM/预算/cancel/infra与invalid/unsupported/trap分态，分配前拒绝；cleanup预留记账不受正常fuel耗尽取消，不暗加语法深度限制。

### 5.3 存储、注册与兼容

保留D-798冻结的标量编辑负载和请求到父侧发布终点，按用户 r9 裁定定向重放 10 次，至少 9 次≤2000ms（P90≤2s）；所有样本正确性仍须通过。冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。用户授权直接执行，无须等待其他构建结束，但需如实记录并发负载和测量条件；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

首例沿783真实代码producer、root/lease/GC，cache恢复重走MIR/ABC门；类型/字段集/layout/profile/语义版本入依赖，坏内层即使好hash也不得执行。活帧/borrow/宿主地址不存cache。首例独立recipe注册和父侧expected比较是T-02交付，不允许“有fixture但execution pending”结项。

共享注册入口现观察为 `crates/auto-lang/src/test_cases/{sema,mod,stale}.rs`（若已迁移以816交付为准）。只追加自己的recipe/身份和独立worker处理，不复制整个sema worker，不修改v1/v2解码或他方recipe合同。若协议不足，在T-01由实际owner决定最小版本化增量，依赖实施暂停。

旧标量语料与实际已合入817/818路径必须重放：仅保证原profile兼容及未知新profile明确拒绝；不强制其他后端实现新数据。不宣称旧native/818已支持新闭包。以固定结果、调用trace、真实资源事件及独立生命周期模型验收；旧VM仅提供帧复用/跨任务反例素材。

### 5.4 ABC共享合同与本计划责任（r2）

依据[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44 §15](../design/44-av-data-layout-place-and-ownership.md#15-abc补充合同与逐profile验收r2)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)，本计划承担**环境所有权与调用合同**：ClosureEnv字段构造/capture/invoke/drop统一到InstructionSpec；参数模式、返回owner/效果与env/frame代际共同验证。环境是owned aggregate，不因可调用就Copy；move捕获杀死旧根，显式clone独立拥有。函数/环境必要类型、owner和清理信息为语义，不能随debug剥离；帧复用不得复活旧token。

语义MIR、portable ABC和私有LoadedProgram分层，不另建语义IR。825不是本计划硬依赖：其已合入时适配受检执行表示，未交付时允许同一私有接口的安全参考路径；未知profile不进无验证fallback。kind/entry/imports及歧义旧缓存策略消费D-798修复，不猜函数数。owner、活分支和Drop/必要类型是语义，strip debug不得移除。

类型/路径分析、clone/drop/编解码及cleanup计入既有合计预算；输入要求的capabilities与当前host grants分开，优化不绕过授权、逻辑fuel和取消。帧arena、slot复用、融合及紧凑wire属825/826或另案性能，不以其完成替代本计划正确性。T-01冻结V-11具体fixture/反控和各层合同映射，T-03/04实现，T-05覆盖恢复，T-06汇总；不得只放设计注释而漏验收。

### 5.5 文档复审补充合同（r3）

capture所有fallible分配/clone/来源验证/记账预留先prepare，保护待move根后才进入无失败、无回调、无取消观察的commit；提交前失败仅清理临时，未提交源仍拥有。提交后失败清理新env而非复活源，普通求值效果不回滚。嵌套env字段转移仅专用受检capture操作，活字段位图及invoke读集参与验证/清理；普通资源字段MoveOut仍拒绝。

准入错误在decode/verify/load门拒绝、dispatch=0；已准入程序的越界、失效句柄/guard、错型take、取消等允许此前合法执行，须断言危险目标访问/提交/目标callee计数0及原值/清理，不要求整次dispatch=0。prepare/commit只保护尚未提交的转移；已发生的guest求值/调用效果不回滚。

消费前冻结已合入(profile, wire/schema, semantic-version, producer)矩阵；825/826可前置但不是本计划硬依赖，820指令表尚未交付时它们沿D798现有合同，不反向等待本计划。当前core/注册窗口只能在依赖满足后持有，不持锁等其他owner。注册的最小接线是范围内例外，不等于修改旧VM或产品默认；814–816只按实际已迁移owner交接。

资源表沿798精确族口径：模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536；index上限还须小于对应表长及总额，不能为取最大索引挤掉必要块/指令。32MiB分区和128MiB进程增量分别验收，不扣前端/allocator/暂存，已有约211MiB前端证据不能记债后当D798已达。T-01核当前D门及同口径证据，冲突先回规划，不擅自豁免或提高数字。

T-01在contract-decision冻结case→子case→Scope→真实入口→阶段/码→oracle→必需/适用矩阵，包含V-12；T-02使first-case和recipe正控可执行，T-03/04实现与反控，T-05核恢复/兼容，T-06验必需集合无漏例/重复冒计/skip。合法特定profile外输入应按合同拒绝，不能误算成适用例未执行而跳过。

### 全链路增量兼容与828优先排期（r4）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：稳定callee/环境布局身份，旧闭包/帧保留旧代码与清理责任；body编辑不全局重编号或重建其他闭包/代码。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，10次不同修订，结构与稳定会话 P90≤2s 双门（至少9次≤2000ms）；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控；样本数、分位数和门限以用户 r9 裁定为准，其余门不降。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 并行模块与集成两阶段合同（2026-10-26）

保留D822：完整首例包含List<record>捕获，需真实822类型/集合/资源合同；不等824整计划。

**T-01→T-10独立模块门**：依D821/D828及已合入公开接口开展821已交付Str/record的owned捕获准备/提交、环境载荷与清理；失败分配/clone不消费待move源，不实现第二套间接调用器。单一生产源为`core/src/closures.rs`（拟新增），测试壳拟为`experimental/ac-av-probe/parallel/823/`独立workspace/lock，仅用同树已合入auto-av-core。T-01冻结可达符号、具体路径/owner、支持子集和预算；T-10先证明最小可编译接缝，再实现真实模块与故障oracle。可由harness重导出已交付core公开模块并路径编译同一源，生产阶段只加薄adapter；不复制值/IR/store/VM，不改core/lib.rs/父members，不产生依赖环。若需要pub(crate)私有符号，不绕过权限，登记具体缺口并申请独立验收合入的小接缝；不能从上游WIP取代码。

**T-02集成门**：T-10通过不等于profile/Plan交付。满足原D门后同步最新master，刷新接口/版本/预算，取得共享核心/注册窗口，完成原首例真实verify→ABC→AV/store/new-worker/AutoDown和trace/资源；继续T-03..06/T-09及最终review。所有模块独立通过不能代替原全路径、精确增量、r9延迟门/内存或反控。T-10测试命令拟为`cargo test --manifest-path experimental/ac-av-probe/parallel/823/Cargo.toml`（入口由T-10建立，当前未运行，selected>0），正式脚本必须汇总新V-14，不是独立壳绿就全计划绿。

**写面与交接**：本计划只写上述私有新增源、parallel/823及自身fixture/报告；现有mir/contract/ownership/place/layout/instr_spec、codec/verifier、av派发/装载/snapshot、公共lib/manifest和共用driver/recipe注册默认一个核心窗口。基面/版本/文件符号、前后owner、兼容证据及接受/释放需记录；公共tag/opcode/version由核心owner统一登记。复审期间其他Plan可以继续无公共写面的T-10，不能拿未审公共定义开工或持窗等上游。最后集成者重核已交付消费者并集，复审独立、整档仍merge-owned，测量单实例。详见[共同审计/窗口表](reports/ace-parallel-dependency-audit-2026-10-26.md)。

新borrow capture的frame/env/store保护必须本计划定义和实现；821 r6没有运行期失效句柄/旧帧/跨store校验，不得当既有保证。若824先交付，T-04/T-05在加入真实Closure类别时负责嵌套pack/freeze拒绝及824合法值正控；若824后交付，记录类型/拒绝接缝交其验收。

### 852底座交付后的统一能力验证phase（r7）

本计划新增T-11，原实现可以继续；仅T-11及最终汇总/送审消费D852。D852是852当前revision的独立review、
冻结门禁、实际合入、Spec和merge收据齐全的**底座交付**，不要求M1全绿；852不反向等待本计划。
不消费852草案/工作分支WIP。原D门、单写窗口与原非目标不变，详见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

固定映射：**source:K01～K06；scenario:R05/I01/02/04～07/10～12**，对照本计划原V表。T-11工作入口先记录每个ID的实际profile/version、
source/checked HIR/未信任typed MIR起点、正式recipe、具体V子例、独立oracle及raw证据。
本范围必须实测：闭包捕获/创建者退出/clone独立/mutable环境、非逃逸借用及逃逸拒绝，真实MIR→ABC→AV、store新worker恢复、清理与原T-09增量。
边界：源码闭包前端/推导、native closure ABI/codegen不加入本计划；typed起点只关闭低层验证行，源码/native的M1行仍gap。不得删必需适用行、以skip/模型/旧VM回退补绿。

typed验证与Auto源码验证分轴；对应源码尚不支持时，保留M1源码/native缺口及明确owner/关闭条件，
不得把同号typed fixture当作同源四路径通过。真正范围外且有明确owner的M1缺口不阻塞本计划，
但本计划原已承诺的能力/实际适用T-11项缺证据仍阻止AC-11。

T-09在T-02～05/T-10后做既有增量验证（不等待T-06/T-11）；T-11在T-05/T-09和D852后完成；
T-06最后汇总原门及新V-15，T-07独立核AC-11。避免T-09“其他work完成”与T-11/T-06互等。
原T-01/T-10已验子集证据保持；新增AC/V不能用旧模块绿关闭，受影响内容按实际输入/代码/依赖变化重验。

852拥有通用case/证书/报告合同；本计划只追加自身binding/证据，协议/registry/SDK adapter按共享测试窗口
单写交接，关联MIR/AV变更另守核心窗口，不复制runner/VM/store。a2r有效认证按内容/依赖复用，
日常只emit+金样比较，不由T-11重复编译认证。延迟按用户 r9 口径，内存/磁盘及负控不降，work仍只定向测试。


### T-09 env 契约阶段（r8，2026-10-27 用户裁定在计划内加 phase）

D828 增量机制是单元级，CREATE_CLOSURE 原 callee:FuncIdx 与单元纪律（callee 走 import）冲突，且 body 单元独立验证看不到创建者 site 的 env 布局。冻结有界修订：**闭包 body 自带 env 声明**——`MirFunction.env: Option<Vec<TypeIdx>>`（Some=闭包体；声明即 env 布局，borrow 捕获条目=StrView）；`ENV_GET` 对本函数声明定型（单元独立可验）；`CREATE_CLOSURE.callee: CalleeRef`（模块=Func、单元=Import，import 表携 env 声明），captures 逐位等于被调 env 声明；link 期 `fn:` 绑定加 env 等价检查（V-13 子 case 3 精确失效轴）。wire v4 未发布、原位扩展。site 唯一规则保留（模块内）；既有 fixture 全部补 env 声明。本阶段不改变 borrow 纪律/资源门/旧 profile 面。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/closure-environments.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/closure-capture-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-823独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |
| SD-07 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 852底座分母→追加本计划实际支持起点/recipe/证据与范围外gap；不冒领source/native能力 | 统一验证且不扩原语义范围 | AC-11 / V-15 |
| SD-08 | modify | docs/specs/auto-ac/artifact-contract.md | 保留墓碑/未完成恢复记录及活数据；独占 GC 中清理已彻底删除且无引用的块元数据与已提交事务终态，保持长期编辑的数据库空间有界，故障恢复与重新发布不得退化 | 真实512次编辑发现元数据线性增长 | AC-05/06/09/11 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。


## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、内存及反控保持，延迟使用 r9 口径。

不变量为Design44 I-01..12及§5。以下V均为**必需补充反例/正控**，T-02/T-04将其变为真实入口；固定语料总数不能替代。每案保存输入身份、阶段/码、result/trace/bits/owner与原始日志。期望由人工或独立小模型确定，不能由被测布局/清理算法生成。

| Case ID | AC / work owner | 输入/状态 | 实际或拟入口 | 可观察oracle与区分断言 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02；T-02 | 创建者返回后调用move捕获Str/List的闭包 | 新profile verify/ABC/AV，av_closures_823 family | 返回固定结果且资源仍活；drop闭包后live=0 | 逃逸正例；仅声明支持面 |
| V-02 | AC-02/03；T-03/04 | 合法同步借用，返回/入容器/跨未知call的同型借用闭包 | 新profile verify/ABC/AV，av_closures_823 family | 合法成功；逃逸前门拒绝，不靠运行期悬垂报错 | 借用边界；仅声明支持面 |
| V-03 | AC-03/05；T-03/04 | 祖父→父→子env、同bp帧复用、同名遮蔽 | 新profile verify/ABC/AV，av_closures_823 family | 捕获当前语义来源；旧frame token读取前拒绝 | 嵌套/复用；仅声明支持面 |
| V-04 | AC-04；T-03/04 | 同宽错签名/错generation/跨store闭包与合法对照 | 新profile verify/ABC/AV，av_closures_823 family | CallClosure前拒绝，无目标函数副作用 | 间接调用；仅声明支持面 |
| V-05 | AC-02/04；T-03/04 | 第n个capture clone分配失败、move/copy源混合 | 新profile verify/ABC/AV，av_closures_823 family | 部分env逆序清理，未提交源仍可用 | 构造失败；仅声明支持面 |
| V-06 | AC-02/03；T-03/04 | mut env调用、trap/cancel、闭包clone后分别drop | 新profile verify/ABC/AV，av_closures_823 family | 无重入别名冲突，调用中env不误释放，副本资源独立 | 调用清理；仅声明支持面 |
| V-07 | AC-05；T-04 | 合法输入cap内/超1单位、瞬态峰值、故障第n次 | 新worker+父侧budget/allocator hook | resource分态；预算前拒的该申请attempt=0，故障注入attempt存在但successful_alloc=0；先前合法事件保留，危险访问/提交0；cleanup/真实峰值断言触发，无panic | 不能只看终态RSS |
| V-08 | AC-06；T-02/05 | fresh、同store新worker重启、改type/layout/profile、好hash坏payload | 783真实恢复→verify→AV | 合法恢复结果相同；依赖变化不复用；非法执行计数0 | 不缓存PASS或trusted |
| V-09 | AC-06；T-05 | core-i32及实际交付817/818旧profile、新profile送旧consumer | 实际原recipe/consumer | 旧结果/诊断不变，未知profile明确拒绝 | 无假native/CTE支持 |
| V-10 | AC-01/04/06；T-06 | worker错结果/错bits、漏case/skip、错工具身份 | 正式AutoDown recipe+父侧oracle | 合法正控PASS；每项错实际使总体非零/ERROR | 缺必要工具/路径不可绿 |
| V-11 | AC-02/03/04/05/06；T-01/03/04/05/06 | capture后旧根读、错invoke模式、错env/frame代际、创建帧退出后的合法owned调用、strip后trap/cancel的env释放 | §3真实MIR/ABC授信、执行及store新worker恢复；T-01冻结输入，T-03/04使可运行 | 非法旧根/模式/代际被拒绝；逃逸owned调用仍正确；每一仍归env的字段仅清理一次，strip前后结果及清理一致；好hash不得绕过；准入非法dispatch=0，运行期故障按§5.5断言目标危险操作0 | 仅本profile；标量/旧消费者仍V-09，受管/进程峰值沿V-07 |
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | 先move候选后clone失败及逆序；提交前/后取消；父env字段转给子env再退出父/调用子；已转出字段的父invoke与同根重复capture | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | clone失败不提前消费待move源；提交前源保留、提交后仅新owner清理；父子字段资源仅释放一次，子env仍正确；读取已转出字段/重复move在适用门拒绝；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_closures_823`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-closures-823.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
- fmt/check按该独立manifest执行；开发与修复只定向测试。禁止work/internal/final review触发整档t/tv/tt/tb/tu/taa。
- 合入：最终review后冻结HEAD，由merge执行裸`cargo t`+VM/编译器触面`cargo tv`，以及新workspace完整有限合同档；按实际diff再选其他触面，tf只主检出到期单实例。review pass必须记录merge门禁pending。

角色：用户未指定模型或分离实现者。work负责实现和反例；内部检查只表示可送审；最终review必须独立上下文、同树只读核验，不修代码；merge负责冻结门禁/规范/归档/guard清树。当前未指派或启动任何角色/代理。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-13 | AC-09 / T-09 | D828 source 10 修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，10样本至少9次≤2000ms；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

### 并行同源与依赖边界验收（新增work-owned）

| Case ID | AC / owner | 输入与真实入口 | 区分性oracle | 覆盖边界 |
|---|---|---|---|---|
| V-14 | AC-10；T-10/T-03/T-05 | 同一拟新增生产源由parallel/823构建并在核心窗口后由真实生产adapter消费；坏输入/失败分配/版本或窗口负控 | 真实StrArena/Value上的构造取消、故障clone前源仍可读、成功转移后env仅释放一次；生产接线使用同一closures源和真实AV，若824已交付新增Closure嵌套拒绝同时成立；同源hash/函数映射一致，越写公共面、WIP依赖或适用必需组合缺例不得就绪 | T-10只证明已交付子集模块；T-02..06补正式profile/cache/AutoDown，未知/未交付类别诚实拒绝，不冒领全路径 |

### 852统一能力验证（新增work-owned）

| Case ID | AC / owner | 输入/实际入口 | 可观察oracle与反控 | 边界 |
|---|---|---|---|---|
| V-15 | AC-11；T-11 | source:K01～K06；scenario:R05/I01/02/04～07/10～12；D852已合入runner及本计划真实recipe/V子例 | 闭包捕获/创建者退出/clone独立/mutable环境、非逃逸借用及逃逸拒绝，真实MIR→ABC→AV、store新worker恢复、清理与原T-09增量；错结果/漏适用行/假source起点/旧证据错版必非绿；保留原资源/增量及独立expected | 源码闭包前端/推导、native closure ABI/codegen不加入本计划；typed起点只关闭低层验证行，源码/native的M1行仍gap；实际起点/范围外缺口有owner，无source支持不冒领M1 |

## 7. 验收标准

| ID | owner_stage | 可观察完成标准与方法 |
|---|---|---|
| AC-01 | work | T-02真实首例经source支持面/typed fixture→verify→ABC→AV，内容/位型/来源和独立oracle一致，V-01/10通过 |
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（本profile含owned值）；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 ；核AC-10及V-14的真实模块/集成边界；同时独立核AC-11/V-15 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |
| AC-10 | work | V-14的T-10独立同源模块与T-03/T-05真实集成证据齐全，D门/公共写面接受释放/版本与消费者矩阵可核；不以模块绿替代原全路径，适用拒绝/失败控制与原增量/资源门不减 |
| AC-11 | work | T-11及V-15映射/真实执行/负控/证据齐全，消费D852已交付身份；本计划适用行不得漏测/skip，范围外source/native/M1 gap保留owner且不冒领；日常a2r复用有效认证、零Rust重编译；原AC/V/预算不减 |


## 8. 执行步骤

已完成任务以本表[x]和原始收据为准；新T-11未完成，current_step不增加，review/merge不能提前勾选。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施授权；只读准备可先 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/823/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜；冻结§5私有同源模块/harness可编译接缝、T-10子集与集成门、实际公开API和窗口登记 | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总。[✅ 已完成 2026-10-26] [contract-decision](attachments/823/contract-decision.md)：D821/D828 archived 满足、D822 executing 未满足（T-02 阻塞如实登记）；av.closures/1 命名冻结、wire/opcode/recipe 编号仅请求不占号；T-10 API/预算（MAX_ENV_FIELDS=1024、8MiB 默认）/九用例 oracle/cap 两侧输入/同源接缝（parallel/823 + #[path] + re-export）/av.rs clone_owned_value 私有缝收敛裁定（T-03 av.rs CLONE 改接 deep_clone_value 单源）全部冻结；核心公开 API @ ff7547f9d 逐文件 SHA-256 指纹入档 |
| [x] T-10 | work | T-01、D821、D828、独立私有写面登记 | 新`core/src/closures.rs`与parallel/823独立harness/lock；最小接缝先编译，再完成821已交付Str/record的owned捕获准备/提交、环境载荷与清理；失败分配/clone不消费待move源，不实现第二套间接调用器；不改公共装配 | V-14模块子集；AC-10；未建入口/0例不能绿，完整case待T-03/T-05生产同源证明。[✅ 已完成 2026-10-26 @ plan-823-dev d6838a93e] 最小#[path]接缝一次编译通过→完整实现；`cargo test --manifest-path experimental/ac-av-probe/parallel/823/Cargo.toml av_closures_823` = **10 passed/0 failed**（selected=10>0：混合prepare/commit、env逆序drop恰一次、故障clone源可读+staged逆序回滚、move源故障后保源且可重试、StrView/MutRef拒绝attempt=0、commit漂移三态返还源、fields/bytes cap两侧、av.closures/1现verifier拒绝、子集身份≠注册profile、深克隆独立缓冲+故障attempt传播）；rustfmt过、零警告；core/lib.rs与父workspace members零改动，worktree内仅两条新路径 |
| [x] T-02 | work | T-01/T-10、D822、D828、核心/注册窗口接受 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记；只消费T-10同源模块，不另实现；刷新已交付基面/版本、记录接受 | AC-01/06；first-case、V-01/08/10正控。[✅ 已完成 2026-10-27 @ plan-823-dev e9bb30606（基面 c6c5ddb25=822 archived 后 rebase）] 合同 r2 定稿（[contract-decision §8](attachments/823/contract-decision.md)：wire v4/id5/tag16/opcode 0x2F-31 登记为实际编号、深克隆收敛=委托 av::clone_owned_value、唯一 site 规则、捕获矩阵含 List/Variant/Place 实测）；§5.2 全 oracle 过：首例四测试 **4/4**（creator 返回后新帧调用+可变 env 字段+drop→手算 29/ledger 5alloc 5free/live=0；byte-stable wire 往返；code-cache 恢复同果+types-fp/entry/snapshot 身份轴 clean miss；四负控真实拒绝）；`scripts/verify-av-closures-823.ps1 -Scope first-case` PASS（含 harness 11/11）；只消费 T-10 同源 closures.rs（core/lib.rs 注册+parallel/823 双编译同文件）；受影响面回归全绿：820 semantics 13/13（pin 46→49）、821 first 5/5+semantics 25/25+recipe 3/3+resources 7/7、822 first 4/4+semantics 59/59+incremental 12/12+cache_compat 8/8、abc 15/15+cache 20/20+mir 8/8（~180 例零回归）。V-10 正控=首例 recipe 面（A23 layer+fixture+恢复）；AutoDown 全反控仍 T-06 |
| [x] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile；生产直接消费T-10同源，完成适用V-14集成轴 | AC-02/03；av_closures_823定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总。[✅ 已完成 2026-10-27 @ plan-823-dev fa06bfe8b] CaptureKind 三态（Move/Clone/Borrow，wire tag 0/1/2）；borrow 域=owned Str、env 字段=StrView share（快照语义+静态门）；纪律扫描（borrow 闭包仅 CallClosure/Drop 消费+必须调用并 drop、源槽全函数钉住仅捕获点/VIEW 可用、terminator 覆盖 Return 逃逸）；CLONE 闭包=env 深克隆+新句柄；语义十测 10/10（V-02 合法借用 8 手算+逃逸/钉住/域三负控、V-03 嵌套子闭包+同闭包两帧复用 60 手算、V-04 arity/result/operand 三负控、V-06/V-11 trap 期 env 释放 live=0、V-06 CLONE 独立 oracle 80≠共享 90、V-11 move 捕获后读拒绝、V-12 提交后取消释放）；harness 12/12；ps1 -Scope semantics PASS |
| [x] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总。[✅ 已完成 2026-10-27 @ plan-823-dev 81a29f1d7] CreateClosure 值分配故障缝（fault_after_allocs：#2=env Infra 含 attempt、#1=const 物化区分性对照、零源消费——kill 循环不运行）；克隆走查 fuel/wall 双门（InstrSteps/ExecutionWall Resource+部分 staging 释放 live=0）；模块级 attempt=0 律 harness 已钉；resources 3/3 |
| [x] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总。[✅ 已完成 2026-10-27 @ plan-823-dev 81a29f1d7] entry/snapshot 轴 clean miss；全新 store 句柄重验重放同果（code-only 缓存）；V-09 隔离：closures wire 对 owned/collections scope 深验证拒绝（schema-profile，dispatch 前控制=同 wire 闭包 scope 准入）；旧 consumer 重放=compatibility scope+回归族（820/821/822 全绿 149 例）；cache_compat 4/4 |
| [x] T-06 | work | T-05/T-09/T-11 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据；正式脚本汇总V-14/AC-10，无模块通过冒领集成或缺适用必需例 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总；最终汇总AC-11/V-15，不以852基础层PASS代替本能力验证。[✅ 已完成（AC-01..06 面）2026-10-27 @ plan-823-dev 81a29f1d7] ps1 全 Scope 聚合透传失败（first-case/semantics/admission/resources/cache/compatibility/autodown/acceptance）；V-10 worker 侧 oracle=手算 truths 表+judge 六类谎言反控（错bits/泄漏/漏例/错形状/清理短缺/缺节，3/3）；受管峰值观测（hot/staging/exec high-water ≤8MiB 线）；acceptance **36 例全绿**；AC-11/V-15/T-11 与 -Scope baseline 仍待 D852 后 T-11；fmt/零警告；SD 草案证据在案 |
| [x] T-09 | work | D828、T-02～05/T-10（不等T-06/T-11） | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档。[✅ 已完成 2026-10-27 @ plan-823-dev 5535b6e4d（r8 env 契约 9468704ac 之上）] ClosuresUpdateSession（真门生产计数/结构退化门/原子提交，镜像822）；program_snapshot link 对 `fn:` 绑定 env 等价检查；单元入口体豁免；cache 抽取/合并 env 透传。V-13 四测全绿：子case2 body-only修订（未改单元 Arc::ptr_eq 保留+生产/IO全零+语义 5/8/13 手算）、子case3 env 漂移精确 link 拒绝+会话恢复、子case4 强制退化非零门、r8 单元独立验证；子case1（D828 基线 273575B/115函数/30修订）为 828 自有交付、822 已按同型会话重放，823 消费同一机制（非重复语料）。回归 ~170 例零回归（含 828 增量族）；acceptance PASS |
| [x] T-11 | work | T-05/T-09、D852、原D门及所需测试/核心窗口 | 执行§5统一能力验证phase：source:K01～K06；scenario:R05/I01/02/04～07/10～12逐行映射到真实起点/recipe/V子例；闭包捕获/创建者退出/clone独立/mutable环境、非逃逸借用及逃逸拒绝，真实MIR→ABC→AV、store新worker恢复、清理与原T-09增量；在attachments/823/852-validation记录binding与raw，范围外M1 gap保留owner；不实现额外前端/native/AJ | AC-11/V-15；本计划既有正式脚本新增显式`-Scope baseline`（T-11建立），acceptance最终包含其结果；定向执行/反控，selected>0且无漏适用项，不触发整档或隐式Rust认证 。[✅ 已完成 2026-10-27 @ plan-823-dev ed6997394] D852=archived 已消费；binding.tsv 仅翻自有 7 格（wait:av-closures-823→ok，K01..K06+scenario R05 按 id+ns 双键防 corpus record R05 误伤）+I01/02/04/05/06/10/11/12 指针追加，schema/金样/他方行零改；[binding.md](attachments/823/852-validation/binding.md) 逐行映射（语义ID→真实recipe入口→V子case→手算oracle）+raw 收据；K01..K06/R05 全部真实执行（typed 证据关 av 层，源码列 M1 缺口原样保留 owner=a2r复合/823-future-source）；I07 范围外缺口保留；I10/I11 闭包轴实测：**512 真实 body 修订**全门绿（Arc::ptr_eq 身份、生产零 IO、退化门 0），warm 会话 P95<1000ms（全程 0.05s 量级）；I12=judge 六谎言+退化门非零；`-Scope baseline` 5 入口全绿并入 acceptance（全 PASS）；零 a2r 认证/零 Rust 重编译 |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成、T-10及AC-10完成、T-11/AC-11完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论；独立核依赖/同源与窗口并集及V-14 | AC-07；merge-owned全档仍pending；独立核AC-11/V-15，新证据未验不得沿用旧结论 |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |


## 9. 复审记录
2026-10-27 stage:new | PLAN-823 r8→r9 | outcome:pass（合同修订，不代表实现验收） | 用户确认本计划增量编译及798兼容门均改为10次采样、至少9次≤2000ms（P90≤2s），并授权直接测试；功能/结构/资源和512次持久编辑+GC不减少；GC新增SD-08处理长期元数据增长 | next:work完成定向验证、存储恢复与提交，之后独立review；不merge。

2026-10-27 stage:work | PLAN-823 r8 | outcome:partial（继续 executing；未送审） | code:ea88414456bf89c98307c8fa2fe99a06dcaa1a5d | 798 冻结延迟门：自然连续空闲五分钟后以正式 release worker 和原口径 30 样本复测，编辑 P50/P95/P99=635.1/666.4/671.2ms，签名冲突 P95=256.5ms，门限 1000ms，1 passed；分次进程检查未发现其他构建/测试重叠 | evidence:[诊断及环境](attachments/823/repair-r2-diagnosis.md)、[完整日志](attachments/823/repair-r2/798-final-ea8841445.txt) | current_step:3/11 保持；其他 work-owned 验收与规范增量待核对、独立复审待执行，merge-owned 裸 t+tv 及必要触面档 pending；不 merge、不发布 canonical Spec/ledger。

2026-10-27 stage:work | PLAN-823 r8 | outcome:partial（继续 executing；未送审） | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | base:0d5d58f6b93bbf3f7e687b103f4b19b7ae8fe6ef | code:ea88414456bf89c98307c8fa2fe99a06dcaa1a5d | deps:auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4、auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde | tasks:T-02..06/T-11受影响修复 | evidence:[两次未通过后的诊断、关闭表、拟议Spec及逐项测试](attachments/823/repair-r2-diagnosis.md)；定向语义、资源、真实新进程/正式裁判、旧consumer及失败反控通过；提交前组合预算/逆序释放另有区分性反例 | remaining:798冻结延迟门未通过：中间版本30样本编辑P95=1116.7ms（限1000ms），最终提交仍需单实例重测；当前其他会话持续进行 Cargo 构建和测试，不并行测量；其余适用验收及规范增量须按合同核对 | current_step:3/11 保持，旧已完成叙述仍为历史 | merge-owned裸t+tv+必要触面档 pending、未运行 | next:work待机器空闲在最终代码上重测冻结门，归因并解决任何失败，再核对剩余work-owned条件并交独立review；不merge、不发布canonical Spec/ledger。
2026-10-27 stage:review | review_scope:final（R1受影响面复审） | reviewer_identity:本聊天Codex，继续独立审查上下文，未实施修复/未创建代理 | PLAN-823 r8 | outcome:needs_fix | main:95de4f14279ceb5d5ab8b7ac6875fac49851ede2 | worktree:D:/autostack/.wt/lang-823/auto-lang | branch:plan-823-dev | reviewed_commit:0d5d58f6b93bbf3f7e687b103f4b19b7ae8fe6ef | repair_base:ed69973940839c0c52042dad5fb4e40971215a6d | base_commit:c6c5ddb256089e6b628d7651cb9759664d15f22a | evidence:[R2报告/关闭表](attachments/823/review-r2/review.md)、[反例实测](attachments/823/review-r2/probe-output.txt)、[正式worker重放](attachments/823/review-r2/worker-replay.ps1)、[SD冻结](attachments/823/review-r2/spec-delta-frozen.md)、[输入指纹](attachments/823/review-r2/input-hashes.json) | acceptance_results:定向25通过、fmt通过；F05源码campaign/typed三控制关闭（AC09/T09完成）、F06格式关闭；真进程恢复子项保留 | findings:F01 env别名经Move洗掉仍Success29、F02 recipe白名单/身份/父judge漏接、F03资源/兼容适用门仍partial、F04 park30仍只有3alloc且提交前取消，均OPEN；新增F-823-R2-01 P1未信任EnvGet dst越界panic、F-823-R2-02 P2 unused_mut | 当前进度T01/T10/T09=3/11，其余work任务未闭合；不沿用下方旧“全完成”声明 | pending_closeout_gates:merge-owned裸t+tv+新workspace完整合同档，未运行；SD最终审查/完整依赖窗口并集pending | next:同树work修复、提交后受影响复审；第二次非pass，送审需先补不变量/漏检原因/设计与oracle调整的关闭表；不merge、不写canonical Spec/ledger。
2026-10-27 stage:review | review_scope:final（就绪核验发现未完成项，停止完整最终验收） | reviewer_identity:本聊天Codex，独立于此前实施上下文，无代理/其他模型身份声明 | PLAN-823 r8 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-823/auto-lang | branch:plan-823-dev | reviewed_commit:ed69973940839c0c52042dad5fb4e40971215a6d | base_commit:c6c5ddb256089e6b628d7651cb9759664d15f22a | evidence:[R1报告/验收映射](attachments/823/review-r1/review.md)、[反例原始结果](attachments/823/review-r1/probe-output.txt)、[输入SHA256](attachments/823/review-r1/input-hashes.json)、[冻结规范增量](attachments/823/review-r1/spec-delta-frozen.md) | findings:F-823-R1-01..06 OPEN（父env move后仍可读；正式recipe/新worker缺失；资源/兼容性能oracle后置；取消/代际等状态反控不足；V13源基线未重放；fmt失败） | 定向29/29通过不抵消新反例；HEAD稳定且实施树干净 | 当前任务认定：T02/03/04/05/06/09重开，T11保持未完成，保留T01/T10，current_step=2/11；旧完成描述仅保留子集历史 | pending_closeout_gates:merge-owned裸t+tv+新workspace完整合同档（未跑） | next:work在同一树修复并提交，受影响范围复审；不merge、不改canonical Spec/ledger。下方work-owned全部完成/9/11声明已被本轮核验证伪。

2026-10-27 stage:work | PLAN-823 r8 | T-11 完成（852 统一能力验证 phase）——**work-owned 任务全部完成** | code:ed6997394 | deps:D852✅ archived | evidence:binding.tsv 自有 7 格翻 ok+指针（零改共享面）；attachments/823/852-validation（binding.md+raw）；baseline 5 入口+acceptance 全 PASS；512 修订 churn+P95 实测 | blockers:无 work 侧 | next:**T-07 独立复审**（同树冻结上下文核 AC-01..11/V-01..15/SD、绑定 HEAD ed6997394）；复审 pass 后 T-08 merge（merge-owned 裸 t+tv+新 workspace 完整合同档照常 pending）

2026-10-27 stage:work | PLAN-823 r8 | T-09 完成（增量消费者+V-13） | code:5535b6e4d | 交付：closures_session（822同型真门会话）、link 期 env 等价检查（fn: 绑定漂移精确拒绝）、单元入口体豁免（单元非可执行入口）、cache env 透传（含 CalleeRef 重写携带被调声明） | evidence:V-13 四测 4/4（body-only 身份/生产零 IO、env 漂移精确拒绝+恢复、强制退化非零、单元独立验证）；closures 全族 28 例；回归 ~170 例零回归（820/821/822/828/abc/cache/mir）；acceptance PASS | blockers:无（T-09 面） | next:work-owned 仅余 T-11（等 D852）；其后 T-07 独立复审→T-08 merge；merge-owned 整档 pending

2026-10-27 stage:work | PLAN-823 r8 | env 契约阶段核心落地（用户裁定在计划内加 phase） | code:9468704ac | 交付：MirFunction.env/MirImport.env 声明、CREATE_CLOSURE.callee:CalleeRef（单元经 import 绑定闭包体、模块走 Func）、captures 逐位等于被调声明（borrow 条目=StrView）、wire v4 原位扩展（函数/import 记录 env 字段+callee 判别字节）、入口体/直接 Call 门改按声明判定 | evidence:closures 全族 40 例+回归 149 例全绿（820/821/822+abc/cache/mir；65 处旧 fixture 补 env:None） | blockers（T-09 剩余）：①closures 更新会话消费者（镜像 822 CollectionsUpdateSession）②link 期 fn: 绑定的 env 等价检查（program_snapshot）③V-13 子 case 2/3/4 测试接线——三者均为纯消费者侧，核心合同已就绪 | next:下会话完成上述三项后 T-09 勾选；T-07 review 等 work 全收口；merge-owned 整档 pending

2026-10-27 stage:work | PLAN-823 r7 | T-03/T-04/T-05/T-06 完成（全支持语义+资源分态+缓存兼容+聚合收口）；T-09 部分推进后定位精确设计缺口 | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | base:e9bb30606(T-02) | code:fa06bfe8b(T-03)、81a29f1d7(T-04..T-06) | deps:D822/D828/D821✅（核心窗口本树持续持有） | tasks:T-03..T-06 | evidence:semantics 10/10、resources 3/3、cache_compat 4/4、recipe 3/3、harness 12/12、first-case 4/4 —— **acceptance 36 例全绿**（ps1 全 Scope 聚合透传）；回归 149 例（820/821/822+abc/cache 档）零回归零警告；borrow 语义冻结=share 视图+静态钉住（保守扫描，§5.1.6 允许集） | blockers:**T-09 闭包体单元化**：D828 增量机制是单元级（FunctionCode::admit_unit），而 CREATE_CLOSURE 在单元中被禁（callee 走 FuncIdx、单元只准 import）；将 callee 扩为 CalleeRef 后仍余一个真设计缺口——**body 单元独立验证时看不到创建者 site 的 env 布局，EnvGet 无法定型**，需要"env 布局进入 body 单元契约"（类签名声明）的有界合同修订；不硬做、不弱化单元纪律，交回 new 做有界修订或下会话按修订后合同实现。子 case 1（D828 基线 273575B/30 修订）为 828 自有交付、822 已按同型会话重放，823 consumer 待上述修订后接线 | next:T-09 需先解 env 契约设计（建议 /auto-plan:new 有界修订 T-01 合同）；T-11 等 D852；T-07 review 等 work-owned 全部完成；merge-owned 整档门禁 pending

2026-10-27 stage:work | PLAN-823 r7 | T-02 完成（首例集成门） | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | base:master@c6c5ddb25（822 archived；T-10 经 rebase b095dc460 保留） | code:e9bb30606 | deps:D822✅ archived、D828✅、D821✅（822 交付的 List/Variant/Place 公开面为 T-02 消费基础） | tasks:T-02 | evidence:首例四测试 4/4 + harness 11/11 + 受影响面回归 ~180 例全绿（820/821/822 全 family+abc/cache/mir 档；明细见任务行）；ps1 first-case PASS；contract-decision r2/§8 窗口登记 | blockers:T-03..T-06/T-09/T-11 按 r7 依赖推进；merge-owned 整档门禁 pending | next:T-03 共享核心全语义（borrow 捕获/V-02..07 反控/§5.4 V-11 轴）；核心写窗口在本树继续持有直至交接登记

2026-10-26 stage:new | PLAN-823 r6→r7 | outcome:pass（验证phase合同可交work，非实现/独立最终review） | authority:用户确认先852底座、再接本范围验证 | changed:T-11/AC-11/V-15/SD-07、T-09/T-06/T-07无环依赖；total10→11，原完成项/current_step/状态保留 | 原模块与旧review证据保持历史，新验收尚pending；若代码/依赖/输入变化重验受影响项 | next:原D门/窗口下继续已有work，D852后做T-11 | evidence:[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)、[修订预检](attachments/852-phase-integration-20261026/preflight.json)。本轮不启动实施。

2026-10-26 stage:work | PLAN-823 r6 | T-10 完成（独立模块门） | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | base:master@929c63fd2 | code:d6838a93e | deps:D821✅、D828✅（auto-av-core 公开 API @ ff7547f9d 指纹见 contract-decision §1） | tasks:T-10 | evidence:parallel/823 十用例 10/0 绿（selected=10>0）、rustfmt/零警告、core 编译面零变化（closures.rs 未注册+父 members 未动） | blockers:T-02..T-06/T-09 集成门等 D822 review/merge 收口（执行 profile/wire/opcode/recipe 编号届时由核心 owner 登记）；merge-owned 整档门禁自然 pending | next:work 侧本轮可做范围已尽——T-10 通过≠profile/Plan 交付；D822 收口后按 §5 并行合同 T-02 刷新合同/同步 master/申请核心窗口

2026-10-26 stage:work | PLAN-823 r6 | T-01 完成（contract-decision 冻结） | worktree:主检出簿记（T-10 起唯一实施树 `D:/autostack/.wt/lang-823/auto-lang`，branch `plan-823-dev`） | base:master@ff7547f9d | deps:D821✅ archived(92b8daf60)、D828✅ archived、D822❌ executing（needs_fix 修复后再送审，未 merge） | tasks:T-01 | evidence:[contract-decision.md](attachments/823/contract-decision.md) §1–7（指纹/oracle 映射/写面登记） | blockers:T-02+ 集成门等 D822 review/merge 收口 | next:本会话继续 T-10（D821/D828 已满足，私有写面已登记）

2026-10-26 stage:new | PLAN-823 r5→r6 | outcome:pass（依赖/并行合同可交work，非实现通过或D门自动解锁） | authority:用户明确要求实际依赖审计与独立模块并行 | changed:§4–8两阶段门、T-10/AC-10/V-14、SD-01和当前入口；total9→10，原0进度/drafting及预算保持 | next:T-01后依D821/D828和私有写面登记做T-10，T-02仍等D822/D828与核心窗；不代启实施 | evidence:[审计](reports/ace-parallel-dependency-audit-2026-10-26.md)、[预检](attachments/ace-parallel-20261026/preflight.md)。

2026-10-21 stage:new | PLAN-823 r4→r5 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-823 r3→r4 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增环境所有权与调用合同的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-823 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-822及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..05/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

当前r7已获得用户对新增验证phase的授权，无需再次确认方向。原实现按原D门/窗口推进；T-11及最终汇总/送审须D852，当前没有新验证执行证据。未实现source/native/JIT等仅保留M1 owner/gap，不扩大本计划。任务总数11，current_step沿原完成项。

用户已授权本次依赖/并行合同修订，无需重新批准同范围方向。**当前状态（2026-10-27）**：T-01/T-10/T-02 已完成——T-02 首例集成门在 D822 合入（c6c5ddb25）后落地（plan-823-dev@e9bb30606，首例 4/4+harness 11/11+受影响面回归 ~180 例零回归；wire v4/id5/tag16/opcode 0x2F-31 已按核心窗口登记为实际编号，见 [contract-decision §8](attachments/823/contract-decision.md)）。核心写窗口由本树持有中（后续会话接手 T-03 前须核对该树与窗口登记）。下一步 T-03 共享核心全支持语义（borrow 捕获逃逸前门、V-02..07 反控、§5.4/V-11 轴）。保留D822已满足：首例已用真实 List\<record\> 捕获；不等824整计划。

新borrow capture的frame/env/store保护必须本计划定义和实现；821 r6没有运行期失效句柄/旧帧/跨store校验，不得当既有保证。若824先交付，T-04/T-05在加入真实Closure类别时负责嵌套pack/freeze拒绝及824合法值正控；若824后交付，记录类型/拒绝接缝交其验收。 若实际接口无法兑现，应在T-10最小接缝阶段给具体输入/符号/预算反例并回new，不静默复制WIP、弱化验证/资源门或退回第二实现。当前所有任务未实施，不因审计/模块准备改变状态或宣布Plan完成；旧§9串行排期为历史。


2026-10-27 stage:work | PLAN-823 r9 | outcome:partial，继续 executing，3/11 | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | code:c3da0e6c5f5a332c2f75a61e9264bc8c93e8a819 | dependencies:主分支已含824归档 9a9ec2256；尚未集成到本树 | evidence:[诊断、延迟原始数据与512编辑水位](attachments/823/repair-r2-diagnosis.md)；798编辑P90=704.6ms、签名P90=261.8ms、各10次；源增量P90=226.602ms，均为824整合前基线；定向代码检查通过 | remaining:824与823独立占用同一wire/profile/type/opcode编号，需完整集成并复测；非Windows锁与长期水位缺口待核；SD候选待刷新；独立review与merge-owned门禁pending | next:完成824交叉拒绝和合法控制、最新规范增量、受影响的性能与资源重验；不宣称 reviewed 或 merge。

2026-10-27 stage:work | PLAN-823 r10 | 824 整合与编号冲突解决完成（work-owned 全部收口） | worktree:`D:/autostack/.wt/lang-823/auto-lang` | branch:plan-823-dev | code:6ff173335(merge master@cefbbb524)+43304cdfb+bf4391dae | 交付：r10 重编号落地——closures=AVAD wire v5/profile id 6/TY_CLOSURE tag 17/opcode 0x32-0x34，824 保留 v4/5/16/0x2F-0x31 不变；WireFamily 分族互斥（closures/dynamic tag 集各自封闭）；r10 交叉正反控全部接线（av_closures_823_dynamic_824_compat：两 profile 真实首例身份各自正确+交叉 owner verify 双向拒绝+824 合法值正控；闭包直接/嵌套 pack/freeze 任意深度拒绝=portable/dynamic 双侧；closures 逃逸集新增拒 Dyn）；消费者并集重核（最后集成者责任）：820 union pin 46→52（含补 824 漏刷的 46→49——master 预存红）、mir-native fingerprint/lower 对 closures/dyn 指令双向受控拒绝、binding.tsv K01 备注 v5、[contract-decision §9 重登记](attachments/823/contract-decision.md)；852 证据身份 r10 重注册（41 文件 hash+依赖 HEAD：auto-ui 4903bef6）；824 侧合并缝修复（av_dynamic_824_incremental MirImport env 字段、828 P90 门控负控轴——master 预存红一并与 P95→P90 口径对齐） | evidence:probe workspace 全量 662/0 绿、parallel/823 12/0 绿、cargo check -p auto-lang 绿、`-Scope acceptance` 9 档全 PASS（first-case/semantics/admission/resources/cache/compatibility/autodown/baseline/turnaround） | blockers:无 work 侧 | next:T-07 独立复审（绑定 r10 整合后 HEAD bf4391dae）；merge-owned 裸 t+tv+新 workspace 完整合同档 pending，通过前不合入

2026-10-27 stage:new | PLAN-823 r9→r10 | outcome:pass（经用户明确要求解决824冲突的有界身份修订，非实现/最终复审通过） | authority:用户要求整合已合入824并补齐全部验收 | changed:823 wire/profile/type/opcode分配，新增824交叉正反控，824已发布身份不变；原AC/V与预算、10次至少9次≤2000ms保留 | next:work整合并以新基线重验；独立review和merge-owned门禁pending。
