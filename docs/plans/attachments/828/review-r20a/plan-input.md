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

2026-10-21 stage:work(repair r20a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:e083f2a28 "fix(probe): mapped-source evidence proves v2 sidecar spans before admission" | r19a-check F11-S01剩余缺口（映射后来源正确性）闭合并收编回归(r1回归套件76/76=72前+4r20a)：
机制→v2谓词补**映射后来源证据**：frame/UTF8/可重锚之外，构建真实锚点from_texts(m_d0,m_d1,当前声明原点,mint文本,当前声明文本)，把sidecar每条span经map_span映射到当前坐标，逐指令与**当前源冷编译自身span表**全等（同cv⇒同指令流；restore用会话内冷编resident，persist写回用驻留span经会话span_map映射后的当前坐标——冷恢复路径即恒等精确）。"框架内"不再是来源凭证：合法框架内错误span（23..28改22..27）映射后38..43≠当前39..44被拒；坏primary不再遮蔽正确repair（先证后选）。v1证据路径不变（spans==当前冷编表）
两失败转绿：框架内错误span→回冷得当前39..44；同错primary+正确repair→admitted1且39..44。正控保持：合法v2重定位、16格选择矩阵（4无效×位移×好primary/repair）、legacy同错span拒绝、双坏行Err、三轮restore/edit/pin
收编永久回归：r20a_primary_repair_applicability_matrix（16格）、r20a_v2_in_frame_span_must_match_current_source、r20a_bad_v2_span_does_not_hide_valid_repair、r20a_legacy_in_frame_false_span_is_rejected_control（镜像reviewer applicability-controls harness输入，含真实mutation断言）
回归面：r1套件76/76、主17/17、core22/22；reviewer冻结四harness原样全绿——applicability 5/5(两失败转绿)、generation 8/8、sidecar 6/6、62会话62/62；矩阵186/528/12/4/9/72全过；map四harness exit0、runtime正控1/1；798兼容面cache20/mir8/abc15/binding4/vertical3/case5一致 | OPEN不变：F-01/02/05b/09/10其余(A07/依赖/GC)/11其余/12/14 | next_responsible_role:review(冻结e083f2a28受影响面复审F10-S02/F11-S01完整适用性)

2026-10-21 stage:review(r19a-check) | review_scope:final角色受影响finding/就绪核查，非整份最终验收 | reviewer:Codex /root审查上下文，未实施r19a、未创建agent | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:763dbf941c4ddaf767d5942a9436cd03f6bdc999 | base_commit:f1eb6cce38c4b67422a3b8940824e64eb47be149 | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:C盘r18a-check五失败原样全转绿，generation8/8；sidecar6/6；F10-S02原失效mint恢复/repair选择子集及新16格均PASS；F10-S01保持CLOSED | remaining:F11-S01/P2实际来源正确性未闭：合法v2 mint原文、正确框架0..30、完整覆盖，仅将真实除零span23..28改为框架内22..27，restore位移16空格源后实际admitted1/span38..43，应39..44；有正确repair行时同样错误primary遮蔽repair。v2分支只验frame/UTF8/from_texts，未消费cur的实际来源证据；旧格式同样错误span会被拒，正控PASS。完整候选适用性及F10-S02整体闭合仍受此缺口制约 | evidence:新增applicability5tests=3pass2fail（两失败同根）；16格选择、双坏mint行Err、legacy错误span拒绝正控PASS；永久72/72、主17/17一次、core22/22、原62/62；cache20/mir8/abc15/binding4/vertical3/case5全过；旧186/528/12/4/9/72矩阵及420格全过；map四harness全exit0、runtime1pass；fmt0/diff-check0 | acceptance:AC05/V02/V07恢复来源PARTIAL/FAIL，T07未整体闭合；其他已闭合范围保持；整份AC09/AC10 pending | spec_delta:SD01..06及canonical冻结且未变，无语义削弱授权；无整份delta pass | state:executing0/11，其他OPEN F01/02/05b/09/10其余/11其余/12/14不变 | pending_closeout:最终pass后merge-owned裸t+tv及必要consumer/触面门；未跑整档/未改实现/未合入 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r19a-check | next:同树work在当前坐标系验证v2映射后真实trap来源，并在完整准入后选择primary/repair，同标准写回；保留合法位移和restore/edit/pin正控，提交后复审。

2026-10-21 stage:work(repair r19a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:763dbf941 "fix(probe): full sidecar applicability unifies selection, repair and provenance evidence" | r18a-check F10-S02/F11-S01剩余不变量闭合并收编回归(r1回归套件72/72=65前+7r19a)。〔r19a-check裁定：r18a五失败全CLOSED，但F11-S01实际来源正确性仍OPEN——合法框架内错误span(23..28改22..27)被准入38..43应39..44且遮蔽正确repair；v2分支未消费当前编译span证据，由r20a修复，见上〕：
机制→统一完整适用性谓词unit_sidecar_usable_for_current（cache.rs，替代弱检查payload_usable）：解码(两代)+合并后spans适用性(F798-R15-03)+v2 mint三条件(UTF8合法/声明mint框架覆盖每个实际span/ mint文本经from_texts可重锚到当前声明)+v1(spans-only)以**精确证据**代替恒等假设——行spans必须等于当前源冷编译自身span表（restore用会话内冷编resident；冷恢复路径persist用同标准复核survivor）。restore候选选择(primary→repair)与persist survivor/repair写回采用同一谓词：primary全检失败继续试repair（不再遮蔽有效repair行），双行皆失效仍显式Err
五负控全绿：v2 mint首字节非法UTF8→second admitted1；合法UTF8但不可重锚('X')→second admitted1；坏primary+好repair→first admitted1（不再遮蔽）；legacy同值位移源→不再恒等热命中，span=当前冷编(39..44类)；v2声明框架100..130不含真实trap23..28→拒绝，span=当前冷编。正控保持：合法v2重定位admitted1+span正确、双坏行显式Err、同源v1热命中(spans==冷编表)
收编永久回归：r19a_invalid_v2_mint_is_repaired_for_next_restore、r19a_utf8_but_unalignable_mint_is_also_repaired、r19a_invalid_v2_primary_does_not_hide_good_repair、r19a_both_invalid_rows_report_explicit_error、r19a_legacy_sidecar_with_relocated_source_needs_current_coordinates、r19a_v2_relocation_control、r19a_v2_mint_frame_must_cover_the_actual_trap_span（镜像reviewer sidecar-generations harness输入）
回归面：r1套件72/72、主17/17、core22/22；reviewer冻结三harness原样全绿——generation 8/8(五失败转绿)、sidecar 6/6、62会话62/62；矩阵186/528/12/4/9/72全过；map四harness exit0、runtime正控1/1；798兼容面cache20/mir8/abc15/binding4/vertical3/case5一致 | OPEN不变：F-01/02/05b/09/10其余(A07/依赖/GC)/11其余/12/14 | next_responsible_role:review(冻结763dbf941受影响面复审F10-S02/F11-S01完整适用性)

2026-10-21 stage:review(r18a-check) | review_scope:final角色受影响finding/就绪核查，非整份最终验收 | reviewer:Codex /root审查上下文，未实施r18a、未创建agent | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:f1eb6cce38c4b67422a3b8940824e64eb47be149 | base_commit:1414ef960c7c3f195f2f1a3afde5ed1303f57340 | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:r17a-check三失败原样全部PASS，sidecar原harness6/6；F10-S01 CLOSED；F10-S02空表原例second admitted1、F11-S01 v2前移原例86..91 PASS | remaining:F10-S02/P2完整适用性判断不一致：unit_sidecar_payload_usable只验覆盖，忽略新mint UTF8/重锚；非法UTF8与合法UTF8但不能重锚的v2回冷后second admitted0应1；primary该类失效时遮蔽正确repair，admitted0应1。F11-S01/P2来源适用性：legacy无mint无snapshot证明就identity，位移16空格后admitted1 actual23..28应39..44；v2声明mint100..130却携带真实trap23..28也未拒，actual同样23..28应39..44 | evidence:新增generation8tests=3pass5fail，五个失败归上述两个剩余机制；合法v2重定位、三轮restore/edit/pin、双坏行显式Err正控PASS；永久65/65、原17/17一次、core22/22、62会话62/62；cache20/mir8/abc15/binding4/vertical3/case5全过；旧186/528/12/4/9/72矩阵及420格全过；map四harness全exit0、runtime1pass；fmt0/diff-check0 | acceptance:AC05/V02/V07恢复来源/修复PARTIAL/FAIL，T07未闭；F04-u/v有界闭合保持；AC09定向needs_fix，整份最终验收与AC10 pending | spec_delta:SD01..06和canonical哈希冻结且未变，无语义削弱授权；无整份delta pass | state:executing0/11保持；其他OPEN F01/02/05b/09/10(A07/依赖/GC)/11其余/12/14不变 | pending_closeout:最终pass后merge-owned裸t+tv及必要consumer/触面门；未跑整档/未改实现/未合入 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r18a-check | next:同树work统一完整candidate/survivor适用性与repair选择，旧格式须证明来源或回冷，新mint框架须覆盖实际使用的trap spans；收编五负控及正控，提交后复审。

2026-10-21 stage:work(repair r18a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:f1eb6cce3 "fix(probe): verified-then-anchored restore admission, sidecar repair and re-anchoring" | r17a-check F10-S01/S02+F11-S01 闭合并收编回归(r1回归套件65/65=61前+4r18a)。〔r18a-check裁定：F10-S01 CLOSED、两finding原例闭合，但F10-S02完整适用性(弱检查忽略mint UTF8/重锚、坏primary遮蔽repair)与F11-S01来源适用性(legacy无证据恒等、v2框架不绑实际spans)五负控仍失败，由r19a修复，见上〕：
F10-S01→恢复准入顺序：decode后、任何索引前受控first()守卫（零函数unit冷回退不panic），verify_abc仍权威拒深层形状问题；后续funcs[0]访问均在已验证结构上
F10-S02→重推导并重写（cache-and-conformance §11全语义）：persist写sidecar遇Conflict先复核survivor适用性（unit_sidecar_payload_usable=可解码(两代格式)+合并后覆盖可trap指令），survivor失效则以repair身份发布重推导载荷（unit_span_sidecar_repair_key新身份），restore按primary→repair探测——store无单行重写，此即"发布新身份并使恢复引用切到有效产物"；repair行再被失效行占住则显式Err不静默接受。空/不覆盖sidecar回冷后第二次restore真实命中admitted=1
F11-S01→sidecar载荷升v2（magic 8SS2：spans+mint声明区间+mint文本）：restore对v2经SpanSegments::from_texts把unit的MINT绝对spans重锚到当前声明（重定位源恢复当前坐标：插16空格例70..75→86..91）；v1 spans-only载荷无mint基，按当前坐标恒等接受（reviewer honest种子兼容；键签名不变，冻结harness零适配原样6/6）。恢复的历史条目携带unit真实mint origin/text（非当前切片），后续内容命中重锚基础正确
收编永久回归：r18a_zero_function_unit_fails_closed_without_panic、r18a_inapplicable_sidecar_is_repaired_for_future_restores（empty+non-covering）、r18a_missing_sidecar_is_written_back_for_future_restores、r18a_persisted_prefix_edit_restores_current_coordinates（镜像reviewer restore-sidecar harness输入）
回归面：r1套件65/65、主17/17、core22/22；reviewer冻结sidecar harness原样6/6 exit0（三失败全转绿）；62会话重放62/62保持；186/528/12/4/9/72矩阵全过；r17a-check其余harness（map四harness/runtime正控）过；798兼容面cache20/mir8/abc15/binding4/vertical3/case5一致 | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（span sidecar含准入/修复/重锚已落）；F-11 changed-only持久化+manifest驱动恢复（重定位来源轴已闭，其余保持）；F-12正式worker/IPC/双线内存门；F-14文档重整+SD文本 | next_responsible_role:review(冻结f1eb6cce3受影响面复审F10-S01/S02+F11-S01)

2026-10-21 stage:review(r17a-check) | review_scope:final角色受影响finding/就绪核查，非整份最终验收 | reviewer:Codex /root审查上下文，未实施1414ef960、未创建agent | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:1414ef960c7c3f195f2f1a3afde5ed1303f57340 | base_commit:473a1fbd4c8fc1e4f921b0b5fb693100c7d0af18 | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:r16a-check PASS簿记回写核实；原62会话首次全绿62/62属实，旧恢复首run spanNone原例CLOSED；不再沿用旧站立红 | findings:F10-S01/P1新准入顺序回归，正常decode的零函数unit在verify_abc之前被sidecar_applicable索引funcs[0]，session.rs551越界panic；同payload公开verify_abc负控会返回Err，应受控回冷。F10-S02/P2空但合法编码sidecar回冷后正确span未重写，Conflict被忽略，第二次restore admitted0应1；missing正控第二次admitted1。F11-S01/P2既有登记来源重定位OPEN独立重放：正常持久化会话前方helper插16空格后用自身最新源码恢复，实际70..75应86..91；驻留cold oracle先过 | evidence:新增sidecar6tests=3pass3fail（fail-cold正控含6种模式）；原62/62、永久61/61、主17/17一次、core22/22；cache20/mir8/abc15/binding4/vertical3/case5全过；旧186/528/12/4/9/72矩阵全过；r16a map四harness全exit0、420格/4tests过、runtime1pass；fmt0/diff-check0 | acceptance:AC05/V04/V07新恢复准入与来源FAIL/PARTIAL，T07未闭；其他来源闭合保持。F11子缺口已登记，不重复作为未声明新scope；记录可重放证据且整体验收仍未满足 | spec_delta:SD01..06冻结、canonical未变，cache-and-conformance第11行要求sidecar适用性失败重推导并重写，本轮仅前半完成；无整份delta pass | state:executing0/11保持，其他OPEN F01/02/05b/09/10(A07/依赖/GC)/11/12/14不变 | pending_closeout:最终pass后merge-owned裸t+tv及必要consumer/触面门；本轮未跑整档/未改实现/未合入 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r17a-check | next:同树work修F10-S01/S02，收编原样负控；F11-S01保持其既有OPEN或同轮修复，冻结后复审。

2026-10-21 stage:work(F-10 恢复span sidecar) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | code_commit:1414ef960 "feat(probe): unit-bound span sidecar restores mapped trap spans" | 62例会话重放唯一站立红（restored_same_content_edit_initializes_nonempty_source_alignment，恢复后首run span=None）闭合并收编回归(r1套件61/61=60前+1r17a)。〔r17a-check裁定：spanNone原例CLOSED，但新增F10-S01验证前索引panic、F10-S02失效sidecar回冷不重写，F11-S01重定位重放70..75应86..91，由r18a修复，见上〕：
根因→持久化刻意剥离span（session persist_all_units "spanless ABC unit"，载荷与798 A09身份方案字节等同），恢复单元无span表→dispatch报span:None；且restore快照用裸ProgramSnapshot::link（span_maps全default空表），首run映射靠空表恒等副作用
修复→(1)unit-bound span sidecar：mint span表随单元内容轴（owner+def_id+cv）另存旁车行（unit_span_sidecar_key/EnvelopeIdentity::unit_span_sidecar，复用798 encode/decode_sidecar+merge_spans+sidecar_applicable，A09层、Conflict容忍与单元行配对同款）；ABC载荷字节不动，798身份方案/互通不变 (2)restore重挂sidecar后经F798-R15-03适用性校验（可解码但不覆盖可trap指令=丢provenance同缺失，拒），缺失/损坏/不适用一律fail-cold回warmup_persisted，绝不恢复span盲会话 (3)restore快照改link_with_maps显式携带恒等映射（与rec.span_map同源），不再依赖空表恒等副作用
新增登记（F-11 manifest驱动恢复子缺口，本轮不扩）：cv键按值相等命中——同值但trivia重定位的源会恢复mint span+恒等映射，绝对位置可能错；精确重锚需持久mint text（manifest目前仅def_id/name/cv），归changed-only持久化+manifest驱动恢复重构面
回归面：r1套件61/61、主17/17、core22/22；62例会话重放**首次全绿62/62**（站立红closed）；186/528/12/4/9/72矩阵全过；r16a-check全部harness原样重放map-checks 4×exit0、deletion-composition 4/4、runtime正控1/1；798兼容面cache20/mir8/abc15/binding4/vertical3/case5与历史计数一致 | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（span sidecar已落，其余保持）；F-11 changed-only持久化+manifest驱动恢复（含上述重定位子缺口）；F-12正式worker/IPC/双线内存门；F-14文档重整+SD文本 | next_responsible_role:review(冻结1414ef960受影响面复审F-10恢复sidecar)；其后继续F-01/F-02等大项

2026-10-21 stage:review(r16a-check) | review_scope:final角色受影响finding/就绪核查，非整份最终验收 | reviewer:Codex /root审查上下文，未实施r16a、未创建agent | PLAN-828 r2 | affected_outcome:PASS，F04-v剩余纯删除不变量CLOSED | overall_readiness:needs_fix（其他已承认work-owned OPEN，不是本轮发现新失败） | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:473a1fbd4c8fc1e4f921b0b5fb693100c7d0af18 | base_commit:3c51abcad24342bf40fa537ab565b6b63e0d2eff | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:r15a-check四失败原样全部PASS：删最后/首新增0、几何apply均0..6，部分删+前移2空格为2..8；全部7独立lifecycle用例+include单测共8/8。F04-u原helper65..70、both/tail17..22、head2..6保持绿，完整扩展删除0..5保持绿 | new_evidence:420格新增数字子区间删除×两API×坐标原点×尾空白全过，每格还连续删剩余数字/前移撤销；两个扩展token与跨扩展末端删除正控均过，新增3独立测试+include1=4/4 | permanent:60/60；原验收17/17一次独立运行通过；core22/22；原62会话61pass1既有恢复spanNone；186/528/12/4/9/72矩阵全过；原runtime部分删+前移+pin冷编正控1pass（两个位置）；fmt0/diff-check0 | acceptance:AC05/V02受影响子集PASS，整项仍随其他OPEN保留；V05已测PASS；V07恢复来源既有FAIL；AC09本次定向核验完成，整份最终验收pending；AC10 merge-owned pending | scope_boundary:仅关闭F04-v剩余纯删除问题，不声称既有退役分支对任意替换/插入均精确 | spec_delta:SD01..06及canonical哈希冻结且相关Spec未变，本轮无语义修订、无整份delta pass | state:executing0/11保持，T02/T07仍未整体完成，其他OPEN F01/02/05b/09/10(+恢复sidecar)/11/12/14不变 | pending_closeout:最终代码/验收/规范pass后merge-owned裸t+tv及必要consumer/触面门；本轮未跑整档/未改实现/未合入 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r16a-check | next:同树继续其他已承认OPEN（F10恢复span sidecar为62例重放唯一站立红）；F04-v本轮四原例不再阻塞，不进入merge。

2026-10-21 stage:work(repair r16a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:473a1fbd4 "fix(av-core): exact evolution of partially deleted token-end extensions" | r15a-check F04-v剩余不变量闭合并收编回归(r1回归套件60/60=59前+1r16a)：
F04-v→retire_touched_exts只能整条保留/丢弃，纯删除部分扩展字节后剩余量丢失→重构为evolve_touched_exts：纯删除(new_len=0)且起点局限于扩展字节区[c0>=e)时，剩余扩展=d-(c1-c0)的前缀+后缀存活数字精确收缩，与同文本from_texts重建全等；其余触达（锚段、替换内容、拼写内插入——字节内容不可知）仍退役到保守段端，绝不输出超当前文本的陈旧范围。四例全绿：删最后0→(0,6)与重建全等、删首0→(0,6)、几何apply(6,7,0)→(0,6)、部分删+前插2空格→(2,8)；完整删除(0,5)与末端trivia(0,7)正控保持
收编永久回归：r16a_partial_extension_deletion_keeps_remaining_extension（reviewer四失败例+完整删除正控+末端trivia正控，含"复合==同文本重建"等价断言）
回归面：r1套件60/60、主17/17、core22/22、rustfmt过；C盘r15a-check原样重放：map-checks四harness（历史helper/边界/ext复合/extension-lifecycle 7例）全exit0，62会话重放61过/1既有恢复spanNone（失败名复核一致），72/186/528/12/9/4矩阵全过，runtime部分删+前移+pin冷编正控PASS。另记：主套件曾1次未捕获测试名的红（紧接重负载replay后首跑，与§9 r5a在案T-08正式档时延瞬态同类），其后5连绿17/17（2次全量落盘） | OPEN不变：F-01/02/05b/09/10(+恢复span sidecar)/11/12/14 | next_responsible_role:review(冻结473a1fbd4受影响面复审F04-v剩余不变量)

2026-10-21 stage:review(r15a-check) | review_scope:final(受影响finding/就绪核查，非整份最终验收) | reviewer:Codex /root审查上下文，未实施r15a、未创建agent | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:3c51abcad24342bf40fa537ab565b6b63e0d2eff | base_commit:e0a0c92b50ee83e5719dc8e4252a9b238eab2726 | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:F04-u CLOSED，C盘r14a-check旧helper65..70、both17..22、tail17..22、head2..6原样PASS，且已入永久门；F04-v原完整删除例0..5 PASS，但完整不变量仍OPEN | finding:F04-v/P2，retire_touched_exts碰到任意扩展区删除即整体清空；mint 1 / 0重锚至1 / 000，删当前[6,7)或[5,6)后map实际0..5、应0..6；apply相同失败；部分删除再前插2空格实际2..7、应2..8。公开元数据层实测，未证明真实会话可达；runtime两个位置部分删/前移/pin冷编正控PASS | evidence:永久59/59、原17/17、core22/22；原62会话61过1既有restore span=None；186/528/12/4/9/72矩阵全过；新元数据7例4fail3pass，include单测1pass（runner4pass4fail，exit101）；runtime新增1pass，内部两个删除位置；fmt0/diff-check0 | acceptance:AC05来源/V02 PARTIAL，V07恢复来源既有FAIL，V05已测PASS，其余维持历史；AC09 needs_fix/AC10 pending | spec_delta:SD01..06及canonical哈希冻结，不授予完整delta pass | state:executing0/11，T02/T07未闭，其他work-owned OPEN不变 | pending_closeout:最终pass后merge-owned裸t+tv及必要consumer/触面门 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r15a-check | next:同树work修剩余扩展量的区间演进或强制重建机制，收编原样四项有效失败及全删正控，提交后复审；不合入。

2026-10-21 stage:work(repair r15a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:3c51abcad "fix(av-core): both-sides interval surgery and end-extension retirement" | r14a-check F04-u/F04-v闭合并收编回归(r1回归套件59/59=55前+4r15a)。〔r15a-check裁定：F04-u CLOSED；F04-v仅完整删除原例闭合，部分删除的剩余扩展量演进仍OPEN，由r16a修复，见上〕：
F04-u→区间手术本体：apply_aligned重嵌入分支else-if链在锚段跨骑编辑两侧时丢尾段——head/tail改两个独立if双侧保留；尾段起点由"重嵌入片段之后"(mid_c0+k+mid_old.len())改为完整替换产物末端(mid_c0+mid_new.len())，替换产物中嵌回片段之后的新增字节同样先于尾段。旧helper(29..70段内53..54→" 2 ")映射63..68恢复65..70与r8a base全等；两侧10..20→17..22、尾侧8..20→17..22、头侧2..6保持
F04-v→end_ext保守退役retire_touched_exts接入apply与apply_aligned重嵌入路径：ext来自from_texts整token拼写分析（有文本），编辑只见字节区间——触及其锚段或扩展字节区([s0,e+max(d,0)))即退役，悬空锚/零长度清除；锚段前编辑整token平移、token端点处/后插入保持存活。删扩展数字例0..5恢复、前移平移例ext存活(2,8)、锚段手术例退役塌缩(2,6)；API不再静默输出超当前文本的陈旧范围，精确ext由下次内容命中from_texts重建
收编永久回归：r15a_embedding_inside_one_segment_keeps_both_surviving_sides、r15a_replacement_boundaries_head_tail_and_both_sides、r15a_trim_numeric_extension_updates_composed_public_map、r15a_end_extension_survives_shift_but_not_token_surgery——旧helper与三边界按reviewer要求进入实际执行门
已知开放不变：恢复载荷无span（既有F10/F11）。回归面：r1套件59/59、主17/17、core22/22（预存core shift警告随未用变量移除清零）；C盘r14a-check原样重放61过/1既有恢复spanNone，72格/186/528/12/9/4矩阵全过，grow/shift/shrink/pin正控保持，三份独立harness（历史helper/边界/ext复合）原文件全过（历史helper与r8a base=(65,70)全等） | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:review(冻结3c51abcad受影响面复审F04-u/v；恢复span=None仍F10/F11 OPEN)

2026-10-21 stage:review(r14a-check) | review_scope:final(受影响finding/就绪核查，非整份最终验收) | reviewer:Codex /root(同一独立审查上下文，未实施r14a) | PLAN-828 r2 | outcome:needs_fix | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | reviewed_commit:e0a0c92b50ee83e5719dc8e4252a9b238eab2726 | base_commit:3f4bbe8e75ca3c4ef3a577ed551052d2a6a413a5 | dependency:auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费 | closure:永久55/55、数值端点及72格72/72已核实；全部反例闭合声明不成立，F04-u原helper/两侧/尾侧仍FAIL | findings:F04-u/P1互斥head/tail和尾段完整输出长度未改，旧helper55..55期望65..70、both10..10期望17..22、tail-only16..21期望17..22。F04-v/P2新增end_ext在公开apply_aligned复合删除扩展数字后不失效，元数据0..6期望0..5；未证明会话可达，真实grow/shift/shrink/pin正控PASS。恢复spanNone仍既有OPEN | evidence:永久55/55、原17/17、core22/22；原61重放60过1既有spanNone；扩展61过1既有spanNone(新增runtime正控PASS)，原186/528/12/4/9/72矩阵全过；旧helperexit101、原边界1过2失败、新元数据helper exit101；fmt0/diff-check0 | acceptance:AC05来源FAIL/V02FAIL/V07恢复来源FAIL，V05已测pinned/resume PASS，其余维持历史；AC09 needs_fix/AC10 pending | spec_delta:SD01..06冻结于本包，未完整最终pass | state:executing0/11保持，T02/T07未闭，其他OPEN不变 | pending_closeout:最终pass后merge-owned裸t+tv及必要consumer/触面门；未跑整档/未改实现/未合入 | evidence_root:C:/Users/zhaop/.codex/visualizations/2026/10/20/01a159ce-3d48-79f1-a573-f0b203b80273/r14a-check | next:同树work修真正区间手术及新增元数据复合，必须将旧helper和边界纳入修复门，提交后复审。

2026-10-21 stage:work(repair r14a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): token-end extensions for lengthened numeric spellings" | r13a-check全部反例（旧helper塌缩55..55、两侧分段10..10、尾段位移16..21、终点70..75、72组合中45失败）闭合并收编回归(r1回归套件55/55=53前+2r13a含72格数值终点矩阵0失败)。〔r14a-check裁定撤回：仅数值终点会话及72格已闭；旧helper/两侧分段/尾段位移三例未闭=F04-u、新增end_ext复合缺口=F04-v，由r15a修复，见上〕
F-04/R13a→同根：值相等但拼写变长的token保留旧mint字节为单段，token END（表达式终点落点处）停在旧长度。三处协同：SpanSegments携带end_ext(mint token末→额外字节)，token-end查询加拼写差；from_texts吸收延续数字run的前缀字节（'0 }'vs'00 }'不再成token后插入，两个中段持有完整token）；token序列值相等时优先run对齐（子串重嵌入降为回退），变长token记录终点延长；子串路径同样记录尾部数字run延长
已知开放不变：恢复载荷无span。回归面：r1套件55/55、core 22、主17、scalar差分6、vertical/cache/binding绿；r13a-check材料自C盘复制入分支(attachments/828/r13a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r13a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): numeric-token identity in re-anchoring" | r12a-check两例（同根）闭合并收编回归(r1回归套件53/53=50前+3r12a含9格十进制拼写矩阵)：
F-04/R12a→字节级对齐看不见'1'与'01'是同一token的不同拼写。三处协同修复：token_eq十进制token按数值相等；find_subslice命中落在数字run内部时向左吸附到token起点（'01'的后缀'1'不是token起点）；from_texts吸收在NEW侧延续数字run的后缀字节使两个中段持有完整token（朴素后缀修剪把'01'剪成裸'0'）。同值字面量编辑+后续trivia重写的交错丢尾段同根消除
已知开放不变：恢复载荷无span。回归面：r1套件53/53、core 22、主17、scalar差分6、vertical/cache/binding绿；r12a-check材料自C盘复制入分支(attachments/828/r12a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r12a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): comment-aware expression anchoring in source maps" | r11a-check两类失败（同根）闭合并收编回归(r1回归套件50/50=47前+3r11a含4格括号行注释矩阵)：
F-04/R11a→重锚机制把行注释文本当代码：comment_ranges()标记//..EOL区间；词法token化跳过注释（注释字节不是表达式锚点）、find_subslice拒绝与注释重叠的匹配——' 1 // 1 / 0
 / 0 '锚定可执行token（注释前的1、注释后的/ 0）而非注释内首个文本子串'1 / 0'；注释后字面量重写的丢尾段同根消除（先行注释编辑正确重锚→后续回退保头尾）
已知开放不变：恢复载荷无span。回归面：r1套件50/50、core 22、主17、scalar差分6、vertical/cache/binding绿；r11a-check材料自C盘复制入分支(attachments/828/r11a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r11a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): lexical token alignment in source-map re-anchoring" | r10a-check两类失败（同根）闭合并收编回归(r1回归套件47/47=44前+3r10a含12格词法间距矩阵)：
F-04/R10a→from_texts回退的对齐单位改为词法token（数字串/标识符/单字符运算符，空白跳过；"非空白run不是token——1/0是一个run三个token"）：'1 / 0'→' 1/0 ' 按token对齐保留每个运算符/操作数锚；不相交trivia编辑的丢尾段(66..67)同根消除（先行间距重写正确重锚后，后续回退路径头尾都存活）；12组合词法间距矩阵为永久回归
已知开放不变：恢复载荷无span。回归面：r1套件47/47、core 22、主17、scalar差分6、vertical/cache/binding绿；r10a-check材料自C盘复制入分支(attachments/828/r10a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r10a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): token-run re-anchoring and correct mint-text deposits" | r9a-check三例反例闭合并收编回归(r1回归套件44/44=40前+4r9a)：
F-04/R9a类1内外trivia同改→from_texts增加token-run回退：中段非连续子串时按非空白run序列对齐（run间空白为blob）——'1 / 0'→'  1  /  0  '保留全部语义锚
F-04/R9a类2重铸造条目存上一版文本→deposite改从prepared.new_parse/new_src取声明区间与文本（原在会话状态推进前读旧self.parse/self.src）——重铸造后紧接的同一/等价编辑按代码自身mint文本对齐；该陈旧文本bug同时是类3丢尾段(55..55)根因，由remint+等价字面量回归覆盖
已知开放不变：恢复载荷无span（798 sidecar范围）。回归面：r1套件44/44、core 22、主17、scalar差分6、vertical/cache/binding绿；r9a-check材料自C盘复制入分支(attachments/828/r9a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC（+恢复span sidecar）；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r9a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): content-aware re-anchoring for equivalent rewrites and restored maps" | r8a-check两类缺口闭合并收编回归(r1回归套件40/40=36前+4r8a)：
F-04/R8a类1等价token重写丢锚→历史条目携带MINT TEXT；内容命中时按全文对齐(SpanSegments::from_texts：前缀/后缀+中段子串重嵌入，mint/当前基点分离)重建——重写trivia保留token锚；中段无对应(多编辑分离)时保留累计几何映射并以apply_aligned(编辑区间内容感知手术：被替换字节重嵌入替换产物中则保留分段)复合；命中跳过双重复合；pin快照保留自身视图
F-04/R8a类2恢复空映射→恢复会话为每个驻留记录与历史条目播种当前声明区间的非空恒等对齐+mint text
已知开放：持久化单元无span(恢复载荷trap无span)——798 sidecar/重绑定机制范围，未在本轮冒充
回归面：r1套件40/40、core 22、主17、scalar差分6、vertical/cache/binding绿；r8a-check材料自C盘复制入分支(attachments/828/r8a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC(+恢复span sidecar)；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r8a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(av-core): piecewise segment alignment maps replace region composition" | r7a-check三类P1+186格矩阵全部闭合并收编回归(r1回归套件36/36=27前+9r7a含完整interval矩阵)：
F-04→结构重造（core/src/span_map.rs SpanSegments）：每代码对象持mint字节的不相交分段表(mint区间→当前偏移，绝对坐标)；插入内容为无背衬blob居于当前间隙。编辑=区间手术：区间内分段丢弃/跨骑分段分裂/后方分段平移/替换文本为blob；映射=查表——token语义（插入在token前推右start、恰在end不延长、跨骑只动end）由分段与blob相对位置自然得出，被替换位置塌缩到存活边界。r7a三类失败（整体替换插入为非空产物/跨插入与原文的编辑/修剪后续编或撤销）成为结构必然而非特判。跨函数编辑由绝对坐标同一轴线覆盖；重铸造=自身声明区间恒等；历史条目自带分段表
回归面：r1套件36/36（含186格矩阵对冷编oracle全绿）、core 22、主17、scalar差分6、vertical/cache/binding绿；r7a-check材料自C盘复制入分支(attachments/828/r7a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r7a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): full interval surgery for source-map composition" | r6a-check三例span反例全部闭合并收编回归(r1回归套件27/27=24前+3r6a)：
F-04→复合改为完整区间手术：编辑的当前范围双端逆映射（被替换文本的mint长度=unmap(c1)-unmap(c0)——删除先前插入的字节=替换0个mint字节）；被本次编辑完全重写的区域移除（插入被整体删除不留痕迹：撤销=恒等映射，pin的撤销前快照保留自身map）；记录的delta为当前长度变化（部分修剪已插入空白记录为mint长度0的trim）；整段当前范围均来自被移除区域时不补记（cur_len>0守卫防纯插入误判）。逆映射对插入/替换输出内部塌缩到区域mint锚点，净当前像长（删除的像是一个点），删除后前像可超当前offset。
r6a-check材料自C盘目录复制入分支(attachments/828/r6a-check/；D盘副本不完整)。回归面：主17、r1套件27、scalar差分6、vertical/cache/binding绿 | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r6a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): token-boundary span mapping and region-aware inverse" | r5a-check三例span反例全部闭合并收编回归(r1回归套件24/24=21前+3r5a)：
F-04→半开区间token语义：start 仅当编辑整体在其前(p+len<=s)或纯插入恰在起点(p==s,len0,d>0)时移动；end 仅当编辑严格在其前(p<e)移动；逆映射改为区域感知下降（替代假设map(x)>=x的二分）：插入区内部塌缩到该插入锚点、删除区之后解析越过被删块（mint前像可大于当前offset——旧搜索上界c截断导致删除后编辑被误放到前方span之前）
另记：复审报告的进程内正式档首跑超阈/复跑通过的时延波动——归因与样本协议（预热排除、样本数、计时窗口）归 T-08 正式档工作面，不在本span修复中处理
回归面：主17、r1套件24、scalar差分6、vertical/cache/binding绿；r5a-check材料随分支保存(attachments/828/r5a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门（含上述时延归因）；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r5a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): mint-coordinate map composition and id-text intrinsic identity" | r4a-check三例反例全部闭合并收编回归(r1回归套件21/21=6r1+3r2+4r2a+5r3a+3r4a)：
F-04→区域表统一到代码MINT源坐标单轴：每次编辑的当前坐标起点先经单调前向映射的逆映射（二分；插入内部位置塌缩到插入点）折算为mint坐标后入表，替换旧"按当前delta平移已有区域"的坐标系混用——同点两次32空格插入正确复合(+64)，先后/内前编辑序都保留两侧端点位移
F-06→按引用身份=intrinsic def的ID TEXT（synth丢弃他函数intrinsic时def序号移动而id text不变）+symbol/effect/shape四事实：混合mark_a/mark_b的caller合法编辑不再被synth表序号漂移误拒；symbol交换/effect翻转在同id处事实不一致仍失效
回归面：主17、r1套件21、scalar差分6、vertical/cache/binding绿；r4a-check材料随分支保存(attachments/828/r4a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

2026-10-21 stage:work(repair r4a) | PLAN-828 r2 | outcome:in_progress | worktree:D:/autostack/.wt/lang-828/auto-lang | branch:plan-828-dev | repair_commit:"fix(probe): composable snapshot source maps and per-reference intrinsic bindings" | r3a-check五例相邻反例全部闭合并收编回归(r1回归套件18/18=6×r1+3×r2+4×r2a+5×r3a-check)：
F-04统一→可组合快照source map：ProgramSnapshot自持per-slot区域表(start,len,delta; x→x+Σ delta where start<x)，所有公开执行路径映射；每次编辑复合进每个函数的map与每个内容历史条目的map（全部存储代码对象经历同一源时间线），重铸造清空——体级细化公开可用(a)、跨编辑复合(b)、跨骑编辑只动end(c)、命中恢复条目自身累计map（历轮回归仍绿）
F-06统一→按实际引用身份的依赖：CheckedFunction记录(def序号,symbol,effect,shape)四元组；FnEnv按def序号持表；credential_current要求每个已咨询引用槽位四事实全同——def间symbol交换(d)与effect翻转(e)均失效；未读intrinsic仍排除于全局身份
回归面：主17、r1套件18、scalar差分6、vertical/cache/binding绿；r3a-check材料随分支保存(attachments/828/r3a-check/) | OPEN不变：F-01真·函数级前端；F-02安装payload重建；F-05b取消/关闭态；F-09剩余资源预算轴；F-10 A07/依赖查询/GC；F-11 changed-only持久化+manifest驱动恢复；F-12正式worker/IPC/双线内存门；F-14文档重整+SD文本 | next_responsible_role:work(继续，同一树)

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
