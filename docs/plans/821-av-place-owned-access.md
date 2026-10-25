---
plan_id: PLAN-821
status: executing
feature_name: AV Place拥有型读写与全出口清理
plan_revision: 5
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-25
current_step: 1
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-hir/stage-contract.md
new_spec_components:
  - docs/specs/auto-av/place-ownership.md
  - docs/specs/auto-mir/owned-access-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-821] AV Place拥有型读写与全出口清理

## 0. 变更摘要

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付Str/char、拥有型record、Place权限与Drop。2026-10-25 用户指令实施（/auto-plan:work）：T-01 已完成冻结（[attachments/821/contract-decision.md](attachments/821/contract-decision.md)）；T-02 等共享核心单写窗口交接（§10）。不得将计划支持面写成现状Spec。

## 1. 目标

在820布局/字段Place上，沿774/782接入char、borrowed str、owned Str及拥有Str字段的record，统一ReadCopy/Borrow/Move/Initialize/Replace/Drop。提供LHS reservation、明确初始化/所有权状态和AV所有受控退出的清理。已有text/text-record源码路径必须真实接入；不是只模拟RC计数。

成功标志：真实受检MIR→ABC→AV在新profile下执行，正例独立oracle一致、坏输入在消费前拒绝、资源/版本/恢复可验证，旧core-i32及实际已合入的817/818标量路径兼容。

非目标：部分字段move、完整NLL/生命周期参数、用户析构/可挂起析构、循环共享/GC、List/闭包/Dyn、真实async、native ABI/FFI及817/818能力扩大。

范围只在auto-lang仓的共享实验核心、独立fixtures/recipe/本计划脚本/证据；根workspace成员/产品manifest、旧VM、UI/Man/SDK及他方在途实现不改；仅允许本recipe在当前真实owner处追加最小注册/父侧判定薄接线，并按实际根Rust触面验证。共享注册位置服从814–816实际迁移后的owner，不复制协议/执行器。

## 2. 架构方案

```text
支持的已有Checked HIR / 明确未信任typed fixture
  → av.owned/1 MIR + verify
  → ABC encode → decode/verify → LoadedModule
  → AV typed access / layout / lifecycle → typed result/diagnostic
             ↕ 783 managed store（代码与portable载荷分域）
   新worker恢复重验 + AutoDown父侧独立oracle
```

执行profile拟为 `av.owned/1`，名字/编号由T-01确认不冲突后冻结；不复用core-i32授信。纯语义TypeRef与AV LayoutRef、native target ABI分离；不再创建VM私有推导器或第二MIR。

## 3. 技术栈

Rust、D依赖交付的 `experimental/ac-av-probe/` 独立workspace（已观察package为auto-av-core/auto-av-probe，work须重核），783 ManagedStore、796语义事实、现有probe/run/contained worker。

影响入口：既有共享MIR/verifier/ABC/AV/adapter及820 layout；拟新增 `core/src/{place,ownership,owned_text}.rs` 和受限测试资源shim，新增对应测试/fixtures。`experimental/ac-core`与旧VM只读。

拟新增 `scripts/verify-av-owned-821.ps1`，本计划测试前缀 `av_owned_821`、`test/cases/av-data/821/`、独立recipe `av-data-821`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-820**=前序profile实际交付，且独立最终review、冻结门禁、合入、Spec与merge收据齐全；实际启动重验提交祖先/Spec，不只看frontmatter。T-01只读准备可先做，T-02及依赖构建等D和用户执行授权。797不是本计划前置；817/818共享API消费映射按Design44 §12单写交接，不能修改其在途合同。若仍在改公共定义，先等owner交接/实际合入，不从其分支抄代码开工。

拟唯一实施树 `D:/autostack/.wt/lang-821/auto-lang`，分支 `plan-821-dev`；本轮未创建。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

## 5. 详细设计

### 5.1 当前执行合同

1. 完全沿774的UTF-8/Unicode标量索引、char名义类型、默认view、move-in/return责任、str不入字段/容器。拥有型record只允许整根move/clone；部分move前门拒绝。
2. Place base+projection+type+access绑定原存储；LHS从左到右各求值一次，RHS前reservation按根保守冲突。允许RHS只读/显式clone但禁止move/drop/写重叠根、扩容、未知effect/回调和挂起；RHS后激活并重验。
3. 复制定位handle不复制拥有权；复合Value不能延用798标量Copy derive。BorrowRead/Write使用受限token，权限和存活单独检查；move源Uninit，分支/循环must分析随状态验证。
4. Replace先准备新值，再一次提交init/move状态，后清理旧值；失败保留旧值和源责任。纯Copy自赋值合法；owned自move及重叠move拒绝。分配器/shim失败使用明确resource/infra，不以Null成功。
5. 正常return/break/continue/语句尾临时按逆建立序清理；AV trap/cancel/fuel/frame/memory耗尽与构造失败同样回滚。内建drop无guest分配/回调/挂起，用显式栈和预留记账；cleanup失败单报并保留原错误。
6. 帧实例/store/槽位generation和borrow token隔离；generation回绕退役。内建owned数据不形成环；shared/weak、借用逃逸和跨挂起明确拒绝，不拿运行期兜底冒称静态完整。
7. 新profile的typed entry/result建立宿主接收owned结果的唯一责任，不放松798旧入口。borrow返回追踪真实owner，合法静态字面量/借入参数view在callee返回后继续有效；局部/move-in/活mut来源沿774拒绝，不把所有跨callee token一概判死。

### 5.2 有界首例与资源政策

T-01冻结支持/拒绝、类型/指令/wire版本、生命周期、输入前门及计量决策，不在下游实现中临时猜规则。T-02必须完成：text-record源码→既有verify_text_record→新owned MIR/ABC→AV；从含非空Str的record读取view、clone RHS后replace字段，return转移所有权，宿主接收并显式释放结果后live=0。与旧782 native/固定文本及顺序对照；首例cache/restart/AutoDown真实接线。

首例不可行则记录实际输入、拒绝/资源原因和合同冲突，回new显式revision；不能改用旧VM、弱化verifier、去掉oracle或只留纯模型。未知语法/接口的fixture是未信任typed输入，真verify授信后执行，不把未落地接口描述当证据。

沿D依赖已冻结输入≤4MiB、module/每函数表≤65536、验证≤200万步/5s、执行≤100万步/1024帧/2s的**适用上限交集**；旧profile更小cap仍生效，单位/计步在T-01核对，不按本段放宽旧合同。新增类型描述、路径、图、construct/drop/freeze必须申请预算并使用有界显式遍历。值数据合计≤8MiB（槽+聚合+buffer+环境+Dyn），受管总32MiB（hot8/staging16/execution8），不是各类各领8MiB；allocator/库暂存按真实峰值报告。新worker private-bytes相对空载增量沿798上限128MiB，包含本次allocator/解码/临时对象；父侧采样瞬态峰值，与自有计数共同判定。native工具进程另列归因，不从worker实际峰值扣掉前端。

这些是拟议实验资源政策，非用户给定或永久语言限制；T-01如不能成立须带边界输入回规划，不能在work直接提高。OOM/预算/cancel/infra与invalid/unsupported/trap分态，分配前拒绝；cleanup预留记账不受正常fuel耗尽取消，不暗加语法深度限制。

### 5.3 存储、注册与兼容

保留D-798冻结的标量编辑负载和采样口径，定向重放其请求到当前结果P95<1000ms；冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。性能/峰值测量本机单实例串行，负载状态入证据；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

首例沿783真实代码producer、root/lease/GC，cache恢复重走MIR/ABC门；类型/字段集/layout/profile/语义版本入依赖，坏内层即使好hash也不得执行。活帧/borrow/宿主地址不存cache。首例独立recipe注册和父侧expected比较是T-02交付，不允许“有fixture但execution pending”结项。

共享注册入口现观察为 `crates/auto-lang/src/test_cases/{sema,mod,stale}.rs`（若已迁移以816交付为准）。只追加自己的recipe/身份和独立worker处理，不复制整个sema worker，不修改v1/v2解码或他方recipe合同。若协议不足，在T-01由实际owner决定最小版本化增量，依赖实施暂停。

旧标量语料与实际已合入817/818路径必须重放：仅保证原profile兼容及未知新profile明确拒绝；不强制其他后端实现新数据。既有text/text-record合法语料使用770/782的旧HIR-native；AV取消/trap清理另用真实资源事件与分配故障oracle，native进程退出不能代证。

### 5.4 ABC共享合同与本计划责任（r2）

依据[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44 §15](../design/44-av-data-layout-place-and-ownership.md#15-abc补充合同与逐profile验收r2)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)，本计划承担**move-kill与清理分析**：将只增加def的初始化分析扩为move-kill、借用权限、reservation/commit和全出口清理义务；参数view/mut/move及返回owner、效果进入签名，非Copy资源不得复用标量Copy实现。MovePath只intern实际使用根/投影及前缀，有界worklist计量；继续拒绝部分move，不预展开所有字段组合。

语义MIR、portable ABC和私有LoadedProgram分层，不另建语义IR。825不是本计划硬依赖：其已合入时适配受检执行表示，未交付时允许同一私有接口的安全参考路径；未知profile不进无验证fallback。kind/entry/imports及歧义旧缓存策略消费D-798修复，不猜函数数。owner、活分支和Drop/必要类型是语义，strip debug不得移除。

类型/路径分析、clone/drop/编解码及cleanup计入既有合计预算；输入要求的capabilities与当前host grants分开，优化不绕过授权、逻辑fuel和取消。帧arena、slot复用、融合及紧凑wire属825/826或另案性能，不以其完成替代本计划正确性。T-01冻结V-11具体fixture/反控和各层合同映射，T-03/04实现，T-05覆盖恢复，T-06汇总；不得只放设计注释而漏验收。

### 5.5 文档复审补充合同（r3）

初始化合流取交集，可能活owner/loan保守并集；cleanup按动态init/drop guard清理实际活根，不用must-init集合代替清理表。分支一侧move、条件构造、零次循环及loop back-edge均求有界不动点；活loan任一路径存在都阻止冲突写，返回view所有路径来源合法。正常/取消/耗尽都核跨类型LIFO。

准入错误在decode/verify/load门拒绝、dispatch=0；已准入程序的越界、失效句柄/guard、错型take、取消等允许此前合法执行，须断言危险目标访问/提交/目标callee计数0及原值/清理，不要求整次dispatch=0。prepare/commit只保护尚未提交的转移；已发生的guest求值/调用效果不回滚。

消费前冻结已合入(profile, wire/schema, semantic-version, producer)矩阵；825/826可前置但不是本计划硬依赖，820指令表尚未交付时它们沿D798现有合同，不反向等待本计划。当前core/注册窗口只能在依赖满足后持有，不持锁等其他owner。注册的最小接线是范围内例外，不等于修改旧VM或产品默认；814–816只按实际已迁移owner交接。

资源表沿798精确族口径：模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536；index上限还须小于对应表长及总额，不能为取最大索引挤掉必要块/指令。32MiB分区和128MiB进程增量分别验收，不扣前端/allocator/暂存，已有约211MiB前端证据不能记债后当D798已达。T-01核当前D门及同口径证据，冲突先回规划，不擅自豁免或提高数字。

T-01在contract-decision冻结case→子case→Scope→真实入口→阶段/码→oracle→必需/适用矩阵，包含V-12；T-02使first-case和recipe正控可执行，T-03/04实现与反控，T-05核恢复/兼容，T-06验必需集合无漏例/重复冒计/skip。合法特定profile外输入应按合同拒绝，不能误算成适用例未执行而跳过。

### 全链路增量兼容与828优先排期（r4）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：函数局部init/owner/loan/cleanup复用，旧snapshot的layout/drop在引用期间保活；无活值布局迁移支持时拒绝不兼容安装。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 执行排期与单写交接（2026-10-21）

优先功能链820→821→822→823→824的第二项，T-02必须等D820和D828；不等817/818全部完成。只读准备可并行，依赖实现不可并行消费820 WIP；不等825/826。

所有后继在D828前仅作只读准备，现有实现与证据保留。共用[执行顺序与交接合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)：T-01准备写面/owner/消费者映射，T-02在D门满足后刷新基面、版本、预算与接口并记录窗口接受；既有兼容/coverage工作记录定向结果，独立review核记录，merge补实际合入与窗口释放。公共核心默认单写，公共注册追加也串行；不同worktree不豁免。不能持窗等未交付依赖、复制WIP或改变他方在途合同；无公共写面者可并行消费稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/place-ownership.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/owned-access-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-821独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-hir/stage-contract.md | 已有HIR域语义保持→追加其到av.owned/1 MIR/ABC的实际消费支持/拒绝/资源责任 | 不将新后端能力反授旧域 | AC-01/02/04/06 |
| SD-07 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

不变量为Design44 I-01..12及§5。以下V均为**必需补充反例/正控**，T-02/T-04将其变为真实入口；固定语料总数不能替代。每案保存输入身份、阶段/码、result/trace/bits/owner与原始日志。期望由人工或独立小模型确定，不能由被测布局/清理算法生成。

| Case ID | AC / work owner | 输入/状态 | 实际或拟入口 | 可观察oracle与区分断言 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02；T-02 | 嵌套字段replace，RHS读同根；RHS修改/移动同根对照 | 新profile verify/ABC/AV，av_owned_821 family | 只读RHS成功且仅求值一次；冲突前门拒绝 | 原位/reservation；仅声明支持面 |
| V-02 | AC-02/03；T-03/04 | move后读、两次move、只读写、重叠mut与合法多view | 新profile verify/ABC/AV，av_owned_821 family | 各自特定ownership诊断，合法对照不被误拒 | Move/borrow；仅声明支持面 |
| V-03 | AC-03/05；T-03/04 | 第n次用户/记账分配失败、构造中途失败、重叠clone替换 | 新profile verify/ABC/AV，av_owned_821 family | 旧bytes/owner不变，未提交源未消耗，临时恰清理一次 | 事务失败；仅声明支持面 |
| V-04 | AC-04；T-03/04 | return/break/continue/trap/cancel/fuel/frame/memory耗尽 | 新profile verify/ABC/AV，av_owned_821 family | 独立事件oracle证明逆序唯一drop；结果转出不误释放 | 所有退出；仅声明支持面 |
| V-05 | AC-02/04；T-03/04 | 普通整数等于活资源id、旧帧复用、跨store、generation边界 | 新profile verify/ABC/AV，av_owned_821 family | 整数仍整数；无效访问在读取前受控拒绝 | 句柄碰撞；仅声明支持面 |
| V-06 | AC-02/03；T-03/04 | 空串/NUL/组合Unicode、view返回合法来源/局部逃逸、部分move | 新profile verify/ABC/AV，av_owned_821 family | 沿774结果/诊断；owned返回接收方释放后live=0 | 文本/拥有；仅声明支持面 |
| V-07 | AC-05；T-04 | 合法输入cap内/超1单位、瞬态峰值、故障第n次 | 新worker+父侧budget/allocator hook | resource分态；预算前拒的该申请attempt=0，故障注入attempt存在但successful_alloc=0；先前合法事件保留，危险访问/提交0；cleanup/真实峰值断言触发，无panic | 不能只看终态RSS |
| V-08 | AC-06；T-02/05 | fresh、同store新worker重启、改type/layout/profile、好hash坏payload | 783真实恢复→verify→AV | 合法恢复结果相同；依赖变化不复用；非法执行计数0 | 不缓存PASS或trusted |
| V-09 | AC-06；T-05 | core-i32及实际交付817/818旧profile、新profile送旧consumer | 实际原recipe/consumer | 旧结果/诊断不变，未知profile明确拒绝 | 无假native/CTE支持 |
| V-10 | AC-01/04/06；T-06 | worker错结果/错bits、漏case/skip、错工具身份 | 正式AutoDown recipe+父侧oracle | 合法正控PASS；每项错实际使总体非零/ERROR | 缺必要工具/路径不可绿 |
| V-11 | AC-02/03/04/05/06；T-01/03/04/05/06 | 分支一侧move后合流读、零次循环、move后call、参数模式不符、移除debug后的trap/cancel清理、稀疏字段路径及超预算 | §3真实MIR/ABC授信、执行及store新worker恢复；T-01冻结输入，T-03/04使可运行 | use-after-move/模式冲突授信前拒绝；合法根move仅新owner清理一次；strip不漏drop，稀疏路径状态按实际路径增长，超额受控拒绝；好hash不得绕过；准入非法dispatch=0，运行期故障按§5.5断言目标危险操作0 | 仅本profile；标量/旧消费者仍V-09，受管/进程峰值沿V-07 |
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | 分支一侧move另一侧保留；仅一臂初始化的未发布临时；单臂活view合流后mut；零次/多次循环；合法两臂end-borrow后mut | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | 合流read/mut不安全者前门拒绝；合法两臂结束借用后mut可行；两个运行分支退出时各按真实活根清理一次，不泄漏/双drop，未初始化临时不清理；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_owned_821`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-owned-821.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
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
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应move-kill与清理分析规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（821含owned资源）；§5.4及V-11对应move-kill与清理分析规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应move-kill与清理分析规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应move-kill与清理分析规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应move-kill与清理分析规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

所有任务初始未完成；current_step仅计实际完成任务，不提前勾review/merge。T-01在本轮亦未执行实施调查，仅起草取材已完成。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户授权；只读准备可早于D-820 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/821/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜 [✅ 2026-10-25 已完成：冻结件 r1 落 attachments/821/contract-decision.md——D门核验（D-820/D828 archived、817/818 已合入）、av.owned/1=profile id 3/AVAD v2 路由、指令集 0A..1B+共享 opcode owned 语义扩展（InstructionSpec 行）、move-kill/借用/清理/reservation 分析冻结（ownership.rs 槽状态格+保守并集）、预算表（8MiB 合计含 Str buffer）、V-01..13 矩阵含 cap 两侧、写面登记与 T-02 解锁条件；零代码，只读] | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-02 | work | T-01、D-820、注册owner交接、D828排期门、当前单写窗口交接 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记 | AC-01/06；first-case、V-01/08/10正控 |
| [ ] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile | AC-02/03；av_owned_821定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录

2026-10-25 stage:work | PLAN-821 r5 | outcome:**blocked**（T-01 pass 1/9；T-02 窗口阻塞，见 §10） | worktree:`D:/autostack/.wt/lang-821/auto-lang` | branch:`plan-821-dev` | base:`c66528e80`(master=820 收口) | code_commit:无（T-01 只读零代码；本树暂无提交） | dependency_revisions:D-820=archived 9/9（merge 10:20–10:55 收据齐全）、D828=archived 11/11、817/818=均已合入 | task_ids:T-01 | evidence:[attachments/821/contract-decision.md](attachments/821/contract-decision.md) r1（§0 D门表/§1 身份/§4 指令集/§5 分析/§8 预算/§12 矩阵/§13 写面登记） | blockers:共享核心单写窗口被 PLAN-827 在途未合入核心写持有（core/src/av.rs+frame_allocation.rs+lib.rs，2026-10-25 08:10，R8 修复轮中）；与 820 合入并行未收敛 | next_responsible_role:work 续 T-02——解锁=827 merge+窗口释放收据（本计划 rebase 重核基面后记窗口接受），或 827 显式记录写面不重叠细分交接（默认不成立，av.rs/Value 扩展重叠面高）；持窗等待期间不复制 827 WIP。

2026-10-25 stage:work | PLAN-821 r5 | entry:用户指令"计划821: /auto-plan-work 实施"=本Plan实施授权 | 入口门核验：D-820 满足（archive/820，9/9，merge 收据 2026-10-25 10:20–10:55 齐全：Spec 沉淀 227101fd8/归档 81250e0a7/收口 c66528e80）；D828 满足（archive/828，11/11）；注册 owner=820 已随合并释放（827 复用既有 recipe 零注册追加）；主检出自查=无已跟踪代码路径 WIP（仅 docs/plans、docs/design、.autoos 簿记改动与未跟踪构建产物，非本会话所为，保留不动）。唯一实施树 `D:/autostack/.wt/lang-821/auto-lang` @c66528e80 分支 `plan-821-dev` 已创建；status drafting→executing。**T-02 单写窗口约束**：PLAN-827 在途未合入分支含共享核心写（core/src/av.rs +34、新增 frame_allocation.rs、lib.rs，2026-10-25 08:10），与 820 合入（10:20+）在核心上并行未收敛；按 ACE 合同公共核心默认单窗，T-02 须待 827 窗口释放交接。T-01 只读准备先行。

2026-10-21 stage:new | PLAN-821 r4→r5 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-821 r3→r4 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-07 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增move-kill与清理分析的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-821 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-820及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..06/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

无须用户再裁定已认可的总体方向。精确opcode/布局实现/资源测量由有界T-01和T-02负责冻结，不能改变§5语义或预算；需要扩大支持/资源/兼容承诺时回new显式revision。当前依赖与执行授权门如§4；不以等待实施为由伪造通过或删AC。NaN-boxing、完整数值运算/用户泛型、共享图、async、native与CTE复合消费者另案，不是本计划“已完成后续”。

**当前阻塞（2026-10-25 work T-01 后）**：T-02 等共享核心单写窗口交接——PLAN-827 在途未合入分支持有核心写（core/src/av.rs、frame_allocation.rs、lib.rs），按 ACE 合同默认单窗须待其 merge 释放或显式不重叠交接（[contract-decision.md §13](attachments/821/contract-decision.md)）；解锁动作=827 窗口释放收据后本计划 rebase 重核基面并记录窗口接受。


**当前排期与验收（2026-10-21）**：D828后按§5本计划优先级/原D门及单写交接推进，不等待无硬依赖的整批；T-09 / AC-09 / V-13 未完成，旧revision/测试总数不覆盖新增义务。用户已确认该顺序/目标，不重复请求同范围授权。
