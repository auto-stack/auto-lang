---
plan_id: PLAN-820
status: executing
feature_name: AV类型化数据表示与布局首个闭环
plan_revision: 6
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-25
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-hir/stage-contract.md
new_spec_components:
  - docs/specs/auto-av/data-layout.md
  - docs/specs/auto-mir/data-place-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-820] AV类型化数据表示与布局首个闭环

## 0. 变更摘要

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付宽标量、纯值record、字段Place与布局。本文件是实施草案，当前仅授权起草，未实施；不得将计划支持面写成现状Spec或自动启动work。

## 1. 目标

扩展已交付的798共享核心，交付独立data profile：i32/bool、i64/u64/f32/f64的数据常量/存取/参数返回（宽标量不含新算术/源码语法）、纯值非递归嵌套record、字段Place和执行布局。对已有record源码/Checked HIR建立真实adapter闭环；新宽标量用显式typed MIR fixture，不伪称Auto源码支持。

成功标志：真实受检MIR→ABC→AV在新profile下执行，正例独立oracle一致、坏输入在消费前拒绝、资源/版本/恢复可验证，旧core-i32及实际已合入的817/818标量路径兼容。

非目标：资源/Str/借用/Drop、List/enum/闭包/Dyn、新宽标量算术、用户泛型、生产默认接线、native新数据lowering、CTE扩面、NaN-boxing及unsafe union。

范围只在auto-lang仓的共享实验核心、独立fixtures/recipe/本计划脚本/证据；根workspace成员/产品manifest、旧VM、UI/Man/SDK及他方在途实现不改；仅允许本recipe在当前真实owner处追加最小注册/父侧判定薄接线，并按实际根Rust触面验证。共享注册位置服从814–816实际迁移后的owner，不复制协议/执行器。

## 2. 架构方案

```text
支持的已有Checked HIR / 明确未信任typed fixture
  → av.data/1 MIR + verify
  → ABC encode → decode/verify → LoadedModule
  → AV typed access / layout / lifecycle → typed result/diagnostic
             ↕ 783 managed store（代码与portable载荷分域）
   新worker恢复重验 + AutoDown父侧独立oracle
```

执行profile拟为 `av.data/1`，名字/编号由T-01确认不冲突后冻结；不复用core-i32授信。纯语义TypeRef与AV LayoutRef、native target ABI分离；不再创建VM私有推导器或第二MIR。

## 3. 技术栈

Rust、D依赖交付的 `experimental/ac-av-probe/` 独立workspace（已观察package为auto-av-core/auto-av-probe，work须重核），783 ManagedStore、796语义事实、现有probe/run/contained worker。

影响入口：既有 `core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs`、`probe/src/hir_adapter.rs`（路径以D-798为准）；拟新增 `core/src/layout.rs` 和本计划tests/fixtures；旧HIR/source/native只读。

拟新增 `scripts/verify-av-data-820.ps1`，本计划测试前缀 `av_data_820`、`test/cases/av-data/820/`、独立recipe `av-data-820`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-798**=798共享核心实际交付，且独立最终review、冻结门禁、合入、Spec与merge收据齐全；实际启动重验提交祖先/Spec，不只看frontmatter。T-01只读准备可先做，T-02及依赖构建等D和用户执行授权。797不是本计划前置；817/818共享API消费映射按Design44 §12单写交接，不能修改其在途合同。若仍在改公共定义，先等owner交接/实际合入，不从其分支抄代码开工。

拟唯一实施树 `D:/autostack/.wt/lang-820/auto-lang`，分支 `plan-820-dev`；本轮未创建。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

## 5. 详细设计

### 5.1 当前执行合同

1. 类型身份沿796/既有HIR，ExecTypeIdx、LayoutRef、FieldRef分域；同名不同owner、同宽不同名义record不可互换。runtime不复制推导器。
2. 首版安全enum/聚合存储；描述符给出逻辑size/align/字段位置与Copy分类。物理存储可不同于描述符offset，不允许reinterpret Rust对象。Copy record保值语义，所有字段读写定位原根。
3. ABC新增profile独立版本/编码，旧core-i32语义和启动时已交付版本路由/拒绝矩阵保持，826已批准迁移按新版本政策消费而不重解释旧字节；descriptor恢复重新计算并校验，不信磁盘offset或trusted。按值递归、checked layout溢出、未初始化字段读和错字段归属在执行前拒绝。
4. 保留精确标量位型。bool只有false/true，char尚归821；i64/u64全宽不按48位截断，f32/f64保±0、±∞、NaN payload。新宽标量仅const/copy/call/return，未知运算拒绝。
5. 字段构造按源eval_args顺序再映射字段；copy赋值重叠时先完整物化RHS，out返回与实参不重叠。边界/source能力表逐项区分，禁止新增类型使818/817默认支持。
6. 新data profile单独准入typed entry/result及参数类型，宽标量可经其入口返回并逐位核验；旧core-i32零参/i32 entry和旧ExecOutcome消费语义不放宽。若公共Rust定义扩展导致已交付消费者需要编译兼容，只在owner交接后做最小版本化适配，并重放旧profile。

### 5.2 有界首例与资源政策

T-01冻结支持/拒绝、类型/指令/wire版本、生命周期、输入前门及计量决策，不在下游实现中临时猜规则。T-02必须完成：已有record源码→既有对应域Checked HIR→共享MIR→ABC→AV，嵌套字段写后返回42；同输入旧HIR-native与手审expected一致。另作u64::MAX及f64负NaN的typed MIR→ABC→AV位型往返。首例同store持久化、新worker恢复重验和正式AutoDown实际父侧判定均可达。

首例不可行则记录实际输入、拒绝/资源原因和合同冲突，回new显式revision；不能改用旧VM、弱化verifier、去掉oracle或只留纯模型。未知语法/接口的fixture是未信任typed输入，真verify授信后执行，不把未落地接口描述当证据。

沿D依赖已冻结输入≤4MiB、module/每函数表≤65536、验证≤200万步/5s、执行≤100万步/1024帧/2s的**适用上限交集**；旧profile更小cap仍生效，单位/计步在T-01核对，不按本段放宽旧合同。新增类型描述、路径、图、construct/drop/freeze必须申请预算并使用有界显式遍历。值数据合计≤8MiB（槽+聚合+buffer+环境+Dyn），受管总32MiB（hot8/staging16/execution8），不是各类各领8MiB；allocator/库暂存按真实峰值报告。新worker private-bytes相对空载增量沿798上限128MiB，包含本次allocator/解码/临时对象；父侧采样瞬态峰值，与自有计数共同判定。native工具进程另列归因，不从worker实际峰值扣掉前端。

这些是拟议实验资源政策，非用户给定或永久语言限制；T-01如不能成立须带边界输入回规划，不能在work直接提高。OOM/预算/cancel/infra与invalid/unsupported/trap分态，分配前拒绝；cleanup预留记账不受正常fuel耗尽取消，不暗加语法深度限制。

### 5.3 存储、注册与兼容

保留D-798冻结的标量编辑负载和采样口径，定向重放其请求到当前结果P95<1000ms；冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。性能/峰值测量本机单实例串行，负载状态入证据；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

首例沿783真实代码producer、root/lease/GC，cache恢复重走MIR/ABC门；类型/字段集/layout/profile/语义版本入依赖，坏内层即使好hash也不得执行。活帧/borrow/宿主地址不存cache。首例独立recipe注册和父侧expected比较是T-02交付，不允许“有fixture但execution pending”结项。

共享注册入口现观察为 `crates/auto-lang/src/test_cases/{sema,mod,stale}.rs`（若已迁移以816交付为准）。只追加自己的recipe/身份和独立worker处理，不复制整个sema worker，不修改v1/v2解码或他方recipe合同。若协议不足，在T-01由实际owner决定最小版本化增量，依赖实施暂停。

旧标量语料与实际已合入817/818路径必须重放：仅保证原profile兼容及未知新profile明确拒绝；不强制其他后端实现新数据。已有record域用旧HIR→native和固定expected；宽标量仅数据存取用手审bits oracle，不强行让818支持新类型。

### 5.4 ABC共享合同与本计划责任（r2）

依据[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44 §15](../design/44-av-data-layout-place-and-ownership.md#15-abc补充合同与逐profile验收r2)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)，本计划承担**类型等价与执行表示**：primitive重复条目统一归一化或拒绝，Const/Copy/Bin/Call/Return都走同一等价规则；Fn比较参数/返回及模式/效果，record按owner/声明revision，布局同宽不能互换。Bool比较与I32比较语义在表中明确列出，不借I32命名隐式扩大类型；建立InstructionSpec合同表，逐指令记录operand域/角色、类型、read/def/kill、借用/owner、trap/cleanup、逻辑fuel、来源和profile，建立codec/verifier/loader/executor覆盖检查，不强制DSL。

语义MIR、portable ABC和私有LoadedProgram分层，不另建语义IR。825不是本计划硬依赖：其已合入时适配受检执行表示，未交付时允许同一私有接口的安全参考路径；未知profile不进无验证fallback。kind/entry/imports及歧义旧缓存策略消费D-798修复，不猜函数数。owner、活分支和Drop/必要类型是语义，strip debug不得移除。

类型/路径分析、clone/drop/编解码及cleanup计入既有合计预算；输入要求的capabilities与当前host grants分开，优化不绕过授权、逻辑fuel和取消。帧arena、slot复用、融合及紧凑wire属825/826或另案性能，不以其完成替代本计划正确性。T-01冻结V-11具体fixture/反控和各层合同映射，T-03/04实现，T-05覆盖恢复，T-06汇总；不得只放设计注释而漏验收。

### 5.5 文档复审补充合同（r3）

Copy/needs_drop及布局从受检类型图推导，输入自称Copy/无drop/任意offset不能授信。源码空record按既有source-record前门拒绝；首批typed空布局不作为已支持能力，若另提支持先在T-01明确零size/stride规则并显式revision。类型表/各函数预算是族合计，不是每一类独享65536。

准入错误在decode/verify/load门拒绝、dispatch=0；已准入程序的越界、失效句柄/guard、错型take、取消等允许此前合法执行，须断言危险目标访问/提交/目标callee计数0及原值/清理，不要求整次dispatch=0。prepare/commit只保护尚未提交的转移；已发生的guest求值/调用效果不回滚。

消费前冻结已合入(profile, wire/schema, semantic-version, producer)矩阵；825/826可前置但不是本计划硬依赖，820指令表尚未交付时它们沿D798现有合同，不反向等待本计划。当前core/注册窗口只能在依赖满足后持有，不持锁等其他owner。注册的最小接线是范围内例外，不等于修改旧VM或产品默认；814–816只按实际已迁移owner交接。

资源表沿798精确族口径：模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536；index上限还须小于对应表长及总额，不能为取最大索引挤掉必要块/指令。32MiB分区和128MiB进程增量分别验收，不扣前端/allocator/暂存，已有约211MiB前端证据不能记债后当D798已达。T-01核当前D门及同口径证据，冲突先回规划，不擅自豁免或提高数字。

T-01在contract-decision冻结case→子case→Scope→真实入口→阶段/码→oracle→必需/适用矩阵，包含V-12；T-02使first-case和recipe正控可执行，T-03/04实现与反控，T-05核恢复/兼容，T-06验必需集合无漏例/重复冒计/skip。合法特定profile外输入应按合同拒绝，不能误算成适用例未执行而跳过。

### 全链路增量兼容与828优先排期（r4）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：类型/布局按实际读取事实建立依赖；同布局body编辑不重建其他函数/全类型表，布局变化精确失效；scalar完整链保留。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 执行排期与单写交接（2026-10-21）

优先功能线起点；D798和D828满足后可申请核心窗口，不以D817/D818为整计划硬前置。公共定义仍有在途owner时等待该面交接。随后821→822→823→824按实际交付串行；本计划不等825/826。

所有后继在D828前仅作只读准备，现有实现与证据保留。共用[执行顺序与交接合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)：T-01准备写面/owner/消费者映射，T-02在D门满足后刷新基面、版本、预算与接口并记录窗口接受；既有兼容/coverage工作记录定向结果，独立review核记录，merge补实际合入与窗口释放。公共核心默认单写，公共注册追加也串行；不同worktree不豁免。不能持窗等未交付依赖、复制WIP或改变他方在途合同；无公共写面者可并行消费稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/data-layout.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/data-place-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 798共享核心实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 798共享核心实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-820独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-hir/stage-contract.md | 已有HIR域语义保持→追加其到av.data/1 MIR/ABC的实际消费支持/拒绝/资源责任 | 不将新后端能力反授旧域 | AC-01/02/04/06 |
| SD-07 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

不变量为Design44 I-01..12及§5。以下V均为**必需补充反例/正控**，T-02/T-04将其变为真实入口；固定语料总数不能替代。每案保存输入身份、阶段/码、result/trace/bits/owner与原始日志。期望由人工或独立小模型确定，不能由被测布局/清理算法生成。

| Case ID | AC / work owner | 输入/状态 | 实际或拟入口 | 可观察oracle与区分断言 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02；T-02 | 同宽不同owner record与互换FieldRef；合法相同owner对照 | 新profile verify/ABC/AV，av_data_820 family | 错误在verify拒绝、av_execute=0；合法值42 | 布局/身份；仅声明支持面 |
| V-02 | AC-02/03；T-03/04 | 嵌套p.inner.x写；b=copy(a)后改b；RHS读取目的根 | 新profile verify/ABC/AV，av_data_820 family | 原根命中、a不变；RHS旧值先物化，结果/trace固定 | 原位/重叠；仅声明支持面 |
| V-03 | AC-03/05；T-03/04 | MIN_I64/MAX_U64、±0/±∞、正负NaN payload | 新profile verify/ABC/AV，av_data_820 family | store/ABC/call/return逐位相等，不误判handle | 精确宽度；仅声明支持面 |
| V-04 | AC-04；T-03/04 | 按值递归、size/align/offset溢出、错型字段/非法bool | 新profile verify/ABC/AV，av_data_820 family | 受控invalid/unsupported，av_execute=0；非法layout不发布，无panic | 坏布局；仅声明支持面 |
| V-05 | AC-02/04；T-03/04 | 构造缺字段/重复字段、分支仅一臂初始化、零次循环 | 新profile verify/ABC/AV，av_data_820 family | 未初始化读取前拒绝；完整合法构造接受 | 初始化；仅声明支持面 |
| V-06 | AC-02/03；T-03/04 | 字段声明序与源码求值序不同、嵌套调用/未走trap | 新profile verify/ABC/AV，av_data_820 family | trace沿eval_args，未走trap保持潜在行为 | 顺序；仅声明支持面 |
| V-07 | AC-05；T-04 | 合法输入cap内/超1单位、瞬态峰值、故障第n次 | 新worker+父侧budget/allocator hook | resource分态；预算前拒的该申请attempt=0，故障注入attempt存在但successful_alloc=0；先前合法事件保留，危险访问/提交0；cleanup/真实峰值断言触发，无panic | 不能只看终态RSS |
| V-08 | AC-06；T-02/05 | fresh、同store新worker重启、改type/layout/profile、好hash坏payload | 783真实恢复→verify→AV | 合法恢复结果相同；依赖变化不复用；非法执行计数0 | 不缓存PASS或trusted |
| V-09 | AC-06；T-05 | core-i32及实际交付817/818旧profile、新profile送旧consumer | 实际原recipe/consumer | 旧结果/诊断不变，未知profile明确拒绝 | 无假native/CTE支持 |
| V-10 | AC-01/04/06；T-06 | worker错结果/错bits、漏case/skip、错工具身份 | 正式AutoDown recipe+父侧oracle | 合法正控PASS；每项错实际使总体非零/ERROR | 缺必要工具/路径不可绿 |
| V-11 | AC-02/03/04/05/06；T-01/03/04/05/06 | 重复primitive索引的const/copy/call/return、同宽异名record/错owner、缺失指令合同项、kind/entry被篡改；合法别名索引按冻结政策统一判定 | §3真实MIR/ABC授信、执行及store新worker恢复；T-01冻结输入，T-03/04使可运行 | 类型判定在全部消费点一致；错名义类型执行前拒绝且dispatch=0；合同漏项使覆盖检查失败；好hash不得绕过；准入非法dispatch=0，运行期故障按§5.5断言目标危险操作0 | 仅本profile；标量/旧消费者仍V-09，受管/进程峰值沿V-07 |
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | 同布局不同owner与伪Copy/drop/layout描述符；合法record构造/拷贝；源码空record；模块/函数表族合计cap两侧 | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | 伪造字段不能获得复制/布局权限，非法输入授信前拒绝，合法派生描述符结果42；源码空record保持既有拒绝；资源超额不是类型错误；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_data_820`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-data-820.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
- fmt/check按该独立manifest执行；开发与修复只定向测试。禁止work/internal/final review触发整档t/tv/tt/tb/tu/taa。
- 合入：最终review后冻结HEAD，由merge执行裸`cargo t`+VM/编译器触面`cargo tv`，以及新workspace完整有限合同档；按实际diff再选其他触面，tf只主检出到期单实例。review pass必须记录merge门禁pending。

角色：用户未指定模型或分离实现者。work负责实现和反例；内部检查只表示可送审；最终review必须独立上下文、同树只读核验，不修代码；merge负责冻结门禁/规范/归档/guard清树。当前未指派或启动任何角色/代理。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-13 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

## 7. 验收标准

| ID | owner_stage | 可观察完成标准与方法 |
|---|---|---|
| AC-01 | work | T-02真实首例经source支持面/typed fixture→verify→ABC→AV，内容/位型/来源和独立oracle一致，V-01/10通过 |
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应类型等价与执行表示规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（820只Copy无资源）；§5.4及V-11对应类型等价与执行表示规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应类型等价与执行表示规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应类型等价与执行表示规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应类型等价与执行表示规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

所有任务初始未完成；current_step仅计实际完成任务，不提前勾review/merge。T-01在本轮亦未执行实施调查，仅起草取材已完成。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户授权；只读准备可早于D-798 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/820/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜 | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 | [✅ 已完成 2026-10-24：依赖门全绿（D-798 archived/D828 archived 11/11/817 合入/818 在途非硬前置且持独立 workspace/ac-av-probe 单写窗口空闲——821-827 全 drafting 0 步/注册 owner=crates/auto-lang-core/src/test_cases/ac_av.rs 814 迁移后落位/主检出代码路径零 WIP）。contract-decision.md r1 冻结：av.data/1（profile id 2）+ AVAD v1 wire + auto.mir.av-data/1 MIR schema + producer plan820/av-data + 语义版本 1/1/1（core-i32 1/2/1 不变）；MirType 增 I64/U64/F32/F64/Record（名义身份=owner+name+fields，无环迭代检测，空 record 不支持）；MirConst 增宽标量位型（to_bits 保真，V-03 样本冻结）；新指令 07 FIELD_GET/08 FIELD_SET/09 AGGREGATE（仅 data；BIN/UN 零新子码，宽标量运算 mir.type-op 拒）+ InstructionSpec 表 + 四层覆盖检查；layout.rs 逻辑描述符（checked 布局、不 reinterpret、恢复重算）；Value 增 Record(Rc) 不可变纯值+deep_size 记账入 8MiB 槽预算；adapter 面=compile_record_source+自由函数+RecordCtor/FieldGet/FieldPath 赋值（Method/Borrow 显式拒）；预算=798 冻结值零放宽+每类型字段≤1024+路径深≤64；§12 case→Scope→入口→oracle 全矩阵（V-01..13）；§13 显式留 T-02/T-09 核实项。见 attachments/820/contract-decision.md ]
| [x] T-02 | work | T-01、D-798、注册owner交接、D828排期门、当前单写窗口交接 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记 | AC-01/06；first-case、V-01/08/10正控 | [进度 2026-10-24（进行中，未勾）：whole-path 首例+位型往返+store/新进程恢复+脚本已交付——commit 5323bf863/1a21a9752 @ plan-820-dev（树 D:/autostack/.wt/lang-820/auto-lang，基面 37e512536）：首例 record 源码→HIR→MIR→AVAD→AV 返回 42（av_data_820_first_case_whole_path）；u64::MAX/i64::MIN/-NaN payload(0xFFF8_0000_DEAD_BEEF)/-0.0 逐位往返；同 store A20 层持久化+进程内恢复+新进程 worker data-restore-run 恢复重验执行 42+错 types-fp clean miss（V-08 首刀）；scripts/verify-av-data-820.ps1 first-case PASS（10/10 选择断言）；首批反控绿（V-01 同宽异 owner mir.copy-type/V-02 copy 保原值/V-04 mir.type-cycle+mir.record-empty+mir.type-op/entry 边界/wire 族隔离+校验和）。全 probe 套件绿（含 828 全量增量族 21/21，复现并修复 fn:导入调用 dst 类型回归）+core 25/25。**剩余**：正式 AutoDown recipe（ac-av/data-run-1 注册+父侧逐位判定，公共注册写面串行）、CPU/内存系统记录（B 线实测）、native record 支持面核对 ] [✅ 已完成 2026-10-24（commits 5323bf863/1a21a9752/3b3d76a9c）：whole-path 首例 42；u64::MAX/i64::MIN/-NaN payload/-0.0 逐位往返；A20 层同 store 持久化+进程内恢复+新进程 worker data-restore-run 42+错 types-fp clean miss；正式 recipe ac-av/data-run-v1（ac_av.rs RecipeFamily/mod.rs 分派/case.ad+fixture/suite；corpus 实测绿含 data case 经 release worker+父侧判定）；B 线 25×全管线 private-bytes 增量≤128MiB 实测；native 面=scalar-only 入档合同§13；脚本 first-case PASS] [✅ R1 修复 2026-10-24（baf4d7d49）：F820-R1-03 关闭——首例 native 独立对照按 §5.2 原文恢复（同 record 源码经 worker `native-build --profile record`→link→run，退出码=42=手审 expected=data 路径 AV 结果；av_data_820_first_case_native_independent_comparison）；contract-decision §11/§13"native scalar-only"错误声明已修正（实际受限点仅 worker 子命令硬编码 scalar 前门，ac-core native 自 771 支持 record）；B 线口径改父侧采样入 T-04 证据] [✅ R6 修复 2026-10-25（278b1dce0）：F820-R6-01 关闭（typed 正式面）——av.data-fixture/1 冻结七样本 typed:<id> 入口经真实准入链，worker 报告实际产值规范 {kind,bits}，父侧独立冻结表比对 tag+精确 rawbits；record 源成功臂同附 typed 报告（R6 原复现 bool→null 形态修复：av-data-bool.case.ad 入套件 corpus 绿）；av-data-typed.case.ad 七正控入套件] [✅ R7 修复 2026-10-25（7308acad6）：F820-R7-01/02 关闭——typed 存在性按 family 隔离（core/i32 成功仍必须旧 value 通道；非 data 响应带 typed=准入门协议违规 infra 非绿；data typed 形状校验六 tag+规范宽度，畸形=typed-shape 红）；冻结 f32 NaN payload 0x7F80_AD00 入双冻结表+第八正控检查+wrong-bits 反控绑定（一位错经真实父侧判红）；judge_tests 三单测钉生产判定（含复审故障 worker 双形态）] |
| [x] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile | AC-02/03；av_data_820定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 | [✅ 已完成 2026-10-24（3b3d76a9c）：core/instr_spec.rs 12 opcode 合同表+coverage_check 门（漏项即红）+trap/data-only 面断言；adapter 支持面显式（Method/BorrowMut 拒、BorrowView 按值传参直通）；av_data_820_semantics 13/13：求值序 trace=[b,a]/swap RHS 先物化/构造缺重字段拒/未初始化读拒/-NaN 双帧存活/等价网格五消费点/重复条目归一/unit kind/全 opcode 往返] 
| [x] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 | [✅ 已完成 2026-10-24（2a270f0b4）：fault_after_allocs seam（第N次值分配 Infra 前拒、先前状态完整）；deep_size 按字段计（core 步数不变）；取消轮询 last_poll 间隔；av_data_820_resources 6/6：8MiB 内控/超限中程拒/差1字节拒/故障保状态/取消保峰值/layout 溢出文档化不可达] [✅ R1 修复 2026-10-24（baf4d7d49）：F820-R1-02 关闭——B 线重写为父侧 ChildPeakSampler 按 PID 对真实新 worker 全生命周期采样（A=同二进制空载基线，B=25 轮数据管线负载，实测增量 ~1.0MiB ≤128MiB，样本 162/470）；~64MiB 中程暂态反控实测捕获 64.1MiB（122 样本，末端采样不可见）；worker 新增 data-mem-envelope 模式，子进程不自报判定] [✅ R3 修复 2026-10-24（a789a2adc）：F820-R3-01+V-04 缺口关闭——(1) 布局推导共享验证预算：compute_layout_shared（caller 持有步数计数器+跨 record 持久 memo，链式 O(N²) 重访消除），mir_verify 用同一 Verifier.steps，LayoutError::Budget→Resource(VerifySteps) 正确分态；R3 复现 2100 链（原 2,205,050 访问超 2M 仍 accepted）现合法通过且 work_steps 远低上限，近顶预算单元反控必拒，超量模块资源拒绝；(2) V-04 真回归：62 层二叉展开 2^64 溢出在 compute_layout/verify_mir/ABC 三层受控 mir.layout-overflow（61 层 2^63 合法对照全链绿），替换 r6 错标"不可达"的 defense-only 测试。resources 6→7/7] |
| [x] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 | [✅ 已完成 2026-10-24（2a270f0b4）：av_data_820_cache_compat 5/5：类型表变化 clean miss/好hash坏载荷校验和拒/跨profile信封身份拒；scalar vertical 重放绿/record 源过旧消费者 parse 拒/旧 manifest 恢复拒 data-only store] 
| [x] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 | [✅ 已完成 2026-10-24（2a270f0b4）：脚本全 8 Scope 接线（0选防绿/跨目标合计）；V-10 反控 4/4（wrong-exit/wrong-diag/未注册 recipe/AV798_FAULT 三态=infra ERROR）；recipe 注册三处追加（白名单/step 词表/av binding）；SD 草案 attachments/820/sd-draft.md；acceptance PASS（全族+core+corpus+check）] [✅ R1 修复 2026-10-24（baf4d7d49）：F820-R1-02 接线——resources 档滤串补 av_data_820_worker_peak_sampler_catches_transient；脚本 first-case/resources Scope 实测 PASS；sd-draft SD-07 改为全链增量描述（消除"前门全量重编"与 AC-09 冲突），contract-decision §13 B 线口径同步修正] [✅ R2 修复 2026-10-24（e8497f2f8）：F820-R2-02 关闭——脚本守卫重写为必需 ID 集（每档全名清单逐 id `test <id> ... ok` 核对）、ignored>0 必红、ok 行数==passed 合计一致性、0 选必红；-GuardSelfTest 自检 2 正控+5 负控（含 R2 复审原样 1 passed/8 ignored 输入）实测 PASS；滤串档必需 ID 修正为全名（守卫对真实输出先红后修=守卫生效的实测证据）；6 Scope 全 PASS。健康门：probe 全目标零警告（Local0→local0、未用 import/参数/死 trait 清除）] [✅ R3 修复 2026-10-24（a789a2adc）：F820-R2-02 余量关闭——必需 ID 集改由同选择 cargo --list 推导（root 全限定 tests::<module>::<name> 自动适配，R3 误红消除）；新增重复 ok id/意外 ok id 守卫（重复冒计必红）；GuardSelfTest 扩 3 正+7 负控；真实正控=autodown Scope 实测 PASS（corpus 1 全限定+v10 4 id）；acceptance 各家族档全部经 --list 绑定实际完整 ID 集（R3 missing-id 控制的漏检面消除）] [✅ R4 修复 2026-10-24（affb907bd）：F820-R2-02 全关+R4-01 关闭——(1) 独立合同清单 $TIER_CONTRACT（13 档逐案必需 ID，不从被验证对象生成）：Run-Tier 于 --list 推导后逐一核对合同项在列（精确/::<name> 后缀匹配，root 全限定自动绑定、重命名=显式清单编辑），必需用例删除/cfg 排除时 listing+执行同缺也必红；GuardSelfTest 扩 5 正+7 负（合同覆盖正控+删除形态负控）；端到端 mocked-cargo 负控脚本 verify-av-data-820-missing-id-control.ps1（R4 复现形态实测 exit1 逐项点名 12 缺失合同项）；真实正控=7 Scope 全 PASS 零误红。(2) R3 新增资源测试已格式化，rustfmt --check 定向五触面文件全绿，resources 7/7 复跑零行为变化] [✅ R6 修复 2026-10-25（278b1dce0）：F820-R6-01 关闭（反控/脚本）——wrong-bits/wrong-type-tag（父侧期望破坏缝 AV820_TYPED_EXPECT_SABOTAGE，RAII 防泄漏）+unknown-fixture 三反控经正式 recipe 父侧判定，ac_av_data_probe 4→7/7；v10 合同清单同步 7；顺手修 prep_executor 故障 env 无锁并行缺陷（mass 并行 ac_av_probe 14/14 绿）；sd-draft SD-05 typed 面与 case→入口→oracle 绑定更新；autodown Scope/GuardSelfTest 实测 PASS；rustfmt 定向三触面文件绿] [✅ R7 修复 2026-10-25（7308acad6）：F820-R7-01/02 关闭——同上判定/准入门/形状校验+judge_tests 3/3+wrong-bits 扩 f32 NaN 绑定；v10 7/7（串行实测）+corpus 绿维持；mass ac_av_probe 13/14（唯一红=formal_turnaround P95 1064.5ms≥1000 环境归因：常驻负载 63%（ToDesk/Everything 等外部进程）、P50/P95 全分布上移 ~40%、该门属 798/828 标量面与本次 diff 无代码交集、不在修复脚本档；用户裁定环境问题不重试）；autodown Scope/GuardSelfTest PASS；rustfmt 定向四文件绿] |
| [x] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 | [✅ 已完成 2026-10-24（a33538088）：lower_function_data_unit+data_session（body 编辑仅重建被改函数/类型变化精确重建/跨修订签名失配 link 拒且前快照存活/强制全量=身份轴全移）；av_data_820_incremental 3/3+828 标量套件 21/21=子案1 共享基线；ac-syntax 驻留层 scalar-only 边界入档] [✅ R1 修复 2026-10-24（baf4d7d49）：F820-R1-01 关闭——data_session 重写为真实全链增量：字节分段 diff+等长注释掩码单函数解析文档（解析面=1）、保留 CheckedEnv 的 828 单体事务（recheck resolve/typecheck 单 body+签名漂移受控拒+emit_function_body 收据 1 真 body/n-1 scaffold+body_work 生产窗=未改函数零 body 工作）、结构门内建 edit_body 生产路径（未改函数 HIR Arc/代码指针+代际不变、生产/IO=0，退化即使编辑红→正式脚本档非零）；apply_source 依赖事实精确失效（record mention 嵌套闭包+跨 bundle 签名形状等价：签名内容变化→全 unit 重 lower（import 表嵌入签名）而 body 未变函数保留 HIR Arc；helper body 变化不重 lower 调用者；no-change→空集）；apply_caps 授权轴（全 unit 重 lower 而 HIR 身份不动）；双退化反控（whole 前端/只保 hash 换对象）经同一门非零；incremental 族 3/3→5/5（四子 case+旧快照存活对照+non-fast 显式拒绝）] [✅ R2 修复 2026-10-24（e8497f2f8）：F820-R2-01+F820-R1-01 余量关闭——(1) 三入口事务化（build-aside+run-before-publish）：准入/link/真实运行/结构门全成才发布，/0 trap 等失败零安装（会话全部状态保持失败前值、后续合法编辑可继续；R2 重放 after=Trap→现 42/retry 43），failed_edit_installs_nothing 覆盖 fast+apply_source 双入口；(2) 真实事件计量：zero_io 常量废弃，ProductionEvents 在真实调用点逐 def 实计 verify_mir/decode/verify_abc/admit、parse_fn_items 用解析实测值；HirIdentity 增 bundle_ptr=产出该 def HIR 的前端 bundle Arc（生产对象，非消费者克隆），apply_source 对 body 未变 def 钉原生产 bundle；(3) 注入反控：set_control_disk_read/set_control_reverify_def 经同一计量入口写入使同一门判红（读盘 4096B/未改 def 重验均红且零安装）=计量活性证据；incremental 5→7/7] [✅ R3 修复 2026-10-24（a789a2adc）：F820-R1-01 关闭——注入反控改真实操作：set_control_disk_read(path) 编辑路径真实读文件、计量入口记录实际字节数（门引用实测值）；set_control_reverify_def 对未改 def 从生产 bundle 真实 re-lower+verify_mir；同一门判红且零安装；计量/身份绑定真实 owner 维持（R2 轮）] [✅ R5 修复 2026-10-24（7c9328342）：F820-R5-01 关闭——apply_caps 增候选前端环境重建（parse+resolve_env 按新 caps，随 caps/code/snapshot 同一成功发布点安装；被用撤销在 env 解析受控拒且旧值全保；HIR 不重生）；R5 重放场景实测 grant→return mark_a() 热编辑成功、冷热同值=1、后续编辑继续可用；被用撤销 Err+会话保持；成功撤销后无权热编辑被拒零安装。incremental 7→9/9，合同清单同步；sd-draft SD-07 caps 跨层义务与统计（9/9）更新] |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录
2026-10-25 stage:work | PLAN-820 R7 修复轮 | outcome:pass（T-02/T-06 按 review-r7 两 finding 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:278b1dce0（R6 冻结基线） | code_commit:7308acad6（树 clean） | dependency_revisions:无新增 | task_ids:T-02/T-06（重开→重闭） | evidence:judge_tests 3/3（生产 judge+factored 准入规则直测：core 无 value 双形态含复审故障 worker include_typed=1 形态均非绿、data presence、typed 形状三负例+四规范正例）；v10 家族 7/7 串行实测（wrong-bits 扩 f32 NaN 绑定 wrong-bits-f32-nan Fail）；corpus 绿（第八正控 typed-f32-nan-payload）；mass ac_av_probe --test-threads=1 13/14——**唯一红=ac_av_probe_formal_turnaround_gates P95 1064.5ms≥gate 1000 环境归因**：单跑复现 P50=871.8/P95=1064.5/P99=1155.5 全分布上移，机器常驻负载 63%（ToDesk/Everything/python 等外部进程），该门属 798/828 标量 turnaround 面与本次 diff 零代码交集、不在本修复任何脚本档（autodown=corpus+v10）；当日早间同树同测绿（R6 轮 mass 153s 14/14）；用户裁定环境问题不重试。autodown Scope/GuardSelfTest 5正7负/av_data_820 13/13/workspace check/rustfmt 定向四文件全绿。**具名根因**：(1) R7-01 根因=R6 把 typed 存在性放宽（value.is_none()&&typed.is_none()）写入共享 judge_expectation 未按 RecipeFamily 隔离——family-specific 协议增量泄漏进 core 旧消费语义——修复=presence 按 family+准入门 typed_on_non_data+data typed 形状校验三层；(2) R7-02 根因=正式 fixture 集以 f32 1.0 替代了合同 §3 冻结的 NaN payload 样本——修复=0x7F80_AD00 入双冻结表+case+反控绑定。**关闭表**：F820-R7-01→judge_tests 三钉+family 隔离三层+既有 e2e 维持；F820-R7-02→第八正控+wrong-bits-f32-nan 反控。R6 typed 通道及既有 CLOSED 维持。check→finding 映射：R7-01→ac_av::judge_tests 三测+typed_on_non_data；R7-02→typed-f32-nan-payload case+wrong-bits-f32-nan。未跑整档（2026-10-14 时序裁定） | blockers:无（turnaround 红=环境归因在案，非本计划门） | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读重放 judge-replay 双形态与 f32 NaN 样本、核 family 隔离/形状校验/SD-05 约束与未审面（V-12/SD 逐目标对账），绑定 7308acad6）
2026-10-25 stage:review | review_scope:final+affected-repair+shared-judge-compatibility | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:278b1dce0e94cbe7e5e68a921e5ee569abc88314 | base_commit:7c93283423f23b52020e7e98c7bd235eeb9bd51a | dependency_revisions/spec_inputs:完整final对账pending，证据见报告 | acceptance_results:原typed通道根因已消除，7项真实父侧反控/corpus正例通过；新增F820-R7-01 P1共享judge让旧core无i32值时附data typed后从Fail变Pass；F820-R7-02 P2正式f32-one替代冻结NaN样本0x7f80ad00，原覆盖余量未闭 | evidence:[R7报告/真实父侧runner/负控worker/原始结果](attachments/820/review-r7/review.md)，v10定向7/7/corpus1/1，四触面格式及diff-check绿，HEAD/clean一致 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:T02/T06重开，executing5/9；work按family隔离结果判定并补冻结f32正式位型正反控，保留其他已闭证据后提交再审。未审V12/SD不冒领pass，未合入。
2026-10-25 stage:work | PLAN-820 R6 修复轮 | outcome:pass（T-02/T-06 按 review-r6 F820-R6-01 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:7c9328342（R5 冻结基线） | code_commit:278b1dce0（树 clean） | dependency_revisions:无新增（probe 新增 typed_fixture 模块；recipe 注册面无协议字段变更——typed:<id> 入口复用 av.entry 字符串） | task_ids:T-02/T-06（重开→重闭） | evidence:corpus 全绿（新增 av-data-typed.case.ad 七正控 + av-data-bool.case.ad=R6 原复现 bool→null 形态修复）；ac_av_data_probe 4→7/7（新增 wrong-bits/wrong-type-tag/unknown-fixture 三反控，经正式 recipe 父侧判定）；mass 并行 ac_av_probe 14/14（153s）；autodown Scope 实测 PASS（v10 合同 7 id）；GuardSelfTest 5正7负；incremental 9/9 维持；rustfmt 定向 typed_fixture/av_probe_worker/ac_av/ac_av_probe_tests 绿；双 workspace check 绿。**具名根因**：R6-01 根因=worker data 分支 Success 仅 value.as_i32() 序列化、父侧 StepOut.value: Option<i32> deny_unknown_fields——bool/宽标量结果在进父侧前即丢失，且无 typed fixture 请求分支与位型判定面——修复=冻结 av.data-fixture/1 入口（typed:<id>）+worker 规范 {kind,bits} 报告（实际产值，canonical：i64/u64/f64 十六位/f32 八位/i32/bool 不补零）+父侧独立冻结期望表逐项比对（tag+精确 rawbits）+record 源成功臂同附 typed（存在性规则认 value 或 typed）+未知 id 父侧检查错误+typed 修订哈希绑定入口身份。**关闭表**：F820-R6-01→七正控 case（corpus）+三反控（wrong-bits/wrong-tag/unknown-fixture）+bool 原形态 case；V-03 位型父侧判定=typed 七样本经正式 recipe；V-10 wrong-bits 反控=父侧期望破坏缝（RAII）；SD-05/合同清单绑定更新。R5-01 等既有 CLOSED 维持。check→finding 映射：R6-01→av-data-typed/bool.case.ad+ac_av_data_probe_wrong_bits/wrong_type_tag/unknown_typed_fixture。未跑整档（2026-10-14 时序裁定） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读重放 formal-bool-replay 与位型反控、核 V-03/V-10/SD-05 绑定与未审面（V-12 精确 cap 两侧/SD 逐目标对账），绑定 278b1dce0）
2026-10-25 stage:review | review_scope:final+affected-repair+cache/formal-result | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:7c93283423f23b52020e7e98c7bd235eeb9bd51a | base_commit:affb907bd8657d32a027453268cb209450b6c73b | dependency_revisions/spec_inputs:最终完整对账pending，Plan/SD hash见报告 | acceptance_results:F820-R5-01独立CLOSED（caps热编辑=1=冷对照、撤权反控/2测试绿）；补V08好hash有效信封但类型自环真实store恢复拒且dispatch增量0；新增F820-R6-01 P1正式data recipe只有Option<i32>，bool结果丢为success/null、无宽标量typed fixture位型父侧通道及wrong-bits正式反控，违反原contract-decision§10/V10 | evidence:[R6报告/原始输入与输出](attachments/820/review-r6/review.md)，定向格式及diff-check绿，HEAD/clean一致 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:T02/T06重开、executing5/9；保留T09及全部既有finding关闭。原树work补正式typed-result父侧oracle与端到端位型正反控/SD05绑定后提交；其余未审V/SD不冒领pass。未合入。
2026-10-24 stage:work | PLAN-820 R5 修复轮 | outcome:pass（T-09 按 review-r5 F820-R5-01 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:affb907bd（R4 冻结基线） | code_commit:7c9328342（树 clean） | dependency_revisions:无新增 | task_ids:T-09（重开→重闭） | evidence:incremental 7→9/9（新增 caps_grant_syncs_the_hot_frontend：R5 重放场景 grant→return mark_a() 热编辑成功、冷热同值=1、同一会话后续编辑继续可用；caps_revoke_is_controlled_and_gates_hot_edits：被用撤销 Err（mark_a/capability）+会话保持 1，成功撤销后无权热编辑被拒+零安装）；GuardSelfTest 5正7负/missing-id 端到端负控/first-case 真实 Scope 维持 PASS；rustfmt 定向 data_session/incremental 绿。**具名根因**：R5-01 根因=apply_caps 只替换 self.caps 并重 lower/admit/link/publish 代码面，self.env（热编辑 resolve/typecheck 所依的驻留接口目录）仍按旧授权——新增能力的接口（trace marks）不进热前端，edit_body 用旧 env 拒绝合法编辑——修复=候选 env（新 caps 下 parse+resolve_env）随同一成功发布点安装；被用撤销在 env 解析即受控拒（全旧值保留）；HIR 不重生。**关闭表**：F820-R5-01→两个新定向测试（重放场景+兄弟撤销双控）+env 同发布点安装；R2-02/R4-01 维持 R5 CLOSED。check→finding 映射：R5-01→av_data_820_v13_caps_grant_syncs_the_hot_frontend+av_data_820_v13_caps_revoke_is_controlled_and_gates_hot_edits。未跑整档（2026-10-14 时序裁定） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读核验 R5 修复（重放 caps-replay 场景）及未审面（V-08 现场篡改/V-10 全子控/V-12/SD 对账），绑定 7c9328342）
2026-10-24 stage:review | review_scope:final+affected-repair+authorization-session | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:affb907bd8657d32a027453268cb209450b6c73b | base_commit:a789a2adc1afdab97025a0c294bb6121c97aa797 | dependency_revisions/spec_inputs:完整final对账pending，Plan/SD hash见报告 | acceptance_results:R4两项F-R2-02必需集合及F-R4-01格式均独立CLOSED；新增F820-R5-01 P1授权变更只更新代码不更新self.env，新增trace授权后合法热编辑mark_a报undefined，冷对照返回1 | evidence:[R5报告/源码/原始输出](attachments/820/review-r5/review.md)；missing-id端到端控制现拒12项缺失、GuardSelfTest5正7负、定向五文件rustfmt绿、真实semantics13及cache3+restore1均绿、caps组合反例复现；HEAD/clean一致 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:只重开T09，executing6/9；保留T06完成和所有已闭finding，work修授权前端环境/代码同一事务发布及组合正反控后提交再审，完整未审AC/V/SD不冒领pass。未合入。
2026-10-24 stage:work | PLAN-820 R4 修复轮 | outcome:pass（T-06 按 review-r4 两 P2 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:a789a2adc（R3 冻结基线） | code_commit:affb907bd（树 clean） | dependency_revisions:无新增 | task_ids:T-06（重闭） | evidence:GuardSelfTest 5正7负 PASS；端到端 missing-id 负控（mocked cargo：listing+执行仅剩 1 项/12 合同项删除）verify script 实测 exit1 逐项点名；真实正控 7 Scope 全 PASS（first-case/semantics/admission/resources/cache/compatibility/autodown——含 root 全限定 autodown）；rustfmt --edition 2021 --check --config skip_children=true 定向五触面文件（layout/mir_verify/data_session/incremental/resources）全绿；resources 7/7 格式化后复跑。**具名根因**：(1) R2-02 余量根因=--list 推导的必需集由被验证对象自身生成——必需用例删除/cfg 排除时 listing 与执行同缺，门禁无从比对——修复=独立合同清单 $TIER_CONTRACT 对账 listing（oracle 不自生成）+端到端负控；(2) R4-01 根因=R3 轮资源测试编辑晚于最后一次 cargo fmt——修复=格式化+定向 rustfmt --check 证据。**关闭表**：F820-R2-02 余量→$TIER_CONTRACT 对账+missing-id-control.ps1 端到端负控+7 Scope 真实正控；F820-R4-01→rustfmt 定向五文件绿+resources 7/7 复跑。R3-01/V-04/R1-01 维持 R4 CLOSED 未回退。check→finding 映射：R2-02 余量→verify-av-data-820-missing-id-control.ps1+GuardSelfTest；R4-01→rustfmt 定向检查。未跑整档（2026-10-14 时序裁定） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读核验 R4 两项（missing-id 控制重放+定向 rustfmt）及未审面，绑定 affb907bd）
2026-10-24 stage:review | review_scope:final-readiness+affected-repair | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:a789a2adc1afdab97025a0c294bb6121c97aa797 | base_commit:e8497f2f8d275873f55b4d7c0e9dd6ac1b75e3bf | dependency_revisions/spec_inputs:final全核pending，Plan/SD hash见报告 | acceptance_results:F820-R3-01共享预算/V04真实溢出/F820-R1-01真实操作反控独立CLOSED；F820-R2-02仍partial（listing与执行同时缺必需项仍报绿），新增F820-R4-01本轮资源测试fmt失败 | evidence:[R4报告/原始控制与输出](attachments/820/review-r4/review.md)：layout定向2/2、真实操作注入1/1、GuardSelfTest3正7负；仅保留firstcase1项的listing+执行仍PASS；本轮资源测试定向rustfmt exit1，其余四个R3改动Rust文件格式绿；HEAD/clean一致 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:只重开T06，executing6/9；保留T04/T09及旧关闭证据，work修独立必需集合对账+定向格式后提交再审。未合入，未审AC/SD不冒领pass。
2026-10-24 stage:work | PLAN-820 R3 修复轮 | outcome:pass（T-04/T-06/T-09 按 review-r3 全部 finding 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:e8497f2f8（R2 冻结基线） | code_commit:a789a2adc（树 clean） | dependency_revisions:无新增 | task_ids:T-04/T-06/T-09（重闭） | evidence:resources 6→7/7（layout_budget：2100 链合法通过+近顶预算单元反控 Budget+超量资源拒绝；overflow_real：62 层三层受控拒+61 层合法对照）；incremental 7/7（真实读盘/真实重验注入）；av_data_820 13/13；semantics 13/13；cache_compat 5/5；core 22/22（layout 重构后单测绿）；脚本 7 Scope 全 PASS+GuardSelfTest 3正7负；autodown=真实 cargo 正控（全限定 ID 推导）；probe 全目标零警告。**具名根因**：(1) R3-01 根因=mir_verify 每 record 调 compute_layout 而后者每次重置 steps=0+全新 memo——布局工作既不扣共享预算又 O(N²) 重复访问祖先（2100 链 2,205,050 访问>2M 不被计数）——修复=compute_layout_shared 持久 memo 线性化+扣 caller 的 Verifier.steps 同一上限+Budget→Resource(VerifySteps) 分态；(2) V-04 缺口根因=r6 测试用"每 record ≤8KiB"错误推断宣称嵌套溢出不可达——修复=62 层二叉展开真溢出回归（2^64）+61 层（2^63）合法对照；(3) R1-01 余量根因=R2 注入仅 ev.record_disk_read(n)/bump 模拟数字非真实操作——修复=真实文件读取（实测字节数入计量）+未改 def 真实 re-lower+verify_mir；(4) R2-02 余量根因=手写必需 ID 集无 root 前缀（真实输出误红）且 acceptance 三档传空集+无重复守卫——修复=--list 推导实际完整 ID 集（全限定自动适配）+duplicate/unexpected ok id 守卫+GuardSelfTest 3正7负+真实 autodown 正控。**关闭表**：F820-R3-01→av_data_820_layout_work_shares_the_verify_budget（正控+单元反控+资源分态）；V-04 缺口→av_data_820_layout_overflow_real_input_rejected；F820-R1-01→av_data_820_v13_metering_injection_controls_redden_the_gate（真实操作）；F820-R2-02→--list 推导+7 负控自检+真实 autodown PASS。R2-01/health 维持 R3 CLOSED 未回退。check→finding 映射：R3-01/V-04→resources 族两测；R1-01→metering 注入测；R2-02→脚本 GuardSelfTest+autodown 实测。未跑整档（2026-10-14 时序裁定） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读核验 R3 修复+重放 R3 复现证据（2100 链预算/62 层溢出/missing-id 控制），绑定 a789a2adc）
2026-10-24 stage:review | review_scope:final-readiness+affected-repair+resource | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:e8497f2f8d275873f55b4d7c0e9dd6ac1b75e3bf | base_commit:baf4d7d49492444cfd8063495a0bbd26af1b1d86 | dependency_revisions/spec_inputs:最终全核pending，入审Plan/SD hash见报告 | acceptance_results:R2-01原失败安装反例独立关闭、health关闭；R1-01计量反控partial、R2-02 required ID聚合partial；新增R3-01共享布局验证预算fail | evidence:[R3报告/可复现输入/原始输出](attachments/820/review-r3/review.md)：incremental7/7、GuardSelfTest7控、admission Scope绿、root V10定向4/4；原除零失败after42/identity保留/retry43；2100层record至少2206050字段访问但verify接受；63类型真实layout溢出受控拒（原“不可达”验收依据错误）；missing-ID反控仍acceptance假绿、root完整ID成功输出误红；HEAD/clean复核一致 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:work按报告修原finding余量/共享预算与必需overflow测试，T04/06/09重开，executing4/9；未审AC/V/SD不声称pass，未合入。
2026-10-24 stage:work | PLAN-820 R2 修复轮 | outcome:pass（T-06/T-09 按 review-r2 三 finding 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:baf4d7d49（R1 冻结基线） | code_commit:e8497f2f8（树 clean） | dependency_revisions:无新增 | task_ids:T-06/T-09（重闭） | evidence:incremental 7/7（新增 failed_edit_installs_nothing、metering_injection_controls_redden_the_gate）；av_data_820 13/13；semantics 13/13（健康修复后重放）；resources 6/6；脚本 6 Scope+GuardSelfTest 实测 PASS；probe 全目标零警告；workspace check 绿。**具名根因**：(1) R2-01 根因=edit_body/apply_source/apply_caps 均先安装状态后运行验证，错误返回早于 candidate.commit 造成会话/环境事务分裂——修复=三入口统一 build-aside+run-before-publish 事务（候选准入/link/真实 run/结构门全成才发布）；(2) R1-01 余量根因=zero_io() 常量充当 raw、parse_fn_items/HIR Arc 为消费者侧填值而非生产对象绑定——修复=ProductionEvents 真实调用点计数+bundle_ptr 生产对象身份轴+注入反控缝经同一计量入口判红；(3) R2-02 根因=Run-Tier 只看合计 passed>0 与 failed，ignored/漏必需 id/账目不一致均可绿——修复=必需 ID 全名集逐 id ok 核对+ignored 红+ok 行数==passed 合计一致性+0 选红+GuardSelfTest 正反控自检。**关闭表**：F820-R2-01→failed_edit_installs_nothing（fast+apply_source 双入口，/0 trap 零安装+retry 43）+三入口事务化；F820-R1-01 余量→metering_injection_controls（读盘/重验注入同一门红）+body_edit 计量断言（events 四轴仅被改 def 各 1）+bundle_ptr 轴；F820-R2-02→GuardSelfTest（2正5负）+滤串档全名修正的先红后修实测；健康→全目标零警告。R1-02/03 维持 R2 复审 CLOSED 判定未回退。check→finding 映射：R2-01→av_data_820_v13_failed_edit_installs_nothing；R1-01余量→av_data_820_v13_metering_injection_controls_redden_the_gate+av_data_820_v13_body_edit_is_function_level；R2-02→scripts/verify-av-data-820.ps1 -GuardSelfTest+6 Scope。未跑整档（2026-10-14 时序裁定） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读核验 R2 修复+重放 R2 复现证据，绑定 e8497f2f8）
2026-10-24 stage:review | review_scope:final-readiness+affected-repair | reviewer_identity:本聊天Codex，独立于实施，未启动代理 | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:baf4d7d49492444cfd8063495a0bbd26af1b1d86 | base_commit:a33538088cd4ce309176d217361edc68a1d2ced8 | dependency_revisions/spec_inputs:完整再核pending，SD hash与Plan入审hash见报告 | acceptance_results:R1-02父侧峰值与R1-03同源码native独立关闭；R1-01 partial（真实事件计量缺口）；AC09 fail/AC06聚合fail，未审面pending | findings:F820-R2-01 P1失败edit返回Err却已发布坏snapshot（独立除零反例）；F820-R2-02 P2 required skip仍被脚本报PASS；新增测试Local0 warning待清 | evidence:[R2报告/原始输出/反例](attachments/820/review-r2/review.md)；incremental5/5、worker2/2（1.1MiB增量/64.1MiB暂态捕获）、native1/1，diff-check绿，受审HEAD/clean前后相同 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:原树work先给根因/覆盖与设计调整表（两轮non-pass），修失败安装全出口、真实事件oracle及聚合反控，清warning后提交再审；T06/T09重开、executing5/9，其他完成主张保留但不代替独立AC pass。
2026-10-24 stage:work | PLAN-820 R1 修复轮 | outcome:pass（T-02/T-04/T-06/T-09 按 review-r1.md 三 finding 修复重闭，7/9，可再送审） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:a33538088（r6 冻结基线） | code_commit:baf4d7d49（树 clean） | dependency_revisions:无新增（probe 独立 workspace 自足；windows-sys 0.59 为 probe 新增直依赖，与 ac-artifact-probe 同版本同源） | task_ids:T-02/T-04/T-06/T-09（重闭） | evidence:av_data_820_incremental 5/5（fast path 解析面=1/body_work 仅被改函数/收据 1:n-1/身份+代际不变/IO=0；双退化反控经同一结构门非零；依赖四轴精确失效：布局/嵌套/签名/caps+no-change 空集+helper 不波及调用者+link 拒绝前快照存活；旧快照对照；non-fast 显式拒）；av_data_820 13/13（含 F-03 native 独立对照 42=手审=data）；B 线父侧 A/B 实测增量 ~1.0MiB（≤128MiB）+暂态反控捕获 64.1MiB/122 样本；probe workspace cargo check 绿；脚本 first-case/resources Scope PASS。check→finding 映射：F820-R1-01→incremental 族+edit_body 内建门；F820-R1-02→av_data_820_worker_memory_envelope_b_line+av_data_820_worker_peak_sampler_catches_transient；F820-R1-03→av_data_820_first_case_native_independent_comparison。未跑整档（2026-10-14 时序裁定） | findings:F820-R1-01/02/03 全闭；SD 草案 SD-07 与 contract-decision §11/§13 事实错误已修正（r1 报告绑定 hash 失效，新 hash 由送审时再审冻结） | blockers:无 | merge-owned 门禁仍 pending：裸 cargo t+VM/编译器触面 cargo tv+独立 workspace 完整有限合同档（最终 review pass 后由 merge 在冻结 HEAD 执行） | next_responsible_role:review（T-07 再审：独立上下文、同树只读核验 R1 修复与 r1 未审面，绑定 baf4d7d49）
2026-10-24 stage:review | review_scope:final-readiness | reviewer_identity:本聊天 Codex（独立于实施上下文；未启动代理） | PLAN-820 r6 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | reviewed_commit:a33538088cd4ce309176d217361edc68a1d2ced8 | base_commit:37e51253689631ffa61a9af58041c5f6646a7569 | dependency_revisions/spec_inputs:未完成全量再核，见报告 | acceptance_results:AC01/05 partial、AC09 fail，其他行为验收 pending | findings:F820-R1-01 全链增量仍整源码前端重编且正式退化反控缺失；F820-R1-02 请求结束采样不能证明父侧worker瞬态峰值；F820-R1-03 同源码native oracle被未授权替换 | evidence:[复审报告](attachments/820/review-r1.md)，独立 incremental 3/3 绿证明现有测试漏掉前端退化，diff-check绿、HEAD/clean复核一致；SD草案hash冻结于报告 | pending_closeout_gates:merge-owned裸cargo t+tv+独立workspace完整有限合同档 | next:回work关闭readiness缺口，T02/04/06/09重开，保留T01/03/05完成（3/9）；不得把历史完成行作为本轮pass。

2026-10-24 stage:work | PLAN-820 r6 | outcome:pass（T-02..T-06+T-09 全部 work-owned 任务完成，execution_done） | worktree:D:/autostack/.wt/lang-820/auto-lang（组含 auto-down 兄弟 detached@c39ac7f，826 跨仓 path 依赖） | branch:plan-820-dev | base_commit:37e512536 | code_commit:5323bf863→1a21a9752→3b3d76a9c→2a270f0b4→a33538088（5 commits，树 clean） | dependency_revisions:D-798/D828 archived；auto-lang-core/auto-test-case 公共写面追加（recipe 三处注册，817 cte 同模式）；818 在途注册竞争按批次合同合并时串行 | task_ids:T-02..T-06+T-09（7/9） | evidence:av_data_820 五族 11+13+6+5+3 全绿；core 26/26；全 probe 套件 25 行零回归（828 增量族 21/21=379s）；corpus 含 data case 绿；ac_av_data_probe 4/4；acceptance 脚本 exit0；AC-01..06+09 的 V-01..13 落点见各任务行与 sd-draft.md | blockers:无 | merge-owned 门禁 pending：裸 cargo t+tv（VM/编译器触面）+ 新 workspace 完整有限合同档——最终 review pass 后由 merge 在冻结 HEAD 执行 | next_responsible_role:review（T-07：独立上下文同树只读核验 AC/V/SD、遗漏/债、健康检查、填规范影响）

2026-10-24 stage:work | PLAN-820 r6 | outcome:in-progress（T-02 部分交付，保持 executing） | worktree:D:/autostack/.wt/lang-820/auto-lang | branch:plan-820-dev | base_commit:37e512536 | code_commit:5323bf863+1a21a9752 | dependency_revisions:D-798/D828 archived；ac-av-probe 写窗口持有中 | task_ids:T-02 进行中（1/9 完成计数不变） | evidence:probe/tests/av_data_820.rs 10/10；core 25/25；全 probe 套件绿（828 增量族 21/21 实测 287.85s）；verify-av-data-820.ps1 first-case exit0 | blockers:无（T-02 剩余=AutoDown recipe 注册（公共写面串行窗口）、双线内存实测、native record 面核对；随后 T-03..T-09） | next_responsible_role:work 继续 T-02 收尾→T-03

2026-10-24 stage:work | PLAN-820 r5→r6（执行启动） | outcome:pass（T-01 完成，drafting→executing；用户授权"实施"） | worktree:无（T-01 只读准备，产物落 docs/plans/attachments/820/；实施树 T-02 创建）| branch:—| base_commit:11ba8fd0a（主检出，未动代码）| code_commit:—（仅计划簿记）| dependency_revisions:D-798 archived、D828 archived 11/11、817 已合入；ac-av-probe 写窗口空闲（821-827 drafting 0 步）| task_ids:T-01（1/9）| evidence:attachments/820/contract-decision.md r1；主检出预检=crates/test/experimental 零 WIP | blockers:无 | next_responsible_role:work 继续 T-02（建树 lang-820、刷新基面记录、first-case whole-path）

2026-10-21 stage:new | PLAN-820 r4→r5 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-820 r3→r4 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-07 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增类型等价与执行表示的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-820 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-798及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..06/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

无须用户再裁定已认可的总体方向。精确opcode/布局实现/资源测量由有界T-01和T-02负责冻结，不能改变§5语义或预算；需要扩大支持/资源/兼容承诺时回new显式revision。当前依赖与执行授权门如§4；不以等待实施为由伪造通过或删AC。NaN-boxing、完整数值运算/用户泛型、共享图、async、native与CTE复合消费者另案，不是本计划“已完成后续”。


**当前排期与验收（2026-10-21）**：D828后按§5本计划优先级/原D门及单写交接推进，不等待无硬依赖的整批；T-09 / AC-09 / V-13 未完成，旧revision/测试总数不覆盖新增义务。用户已确认该顺序/目标，不重复请求同范围授权。
