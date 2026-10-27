---
plan_id: PLAN-824
status: executing
feature_name: AV独立动态Variant与可移植冻结边界
plan_revision: 7
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
new_spec_components:
  - docs/specs/auto-av/dynamic-portable-values.md
  - docs/specs/auto-mir/dynamic-cast-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-824] AV独立动态Variant与可移植冻结边界

## 0. 变更摘要

本次r7仅新增**T-11统一能力验证阶段 / AC-11 / V-15**，消费852已交付底座；原T-01/T-10完成项、私有模块证据及其他原验收/预算保留，任务总数10→11，current_step不增加。范围与无环交接见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。当前已实施，旧稿“仅起草/尚未创建”措辞属于历史，实际进度以§8和§9为准。

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付DynValue、受检类型擦除/恢复与portable冻结。2026-10-26 用户授权实施:T-01 合同冻结与 T-10 独立模块门已完成(2/10,executing);T-02 集成门仍等 D822/D828 及核心/注册窗口。

## 1. 目标

实现与静态表示分离的安全DynValue，pack/is/borrow_as/take_as及失败保留，保持owned语义；对已交付白名单数据实现版本化portable树冻结/恢复和typed宿主入口。代码缓存/数据冻结分别命名，不把活handle持久化，不改817 scalar结果合同。

成功标志：真实受检MIR→ABC→AV在新profile下执行，正例独立oracle一致、坏输入在消费前拒绝、资源/版本/恢复可验证，旧core-i32及实际已合入的817/818标量路径兼容。

非目标：NaN-boxing/unsafe union、通用Batom/JSON引擎、任意反射/完整用户泛型、资源/闭包/借用冻结、共享循环图、真实CTE复合结果接线、native AutoABI/FFI/不可信插件隔离、热重载迁移。

范围只在auto-lang仓的共享实验核心、独立fixtures/recipe/本计划脚本/证据；根workspace成员/产品manifest、旧VM、UI/Man/SDK及他方在途实现不改；仅允许本recipe在当前真实owner处追加最小注册/父侧判定薄接线，并按实际根Rust触面验证。共享注册位置服从814–816实际迁移后的owner，不复制协议/执行器。

## 2. 架构方案

```text
支持的已有Checked HIR / 明确未信任typed fixture
  → av.dynamic/1 MIR + verify
  → ABC encode → decode/verify → LoadedModule
  → AV typed access / layout / lifecycle → typed result/diagnostic
             ↕ 783 managed store（代码与portable载荷分域）
   新worker恢复重验 + AutoDown父侧独立oracle
```

执行profile拟为 `av.dynamic/1`，名字/编号由T-01确认不冲突后冻结；不复用core-i32授信。纯语义TypeRef与AV LayoutRef、native target ABI分离；不再创建VM私有推导器或第二MIR。

## 3. 技术栈

Rust、D依赖交付的 `experimental/ac-av-probe/` 独立workspace（已观察package为auto-av-core/auto-av-probe，work须重核），783 ManagedStore、796语义事实、现有probe/run/contained worker。

影响入口：共享MIR/verifier/ABC/AV及typed host seam（D-822）；拟新增 `core/src/{dynamic,portable}.rs`、`probe`独立producer/恢复门及fixtures；796模型/817冻结schema只读。

拟新增 `scripts/verify-av-dynamic-824.ps1`，本计划测试前缀 `av_dynamic_824`、`test/cases/av-data/824/`、独立recipe `av-data-824`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

2026-10-26用户明确授权修订852及823～826，先交付测试底座，再由在途计划加入本范围验证phase。本次r6→r7只修订合同，不启动实现/会话、不改822或实施树，不追加语言/native/AJ能力。新增验收属于用户明确授权，原已完成任务和证据按其实际范围保留；新AC-11尚无执行证据。

2026-10-26用户授权审计/修订823–826并行边界，本轮仅改计划与战略，不启动实现或其他会话。取材初始821 archived r6、822 executing 6/9；期间822 R1 needs_fix重开至4/9，两次均非D822（他方进度原样保留）；实际输入/代码接缝与指纹见[依赖审计](reports/ace-parallel-dependency-audit-2026-10-26.md)及[inputs](attachments/ace-parallel-20261026/inputs.json)。旧“整计划串行/仅只读准备”排期按下文两阶段合同更新，原目标/非目标/预算和已有进度保留。

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-822**=上游独立最终review、冻结门禁、合入、Spec/merge收据齐全，不消费WIP。移除D823，正式集成只等D822/D828及核心窗口；白名单不消费ClosureEnv/CallClosure，因此与823是分支，不是后继。 T-01只读准备可先；T-10在D821/D828及私有写面登记后可做真实独立模块开发，不占公共核心窗口。实际全路径构建T-02起遵守本阶段D门/窗口与原授权。具体规则见§5并行合同。

拟唯一实施树 `D:/autostack/.wt/lang-824/auto-lang`，分支 `plan-824-dev`；本次修订不创建/切换实施树，现有工作依§8完成项及§9收据。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

## 5. 详细设计

### 5.1 当前执行合同

1. Dyn只接显式类型擦除，Unknown/Error不能pack。白名单为已交付bool/精确数值/char/owned Str/纯owned record/List/Option/Result/有限enum；Fn、外部资源、borrowed str、闭包类别明确拒绝，不因823支持闭包就自动可Any/freeze。
2. 安全enum/结构实现，静态数据路径无Dyn依赖；复杂载荷保留具体TypeRef及拥有责任。pack非Copy值须move或显式clone；take_as失败保留原值，borrow_as无释放权。
3. 同宽异名record仍不相容；bool/int/句柄不猜测；None/Null/Uninit/缺字段不同态（普通静态profile无Null时拒该类别）。任何host入口核验store/type/permission，禁止铸造handle或任意宿主地址。
4. portable mapping独立schema/producer，保存稳定类型与revision/字段分支身份/精确bits/UTF-8 bytes/树内ref；拒进程地址、ExecTypeIdx、frame/borrow token/trusted。首版仅owned无环树，任何引用环/共享图类别拒绝，失败原值不消耗。
5. decode先界限/预算后语义/类型/当前profile重验；在新执行域重建handle，不反序列化凭证。名义类型/布局revision变化必须正确失效或受控拒绝，不能复用旧访问offset。
6. 沿783真实store与Design40/43 root/lease/GC；适配Atom/Batom现有能力时只声明实际carrier，新增类型使用具名payload；不替换796 PortableValue或817 scalar schema。跨进程恢复必须有当前依赖/权限检查。

### 5.2 有界首例与资源政策

T-01冻结支持/拒绝、类型/指令/wire版本、生命周期、输入前门及计量决策，不在下游实现中临时猜规则。T-02必须完成：typed输入构造含i32/Str/List/Option/owned record的Dyn，受检take和失败对照；freeze成真实portable bytes存store，在新worker中恢复/重验/重建handle后再执行读取。固定结果/类型名义/位型/资源oracle及正式AutoDown真实父侧判定。

首例不可行则记录实际输入、拒绝/资源原因和合同冲突，回new显式revision；不能改用旧VM、弱化verifier、去掉oracle或只留纯模型。未知语法/接口的fixture是未信任typed输入，真verify授信后执行，不把未落地接口描述当证据。

沿D依赖已冻结输入≤4MiB、module/每函数表≤65536、验证≤200万步/5s、执行≤100万步/1024帧/2s的**适用上限交集**；旧profile更小cap仍生效，单位/计步在T-01核对，不按本段放宽旧合同。新增类型描述、路径、图、construct/drop/freeze必须申请预算并使用有界显式遍历。值数据合计≤8MiB（槽+聚合+buffer+环境+Dyn），受管总32MiB（hot8/staging16/execution8），不是各类各领8MiB；allocator/库暂存按真实峰值报告。新worker private-bytes相对空载增量沿798上限128MiB，包含本次allocator/解码/临时对象；父侧采样瞬态峰值，与自有计数共同判定。native工具进程另列归因，不从worker实际峰值扣掉前端。

这些是拟议实验资源政策，非用户给定或永久语言限制；T-01如不能成立须带边界输入回规划，不能在work直接提高。OOM/预算/cancel/infra与invalid/unsupported/trap分态，分配前拒绝；cleanup预留记账不受正常fuel耗尽取消，不暗加语法深度限制。

### 5.3 存储、注册与兼容

保留D-798冻结的标量编辑负载和采样口径，定向重放其请求到当前结果P95<1000ms；冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。性能/峰值测量本机单实例串行，负载状态入证据；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

首例沿783真实代码producer、root/lease/GC，cache恢复重走MIR/ABC门；类型/字段集/layout/profile/语义版本入依赖，坏内层即使好hash也不得执行。活帧/borrow/宿主地址不存cache。首例独立recipe注册和父侧expected比较是T-02交付，不允许“有fixture但execution pending”结项。

共享注册入口现观察为 `crates/auto-lang/src/test_cases/{sema,mod,stale}.rs`（若已迁移以816交付为准）。只追加自己的recipe/身份和独立worker处理，不复制整个sema worker，不修改v1/v2解码或他方recipe合同。若协议不足，在T-01由实际owner决定最小版本化增量，依赖实施暂停。

旧标量语料与实际已合入817/818路径必须重放：仅保证原profile兼容及未知新profile明确拒绝；不强制其他后端实现新数据。本计划没有完整native/CTE复合消费oracle。固定portable样本/手审bit oracle及独立编码检查器与AV结果对照；不得以796模型或817 scalar结果冒充真实复合CT执行。

### 5.4 ABC共享合同与本计划责任（r2）

依据[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44 §15](../design/44-av-data-layout-place-and-ownership.md#15-abc补充合同与逐profile验收r2)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)，本计划承担**动态类型与可移植授权**：Dyn的is/borrow/take共用820类型等价，名义owner/revision不因同布局擦除；模式和失败take的非消耗语义接入InstructionSpec。portable必需kind/类型/版本/布局与权限要求不可作为debug剥离；宿主grants由当前host提供，payload的requires不能自授权。解码/freeze/unfreeze/clone均有数据量预算和取消点，恢复须重验当前类型和授权。

语义MIR、portable ABC和私有LoadedProgram分层，不另建语义IR。825不是本计划硬依赖：其已合入时适配受检执行表示，未交付时允许同一私有接口的安全参考路径；未知profile不进无验证fallback。kind/entry/imports及歧义旧缓存策略消费D-798修复，不猜函数数。owner、活分支和Drop/必要类型是语义，strip debug不得移除。

类型/路径分析、clone/drop/编解码及cleanup计入既有合计预算；输入要求的capabilities与当前host grants分开，优化不绕过授权、逻辑fuel和取消。帧arena、slot复用、融合及紧凑wire属825/826或另案性能，不以其完成替代本计划正确性。T-01冻结V-11具体fixture/反控和各层合同映射，T-03/04实现，T-05覆盖恢复，T-06汇总；不得只放设计注释而漏验收。

### 5.5 文档复审补充合同（r3）

Dyn/freeze白名单递归检查全部类型实参/字段，List/Option等外层合法不掩护内部closure/borrow/resource。合法Dyn错型take属于运行期checked失败而非坏字节码，源保留；不存在/过期TypeRef属于准入非法。portable owned树每节点唯一入边，重复拥有引用（无环DAG也一样）/环/重复字段/缺字段拒绝；重建半图失败逐个回滚。

准入错误在decode/verify/load门拒绝、dispatch=0；已准入程序的越界、失效句柄/guard、错型take、取消等允许此前合法执行，须断言危险目标访问/提交/目标callee计数0及原值/清理，不要求整次dispatch=0。prepare/commit只保护尚未提交的转移；已发生的guest求值/调用效果不回滚。

消费前冻结已合入(profile, wire/schema, semantic-version, producer)矩阵；825/826可前置但不是本计划硬依赖，820指令表尚未交付时它们沿D798现有合同，不反向等待本计划。当前core/注册窗口只能在依赖满足后持有，不持锁等其他owner。注册的最小接线是范围内例外，不等于修改旧VM或产品默认；814–816只按实际已迁移owner交接。

资源表沿798精确族口径：模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536；index上限还须小于对应表长及总额，不能为取最大索引挤掉必要块/指令。32MiB分区和128MiB进程增量分别验收，不扣前端/allocator/暂存，已有约211MiB前端证据不能记债后当D798已达。T-01核当前D门及同口径证据，冲突先回规划，不擅自豁免或提高数字。

T-01在contract-decision冻结case→子case→Scope→真实入口→阶段/码→oracle→必需/适用矩阵，包含V-12；T-02使first-case和recipe正控可执行，T-03/04实现与反控，T-05核恢复/兼容，T-06验必需集合无漏例/重复冒计/skip。合法特定profile外输入应按合同拒绝，不能误算成适用例未执行而跳过。

### 全链路增量兼容与828优先排期（r4）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：Dyn/冻结值绑定schema/layout/语义事实并受检恢复；未变编译/装载单元不重做，旧值不按新布局解释。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 并行模块与集成两阶段合同（2026-10-26）

移除D823，正式集成只等D822/D828及核心窗口；白名单不消费ClosureEnv/CallClosure，因此与823是分支，不是后继。

**T-01→T-10独立模块门**：依D821/D828及已合入公开接口开展821已交付scalar/Str/record的Dyn容器、错型take保源、portable树验证/冻结/恢复与半图失败回收；List/Variant保留给D822后，不假造其类型。单一生产源为`core/src/dynamic.rs、core/src/portable.rs`（拟新增），测试壳拟为`experimental/ac-av-probe/parallel/824/`独立workspace/lock，仅用同树已合入auto-av-core。T-01冻结可达符号、具体路径/owner、支持子集和预算；T-10先证明最小可编译接缝，再实现真实模块与故障oracle。可由harness重导出已交付core公开模块并路径编译同一源，生产阶段只加薄adapter；不复制值/IR/store/VM，不改core/lib.rs/父members，不产生依赖环。若需要pub(crate)私有符号，不绕过权限，登记具体缺口并申请独立验收合入的小接缝；不能从上游WIP取代码。

**T-02集成门**：T-10通过不等于profile/Plan交付。满足原D门后同步最新master，刷新接口/版本/预算，取得共享核心/注册窗口，完成原首例真实verify→ABC→AV/store/new-worker/AutoDown和trace/资源；继续T-03..06/T-09及最终review。所有模块独立通过不能代替原全路径、精确增量、P95/内存或反控。T-10测试命令拟为`cargo test --manifest-path experimental/ac-av-probe/parallel/824/Cargo.toml`（入口由T-10建立，当前未运行，selected>0），正式脚本必须汇总新V-14，不是独立壳绿就全计划绿。

**写面与交接**：本计划只写上述私有新增源、parallel/824及自身fixture/报告；现有mir/contract/ownership/place/layout/instr_spec、codec/verifier、av派发/装载/snapshot、公共lib/manifest和共用driver/recipe注册默认一个核心窗口。基面/版本/文件符号、前后owner、兼容证据及接受/释放需记录；公共tag/opcode/version由核心owner统一登记。复审期间其他Plan可以继续无公共写面的T-10，不能拿未审公共定义开工或持窗等上游。最后集成者重核已交付消费者并集，复审独立、整档仍merge-owned，测量单实例。详见[共同审计/窗口表](reports/ace-parallel-dependency-audit-2026-10-26.md)。

Fn/闭包/borrow/resource仍递归拒绝。823未交付时V-12用已交付Fn及未知/保留tag拒绝证明封闭性，不制造Closure类型；823先交付则本计划必须测真实嵌套Closure，824先交付则823 T-04/T-05承担该新类别兼容控制。新borrow_as/typed host访问的store/type/生命周期由本计划T-01冻结并实施，不继承821未交付的runtime保证。

### 852底座交付后的统一能力验证phase（r7）

本计划新增T-11，原实现可以继续；仅T-11及最终汇总/送审消费D852。D852是852当前revision的独立review、
冻结门禁、实际合入、Spec和merge收据齐全的**底座交付**，不要求M1全绿；852不反向等待本计划。
不消费852草案/工作分支WIP。原D门、单写窗口与原非目标不变，详见[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)。

固定映射：**scenario:D01～D09/W01～W05/I04～06/10～12**，对照本计划原V表。T-11工作入口先记录每个ID的实际profile/version、
source/checked HIR/未信任typed MIR起点、正式recipe、具体V子例、独立oracle及raw证据。
本范围必须实测：Dyn正确/错型take保源、名义身份/递归白名单、owned树冻结与新worker恢复、精确位型、版本/权限失效、资源与原T-09增量。
边界：共同Dyn/portable源码API、复合CTE、native/FFI不加入本计划；实际缺失的source/native行仍为M1 gap。不得删必需适用行、以skip/模型/旧VM回退补绿。

typed验证与Auto源码验证分轴；对应源码尚不支持时，保留M1源码/native缺口及明确owner/关闭条件，
不得把同号typed fixture当作同源四路径通过。真正范围外且有明确owner的M1缺口不阻塞本计划，
但本计划原已承诺的能力/实际适用T-11项缺证据仍阻止AC-11。

T-09在T-02～05/T-10后做既有增量验证（不等待T-06/T-11）；T-11在T-05/T-09和D852后完成；
T-06最后汇总原门及新V-15，T-07独立核AC-11。避免T-09“其他work完成”与T-11/T-06互等。
原T-01/T-10已验子集证据保持；新增AC/V不能用旧模块绿关闭，受影响内容按实际输入/代码/依赖变化重验。

852拥有通用case/证书/报告合同；本计划只追加自身binding/证据，协议/registry/SDK adapter按共享测试窗口
单写交接，关联MIR/AV变更另守核心窗口，不复制runner/VM/store。a2r有效认证按内容/依赖复用，
日常只emit+金样比较，不由T-11重复编译认证。原P95/内存/磁盘及负控不降，work仍只定向测试。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/dynamic-portable-values.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/dynamic-cast-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-824独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |
| SD-07 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 852底座分母→追加本计划实际支持起点/recipe/证据与范围外gap；不冒领source/native能力 | 统一验证且不扩原语义范围 | AC-11 / V-15 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。


## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

不变量为Design44 I-01..12及§5。以下V均为**必需补充反例/正控**，T-02/T-04将其变为真实入口；固定语料总数不能替代。每案保存输入身份、阶段/码、result/trace/bits/owner与原始日志。期望由人工或独立小模型确定，不能由被测布局/清理算法生成。

| Case ID | AC / work owner | 输入/状态 | 实际或拟入口 | 可观察oracle与区分断言 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02；T-02 | 同一i32/Str/record pack→is/borrow/take与错型take | 新profile verify/ABC/AV，av_dynamic_824 family | 保值/名义身份，错型take原值仍可读且未drop | 擦除/恢复；仅声明支持面 |
| V-02 | AC-02/03；T-03/04 | pack move后读源、clone两份、borrow入容器 | 新profile verify/ABC/AV，av_dynamic_824 family | 源读/逃逸拒绝；显式clone独立清理，无任意Copy | owned行为；仅声明支持面 |
| V-03 | AC-03/05；T-03/04 | MAX_U64/MIN_I64、±0/±∞/NaN payload经Dyn/freeze | 新profile verify/ABC/AV，av_dynamic_824 family | 逐位保真；整数与活对象id同值仍整数 | 精确bits；仅声明支持面 |
| V-04 | AC-04；T-03/04 | 闭包/资源/借用/环/共享图/地址伪装输入 | 新profile verify/ABC/AV，av_dynamic_824 family | 受控unsupported/invalid，原值不消耗、无资源泄漏 | portable边界；仅声明支持面 |
| V-05 | AC-02/04；T-03/04 | 相同payload不同store重建、错owner/type/layout版本、坏内层好hash | 新profile verify/ABC/AV，av_dynamic_824 family | 新handle合法；伪造输入在执行前拒绝，不信trusted | 新worker准入；仅声明支持面 |
| V-06 | AC-02/03；T-03/04 | 深链/大树边界、截断/超长/未知分支、分配第n次失败 | 新profile verify/ABC/AV，av_dynamic_824 family | 分配前resource或invalid，恢复半图回滚，当前根保留 | 预算/回滚；仅声明支持面 |
| V-07 | AC-05；T-04 | 合法输入cap内/超1单位、瞬态峰值、故障第n次 | 新worker+父侧budget/allocator hook | resource分态；预算前拒的该申请attempt=0，故障注入attempt存在但successful_alloc=0；先前合法事件保留，危险访问/提交0；cleanup/真实峰值断言触发，无panic | 不能只看终态RSS |
| V-08 | AC-06；T-02/05 | fresh、同store新worker重启、改type/layout/profile、好hash坏payload | 783真实恢复→verify→AV | 合法恢复结果相同；依赖变化不复用；非法执行计数0 | 不缓存PASS或trusted |
| V-09 | AC-06；T-05 | core-i32及实际交付817/818旧profile、新profile送旧consumer | 实际原recipe/consumer | 旧结果/诊断不变，未知profile明确拒绝 | 无假native/CTE支持 |
| V-10 | AC-01/04/06；T-06 | worker错结果/错bits、漏case/skip、错工具身份 | 正式AutoDown recipe+父侧oracle | 合法正控PASS；每项错实际使总体非零/ERROR | 缺必要工具/路径不可绿 |
| V-11 | AC-02/03/04/05/06；T-01/03/04/05/06 | 同宽异名Dyn take、错revision/跨store、伪造requires当grants、缓存后撤销授权、去debug的类型/owner、冻结/恢复取消或超额 | §3真实MIR/ABC授信、执行及store新worker恢复；T-01冻结输入，T-03/04使可运行 | 合法错型take运行期失败且原值保留；坏身份/类型ref/能力在适用准入门拒绝，运行期失效访问按目标计0；撤销授权后的cache不执行；strip后仍严校类型/owner，失败冻结原值和清理责任保真；好hash不得绕过；准入非法dispatch=0，运行期故障按§5.5断言目标危险操作0 | 仅本profile；标量/旧消费者仍V-09，受管/进程峰值沿V-07 |
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | List<Fn>/record含borrow的嵌套pack/freeze；已交付Closure时另测真实嵌套Closure，未交付时测保留/未知tag拒绝；合法Dyn<T>请求已知U≠T；未知TypeRef；同owned子节点双引用/环/重复字段；恢复第n次分配失败 | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | 递归不支持类型/坏树在规定准入门拒绝；合法错型take可执行并返回失败且源可用；无复制owner，恢复半图回滚；未消耗原根的普通freeze失败保持原值；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力；Closure组合按§5由最后集成者验证，不以未知tag替代已存在类别 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_dynamic_824`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-dynamic-824.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
- fmt/check按该独立manifest执行；开发与修复只定向测试。禁止work/internal/final review触发整档t/tv/tt/tb/tu/taa。
- 合入：最终review后冻结HEAD，由merge执行裸`cargo t`+VM/编译器触面`cargo tv`，以及新workspace完整有限合同档；按实际diff再选其他触面，tf只主检出到期单实例。review pass必须记录merge门禁pending。

角色：用户未指定模型或分离实现者。work负责实现和反例；内部检查只表示可送审；最终review必须独立上下文、同树只读核验，不修代码；merge负责冻结门禁/规范/归档/guard清树。当前未指派或启动任何角色/代理。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-13 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

### 并行同源与依赖边界验收（新增work-owned）

| Case ID | AC / owner | 输入与真实入口 | 区分性oracle | 覆盖边界 |
|---|---|---|---|---|
| V-14 | AC-10；T-10/T-03/T-05 | 同一拟新增生产源由parallel/824构建并在核心窗口后由真实生产adapter消费；坏输入/失败分配/版本或窗口负控 | 实际821值的冻结/恢复逐位或字节等价、错型take保留源、分配失败半图全回收；List<Fn>/record含借用拒绝；真实Closure已交付时不能只用unknown-tag代测；生产同源模块接真实verifier/AV/store；同源hash/函数映射一致，越写公共面、WIP依赖或适用必需组合缺例不得就绪 | T-10只证明已交付子集模块；T-02..06补正式profile/cache/AutoDown，未知/未交付类别诚实拒绝，不冒领全路径 |

### 852统一能力验证（新增work-owned）

| Case ID | AC / owner | 输入/实际入口 | 可观察oracle与反控 | 边界 |
|---|---|---|---|---|
| V-15 | AC-11；T-11 | scenario:D01～D09/W01～W05/I04～06/10～12；D852已合入runner及本计划真实recipe/V子例 | Dyn正确/错型take保源、名义身份/递归白名单、owned树冻结与新worker恢复、精确位型、版本/权限失效、资源与原T-09增量；错结果/漏适用行/假source起点/旧证据错版必非绿；保留原资源/增量及独立expected | 共同Dyn/portable源码API、复合CTE、native/FFI不加入本计划；实际缺失的source/native行仍为M1 gap；实际起点/范围外缺口有owner，无source支持不冒领M1 |

## 7. 验收标准

| ID | owner_stage | 可观察完成标准与方法 |
|---|---|---|
| AC-01 | work | T-02真实首例经source支持面/typed fixture→verify→ABC→AV，内容/位型/来源和独立oracle一致，V-01/10通过 |
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（本profile含owned值）；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 ；核AC-10及V-14的真实模块/集成边界；同时独立核AC-11/V-15 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |
| AC-10 | work | V-14的T-10独立同源模块与T-03/T-05真实集成证据齐全，D门/公共写面接受释放/版本与消费者矩阵可核；不以模块绿替代原全路径，适用拒绝/失败控制与原增量/资源门不减 |
| AC-11 | work | T-11及V-15映射/真实执行/负控/证据齐全，消费D852已交付身份；本计划适用行不得漏测/skip，范围外source/native/M1 gap保留owner且不冒领；日常a2r复用有效认证、零Rust重编译；原AC/V/预算不减 |


## 8. 执行步骤

已完成任务以本表[x]和原始收据为准；新T-11未完成，current_step不增加，review/merge不能提前勾选。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施授权；只读准备可先 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/824/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜；冻结§5私有同源模块/harness可编译接缝、T-10子集与集成门、实际公开API和窗口登记 | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 [✅ 2026-10-26:contract-decision.md 冻结(基面7513bcee0、D821✅/D828✅/D822❌4/9、支持矩阵/预算/oracle/写面登记),T-10 落地后补 §11 r1 实施修订(深度256/计费口径/V-14c oracle定稿)] |
| [x] T-10 | work | T-01、D821、D828、独立私有写面登记 | 新`core/src/dynamic.rs、core/src/portable.rs`与parallel/824独立harness/lock；最小接缝先编译，再完成821已交付scalar/Str/record的Dyn容器、错型take保源、portable树验证/冻结/恢复与半图失败回收；List/Variant保留给D822后，不假造其类型；不改公共装配 | V-14模块子集；AC-10；未建入口/0例不能绿，完整case待T-03/T-05生产同源证明 [✅ 2026-10-26:worktree plan-824-dev 29461c706(基面7513bcee0),DynValue pack/is/borrow_as/take_as/deep_clone+portable freeze/unfreeze/RestoreContext,av_dynamic_824 22用例 22/0 绿 selected>0 零warning,core 构建面零改动(cargo check 过),V-14a..f 模块轴全覆盖;**R1修复 2026-10-27(6a68240e8)**:F01/F02 修复入同源模块(expect_leaf_kind/binding 携带类型+根 tydesc 全量准入/迭代+计费),harness 24→27 绿(F01 8×8 kind 门+checksum 重算重放、F02 空 List/不活跃 arm 名义依赖+改型拒+新位号恢复),零警告零 fmt 漂移] |
| [x] T-02 | work | T-01/T-10、D822、D828、核心/注册窗口接受 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记；只消费T-10同源模块，不另实现；刷新已交付基面/版本、记录接受 | AC-01/06；first-case、V-01/08/10正控 [✅ 2026-10-27:合同r2/r3(身份冲突回记 id5/AVAD v4、白名单+List/Variant、核心窗口登记,合同§12/§13);同源模块r2 24/24绿(e23c2b628);核心 av.dynamic/1 全管线(645378ab5);probe 首例 4/4=执行9分/台账冻结/恢复重验/portable逐位往返/错型take陷阱(ff120762c);recipe链 ac-av/dynamic-run-v1+av-data-824 语料3/3+verify脚本四Scope全绿(9882277dd);822/core 回归绿;资源采样档与V-02..07反控归T-03/T-04] |
| [x] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile；生产直接消费T-10同源，完成适用V-14集成轴 | AC-02/03；av_dynamic_824定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 [✅ 2026-10-27:语义档 av_dynamic_824_semantics 20/20 绿(V-02 pack 源可读/两包两 Dyn/take 共享可再取/is-false 不 trap、V-03 f64/f32/i64/u64 位保真经 entry-param 缝、V-04 共享 DAG freeze 拒+恢复 handle detached、V-05 双域重建+unknown/layout 准入拒、V-11 同宽异名+Fn 操作数+trace 无授权载荷、V-12 类型门封闭 List<Fn>/RecFn/RecP/Place 槽/Dyn 嵌套、V-14 生产/模块冻结字节等价)；verify 脚本 semantics Scope=probe 20+同源 harness 24 双绿;实测边界:宽标量常量 data-profile 专属→dynamic 经 entry-param 缝(合同 r4);**R1修复 2026-10-27(6a68240e8)**:F02 adapter 轴 portable_bindings_of 记忆化类型解析(总节点预算),新增语义档 21/21 含真实 adapter 改型拒用例;admit/ty_ident 全迭代化(1MiB 线程 ~242 深边界=基线一致)] |
| [x] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 [✅ 2026-10-27:资源/故障档 av_dynamic_824_resources 11/11 绿(V-06 深链 255 全层往返/258 白名单准入拒、V-07 双故障缝 fault_after_allocs 无半提交+arena attempt≥1/successful=0/先前事件保留/peak 两侧边界、trap/cancel 分态清理、worker 父侧采样 128MiB B-line×4 含 dyn-out portable 轴)；admission Scope=首例准入负控、resources Scope 全绿;**R1修复 2026-10-27(6a68240e8)**:F03 修复 resolve_dyn_type MAX_DYN_NODES 上限+节点计费 steps(DynTypeNodes/InstrSteps 分态)、dyn_whitelisted 记忆化、deep_size 计入 Dyn 类型树;resources 11→12 绿含 DAG 展开两侧+review 原形(17 层→Resource)+紧预算 InstrSteps;deep_size 口径变化后既有 peak/边界控制全绿] |
| [x] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 [✅ 2026-10-27:cache 档 9/9 绿(A23/A24 恢复身份+三类依赖轴 clean miss、好信封坏 ABC 字节/重算 checksum 的坏结构均拒、R14/R15/R16 伪造负控双指纹具名、AVAD v4↔旧代际互不可读、A23/A24/A21/A22 层分离)；compatibility Scope=821 首例+cache/822 首例/821·822 语料/798·820/817/818 语料全绿——**发现并修复 T-02 核心窗口遗漏**:ac-mir-native fingerprint/lower 两处 MirInstr 穷尽 match 补 DYN 三臂(渲染+受控拒绝),818 ac_mn_probe 20 红→24/24 绿;**R1修复 2026-10-27(6a68240e8)**:cache 9/9 复绿;兼容六族语义面全绿(821 4+7+3/822 5+3/798·820 14(--skip turnaround)/817 5/818 18(--skip 时序门));时序门(798 formal_turnaround/818 turnaround+liveness)红=Plan 856 merge 串行门占机负载伪影(时窗 1.7-2.5× 基线,两轮轮替不同测试),安静机器复跑 pending,闭合表在案] |
| [x] T-06 | work | T-05/T-09/T-11 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据；正式脚本汇总V-14/AC-10，无模块通过冒领集成或缺适用必需例 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总；最终汇总AC-11/V-15，不以852基础层PASS代替本能力验证 [✅ 2026-10-27:acceptance=9 Scope 子进程聚合透传+包级 fmt/check 零警告健康门全 PASS(first-case 4/semantics 20+harness 24/admission 1/resources 11/cache 9/compatibility 旧六族/incremental 5/baseline 6 入口/autodown 3)；V-10 负控扩展至 take-probe 行(result 轴必红,ledger 单事件轴如实不适用)；受管/进程峰值=worker B-line 父侧采样门(resources 档)；SD 草案随 frontmatter 待 merge 沉淀;**R1修复 2026-10-27(6a68240e8)**:acceptance 复跑 8/9 Scope PASS(first-case/semantics 21+harness 27/admission/resources 12/cache/incremental/baseline/autodown)+health 手工核(0 警告/fmt 零漂移);REQ 钉住三项 close-oracle 新用例;compatibility 语义面手工全绿、时序门负载归因待复跑] |
| [x] T-09 | work | D828、T-02～05/T-10（不等T-06/T-11） | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 [✅ 2026-10-27:DynamicUpdateSession(D828 unit-admit/link/snapshot 真实消费者,A23 持久化)+V-13 四子case 5/5 绿:①基线 campaign 重放(30 修订,结构+全对+P95<1s 三门)②两函数 body 编辑身份(beta-only 计数=1/Arc::ptr_eq alpha 携带/旧快照旧真值)③签名改 link 拒+授权生命周期(未授权拒于生产前计数=0/授予通过/撤销再拒/会话存活)④强制退化 unchanged_degraded=1 门非零⑤(超合同)512 修订 churn 有界+worst 0.2ms/修订] |
| [x] T-11 | work | T-05/T-09、D852、原D门及所需测试/核心窗口 | 执行§5统一能力验证phase：scenario:D01～D09/W01～W05/I04～06/10～12逐行映射到真实起点/recipe/V子例；Dyn正确/错型take保源、名义身份/递归白名单、owned树冻结与新worker恢复、精确位型、版本/权限失效、资源与原T-09增量；在attachments/824/852-validation记录binding与raw，范围外M1 gap保留owner；不实现额外前端/native/AJ | AC-11/V-15；本计划既有正式脚本新增显式`-Scope baseline`（T-11建立），acceptance最终包含其结果；定向执行/反控，selected>0且无漏适用项，不触发整档或隐式Rust认证 [✅ 2026-10-27:D852=archived(landed 4a81d2ebc,早于本树基面)已消费；binding.tsv D01..D09 自有行翻 ok:av-dyn-824(仅翻自有格+追加指针,schema/金样零改)，D02 新增 take-probe 冻结面入正式语料(3/3)；W01..05/I04..06/I10..12 证据指针追加；512 churn(I10)+worst 0.2ms(I11)+campaign P95；attachments/824/852-validation(binding.md+raw 收据)；-Scope baseline 6 入口全绿并入 acceptance；零 a2r 认证/零 Rust 重编译；范围外 M1 gap(ac 复合 native/avm/a2r dyn API/源码行)保留 owner 在案;**R1修复 2026-10-27(6a68240e8)**:baseline 6 入口复绿(修复后代码 6a68240e8);修复闭合表 attachments/824/review-r1/repair-closure.md] |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成、T-10及AC-10完成、T-11/AC-11完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论；独立核依赖/同源与窗口并集及V-14 | AC-07；merge-owned全档仍pending；独立核AC-11/V-15，新证据未验不得沿用旧结论 |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |


## 9. 复审记录

2026-10-27 stage:work | PLAN-824 r7 | outcome:pass（**T-07 R1 needs_fix 修复轮**：三项 P1 + unused import 修齐并提交，受影响面定向验证完成；不送最终 review——受影响面复审由独立上下文执行） | worktree:D:/autostack/.wt/lang-824/auto-lang | branch:plan-824-dev | base:2093675731(R1 被审提交) | code:**6a68240e8**(11 文件 +1198/−97) | deps:不变(D821/D822/D828/D852 archived) | tasks:T-10/T-03/T-04/T-05/T-06/T-11 重开项全部翻绿(current_step 3→9) | evidence:F01 expect_leaf_kind 8×8 门+checksum 重算重放、F02 binding 携带类型+根 tydesc 全量准入(空容器/不活跃 arm/改型/新位号)+真实 adapter 轴、F03 MAX_DYN_NODES 上限+steps 计费+记忆化白名单+deep_size 计类型树;av_dynamic_824 全族绿(first-case 4/semantics 21/harness 27/resources 12/cache 9/incremental 5/baseline 6/autodown 3)+兼容六族语义面绿;闭合表 [repair-closure.md](attachments/824/review-r1/repair-closure.md) | 修复副作用裁定:admit/ty_ident 迭代化(1MiB 线程 ~242 深边界与基线一致,stash 对照在案);budget_nodes_freeze 控制迁 pack 侧类型上限+freeze 侧改嵌套 List 控制覆盖 | blockers:①compatibility 时序门(798 formal_turnaround/818 turnaround+liveness)因 Plan 856 merge 串行门占机呈负载红,安静机器复跑 pending(语义面已绿,不阻塞送审);②823 编号冲突归 merge;③merge-owned 裸 cargo t+tv+新 workspace 档全程 pending | next:T-07 受影响面独立复审(冻结 6a68240e8),闭合表 per finding 供核

2026-10-27 stage:review | review_scope:final | reviewer_identity:本聊天 Codex，独立于实施 handoff、只读实施代码，无额外代理/模型身份声明 | PLAN-824 r7 | outcome:**needs_fix** | worktree:D:/autostack/.wt/lang-824/auto-lang | branch:plan-824-dev | reviewed_commit:2093675731fb1fe1b9aaa7291cdbbdee672bf9b5 | base_commit:c6c5ddb256089e6b628d7651cb9759664d15f22a | dependency_revisions:见 [R1 报告](attachments/824/review-r1/review.md) 与 inputs.json | acceptance_results:AC-02/03/04/05/06/10/11 fail、AC-01/09 partial、AC-07未通过、AC-08 merge pending | findings:F-824-R1-01(P1,leaf type/payload错配使take_as(I32)成功返回Bool)、F-824-R1-02(P1,恢复丢失字段/分支类型并漏空容器名义依赖)、F-824-R1-03(P1,721B合法ABC展开262143类型节点约20MiB却Success/steps8/peak160B)；另test target unused import警告 | evidence:独立可重放probe+raw；现有semantics20/20、incremental5/5仍绿，说明未覆盖反例；SD-01..07冻结副本/hash已存，canonical Spec/ledger未改 | tasks:重开T-03/T-04/T-05/T-06/T-10/T-11及依赖证据，current_step=3/11，T-01/T-02有界首例/T-09现有增量证据保留；历史✅只是旧收据不再表示完成 | pending_closeout_gates:merge-owned裸cargo t+tv+新workspace档，tf仅主检出批量；823编号冲突仍须实际集成重核 | next:work同树修齐三项并提交，再受影响面复审；本轮未修代码/未合入。下方旧work全绿结论已被本记录限定，不得据以进入merge。

2026-10-27 stage:work | PLAN-824 r7(合同 r2/r3/r4) | outcome:pass(work-owned 全部完成:T-03/T-04/T-05/T-09/T-11/T-06；不送最终 review——T-07 由独立上下文执行) | worktree:D:/autostack/.wt/lang-824/auto-lang | branch:plan-824-dev | base:efea15382(T-02 recipe 链) | code:30c1532de(T-03)→df502f597(T-04)→488415eb9(T-05)→c498b8e51(T-09)→c9e8754dc(T-11)→209367573(T-06) | deps:D822✅/D828✅/D821✅/D852✅(archived,早于基面)；核心窗口本树持续持有(合同 §8) | tasks:T-03、T-04、T-05、T-09、T-11、T-06 | evidence:acceptance 10 段全 PASS(first-case 4/semantics 20+同源 harness 24/admission 1/resources 11 含 worker 128MiB 父侧采样/cache 9/compatibility 821+822+798/820+817+818 全族/incremental 5 含 512-churn/baseline 6 入口/autodown 3 含 V-10 take-probe 轴)+fmt/check 零警告；[852 binding](attachments/824/852-validation/binding.md)+[收据](attachments/824/852-validation/raw/baseline-receipts.md) | 修复:T-02 核心窗口遗漏 ac-mir-native 穷尽 match(818 20红→24/24 绿)；cache_compat 3 处 unused-mut 清偿 | blockers:无 work-owned 阻塞;①id5/AVAD v4/opcode 0x2F..31 与 823 树(plan-823-dev)并行登记冲突已核实(两家 T-02 同窗 2 分钟内完成)——merge 时后合者按共同审计"最后集成者重核消费者并集"统一改号,机制集中常量+冻结表,改号机械;②T-07 独立复核待启动,merge-owned 裸 cargo t+tv+新 workspace 档全程 pending | next:T-07 独立最终 review(同树冻结 209367573)

2026-10-27 stage:work | PLAN-824 r7(合同 r2/r3) | outcome:pass(仅 T-02 集成门范围——核心 av.dynamic/1 全管线+probe 首例+recipe 链;V-02..07 反控/资源采样档未做,归 T-03/T-04;不送最终 review) | worktree:D:/autostack/.wt/lang-824/auto-lang(组内含 auto-ui/auto-down detached 依赖树) | branch:plan-824-dev | base:master c6c5ddb25(merge b7837c63d) | code:e23c2b628(模块r2)→645378ab5(核心窗口)→ff120762c(probe首例)→9882277dd(recipe链) | deps:D822=archived(合并收据齐全)已消费其 List/Variant 面 | tasks:T-02(+合同 r2/r3 刷新) | evidence:av_dynamic_824_first_case 4/4、av-data-824 语料 3/3(corpus/negative oracle/worker faults)、verify-av-dynamic-824.ps1 guard/first-case/autodown/acceptance 全绿、822 首例+语义 4/4+59/59 与 core 31/31 与 harness 24/24 回归绿;身份冲突回记(id 4/v3 被 collections 占用→id 5/v4) | blockers:无(下一任务 T-03);核心窗口仍由本计划持有(合同§8) | next:T-03 共享核心全部支持语义(V-02..07 正反控)+T-04 拒绝/回滚/资源 hook;merge-owned 整档门禁全程 pending | 关键裁定(合同§13):DYN_PACK v1=显式克隆共享、DYN_TAKE 失败=av.trap.dyn-type 源保持、entry result 可增 Dyn、A23/A24 代码与 portable 分域

2026-10-26 stage:new | PLAN-824 r6→r7 | outcome:pass（验证phase合同可交work，非实现/独立最终review） | authority:用户确认先852底座、再接本范围验证 | changed:T-11/AC-11/V-15/SD-07、T-09/T-06/T-07无环依赖；total10→11，原完成项/current_step/状态保留 | 原模块与旧review证据保持历史，新验收尚pending；若代码/依赖/输入变化重验受影响项 | next:原D门/窗口下继续已有work，D852后做T-11 | evidence:[共同合同](reports/ace-852-baseline-and-capability-validation-2026-10-26.md)、[修订预检](attachments/852-phase-integration-20261026/preflight.json)。本轮不启动实施。

2026-10-26 stage:work | PLAN-824 r6 | outcome:pass(仅 T-01 合同冻结 + T-10 独立模块门范围;非 Plan 交付、非 D 门解锁、不送最终 review) | worktree:D:/autostack/.wt/lang-824/auto-lang | branch:plan-824-dev | base:7513bcee0 | code:29461c706 | deps:D821=archived(92b8daf60)、D828=archived、D822=executing 4/9(未消费,含 needs_fix 修复在途) | tasks:T-01、T-10 | evidence:[contract-decision.md](attachments/824/contract-decision.md)(§1 基面核对/§7 预算/§9 V-14 映射/§11 r1 实施修订);`cargo test --manifest-path experimental/ac-av-probe/parallel/824/Cargo.toml`=22 passed/0 failed/0 ignored(selected>0),零 warning,rustfmt 过;`cargo check -p auto-av-core` 证明公共构建面零改动;work tree clean | blockers:T-02 集成门等 D822(独立最终review+冻结门禁+实际合入+Spec/merge收据)与核心/注册窗口接受 | next:D822 满足后由 work 刷新 T-01 合同基面/接口/预算再进 T-02;T-03..06/T-09 依序;最终 review(T-07)独立上下文。merge-owned 整档门禁(裸 cargo t+tv+新workspace档)全程 pending,不由本 handoff 触发

2026-10-26 stage:new | PLAN-824 r5→r6 | outcome:pass（依赖/并行合同可交work，非实现通过或D门自动解锁） | authority:用户明确要求实际依赖审计与独立模块并行 | changed:§4–8两阶段门、T-10/AC-10/V-14、SD-01和当前入口；total9→10，原0进度/drafting及预算保持 | next:T-01后依D821/D828和私有写面登记做T-10，T-02仍等D822/D828与核心窗；不代启实施 | evidence:[审计](reports/ace-parallel-dependency-audit-2026-10-26.md)、[预检](attachments/ace-parallel-20261026/preflight.md)。

2026-10-21 stage:new | PLAN-824 r4→r5 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-824 r3→r4 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增动态类型与可移植授权的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-824 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-823及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..05/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

2026-10-27 T-07 R1 needs_fix 修复轮已完成：三项 P1 + unused import 修齐(修复提交 6a68240e8,闭合表 attachments/824/review-r1/repair-closure.md),重开任务全部翻绿,executing 9/11。待办：T-07 受影响面独立复审(冻结 6a68240e8)；compatibility 时序门安静机器复跑；T-08 merge。R1 报告原文如下(历史)。


当前r7已获得用户对新增验证phase的授权，无需再次确认方向。原实现按原D门/窗口推进；T-11及最终汇总/送审须D852，当前没有新验证执行证据。未实现source/native/JIT等仅保留M1 owner/gap，不扩大本计划。任务总数11，current_step沿原完成项。

2026-10-27:D822 已合并,T-02 集成门完成(3/11)。**同日后续会话完成 T-03/T-04/T-05/T-09/T-11/T-06(9/11,executing→work-owned 全毕)**:acceptance 全绿、852 D01..D09 翻绿、818 mn 穷尽 match 遗漏已修。剩余 T-07(独立 review)/T-08(merge)。

已知裁定/边界(合同 r4 在案):宽标量常量为 data-profile 专属,dynamic 经 entry-param 缝(§5.1.1 白名单类型仍全支持);Fn/Place/Dyn 容器嵌套在类型门(list-elem/variant-payload/record-field-type/field-borrow/aggregate-type)封闭,直接槽位由 dyn 白名单拒;id5/AVAD v4/opcode 0x2F..31 与 823 并行登记冲突归 merge 统一裁定(后合者改号,机械)。

2026-10-26 实施授权已落地:T-01 合同冻结完成,T-10 独立模块门完成(2/10,executing);无公共写面登记在案(§8 写面登记)。当前阻塞唯一:T-02 正式首例与集成须 D822/D828 及核心/注册窗口——D822 截至 2026-10-26 为 executing 4/9(execution_done 送审在途),满足后 work 先刷新 T-01 合同基面/接口/预算再进 T-02。

Fn/闭包/borrow/resource仍递归拒绝。823未交付时V-12用已交付Fn及未知/保留tag拒绝证明封闭性，不制造Closure类型；823先交付则本计划必须测真实嵌套Closure，824先交付则823 T-04/T-05承担该新类别兼容控制。新borrow_as/typed host访问的store/type/生命周期由本计划T-01冻结并实施，不继承821未交付的runtime保证。 若实际接口无法兑现，应在T-10最小接缝阶段给具体输入/符号/预算反例并回new，不静默复制WIP、弱化验证/资源门或退回第二实现。当前所有任务未实施，不因审计/模块准备改变状态或宣布Plan完成；旧§9串行排期为历史。
