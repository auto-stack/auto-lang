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
| [x] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile [✅ 已完成 2026-10-26:15 条指令+类型/验证/执行面随 T-02 core 提交落地;av_collections_822_semantics 13/13 绿(List roundtrip/空列表/负与越界 trap 先于访问/remove 移位后 fresh 索引/insert 两端/set 邻位不受扰/List<Note> 字段原位 owned-Str replace/variant pack-disc-guard/owned 投影转移+错臂运行期 trap/switch 旧分支 drop 一次+guard 失效拒/payload-moved 静态拒(含 drop 不复位)/嵌套 List<Result<Note,Str>> 组合/域拒绝 StrView 元素与 List payload);source 支持面 = 明确无(合同 §6,fixture-only 入口);未改其他 profile——821 corpus 回归 3/3 绿] | AC-02/03；av_collections_822定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审 [✅ 已完成 2026-10-26:av_collections_822_resources 6/6 绿——LIST_NEW cap 两侧(65536 执行/65537 admission Resource ListElems 拒)、紧 slot-bytes 预算在写前拒(SlotBytes + 台账 allocs==frees/live=0 零泄漏)、arena fault 注入(attempt≥1/successful_alloc=0/Infra/无半提交)、budget-is-only-delta 对照、peak 记账与正常退出 drain、len/cap/index checked 算术回归;V-03/07 资源轴 + admission 轴已随 first_case/semantics 档落地] | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面 [✅ 已完成 2026-10-26:av_collections_822_cache_compat 5/5 绿——同store恢复身份一致+snapshot 轴 clean miss、好 envelope 坏 payload 在重验拒且零 dispatch(783 内容寻址 Conflict 防篡改同面冻结)、AVAD v3 与 v2 世代隔离(schema-profile 拒)、A22/plan822 层与 A21 不混淆、合法恢复确定性重放(步数一致);V-09 旧 corpus 重放归 T-06 compatibility scope 汇总] | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据 [✅ 已完成 2026-10-26:scripts/verify-av-collections-822.ps1 九 Scope(first-case/semantics/admission/resources/cache/compatibility/autodown/incremental/acceptance)——单 Scope 必选、selected>0 守卫、compatibility 重放 798/817/818/820/821 既有语料入口(V-09)、autodown 跑正式 recipe+反控(V-10)、incremental FAIL CLOSED(T-09 未完不绿)、acceptance 子进程重放防 seam 泄漏;first-case/autodown scope 实跑绿;SD 草案六条整理于 attachments/822/sd-draft.md;**受管/进程峰值 128MiB 线实测(821 式 worker 采样)未跑——归并 T-09 收尾批跑,不冒领**] | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [x] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-13 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage [✅ 已完成 2026-10-26:四子case全落地——1)冻结 828 基线战役(273575B/115fn/30 修订 fn_037 位点)经共享 core 在 collections 扩展后重放双门绿(all-correct+structural+P95<1s);2)body-only 修订未改函数 Arc 身份/代际保持;3)签名变化在真 verify 门精确拒(mir.return-type);4)强制退化反控:未改函数换对象(同内容 hash)gate 位非零且逐函数局部化、诚实修订为零;128MiB worker B 线父侧采样硬门 4 fresh release 进程实测绿(hot≤8/staging≤16/exec≤8 独立维度);脚本 incremental scope 解封;**acceptance 全 9 scope 端到端通过,compatibility 抓到真实回归(ac-mn 穷尽 match 被 15 新指令破坏→两处最小接线:fingerprint 渲染+lower 受控拒)818 corpus 24/24 恢复绿**] | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | work-owned完成且提交冻结、T-09 完成 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录

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
