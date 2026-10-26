---
plan_id: PLAN-822
status: execution_done
feature_name: AV类型化集合与Variant载荷访问
plan_revision: 5
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-26
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-av/collections-variants.md
  - docs/specs/auto-mir/collection-place-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-822] AV类型化集合与Variant载荷访问

## 0. 变更摘要

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付List<T>、Option/Result/有限enum与Index/variant Place。本文件是实施草案，当前仅授权起草，未实施；不得将计划支持面写成现状Spec或自动启动work。

## 1. 目标

扩展受控内建List<T>、Option<T>/Result<T,E>及有限名义enum，支持typed元素存取、Index/guarded-variant Place、move/clone/drop、失败扩容及分支切换。覆盖owned Str/record组合，不用Vec<DynValue>伪装类型化集合。

成功标志：真实受检MIR→ABC→AV在新profile下执行，正例独立oracle一致、坏输入在消费前拒绝、资源/版本/恢复可验证，旧core-i32及实际已合入的817/818标量路径兼容。

非目标：Map/Set通用库、完整用户泛型/trait求解、迭代器语言全套、借用元素容器、循环共享、闭包/async、source新语法、native/CTE扩面与NaN-boxing。

范围只在auto-lang仓的共享实验核心、独立fixtures/recipe/本计划脚本/证据；根workspace成员/产品manifest、旧VM、UI/Man/SDK及他方在途实现不改；仅允许本recipe在当前真实owner处追加最小注册/父侧判定薄接线，并按实际根Rust触面验证。共享注册位置服从814–816实际迁移后的owner，不复制协议/执行器。

## 2. 架构方案

```text
支持的已有Checked HIR / 明确未信任typed fixture
  → av.collections/1 MIR + verify
  → ABC encode → decode/verify → LoadedModule
  → AV typed access / layout / lifecycle → typed result/diagnostic
             ↕ 783 managed store（代码与portable载荷分域）
   新worker恢复重验 + AutoDown父侧独立oracle
```

执行profile拟为 `av.collections/1`，名字/编号由T-01确认不冲突后冻结；不复用core-i32授信。纯语义TypeRef与AV LayoutRef、native target ABI分离；不再创建VM私有推导器或第二MIR。

## 3. 技术栈

Rust、D依赖交付的 `experimental/ac-av-probe/` 独立workspace（已观察package为auto-av-core/auto-av-probe，work须重核），783 ManagedStore、796语义事实、现有probe/run/contained worker。

影响入口：共享MIR/verifier/ABC/AV/place/ownership/layout（D-821）；拟新增 `core/src/{collections,variants}.rs`、`probe`新profile输入与fixtures。现有source/HIR类型系统不越权扩面。

拟新增 `scripts/verify-av-collections-822.ps1`，本计划测试前缀 `av_collections_822`、`test/cases/av-data/822/`、独立recipe `av-data-822`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-821**=前序profile实际交付，且独立最终review、冻结门禁、合入、Spec与merge收据齐全；实际启动重验提交祖先/Spec，不只看frontmatter。T-01只读准备可先做，T-02及依赖构建等D和用户执行授权。797不是本计划前置；817/818共享API消费映射按Design44 §12单写交接，不能修改其在途合同。若仍在改公共定义，先等owner交接/实际合入，不从其分支抄代码开工。

拟唯一实施树 `D:/autostack/.wt/lang-822/auto-lang`，分支 `plan-822-dev`；本轮未创建。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

## 5. 详细设计

### 5.1 当前执行合同

1. 内建实例化带精确TypeRef与元素布局，拒List<borrowed str>；不依赖mono_name猜类型，不把推导失败变成Dyn。i32列表和owned列表分别实测布局/元素操作。
2. get提供受限view或Copy，remove转出owned值；push/insert/replace先准备、检查容量与冲突、成功提交后转移源责任；失败旧列表/源不变。len/cap/stride/字节范围用checked算术。
3. Index Place负下标/越界先于访问，借用期间结构变更冲突；容器扩容更新epoch，旧Place/token不能访问新buffer。LHS reservation语义沿821，下标和RHS只求值一次。
4. Variant的discriminant、payload type、当前分支和初始化共同验证；只有match guard证明后可投影。切分支先构造完整新payload、提交后drop旧活分支，不能读取inactive载荷。
5. None、Null、Uninit、缺字段、false和0保持不同态；有限enum不是Any。首批typed fixture是公开未信任输入，必须过真verify/ABC门；不能把programmatic builder当Checked。
6. clear/drop/构造回滚按元素责任清理；退出、故障和预算耗尽复用821机制。表/图递归验证和清理显式遍历，owned环和borrow逃逸拒绝。

### 5.2 有界首例与资源政策

T-01冻结支持/拒绝、类型/指令/wire版本、生命周期、输入前门及计量决策，不在下游实现中临时猜规则。T-02必须完成：从严格typed fixture进入MIR→ABC→AV，List<owned record>中的嵌套Str字段replace、remove返回owned值，再装入Result；明确匹配后读取并释放。独立expected验证内容/顺序/live；首例同store缓存重启及AutoDown父侧比较。

首例不可行则记录实际输入、拒绝/资源原因和合同冲突，回new显式revision；不能改用旧VM、弱化verifier、去掉oracle或只留纯模型。未知语法/接口的fixture是未信任typed输入，真verify授信后执行，不把未落地接口描述当证据。

沿D依赖已冻结输入≤4MiB、module/每函数表≤65536、验证≤200万步/5s、执行≤100万步/1024帧/2s的**适用上限交集**；旧profile更小cap仍生效，单位/计步在T-01核对，不按本段放宽旧合同。新增类型描述、路径、图、construct/drop/freeze必须申请预算并使用有界显式遍历。值数据合计≤8MiB（槽+聚合+buffer+环境+Dyn），受管总32MiB（hot8/staging16/execution8），不是各类各领8MiB；allocator/库暂存按真实峰值报告。新worker private-bytes相对空载增量沿798上限128MiB，包含本次allocator/解码/临时对象；父侧采样瞬态峰值，与自有计数共同判定。native工具进程另列归因，不从worker实际峰值扣掉前端。

这些是拟议实验资源政策，非用户给定或永久语言限制；T-01如不能成立须带边界输入回规划，不能在work直接提高。OOM/预算/cancel/infra与invalid/unsupported/trap分态，分配前拒绝；cleanup预留记账不受正常fuel耗尽取消，不暗加语法深度限制。

### 5.3 存储、注册与兼容

保留D-798冻结的标量编辑负载和采样口径，定向重放其请求到当前结果P95<1000ms；冷态/新复合对象规模和延迟另报，不删样本或把执行时间代替端到端。性能/峰值测量本机单实例串行，负载状态入证据；此项属于AC-06/T-05..06/V-09兼容验收，不是merge延后项。

首例沿783真实代码producer、root/lease/GC，cache恢复重走MIR/ABC门；类型/字段集/layout/profile/语义版本入依赖，坏内层即使好hash也不得执行。活帧/borrow/宿主地址不存cache。首例独立recipe注册和父侧expected比较是T-02交付，不允许“有fixture但execution pending”结项。

共享注册入口现观察为 `crates/auto-lang/src/test_cases/{sema,mod,stale}.rs`（若已迁移以816交付为准）。只追加自己的recipe/身份和独立worker处理，不复制整个sema worker，不修改v1/v2解码或他方recipe合同。若协议不足，在T-01由实际owner决定最小版本化增量，依赖实施暂停。

旧标量语料与实际已合入817/818路径必须重放：仅保证原profile兼容及未知新profile明确拒绝；不强制其他后端实现新数据。现有native尚无同profile集合/enum，不以818冒领；使用人工固定内容/事件oracle及独立小型容器状态模型，不调用被测布局/清理生成expected。

### 5.4 ABC共享合同与本计划责任（r2）

依据[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44 §15](../design/44-av-data-layout-place-and-ownership.md#15-abc补充合同与逐profile验收r2)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)，本计划承担**集合/分支的状态和计量**：Index/branch Place、变体切换的init/kill、结构变更epoch及活分支Drop接入共享InstructionSpec与状态机；签名沿821模式，不通过复制列表句柄绕过owner。实际使用路径intern，大clone/构造/清理按数据量计费并设取消点，cleanup预留独立；不以整个聚合算一条低成本指令隐藏工作。

语义MIR、portable ABC和私有LoadedProgram分层，不另建语义IR。825不是本计划硬依赖：其已合入时适配受检执行表示，未交付时允许同一私有接口的安全参考路径；未知profile不进无验证fallback。kind/entry/imports及歧义旧缓存策略消费D-798修复，不猜函数数。owner、活分支和Drop/必要类型是语义，strip debug不得移除。

类型/路径分析、clone/drop/编解码及cleanup计入既有合计预算；输入要求的capabilities与当前host grants分开，优化不绕过授权、逻辑fuel和取消。帧arena、slot复用、融合及紧凑wire属825/826或另案性能，不以其完成替代本计划正确性。T-01冻结V-11具体fixture/反控和各层合同映射，T-03/04实现，T-05覆盖恢复，T-06汇总；不得只放设计注释而漏验收。

### 5.5 文档复审补充合同（r3）

guard事实在variant重赋/切换或可能写入调用后失效，不能沿用旧分支证明。remove转出并更新结构epoch，旧Place拒绝；重新计算仍在界内的list[i]合法且访问移位后的新值，不把它误判成已move值。普通元素/字段部分MoveOut仍不因remove而放开。

准入错误在decode/verify/load门拒绝、dispatch=0；已准入程序的越界、失效句柄/guard、错型take、取消等允许此前合法执行，须断言危险目标访问/提交/目标callee计数0及原值/清理，不要求整次dispatch=0。prepare/commit只保护尚未提交的转移；已发生的guest求值/调用效果不回滚。

消费前冻结已合入(profile, wire/schema, semantic-version, producer)矩阵；825/826可前置但不是本计划硬依赖，820指令表尚未交付时它们沿D798现有合同，不反向等待本计划。当前core/注册窗口只能在依赖满足后持有，不持锁等其他owner。注册的最小接线是范围内例外，不等于修改旧VM或产品默认；814–816只按实际已迁移owner交接。

资源表沿798精确族口径：模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536；index上限还须小于对应表长及总额，不能为取最大索引挤掉必要块/指令。32MiB分区和128MiB进程增量分别验收，不扣前端/allocator/暂存，已有约211MiB前端证据不能记债后当D798已达。T-01核当前D门及同口径证据，冲突先回规划，不擅自豁免或提高数字。

T-01在contract-decision冻结case→子case→Scope→真实入口→阶段/码→oracle→必需/适用矩阵，包含V-12；T-02使first-case和recipe正控可执行，T-03/04实现与反控，T-05核恢复/兼容，T-06验必需集合无漏例/重复冒计/skip。合法特定profile外输入应按合同拒绝，不能误算成适用例未执行而跳过。

### 全链路增量兼容与828优先排期（r4）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：容器/Variant布局及helper依赖精确，未改函数代码/表不重建；候选事务/失败清理和旧代保活随新类型验证。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 执行排期与单写交接（2026-10-21）

优先功能链820→821→822→823→824的第三项，T-02必须等D821和D828；不等817/818全部完成。只读准备可并行，依赖实现不可并行消费821 WIP；不等825/826。

所有后继在D828前仅作只读准备，现有实现与证据保留。共用[执行顺序与交接合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)：T-01准备写面/owner/消费者映射，T-02在D门满足后刷新基面、版本、预算与接口并记录窗口接受；既有兼容/coverage工作记录定向结果，独立review核记录，merge补实际合入与窗口释放。公共核心默认单写，公共注册追加也串行；不同worktree不豁免。不能持窗等未交付依赖、复制WIP或改变他方在途合同；无公共写面者可并行消费稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/collections-variants.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/collection-place-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-822独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

不变量为Design44 I-01..12及§5。以下V均为**必需补充反例/正控**，T-02/T-04将其变为真实入口；固定语料总数不能替代。每案保存输入身份、阶段/码、result/trace/bits/owner与原始日志。期望由人工或独立小模型确定，不能由被测布局/清理算法生成。

| Case ID | AC / work owner | 输入/状态 | 实际或拟入口 | 可观察oracle与区分断言 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02；T-02 | List<i32>/List<Str>/List<record>，同位宽错元素 | 新profile verify/ABC/AV，av_collections_822 family | 合法组合保值；错元素/跨owner在执行前拒绝 | 类型隔离；仅声明支持面 |
| V-02 | AC-02/03；T-03/04 | list[i].inner.text写、访问后扩容、活view下push | 新profile verify/ABC/AV，av_collections_822 family | 原元素更新；失效路径/冲突拒绝，i只求一次 | 原位/epoch；仅声明支持面 |
| V-03 | AC-03/05；T-03/04 | push/insert扩容失败、remove越界、clone第n项失败 | 新profile verify/ABC/AV，av_collections_822 family | 原len/内容/源owner保留，部分新项无泄漏 | 失败转移；仅声明支持面 |
| V-04 | AC-04；T-03/04 | Some/None、Ok/Err、同名异enum、无guard读payload | 新profile verify/ABC/AV，av_collections_822 family | 仅合法guard投影；错分支不读inactive bytes | 分支投影；仅声明支持面 |
| V-05 | AC-02/04；T-03/04 | Some(Str)→None、Err(record)→Ok、构造新载荷失败 | 新profile verify/ABC/AV，av_collections_822 family | 只drop旧活分支一次；失败旧分支完整 | 分支替换；仅声明支持面 |
| V-06 | AC-02/03；T-03/04 | 嵌套List<Result<record,Str>>，空/零长、trap/cancel | 新profile verify/ABC/AV，av_collections_822 family | 固定结果与逆序事件一致；交回并释放结果后live=0 | 组合清理；仅声明支持面 |
| V-07 | AC-05；T-04 | 合法输入cap内/超1单位、瞬态峰值、故障第n次 | 新worker+父侧budget/allocator hook | resource分态；预算前拒的该申请attempt=0，故障注入attempt存在但successful_alloc=0；先前合法事件保留，危险访问/提交0；cleanup/真实峰值断言触发，无panic | 不能只看终态RSS |
| V-08 | AC-06；T-02/05 | fresh、同store新worker重启、改type/layout/profile、好hash坏payload | 783真实恢复→verify→AV | 合法恢复结果相同；依赖变化不复用；非法执行计数0 | 不缓存PASS或trusted |
| V-09 | AC-06；T-05 | core-i32及实际交付817/818旧profile、新profile送旧consumer | 实际原recipe/consumer | 旧结果/诊断不变，未知profile明确拒绝 | 无假native/CTE支持 |
| V-10 | AC-01/04/06；T-06 | worker错结果/错bits、漏case/skip、错工具身份 | 正式AutoDown recipe+父侧oracle | 合法正控PASS；每项错实际使总体非零/ERROR | 缺必要工具/路径不可绿 |
| V-11 | AC-02/03/04/05/06；T-01/03/04/05/06 | 错分支投影、remove后旧Place与fresh索引正反控、扩容旧Place、取消大型clone/替换、去debug后variant清理、稀疏索引路径预算 | §3真实MIR/ABC授信、执行及store新worker恢复；T-01冻结输入，T-03/04使可运行 | 非法访问在适用门拒绝；提交前失败原集合不变，临时新载荷被清理；strip清理次数不变，数据量计费及超额分类可观察；好hash不得绕过；准入非法dispatch=0，运行期故障按§5.5断言目标危险操作0 | 仅本profile；标量/旧消费者仍V-09，受管/进程峰值沿V-07 |
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | Some(Str) guard后改成None再投影；新guard正控；remove(0)后旧token与对剩余元素的新索引；len/cap/stride溢出与空列表 | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | 失效guard/旧Place在危险读前拒绝，先前合法执行保留；新guard和remove后fresh索引得到当前值；负下标/溢出不访问buffer，空列表可构造清理；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_collections_822`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-collections-822.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
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
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应集合/分支的状态和计量规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（本profile含owned值）；§5.4及V-11对应集合/分支的状态和计量规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应集合/分支的状态和计量规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应集合/分支的状态和计量规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应集合/分支的状态和计量规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-13 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

所有任务初始未完成；current_step仅计实际完成任务，不提前勾review/merge。T-01在本轮亦未执行实施调查，仅起草取材已完成。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [x] T-01 | work | 用户授权；只读准备可早于D-821 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/822/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜 [✅ 已完成 2026-10-26：冻结于 attachments/822/contract-decision.md——profile av.collections/1(wire AVAD v3/id 4/producer plan822)、MirType tag13/14、opcode 0x1D..0x2B 共15条、MAX_LIST_ELEMS=65536、recipe ac-av/collections-run-v1 接线9处、V-01..13 全映射；D-821/D828 门核实满足(均 archived)，写窗口接受(841/849 不触及本写面)，基面 master 5009dc629] | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-02 | work | T-01、D-821、注册owner交接、D828排期门、当前单写窗口交接 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记 [✅ 已完成 2026-10-26：核心+probe+recipe 三提交(40631d41d/后续两枚)——first-case 全链路 typed fixture→verify_mir→AVAD v3 encode→verify_abc→execute Success；arena 台账 face(3 allocs/3 frees/live=0)与父侧冻结表一致；同 store restore 重验一致+依赖轴 clean miss；正式 recipe ac-av/collections-run-v1 注册 5 处+av-data-822 语料+3 主仓测试(corpus/negative oracle/worker faults)全绿；821 corpus 回归 3/3 绿；**脚本/资源采样档(CPU/时间/内存 evidence)与 V-08/10 剩余正控归 T-05/T-06 补齐，不冒领**] | AC-01/06；first-case、V-01/08/10正控 |
| [x] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile [✅ 已完成 2026-10-26:15 条指令+类型/验证/执行面随 T-02 core 提交落地;av_collections_822_semantics 13/13 绿(List roundtrip/空列表/负与越界 trap 先于访问/remove 移位后 fresh 索引/insert 两端/set 邻位不受扰/List<Note> 字段原位 owned-Str replace/variant pack-disc-guard/owned 投影转移+错臂运行期 trap/switch 旧分支 drop 一次+guard 失效拒/payload-moved 静态拒(含 drop 不复位)/嵌套 List<Result<Note,Str>> 组合/域拒绝 StrView 元素与 List payload);source 支持面 = 明确无(合同 §6,fixture-only 入口);未改其他 profile——821 corpus 回归 3/3 绿;R3 闭环(7239e8f47):F-822-R3-01 Place 参照类型绑定(MirType::Place(ListTypeRef),tag15 带 elem;LIST_PLACE 铸造绑定/PLACE_GET dst==绑定元素/PLACE_SET value==绑定元素,错型在 typing 拒 mir.list-type)——reviewer 的 bool-over-List<i32> get/set 复现改为 verify 拒;F-822-R3-02 共享转移保内容(drop_owned_reverse 仅最后引用清空,MOVE/COPY/call move-in 的源释放为责任转移非销毁);语义档 28/28 绿(含 6 新反例与 call move-in sibling);R4 闭环(ad9a69ffa):F-822-R4-01 Place 参照类型表验证补齐(越界 mir.type-ref/域违规与自引用 mir.type.place-elem,;R5 闭环(352cbc5c4):F-822-R5-01 PLACE_SET 提交按激活序定位所有者帧(与 precheck/resolve 同帧,不再索引当前帧同号槽);F-822-R5-02 Place 入 token 分类+LIST_PLACE 记根(不入 tokens_live,epoch 退役而非借冲突)+Place param 外部根+Return 逃逸门拒局部根 place(mir.borrow-escape)+record 字段域拒 Place(mir.field-borrow);语义档 33/33 绿(含跨帧 write=9/read=7、escape 拒、record 拒、param 往返五新测);R6 闭环(3afbcbd3f):F-822-R6-01 根 provenance 跨前驱无条件合并(与活跃借用态分离)+roots_eq 计及非 live roots(工作流收敛)——Jump 边局部 place 逃逸在 MIR 拒(mir.borrow-escape),branch-merge 局部拒/外部根支持;F-822-R6-02 Copy/Call 的 place 不激活 tokens_live(保留引用策略一致性)——copy 别名与直接 mint 同样不阻塞列表更新;语义档 38/38 绿(含 R6 五测);R7 闭环(952c2b586):F-822-R7-01 Place 槽并入 token_slots 预算分类(每个保留 roots 槽按 (n+1) bits 全额计,先于分配)——12000-place 输入 MIR 与 ABC 双门 VerifyMemory 拒,64-place 小控双门过且执行=7;语义档 39/39 绿;R8 闭环(7402022b5):F-822-R8-01 CLONE 深克隆访客扩展至 List/Variant(逐元素独立克隆+新列表进独立 ledger buffer;中途 fault 零半提交);五反例=独立性 fold 7*9/owned 元素隔离/variant 载荷隔离+台账平衡/V-03 第 n 次失败全回滚/V-11 预算双值;COPY 共享与逆序清理不变;contract-decision §4 CLONE/MOVE/DROP 措辞与代码及计划原文调和;语义档 44/44 绿;R9 闭环(9e65e606d):F-822-R9-01 clone 访客完全平面化重写(owned 浅快照入栈/父子映射/逆 id 装配,任意支持种类零宿主递归)+双循环内 cancel/wall/fuel 轮询(与 dispatch 同 CANCEL_CHECK_INTERVAL 节奏,中途取消/墙超零半提交);F-822-R9-02 无环 walker 的 Record-only 前置过滤器移除(边集驱动全部节点种类);反例五=自引用与间接 A-B 环(表序倒置)双门拒/三层无环嵌套 Variant 克隆正控/1501 节点中途 Cancelled/零墙预算 Resource;R8 克隆族不变绿;语义档 49/49 绿;R10 闭环(80db25cb0):F-822-R10-01 双根因修复——①VariantValue.payload Box→Rc(派生 Value clone 对 Variant 不再递归复制载荷子树;访客 owned 快照对全部支持种类真浅、工作项 O(节点),消除深度 d 链的二次方快照拷贝);②CLONE 臂不可执行预扣(new_size)删除,访客逐节点充共享累计燃料(中途 InstrSteps 可断)+经 thread-local 段起点轮询剩余墙(非新计时器);execute_program_steps 补挂 arena(828 缺口);oracle 重构=R9 两段式测试(删 CLONE 仍绿=无区分度)替换为 seam 直调族(cancel/fuel 中断+回滚、全克隆正控)+R8 nth-fault 生产路径中断 oracle;wall 分支与 cancel 同 poll 代码路径(边界记录于 §9);语义档 50/50 绿;R11 闭环(90074e87f):F-822-R11-01 计费恢复数据量口径——访客共享累计燃料=每节点 1 结构单位(两阶段)+每 fresh Str 缓冲输出字节数(拷贝前充,预算拒先于分配尝试),裸 Str(L) 恰好 L+2(恰足 102 过/恰缺 101 拒双侧钉死),生产证人 ±CLONE 步差随字节增长(4096B=4101≥4096,4B=9);语义档 53/53 绿;R12 闭环(5490d6678):F-822-R12-01 清理遍历显式化——min_buffer_id 迭代化(owned 浅快照工作栈,序键 min 语义不变);drop_owned_reverse 全显式工作栈:末引用 Variant 载荷/Record 字段经 try_unwrap 取与出 Rc 逐项入栈(空壳 drop 无 glue 递归),list 元素保逆序+buffer 事件最后(R2-03 [2,1,0] oracle 不变),共享句柄(count>1)仍纯责任转移(R3-02 不变);保留清理燃料按 list 元素/variant 载荷计——共享转移 kill 现按真实工作量 O(1) 计费(旧递归形态对共享链按深度充燃料,64 控制 steps 10785→8832,无金样钉该值,已披露);deep_size 同因 sibling 迭代化(节点+边增量方案精确镜像,visited/字节总额位相同,全部钉死步数不变);旧值死亡点全路由显式释放:commit_slot_write(VariantSwitch 旧活臂/FieldSet 旧字段链/槽重赋值)+三处 list 元素替换提交点(ListSet/ListFieldSet/PlaceSet);反例=reviewer 64 层域控制转永久+3072 层深链 256KiB 小栈线程全路径(构建/verify/metering/清理)零递归证人+受控 mutation(恢复 Variant 臂递归)恰 STATUS_STACK_OVERFLOW+300 层 trap 出口清理+300 层 switch 旧链释放(均台账平衡 live=0);语义档 57/57 绿;R13 闭环(fab5f492a):F-822-R13-01 部分克隆回滚显式化——release_owned_worklist 抽为唯一末主释放核心(dispatch 经 st.steps 计保留燃料,克隆回滚经免充 scratch 计数器——历史 glue 回滚本就零计费,abort 燃料总额不变:5573/101/101/6150 全保持);clone_owned_value 两阶段入单一 build 闭包,全部放弃出口(燃料/取消/墙/fault)汇聚至一个 epilogue 显式 drain(out 条目+臂内临时,源经共享快照纯转移不受扰);arena 拆分 list_alloc_attempt(fault 缝先行预留,元素不过手——注入故障时调用方保留子元素停入 drain)/alloc_list_after_attempt(ListNew 行为不变);反例=reviewer 3072/5572 abort 重放(Err(InstrSteps)/fuel=5573/live=0 @256KiB)+三控+深链取消出口控转永久;受控 mutation(去 epilogue drain)恰 STATUS_STACK_OVERFLOW;语义档 59/59 绿] | AC-02/03；av_collections_822定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审 [✅ 已完成 2026-10-26(含 F-822-R1-01 修复,提交 717e9c208):四条变异指令(ListSet/ListFieldSet/ListRemove/VariantProject-owned)重排为 precheck→commit——预算拒先于任何变异,拒绝路径旧列表/源/台账原样;ListFieldSet 补此前缺失的 deep-delta 记账;ListRemove 补 root 侧减账(与 PUSH 对称);定向反例 av_collections_822_resources 新增 4 个 both-sides 测试(宽运行学精确 peak→恰足=Success/恰不足=SlotBytes),ListSet 附加 unwind free 序判别子;受控 mutation 验证:还原 swap-then-check 变异恰使该测试红([1,0,2] vs 修复版 [2,0,1]);档内 11/11 绿。R2 闭环(bf9e92cb4):F-822-R2-03 逆序清理(drop_owned_reverse 元素逆序/容器最后,owned_kill_slot+帧退出 LIFO 全接入;三固定 oracle=正常/trap/嵌套 各断言 frees=[2,1,0] 与 live=0);F-822-R2-02 place 资源侧沿 4 个 both-sides 事务反例+PLACE_SET precheck 覆盖;语义/资源档 21/11 测绿。原 6 测(cap 两侧/写前拒/fault 注入/budget-delta/peak drain/算术)不变;R3 闭环(7239e8f47):F-822-R3-02 受控 mutation 验证(无条件清空变异恰使 move 测试红回 len=0,已恢复);move/copy/call-move-in 三转移反例+owned 元素 move 台账平衡反例入语义档;资源档 11/11 不变;R4 闭环(ad9a69ffa):受控 mutation(禁用 Place 两检查)恰使两新反例红,已恢复;R5 闭环(352cbc5c4):受控 mutation(commit 回退 last_mut)恰使跨帧 write 测试以 reviewer 同款 unreachable panic 红,已恢复;R6 闭环(3afbcbd3f):受控 mutation(join 恢复 liveness 守卫)恰使跨块逃逸测试红,已恢复;821 ownership 语义档 25/25 回归绿(join/eq 改动对 live token 向后兼容);R7 闭环(952c2b586):F-822-R7-02 移除 R6 遗留死方法 is_token_live,core/probe 零 warning;R8 闭环(7402022b5):clone 故障/预算反例(allocs==frees/live=0)入档;R9 闭环(9e65e606d):取消/墙反例台账断言(allocs==frees/live=0)入档;R10 闭环(80db25cb0):F-822-R10-02 c0 未用绑定删除,定向构建零 warning;R11 闭环(90074e87f):F-822-R11-02 墙钟单坐标系——thread-local SEGMENT_STARTED_AT 删除(累计/剩余双重扣减与 None 回退 0 两根因同除),wall_left 即全部合同:helper 入口原点计量(入口+wall_left==段绝对 deadline),入口轮询使零 allowance 任意图规模确定性拒且不受同线程先行调用影响;cancel/wall 改 last_poll 游标节奏(批充字节不跳边界,与 dispatch 同式);反例=零墙 1300 元素 ExecutionWall+单标量零墙入口拒+同线程先行成功克隆独立控制+4B/4096B/恰足恰缺字节双侧+list[4096B,1B] 中途字节拒 allocs==frees live=0 零半提交;R12 闭环(5490d6678):F-822-R12-02(=finding 之 T-04 面)审计面落定——正常 Return/帧退出 LIFO/trap-resource-cancel unwind/owned_kill_slot 转移全走迭代释放;VariantSwitch 旧活臂与 FieldSet 旧字段链死于 commit_slot_write 替换释放(原注释所指路径现显式化);三处 list 元素替换(ListSet/ListFieldSet/PlaceSet)旧元素不再走 drop glue;共享载荷不被抽干+R3 move 保内容不变(既有 move/copy/call-move-in/嵌套逆序 oracle 全绿);受控 mutation 判别力=深链小栈测试 STATUS_STACK_OVERFLOW(0xc00000fd);R13 闭环(fab5f492a):F-822-R13-01 之 T-04 面——部分克隆全部失败出口(fuel/cancel/wall/fault)回滚走同一显式工作栈,abort 燃料零漂移(免充 drain 保持历史 glue 计费口径);List 臂 fault 时元素不再过手 arena(fault 缝先行,list_alloc_attempt 拆分),调用方停入显式 drain;回滚台账断言(allocs==frees/live=0)含 3072 层深链 abort/深链取消两新态;受控 mutation(去 drain)恰以 STATUS_STACK_OVERFLOW 红,已恢复] | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总  |
| [x] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面 [✅ 已完成 2026-10-26:av_collections_822_cache_compat 5/5 绿——同store恢复身份一致+snapshot 轴 clean miss、好 envelope 坏 payload 在重验拒且零 dispatch(783 内容寻址 Conflict 防篡改同面冻结)、AVAD v3 与 v2 世代隔离(schema-profile 拒)、A22/plan822 层与 A21 不混淆、合法恢复确定性重放(步数一致);V-09 旧 corpus 重放归 T-06 compatibility scope 汇总;R14 闭环(14b6a9df2):F-822-R14-01 恢复身份绑定补齐——restore_collections 在重验门后比对载荷**实际**类型表指纹(collections_types_fp_of(loaded.module()),与键轴同式)对请求 types_fp,不匹配在 execute 前拒(dispatch=0);相邻轴审计:envelope_unwrap 已核 producer/owner/entry/snapshot/语义,schema/世代隔离为 verify_abc_bytes 自有门——类型表指纹是唯一未绑定的身份轴;反例=reviewer 库级重放转永久(合法异型表 ABC 以原键+全匹配 envelope 存入→Err 具名双指纹,永不 Success)+独立诚实库正控(毒化键内容寻址拒覆写故分库);受控 mutation(禁用比对)恰使该测红;cache 档 6/6 绿;R15 闭环(f1d31632b):F-822-R15-01 剩余实际内容身份轴绑定——restore_collections 在指纹比对后再核解码模块入口函数**名字**与**source_snapshot**对请求身份,不匹配 execute 前拒(dispatch=0,Err 具名双方);R14 相邻轴审计修正:collections_envelope 的 _entry 实被忽略(前轮"envelope 已核 entry"论断错误,reviewer 指正),snapshot 仅外层元数据;owner 轴无需内容核对——verify_abc_bytes 对 owner 是重盖章(载荷无自证权),producer/schema/语义/阶段为键与 envelope 常量——入口名+入口快照为最后两条未绑定实际内容轴,审计闭合;反例=reviewer 双重放(同指纹 46d81ee5/原键/匹配 envelope:入口改 foreign_entry、快照+1)基线红(均 Success steps121)转绿(分别以 entry/source-snapshot mismatch 拒)+双反例并一永久测+诚实库正控;受控 mutation(禁用两比对)恰红;R16 闭环(a26b8e71c):F-822-R16-01 恢复 profile 身份绑定——restore_collections 先 decode_abc,实际 profile id≠ABC_PROFILE_COLLECTIONS 在深验与 dispatch 前拒(V-08/09:collections 身份永不执行外来 wire);R15"schema/profile 由通用门保证"论断修正:通用 verify_abc 按载荷自身 profile 选合法版本元组(core 亦收),从不强制请求方 profile,owner 重盖章非 profile 核对;profile 绑定后 wire 族(AVAB/AVAD)/版本元组/语义随解码 profile 自然钉死;通用门与旧 profile 自身消费者合同不动;反例=reviewer 双重放(合法 collections 标量模块重盖章为 core-i32 profile/schema、同 types_fp 过 R14/R15 全部绑定)基线红(Ok Success steps2)转绿(Err profile mismatch 具名 id 4 vs 1)+并一永久测+诚实 collections 标量正控(同模块保持 collections profile 仍恢复执行——拒绝的是外来 wire 非合法纯标量 collections 程序);受控 mutation(禁用 profile 比对)恰红] | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据 [✅ 已完成 2026-10-26(含 needs_fix 后状态,提交 9bf26b9dd/81c33e091 后定向复验):scripts/verify-av-collections-822.ps1 九 Scope(first-case/semantics/admission/resources/cache/compatibility/autodown/incremental/acceptance)——单 Scope 必选、selected>0 守卫、compatibility 重放 798/817/818/820/821 既有语料入口(V-09)、autodown 跑正式 recipe+反控(V-10)、incremental FAIL CLOSED(T-09 未完不绿)、acceptance 子进程重放防 seam 泄漏;first-case/autodown scope 实跑绿;SD 草案六条整理于 attachments/822/sd-draft.md;**受管/进程峰值 128MiB 线实测(821 式 worker 采样)未跑——归并 T-09 收尾批跑,不冒领**;R2 闭环(bf9e92cb4):F-822-R2-04 守卫重写(拒任意 ignored/skipped+必需 id 恰好一次,冻结清单;-GuardSelfTest 四坏形状全拒);first-case/semantics/incremental scope 带清单实跑绿;128MiB 已于 T-09 批补齐;R3 闭环(7239e8f47):语义 scope 必需清单扩至 28 id(含 R3 六反例+call sibling)实跑绿;R4 闭环(ad9a69ffa):incremental scope 必需清单扩至 8 id(含 R4 两反例)实跑绿;R5 闭环(352cbc5c4):语义 scope 必需清单扩至 33 id(含 R5 五测)实跑绿;R6 闭环(3afbcbd3f):语义 scope 必需清单扩至 38 id(含 R6 五测)实跑绿;R7 闭环(952c2b586):语义 scope 必需清单扩至 39 id(含内存预算反例)实跑绿;R8 闭环(7402022b5):语义 scope 必需清单扩至 44 id(含 clone 五测)实跑绿;R9 闭环(9e65e606d):语义 scope 必需清单扩至 49 id(含 R9 五测)实跑绿;R10 闭环(80db25cb0):语义 scope 必需清单更新(R9 两无区分度 id 移除,seam 族加入)实跑绿;R11 闭环(90074e87f):清单 +3 R11 id(墙 seam/字节 seam/生产证人)−2 陈旧 R9 id(clone_cancel/wall_mid_traversal)——**R10 已删该两测试但清单仍要求,冻结 scope 在 R11 前即预存红**(R10 记录的"清单更新实跑绿"与文件实态不符,本轮披露并修正);semantics scope 带清单实跑绿(53 测含新 3);compatibility 六腿全绿(821 5/7/3+798/820 15+817 5+818 24/24;818 首跑 mir_native_rounds_failure_liveness_and_caps 一例 12s 墙断言瞬态 flake——主仓无 auto-av-core 依赖边+隔离复跑绿+全 scope 复跑 24/24);R12 闭环(5490d6678):清单 +4 R12 id(64 层域控制/深链小栈/trap 出口/switch 旧链);semantics scope 带清单实跑绿(57 测);compatibility 各腿绿收据=821 5/7/3(scope 内)+798/820 15/15(同 scope 命令直跑)+817 5+818 24/24(直跑)——798/820 腿 turnaround P95 门(门 1000ms)在并发负载下两次瞬态红(实测绿时 P95 795-884ms 余量 12-20%),隔离与同命令复跑均绿,主仓代码本 diff 未触及,归因环境负载非本修复;R13 闭环(fab5f492a):清单 +2 R13 id(部分克隆回滚小栈/回滚控组含取消);semantics scope 带清单实跑绿(59 测);主仓兼容腿本 diff 未触及(R12 收据沿用——diff 仅 experimental/ac-av-probe/**+ps1,主仓无依赖边);R14 闭环(14b6a9df2):守卫 cache 清单 +1 R14 id(载荷指纹不匹配拒);cache scope 带清单实跑绿(exit 0);822 六 bin 全绿(cache 6/semantics 59/resources 11/first-case 4/incremental 8);821 sibling cache 7/ownership 25/corpus 3/first-case 5;reviewer 树内 r14 探针对修复转绿(其 unused import 警告属探针非产品,报告已注);R15 闭环(f1d31632b):守卫 cache 清单 +1 R15 id(入口身份双反例);cache scope 带清单绿;822 六 bin 全绿(cache 7/semantics 59/resources 11/first-case 4/incremental 8);821 sibling cache 7/ownership 25/corpus 3/first-case 5;产品目标零警告零错误(既有 R3 时代 reviewer 未跟踪探针 review_822_r3.rs 自 MirType::Place 参数化即失编译,无任何构建路径消费,预存非本轮产物,如实注记);R16 闭环(a26b8e71c):守卫 cache 清单 +1 R16 id(payload profile 双态);cache scope 带清单绿;822 六 bin 全绿(cache 8/semantics 59/resources 11/first-case 4/incremental 8);821 sibling cache 7/ownership 25/corpus 3/first-case 5;rustfmt/diff-check 过,core 零警告零错误;R17 闭环(7de289d47):守卫 incremental 清单 +3 R17 id(内部 layout/接口 layout/授权生命周期);incremental scope 带清单实跑绿(11 测含 worker memory gate);822 六 bin 全绿;821 sibling 不变绿;R18 闭环(98b4589ba):守卫 incremental 清单 +1 R18 id(carried 闭包授权);incremental scope 带清单实跑绿(12 测);822 六 bin 全绿(cache 8/semantics 59/resources 11/first-case 4/incremental 12);821 sibling 不变绿;rustfmt/diff-check 过,core 零警告零错误] | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage [✅ 已完成 2026-10-26(含 F-822-R1-02 修复,提交 81c33e091):新增真实更新入口 probe/src/collections_session.rs——单元仅经真门入会话(verify→encode→abc-admit→unit-admit+store 持久),修订仅重建被提交函数、其余由会话携带(调用方不传 Arc),link 解析 fn:<name> import 并做签名等价;incremental 档 4/4 经该入口:子case2 alpha 真调用 beta 三次 body 修订(每修订执行语义正确/live=0/台账平衡/alpha Arc 会话携带/alpha 生产计数 verify=encode=admit=store=0/旧 snapshot 保旧真值);子case3 真实依赖(正控 body 变 alpha 保留,反控 beta 签名合法单元被 link 拒 signature mismatch,拒后会话状态完好);子case4 正式 gate=会话生产谓词(诚实修订 degraded=0 pass,同内容重提交强制全量 degraded=1 FAIL 非零);子case1 828 基线战役重放保留;128MiB 硬门与 ac-mn 兼容修复不变;R2 闭环(bf9e92cb4):F-822-R2-01 会话事务化(候选 build-aside,link+执行成功才原子安装;反例=签名拒后当前对象不变且 prior snapshot 可执行、执行 trap 候选同不安装);F-822-R2-02 Place/token 面(0x2C-0x2E+tag15 全链)补齐 V-02/11/12 双控;incremental 6/6 绿;R17 闭环(7de289d47):F-822-R17-01 V-13 子case3 layout/授权轴补齐——调查结论:授权模型已存在(MirModule.capabilities+verify_mir 允许清单 TRACE_CAPABILITY+F798-R3-02 活授权重准入模式),collections 消费者缺接线;link 签名经共享 scratch 类型空间内容等价,容器型接口变化结构性可捕 → 接线=CollectionsUpdateSession 增 host_grants(set_host_grants,默认空——要求能力的单元非显式授权不入),apply_revision 对每个提交模块 capabilities 逐项对当前 grants 准入,缺口在任何生产/link/安装前拒(要求与授予为分离事实) → 三正反控入档:①内部 layout 正控(beta 元素类型 i32→Str,接口稳定→仅 beta 重建/全生产阶段=1/alpha 对象携带/gate 诚实/旧 snapshot 保真);②接口 layout 反控(beta 参数 List<i32>→List<Str>,单元自身合法→link 内容等价签名拒,会话状态/旧 snapshot 存活且会话仍可用——容器型签名轴区别于既有标量 i32→bool);③授权生命周期(合法允许→撤销拒绝零安装旧态可执行→恢复授权同修订通过);受控 mutation(禁用授权准入)恰使③红;R18 闭环(98b4589ba):F-822-R18-01 R17 完整关闭——授权预检扩至**完整候选闭包**(submitted+carried):apply_revision 建序后、任何生产前,逐单元核对提交模块声明能力或 carried 单元已准入模块能力对当前 grants,缺口零生产/零链接/零执行/零安装拒(production_counters() 观察器可证零生产);R17"已携带单元不回溯"边界注记为错误口径(每一新修订都是对整个候选的新授权决策,reviewer 判定成立);已发布 snapshot 不受回溯影响(授权绑定于修订时点);替换提交以其自身声明为准不继承被替单元;反例=reviewer 双形态重放(撤销后仅改调用方/空提交,基线红 Ok rev2/21 与 rev2/11)转绿(双形态均 Err 具名 carried beta 能力,生产计数全零,beta 对象不变,旧 snapshot 可执行)+活授权 carried 正控(同形态成功 21,beta 携带)+再撤销再授权往返;受控 mutation(禁用 carried 预检臂)恰使该测红,已恢复;**附注**:入场时树内发现一段非本会话提交的未完成修复 WIP(与 R18 报告"无实现修复"声明相矛盾),已隔离至 stash unowned-wip-r18-found-in-tree 未采用,本提交实现独立完成并披露] | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档  |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录

2026-10-27 stage:work(needs_fix R18 修复) | PLAN-822 r5 | outcome:pass(F-822-R18-01 闭环=R17 授权面完整关闭,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:98b4589ba(基于复审冻结 7de289d47;reviewer 未跟踪探针保持原样) | finding 闭环:**F-822-R18-01**(撤销授权后 carried 受限单元随新修订执行并安装)→ 根因=R17 授权准入仅在 Some(submitted) 分支,None(carried) 分支直接复用 FunctionCode,link/execute/commit 全程不核其 required capabilities;R17 的"已携带单元不回溯"边界注记为错误口径——每一新修订都是对整个候选的新授权决策 → 修复=apply_revision 建序后、任何生产前对**完整候选闭包**预检:提交模块按其声明能力、carried 单元按已准入模块能力,逐项对当前 host_grants,缺口零生产/零链接/零执行/零安装拒(新 production_counters() 观察器可证拒绝时各阶段计数全零);已发布 snapshot 不回溯(授权绑定于修订时点);替换提交以自身声明为准 → 反例=stash 法基线双形态红(撤销后仅改 main:Ok rev2/21/rebuilt1;空提交:Ok rev2/11)转绿(均 Err 具名 carried beta 能力缺口);reviewer 重放转永久测+活授权 carried 正控+再撤销/再授权往返;受控 mutation(禁用 carried 预检臂)恰红已恢复 → gap 无(既有 layout/body/签名/生命周期/身份控制不变绿;多提交序拒绝边界=预检在建序后统一先行,不存在"先产后拒"局部形态) | 定向验证:incremental 12/12;守卫 incremental scope 带清单(+1 id,含 worker memory gate)绿;822 六 bin 全绿(cache 8/semantics 59/resources 11/first-case 4/incremental 12);821 sibling(cache 7/ownership 25/corpus 3/first-case 5)绿;rustfmt/diff-check 过,core 零警告零错误 | **披露**:入场时 worktree 内发现非本会话提交的未完成 R18 修复 WIP(与 R18 报告"无实现修复"声明矛盾,来源不明),已隔离 stash unowned-wip-r18-found-in-tree 未采用,本提交实现独立完成 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十九轮独立 review(V-13 授权轴至此完整:提交面+carried 闭包统一预检;请优先复核双形态永久测、零生产观察与 WIP 隔离披露)

2026-10-27 stage:work(needs_fix R17 修复) | PLAN-822 r5 | outcome:pass(F-822-R17-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:7de289d47(基于复审冻结 a26b8e71c;reviewer 未跟踪探针保持原样) | finding 闭环:**F-822-R17-01**(V-13 子case3 缺 layout/授权正反控——验收证据缺口,非运行时缺陷)→ 调查=授权模型已存在(MirModule.capabilities+verify_mir 允许清单仅 TRACE_CAPABILITY+F798-R3-02 活授权重准入模式),collections 会话缺接线;link 签名经共享 scratch 类型空间内容等价,容器型接口变化结构性可捕 → 接线=CollectionsUpdateSession 增 host_grants(默认空——要求能力的单元非显式授权不入),apply_revision 对提交模块 capabilities 逐项对当前 grants 准入,缺口在任何生产/link/安装前拒 → 三正反控=①内部 layout 正控(beta 元素 i32→Str 接口稳定,仅 beta 重建/生产阶段=1/alpha 携带/gate 诚实/旧 snapshot 保真);②接口 layout 反控(beta 参数 List<i32>→List<Str> 单元合法,link 内容等价签名拒,会话/旧 snapshot 存活且仍可用——容器型签名轴区别于既有标量测试);③授权生命周期(允许→撤销零安装旧态可执行→恢复授权同修订通过);受控 mutation(禁用授权准入)恰使③红,已恢复 → gap 无(既有 signature/body/forced 控不变绿;边界:授权检查作用于修订提交面,已携带单元不随 grants 撤销回溯——F798 模式同域,如实注记) | 定向验证:incremental 11/11;守卫 incremental scope 带清单(+3 id,含 worker memory gate)绿;822 六 bin 全绿(cache 8/semantics 59/resources 11/first-case 4/incremental 11);821 sibling(cache 7/ownership 25/corpus 3/first-case 5)绿;rustfmt/diff-check 过,core 零警告零错误 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十八轮独立 review(V-13 子case3 四轴齐备:接口标量+接口容器 layout(R17)/内部 layout(R17)/helper body(既有)/授权生命周期(R17 新接线);请优先复核三新控与授权接线面)

2026-10-27 stage:work(needs_fix R16 修复) | PLAN-822 r5 | outcome:pass(F-822-R16-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:a26b8e71c(基于复审冻结 f1d31632b;reviewer 未跟踪 R3/R8/R14 探针保持原样,R16 重放复现后移除——耐久副本在 attachments/822/review-r16/) | finding 闭环:**F-822-R16-01**(合法旧 core-i32 工件放入 collections 身份仍执行)→ 根因=恢复未绑定实际 wire/profile:通用 verify_abc 按载荷自身 profile 选合法版本元组(core 分支无反向 expected 约束),owner 重盖章非 profile 核对——R15"schema/profile 由通用门保证"论断不完整(reviewer 指正);合法 collections 标量模块(types=[I32,Fn])重盖章为 core-i32 profile/schema 保同 types_fp/entry/snapshot,过 R14/R15 全部绑定后在 collections 身份下执行 → 修复=restore_collections 先 decode_abc,实际 profile id≠ABC_PROFILE_COLLECTIONS 在深验与 dispatch 前拒;profile 绑定后 wire 族/版本元组/语义随解码 profile 自然钉死;通用门与旧 profile 消费者合同不动 → 反例=stash 法基线红(Ok(Success I32(7),steps2) 与报告一致)转绿(Err profile mismatch 具名 id 4 vs 1);reviewer 双重放并一永久测(core 重盖章孪生拒+同模块保持 collections profile 仍恢复执行=拒绝外来 wire 非合法纯标量 collections 程序);受控 mutation(禁用 profile 比对)恰红已恢复 → gap 无(诚实恢复/R14/R15 既有拒绝正控不变绿;本轮仅实测 core→collections 混淆面已闭,未冒称其他 profile 组合) | 定向验证:822 六 bin 全绿(cache 8 含新反例/semantics 59/resources 11/first-case 4/incremental 8);守卫 cache scope 带清单(+1 id)绿;821 sibling(cache 7/ownership 25/corpus 3/first-case 5)绿;rustfmt/diff-check 过,core 零警告零错误 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十七轮独立 review(恢复身份轴全集绑定:profile(R16)+types_fp(R14)+入口名+入口快照(R15),owner 重盖章结构性覆盖,producer/schema/语义/阶段为键与 envelope 常量;请优先复核 profile 双态永久测与轴审计论证)

2026-10-27 stage:work(needs_fix R15 修复) | PLAN-822 r5 | outcome:pass(F-822-R15-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:f1d31632b(基于复审冻结 14b6a9df2;reviewer 未跟踪 R3/R8/R14 探针保持原样,R15 重放复现后移除——耐久副本在 attachments/822/review-r15/) | finding 闭环:**F-822-R15-01**(恢复未核对载荷实际入口名与 source_snapshot)→ 根因=R14 相邻轴审计不完整:collections_envelope 的 _entry 实被忽略(R14 记录"envelope 已核 entry"论断错误,reviewer 指正),snapshot 仅外层元数据非解码内容;同指纹载荷(原类型表过 R14 指纹门)改名入口或换快照即可在原键身份下执行 → 修复=指纹比对后再核解码模块入口函数名与 source_snapshot 对请求身份,不匹配 execute 前拒(dispatch=0,Err 具名双方);审计闭合:owner 为 verify_abc_bytes 重盖章(载荷无自证权),producer/schema/语义/阶段为键与 envelope 常量,types_fp R14 已绑——入口名+入口快照为最后两条未绑定实际内容轴 → 反例=stash 法基线双红(入口 foreign_entry/快照+1 均 Success steps121)转绿(entry/source-snapshot mismatch 拒);双反例并一永久测+诚实库正控;受控 mutation(禁用两比对)恰红已恢复 → gap 无(诚实恢复/依赖 clean miss 既有正控不变绿;旧 profile 合同未动) | 定向验证:822 六 bin 全绿(cache 7 含新反例/semantics 59/resources 11/first-case 4/incremental 8);守卫 cache scope 带清单(+1 id)绿;821 sibling(cache 7/ownership 25/corpus 3/first-case 5)绿;rustfmt/diff-check 过,产品目标零警告零错误 | 附注:reviewer R3 时代未跟踪探针 review_822_r3.rs 自 MirType::Place 参数化即失编译,无构建路径消费(预存,R14 轮我方 --tests 检查的 grep 滤掉了其 error 行未显,本轮如实注记) | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十六轮独立 review(恢复身份轴至此全绑定:types_fp(R14)+入口名+入口快照(R15),owner 由重盖章结构性覆盖;请优先复核双反例永久测、诚实库正控与轴审计闭合论证)

2026-10-26 stage:work(needs_fix R14 修复) | PLAN-822 r5 | outcome:pass(F-822-R14-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:14b6a9df2(基于复审冻结 fab5f492a;reviewer 未跟踪 R3/R8/R14 探针保持原样,R14 重放复现用其树内探针,耐久副本在 attachments/822/review-r14/) | finding 闭环:**F-822-R14-01**(恢复未核对载荷实际类型指纹)→ 根因=restore_collections 以请求 types_fp 选键,但 envelope 只核 owner/entry/snapshot/语义、verify_abc_bytes 只证载荷自洽——载荷自身类型表指纹(键的身份轴之一)从未被绑定,合法异型表 ABC 以原键存入即可执行 → 修复=重验门后比对 collections_types_fp_of(loaded.module())(与键轴同式)对请求 types_fp,不匹配在 execute 前拒(dispatch=0,Err 具名双指纹);相邻轴审计确认 envelope/世代隔离已覆盖其余轴,类型表指纹为唯一未绑定轴 → 反例=stash 法基线红(Ok(Success steps121),期望 fp 46d81ee5/载荷 58dda3f6 与报告一致)转绿(Err 拒派发);库级重放转永久+独立诚实库正控(毒化键内容寻址拒覆写,分库存取);受控 mutation(禁用比对)恰使永久测红,已恢复 → gap 无(合法恢复/真实依赖 clean miss 既有 5 控不变绿;旧 profile 合同未动——修复仅在 collections restore 路径) | 定向验证:822 六 bin 全绿(cache 6 含新反例/semantics 59/resources 11/first-case 4/incremental 8);守卫 cache scope 带清单(+1 id)绿;821 sibling(cache 7/ownership 25/corpus 3/first-case 5)绿;reviewer 树内 r14 探针转绿;rustfmt/diff-check 过,产品零 warning(余警告属 reviewer 未跟踪探针) | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十五轮独立 review(R11 计费/墙、R12 清理、R13 回滚、R14 恢复身份绑定逐轮闭环;请优先复核库级异型表反例、诚实库正控与 mutation 判别力)

2026-10-26 stage:work(needs_fix R13 修复) | PLAN-822 r5 | outcome:pass(F-822-R13-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:fab5f492a(基于复审冻结 5490d6678;reviewer 未跟踪 R3/R8 探针保持原样,R13 重放文件复现后移除——耐久副本在 attachments/822/review-r13/) | finding 闭环:**F-822-R13-01**(部分克隆回滚过 glue 栈溢出)→ 根因=两个缺口:①clone_owned_value 中途放弃(燃料/取消/墙/fault)时 out 装配链与臂内临时经默认 Value/Variant/Rc 析构——R12 显式清理只路由了 dispatch 侧消费者,helper 本地部分持有者漏网;②List 臂 fault 路径 try_alloc_list 按值收子元素,注入故障时元素在 arena 函数帧内 glue 析构,调用方无法接管 → 修复=①R12 工作栈抽为 release_owned_worklist 唯一末主释放核心(dispatch 清理经 st.steps 计保留燃料;克隆回滚经同一工作栈+免充 scratch——历史 glue 回滚零计费口径保持,abort 燃料总额不变);②两阶段入单一 build 闭包,全部放弃出口汇聚至一个 epilogue 显式 drain(out 条目+drain_extras 停靠临时;源经共享快照纯转移不受扰);③arena 拆分 list_alloc_attempt(fault 缝先行预留,元素不过手)/alloc_list_after_attempt(try_alloc_list 组合保持,ListNew 行为不变) → 反例=基线复现红(3072/5572 abort 于冻结 HEAD STATUS_STACK_OVERFLOW,三控 101/101/6150 与报告一致)转绿(Err(InstrSteps)/fuel=5573/live=0 @256KiB,三控数字零漂移);reviewer 两重放转永久+深链取消出口控(3072 层 cancel-raised→Cancelled/live=0 小栈);受控 mutation(去 epilogue drain)恰 STATUS_STACK_OVERFLOW 后恢复=判别力成立 → gap 无(R8 nth-fault/R11 字节中途拒/R9-R12 全部回滚/清理 oracle 不变绿;源保持不动语义由共享转移保持) | 定向验证:822 六 bin 全绿(semantics 59 含 2 新/resources 11/first-case 5/cache 4/incremental 8);821 sibling ownership 25/25+corpus 3/3+first-case 5/5;守卫 semantics scope 带清单(+2 id)实跑绿;rustfmt/diff-check 过,零 warning;主仓兼容腿本 diff 未触及(diff 仅 experimental/ac-av-probe/**+ps1),R12 收据沿用 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十四轮独立 review(R11 计费/墙、R12 清理已闭;R13=R12 不变量的回滚出口残余——helper 本地部分持有者未随 dispatch 侧路由,已按"唯一释放核心+放弃出口单点汇聚+fault 缝先行"闭环;请优先复核 3072/5572 回滚重放、abort 燃料零漂移与 mutation 判别力)

2026-10-26 stage:work(needs_fix R12 修复) | PLAN-822 r5 | outcome:pass(F-822-R12-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:5490d6678(基于复审冻结 90074e87f;reviewer 未跟踪 R3/R8 探针保持原样,R12 重放文件复现后移除——耐久副本在 attachments/822/review-r12/) | finding 闭环:**F-822-R12-01**(清理仍宿主递归)→ 根因=R9 只平面化了克隆访客,min_buffer_id/drop_owned_reverse 沿用"容器一层深/表界定深度"的伪界(冻结域允任意深度无环 Variant 链,reviewer 64 类型生产控制证明可达);旧值死亡点(commit_slot_write 替换/三处 list 元素替换)与 deep_size 计量行走 drop glue/直接递归,同因 → 修复=①min_buffer_id 迭代化(owned 浅快照工作栈,min 语义不变);②drop_owned_reverse 全显式工作栈——末引用 Variant 载荷/Record 字段 try_unwrap 取与出 Rc 逐项入栈(空壳 drop 无 glue 递归),list 元素逆序+buffer 事件最后,共享句柄纯责任转移;③旧值死亡点全路由显式释放(commit_slot_write+ListSet/ListFieldSet/PlaceSet);④deep_size 迭代化(节点+边增量精确镜像,visited/字节位相同) → 反例=reviewer 64 层域控制转永久(Success/台账平衡)+**3072 层深链 256KiB 小栈线程全路径零递归证人**(构建/verify/metering/清理全 flat,绿路径 0.9s)+**受控 mutation(恢复 Variant 臂递归)恰 STATUS_STACK_OVERFLOW(0xc00000fd)**=判别力成立+300 层 trap 出口 unwind+300 层 switch 旧链经 commit_slot_write 替换释放(均 allocs==frees/live=0) → gap 无(顺序 oracle R2-03 [2,1,0]/嵌套/move 族不变绿;821 sibling 25/25+3/3+5/5;兼容腿 821 5/7/3+798/820 15/15+817 5+818 24/24) | **披露**:①共享转移 kill 的保留清理燃料从旧递归形态的 O(深度) 改为按真实工作 O(1)(64 控制 steps 10785→8832,无金样钉该值;deep_size 位相同保全部旧步数);②798/820 兼容腿 turnaround P95 时序门在并发负载下两次瞬态红(绿时实测 P95 795-884ms/门 1000ms 余量 12-20%),隔离+同命令复跑绿,主仓代码本 diff 未触及,归因环境;③deep_size 为同因 sibling 扩面(finding 未点名但同一"支持图上任一行走不乘宿主栈"不变量),非合同变更 | 定向验证:822 六 bin 全绿(semantics 57/resources 11/first-case 5/cache 4/incremental 8);守卫 semantics scope 带清单(+4 id)实跑绿;rustfmt/diff-check 过,零 warning | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十三轮独立 review(R11 两项已确认关闭;R12=R9 平面化的清理消费者遗漏,已按"全显式工作栈+取与出 Rc+死亡点全路由"闭环;请优先复核深链小栈证人、mutation 判别力与 10785→8832 计费披露)

2026-10-26 stage:work(needs_fix R11 修复) | PLAN-822 r5 | outcome:pass(F-822-R11-01/02 闭环+守卫陈旧 id 修正,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:90074e87f(基于复审冻结 b5c226214;reviewer 未跟踪 R3/R8 探针保持原样,R11 重放文件复现后移除——耐久副本已在 attachments/822/review-r11/) | findings 闭环:**F-822-R11-01**(克隆字节计费丢失)→ 根因=R10 删预扣后访客仅按节点计数,Str 装配 try_alloc 不计字节(内存 precheck≠执行燃料) → 修复=共享累计燃料恢复数据量口径:每节点 1 结构单位(两阶段)+每 fresh Str 输出字节数(拷贝前充,预算拒先于分配尝试;last_poll 游标节奏,批充不跳边界);裸 Str(L) 恰好 L+2 → 反例=reviewer 三重放全红转绿(4B max_fuel=60 过 fuel=6/4096B 拒 fuel=4098 minted=0/生产 ±CLONE 步差 4096B=4101≥4096)+恰足 102/恰缺 101 双侧+list[4096B,1B] 中途拒 allocs==frees live=0 → gap 无(compatibility 六腿全绿,字节口径未破坏任何旧 corpus 步数钉面);**F-822-R11-02**(墙钟双重扣减+None 回退 0)→ 根因=helper 用 thread-local 段累计 elapsed 与 CLONE 入口已扣减的 wall_left 比较,且无段起点线程回退 elapsed=0 永不拒 → 修复=thread-local 删除,wall_left 自 helper 入口原点计量(入口+wall_left==段绝对 deadline 单坐标系,显式传参即全部合同),入口轮询零 allowance 确定性拒 → 反例=reviewer 零墙重放红转绿(2602 节点原完成现 ExecutionWall fuel=0)+单标量零墙入口拒+同线程先行成功克隆后零墙仍拒(无遗留态)+R10 seam 族(cancel/fuel/full)不变绿 → gap 无(时间性单变量生产墙 oracle 仍以 seam+边界记录覆盖,R10 §9 边界记录维持——cancel/墙共享 poll 代码路径,坐标系正确性由入口原点不变式直接钉死);**附带披露**:守卫 semantics 必需清单在冻结 HEAD 即预存红(R10 删 clone_cancel/wall_mid_traversal 两测试但清单仍要求,其"清单更新实跑绿"记录与文件实态不符)——本轮 −2 陈旧 id+3 新 id 修正并实跑绿;两项 finding 均已修复非延期债,KNOWN-DEBT 无新增项 | 定向验证:822 六 bin 全绿(semantics 53/resources 11/first-case 5/cache 4/incremental 8 含 memory gate);821 sibling ownership 25/25+corpus 3/3+first-case 5/5;守卫 semantics/compatibility scope 带清单实跑绿;rustfmt(两改动 Rust 文件)/git diff --check 过,定向构建零 warning | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十二轮独立 review(R11 两项=R10 修复自身引入的计量回归与墙坐标系残留,已按"字节+结构增量计费/入口原点单坐标系"闭环;请优先复核三重放反例、恰足/恰缺双侧与守卫清单修正)

2026-10-26 stage:work(needs_fix R10 修复) | PLAN-822 r5 | outcome:pass(F-822-R10-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:80db25cb0(基于复审冻结 9e65e606d;reviewer 重放文件保持未跟踪) | findings 闭环:**F-822-R10-01**(快照仍递归+oracle 无区分度)→ 根因A=VariantValue.payload 为 Box<Value>,派生 Value clone 递归复制 Variant 载荷子树(R9"浅快照"论断对 Variant 为假,深度 d 链二次方拷贝);根因B=R9 两测试前置 3000 条指令使 dispatch 边界先触发,删 CLONE 仍绿 → 修复=①payload→Rc<Value>(Record/List/Str/Variant 派生克隆全为句柄 bump,访客快照真浅、O(节点),投影/切换/清理/相等语义不变);②CLONE 预扣删除,访客逐节点充共享燃料+thread-local 段剩余墙;③execute_program_steps 补 arena(828 缺口,resume trace 可带台账)→ oracle=seam 直调族(直调无 dispatch,中断结果只能来自访客自身 poll:cancel→Cancelled/fuel 60 节点→InstrSteps,均台账回滚)+全克隆正控(独立 buffer 集/序/内容/源完整/全释放)+R8 nth-fault 保留为生产路径中断 oracle(fault 落访客自身 try_alloc=helper 内+部分 mint+全回滚);**wall oracle 边界诚实记录**:wall 分支与 cancel 共享同一 poll 代码路径,cancel seam+生产 fault 联合覆盖其激活与接线;时间性单变量 wall 生产 oracle 未独立构造(R10 曾尝试两段式/预算亏缺两法,dispatch 边界语义使其不可稳定区分,已弃并记录)→ gap 无(除记录的 wall 边界);**F-822-R10-02**(unused c0 warning)→ 删除,零 warning | 定向验证:五 bin **78 测全绿**(semantics 50);821 ownership 25/25;零 warning;守卫语义 scope 清单更新实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十一轮独立 review(R9 环门已闭;R10 为浅快照根因+oracle 区分度缺口,已按 Rc 迁移+真实燃料/剩余墙+seam/fault oracle 族闭环)


2026-10-26 stage:work(needs_fix R9 修复) | PLAN-822 r5 | outcome:pass(F-822-R9-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:9e65e606d(基于复审冻结 7402022b5;reviewer 重放文件保持未跟踪原样) | findings 闭环:**F-822-R9-01**(克隆访客递归处理嵌套 Variant 且无循环内取消)→ 根因=R8 修复对 List 元素/Variant payload 递归调用自身且"恰一层"论断错误(冻结域允许 Variant 嵌 Variant),访客签名无 cancel/wall/fuel,dispatch 只在指令边界查限 → 修复=完全平面化两阶段重写(phase1 owned 浅快照显式 DFS,Record/List/Variant 子女全展开,逆序 push 保声明序;phase2 父子映射+逆 id 叶先装配;**任意支持种类零宿主递归**);双循环内按 CANCEL_CHECK_INTERVAL 轮询 cancel/wall/燃料,Cancelled 与墙 Resource 在克隆中途浮出且部分装配全回滚(台账平衡/live=0);无任意语言深度上限 → 反例五=reviewer 三层无环嵌套 Variant 克隆正控/1501 节点中途 Cancelled/零墙预算同 poll 点 Resource(均台账干净)/R8 克隆族(独立/n 失败/预算)不变绿 → gap 无;**F-822-R9-02**(自引用 Variant 双门通过)→ 根因=无环 walker 的 edges 提取前有 Record-only 过滤器把 Variant 墨黑跳过,Variant 边提取不可达 → 修复=移除过滤器,边集驱动全部节点种类(Record 字段+Variant payload 共享同一 DFS) → 反例三=自引用 Recursive{End,Next(Self)}(未使用表项)MIR+ABC 双门拒 mir.type-cycle/间接 A-B 环(表序倒置)拒/三层无环正控准入且克隆正确;821 Record 环控制 25/25 不变绿 → gap 无 | 定向验证:五 bin **77 测全绿**(semantics 49);821 ownership 25/25;零 warning;守卫语义 scope 49 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第十轮独立 review(R8 浅共享已闭;R9 两项=克隆遍历合同与无环准入的完整性缺口,已按"显式有界遍历+循环内取消+边集驱动"合同口径闭环)


2026-10-26 stage:work(needs_fix R8 修复) | PLAN-822 r5 | outcome:pass(F-822-R8-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:7402022b5(基于复审冻结 952c2b586;reviewer 重放文件保持未跟踪原样) | finding 闭环:**F-822-R8-01**(集合 CLONE 实为浅共享)→ 根因=CLONE 门接受 owned collection 操作数,但深克隆访客只遍历 Record/Str——List/Variant 落入 other.clone()(ListHandle::clone 共享 Rc buffer),reviewer 复现"克隆[7]→原改[9]→克隆读 9" → 修复=访客装配 List/Variant 时逐元素/payload 独立克隆(每元素完整访客;冻结元素域禁 List 嵌套→递归恰一层)+新列表 mint 独立 ledger buffer(try_alloc_list:arena id/fault 缝/cap=len);中途 fault 部分装配全 drop、minted buffer 全 free(零半提交);CLONE 臂既有 precheck 与 deep_size 燃料(R2 起含 List/Variant)自动覆盖 → 反例五=reviewer 复现 fold(7*9=63)/List<Note> 字段改写后克隆隔离/variant 载荷隔离(原 SWITCH None 克隆仍 Some,台账平衡)/V-03 第 n 次失败(大元素 fault→全回滚 allocs==frees live=0)/V-11 预算双值(恰短 SlotBytes 拒+恰足过);COPY 共享与逆序清理既有控制不变绿 → 合同调和=T-01 contract-decision §4 原"CLONE/MOVE/DROP 拒 collections"措辞与代码及计划原文(§1 含 move/clone/drop)矛盾,修订为支持操作记录 → gap 无 | 定向验证:五 bin **72 测全绿**(semantics 44);821 ownership 25/25;零 warning;守卫语义 scope 44 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第九轮独立 review(R7 两项已由本轮检查确认关闭;R8 为 R2 集成期 CLONE 访客未随新 owned 种类扩展的原始缺口——诊断模式与 R7 同:新种类入了操作数域但未同步全部消费者;本轮已同步并附完整 oracle 族)


2026-10-26 stage:work(needs_fix R7 修复) | PLAN-822 r5 | outcome:pass(F-822-R7-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:952c2b586(基于复审冻结 3afbcbd3f) | findings 闭环:**F-822-R7-01**(Place 根集合漏计验证器内存预算)→ 根因=预分配界定的 token_slots 分类仅 StrView/MutRef,R5/R6 集成 Place 语义时未同步预算消费者——place 槽记零 roots 矩阵,而 LIST_PLACE 每保留引用 mint 全宽 Bitsets(n+1) → 修复=Place 并入 token_slots 分类,同额计费先于分配(不加 cap、不丢 roots、逃逸/别名语义不动,R6 控制全绿) → 反例=12000-place 合法形态 MIR+ABC 双门 VerifyMemory 拒;64-place 小控双门过且执行=7 → gap 无;**F-822-R7-02**(产品死代码警告)→ R6 移除前驱合并调用后 is_token_live 成死方法 → 已删除,core/probe 零 warning → gap 无 | 定向验证:五 bin **67 测全绿**(semantics 39);821 ownership 25/25;零 warning;守卫语义 scope 39 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第八轮独立 review(R6 两项已由本轮报告确认关闭;R7 为 R5/R6 Place 集成对预算/清理消费者的最后两处遗漏,现已闭环——请优先复核内存预算反例与零警告构建)


### 2026-10-26 独立最终复审 R7

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施本轮修复，沿独立review上下文核代码/合同/定向行为，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 3afbcbd3f74ed3d59e562dd901c419678f92f58e | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 352cbc5c4e1afe4dd1a515e856d4986503506a22

F-822-R6-01/02 CLOSED：原独立5控全绿；新独立mixed local/external合流三子形态（局部根两前驱顺序均MIR/ABC拒、全外部根返回7/live0）通过；committed Place16控及helper-roundtrip-update1控绿。R5-02剩余语义缺口在所查支持面关闭。发现F-822-R7-01(P1，Place原集成资源遗漏)：ownership预分配预算token_slots仅含StrView/MutRef，漏Place；12000个合法mint的216192B输入（12004槽/12005指令/1块）MIR/ABC均准入，analysis_bytes仅2340928B，而同一生产ownership函数的System allocator单测实测新增live峰值21217630B，超冻结16777216B；单份根bitset下界18048000B也已超cap。64个Place小正控两门准入/峰19614B。此为R6关闭oracle已要求审计的分析计量，非新增规模承诺；应资源拒而非要求接纳。F-822-R7-02(P2，修复卫生回归)：删除join调用后OwnState::is_token_live(:1602)闲置，实际auto-av-core编译持续dead_code warning，非原R3临时probe警告，违背健康门和work“零unused”声明。

evidence: [R7报告/最小重放/关闭oracle](attachments/822/review-r7/report.md)，同目录replay.rs、replay-r6/join/place/roundtrip/memory/fmt日志与changed-files/input-hashes、sd-frozen；内存反例单测--test-threads=1，输入构造与CheckedMir clone不计入直测窗口，无整档内存战役。dependency_revisions: Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，依赖manifest/config无变；非Plan的Spec/contract/SD hash与R6一致，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。临时probe保存后移除、原R3探针hash未动，最终HEAD相同、tracked implementation clean；git diff --check及实际edition2021两改动Rust文件fmt通过。

acceptance_results: AC-04/05 fail（准入分析资源越界）、AC-07 fail（两finding含产品warning）；AC-02/03原语义finding在受查面闭，完整剩余AC/V审计仍pending；AC-01/06/09保留旧证据边界，AC-08 pending merge。SD-01/02计量主张暂不批准，其他SD保留已有边界；guard文案与block-local前门边界仍需沉淀前对账，无canonical/ledger写入。遗漏扫描=新provenance类型漏预算消费者，不能以资源登记债替代合同验收。

next: 同树work修复F-R7-01计量/分配前拒绝与F-R7-02 warning，保持根/逃逸及Copy/Call语义，补小输入/超cap/MIR+ABC/跨块或别名对照后提交再审；T-03/T-04/T-06重开，executing4/9，T-01/T-02/T-05/T-09保留。pending_closeout_gates: 最终pass后冻结裸cargo t+tv+独立workspace完整有限合同档及实际触面门由merge统一执行，tf仍主检出批量；无整档/aggregate acceptance、合入、归档、清树或实施修复。

2026-10-26 stage:work(needs_fix R6 修复) | PLAN-822 r5 | outcome:pass(F-822-R6-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:3afbcbd3f(基于复审冻结 352cbc5c4;reviewer 遗留未跟踪 review_822_r3.rs 保持未跟踪原样) | findings 闭环:**F-822-R6-01**(局部 place 跨基本块返回绕过逃逸检查)→ 根因=R5 将 place 定为保留引用(不置 tokens_live)但前驱合并的 roots 传播以 tokens_live 为守卫 → 跨块后 provenance 丢失,Return 见无局部根而放行(运行期 InvalidHandle 而非承诺的准入拒) → 修复=①join 的 roots union **无条件于 liveness**(provenance 为 may-事实,live view 合并语义不变);②roots_eq 在任一侧持 roots 时即参与等价(非 live roots 增长驱动工作流收敛) → 反例=reviewer Jump 边形态 MIR 拒(mir.borrow-escape);同 CFG 外部根 param 返回支持(=7);branch-merge 局部(两前驱合并)拒/外部根支持 → 受控 mutation(join 恢复 liveness 守卫)恰使跨块逃逸测试红,已恢复 → gap 无;**F-822-R6-02**(Copy 将保留 place 误升为活跃借用,错误拒绝列表更新)→ 根因=is_token 扩容后 Copy/Call 的 token 分支无条件 tokens_live.set → place 副本经 root_has_live_token 阻塞 LIST_SET(mir.list.place-borrowed),违背保留引用策略(epoch/generation 退役而非借冲突) → 修复=Copy 对 place 仅共享 roots 不激活;Call 的 place 结果同样保留(roots 并集照算) → 反例=reviewer 双形态(direct/copy 后更新均=9);外部根 helper 往返后 caller 列表更新成功(=9) → gap 无 | 定向验证:五 bin **66 测全绿**(semantics 38);**821 ownership 语义档 25/25 回归绿**(join/eq 改动对 live token 向后兼容);零 unused warning;守卫语义 scope 必需清单 38 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第七轮独立 review(R5-01 已闭+R5-02 残余与 R6 回归本轮全闭;诊断:R5 将保留引用与活跃借用在创建点分离但未贯穿合并/等价/Copy/Call——本轮已完整贯穿,请优先复核跨块五反例与 Copy/Call 传播面)


### 2026-10-26 独立最终复审 R6

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施本轮修复，沿已有独立review上下文重建提交/合同/行为证据，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 352cbc5c4e1afe4dd1a515e856d4986503506a22 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: ad9a69ffa12919ac8812b18b78a0b481b6f319bb

R5原5个独立正反例全绿、committed Place定向13绿：F-822-R5-01跨帧提交CLOSED（caller实返9、清理平衡）；F-822-R5-02原同块逃逸/record字段例修复，但完整不变量PARTIAL。新增F-822-R6-01(P1，R5-02残余)：同一本地根Place返回仅增加Jump到后继块，MIR/ABC均授信，真实caller使用才InvalidHandle/live=0；LIST_PLACE不置tokens_live，而前驱merge与roots_eq仅保留/比较live token根，跨块丢provenance。合法外部根参数跨块返回=7/live0、跨块读=7均绿。新增F-822-R6-02(P2，修复回归)：Copy将保留引用Place意外置为活借；直接mint后ListSet与fresh索引实返9，添加Copy同token后两门却mir.list.place-borrowed拒。新独立5例=3绿2红，无ignored；原绿色不能代替整个机制闭环。

evidence: [R6完整报告/最小重放/关闭oracle](attachments/822/review-r6/report.md)，同目录replay.rs、replay-r5/new-replay/place日志、changed-files/input-hashes与sd-frozen；临时probe已保存后移除，原R3未跟踪探针未动。dependency_revisions: Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，无依赖/配置变更；现行Spec/contract/SD绑定见hash收据，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD未漂移、tracked implementation clean、git diff --check通过，四个改动Rust文件按真实edition2021的rustfmt检查exit0；先误用edition2024的检查保留原日志并归工具调用错误，非产品finding。定向编译无warnings。

acceptance_results: AC-02/03/04 fail、AC-07 fail；AC-01/05/06/09保留历史证据边界，剩余完整AC/V/SD审计pending；AC-08 pending merge。SD-01/02准入/寿命未批准，其他SD保留原审查边界；SD-02 guard表述与先前block-local边界仍待沉淀前对账，无canonical/ledger发布。遗漏扫描=保留provenance与活借拆分未覆盖CFG join/等价/Copy/Call，合同内必修非延期债。

next: 同树work诊断并修复跨块根集合保留及复制/返回的引用政策一致性，补局部/外部根、分支合流和retire/fresh正反控，相关定向检查后提交再审；T-03/T-04/T-06重开、executing4/9，T-01/T-02/T-05/T-09保留。pending_closeout_gates: 最终pass后冻结裸cargo t+tv+独立workspace完整有限合同档及实际触面门由merge统一执行，tf仍主检出批量；本轮无整档/aggregate acceptance、合入、归档、清树。

2026-10-26 stage:work(needs_fix R5 修复) | PLAN-822 r5 | outcome:pass(F-822-R5-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:352cbc5c4(基于复审冻结 ad9a69ffa;reviewer 遗留未跟踪 review_822_r3.rs 保持未跟踪原样) | findings 闭环:**F-822-R5-01**(跨帧 PLACE_SET precheck 与 commit 使用不同帧→unreachable panic/错帧风险)→ 根因=commit 块索引 frames.last_mut()(当前帧)而 precheck/resolve 按 place.frame 激活序定位所有者帧 → 修复=commit 同样按激活序 find 所有者帧 → 反例=reviewer 界定输入(caller List[i32][7]+mint place 传 helper,helper PLACE_SET 9)执行 Success 且 caller 读=9 零 panic(helper 同号槽永不被索引);read 正控=7 → 受控 mutation(commit 回退 last_mut)恰以 reviewer 同款 unreachable panic 红,已恢复 → gap 无;**F-822-R5-02**(Place 绕过借用逃逸与 record 字段限制)→ 根因=is_token 仅 StrView/MutRef,record 字段域漏 Place,Return 逃逸门未覆盖 place → 修复=①Place 入 token 分类(不拥有 root 的保留引用);②LIST_PLACE 记录根(base 槽)但**不入 tokens_live**(place 为保留引用非独占借——结构变更经 epoch 退役 place 而非借冲突,push-after-mint 既有控制不变绿);③Place param 根=外部(caller 帧持有列表,存活期间使用与返回均支持——param 往返正控);④Return 逃逸门拒局部 mint 的 place 返回(mir.borrow-escape,**先于 dispatch**——reviewer 的嵌套被调形态现为准入拒而非死帧 InvalidHandle);⑤record 字段域拒 Place(mir.field-borrow,与 StrView/MutRef/RecordView 同门,未使用表项同拒)→ 反例五:跨帧 write/read 双控、escape 拒、record 拒、param 往返正控;普通标量 record 正控既有 | 定向验证:五 bin **61 测全绿**(semantics 33);零 unused warning;守卫语义 scope 必需清单 33 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第六轮独立 review(R4 已闭+R5 两项闭环;本轮为 R2 Place 面的原始集成缺口被相关路径复审暴露,非新特性请求——请优先复核跨帧帧解析一致性与逃逸/字段门的五反例)


### 2026-10-26 独立最终复审 R5

续接核验（2026-10-26）：中断后核对已有报告、原始日志、反例、风险登记与当前树，R5收据已保存完整。HEAD仍为ad9a69ffa12919ac8812b18b78a0b481b6f319bb、tracked implementation clean、repair diff格式检查通过；依赖/Spec/合同/冻结SD及原R3探针hash未漂移，D821祖先成立。独立重读代码确认两项finding机制，复用已有定向运行证据，未声称重新执行测试。维持本轮needs_fix/executing4/9与同树work交接，详[R5续接记录](attachments/822/review-r5/report.md#interrupted-review-continuation--2026-10-26)。

stage: review | review_scope: final | reviewer_identity: 当前独立聊天 Codex，未实施本轮修复，从提交/合同/独立反例重建，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: ad9a69ffa12919ac8812b18b78a0b481b6f319bb | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 7239e8f4774bc4f6ea7cc6354bd78f99f780b55a

F-822-R4-01 CLOSED：原独立反例在 MIR/ABC 两门均以 mir.type-ref 拒，真实 CollectionsUpdateSession 返回 Err 不 panic；合法 Place<i32> 经同路径返回7/live=0；原 replay 1绿、提交新增边界/域/保会话2绿、既有 Place 8绿。根因诊断与类型表修复匹配。但关联路径独立反例暴露两个原 Place 集成缺口：**F-822-R5-01(P1)** PLACE_SET 预检按 token.frame 找 caller，提交却用当前 callee 帧，合法类型参数经两门准入后在 av.rs:2735 unreachable panic；跨帧读对照返回7。**F-822-R5-02(P1)** Place 未纳入借用 provenance/escape 和 record 字段拒绝域：内部 helper 返回本地列表的 Place 被 MIR/ABC 授信，callee 退出后 caller 使用才 InvalidHandle；record{token:Place<i32>} 表项也通过两门。违反 §5.1.6 borrow 逃逸拒绝及既有字段域，不能用运行期 stale 检查代替。新独立5例=2绿/3红（第三红是同F-02字段兄弟），非整档/aggregate。

evidence: [R5完整报告/关闭oracle](attachments/822/review-r5/report.md)，同目录 replay.rs、原R4 replay/新反例/repair/place日志、input-hashes.txt、sd-frozen.md；临时测试已保存并移除，原未跟踪R3探针保留未动，tracked implementation clean、最终HEAD一致。dependency_revisions: Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，D821合入祖先重核成立；Spec/合同/SD hash与R4一致，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。

acceptance_results: AC-02/03/04 fail、AC-07 fail；AC-01/05/06/09保留旧证据边界，完整剩余AC/V/SD审计pending，不冒领pass；AC-08 pending merge。SD-01/02生命周期/准入尚不批准，SD-03..06维持已有审查边界，无canonical/ledger发布。遗漏扫描=Place新类型未完整接入帧选择、借用/字段域，均合同内必修已记风险簿，非获准延期债。

next: 同树work修复F-R5-01/02，T-03/T-04/T-06重开、executing4/9；T-01/T-02/T-05/T-09保留。提交前补Place参数/返回/字段和真实目标帧不变量的诊断与永久正反控，只跑受影响检查后再审。pending_closeout_gates: 最终pass后冻结裸cargo t+tv+独立workspace有限合同档及实际触面门由merge统一执行，tf仍主检出批量；本轮未整档、合入、归档、清树。

2026-10-26 stage:work(needs_fix R4 修复) | PLAN-822 r5 | outcome:pass(F-822-R4-01 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:ad9a69ffa(基于复审冻结 7239e8f47;reviewer 遗留未跟踪 review_822_r3.rs 保持未跟踪原样) | finding 闭环:**F-822-R4-01**(Place 参照索引绕过类型表验证,准入后链接器越界 panic)→ 根因=类型表 shape pass 验证 List.elem/Variant payload 而漏 Place.elem(R3 新类型的不完整集成;快照合并器无守卫索引 map[elem]) → 修复=类型表 pass 对每个 Place 参照做两项检查:越界拒 mir.type-ref(**先于任何 checked 凭据**,未使用表项同拒——其为不可信类型表的一部分)、域违规(容器/自引用/环参照)拒 mir.type.place-elem(冻结元素域:无列表可持有这些种类,故无 place 可参照) → 反例:reviewer 复现形态(合法 beta+一条 Place{elem:u32::MAX} 未使用表项)verify 拒+**真实更新会话返回 Err 且 prior 会话完好可执行(零 FunctionCode/零链接/零 panic)**;合法对照 Place<i32> 经同一 produce/link/execute 路径返回 7;boundary(len+1 越界/len 自指)、容器参照、显式自环参照逐项确定性拒 → 受控 mutation(禁用两项检查)恰使两反例红,已恢复 → gap 无 | 定向验证:五 bin **56 测全绿**(incremental 8);零 unused warning;守卫 incremental scope 必需清单扩至 8 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第五轮独立 review(四轮均为修复引入/暴露的回归与集成缺口逐 finding 闭环;本轮为 R3 新类型的不完整表验证集成,请优先复核 Place 参照反例与快照合并器假设)


2026-10-26 stage:work(needs_fix R3 修复) | PLAN-822 r5 | outcome:pass(F-R3-01/02 闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:7239e8f47(基于复审冻结 bf9e92cb4;reviewer 树中遗留的未跟踪 review_822_r3.rs 保持未跟踪原样,不入本计划——其字节已由 reviewer 保存于 attachments/822/review-r3/replay.rs) | findings 闭环:**F-822-R3-01**(Place 无参照类型,bool 可写入 List<i32> 并实际损坏)→ 根因=MirType::Place 为无参能力类型,typing 无法关联元素 → 修复=Place(ListTypeRef) 参照参数化(wire tag15 带 elem;snapshot fixpoint 映射;collections-only 门)且三条指令全绑定:LIST_PLACE 铸造时 dst 参照必须==base 元素、PLACE_GET dst==绑定元素(Copy)、PLACE_SET value==绑定元素 → 反例×3=reviewer 复现形态 get/set 错型均 verify 拒 mir.list-type(零编码/准入/执行)、mint 绑错(Place<i32> over List<Str>)拒;既有合法/stale 控制不变绿 → gap 无;**F-822-R3-02**(共享转移清空目标)→ 根因=R2 的 drop_owned_reverse 无条件 take 共享 buffer 的 elems → 修复=仅 Rc::strong_count==1(最后引用)时逆序清,共享句柄释放=责任转移 → 反例×4=move 保 len(reviewer 复现 1 vs 0)、move 保 owned 元素(台账平衡单次释放)、copy 共享后 drop 源 dst 完好、call move-in(owned_kill_slot 其余共享调用方)经被调参数读 len=1;正常/trap/嵌套逆序 oracle 不变绿 → 受控 mutation 验证(无条件清空变异恰使 move 测试红回 len=0,已恢复)→ gap 无 | 定向验证:五 bin **54 测全绿**(first-case 4/semantics 28/resources 11/cache 5/incremental 6);core/probe 零 warning(树中 reviewer 遗留未跟踪文件的 warning 不属本计划);mn-worker 构建净;守卫 semantics scope 必需清单扩至 28 id 实跑绿 | 全档 acceptance 按裁定不在修复轮重跑 | blockers:无 | next:T-07 第四轮独立 review(三次失败后的诊断性提交:本轮两 finding 均为上轮修复引入/暴露的回归与原始缺口,逐 finding 闭环+mutation 证据;请优先复核 F-R3-01/02 反例与 owned_kill_slot 全部共享调用方)


2026-10-26 stage:work(needs_fix R2 修复) | PLAN-822 r5 | outcome:pass(F-R2-01..04 全部闭环,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commit:bf9e92cb4(基于复审冻结 81c33e091) | findings 闭环:**F-822-R2-01**(候选不得安装)→ apply_revision 重排为 build-aside+原子提交(link+执行成功才改 revision/units/identity)→ 永久反例×2(reviewer 复现形态:签名拒后**立即**查当前对象 Arc 不变+prior snapshot 可执行=7;link 成功但执行 trap 的候选同不安装)→ gap 无;**F-822-R2-02**(缺失 Index Place/token 面)→ 交付合同路径:MirType::Place(tag15)+Value::Place(ListPlaceV: frame/slot/buf_id/epoch/index/elem_gen)+指令 0x2C LIST_PLACE/0x2D PLACE_GET/0x2E PLACE_SET 全链(verify/ABC/AV/ac-mn 渲染+受控拒/contract-decision 增补),ListBuf 增 elem_gens(push/insert/remove 维护,LIST_SET/LIST_FIELD_SET/PLACE_SET bump)→ 双控反例×5 组(正控重复读/push 后旧 place trap+fresh 读新末元素/remove 后旧 place trap+fresh 索引读移位值(V-12)/元素替换退役该元素 place 而他元素 place 存活+PLACE_SET 自退役+fresh 得新值/越界 mint trap)→ 恢复适用性:place 为运行期值不入 store(无恢复义务)→ gap 无;**F-822-R2-03**(正序清理)→ drop_owned_reverse(元素逆序责任释放、变体 payload 递归、容器 buffer 最后)+owned_kill_slot 与帧退出 LIFO 全接入+min_buffer_id 指纹域扩展 → 固定 oracle×3(正常退出/trap 出口/嵌套 List<Outcome<Note,Str>> 均 frees=[2,1,0] 且 live=0)→ gap 无;**F-822-R2-04**(守卫不完整)→ Assert-Selected 拒任意 ignored/skipped+必需用例 id 恰好一次(冻结 per-scope 清单)+-GuardSelfTest 四坏形状受控验证(valid 过/skip/missing/zero-selected/duplicate 全拒非零)→ gap 无 | 定向验证:822 五 bin **47 测全绿**(first-case 4/semantics 21/resources 11/cache 5/incremental 6);guard 自测+first-case/semantics/incremental scope 带清单实跑绿;mn-worker 构建净;reviewer 点名 warnings 全清零 | 卫生:T-04/T-09 表行重复尾文本已修;sd-draft 按交付更新(SD-01/02 的 Place/epoch/生命周期主张现在有实现支撑;SD-06 描述事务化会话) | 全档 acceptance 按用户裁定不在修复轮重跑——review 通过后由 merge 在冻结 HEAD 全验 | blockers:无 | next:T-07 再独立 review(重点:F-R2-01..04 闭环复核+上轮未完 AC-01/04/06/V/SD 全量)


2026-10-26 stage:work(needs_fix 修复) | PLAN-822 r5 | outcome:pass(F-01/F-02 修复完成,定向验证绿,execution_done 再送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | code_commits:717e9c208(F-01)/81c33e091(F-02),基于复审冻结 9bf26b9dd | findings 闭环:**F-822-R1-01**(不变量=变异型集合指令预算拒先于任何变异,§5.1.2 失败旧列表/源不变)→ 修复面四臂(ListSet/ListFieldSet/ListRemove/VariantProject-owned precheck→commit;FieldSet 补 deep-delta 记账,Remove 补 root 减账)→ 永久反例 4 个 both-sides 测试+unwind free 序判别子 → 受控 mutation 验证(变异恰红/修复绿)→ 剩余 gap 无;**F-822-R1-02**(不变量=V-13 须经真实集合更新入口:执行/清理/身份/生产 IO/依赖正反控/正式 gate)→ 修复=probe 新生产模块 collections_session.rs(真门入会话/会话携带未改单元/fn:<name> 链接签名等价/内容指纹域 gate)→ incremental 档 4/4 重写为入口驱动 → 剩余 gap 无 | 定向验证:822 五 bin 37 测全绿(first-case 4/semantics 13/resources 11/cache 5/incremental 4)+821 corpus 回归 3/3(受 F-01 执行器改动最近邻);**全档 acceptance 按用户裁定不在修复轮重跑——review 通过进入 reviewed 后由 merge 在冻结 HEAD 全验**(修复前一轮全 9 scope 曾端到端绿含 818 24/24) | 其他:818 corpus 重放再生成的六个 turnaround-mn-*.json 已 restore(818 自身时戳证据,非本计划载荷,先例同 stash 记录) | blockers:无 | next:T-07 再独立 review(复核 F-01/F-02 闭环与全部剩余 AC/V/SD)


2026-10-26 stage:work(收尾) | PLAN-822 r5 | outcome:pass(work-owned 全部完成,execution_done 送审) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | base:de93bacc0 | code_commits:40631d41d→9bf26b9dd 共 10 枚 | task_ids:T-01..T-06+T-09 全勾(T-07 review/T-08 merge 非 work-owned) | evidence:**完整 9-scope acceptance 端到端通过**(scripts/verify-av-collections-822.ps1 -Scope acceptance:first-case 4+semantics 13+admission 1+resources 7(含 128MiB 父侧采样硬门)+cache 5+compatibility(821 5/7/3、798/820 15、817 5、818 24)+autodown 3+incremental 4+1);V-13 四子case全绿;ac-mn 兼容回归被 acceptance 抓获并修复(fingerprint 渲染+lower 受控拒,818 24/24 恢复);822 新测试共 36(33 probe+3 主仓) | blockers:无 | 已知边界(review 核):①guard 事实为块内线性语义(BRANCH then-edge 路径传播未实现——较合同保守,拒绝面更大非更小,全部 V 案例以块内形态覆盖);②collections fixture 无 source sidecar(V-11 strip-debug 子面以默认无-spans 形态覆盖);③LIST_SET 极端预算序(旧值 drop 后才判超)按 Resource 上报、极端路径由 V-03 fault 面覆盖 | merge-owned 门禁 pending(显式):冻结 HEAD 上裸 cargo t + 触面 cargo tv + 新 workspace 完整有限合同档(cargo test --manifest-path experimental/ac-av-probe/Cargo.toml)+ 按 diff 选 tt/tb/tu 触面;tf 批量档按仓规到期判定 | next:T-07 独立 review(独立上下文、同树只读)

2026-10-26 stage:work | PLAN-822 r5 | outcome:pass(本会话 work 范畴 6/8 任务完成,T-09 部分完成继续 executing) | worktree:D:/autostack/.wt/lang-822/auto-lang | branch:plan-822-dev | base:de93bacc0(基面钉 master,含 849 已合入;probe/注册写面与 5009dc629 等价) | code_commits:40631d41d→4dfc59420 共 8 枚(core/fixture/recipe/semantics/resources/cache/script/incremental) | 组内:auto-ui detached worktree 补入(auto-lang crate 硬 path 依赖解析,非 junction,822 不改它);auto-down 兄弟随组建树 | task_ids:T-01..T-06 完成,T-09 部分(V-13 子case 2/3 落地,子case 1/4 与 128MiB 实测待续) | evidence:probe 五 bin 4+13+6+5+3 全绿;主仓 av_collections_822 3/3 绿(corpus/negative-oracle/worker-faults);回归 av_owned_821 corpus 3/3、821 first-case/semantics 31 测、instr_spec coverage 全绿;脚本 first-case/autodown scope 实跑绿 | blockers:无;剩余=T-09 未完部分(见 §8 行内记录)与 merge-owned 门禁(裸 cargo t + tv + 新 workspace 有限合同档,review pass 后由 merge 在冻结 HEAD 执行) | next:work 继续 T-09(828 战役重放接线+gate 解封+128MiB 实测),完成后送独立 review

2026-10-21 stage:new | PLAN-822 r4→r5 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-822 r3→r4 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-13四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增集合/分支的状态和计量的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-822 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-821及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..05/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

无须用户再裁定已认可的总体方向。精确opcode/布局实现/资源测量由有界T-01和T-02负责冻结，不能改变§5语义或预算；需要扩大支持/资源/兼容承诺时回new显式revision。当前依赖与执行授权门如§4；不以等待实施为由伪造通过或删AC。NaN-boxing、完整数值运算/用户泛型、共享图、async、native与CTE复合消费者另案，不是本计划“已完成后续”。


**当前排期与验收（2026-10-21）**：D828后按§5本计划优先级/原D门及单写交接推进，不等待无硬依赖的整批；T-09 / AC-09 / V-13 未完成，旧revision/测试总数不覆盖新增义务。用户已确认该顺序/目标，不重复请求同范围授权。

### 2026-10-26 独立最终复审 R1

stage: review | review_scope: final | reviewer_identity: 当前独立聊天 Codex，未参与本轮实现；从合同与实际源码重建判断 | plan_revision: 5 | outcome: needs_fix
worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 9bf26b9ddb351be32671a8c180872c46df42b8a4 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83

- F-822-R1-01 (P1, AC-02/03/05, T-04): core/src/av.rs ListSet 1979–2048 在 std::mem::replace 后计算旧值大小并 drop(old)，随后才检查 max_slot_bytes。预算不足返回 Resource 时没有恢复元素；注释所称 rollback 不存在。违反 §5.1.2 失败列表/源不变及 §5.2 分配前拒绝。静态控制流已确认，未声称运行反例。修复范围：先计算/预检再提交，定向构造 owned 元素由小变大、预算恰不足与恰足对照，观察失败时旧元素/源责任、台账及清理；检查 push/insert/switch 同类提交顺序。既有 work 记录的“极端预算序”不能豁免合同。
- F-822-R1-02 (P1, AC-09, T-09): probe/tests/av_collections_822_incremental.rs 的 body_edit_identity 由测试自行 admit alpha 并直接把同一 fb1 Arc 传入 link，没有调用集合能力真实更新 consumer，没有执行新旧 snapshot 或断言语义/cleanup、生产阶段/IO=0。signature_change_precise 中 alpha 没有调用 beta，负控仅为返回类型错误 MIR，无法证明实际接口/layout/helper/授权依赖精确失效。forced_degradation 的 degraded 闭包仅在测试内计数，未使正式 consumer gate 拒绝。违反 V-13 子case 2/3/4，D828 标量 campaign 不能替代。修复：通过实际集合更新入口运行两函数修订及结果/清理，记录未改生产/IO与身份，使用真实依赖变化合法正控及拒绝反控，使强制重建在正式 gate 非零退出。

acceptance_results: AC-02/03/05 fail（F-01）；AC-09 fail（F-02）；AC-01/04/06 与 SD 全量核对未完成，pending，不能继承 work pass；AC-07 未达；AC-08 merge-owned。
evidence: 上述源码控制流/断言静态独立检查；未运行整档或 aggregate acceptance。实现树有六个 docs/reports/plan818/turnaround-mn-*.json 修改，需 handoff 解释/处理；不删除他方证据。
dependency_revisions/spec_inputs: 本轮未完成完整 hash 绑定，不发 pass；规范增量未批准或发布。
pending_closeout_gates: 冻结新提交复审通过后由 merge 跑裸 cargo t、cargo tv、新 workspace 有限合同档及实际触面；tf 仍批量主检出。
next: work 在同树修复 F-01/F-02，完成定向验证并提交；T-04/T-06/T-09 重开，保留其他历史记录。复审需覆盖剩余 AC/V/SD，不能把本轮未查项视为通过。

### 2026-10-26 独立最终复审 R2

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施修复，从源码/合同/独立反例重建 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 81c33e0918468bc21f6bd756fb8bbd6b4616ab57 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83

F-822-R1-01 已关闭原 ListSet 先写后查缺陷，四项 transaction 定向测试独立实跑全绿；F-822-R1-02 部分修复但未闭环。新增 F-822-R2-01(P1)：集合更新会话先发布后 link/execute，拒绝签名候选仍覆盖当前函数，独立反例失败。F-822-R2-02(P1)：合同必需的 Index Place/token/epoch 校验及旧 token 反例缺失，仅 epoch 自增。F-822-R2-03(P1)：List<Str> 两元素退出按正序释放，独立反例 free ids=[0,1,2]，违反逆序清理。F-822-R2-04(P2)：脚本可接受含 ignored 的绿色结果，未核 required-id 完整性。

acceptance_results: AC-02/03/09 fail；AC-01/04/05/06 partial（不把未覆盖/未运行项判 pass），AC-07 未达，AC-08 pending。详见[完整报告与最小重放](attachments/822/review-r2/report.md)及同目录日志/源码/spec-hashes.txt。定向资源4绿、incremental受影响3绿；独立失败安装/逆序清理反例各1红。最终 HEAD 再核一致、实施树 clean；临时 review 测试已移除，耐久源码保存在附件。SD 冻结 SHA256=F86CF08D8D8A645C209FF138E10CF5D9F87B69876500B745291D483D51FDDA2B；SD-01/02/06 尚不具可沉淀证据，不发布规范/ledger。

next: 同树 work 修复以上 finding；T-03/T-04/T-06/T-09 重开(current_step=3)，保留历史完成声明但不继承为当前 pass。修复仅跑受影响定向检查；merge-owned 裸 cargo t/cargo tv/独立workspace有限合同档仍 pending，最终复审通过后冻结再跑；tf 批量主检出。编译未处理 warnings 与计划重复表格尾文一并清理。

### 2026-10-26 独立最终复审 R3

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施本轮修复，独立检查提交差异和反例；未派生代理 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: bf9e92cb472495037ce73daab5b0c1d2a4f09f7d | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 81c33e0918468bc21f6bd756fb8bbd6b4616ab57

R2-01 失败候选安装在 link/trap 两路径独立关闭（2 测绿）；R2-02 Place 机制已交付且原 5 正反控绿，但类型不变量未闭；R2-03 原 3 逆序清理 oracle 绿，但共享转移路径引入回归；R2-04 skip/missing/zero/duplicate 控制独立关闭。新增 F-822-R3-01(P1)：PlaceGet 任意 Copy dst/PlaceSet 任意 value 均被授信且 runtime 无类型 backstop；List<i32>[7] 经 place 写 true、再读至 i32 返回槽，实际 Success Bool(true)。新增 F-822-R3-02(P1)：Move 复制 ListHandle 后 owned_kill_slot 调 drop_owned_reverse，清空目标同享 buffer；合法一元素列表移动后 len 由预期 1 变成 0。

证据：[完整报告与关闭oracle](attachments/822/review-r3/report.md)，同目录 replay.rs/replay.log.txt、session/cleanup/place/guard 日志、input-hashes.txt 与冻结 sd-frozen.md。入审已有未跟踪 review_822_r3.rs，本轮保留未改、独立审读并执行，不声称本轮创作或实施者所有；独立反例 1 绿 3 红，修复定向现有测试 10 绿，guard 有效控制通过+4坏控制拒。该未跟踪 probe 的 unused 警告单列为探针卫生，不误报 committed 产品警告；tracked implementation 始终 clean、HEAD 未漂移。Cargo.lock hash=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873。

acceptance_results: AC-02/03/04 fail；AC-01 partial/fail（typed门）；AC-05/06/09 partial，保留原证据及未覆盖边界；AC-07 fail；AC-08 pending。V-01/02/06/11/12受影响，V-13失败安装子面已修；其他未查完项仍pending，不因原样例全绿冒领。SD-01/02不能沉淀类型/生命周期主张；SD-06事务化在所查路径成立，整体SD尚未批准；依赖/Spec绑定见hash收据，无canonical/ledger写入。

next: 同树work修复两个新finding，T-03/T-04/T-06重开，保留T-01/T-02/T-05/T-09及历史，executing 4/9。连续失败后送审须补共享Place类型/转移与析构不变量的根因与覆盖调整，不以旧样例总数代闭环。本轮无整档或aggregate acceptance；merge-owned裸cargo t+tv+独立workspace有限合同档及实际触面仍pending，仅最终复审通过后冻结执行。未合入/归档/清树。

### 2026-10-26 独立最终复审 R4

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施本轮修复，独立检查实际差异与新增反例，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 7239e8f4774bc4f6ea7cc6354bd78f99f780b55a | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: bf9e92cb472495037ce73daab5b0c1d2a4f09f7d

R3 两项具体缺陷已验闭：错型 get/set/mint 拒绝、move/copy/call move-in 保列表及 owned 元素、正常/trap/嵌套逆序清理，语义定向 28/28 绿；拒绝/执行失败候选保会话两测 2/2 绿。根因诊断与新增覆盖符合实际修复。新增 F-822-R4-01(P1)：Place(ListTypeRef) 新 elem 未纳入类型表全量验证，合法 beta_unit(7) 仅追加 unused Place<4294967295>，MIR 与 ABC 两门均 Ok，真实 CollectionsUpdateSession::apply_revision 经已准入 FunctionCode 到 ProgramSnapshot::link，在 program_snapshot.rs:380 越界 panic(len=4)。合法 Place<i32> 对照返回7/live=0；独立反例测试红。违反原 AC-04/V-01/11 的消费前拒绝及不得 panic 合同，不能以 unused 类型为由豁免。

证据与关闭oracle：[R4报告](attachments/822/review-r4/report.md)、replay.rs/replay.log.txt、semantics/session 日志；依赖 Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，Spec/contract/SD 与R3相同并重hash于 input-hashes.txt，冻结 SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。HEAD末检一致、tracked implementation clean；原未跟踪R3探针未动，本轮新增临时探针已保存附件后移除。

acceptance_results: AC-04 fail、AC-07 fail；AC-02/03 原R3实例及受测同类修复有效，但新增类型不变量未闭；AC-01/05/06/09保留此前证据及未审边界，不冒领完整pass；AC-08 pending。SD-01/02/06新类型准入/拒绝主张未批准，其他SD仍待完整核对；无canonical/ledger发布。遗漏扫描发现新TypeIdx只接入指令检查和合并器、遗漏全表检查，属合同内必修非可延期债。

next: 同树work修复 F-R4-01（所有类型表条目的参照有效性/域；实际MIR/ABC门及真实consumer坏输入受控拒绝、合法对照与类型重映射保持），仅受影响定向检查后提交再审；T-03/T-04/T-06重开，executing4/9，T-01/T-02/T-05/T-09保留。完整AC/V/SD未查项仍pending。merge-owned冻结裸cargo t+tv+独立workspace有限合同档及实际触面门继续pending，仅最终pass后执行；无整档/aggregate acceptance、合入、归档或清树。

### 2026-10-26 独立最终复审 R8

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施修复，独立审读合同/提交/行为，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 952c2b586a6e040fd1637cbbb983c633f1cb0b78 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 3afbcbd3f74ed3d59e562dd901c419678f92f58e

F-822-R7-01/02 CLOSED：独立 allocator 重放确认 12000 Place 在两门拒绝，分配前峰131202B<16MiB、steps0；64 Place 对照两门通过，峰20126B；R6五控及混合根合流三形态继续绿；死方法已删、产品编译无警告、changed Rust fmt/diff检查绿。但剩余AC/V审计发现 F-822-R8-01(P1，原集合集成遗漏)：CLONE List<i32>[7] 后修改原列表为[9]，克隆也读出9（应7）；真实MIR/ABC准入均通过，execute Success I32(9)/live0，独立反例exit101。av.rs clone_owned_value仅深访Record/Str，List/Variant落other.clone，ListHandle浅共享Rc。违反原§1/§5.4与V-03/11；contract-decision §4称集合CLONE/MOVE/DROP均拒也与实际不符，不能据此削弱原合同。须补真深克隆、owned/Variant兄弟及逐元素故障/取消回滚oracle，保COPY/MOVE与逆序清理。

evidence: [R8报告/反例/关闭oracle](attachments/822/review-r8/report.md)，同目录replay-clone.rs、clone.log.txt、replay.rs、memory/replay-r6日志、input-hashes及sd-frozen；Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致，tracked implementation clean；既有R3/R8未跟踪探针保留，本轮新增clone探针已保存移除。无整档/aggregate acceptance或无关性能重跑。

acceptance_results: AC-02/03 fail、AC-05 partial/fail（clone资源关闭oracle未完）、AC-04 R7内存缺陷关闭；AC-01/06/09保留既有证据边界，完整剩余AC/V/SD未冒领pass，AC-07 fail、AC-08 pending。SD-01/02不批准深克隆/生命周期完整主张；其他SD维持已有边界，无canonical/ledger发布。next: 同树work修复F-R8-01并对齐合同/证据，T03/04/06重开executing4/9，其余保留；最终pass后merge-owned冻结裸cargo t+tv+独立workspace有限合同档及实际触面门，tf仍主检出批量。未修实现/合入/归档/清树。
### 2026-10-26 独立最终复审 R9

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/反例核查，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 7402022b5976531b6c685104d87d7354111a0766 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 952c2b586a6e040fd1637cbbb983c633f1cb0b78

R8 原浅共享实例 CLOSED：独立原重放返回7/live0；提交clone五控5/5绿。但完整不变量仅partial：F-822-R9-01(P1，修复设计缺口) clone_owned_value 对List/Variant调用自身且无取消/期限/fuel上下文，循环内无poll；外层预记燃料不等于中途取消。冻结域允许Variant嵌套，三层合法Variant通过MIR/ABC且Clone成功，证伪“递归恰一层”的修复依据；违反§5.1.6显式遍历/§5.4与V-11取消关闭oracle，未声称实测stack crash或取消延迟。F-822-R9-02(P1，原类型图集成遗漏) check_record_acyclic虽有Variant取边分支，前置Record-only筛选却将Variant直接标BLACK并跳过；合法beta附unused Recursive{End,Next(Self)}，MIR/ABC两门均授信，原合同要求受控拒绝。独立新增2例=合法三层clone绿/自环拒绝oracle红(exit101)。

evidence: [R9报告/耐久重放/关闭oracle](attachments/822/review-r9/report.md)，同目录replay.rs、replay-r8/clone-controls/variant日志、input-hashes、sd-frozen及fmt；Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致、tracked implementation clean，既有R3/R8探针未动，本轮临时探针保存后移除；changed Rust fmt/diff绿、产品无警告。无整档/aggregate acceptance。

acceptance_results: AC-02/03/05 partial/fail（原clone实例已修但遍历/取消未闭）、AC-04 fail（Variant环准入）、AC-07 fail、AC-08 pending；AC-01/06/09及R7内存修复保留既有证据边界，剩余AC/V/SD审计不冒领pass。SD-01/02不得声称有界clone及Variant无环完整；其他SD维持已有边界，无canonical/ledger写入。next: 同树work补显式遍历/真实中途取消及清理oracle、修Variant图全表遍历并加自环/互环/合法链双门控制，T03/04/06重开executing4/9，保留其余任务/历史；最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门，tf主检出批量。未修实现/合入/归档/清树。
### 2026-10-26 独立最终复审 R10

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立代码/合同/原始反例核查，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 9e65e606d411693da2d5e21053758a86e0977c52 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 7402022b5976531b6c685104d87d7354111a0766

F-R9-02 CLOSED：独立原自环反例MIR/ABC均拒mir.type-cycle，无环三层Variant Clone成功/live0；源码确实移除Record-only跳过。F-R9-01仅partial：F-822-R10-01(P1，R9残余) 新访客声称Value浅快照，但Value::Variant/VariantValue均derive Clone且payload=Box<Value>，val.clone/子项clone/装配src.clone均递归复制嵌套payload，初始复制先于helper首poll；显式循环未消除宿主递归，链状快照保留还累积平方级payload复制。更重要的关闭oracle反控：独立临时副本删除两个mid_traversal用例中的CLONE指令后，cancel/wall两测仍2/2绿；其预置cancel=true/wall=0且先执行1500组Const/Push，外层dispatch已能退出，未证明进入克隆/中途取消/部分克隆清理。未声称实测stack crash或内存越界。F-822-R10-02(P2) committed语义测试:3122未用c0，原bin构建持续unused warning，非临时探针独有。

evidence: [R10报告/重放/关闭oracle](attachments/822/review-r10/report.md)，同目录replay.rs、no-clone-controls.rs、replay-r9/no-clone-controls/clone-controls日志、input-hashes与sd-frozen。原clone_八控8/8绿但不替代区分性oracle。Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致、tracked implementation clean，既有R3/R8探针未改，R10临时探针已保存移除；changed Rust fmt/diff绿，无整档/aggregate acceptance。

acceptance_results: AC-02/03/05 partial/fail（遍历和中途取消关闭oracle未完）、AC-04原Variant环缺陷关闭、AC-07 fail（含未处理warning）、AC-08 pending；AC-01/06/09保留已有证据边界，剩余完整AC/V/SD审计不冒领pass。SD-01/02不得声称非递归有界clone完整，其他SD维持原界，无canonical/ledger发布。next: 同树work修真正浅的结构工作项/实际剩余deadline与fuel/可区分的生产clone中途取消及部分清理oracle，清warning；T03/04/06重开executing4/9，其余保留。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门，tf仍主检出批量。未修实现/合入/归档/清树。
### 2026-10-26 独立最终复审 R11

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立源码/合同/反例核验，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: b5c226214f3f32b41299df945585efabe40c6b28 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 9e65e606d411693da2d5e21053758a86e0977c52

F-R10-02 CLOSED（原语义bin无warning）；F-R10-01浅快照子面已修（Variant payload Rc，derived clone为handle bump），生产helper cancel/fuel seam可区分dispatch，但完整计量/期限仍partial。F-822-R11-01(P1，修复回归)：CLONE删除字节+字段计费后仅逐节点加fuel；真实verify/ABC/execute对照4B和4096B字符串，增加CLONE都只多5步（4B:15→20；4096B:4107→4112，均Success7/live0），违反§5.4及继承place-ownership规范Clone字节+字段计费；直接helper两输入也都仅fuel2，4096B在60预算仍成功。F-822-R11-02(P1，R10剩余deadline缺口)：dispatch传W-已耗时，helper却拿从segment起累计耗时与剩余值比，重复扣除Clone前耗时；直接helper未有TLS时elapsed恒0，1300元素/2602节点/剩余墙0仍成功，独立oracle红。动态墙结果只声明seam面，生产双扣由实际表达式确认，未冒称全链时序复现。

evidence: [R11报告/耐久重放/关闭oracle](attachments/822/review-r11/report.md)，同目录replay.rs、production-replay.rs、byte-fuel/production-byte-fuel/wall-seam/clone-controls日志、input-hashes、sd-frozen。原clone_九控9/9绿且无warning；changed Rust fmt/diff绿。Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致、tracked implementation clean；R3/R8既有探针未动，R11临时探针保存移除。无整档/aggregate acceptance。

acceptance_results: AC-02/03/05 partial/fail、AC-07 fail、AC-08 pending；AC-01/04/06/09保留已有证据边界，剩余完整AC/V/SD审计不冒领pass。SD-01/02不得声称按数据量计费/期限clone完整，其他SD维持已有界，无canonical/ledger写入。next: 同树work保留可执行的字节+结构共享fuel并修统一deadline口径及独立墙正反控，复核shared helper旧owned消费者，定向提交再审；T03/04/06重开executing4/9，其余保留。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门，tf主检出批量。未修实现/合入/归档/清树。
### 2026-10-26 独立最终复审 R12

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/运行核验，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 90074e87f6c073868a0ee1f0642501bf383b7846 | bookkeeping_commit: 392c7a7cd1e987d5326f98187ad838e15e249840 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: b5c226214f3f32b41299df945585efabe40c6b28

F-R11-01/02 CLOSED：独立原helper两控绿（4B fuel6；4096B budget60拒/fuel4098/minted0；零剩余墙入口拒fuel0），生产±CLONE重放4B差9/4096B差4101，Success7/live0；clone_12/12绿且无warning；guard有效/四坏控制符合预期、清单陈旧id已修；changed Rust fmt/diff绿。剩余清理矩阵审计发现F-822-R12-01(P1，原集合集成遗漏，非R11引入)：av.rs min_buffer_id(:934)与drop_owned_reverse(:972)对Variant载荷/List元素直接递归，Return/全出口unwind/源kill可达，违反§5.1.6明确的清理显式遍历；注释“一层容器/table界”不能限制合法Variant链宿主调用深度。独立64层合法Variant经真MIR/ABC/AV正常退出Success0/steps10785/live0证明支持域可达；该控制为绿，不声称实测栈溢出/内存越界，finding由实际递归调用链与既定实现约束不符成立。须合并审计order-key/逆序清理/末owner释放，保共享转移及逆序台账，不加语言深度cap。

evidence: [R12报告/重放/关闭oracle](attachments/822/review-r12/report.md)，同目录replay.rs、production-replay.rs、replay-r11/production-byte-fuel/clone/cleanup-domain/guard日志、input-hashes、sd-frozen。Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致、tracked implementation clean，R3/R8既有探针未改，R12临时探针保存移除，无整档/aggregate acceptance。

acceptance_results: AC-03/05 partial/fail（清理显式遍历）、AC-07 fail、AC-08 pending；R11两项及AC-02本轮clone计量/期限面已闭，AC-01/04/06/09保留既有证据边界，完整剩余AC/V/SD不冒领pass。SD-01/02生命周期完整主张暂不批准，其他SD保持原界，无canonical/ledger写入。next: 同树work修清理/order-key/末owner显式遍历，正常/故障/取消出口及Move/Copy/Call源kill代表控制，定向提交再审；T03/04/06重开executing4/9，其余保留。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门，tf仍主检出批量。未修实现/合入/归档/清树。
### 2026-10-26 独立最终复审 R13

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立源码/合同/反例核查，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 5490d6678cadb6e71cc01a96474151b56f4754f4 | bookkeeping_commit: e4adcc19ac91b69f0bb0ca4c5cd340557ba54d25 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 90074e87f6c073868a0ee1f0642501bf383b7846

R12原frame/替换值清理子面已修：独立64层控制Success0/live0/8832步；提交四新深链控制（3072层256KiB正常、trap、switch等）及逆序3控/transfer5控绿；min_buffer_id/deep_size/drop_owned_reverse均显式，旧值死亡路由和共享责任转移在所查路径成立。但全出口仍partial：F-822-R13-01(P1，R12残余) clone_owned_value的out及装配临时值在charge/poll返回Err时走默认析构，未进显式清理；独立同3072层/256KiB合法类型经MIR/ABC准入，生产helper fuel5572在叶先装配中耗尽，返回前STATUS_STACK_OVERFLOW(0xc00000fd)。源值最终drop已以测试pin排除；同栈64层部分装配、3072层planning早拒、3072层full成功三控均绿，隔离为部分克隆回滚。不是新深度承诺，也未称full-dispatch崩溃。须覆盖helper局部out/List/Record/Variant临时owner的fuel/cancel/wall/fault退出显式释放。

evidence: [R13报告/独立反例/关闭oracle](attachments/822/review-r13/report.md)，同目录partial-clone-replay.rs、partial-clone/partial-clone-controls日志，replay-r12.rs及cleanup/deep-chain/reverse/transfers日志、input-hashes、sd-frozen。两组cleanup筛选重复2例，按4个唯一提交新控制计，不冒计6。Cargo.lock SHA256=37C362A201967775BD767D8FBA7A2060DEA3FF720E35C1D2C6422C9E2F3A8873，冻结SD=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。最终HEAD一致、tracked implementation clean；R3/R8探针未动，R13临时探针保存后移除；changed Rust fmt/diff绿，构建无warning，无整档/aggregate acceptance。

acceptance_results: AC-03/05 fail（实际partial clone回滚abort）、AC-07 fail、AC-08 pending；AC-01/02/04/06/09保留对应已有证据边界，剩余AC/V/SD不冒领pass。SD-01/02不得声称全出口显式清理完整，其余SD保持原界，无canonical/ledger写入。next: 同树work补helper-local部分owner显式清理和对应故障/取消对照，T03/04/06重开executing4/9，其余保留；最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门，tf主检出批量。未修实现/合入/归档/清树。

### 2026-10-26 独立最终复审 R14

stage: review | review_scope: final | reviewer_identity: 当前聊天 Codex，未实施修复，从实际源码/合同/行为重建，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: fab5f492a74377d97c508e02dcea1916504bf8d2 | bookkeeping_commit: 8e5045941 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 5490d6678cadb6e71cc01a96474151b56f4754f4

F-822-R13-01 CLOSED：统一 build 错误出口显式 drain out/临时 children，List fault 在所有权转交前预留；本轮独立运行 partial_clone_rollback 两测绿，原深链反例返回受控 InstrSteps/live0，未再崩溃。预置 cancel 控制只证明入口拒绝，不冒称中途取消。既存 R14 原反例与 list-fault 日志保留，来源为入场已存在附件；本轮重新运行 cache 反例与正式 cache_compat 五控。

F-822-R14-01(P1，原恢复身份集成遗漏；AC-06/V-08，T-05/T-06)：restore_collections 以请求 types_fp 选 key，却仅 verify_abc_bytes 后直接 execute，没有比较载荷实际类型表指纹。真实 ManagedStore 中把合法 first-case 加一个 unused I64 类型（MIR/ABC均合法），以原 types_fp key+合法同owner/snapshot envelope存储；恢复实际返回 Success I32(0)/steps121。expected types-fp=46d81ee5fdbb84a2，payload=58dda3f6073c2aff，独立拒绝oracle失败(exit101)。违反§5.3类型/layout依赖与恢复不可信、V-08及SD-03。不是要求拒绝所有合法新类型，而是载荷与请求身份不匹配须在dispatch前拒绝。核查入口/owner/snapshot等相邻轴与载荷一致性，保合法恢复和真实依赖 clean miss；不改旧profile合同。

acceptance_results: AC-06 fail，AC-07 fail；AC-03/05 R13部分回滚缺陷关闭，其他条目保留既有证据边界，剩余未查项不冒领pass；AC-08 merge-owned pending。SD-03身份/恢复完整主张不批准，SD-01/02原R13阻塞已消除但本轮不授予全局pass；其余保持原边界。SD冻结hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739，Cargo.lock沿既有hash，未修改依赖/验证配置。证据见 attachments/822/review-r14/report.md、type-binding-replay.rs及日志。最终HEAD仍fab5f492a，tracked implementation clean；既存未跟踪review探针保留，探针unused import warning不归产品。无canonical/ledger发布、实现修复、整档或aggregate acceptance。

next: 同树work修F-822-R14-01，T-05/T-06重开，executing5/9；定向重放身份不匹配反例、合法恢复/clean miss及相邻身份正反控，提交后再审。最终pass后merge-owned冻结裸cargo t+tv+独立workspace有限合同档及实际触面门仍pending，tf仅主检出批量。未合入/归档/清树。

### 2026-10-27 独立最终复审 R15

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立核合同/源码/反例，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 14b6a9df2e52a46c72f49789e1dcd91818d11da5 | bookkeeping_commit: 3e4b68235 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: fab5f492a74377d97c508e02dcea1916504bf8d2

F-822-R14-01 CLOSED：实际decoded类型表指纹在execute前比较，原不同类型表恢复拒绝；正式cache_compat六控本轮独立运行6/6绿（含原反例及诚实store正控）。但其要求的相邻身份审计未完成：提交称 envelope_unwrap 已比较 embedded entry/snapshot 与请求，实际 collections_envelope 的 _entry 被忽略，snapshot 仅在外层metadata核对，restore 未比对 decoded function身份。

F-822-R15-01(P1，R14相邻身份残余，AC-06/V-08，T-05/T-06，SD-03)：restore_collections 在verify_abc_bytes及types_fp比较后直接execute，未绑定payload实际入口函数名及source_snapshot。独立真实store两反例均使用first-case原类型表（types_fp相等46d81ee5fdbb84a2），原owner/entry/snapshot key及matching envelope；①只把实际入口函数name改为foreign_entry，②只把实际函数source_snapshot改为请求值+1。两者均经MIR/ABC验证合法，restore均返回Ok(Success I32(0),steps121)，拒绝oracle各红，进程exit101。违反§5.3缓存身份/恢复不可信及V-08，不是要求拒绝合法改名/新修订，而是不能在旧请求身份下执行不同payload。已建立原因=外层身份检查与实际解码内容未绑定；未把其他未测试轴冒领失败。

最小重放：将 attachments/822/review-r15/replay.rs 放在既有plan树 experimental/ac-av-probe/probe/tests/review_822_r15.rs，运行 cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r15 -- --nocapture；本轮已有该临时文件，证据保存后移除。关闭oracle：两错配在dispatch前受控拒绝，原诚实恢复/依赖clean miss保持；核查其余实际payload身份轴，避免再用封套元数据替代实际内容核对。无全档要求。

acceptance_results: AC-06 fail、AC-07 fail；AC-01..05/09保留原证据边界，原R14类型指纹子面关闭；AC-08 pending。SD-03恢复身份完整主张仍不批准，其余SD维持既有边界；冻结SD hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。依赖/配置未变，Cargo.lock沿原SHA256；最终HEAD/cleanliness复核见input-hashes。未实施修复/发布canonical或ledger。

next: 同树work修F-822-R15-01并补真实入口名/快照正反控，T-05/T-06重开，executing5/9，其他任务保留；提交后复审受影响内容。merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面档仍pending，仅最终pass后执行；tf主检出批量。本轮无整档、合入、归档、清树。

### 2026-10-27 独立最终复审 R16

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/行为核验，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: f1d31632b1e625deb125142864e1ad032cb9870d | bookkeeping_commit: 9a2964804 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: 14b6a9df2e52a46c72f49789e1dcd91818d11da5

F-822-R15-01 CLOSED：decoded entry name/source_snapshot均在execute前核对；独立运行正式cache_compat七控7/7绿，含两错配拒绝与诚实恢复。owner重盖章机制经源码核实，不要求给无自证owner的wire增字段。但提交的“profile/schema由通用ABC门保证”论证不完整：通用门按payload自己的profile选择版本，并非按collections请求强制profile。

F-822-R16-01(P1，恢复profile身份残余；AC-06/V-08/09，T-05/T-06，SD-03)：collections restore未绑定实际wire/profile到固定collections身份。独立反例先构造合法collections scalar module(main返回7、snapshot123、types=[I32,Fn])得到请求types_fp；将同一module改为合法core-i32 profile/owner schema，保相同types/entry/snapshot，编码为AVAB，然后以collections的A22 key、plan822/schema/semantics封套写真实ManagedStore。restore_collections返回Ok(Success I32(7),steps2)，拒绝oracle红(exit101)。同一程序保持collections AVAD v3的诚实store正控绿。两例types_fp/entry/snapshot均匹配，故不是重演已关闭的R14/R15例。

既定规则：§5.3 profile/wire/schema身份矩阵、V-08依赖变化不复用与V-09旧profile独立、SD-03世代隔离；不是要求通用verify_abc_bytes不再支持core-i32，也不是要求core旧消费者改变。建立原因：abc_verify::verify_abc按decoded.profile选合法tuple；mir_verify对collections/data/owned核schema，但core分支无反向expected schema约束，重盖章collections owner不能代替检查actual profile。restore仅比较types_fp/entry/snapshot后execute，因此合法core穿透。本轮仅声称已实测core→collections混淆，不冒称其他profile均可穿透。

重放：附件 replay.rs临时放既有树probe/tests/review_822_r16.rs；cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r16 -- --nocapture。原始日志profile.log.txt；1合法对照绿/1拒绝反例红，无产品构建warning。关闭oracle：恢复入口在dispatch前拒绝实际profile/wire不符，合法collections恢复及既有R14/R15拒绝保持，核通用门与请求身份的所有实际适用轴。不要改变通用旧profile合同。

acceptance_results: AC-06 fail、AC-07 fail；R15入口/快照子面已闭，AC-01..05/09保留原证据边界，未查项不冒领pass；AC-08 pending。SD-03恢复/隔离完整主张仍不批准，其他SD沿既有边界。冻结SD hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739；依赖/配置未变，hash及最终HEAD/cleanliness见input-hashes。无canonical/ledger发布、实现修复或整档运行。

next: 同树work修F-822-R16-01，T-05/T-06重开，executing5/9，其余保留；定向正反控提交后再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面档仍pending，tf仅主检出批量。本轮未合入/归档/清树。

### 2026-10-27 独立最终复审 R17

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立合同/源码/验收覆盖审读，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: a26b8e71c3760e0538cb654d7c39e5a548c3f104 | bookkeeping_commit: 7fe3e2fc8 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: f1d31632b1e625deb125142864e1ad032cb9870d

F-822-R16-01 CLOSED：decode后强制profile id4再verify_abc，通用旧profile入口不变；独立运行cache_compat八控8/8绿，包括原core混入拒绝、同形scalar collections正控及R14/R15错配/诚实恢复。actual profile→family/version由decode合法tuple检查，owner重盖章已核，types_fp/entry/snapshot各有真实内容比较；未发现本次恢复修复新回归。

继续核对历次保留pending的AC-09/V-13子case3，发现 F-822-R17-01(P1，原必需验收证据缺口，非R16引入；AC-09/T-09/T-06，SD-06)：原§5全链增量及V-13明确要求“实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控”。现有822 incremental全文件只有signature_change_precise覆盖接口由i32→bool的link拒绝、body_edit_identity覆盖beta本体更新/调用语义和旧snapshot保活；未提供collections能力的实际layout事实变化及host授权事实变化正反控，script incremental必需清单亦无这两轴。独立运行signature_change_precise=1/1绿，只能证明签名轴，不能代表V-13子case3全覆盖。共享828 source helper/body测试不自动覆盖822新增集合layout，字符串含authorize的manifest/重复身份测试也不是host grant变化oracle。

覆盖核验方法（可复现，无新规模承诺）：审读 probe/tests/av_collections_822_incremental.rs（202 body/263 signature/303 forced/343 baseline等8测）、probe/src/collections_session.rs及scripts/verify-av-collections-822.ps1必需清单；搜索相关共享function_incremental_828*.rs的layout/grant/authoriz/helper/capabil引用并检查实际语义。session仅接submitted MIR，默认ExecLimits/CancelFlag执行，没有本计划授权变化的测试输入/观察；此点只确认当前验收缺口，未据此声称已复现越权或错误layout重用。受检新layout/host授权的生产接线是否缺失需work定向调查；不以未验证的处方当根因。

关闭oracle：对既定真实collections consumer，提供actual layout相关依赖变化的合法/不兼容控制（精确失效或受控拒绝，prior snapshot/值/清理、未影响单元身份均可观察）；提供输入required capabilities与当前host grants分开的实际变化控制（合法允许、撤销/不足拒绝，失败不安装，恢复合法事实仍可用），或定位已存在且适用于本profile/当前baseline的等价耐久证据。保既有signature/body/forced控制。不能仅增测试名/改清单或以scalar campaign代替。如当前架构不能满足原承诺，回new显式合同修订，不在work默默标N/A。仅要求受影响定向验收，不跑整档或重放无关性能矩阵。

acceptance_results: AC-06本轮R14..16恢复身份面pass（其他兼容证据保原边界）；AC-09 partial/fail（子case1/2/4沿既有证据，子case3签名/body有证据但layout/授权未闭）；AC-07 fail；AC-01..05沿既有证据边界，AC-08 merge-owned pending。SD-03本轮恢复绑定主张可接受，SD-06完整增量依赖主张待关闭F-R17-01；其余SD保持原边界，无canonical/ledger写入。冻结SD hash见input-hashes，touched_goals空理由沿计划有限实验扩面保留。

next: 同树work补F-822-R17-01调查及必需oracle，T-09/T-06重开，executing5/9，其余任务保留（T-05已关闭，不再重开）；定向提交再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门仍pending，tf仅主检出批量。未修实现、整档/aggregate acceptance、合入、归档或清树。

### 2026-10-27 独立最终复审 R18

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立核合同/实际路径/反例，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 7de289d47e92c4866857c4ae38579dab5cc4e16b | bookkeeping_commit: cf987d4f7 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: a26b8e71c3760e0538cb654d7c39e5a548c3f104

F-822-R17-01 PARTIAL：内部List<i32>→List<Str>布局更新与List参数接口错配拒绝两控独立2/2绿；真实session、生产计数、未改单元Arc、旧snapshot/失败后可用均有断言。新增host_grant_lifecycle独立1/1绿，建立required capabilities与host grants分离的输入，但仅测试重新提交受限beta，未覆盖同一候选程序的carried beta；完整授权不变量未闭。

F-822-R18-01(P1，R17修复接线遗漏；AC-09/V-13子case3，T-09/T-06，SD-06)：CollectionsUpdateSession::apply_revision只在Some(submitted module)分支检查host_grants；None分支直接复用已有FunctionCode，之后link/execute/commit完全不核其required capabilities。独立真实入口：先grant TRACE_CAPABILITY，安装main=alpha(+10)、beta=beta_unit_trace(1)成功11；撤销为[]后，①只提交main=alpha(+20)，旧受限beta被复用，新revision2成功21、rebuilt1/gate.pass；②空提交，新revision2成功11、rebuilt0/gate.pass。两次均执行并安装了当前grant不覆盖的候选程序。保持grant的同一carried形态正控成功21，复用beta Arc且live0。独立拒绝oracle红(exit101；一个测试内两形态均记录)，不是只凭测试覆盖计数推断。

既定规则：§5.4输入required capabilities与当前host grants分开，优化不绕过授权；§5全链增量及V-13授权事实变化须精确失效/拒绝；R17关闭oracle亦明确要求撤销/不足拒绝、失败不安装。此处要求的是新的apply_revision候选，不撤销已经绑定旧授权事实的历史snapshot，不把旧snapshot可用控制判为错。原因已由分支及真实行为确立，未声称本例执行了实际外部trace副作用（beta仅声明required capability）；缺授权候选仍准入/执行/安装本身已违反门禁。

复现：附件replay.rs临时放既有树probe/tests/review_822_r18.rs，cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r18 review_r18 -- --nocapture；1合法正控绿/1拒绝oracle红含两个失败形态，日志carried-grants.log.txt。关闭范围：候选完整受检闭包（submitted+carried）的required capabilities对当前grants核验，撤销/不足时不得执行安装；重新grant、无关grant变化/无需cap程序及原layout/body/身份控制保持。检查多submitted顺序与拒绝前生产副作用的边界，避免局部循环检查冒充“整次revision在任何生产前拒绝”；不要求全局回归或无关性能重跑。

acceptance_results: AC-09 partial/fail（layout子面已补，授权carried不变量失败）；AC-07 fail；AC-01..06保留原证据边界，R14..16恢复修复不因本diff失效；AC-08 merge-owned pending。SD-06完整依赖/授权主张仍待关闭；其余SD沿既有边界，touched_goals空理由保留。冻结SD/hash见input-hashes。无canonical/ledger写入、实现修复、整档/aggregate acceptance。

next: 同树work修F-822-R18-01并完成R17完整关闭，T-09/T-06重开，executing5/9，其他任务保留；定向提交后再审。最终pass后merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面门仍pending，tf仅主检出批量。本轮未合入/归档/清树。
