---
plan_id: PLAN-824
status: drafting
feature_name: AV独立动态Variant与可移植冻结边界
plan_revision: 3
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-20
current_step: 0
total_steps: 8
supersedes_spec_components:
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-av/dynamic-portable-values.md
  - docs/specs/auto-mir/dynamic-cast-profile.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-824] AV独立动态Variant与可移植冻结边界

## 0. 变更摘要

承接[Design44](../design/44-av-data-layout-place-and-ownership.md)及[Design43](../design/43-ac-av-typed-mir-and-abc.md)，交付DynValue、受检类型擦除/恢复与portable冻结。本文件是实施草案，当前仅授权起草，未实施；不得将计划支持面写成现状Spec或自动启动work。

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

影响入口：共享MIR/verifier/ABC/AV及typed host seam（D-823）；拟新增 `core/src/{dynamic,portable}.rs`、`probe`独立producer/恢复门及fixtures；796模型/817冻结schema只读。

拟新增 `scripts/verify-av-dynamic-824.ps1`，本计划测试前缀 `av_dynamic_824`、`test/cases/av-data/824/`、独立recipe `av-data-824`。均为未来入口，当前不存在、未运行；T-02使first-case可执行，T-06完成其余Scope。

## 4. 需求分析与背景调查

授权：2026-10-20用户认可方向并明确要求正式架构、衔接43/798、拆分计划，**先不写代码**。本轮允许主检出设计/计划簿记；未授权实施、开树、建/联系代理、改canonical Spec或切换产品。未指定实施者/模型/时间预算。

取材：[overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[774资源合同](../specs/auto-ac/data-resource-contract.md)、[record](../specs/auto-ac/source-record.md)、[text](../specs/auto-ac/source-text.md)、[782组合](../specs/auto-ac/source-text-record.md)、[796](../specs/auto-semantics/project.md)、[artifact](../specs/auto-ac/artifact-contract.md)、[测试v2](../specs/auto-test-case/profile-v2-native.md)。路径/提交/文件SHA-256见[共同输入收据](attachments/av-data-20261020/planning-inputs.json.txt)。

初稿r1观察：798 executing r2；当前合同已修订r4（类别/资源证据待收口），auto-av/auto-mir Spec主检出尚未交付；只读其树观察到MirType::I32/Bool/Fn、MirInstr/Terminator、私有LoadedModule、Frame.slots: Vec<Value>和初始化must分析。它们不是已合入依赖。既有HIR有PlaceKind::Local/FieldPath和774/782资源语义，不能另定冲突语义；796不是完整运行时类型系统。

**D-823**=前序profile实际交付，且独立最终review、冻结门禁、合入、Spec与merge收据齐全；实际启动重验提交祖先/Spec，不只看frontmatter。T-01只读准备可先做，T-02及依赖构建等D和用户执行授权。797不是本计划前置；817/818共享API消费映射按Design44 §12单写交接，不能修改其在途合同。若仍在改公共定义，先等owner交接/实际合入，不从其分支抄代码开工。

拟唯一实施树 `D:/autostack/.wt/lang-824/auto-lang`，分支 `plan-824-dev`；本轮未创建。实施/内部检查/最终独立review在同树顺序进行，主检出只记计划。

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

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/dynamic-portable-values.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/dynamic-cast-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-824独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |

前序Spec未合入时不提前写；review按D实际Spec最终确认modify/add及差异，merge才沉淀。touched_goals为空：本计划是有限实验扩面，不凭其单独核销AAC/AAV自举或生产目标。

## 6. 测试设计

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
| V-12 | AC-02/03/04/05/06；T-01/03/04/05/06 | List<closure>或record含borrow的嵌套pack/freeze；合法Dyn<T>请求已知U≠T；未知TypeRef；同owned子节点双引用/环/重复字段；恢复第n次分配失败 | 真实profile verifier/ABC/AV及store新worker；T-01冻结子例，T-03/04使可运行 | 递归不支持类型/坏树在规定准入门拒绝；合法错型take可执行并返回失败且源可用；无复制owner，恢复半图回滚；未消耗原根的普通freeze失败保持原值；计数按§5.5区分准入/运行，T-05重放适用恢复路径 | 仅本profile，不把typed fixture冒称新增源码能力 |

资源观测为纯读；正常释放后live=0，不能由reap_all掩盖泄漏。返回owned结果时先确认责任转出，测试接收者显式drop后断言0。确需runtime拒绝的无效handle与应在verify拒绝的语言借用分别断言，不以一种替代另一种。

拟命令（T-02/06创建后才能执行，当前未运行）：
- 定向：`cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-core av_dynamic_824`；probe同前缀。需证明selected>0，0例不得PASS。
- 首例/定向验收：`pwsh -File scripts/verify-av-dynamic-824.ps1 -Scope first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`（实际调用每次只给一个Scope）。
- fmt/check按该独立manifest执行；开发与修复只定向测试。禁止work/internal/final review触发整档t/tv/tt/tb/tu/taa。
- 合入：最终review后冻结HEAD，由merge执行裸`cargo t`+VM/编译器触面`cargo tv`，以及新workspace完整有限合同档；按实际diff再选其他触面，tf只主检出到期单实例。review pass必须记录merge门禁pending。

角色：用户未指定模型或分离实现者。work负责实现和反例；内部检查只表示可送审；最终review必须独立上下文、同树只读核验，不修代码；merge负责冻结门禁/规范/归档/guard清树。当前未指派或启动任何角色/代理。

## 7. 验收标准

| ID | owner_stage | 可观察完成标准与方法 |
|---|---|---|
| AC-01 | work | T-02真实首例经source支持面/typed fixture→verify→ABC→AV，内容/位型/来源和独立oracle一致，V-01/10通过 |
| AC-02 | work | §5支持语义和求值/Place规则成立；V-01/02/05/06实际反例与合法对照通过，无静默降级；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-03 | work | 已声明的copy/move/borrow/cleanup责任保真；V-02/03/06在适用面逐案断言（本profile含owned值）；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-04 | work | 非法类型/owner/状态/未知profile在对应门拒绝，副作用/执行计数符合V-04/05/10，不以panic/timeout充负例；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-05 | work | V-03/07适用故障/资源边界区分有效，真实峰值≤预算、失败回滚/清理、终态责任正确；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-06 | work | V-08/09/10真实store重启/失效/损坏与旧consumer兼容、正式AutoDown反控全部实际执行，门禁汇总失败不吞；§5.4及V-11对应动态类型与可移植授权规则实际通过；§5.5/V-12及其明确阶段计数、资源与兼容规则通过 |
| AC-07 | review | 独立核对AC-01..06、所有V与SD；无未处理findings，绑定HEAD/依赖/hash；merge门禁pending显式记录 |
| AC-08 | merge | 冻结门禁通过、SD沉淀、真实合入/归档/索引/收据，wt-guard clean后清树分支；批量回归到期按仓规 |

## 8. 执行步骤

所有任务初始未完成；current_step仅计实际完成任务，不提前勾review/merge。T-01在本轮亦未执行实施调查，仅起草取材已完成。

| Task | owner_stage | 依赖 | 文件/操作与可核成果 | AC / 验证 |
|---|---|---|---|---|
| [ ] T-01 | work | 用户授权；只读准备可早于D-823 | 核对实际主检出/唯一树/Spec/API/817/818 owner；在attachments/824/contract-decision.md冻结支持矩阵、profile/codec/访问/预算和oracle，未知项不留给下游猜 | AC-01..06；决策表含全部V、cap两侧输入、公开入口/分类；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-02 | work | T-01、D-823、注册owner交接 | §3入口完成§5.2最小whole-path；新增first-case脚本/真实recipe、同store新worker恢复；记录CPU/时间/内存与独立oracle | AC-01/06；first-case、V-01/08/10正控 |
| [ ] T-03 | work | T-02 | §3共享核心/新模块完成§5全部支持语义，source支持面adapter明确；不偷改其他profile | AC-02/03；av_dynamic_824定向正反例；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | 实现必须的拒绝/回滚/cleanup/资源hook与V-02..07补充对照，先完成所有findings再送审 | AC-03/04/05；admission/resources及真实边界事件；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe真实缓存依赖/重启重验、来源/身份；注册与旧consumer兼容，串行更新唯一公共写面 | AC-06；V-08/09，cache/compatibility；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | 脚本聚合所有Scope并透传失败；正式AutoDown V-10反控、fmt/check、受管/进程峰值，整理SD草案/任务证据 | AC-01..06；acceptance，0例/漏必需用例/skip均非绿；r2 §5.4/V-11由本步负责对应冻结/实现/反控/恢复/汇总证据；r3本步按§5.5负责V-12及case清单对应冻结/实现/反控/恢复/汇总 |
| [ ] T-07 | review | work-owned完成且提交冻结 | 同唯一树独立复核AC/V/SD、遗漏/workaround/债及健康，核依赖输入未变，填规范影响和审查结论 | AC-07；merge-owned全档仍pending |
| [ ] T-08 | merge | T-07 pass | 冻结门禁→合入→SD/ledger/catalog→archive；到期批回归主检出单实例；wt-guard后移除唯一树/分支 | AC-08；真实门禁/合入/清理收据 |

## 9. 复审记录

### 文档合同复审与r3修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，未声称上下文隔离的外部独立审稿；本轮从当前Spec/合同/反例重建判断。入审r2结论needs_replan，按用户授权直接r2→r3修订§5.5、V-12和AC/T/SD；drafting 0/8保持。出审文档合同检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，源码/运行验收及AC-07/08仍pending。无实施/开树/测试，依赖D门和本计划实施授权不因文档修订视为满足。

### r2 ABC设计采纳（2026-10-20）

stage: new；outcome: pass（合同静态预检，非实施验收）。用户认可ABC建议并要求补设计/后续计划、先不写代码。r1→r2在§5.4/6–8新增动态类型与可移植授权的V-11及AC/T责任，SD-01/02/03随之修订。drafting、0/8保持，未开树/运行测试/改Spec；旧r1起草记录仅历史。798 r3正确性实际交付仍是依赖前门；825/826不新增功能链硬依赖，共享核心继续串行交接。用户当前仅授权文档修订，不自动启动work。

2026-10-20 stage:new | PLAN-824 r1 | outcome:blocked（仅指实施handoff：用户明确先不实施，D-823及共享owner交接须开工时实证；本轮草案交付完成） | next:保持drafting，用户授权后work先T-01，T-02须满足D | changed:T-01..08/AC-01..08/SD-01..05/V-01..10。

起草质量预检见[preflight.md](attachments/av-data-20261020/preflight.md)。不是独立实现review，不计最终review轮次；未编译/运行新测试/创建树或承诺交付。

## 10. 待澄清事项

无须用户再裁定已认可的总体方向。精确opcode/布局实现/资源测量由有界T-01和T-02负责冻结，不能改变§5语义或预算；需要扩大支持/资源/兼容承诺时回new显式revision。当前依赖与执行授权门如§4；不以等待实施为由伪造通过或删AC。NaN-boxing、完整数值运算/用户泛型、共享图、async、native与CTE复合消费者另案，不是本计划“已完成后续”。
