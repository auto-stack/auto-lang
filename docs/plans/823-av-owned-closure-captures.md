---
plan_id: PLAN-823
status: executing
feature_name: AV类型化闭包环境与捕获生命周期
plan_revision: 6
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-26
current_step: 1
total_steps: 10
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-av/closure-environments.md
  - docs/specs/auto-mir/closure-capture-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-823] AV类型化闭包环境与捕获生命周期

## 0. 变更摘要

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付typed owned环境、逃逸捕获与frame/env身份。本文件是实施草案，当前仅授权起草，未实施；不得将计划支持面写成现状Spec或自动启动work。

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

2026-10-26用户授权审计/修订823–826并行边界，本轮仅改计划与战略，不启动实现或其他会话。取材初始821 archived r6、822 executing 6/9；期间822 R1 needs_fix重开至4/9，两次均非D822（他方进度原样保留）；实际输入/代码接缝与指纹见[依赖审计](reports/ace-parallel-dependency-audit-2026-10-26.md)及[inputs](attachments/ace-parallel-20261026/inputs.json)。旧“整计划串行/仅只读准备”排期按下文两阶段合同更新，原目标/非目标/预算和已有进度保留。

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-822**=上游独立最终review、冻结门禁、合入、Spec/merge收据齐全，不消费WIP。保留D822：完整首例包含List<record>捕获，需真实822类型/集合/资源合同；不等824整计划。 T-01只读准备可先；T-10在D821/D828及私有写面登记后可做真实独立模块开发，不占公共核心窗口。实际全路径构建T-02起遵守本阶段D门/窗口与原授权。具体规则见§5并行合同。

拟唯一实施树 `D:/autostack/.wt/lang-823/auto-lang`，分支 `plan-823-dev`；本轮未创建。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

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

保留D-798冻结的标量编辑负载和采样口径，定向重放其请求到当前结果P95<1000ms；冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。性能/峰值测量本机单实例串行，负载状态入证据；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

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

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 并行模块与集成两阶段合同（2026-10-26）

保留D822：完整首例包含List<record>捕获，需真实822类型/集合/资源合同；不等824整计划。

**T-01→T-10独立模块门**：依D821/D828及已合入公开接口开展821已交付Str/record的owned捕获准备/提交、环境载荷与清理；失败分配/clone不消费待move源，不实现第二套间接调用器。单一生产源为`core/src/closures.rs`（拟新增），测试壳拟为`experimental/ac-av-probe/parallel/823/`独立workspace/lock，仅用同树已合入auto-av-core。T-01冻结可达符号、具体路径/owner、支持子集和预算；T-10先证明最小可编译接缝，再实现真实模块与故障oracle。可由harness重导出已交付core公开模块并路径编译同一源，生产阶段只加薄adapter；不复制值/IR/store/VM，不改core/lib.rs/父members，不产生依赖环。若需要pub(crate)私有符号，不绕过权限，登记具体缺口并申请独立验收合入的小接缝；不能从上游WIP取代码。

**T-02集成门**：T-10通过不等于profile/Plan交付。满足原D门后同步最新master，刷新接口/版本/预算，取得共享核心/注册窗口，完成原首例真实verify→ABC→AV/store/new-worker/AutoDown和trace/资源；继续T-03..06/T-09及最终review。所有模块独立通过不能代替原全路径、精确增量、P95/内存或反控。T-10测试命令拟为`cargo test --manifest-path experimental/ac-av-probe/parallel/823/Cargo.toml`（入口由T-10建立，当前未运行，selected>0），正式脚本必须汇总新V-14，不是独立壳绿就全计划绿。

**写面与交接**：本计划只写上述私有新增源、parallel/823及自身fixture/报告；现有mir/contract/ownership/place/layout/instr_spec、codec/verifier、av派发/装载/snapshot、公共lib/manifest和共用driver/recipe注册默认一个核心窗口。基面/版本/文件符号、前后owner、兼容证据及接受/释放需记录；公共tag/opcode/version由核心owner统一登记。复审期间其他Plan可以继续无公共写面的T-10，不能拿未审公共定义开工或持窗等上游。最后集成者重核已交付消费者并集，复审独立、整档仍merge-owned，测量单实例。详见[共同审计/窗口表](reports/ace-parallel-dependency-audit-2026-10-26.md)。

新borrow capture的frame/env/store保护必须本计划定义和实现；821 r6没有运行期失效句柄/旧帧/跨store校验，不得当既有保证。若824先交付，T-04/T-05在加入真实Closure类别时负责嵌套pack/freeze拒绝及824合法值正控；若824后交付，记录类型/拒绝接缝交其验收。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/closure-environments.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/closure-capture-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-823独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

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
| V-13 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

### 并行同源与依赖边界验收（新增work-owned）

| Case ID | AC / owner | 输入与真实入口 | 区分性oracle | 覆盖边界 |
|---|---|---|---|---|
| V-14 | AC-10；T-10/T-03/T-05 | 同一拟新增生产源由parallel/823构建并在核心窗口后由真实生产adapter消费；坏输入/失败分配/版本或窗口负控 | 真实StrArena/Value上的构造取消、故障clone前源仍可读、成功转移后env仅释放一次；生产接线使用同一closures源和真实AV，若824已交付新增Closure嵌套拒绝同时成立；同源hash/函数映射一致，越写公共面、WIP依赖或适用必需组合缺例不得就绪 | T-10只证明已交付子集模块；T-02..06补正式profile/cache/AutoDown，未知/未交付类别诚实拒绝，不冒领全路径 |

## 7. 验收标准

| ID | owner_stage | 可观察完成标准与方法 |
|---|---|---|
| AC-01 | work | T-02真实首例经source支持面/typed fixture→verify→ABC→AV，内容/位型/来源和独立oracle一致，V-01/10通过 |
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（本profile含owned值）；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应环境所有权与调用合同规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 ；核AC-10及V-14的真实模块/集成边界 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |
| AC-10 | work | V-14的T-10独立同源模块与T-03/T-05真实集成证据齐全，D门/公共写面接受释放/版本与消费者矩阵可核；不以模块绿替代原全路径，适用拒绝/失败控制与原增量/资源门不减 |

## 8. 执行步骤

所有任务初始未完成；current_step仅计实际完成任务，不提前勾review/merge。T-01在本轮亦未执行实施调查，仅起草取材已完成。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施授权；只读准备可先 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/823/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜；冻结§5私有同源模块/harness可编译接缝、T-10子集与集成门、实际公开API和窗口登记 | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总。[✅ 已完成 2026-10-26] [contract-decision](attachments/823/contract-decision.md)：D821/D828 archived 满足、D822 executing 未满足（T-02 阻塞如实登记）；av.closures/1 命名冻结、wire/opcode/recipe 编号仅请求不占号；T-10 API/预算（MAX_ENV_FIELDS=1024、8MiB 默认）/九用例 oracle/cap 两侧输入/同源接缝（parallel/823 + #[path] + re-export）/av.rs clone_owned_value 私有缝收敛裁定（T-03 av.rs CLONE 改接 deep_clone_value 单源）全部冻结；核心公开 API @ ff7547f9d 逐文件 SHA-256 指纹入档 |
| [ ] T-10 | work | T-01、D821、D828、独立私有写面登记 | 新`core/src/closures.rs`与parallel/823独立harness/lock；最小接缝先编译，再完成821已交付Str/record的owned捕获准备/提交、环境载荷与清理；失败分配/clone不消费待move源，不实现第二套间接调用器；不改公共装配 | V-14模块子集；AC-10；未建入口/0例不能绿，完整case待T-03/T-05生产同源证明 |
| [ ] T-02 | work | T-01/T-10、D822、D828、核心/注册窗口接受 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记；只消费T-10同源模块，不另实现；刷新已交付基面/版本、记录接受 | AC-01/06；first-case、V-01/08/10正控 |
| [ ] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile；生产直接消费T-10同源，完成适用V-14集成轴 | AC-02/03；av_closures_823定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面；按§5核已交付消费者并集/类别拒绝及同源，完成V-14适用轴 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据；正式脚本汇总V-14/AC-10，无模块通过冒领集成或缺适用必需例 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成、T-10及AC-10完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论；独立核依赖/同源与窗口并集及V-14 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录

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

用户已授权本次依赖/并行合同修订，无需重新批准同范围方向。当前先T-01冻结真实公开API/独立harness最小接缝与适用矩阵，再在D821/D828及私有写面登记后做T-10；T-02正式首例与集成须D822/D828及核心/注册窗口。保留D822：完整首例包含List<record>捕获，需真实822类型/集合/资源合同；不等824整计划。

新borrow capture的frame/env/store保护必须本计划定义和实现；821 r6没有运行期失效句柄/旧帧/跨store校验，不得当既有保证。若824先交付，T-04/T-05在加入真实Closure类别时负责嵌套pack/freeze拒绝及824合法值正控；若824后交付，记录类型/拒绝接缝交其验收。 若实际接口无法兑现，应在T-10最小接缝阶段给具体输入/符号/预算反例并回new，不静默复制WIP、弱化验证/资源门或退回第二实现。当前所有任务未实施，不因审计/模块准备改变状态或宣布Plan完成；旧§9串行排期为历史。
