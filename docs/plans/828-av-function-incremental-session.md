---
plan_id: PLAN-828
status: executing
feature_name: 源码到AV安装的全链路函数级增量
plan_revision: 2
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-20
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

拟新增 `probe/src/{session,function_queries}.rs`、`core/src/{function_code,program_snapshot}.rs`（如825已提供等价表示则扩展它）、测试 `function_incremental_828`、`scripts/verify-function-incremental-828.ps1` 和正式recipe薄接线。这些是**待T-01确认、T-02..08建立的新入口**，当前不存在/未验证。

唯一拟实施树 `D:/autostack/.wt/lang-828/auto-lang`、分支 `plan-828-dev`；本轮未创建。实现、内部检查和最终独立review按顺序使用同树，主检出只记计划。无其他仓库写入、工具安装、代理或默认产品入口变更授权。

## 4. 需求分析与背景调查

授权是本聊天最新明确目标和“检查798及架构、明确缺口/是否改计划”；本轮交付具体可审阅草案，范围不含直接实施。未指定实现者/模型/日历或token预算。早先“798现有流程先达标”的决策仍为基础阶段历史；它不满足此次明确的全链路不变对象要求。

来源：[Spec overview](../specs/overview.md)、[HIR stage contract](../specs/auto-hir/stage-contract.md)、[syntax项目](../specs/auto-syntax/project.md)、[796 query/cache](../specs/auto-semantics/query-cache.md)、Design40 §3.2–3.6、Design43 §6、798/825/827当前合同。auto-av/auto-mir目标Spec尚未正式交付，下面add/modify是proposed，D798后按真实Spec核定。源码/规划SHA、主树preflight和串行中央取号见[调查附件](attachments/798/function-incremental-audit-20261020/)；观察树`dbae2e634`clean，非已合入实现。

798当前Driver只持store/project/owner；每次整模块前端，缺一个unit也整模块lower/verify/split，114 hit仍重新decode/verify，最后完整closure编解码/验证。874.5993ms warm body-edit P95包含这些工作，不能当单函数成本；51.1369ms签名冲突是前端提前失败。现有CST splice有声明级复用，尚未接入此scalar编译路径；CheckedModule/LoadedModule是整模块凭证，不能只增加cache map就实现安全函数级增量。

**D798**=基础合同最终独立review、冻结门禁、实际合入、Spec和merge收据齐全；T-01可先只读准备，实际构建/首例/实现等D798。783/787/796消费真实已交付版本，不反向要求798等待828。**用户已明确执行顺序：798完成后优先828，随后817/818及820–827扩展**。本计划不等待本批后继；后继把D828作为新增共享核心实施/集成排期门，原其他依赖仍需满足。已有只读准备/树保留，无并发改同一核心；满足依赖后才持写窗口，不持窗口等上游。详见[顺序与增量兼容合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)。

r2授权补记：用户已确认本计划优先顺序及后继保留增量机制的范围，不再重复请求同一范围确认。本轮先收口798，828仍drafting未开始代码；D798满足后按work进入executing。T-09必须交付可供后继只读消费的函数查询/变更集/受检代码/snapshot接口与版本合同，避免817/818另走整模块前端或后继私建平行loader。

## 5. 详细设计

### 5.1 适用编辑与分轴失效

快路径条件：单个已有函数body内有效编辑，源profile/grammar、函数逻辑身份、签名/可见接口、实际读取的外部语义事实、target/配置/授权均未变。支持字面量、局部变量/槽数、控制流和代码长度变化；新body可改变调用结果，但调用绑定集合和其依赖事实不变。source目录/行号/根修订可更新，未改函数语义对象和已加载代码不变。

调用者只读取callee接口时，callee body改动不导致调用者重编；真正读取body/const/CTE/布局/effect/inline假设的消费者按查询依赖失效。首版不新增这些语言功能，但key/invalidator保留正确区分，不能把未跟踪依赖当“未变”。新增/删除声明、签名/配置/授权变化、跨函数编辑及词法不收敛走明确扩大失效/拒绝重载/全量重建路径，记录原因与真实范围，不计为纯body快路径PASS。

### 5.2 前端与Checked函数单元

保留787增量词法/CST能力，按当前grammar/profile核对全部已支持scalar构造。以顶层声明索引定位函数，复用未改声明的语义投影；局部词法收敛和目录offset元数据可更新，不能重新投影所有函数。body级resolver/typechecker复用签名/声明环境；消除热路径整Program Atom render/bind。已有规则和完整输入入口保留，函数级入口仍执行全部适用类型、初始化、调用/来源和资源规则。

787现有core严格投影不是798 scalar已完整接通的证据。T-01逐项核bool/短路/条件for/命名实参等当前scalar支持，缺项在既有配置驱动语法/投影体系内补齐，必要时新增显式版本化scalar grammar/profile，不悄改旧core grammar或另写平行parser；T-02用旧/新原始源码对照验证全部现有支持。

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

T-01在D门后先完成有界全路径首例，冻结 `attachments/828/contract-decision.md`：scalar语法差异/公共API/信任凭证、函数本地ref/常量、安装可见性、资源归因和正式会话入口。首例为真实源码两个函数 `main→helper`，预热后改helper body，新结果不同、main对象/代码同一、未改阶段计数0、候选错误保留旧版。失败报告具体阻碍并回new作有owner的有界拆解，不能降级到全模块重算后勾验收。

冻结798现有单文件273575B/115函数负载，编辑fn_037的30个不同修订作为纵向基准；另覆盖函数位于前/中/后、局部槽数及源码长度/行数变化、稳定调用/递归。新结果与独立预期及冷编参考一致。签名冲突另测准确诊断，**合法body成功更新**是主性能门，不拿早退sig耗时代表安装。

正式入口一次启动并初始化工程，首个正确结果/ready后开始测量；每次从实际编辑入队到父侧校验当前结果/诊断并发布，包含必要IPC、排队、依赖核对、编译/验证/绑定/安装、执行和比较。不得在计时外预编变更函数或安装候选。首启动/初次装载和重启恢复单列，恒定worker身份校验可在ready前完成，变化的actual准入仍在每次发布门内。写测试报告另报，不替代结果可见时刻。

每性能场景≥30样本，保留全部raw/失败、P50/P95/P99、阶段wall/计数/IO、输入与二进制身份；P95<1000ms且全部正确。结构门要求纯body快路径 `unaffected_{project,resolve,typecheck,hir_build,mir_lower,abc_emit,decode,verify,load}=0`、`whole_module_{frontend,lower,closure_roundtrip}=0`（名字可等价，绑定真实函数入口）；变更函数各阶段真实执行，安装集合恰为一个FunctionId。未变HIR/MIR/代码对象identity、generation及内容hash前后相等；仅hash相同或测试自报counter不足。目录/词法边界检查/根引用更新单列，不宣称整次工作O(1)。

继承D798实际cap及r5预算，不追加一份会话预算：输入/表/验证/执行限额和受管32MiB（hot8/staging16/execution8），A前端峰值≤224MiB/B非前端残差≤128MiB的既定归因方法。驻留查询图、HIR/MIR/代码、候选/旧snapshot、索引和分配器暂存都计入真实峰值，不能移出基线隐去。T-01核Spec/实际cap交集，常驻表示若无法满足先给同口径证据回new，不暗增上限或用LRU全量重编冒充快路径。512真实修订+GC/旧调用释放验证回收/稳定水位，用户cache哨兵不动。

### 规范增量

下表target相对 `docs/specs/`；D798后T-01核实际add/modify。canonical/ledger仅merge沉淀，当前不写已实现状态。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-ac/function-incremental-source.md | 整Program前端→受检接口环境+函数体查询、来源和失效轴；函数HIR实际A07产物/恢复及统一引用 | 完整前端增量 | AC-01/02/05 |
| SD-02 | modify | auto-hir/stage-contract.md、auto-syntax/snapshot-cst.md | 完整模块入口保留→私有函数凭证和真实CST消费；支持不减 | 不复制前端/削弱验证 | AC-02/05 |
| SD-03 | add | auto-mir/function-incremental.md | whole lower/split→函数lower/verify和稳定跨单元引用 | 中段真实增量 | AC-03/05 |
| SD-04 | add | auto-av/resident-function-install.md | 完整LoadedModule→函数代码/受检环境/事务snapshot及寿命 | 装载执行增量 | AC-04/05/06 |
| SD-05 | modify | auto-av/cache-and-conformance.md | 磁盘命中→内存复用/恢复与全链路结构/P95/资源证据 | 不能只看emit/cache计数 | AC-01/06/07/08 |
| SD-06 | modify | auto-test-case/architecture.md | one-shot保留→正式持续会话ready/edit/result准入及失败门 | 真实端到端 | AC-07/08 |

## 6. 测试设计

以下入口均拟建，T-01冻结真实名、T-02..08使其可执行。拟定向命令 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml function_incremental_828 -- --test-threads=1`；脚本Scopes=`first|frontend|compile|install|lifetime|formal|acceptance`，当前未运行。

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

实施者未指定，内部定向检查不等于最终验收。最终review独立于实施上下文，同一冻结树逐AC/V/SD审查，明确merge-owned整档pending；本轮不创建代理/其他聊天。修复先完成finding和必要定向测试，审查通过后merge再运行裸t+触面档；tf仅主检出到期单实例。

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 稳定会话/真实变更和依赖集；V-01/03/07，未改语义对象/代码identity不变，无全量重算伪命中 |
| AC-02 | work | 函数级语法投影/resolve/typecheck/HIR验证实际接线，V-01/02/03，无整模块热前端 |
| AC-03 | work | 单函数MIR/ABC生成/检查，V-01/03/04，未改lower/emit=0，无whole lower/split |
| AC-04 | work | 局部受检装载/原子安装、新旧snapshot可见性；V-01/04/05/06，无未改代码重装载 |
| AC-05 | work | 凭证/依赖/授权/owner/版本/来源门，V-02/03/04；完整输入原支持面不减 |
| AC-06 | work | 旧代保活、失败/关闭/退出真实回收，V-05/06/07/09；继承预算，512修订无无界增长 |
| AC-07 | work | V-08/09正式结构门及P95<1s同时过，≥30新编辑、raw/反控完整，不含初次初始化或移出当次必要工作 |
| AC-08 | work | V-07/09必需子case、旧支持面/实际消费者兼容、SD/版本映射完整，无required skip/缺oracle |
| AC-09 | review | 冻结提交/依赖/配置/输入绑定的独立最终逐AC/V/SD验收，merge门禁pending显式 |
| AC-10 | merge | 冻结根裸t+编译器/VM tv及实际触面档、独立workspace/正式兼容通过；Spec/ledger/归档/guard收尾完成 |

## 8. 执行步骤

| ID / 状态 | owner_stage | 依赖 | 文件/符号与交付 | AC / 验证 |
|---|---|---|---|---|
| [ ] T-01 | work | D798；本轮用户已确认优先范围，只读可先行 | 既有core/probe/source/syntax双函数首例；冻结 attachments/828/contract-decision.md、版本/资源/接口/正式entry；失败回new | AC-01..07；V-01/04最小正反控、main身份不变，非模拟。[R1已重开；下述为历史实施证据] `plan-828-dev@`"feat(ac-av-probe): full-path function-incremental first case and session"：session.rs（synth源驱动真实compile_parts_with、verify_function凭证、lower_function_unit、ABC准入、snapshot link）、core function_code/program_snapshot/execute_program、ac-core verify_function+CheckedFunction、测试 function_incremental_828（V-01: 23→28、main三重Arc identity、11轴计数全0、install={helper}、新旧snapshot双运行；V-04: 类型错误候选拒绝、旧版可运行、恢复编辑成功）全绿；798回归 core21/mir8/abc15/binding4/vertical3/cache20 全绿；语法审计/收敛边界(NoConvergence需≥8尾部token)/接口合同冻结于 attachments/828/contract-decision.md |
| [ ] T-02 | work | T-01 | ac-syntax edit/投影接ac-core source，函数索引和相对来源；保留完整入口 | AC-02/05；Scope frontend，V-01/02/03语法部分。[R1已重开；下述为历史实施证据] `plan-828-dev@`"feat(ac-syntax): auto-scalar grammar, per-decl projection wiring"：新增版本化 grammar auto-scalar.at（全 scalar 表面：6级二元优先级、右结合一元、带符号字面量折叠、条件for→While、while保留字）；投影扩展（CoreKinds 可缺项、一元/调用算子链、Unary span=操作数span 与 parser 逐字节一致）+ `project_decl_strict` 单声明投影；ac-core 新接缝 `compile_program_parts`（整模块入口原样保留）；会话热路径改为 增量嫁接→单声明投影→synth签名桩Program→compile_program_parts（无整模块 parse/投影/垫片串）。对拍：scalar_projection_ 5组差分 AST+span 全等、冻结负载256KiB×2修订全等、adapter 44 绿、会话4测试绿（V-01/V-04/冻结负载对拍/语法错误反控）、vertical/binding/cache 绿。合同 §2 更新于 attachments/828/contract-decision.md |
| [ ] T-03 | work | T-02 | ac-core resolver/typecheck/HIR verifier、probe function_queries：接口/body查询、私有函数凭证、失效；函数HIR A07 codec/producer/恢复 | AC-01/02/05；V-01/02/03/04/07，错上下文不可复用、恢复不带trusted |
| [ ] T-04 | work | T-03。T-04 内存复用。[R1已重开；下述为历史实施证据] 会话内内容寻址复用（796 ContentVersion(canonical HIR dump) → 已验证(body,code)对；A→B→A 第三次编辑 Arc 身份复用 warm-up 对象，reuse_hit 计数、warm-up 内容预置）；miss 路径抽出 compile_and_admit_changed；准入反控：可执行闭包不可入 FunctionCode、link 签名不匹配必拒。测试 8/8 绿 | hir_adapter/cache、mir_verify：lower_function/单元ABC、内存复用 | AC-01/03/05；Scope compile，V-01/03/04，whole lower/未改emit=0 |
| [ ] T-05 | work | T-04。T-05 函数表示/凭证/稳定绑定。[R1已重开；下述为历史实施证据] FunctionCode 增 mint_revision 代际（未改函数对象与代际跨修订不变；内容命中复用原对象原代际）；ProgramSnapshot::find_code/generation_of 稳定 def-id 查询（身份轴=def id，非位置序号）。t05 代际测试绿 | abc_verify/av及等价function_code/loaded_program：函数表示/验证凭证/稳定调用绑定 | AC-04/05；V-01/03/04，未改identity/代际不变，unit不可执行 |
| [ ] T-06 | work | T-05。T-06 prepare/commit/可见性。[R1已重开；下述为历史实施证据] 执行器重构为 LoopState 显式状态（fresh 播种/非空续跑），新增 execute_program_steps/resume_program/PausedRun 步数屏障（确定性，无 sleep）；session 真事务拆分 prepare_edit/commit（陈旧候选按 base revision 拒绝，旧候选不覆盖新结果；edit 为融合路径）。V-05 确定性屏障（链中途暂停、安装新修订、恢复链完成=旧语义、新顶层调用=新语义）与 V-06 乱序提交/fuel 反控绿；core 21 测试在重写循环上不变绿 | session/worker、program_snapshot/AV：prepare/commit、结果revision、顶层调用选择 | AC-01/04/06；Scope install，V-01/05/06及确定性屏障 |
| [ ] T-07 | work | T-06。T-07 生命周期。[R1已重开；下述为历史实施证据] warmup_persisted（A09 单元+会话manifest落盘，与798 Driver store 互通）+ restore（逐单元 decode+verify_abc+载荷名核对，fail-closed 回冷启动；restore_units_admitted/restore_materialize 计数）；commit 持久化+798 同款冲突容忍；HISTORY_CAP=32 有界历史；line_col 来源映射。t07 恢复/miss回退/512修订soak（每64修订独立期望、history≤2×CAP、rev0 快照全程保活）/trap span 位移 绿 | runtime引用、store lease/退役/清理和来源映射 | AC-05/06；Scope lifetime，V-02/05/06/07及512修订GC |
| [ ] T-08 | work | T-07。T-08 正式档双门。[R1已重开；下述为历史实施证据] formal_828.rs：冻结负载 273575B×115函数、30 个 fn_037 修订（源码差分推导编辑、独立期望验证），编辑入队→当前结果发布计时 + 逐编辑结构门delta；validate_and_gate 双门各轴独立可红（零阈值构造性失败/缺样本/伪全量/慢而细粒度）。修复两真实缺陷：编辑函数的跨函数调用在内部lower以 fn: 单元import落地；语法证据计数改累计。17/17 绿；scripts/verify-function-incremental-828.ps1 七 Scope 落地 | 正式持续会话薄注册/worker/父侧judge、observations、脚本/fixtures和ready分界 | AC-07/08；Scope formal，V-08/09；强制全量/坏actual/缺样本/阈值0必红 |
| [ ] T-09 | work | T-08。T-09 覆盖/兼容/后继合同。[R1已重开；下述为历史实施证据] attachments/828/coverage-manifest.md（V-01..09→真实入口+oracle+三项诚实开放项）+ successor-interface-contract.md（817/818/820-827 只读接缝与身份律/版本合同）；兼容证据：828 17/17 + adapter 全套 + 798 面 mir8/abc15/binding4/cache20/vertical3/native11+1/case5/core21 全绿 | case→子case→入口/phase/code/oracle manifest、旧支持面/消费者兼容；给817/818及820–827的只读函数查询/变更集/受检代码/snapshot接口与版本合同；SD/证据提交冻结 | AC-01..08；Scope acceptance，无必需skip；公共接口反控/受检消费可复用，仅定向测试 |
| [ ] T-10 | review | T-09 | 同一clean冻结树独立AC/V/SD审查，遗漏/绕过回work，整档pending明示 | AC-09；auto-plan-review，不因送审跑整档 |
| [ ] T-11 | merge | T-10通过 | 冻结门禁、consumer并集、Spec/ledger/归档/合入/guard；tf到期主检出单实例 | AC-10；auto-plan-merge，实际完成才11/11 |

## 9. 复审记录

2026-10-21 stage:work(repair r3a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): close r2a-check adjacent failures - snapshot-owned span shifts, per-entry mint origins, body-read intrinsic binding" | r2a-check四例相邻反例全部闭合并收编回归(r1回归套件13/13=6×r1+3×r2+4×r2a-check)：
F04-a→公开快照执行路径映射→ProgramSnapshot自持per-code span shifts，execute_program/steps/resume经快照不可变视图映射→r2a_current_snapshot_public_execution PASS→闭合
F04-b→命中条目携带自身铸造origin→history值(body,code,mint_origin)；命中恢复该基面而非继承上次铸造→r2a_history_hit_preserves_mint_source PASS→闭合
F04-c→命中函数体级source map→单编辑区域diff(threshold,delta)叠加于origin位移之上，代码payload不动→r2a_semantic_noop_edit PASS→闭合
F06/R2a→凭证绑定实际读取的intrinsic事实→CheckedFunction记录body实际引用的(symbol,内容规范化签名)；FnEnv自持intrinsic声明表；credential_current要求全部已读绑定仍在且形状一致（未读intrinsic仍排除于全局身份）→r2a_credential_invalidates_when_referenced_intrinsic_changes PASS（含同绑定正控）→闭合
回归面：主17/13、scalar差分6、vertical/cache绿；r2a-check核验材料已随分支保存(attachments/828/r2a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门；F-14文档重整+SD文本+dead_code告警 | next_responsible_role:work(继续，同一树)

2026-10-20 stage:work(repair r2a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): close review-r2 replay failures - span mapping, domain axis, unchanged-callee intrinsics" | R2三例失败反例全部闭合并收编回归(function_incremental_828_r1 9/9=6×r1+3×r2)：
F-04/R2→未改函数诊断位置随当前源映射→FnRecord origin/mint_origin+run()陷阱span按所属函数origin位移映射（代码payload不动；固定旧snapshot运行保持其自身源坐标）→r2_unchanged_trap_location PASS→闭合
F-06/R2→凭证绑定HIR域→FnEnv自持bundle域+credential_current比domain（同域正控/跨域拒绝）→r2_credential_cannot_cross_hir_domains PASS→闭合（完整语义上下文=域+owner+指纹三轴现均由环境自证）
F-08/R2→未改callee的intrinsic使用不阻塞caller合法编辑→声明上下文身份排除intrinsic def（其派生自body内容；增量synth只带changed真body，未改函数intrinsic随其保育代码单元）→r2_trace_in_unchanged_function PASS（含无授权负控）→闭合（与F-01架构修复的正交性：stub面问题另立）
OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余（驻留/候选32MiB预算+typed资源传播+失败释放）；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门；F-14剩余（contract/coverage/successor重整+SD具体文本+session dead_code新告警：context_revision字段已由FnEnv自证后不再读取）| 回归面：828主17+9、scalar差分6、vertical/binding/cache绿 | next_responsible_role:work(继续，同一树)
2026-10-20 stage:review(R2) | review_scope:final | reviewer:Codex /root（与R1相同独立审查上下文，未实施修复） | PLAN-828 r2 | outcome:needs_fix | review_mode:就绪核查+原例/相邻控制，未启动完整最终验收（work明确in_progress及OPEN必需项） | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:791c62ab251b5d0f36d3d5610fade600760d2f52 | base_commit:940445bf57674780bc6ff6c68928c0c7148a0dbf（原合同基面f79b1b83591310b45b3828dd300e6d368987d6c0） | dependency_revisions:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费；配置/锁/测试二进制/Spec/skill输入见[baseline](attachments/828/review-r2/baseline.json.txt) | closure:F-R1-03/07/13 CLOSED；05/06/08/09/14 PARTIAL；01/02/04/10/11/12 OPEN | evidence:永久修复6/6、原suite17/17通过；独立原7例6过1失败，新增4控制2过2失败，总11例8过3失败（未改trap位置、Scalar凭证错入Core、trace在未改callee时caller编辑拒绝）；互递归+旧snapshot控制过；frontend脚本9项正控和真实零匹配/Cargo失败拒绝过；五新模块fmt/diff-check过，修复新引入session.context_revision dead_code待清 | acceptance_results:AC01/02/04/05/07/08 fail，AC03/06 partial，AC09 needs_fix，AC10 pending | spec_delta:SD01..06仍未完整通过，冻结表sha256=d2cf61b3dd120e7e08edd9997877618df6e31fec0199a2effc9762cbfc42f383；未改canonical/ledger | findings/diagnosis/AC-V-SD:[R2报告及原始反例](attachments/828/review-r2/review.md) | state:executing0/11保持，已闭子项证据保留 | pending_closeout_gates:最终独立pass后的merge-owned裸t+tv及必要触面/独立workspace/consumer门、Spec/ledger/归档/guard；未跑整档/未改实施/未合入 | next:原树work完成所有必需项并提交诊断/覆盖调整；第2次送审仍有未完成实现，禁止把原例通过或future/batch登记冒充就绪。


2026-10-20 stage:work(repair r1a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | 冻结送审提交:940445bf5 | repair_commit:"fix(probe): close review-r1 findings F-03/05/06/07/08/09/13/14" | 复审反例7例中6例已闭合并收编为永久回归(probe/tests/function_incremental_828_r1.rs 6/6)+原17例保持绿；验证脚本7 Scope全PASSED | closure-rows:
F-03→单元自引用必须走import约定→hir_adapter def_map/内部调用臂→自引用=fn:<self> import+r1_recursion回归→PASS→闭合
F-05→候选绑定铸造会话身份→PreparedEdit/commit→进程唯一session_id令牌+跨会话拒绝回归→PASS→闭合（取消/关闭态仍开放归F-05b）
F-06→环境自证上下文身份→function_queries/CheckedFunction→FnEnv自持declaration_context_fingerprint+owner；credential_current无外部参数→PASS→闭合
F-07→恢复对象贯通驻留记录→restore/commit→restore同步rec.code+history→恢复后编辑保持restored身份→PASS→闭合
F-08→保留intrinsic支持面→warmup def zip→intrinsic def跳过（走单元import约定）→trace正控→PASS→闭合
F-09→继承输入前门→warmup/prepare→front_door_admit(尺寸+嵌套)于冷启与每次编辑源→4MiB+超限拒绝→PASS→闭合
F-13→脚本真实可运行+零匹配防伪→verify-function-incremental-828.ps1→参数数组/ac-syntax manifest/零选择守卫→7 Scope PASSED→闭合
F-14→簿记一致→本记录+rustfmt已做；contract-decision/coverage/successor文档重整随架构修复后统一做→部分闭合
OPEN(继续修复，不得送审):F-01 真·函数级前端(保留受检接口环境+changed-body resolve/typecheck/HIR producer，计数绑定真实生产入口+强制全量反控)；F-02 安装不重建全部执行payload(执行器经受检链接视图派发不可变单元，身份断言覆盖实际派发MIR)；F-04 相对span+快照映射(未改函数诊断位置，replay反例在案)；F-05b prepare/commit取消与关闭态；F-10 A07函数HIR codec/producer/恢复重验+实际依赖查询负控+真实store GC/lease/Weak水位；F-11 持久会话只发布changed单元+manifest驱动恢复；F-12 正式worker/IPC/父侧judge+双线内存门+真实反控 | next_responsible_role:work(继续，同一树)
2026-10-20 stage:review | review_scope:final | reviewer:Codex /root（独立于实施上下文；无额外模型/会话ID声明） | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:940445bf57674780bc6ff6c68928c0c7148a0dbf | base_commit:f79b1b83591310b45b3828dd300e6d368987d6c0 | dependency_revisions:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（未消费）；锁/配置/Spec/skill/输入哈希见[baseline](attachments/828/review-r1/baseline.json.txt) | acceptance_results:AC01/02/04/05/07/08 fail，AC03/06 partial/fail，AC09本次不通过，AC10 pending | findings:F-828-R1-01..14 OPEN，详见[R1报告/AC-V-SD映射](attachments/828/review-r1/review.md) | evidence:既有828定向17/17、scalar差分6/6通过；独立7例全部复现合同失败（递归热编辑、未改trap位置、跨会话提交、凭证错环境、恢复代码身份、trace支持面、4MiB cap）；公开frontend脚本exit1，新模块fmt exit1，diff-check通过。已承认work-owned正式worker/IPC与双线内存未完成，故仅做就绪核查及有界反例，不冒充完整最终验收通过 | spec_delta:SD01..06均有实现/证据阻断，冻结表sha256=d2cf61b3dd120e7e08edd9997877618df6e31fec0199a2effc9762cbfc42f383 | state:executing0/11；T01/02/04..09重开，T03仍未完成，T10/11未完成，旧实现/测试记录保留但不授完成 | pending_closeout_gates:独立pass后merge-owned裸t+tv及实际触面/独立workspace/正式consumer门、Spec/ledger/归档/guard；未运行整档/未修实施/未合入 | next:原树work一次修齐全部finding与必需oracle、提交新冻结候选后受影响面复审；不批准以debt/future/batch延期必需项。

以下较早work/new/review记录为历史；当前状态以最新复审为准。任务表中的历史实施证据保留，不代表当前验收完成。

2026-10-20 stage:work | PLAN-828 r2 | outcome:pass | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | base_commit:f79b1b835(master) | code_commit:9 commits(T-01 首例→T-09 合同:feat(ac-av-probe) full-path first case / feat(ac-syntax) auto-scalar grammar+projection / feat(probe) query layer / reuse+admission negatives / feat(av-core) generation axis / resumable executor+prepare-commit / store restore+bounded history / formal dual gates / docs manifests) | dependency_revisions: auto-down sibling detached@c39ac7f(未消费) | task_ids:T-01..T-09 全[x] | evidence: function_incremental_828 17/17(--test-threads=1)、scalar_projection_ 5/5+adapter 全套、冻结负载对拍全等、正式档双门 PASS(P95<1000ms+结构零门)、798 兼容面 mir8/abc15/binding4/cache20/vertical3/native_admission11/native_trace1/case_binding5/core21 全绿 | blockers:无 | open_items:coverage-manifest 三项(错误恢复span全等/body事实消费者随语言扩展/进程级双线内存+AutoDown worker包装) 已登记不冒充 | merge_gates_pending:裸t+tv于冻结提交(merge-owned) | next_responsible_role:review(T-10)

2026-10-20 stage:new | PLAN-828 r1→r2 | outcome:pass（排期/消费者合同，非实现） | authority: 用户明确798完成后优先828、其他ACE扩展保留全链路增量 | delta: §§4/8明确已授权顺序与T-09后继公共消费接缝；各后继新增D828排期门/独立work验收，不反向阻塞本计划 | drafting0/11 | next: 先完成并合入798；本轮不启动828代码，不再要求同范围重复确认。

2026-10-20 stage:new | PLAN-828 r1 | outcome:pass（文档合同preflight可交付审阅，非实施/最终验收） | authority: 用户要求全链路函数增量，本轮检查/设计/计划修订 | 0/11 drafting | source: 798 `dbae2e634` clean；调查/hash/中央取号见798附件 | changed: AC-01..10 / V-01..09 / SD-01..06 / T-01..11 | next: 草案确认后work，实际首例/构建等D798，T-01可先只读准备；未执行代码/测试/merge/Spec发布。

## 10. 待澄清事项

用户目标、P95及798后优先828的顺序均已确认，无需重问；当前实际前置为D798交付。本轮先收口798。可信函数接口、787 scalar语法差异、安装物理表示/资源可行性由有界T-01先冻结；不以开放问题豁免验收或减少支持面。不承诺合法签名变更原位重载、产品UI/对象迁移或响应式运行值缓存。
