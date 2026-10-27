---
plan_id: PLAN-828
status: executing
feature_name: 源码到AV安装的全链路函数级增量
plan_revision: 3
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-21
current_step: 0
total_steps: 11
supersedes_spec_components:
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-syntax/snapshot-cst.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-ac/function-incremental-source.md
  - docs/specs/auto-mir/function-incremental.md
  - docs/specs/auto-av/resident-function-install.md
touched_goals: [GOAL-002]
affects: [auto-ac, auto-hir, auto-syntax, auto-mir, auto-av, auto-semantics, auto-test-case]
---

# [PLAN-828] 源码到AV安装的全链路函数级增量

## 0. 变更摘要

承接用户2026-10-20明确要求：工程初始化并得到首次结果之后，单函数body修改且签名与真实依赖事实不变时，**只重新编译并替换该函数；其他函数的中间表示和已加载代码保持不动**。完整切片为 source edit→局部语法/语义→Checked HIR→MIR→ABC→受检链接/安装→真实AV结果，不以其中一步命中缓存代替全链路。

依据[Design43 §6.4](../design/43-ac-av-typed-mir-and-abc.md#64-真正函数级增量的全链路合同2026-10-20)与[798架构核查](attachments/798/function-incremental-audit-20261020/report.md)。r2已获用户确认798后优先本计划；D798已于2026-10-20收口归档（正式周转门PASS 646.9/687.4/690.9ms），用户指令"实施"进入执行：工作树 `D:/autostack/.wt/lang-828/auto-lang`、分支 `plan-828-dev`、基面 master@f79b1b835。798基础成果和开放finding/门禁保留；**本计划验收前，完整目标未交付**。

**当前执行入口（r3，2026-10-21）**：用户授权重整 F-05b/F-01/F-02 合同，目标、支持面、阈值和仓库范围保持。实施基线 `47894c2e6747c8c06032be7a3d40bf00b90a204d`，同树 `plan-828-dev`；F-05b 已获 r23a 受影响面 PASS，作为后续重构的保留不变量；F-01/F-02 仍 OPEN。先读 §5.6–5.8、§6 补充矩阵、§8 唯一任务表，再进入 work/T-01 的有界接口决策。历史完成声明不作为当前准入；[r2全文与历史收据](attachments/828/contract-r3/plan-r2-before.md)保留。

| 当前范围 | 已验证事实 / 下一责任 |
|---|---|
| F-05b | r23a 已测取消、终态守卫、关闭拥有引用释放 CLOSED；证据绑定47894c2e6。F-01/F-02改变相关代码后须按受影响面重放，不据旧PASS覆盖新代码 |
| F-01 | 当前热路径仍构造全部签名桩Program，经整Program语义与Atom中转；work/T-01→T-02/03负责替换并绑定真实生产事件 |
| F-02 | 当前snapshot仍复制全体blocks/slots/spans形成merged，AV实际执行merged；work/T-01→T-04/05/06负责消除此执行载荷重建 |
| 其他OPEN | F-09预算、F-10 A07/实际依赖/GC、F-11 changed-only持久化/manifest、F-12正式worker/IPC/双线内存、F-14文档/SD；保持原任务，不能由本次合同重整延期 |
| 当前状态 | executing0/11；review/merge整体验收仍pending。本次仅修订计划，无实现/构建/整档/合入授权扩展 |

## 1. 目标

1. 在D798已交付的scalar-i32/bool源profile内建立常驻工程/编译/AV会话。单次正常函数体编辑只重做该函数语义、HIR、MIR、ABC和已验证执行代码；未改函数沿用相同不可变对象与代码句柄，不重新读盘、decode、verify或装载。
2. 接入787既有局部词法/CST能力和已有resolver/typechecker，形成函数体级可信检查；模块接口/环境保留，热路径不再整模块parse/resolve/typecheck→Atom render/bind→verify或整模块lower/split。
3. 保持源诊断、短路、实参源序、数值/trap、资源/授权语义；增量检查的私有凭证绑定所有实际读取的上下文，不削弱现有完整输入准入。
4. 候选单函数代码原子安装；新顶层调用选择新snapshot，在途调用链固定旧snapshot，失败/取消/迟到结果不覆盖当前可用版。可达旧版保活、退役后有界回收。
5. 从稳定会话实际编辑入队至新修订的正确结果/诊断发布 **P95<1000ms**；同时以工作计数和对象身份证明全链路增量，性能通过不能替代粒度通过。

本计划允许在唯一auto-lang实施树内修改 `experimental/ac-core`、`ac-syntax`、`ac-av-probe` 的必要公共接口与消费路径；这是相对798只读消费ac-core的明确新增写面。783/796存储/身份合同优先只读复用，若确需改变其共享语义而非薄消费，先带反例回new。正式probe/run的最小注册/父侧判断触面按实际owner更新；不改其他仓库。

非目标：全语言/多文件工程产品接线、字符串/堆对象/闭包、CTE/native/JIT、UI显示、任意签名/布局变更的原位兼容安装、活帧迁移/OSR、缓存任意guest运行结果。每个已交付profile保持原入口/支持面；本次尚未支持的产品能力明确登记，不能以本计划声称完整开发环境上线。

## 2. 架构方案

```text
驻留 ProjectSession：SourceSnapshot / 声明与接口环境 / 依赖查询图
  编辑区 → 局部重扫/声明解析 → changed-function语义检查 → 私有CheckedFunction
    → lower_function → Checked MIR unit → ABC unit → 验证及增量绑定 → 候选FunctionCode
    → prepare/commit ProgramSnapshot根 → 新顶层调用 → 当前revision结果/诊断

未变函数：相同HIR/MIR/FunctionCode引用贯穿前后snapshot；无读盘/重编/重装载
持久store：cold/restart/eviction/miss时恢复并重验；不是每次warm edit的必经层
```

类型/签名环境、函数body、依赖查询结果和运行代码分别有稳定逻辑身份及修订。函数调用绑定稳定FunctionId与签名，不把模块内数组序号、源绝对offset或代码地址作为跨修订身份。候选新函数在当前受检链接环境中校验；不把unit标记翻成executable，也不让未验证单元直接进dispatch。

ProgramSnapshot保存到不可变代码单元的引用；根与必要绑定元数据可更新，未改代码payload/IR不复制重建。顶层调用捕获snapshot，嵌套调用和递归均使用该snapshot，避免同一调用链新旧callee混用。具体私有类型/数据结构由T-01首例后冻结，不提前规定Arc/持久map的物理实现。

## 3. 技术栈

Rust、既有 `ac-syntax` 增量编辑/CST、`ac-core` scalar前端/验证器、D798 MIR/ABC/AV、783 ManagedStore与796语义身份、已有正式AutoDown probe/run及contained执行器。实际取材路径：

- `experimental/ac-syntax/runtime/src/edit/mod.rs`、`adapters/core/src/lib.rs`：局部扫描/声明复用和严格CST投影；grammar/支持差异由T-01核对。
- `experimental/ac-core/src/source/{adapter,resolve,typecheck}.rs`、`verify.rs`、`hir.rs`：抽出函数级检查，保留完整入口与私有信任边界。
- `experimental/ac-av-probe/probe/src/{driver,hir_adapter,cache,observations,worker_restore}.rs` 及 `core/src/{mir,mir_verify,abc_verify,av}.rs`：按函数lower/装载、真实会话和安装。

已存在 `probe/src/{session,function_queries}.rs`、`core/src/{function_code,program_snapshot}.rs`、测试 `function_incremental_828` / `function_incremental_828_r1` 和 `scripts/verify-function-incremental-828.ps1`。其中session/snapshot实际接线仍有F-01/F-02缺口；脚本和单测存在不表示正式recipe、持续worker和资源门已完成。函数体级语义生产接缝、无merged载荷的绑定/派发接缝及其补充测试为待work实现/冻结的新接口，见§5.6/§6；不提前伪造实际API名称。

唯一现有实施树 `D:/autostack/.wt/lang-828/auto-lang`、分支 `plan-828-dev`。实现、内部检查和最终独立review按顺序使用同树，主检出只记计划。无其他仓库写入、工具安装、代理或默认产品入口变更授权。

## 4. 需求分析与背景调查

r3授权：用户2026-10-21明确要求重整F-05b/F-01/F-02的当前执行合同，补所有权/数据流、代表性矩阵、真实生产路径检查与可执行关闭条件；保持原验收目标。沿用已授权的auto-lang单仓写面与同一实施树，不改变P95、预算、兼容面、执行顺序，不切换用户既有模型分工。本轮new只交付合同；后续work按§8承接。未设日历/token预算，不创建代理或其他聊天。

D798已满足；不再等待该已完成前置。798完成后优先828、随后817/818及820–827的顺序继续；其他仓的共享合同若确需变更仍须先回new，不通过本次重整追认。

当前事实基线：plan-828-dev@47894c2e6747c8c06032be7a3d40bf00b90a204d，原实施树clean；auto-down sibling=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。F-05b的独立受影响面PASS绑定该提交。当前热前端构造签名桩Program再调用compile_program_parts；snapshot的link_with_maps遍历并clone全部函数payload形成merged，AV执行merged。它们分别是F-01/F-02的现存缺口，不能以caller计数或包装Arc相同认定完成。

权威输入：[Spec overview](../specs/overview.md)、[HIR stage contract](../specs/auto-hir/stage-contract.md)、[snapshot/CST](../specs/auto-syntax/snapshot-cst.md)、[query/cache](../specs/auto-semantics/query-cache.md)、[AV cache/conformance](../specs/auto-av/cache-and-conformance.md)、[case architecture](../specs/auto-test-case/architecture.md)、Design43 §6.4与§8.1。来源路径和当前SHA256见[调查输入](attachments/828/contract-r3/inputs.json)。783/787/796的既有版本/身份/存储规则只读复用；拟新增Spec保持proposed，由review核定、merge沉淀。

旧首例、反复修复的根因和所有收据保留在[r2全文/历史](attachments/828/contract-r3/plan-r2-before.md)及各review附件。实施树旧contract-decision/coverage-manifest/successor-interface-contract中的完成或延期措辞不是当前授权；T-09须按本合同与真实代码回写。r3为执行机制/任务/判据的明确化修订；不废弃已验证成果，也不让r2局部PASS覆盖未来替换代码。

## 5. 详细设计

### 5.1 适用编辑与分轴失效

快路径条件：单个已有函数body内有效编辑，源profile/grammar、函数逻辑身份、签名/可见接口、实际读取的外部语义事实、target/配置/授权均未变。支持字面量、局部变量/槽数、控制流和代码长度变化；新body可改变调用结果，但调用绑定集合和其依赖事实不变。source目录/行号/根修订可更新，未改函数语义对象和已加载代码不变。

调用者只读取callee接口时，callee body改动不导致调用者重编；真正读取body/const/CTE/布局/effect/inline假设的消费者按查询依赖失效。首版不新增这些语言功能，但key/invalidator保留正确区分，不能把未跟踪依赖当“未变”。新增/删除声明、签名/配置/授权变化、跨函数编辑及词法不收敛走明确扩大失效/拒绝重载/全量重建路径，记录原因与真实范围，不计为纯body快路径PASS。

### 5.2 前端与Checked函数单元

保留787增量词法/CST能力，按当前grammar/profile核对全部已支持scalar构造。以顶层声明索引定位函数，复用未改声明的语义投影；局部词法收敛和目录offset元数据可更新，不能重新投影所有函数。body级resolver/typechecker复用签名/声明环境；消除热路径整Program Atom render/bind。已有规则和完整输入入口保留，函数级入口仍执行全部适用类型、初始化、调用/来源和资源规则。

现有auto-scalar grammar/project_decl_strict及scalar差分可复用；它们不证明下游单body语义生产已接通。T-01/D-01核对bool/短路/条件for/命名实参及intrinsic，T-02用旧/新原始源码对照保留支持。必要语法修复仍在既有配置驱动体系内，不另写平行parser或削减旧profile。

私有CheckedFunction/检查结果绑定body身份、signature/context revision、实际依赖与语义版本，仅验证器构造；不能从旧CheckedModule随意切片盖章或恢复trusted标记。跨函数签名/owner事实由受检环境提供，body检查和增量链接分工须有正反控。未变对象的绝对来源位置变化由相对span和文件映射层处理，不改代码payload或语义凭证。

context revision按实际语义事实定义，不能直接使用文件/程序总revision：单函数body改变导致源码snapshot变化，不应使所有其他函数凭证失效。保留相同对象的前提是其内容不可变且依赖事实仍成立，不能暴露可修改payload的旧句柄后继续复用原验证凭证。

按Design40，实际新增的函数HIR/语义查询产物和模块索引接入统一A07/适用A06/A10引用，不建第二套数据库。T-01冻结真实函数HIR载荷/稳定ref/版本、依赖清单和有界codec；仅变更单元产生新产物，恢复为不可信描述后在当前上下文重验，凭证不落盘。T-03落实HIR producer/恢复和真实cold-vs-warm分界，不能只定义key而未接消费。

### 5.3 单函数MIR/ABC与装载

从CheckedFunction直接lower/verify该函数；不调用whole lower再split，未改函数不重新算HIR dump/指纹。会话内命中直接引用现存不可变HIR/MIR/代码，磁盘仅cold/restart/miss按已有准入恢复。维持实际版本、授权及能力依赖，授信不能因命中而跳过当前消费条件。

沿Design43 §8.1，首版函数作为唯一根区域，身份/依赖/代码引用保持可分解接缝；不把完整函数向量、模块序号或物理帧布局固化为未来block区域公共ABI。本计划不实现block级增量、活帧迁移或无语义占位指令。

函数局部常量/类型/槽/PC及跨函数逻辑引用独立于其他函数长度/顺序。候选ABC单元走shape/语义/上下文验证，再由受检链接环境建立调用绑定；未变函数字节码/IR不重新编解码/全量验证。复用的验证凭证绑定不可变输入和上下文，不是跳过安全检查；变化的接口/授权/版本使相应凭证失效。旧完整ABC公开入口和损坏/错kind/未闭imports反控保留，不绕过unit不可直接执行规则。

### 5.4 安装、执行和寿命

prepare只构造候选代码/绑定/元数据，不修改当前可用版；完整验证后以当前期望revision提交新ProgramSnapshot根。提交核基面未过期且未取消，失败/旧候选不覆盖新结果；诊断携带请求revision，旧版结果不能标成新修订结果。

新顶层调用用新snapshot；已有调用链保持旧snapshot，其递归和嵌套callee也保持旧版。错误span按所属snapshot解释，不把旧帧指向新代码。旧代码和依赖在snapshot/frame/lease引用期间保活，退出后退役并可回收；关闭、trap、fuel、cancel、安装失败均验证释放。只支持标量代码和帧，不做活帧/对象布局迁移。

执行仍遵循语言语义：需要求值的main/调用链实际运行，不缓存PASS、最终值或跳过guest副作用；增量解释指复用已加载代码和更新入口，不自动承诺响应式值级增量求值。

### 5.5 度量、资源与有界首例

T-01在已满足D798基础上按§5.6 D-01..03完成当前真实函数级竖切与接口决策；最小源码main→helper，预热后改helper body，新结果不同、未改main实际语义/派发payload身份不变，原whole/stub管线与实际副本负控可被同一门检出，错误候选保留旧版。语法、存储和正式资源责任仍按原任务交付；首例失败先给具体阻碍与替代设计，不降级到全模块重算后勾验收。

冻结798现有单文件273575B/115函数负载，编辑fn_037的30个不同修订作为纵向基准；另覆盖函数位于前/中/后、局部槽数及源码长度/行数变化、稳定调用/递归。新结果与独立预期及冷编参考一致。签名冲突另测准确诊断，**合法body成功更新**是主性能门，不拿早退sig耗时代表安装。

正式入口一次启动并初始化工程，首个正确结果/ready后开始测量；每次从实际编辑入队到父侧校验当前结果/诊断并发布，包含必要IPC、排队、依赖核对、编译/验证/绑定/安装、执行和比较。不得在计时外预编变更函数或安装候选。首启动/初次装载和重启恢复单列，恒定worker身份校验可在ready前完成，变化的actual准入仍在每次发布门内。写测试报告另报，不替代结果可见时刻。

每性能场景≥30样本，保留全部raw/失败、P50/P95/P99、阶段wall/计数/IO、输入与二进制身份；P95<1000ms且全部正确。结构门要求纯body快路径 `unaffected_{project,resolve,typecheck,hir_build,mir_lower,abc_emit,decode,verify,load}=0`、`whole_module_{frontend,lower,closure_roundtrip}=0`（名字可等价，绑定真实函数入口）；变更函数新内容miss时各生产阶段真实执行；已受检内容hit按T-04允许复用同一对象，仍核当前上下文/依赖并记录真实hit，不伪造emit/verify计数。安装集合恰为一个FunctionId。未变HIR/MIR/代码对象identity、generation及内容hash前后相等；仅hash相同或测试自报counter不足。目录/词法边界检查/根引用更新单列，不宣称整次工作O(1)。

继承D798实际cap及r5预算，不追加一份会话预算：输入/表/验证/执行限额和受管32MiB（hot8/staging16/execution8），A前端峰值≤224MiB/B非前端残差≤128MiB的既定归因方法。驻留查询图、HIR/MIR/代码、候选/旧snapshot、索引和分配器暂存都计入真实峰值，不能移出基线隐去。T-01核Spec/实际cap交集，常驻表示若无法满足先给同口径证据回new，不暗增上限或用LRU全量重编冒充快路径。512真实修订+GC/旧调用释放验证回收/稳定水位，用户cache哨兵不动。

### 5.6 F-01/F-02 的数据流与接口责任

本节是r3当前合同，§1/§5.1–5.5的目标、profile、限额和准入规则继续生效。物理类型/API由T-01在现有owner内抽取；下面的“环境/函数结果/绑定视图”是接口职责，不是已实现类型或另建编译器的许可。

```text
冷初始化：现有完整前端 → 受检声明/类型/授权环境 + 每函数独立不可变产物
热编辑：snapshot/CST局部更新 → 单声明投影
  → [F-01] 复用环境，解析/检查changed body，产生受检函数HIR及实际依赖
  → 函数MIR/ABC与准入 → PreparedEdit（新代码+候选来源/绑定元数据）
  → [F-02] 验证候选与当前接口兼容，构造只持代码引用的snapshot及绑定元数据
  → 原子提交根 → AV从捕获的snapshot读取实际函数单元并运行
```

| 接缝 / 责任owner | 必须接收 / 产生的事实 | 合同边界 |
|---|---|---|
| `ac-core/source/{resolve,typecheck,adapter}.rs`，T-02/03 | 单个投影声明、当前受检环境、profile/caps/版本/来源；产生该函数语义结果和实际查询依赖 | 复用已有resolve_fn/check_block等规则；不得通过全部stub Program重走resolve_with/typecheck_with/Emitter::plan_module/render/bind。局部受检适配的物理方案由T-01决定，不能信任手写HIR或少查语义规则 |
| `ac-core/verify.rs` + `probe/function_queries.rs`，T-03 | 私有函数凭证绑定owner/domain/函数身份、body内容、实际读取事实和语义/授权版本 | 凭证不能由调用方提供“正确字符串”自证；环境元数据读取可单列，不能重建未改body/HIR。读取接口与读取body事实分轴，保留原V-03要求 |
| `probe/session.rs` + `hir_adapter.rs`，T-04 | 受检函数→独立MIR/ABC/代码，或在适用上下文下命中同一受检对象 | 未改产物不dump/clone/encode/decode/verify/load；内容命中保留自身mint来源和代际。A07生产/恢复仍由T-03/T-07交付，不能随本次前端拆分删去 |
| `core/function_code.rs` + `program_snapshot.rs`，T-05 | 不可变已准入代码单元、稳定函数身份/签名、候选引用；产生受检绑定和snapshot | snapshot可更新目录、Arc/等价引用及必要绑定元数据；不得克隆未改blocks/instrs/slots/spans或拼接全程序类型/常量表后重写其指令。跨函数引用不是模块数组序号/物理地址ABI |
| `core/av.rs`，T-05/06 | 捕获的snapshot、帧内函数身份/局部PC、函数所属类型/常量/槽与绑定视图 | 普通执行、execute_program_steps、resume_program须使用同一单元派发机制；嵌套/递归沿捕获snapshot选callee。不得保留只供身份断言看的Arc，同时实际执行另一份merged副本 |

允许遍历接口目录、单列词法边界/来源映射/绑定元数据更新；不承诺整次编辑O(1)。允许冷初始化和显式非快路径重建；不能把正常body编辑重新分类为冷启动，也不能把必要工作移到计时窗外。已有完整ABC/Driver入口和原有语言语义保持。

**T-01有界决策输出**：在实施树提交实际接口与首例，在主检出记录拟建 `attachments/828/contract-r3/interface-decision.md`。一次完整调查与 `main→helper` 竖切应回答下表；未回答的条目阻止依赖它的广泛改造，不阻止其他已授权独立工作。不能仅提交接口空壳、缓存key或counter图作为可行性证明。T-01只要求双函数的最小真实路径；完整scalar支持与所有代表矩阵由T-02..06完成，原有未覆盖能力不能用首例PASS授交付。

| 决策ID（均属T-01交付，不另计任务） | 需要冻结的内容 | 判定证据 |
|---|---|---|
| D-01 | 既有resolver/typechecker如何抽取可复用受检环境和单body检查；HIR构造与验证边界；上下文/实际依赖版本如何绑定 | 实际符号/调用图、旧新源码冷编对拍；changed-body有效/错误各一例；调用者只读callee接口的控制；当前旧stub管线反控能被真实入口观察检出 |
| D-02 | 单元局部type/const/slot/PC如何消费，跨函数调用如何经snapshot绑定；现有三执行入口如何共用同一派发逻辑；持久codec/公共接缝是否改变及适用版本 | 双函数新旧结果、实际派发payload身份、不同槽数/常量的组合；错误绑定拒绝；强制复制未改payload后结构门必红。不能把问题留给后续执行器临时重新merge |
| D-03 | session、候选、snapshot、帧、history与store的拥有关系，提交前后的来源/绑定元数据责任和失败回收点 | §5.7所有权表与当前代码逐项对应；复用r23a正反控并说明F-01/F-02改动后重放集合；必要预算接缝保留F-09责任，不增加上限 |

### 5.7 F-05b 与事务所有权的保留合同

F-05b在47894c2e6已测范围CLOSED，不重新要求实现已通过代码。下表用于约束F-01/F-02改造后的兼容和重新验证。逻辑状态可沿用当前bool/私有类型；不强制引入新的公共状态枚举。

| 拥有者 / 资源 | 保活期间 | 释放 / 可见性oracle |
|---|---|---|
| Open session：当前根、每函数HIR/code/history | 当前可用修订及有界历史；只由成功提交替换当前语义 | 准备、失败、取消、迟到/foreign拒绝都不改变当前root/source/revision及其来源映射；成功提交前不得先改当前来源/绑定再尝试链接 |
| PreparedEdit：候选HIR/code、候选源/语法/来源/绑定 | 候选由外部持有直至commit/cancel/drop；history-hit可与当前代码共享 | consume后释放自身拥有引用；closed commit/cancel拒绝也须释放候选。外部候选尚未处置时，代码Weak存活属于合法保活 |
| ProgramSnapshot：单元引用+自身绑定/来源元数据 | session或外部pin/PausedRun持有 | 无外部owner时close撤销session引用即可使旧根死亡；代码不反向强持有snapshot形成生命周期环 |
| PausedRun/帧 | 固定旧snapshot到完成、trap、fuel或执行取消退出 | 旧链使用旧callee/来源；最后拥有者退出后Weak死亡。执行取消按既有1024指令轮询，不承诺短链即时抢占；测试要实际到达轮询点 |
| Closed session | 保留必要轻量诊断/修订信息；当前实现current()返回无代码vacant根，resident()为None | close幂等；run为受控Infra，prepare/edit为受控Err，commit/cancel为拒绝；无外部pin的根/HIR/code在close即死。vacant普通/分步执行受控Infra，不执行旧程序、不panic |
| store/root/lease | 实际持久化/恢复消费窗口 | close释放session的store句柄；产物GC/lease/512修订水位仍属F-09/F-10/F-11，不能用history_len或纯Arc代替 |

Closed getter当前行为和r23a证据需保留；若替换为等价私有表示，先在D-03写清接口兼容，不允许靠新增panic/重新暴露旧程序实现释放。当前源/CST/接口元数据仍计入F-09总预算；本表不声称close后整个进程内存为零。

### 5.8 真实生产路径检查与可执行关闭条件

以下是观测职责，等价名称可在D-01/D-02冻结；不是已存在的日志schema。计数必须来自生产者/消费者，不能在session外层“推算只有一个函数”后给未改项填零。ready/初次结果之后清晰划分采样窗。

| 检查轴 | 挂接位置与证据 | 区分性负控 |
|---|---|---|
| F-01每函数project/resolve/typecheck/HIR-build/verify | 实际project_decl_strict、resolve_fn/等价单body入口、check_block/等价入口、HIR producer/verifier；事件带owner、稳定FunctionId及阶段，区分body工作与接口元数据读取 | 在同一会话测试配置中强制执行旧全部stub Program→compile_program_parts路径；即使最终值正确、caller外层counter仍为0，同一结构gate必须失败 |
| F-01整Program路径 | compile_program_parts及其整模块resolve/typecheck/Atom plan/render/bind入口的真实热窗调用；单列初次/cold/fallback | 旧管线不能被空函数体、不输出HIR或缺事件伪装为快路径；缺必需观测也应拒绝出PASS证据 |
| F-02实际派发对象 | AV获取函数payload、常量/槽/PC和callee绑定的实际位置；同时核snapshot/代码代际、不可变内容与进程内实际payload身份 | 强制复制至少一个未改函数执行payload并让AV实际使用副本：结果仍可正确，但原结构gate必须失败；仅克隆未被执行的包装对象不算完成负控 |
| F-02安装范围/失败状态 | 受检绑定/prepare/commit入口与普通/分步/resume的真实callee选择；对比当前根、source/revision、来源/绑定元数据 | 好单元+坏调用签名/缺import、foreign/迟到/closed候选必须在安装前拒绝；失败前后旧结果与span不漂移 |
| F-05b资源拥有关系 | 真实root/HIR/code Weak；候选、外部pin与PausedRun最后拥有者释放；vacant/终态公开入口 | 不把drop(session)或history_len=0当close释放；不通过清空外部pin来使Weak死亡 |

进程内地址/Arc身份只作该轮观测，不进入公共ABI或持久key。源码/二进制/配置身份、原始输入与实际mutation、每case选择数和原始退出码必须可重放。复用现有runner和门禁，无需为本次合同另造通用测试平台。

**关闭判据**：F-01须C01-01..05全部满足，真实负控红且合法控制绿，受检环境/单body结果进入真实session；F-02须C02-01..05全部满足，实际三执行入口使用不可变单元且强制副本反控红；F-05b保持C05-01..04及r23a证据边界。每包提交“根因→所有受影响路径→机制改变→正反控→剩余边界”关闭表，给出冻结提交和实际输入。旧例通过不能自动给替换后的前端/执行器授PASS；同一不变量连续两次失败先回T-01补诊断/设计调整，再做下一次修复，不能只追加特殊分支或要求重做已通过且未变的全部测试。

### 规范增量

下表target相对 `docs/specs/`；D798已完成，T-01/T-09据现行Spec核实际add/modify。C01组细化SD-01/02/05，C02组细化SD-03/04/05，C05组保留SD-04的取消/关闭规则；SD-06正式入口仍由T-08/09交付。canonical/ledger仅merge沉淀，当前不写已实现状态。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-ac/function-incremental-source.md | 整Program前端→受检接口环境+函数体查询、来源和失效轴；函数HIR实际A07产物/恢复及统一引用 | 完整前端增量 | AC-01/02/05 |
| SD-02 | modify | auto-hir/stage-contract.md、auto-syntax/snapshot-cst.md | 完整模块入口保留→私有函数凭证和真实CST消费；支持不减 | 不复制前端/削弱验证 | AC-02/05 |
| SD-03 | add | auto-mir/function-incremental.md | whole lower/split→函数lower/verify和稳定跨单元引用 | 中段真实增量 | AC-03/05 |
| SD-04 | add | auto-av/resident-function-install.md | 完整LoadedModule→函数代码/受检环境/事务snapshot及寿命 | 装载执行增量 | AC-04/05/06 |
| SD-05 | modify | auto-av/cache-and-conformance.md | 磁盘命中→内存复用/恢复与全链路结构/P95/资源证据 | 不能只看emit/cache计数 | AC-01/06/07/08 |
| SD-06 | modify | auto-test-case/architecture.md | one-shot保留→正式持续会话ready/edit/result准入及失败门 | 真实端到端 | AC-07/08 |

## 6. 测试设计

V-01..09原合同继续有效。已有真实入口：probe/tests/function_incremental_828.rs、function_incremental_828_r1.rs；ac-syntax的scalar_projection_；review-r23a的close-controls.rs、close-adjacent.rs及run-controls.ps1。既有代码/输入未变时可按记录理由复用证据；替换前端、绑定或执行器时按受影响矩阵重放。现有测试86/17/22等总数是47894c2e6的历史证据，不是r3新增检查已运行。

| Case ID | AC / invariant | 输入/状态 | work owner / 实际入口责任 | 区分性oracle | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02/03/04 仅一函数变化 | main→helper，改值/槽数/块数；函数前中后三位置 | T-01首例、T-02..06真实edit API/生产入口 | 新结果正确；未改HIR/MIR/代码同一对象/代际；未改生产入口调用0；安装集合恰为helper | scalar；非mock阶段模型 |
| V-02 | AC-02/05 来源/语法 | helper内增加行/语句引起后续函数位移，语法错误/恢复 | T-02/03真实CST/语义/诊断 | 未改函数不重新投影/检查，旧/新trap span正确；错误不安装，修好后出新结果 | 含词法边界；显式fallback不算快路径 |
| V-03 | AC-01/05 精确依赖 | 同名异owner、只读callee接口/真实body事实查询；签名/配置/授权/版本改变 | T-03/04/05真实query/verify/loader | 只读接口caller不重编；已读取事实变化则消费者失效；错上下文/owner/授权必拒或正确扩大集 | 不新增CTE/inline；依赖探针走真实query核心 |
| V-04 | AC-03/04/05 信任 | raw unit/坏ABC/错kind/越界ref、好payload+坏import/context | T-04/05真实公开准入/installer | 准入前dispatch/install=0，旧根/代码/结果不变；合法循环/递归正控过 | 内存+恢复，unit不可直接执行 |
| V-05 | AC-04/06 版本可见性 | A执行至确定性屏障，安装B后新顶层调用B，A再调用/递归 | T-06/07真实AV和host调度屏障 | A整链旧结果，B新结果；无callee版本混用；未改代码同一 | 不用sleep竞态，不迁移活帧 |
| V-06 | AC-04/06 失败/迟到 | 候选失败、r1慢/r2先提交、取消/关闭、trap/fuel | T-06/07真实prepare/commit/发布 | 当前根不被失败/r1覆盖；旧结果标旧revision；真实引用/lease释放 | 不回滚已发生guest副作用 |
| V-07 | AC-01/06/08 恢复/回收 | fresh、会话hit、restart、512真实编辑+GC、旧snapshot持有/释放 | T-07/09真实store/runtime | 恢复重验；持有时旧代码可用，释放后回收/水位收敛；每轮值独立正确，哨兵不变 | 恢复不冒充warm更新 |
| V-08 | AC-07 双门 | 冻结273575B负载30新revision及槽/控制流变化；sig错误另报 | T-08正式持续会话+父侧judge | 编辑到当前结果P95<1000ms且全正确，结构门同时过；sig早退不代替更新 | 单文件profile，非普遍工程承诺 |
| V-09 | AC-07/08 反控 | 故意whole lower/全重装载、坏actual/缺样本/skip、gate-ms=0、资源越界 | T-08/09脚本/真实入口 | 每种使gate非零，粒度红不能被时间绿掩盖；raw含失败，越界受控且旧版可用 | counter绑定真实入口并以身份/trace复核 |

### r3代表性补充矩阵（当前必需断言）

每行均为原AC/V的具体化，owner_stage=work；不扩大语言profile、性能规模或取消时延承诺。输入采用既有scalar源码和真实源编辑，包含正常数值与相邻槽/控制流变化；代表性组合不等于任意语法笛卡尔积。T-01冻结实际新API/fixture/filter映射，后续证据按case ID回写，不能仅报套件总数。

| Case ID | AC / V / finding | work责任与实际/待建入口 | 输入/状态与区分性oracle | 当前证据/边界 |
|---|---|---|---|---|
| C01-01 | AC-01/02；V-01；F01 | T-02/03；session.prepare_edit→待建单body语义入口，生产事件挂resolve_fn/check_block及HIR producer | helper(n int)返回n+1、main调用helper(2)，仅+1改+2，值3→4；main/tail的实际语义对象身份及body入口调用保持不变/0。强制旧stub整Program路径值可对但同结构gate必须非零 | 待实现/运行；源码末端词法不收敛走明确fallback，不计快路径 |
| C01-02 | AC-02/05/08；V-02/03；F01/F06/F08保留 | T-02/03；真实单body入口对照compile_scalar_source | i32/bool混合参数、命名实参、前向/递归、短路、条件for、局部槽数与长度变化；trace在changed及unchanged callee两位置，有授权正控/无授权负控；结果、trace、诊断stage/code/span与冷编和独立预期一致 | 用当前scalar fixtures；不新增CTE/text/record |
| C01-03 | AC-01/05；V-03；F01/F10 | T-03；FnEnv/待建实际依赖查询core与credential消费 | callee body改动只读接口caller不重编；实际读取的intrinsic/owner/domain/caps/版本变更须拒绝或正确失效；已读body事实变化使消费者失效、未读事实不误失效 | body事实走真实查询core探针，不用伪造失效计数，也不要求新增CTE语言语法 |
| C01-04 | AC-02/05/06；V-02/04/06；F01 | T-02/03；真实body检查→prepare/commit拒绝 | 类型错误、未初始化/控制流缺返回等已有拒绝规则；失败保持旧root/source/revision、旧函数来源和可执行结果。修好后可继续编辑 | 不从旧CheckedModule任意切片盖章；实际错误输入及冷编诊断共同留存 |
| C01-05 | AC-01/02/05；V-01/02/09；F01 | T-01/02/03；实际入口观测与同一结构gate | body长度/行号变化不使未改语义凭证按全局source revision失效；新增声明/签名变更/跨声明编辑/NoConvergence明确拒绝或扩大集；强制整Program和缺必需观测都不得PASS | 目录/词法/接口元数据单列；不把显式非快路径当已过性能样本 |
| C02-01 | AC-01/03/04；V-01；F02 | T-05/06；link_with_maps替换接缝→AV实际函数fetch | 改helper后新值正确，AV实际读取的未改函数/blocks/指令/槽/来源payload仍同一不可变对象及代际，不仅是外层FunctionCode Arc相同；安装集恰为helper | 待改造；payload hash与进程内身份/实际消费事件联合，地址不进入ABI |
| C02-02 | AC-03/04/05；V-01/04；F02 | T-04/05；单元局部表读取与受检callee绑定 | 函数前/中/后位置；helper局部常量、槽/块数及代码长度变化；caller/callee含不同常量和i32/bool签名，命名实参顺序保持；不因全局表序漂移串用常量/槽/函数 | 复用现有合法语法；不要求签名变更原位安装 |
| C02-03 | AC-04/06；V-05；F02 | T-06；execute_program/execute_program_steps/resume_program | 调用链在真实步数屏障暂停，安装新helper；新顶层取新版，旧链再调用/自递归/互递归仍取旧版，值/trace/来源正确。三入口均消费单元视图 | 不用sleep、不迁移活帧；只换入口包装未改dispatch不算完成 |
| C02-04 | AC-04/05/06；V-04/06；F02/F05保留 | T-05/06；真实链接与提交失败出口 | 已准入好单元+错签名/缺import、foreign/迟到/closed候选：安装前拒绝；root/source/revision和当前来源/绑定元数据不变，随后合法提交正确。需要触发实际失败分支 | 可用组件内测试或仅测试hook进入实际consumer；不新增不可信公共铸造口 |
| C02-05 | AC-01/04/07/08；V-09；F02 | T-05/06/08；实际派发副本负控→原结构gate | 强制复制一个未改执行payload且AV真的使用副本：值可对但结构gate必须红；正常控制绿；普通/分步/resume不得保留另一条全merged派发旁路 | 不以手填FormalReport或只clone未执行包装对象充当mutation |
| C05-01 | AC-04/06；V-06；F05b | T-06保留；cancel_prepared/prepare/edit/commit/run | 取消不安装，兄弟候选仍可提交，迟到/foreign拒绝不污染；closed各入口受控、close幂等，源/修订不漂移 | r23a旧close6/6已验证；修改相关代码后重放 |
| C05-02 | AC-06；V-06/07；F05b-C01 | T-06/07保留；close + root/HIR/code Weak | 无外部owner在close即死；退役history代码死亡；不能用drop(session)或history_len代替。外部pin存活且正确，释放最后pin后可回收 | r23a永久释放例及close-controls；完整store GC/预算不由此代替 |
| C05-03 | AC-04/06；V-05/06；F05b | T-06/07保留；history-hit PreparedEdit、真实PausedRun | 外部候选合法保活，closed commit/cancel消费后Weak死亡；暂停链close后正常完成/实际取消，最后root/code Weak死亡，mapped span正确。新dispatch还须覆盖V-06原fuel/trap退出 | r23a close-adjacent3/3已有证据；fuel等完整失败退出随T-07原矩阵，未冒称r23a已测所有组合 |
| C05-04 | AC-04/06；V-04/06；F05b | T-06保留；closed current/resident + vacant普通/分步入口 | 非零revision、entry保持；vacant不持旧代码/来源图，普通/分步执行受控Infra，0/1/10步预算不panic；resident缺席；重复close同一空根 | r23a独立控制已过；等价表示调整须按D-03记录并重放 |

现有定向命令（须核实际选择数、退出码及库/二进制身份）：

    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1
    cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-r23a/run-controls.ps1 -Source close-controls.rs -Filter close_review_
    powershell -NoProfile -ExecutionPolicy Bypass -File D:/autostack/auto-lang/docs/plans/attachments/828/review-r23a/run-controls.ps1 -Source close-adjacent.rs -Filter close_adjacent_

3e60af2c8已新增probe/tests/function_incremental_828_contract.rs，现有filter为contract_828_frontend（源码4个test）与contract_828_dispatch（源码1个test）；不是全部C01/C02矩阵已落地。work收据报告4/4+1/1，本次合同核对未重跑；interface-decision.md的frontend“5/5”与选择表4须据raw纠正。实际API内部测试归所属ac-core/core owner，名字/命令/断言映射仍由T-01记录。新文件存在、缺观测gate(None,None)或诊断旧merged缺陷，均不能单独作为新机制已过结构门的证据。不得零选择记PASS。

角色沿用用户既定实施/独立审核分工（用户此前说明GLM-5.3实施、ChatGPT 6.1 Sol高负责计划和审核；此处记录分工，不冒称运行时模型身份）。工作、检查、审核顺序使用同树；不新开模型专用checkout/agent。每个能力包可做独立受影响面review，但只有T-09全部work-owned完成才进入T-10整份最终审核；局部PASS不允许阶段合入。开发/修复只跑相关反例、模块检查和必要cargo check；裸t+tv及必要触面档由最终pass后的merge阶段在冻结提交统一执行，tf主检出到期单实例。


## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 稳定会话/真实变更和依赖集；V-01/03/07及C01-01/03/05、C02-01，未改语义对象与实际派发payload不变，无全量重算伪命中 |
| AC-02 | work | 函数级语法投影/resolve/typecheck/HIR验证实际接线，V-01/02/03及C01-01..05，无整模块热前端；真实旧stub管线负控必红 |
| AC-03 | work | 单函数MIR/ABC生成/检查，V-01/03/04，未改lower/emit=0，无whole lower/split |
| AC-04 | work | 局部受检装载/原子安装、新旧snapshot可见性；V-01/04/05/06及C02-01..05、C05-01..04，无未改代码重装载/执行payload重建 |
| AC-05 | work | 凭证/依赖/授权/owner/版本/来源门，V-02/03/04；完整输入原支持面不减 |
| AC-06 | work | 旧代保活、失败/关闭/退出真实回收，V-05/06/07/09及C02-03/04、C05-01..04；继承预算，512修订无无界增长；r23a仅覆盖已测拥有引用范围 |
| AC-07 | work | V-08/09正式结构门及P95<1s同时过，≥30新编辑、raw/反控完整，不含初次初始化或移出当次必要工作 |
| AC-08 | work | V-07/09必需子case、旧支持面/实际消费者兼容、SD/版本映射完整，无required skip/缺oracle |
| AC-09 | review | 冻结提交/依赖/配置/输入绑定的独立最终逐AC/V/SD验收，merge门禁pending显式 |
| AC-10 | merge | 冻结根裸t+编译器/VM tv及实际触面档、独立workspace/正式兼容通过；Spec/ledger/归档/guard收尾完成 |

## 8. 执行步骤

下表是唯一当前任务表；稳定T/AC/V/SD ID与11项计数保持。已有子项PASS不自动勾整任务，当前0/11；历史实施细节已移至r2快照/§9链接。F-05b作为已闭合保留项，不另造重复任务。

**当前执行入口（2026-10-21，技能7703bb6就绪核对）**：基线plan-828-dev@3e60af2c8aa2c287633f4700b377714e94c4ff4f，同树clean；可继续work/T-01的有界修正和真实最小首例，尚不具备直接进入T-02或T-05/06全面换接的前置证据。验收规则已清楚，无新的用户范围决定。

| 既有任务内的就绪缺口 | 当前事实 / 下一步 |
|---|---|
| T-01/D-01（R-01/02） | stub_resolved深clone未改body后清空；scoped计划/render/bind仍生产全部stub bodies；receipt固定1与gate(None,None)未证明同门检出实际旧路径。先修真正单body生产及真实同门正负控，不能把此路径改名接口目录后直接接session |
| T-01/D-02（R-03） | merged的身份/内容缺陷已量化，新单元dispatch/局部表/受检绑定最小正例及AV实际副本负控未证明。冻结索引空间和三入口共用取数路径，完成原D-02有界首例后再交T-05/06 |
| T-01/D-01/D-03（R-04） | recheck成功会改env；须明确候选环境/当前环境的基面、prepare隔离、commit发布及close拥有关系。最小事务首例先证，完整状态矩阵随原T-02/06交付；禁止靠全env深clone解决 |

本表只是对既有§5.2/5.4/5.6/5.8与C01/C02/C05的当前证据核对，不新增任务/验收门，不改变r3语义。详细事实、关闭oracle和GLM下一轮指示见[就绪报告](attachments/828/contract-r3/readiness-20261021/report.md)、[GLM任务单](attachments/828/contract-r3/readiness-20261021/glm-next-round.md)。T-01当前work声明保留为历史输入；本次没有给该实现授最终PASS。F-05b r23a已闭不变，其他OPEN不变。

| ID / 状态 | owner_stage | 依赖 | 文件/符号与当前交付 | AC / 关闭检查 |
|---|---|---|---|---|
| [ ] T-01 | work | D798已满足；r3授权已有 | 在现有core/source/syntax/probe上完成§5.6 D-01..03接口决策与真实双函数竖切；主检出维护现有contract-r3/interface-decision.md，实施树提交代码/实际API/观测映射。冻结schema/授权/资源接缝与新测试选择表 | AC-01..07；C01-01/C02-01最小正控及真实旧管线/副本负控；未解决设计先有界诊断，不广泛改造或降低要求 |
| [ ] T-02 | work | T-01前端/来源决策 | ac-syntax既有局部投影复用，ac-core/source抽取受检接口环境与单body resolve/typecheck，session不再构造全部stub Program；生产入口观测落在真正工作处 | AC-02/05；C01-01/02/04/05，scalar_projection_差分及旧支持面 |
| [ ] T-03 | work | T-01/D-01、T-02 | verify.rs/function_queries：函数HIR producer/私有凭证、真实依赖读取/失效、上下文绑定；A07有界codec/生产及当前上下文恢复重验与T-07实际接通 | AC-01/02/05；C01-01..05/V-03/04/07；F01前端子项闭合不代表A07任务整项完成 |
| [ ] T-04 | work | T-03相关函数接口 | hir_adapter/cache/mir_verify：CheckedFunction直接单函数lower/验证/ABC；适用上下文内存hit复用，保留内容历史与各自mint来源 | AC-01/03/05；V-01/03/04及C02-02；未改lower/emit/decode/verify/load=0 |
| [ ] T-05 | work | T-01/D-02、T-04 | function_code/program_snapshot/av：无merged执行payload重建的不可变单元、受检绑定及实际派发；局部const/type/slot/PC消费与稳定函数身份 | AC-01/04/05；C02-01/02/04/05，强制实际副本反控必红；单元不可直接执行等旧准入保持 |
| [ ] T-06 | work | T-05；F05b证据可先复用 | session/AV事务：prepare仅候选、成功后原子更新根及来源/绑定；新旧snapshot可见性、三执行入口共用派发；取消/close/vacant/最后拥有者兼容 | AC-01/04/06；C02-03/04、C05-01..04；F05b在r23a已闭，相关重构后按影响重放 |
| [ ] T-07 | work | T-03产物、T-06生命周期 | store/A07实际恢复消费者、changed-only持久化、manifest required引用/恢复与lease/GC；驻留/候选/旧代预算与失败释放；512修订及哨兵 | AC-05/06/08；原V-02/05/06/07/09和C05-02/03；F09/10/11保留，无必需skip |
| [ ] T-08 | work | T-07 | 正式持续worker/IPC/父侧judge、ready/edit/result及observations/raw/双线内存；真实全量/副本/坏actual/缺样本/越界反控 | AC-07/08；V-08/09+C02-05，≥30新修订、P95<1000ms且结构门同轮过；进程内测试不代正式门 |
| [ ] T-09 | work | T-08；可随各包维护映射 | 当前case→入口/fixture/断言/原始证据→AC/SD/版本清单、旧消费者兼容、817/818及820–827接缝、具体规范增量文本/告警清理；更正旧完成/延期措辞 | AC-01..08；全矩阵有owner及实际结果，无OPEN必需项才execution_done；各包冻结提交，禁止只交测试总数 |
| [ ] T-10 | review | T-09完成 | 同一clean冻结树独立逐AC/V/C/SD审核；复核实际生产路径、负控和遗漏/绕过；受影响旧证据复用理由明确 | AC-09；最终pass才reviewed，merge-owned整档pending显式；不借局部PASS授整份通过 |
| [ ] T-11 | merge | T-10最终pass | 冻结裸t+tv/必要触面及consumer门、Spec/ledger/归档/guard；tf到期主检出单实例 | AC-10；实际收尾完成才交付/11项全完成，不在review或开发阶段提前整档 |

## 9. 复审记录

2026-10-21 stage:new(contract-readiness) | PLAN-828 r3 | outcome:pass（仅合同可交work/T-01有界修正与最小首例，非实现PASS） | implementation_readiness:T-02/T-05/06全面换接前置尚不满足 | baseline:plan-828-dev@3e60af2c8aa2c287633f4700b377714e94c4ff4f，同树clean | applied_skill:auto-musk@7703bb6 new/work+shared verification-contract | findings:R-01脚手架实际深clone/全部body生产、R-02固定receipt及缺收据负控不证明真实同门、R-03新单元派发首例/副本反控未成立、R-04候选语义环境的事务/close责任待冻结；均映射原T-01/D01..03与C01/C02/C05，未增范围 | changed:§6真实测试入口状态、§8当前执行入口、§10下一步及本收据；原AC/V/C/SD/T、owner、目标/profile/P95/预算和executing0/11不变，plan_revision仍3 | evidence:[静态报告/输入哈希/GLM任务单](attachments/828/contract-r3/readiness-20261021/report.md)，原Plan及interface-decision已保存；未运行cargo/docs_gen、未改实现/Spec/ledger、未启动GLM/新chat | evidence_limit:不是3e60af2c8最终代码审核；既有工作测试数不冒充本轮复现，尚未运行的控制保持待测 | pending:最终pass后的merge-owned整档/consumer/收尾门 | next:同树work按任务单先纠正D01再补D02/D03最小首例，冻结提交与定向raw后做T-01受影响面检查，通过后按原依赖继续。

2026-10-21 stage:work(T-01 接口决策与竖切) | PLAN-828 r3 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | code_commit:3e60af2c8 "feat(ac-core): D-01 single-body checked recompilation seam, D-02 dispatch observation" | 决策与竖切详见[interface-decision.md](attachments/828/contract-r3/interface-decision.md)；要点：
D-01→CheckedEnv(冷编resolve_env保留) + recheck_function(签名漂移受控拒/真resolve_fn/O(1)槽位换入/check_body_in单查/失败回滚) + emit_function_body(改函真body计划+其余签名脚手架stub_resolved，单文档callee DefRef可绑)。竖切两发现：①单def文档需callee存在性→脚手架=接口目录遍历，emit层新旧等价，接缝真实收益在resolve/typecheck按body定界——env_walk_stats生产者计数（冷N/N、每编辑恰+1/+1）+BodyEmitReceipt按"缺必需观测即拒PASS"构成§5.8 F-01门；②merged派发对未改函数不仅身份红、**内容也漂移**（改函增长重索引merged类型/常量空间）——单元载荷内容恒等，F-02量化更强
D-02→决策冻结：链接期绑定表+三入口共用dispatch消费单元局部不可变MirFunction，不重建merged；T-01新增ProgramSnapshot::dispatched_payload_same_object/same_content（merged生产者自报）；代码身份轴绿+强制副本负控红已证，派发轴当前红如实断言（T-05/06落地翻绿）
D-03→§5.7所有权表与3e60af2c8代码逐项对应+F-01/F-02改动后重放集合（r23a close面/62重放/r1+contract+兼容面）
证据→contract_828_frontend 4/4（C01-01冷编逐字节对拍/计数3-3/门双态、C01-04真类型错+环境不中毒、签名漂移拒、caller保留）、contract_828_dispatch 1/1（身份轴+副本负控+派发现状）；回归面：ac-core 15+11+6、scalar_projection 6/6、r1 86/86、主17/17、core22/22、兼容面一致、r23a close-controls 6/6、fmt过；resolve_with重构为resolve_detailed共享体由全套绿证 | 边界：receiver/method面与session热路径换接归T-02；A07产物/恢复归T-03/T-07；派发实装归T-05/06；F-09预算轴不变 | OPEN不变：F-01(T-02/03待session换接)、F-02(T-05/06待实装)、F-09/10其余/11其余/12/14 | next_responsible_role:review(冻结3e60af2c8受影响面复审T-01决策与竖切)；通过后T-02 session换接

本节保留当前合同交接与最近代码收据；r2所有历轮work/review和new收据见[可读历史](attachments/828/contract-r3/history-through-r23a.md)，旧任务详情与原文保存在[冻结全文](attachments/828/contract-r3/plan-r2-before.md)。历史记录不作为竞争的当前任务指示；旧代码PASS只覆盖其明确提交和范围。

2026-10-21 stage:new(r3执行合同重整) | PLAN-828 r2→r3 | outcome:pass（合同可交接work，非实现/代码审核通过） | authority:用户明确要求补F05b/F01/F02所有权/数据流、代表性矩阵、生产检查和关闭条件，验收目标稳定 | baseline:plan-828-dev@47894c2e6747c8c06032be7a3d40bf00b90a204d，D:/autostack/.wt/lang-828/auto-lang，clean；auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | changed:§0/3/4/5.6–5.8/6/7映射/8/10，新增C01-01..05/C02-01..05/C05-01..04明确原AC/V判据；T01决策D01..03，不另计任务 | preserved:目标/profile/P95/预算/仓库/优先顺序、AC01..10/V01..09/SD01..06/T01..11及executing0/11；F05b r23a已闭及来源既有闭合保留 | evidence:contract-r3/inputs.json与plan-r2-before.md，文档结构/ID/相对链接/边界preflight；未改实现/Spec/ledger、未构建/整档/合入 | evidence_reuse:r2仅可作未变子项历史；F01/F02换代码后须用新冻结提交重验，r3文档不自动给未来实现授PASS | next:work/T01先冻结接口/真实竖切/观测负控与case选择表，再按§8推进；同一不变量连续两失败先诊断调整，不逐例补丁循环。

最近代码受影响面收据（原样保留，plan_revision仍为其原r2；不是r3整体PASS）：

2026-10-21 stage:review(r23a-check/F05b-C01) | review_scope:final角色受影响面复审，非整份最终验收 | reviewer:Codex /root当前审查上下文，未实施47894c2e6、未创建agent | PLAN-828 r2 | affected_outcome:PASS | overall_readiness:needs_fix（其他既有work-owned OPEN，无本轮新finding） | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:47894c2e6747c8c06032be7a3d40bf00b90a204d | base_commit:f1cd60778f5438beecc3e35bb4b2afb56a9030fc | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费；入口/冻结clean | closure:F05b-C01/P2 CLOSED，旧close6控制原样6/6，两Weak失败转绿，close即释放无外部owner的根/HIR/code；F05b已测取消/终态/关闭拥有引用范围CLOSED | evidence:永久86/86（新增f05b释放例实际执行，r22a24格保持）、原62会话62/62及186/528/12/4/9/72矩阵、主17/17、core22/22、fmt0/diff-check0；新增独立3/3覆盖非零revision空终态根getter/普通分步执行/幂等、外部history-hit候选closed拒绝后Weak死亡、最后PausedRun正常完成及实际取消后根/code Weak死亡；取消探针初版未到1024轮询点的测试假设已纠正并留存，不计实现finding | acceptance:AC04/06与V05/06受影响子集PASS，T06/T07及整份AC09/AC10仍pending | spec_delta:SD01..06及相关Spec锁配置哈希未变，无语义缩面，SD04本轮生命周期子集通过；空根/getter现状留F14正文沉淀，无整份delta pass、canonical/ledger未改 | state:executing0/11；其余OPEN F01/02/09/10其余/11其余/12/14及既有来源闭合不变，四既有probe warning归F14 | pending_closeout:最终代码/验收/规范pass后merge-owned冻结裸t+tv及必要consumer/触面门、Spec/ledger/归档/guard；未整档/改实施/合入 | evidence:[完整报告/原样重放/所有权组合/冻结输入](attachments/828/review-r23a/review.md) | next:同树work继续F01/F02等已登记必需项，F05b-C01本轮不再阻塞；全部work-owned完成后再进入整份最终验收。

## 10. 待澄清事项

目标、支持面、P95、预算、单仓范围与执行顺序均已授权，无待用户决定的缩面/增额事项。D798已满足。3e60af2c8已交T-01接缝和interface-decision，但D-01的工作边界/观测不满足原合同，D-02仅量化旧现状、未证明新单元消费首例；D-03需补retained环境的事务与关闭责任。已有owner=T-01和可判定oracle，应先按§8当前入口修正/做最小首例；设计失败则给实际阻碍/备选回new，不以“研究中”标记下游完成。

本轮是合同与实施前置的核对，已在§8给出同树work/T-01的具体指示，不代为启动实现/发送GLM消息或合入。F-05b保持r23a已验证闭合；F-01/F-02仍OPEN，完整T-02/T-05/06换接应先满足T-01前置证据。若未来确需改变共同存储身份、公共ABI/原支持面或资源上限，应明确提出具体语义修订，不能由实现反推修改验收。
