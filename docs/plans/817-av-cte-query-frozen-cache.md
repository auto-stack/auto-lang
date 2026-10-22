---
plan_id: PLAN-817
status: executing
feature_name: AV驱动的CTE查询与冻结缓存闭环
plan_revision: 3
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-23
current_step: 2
total_steps: 11
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-semantics/semantic-contract.md
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-cte/project.md
  - docs/specs/auto-cte/scalar-query-profile.md
  - docs/specs/auto-cte/frozen-cache.md
touched_goals: []
affects: [auto-cte, auto-semantics, auto-av, auto-test-case]
---

# [PLAN-817] AV驱动的CTE查询与冻结缓存闭环

## 0. 变更摘要

承接[Design42](../design/42-semantic-metadata-inference-and-comptime.md)、[Design43](../design/43-ac-av-typed-mir-and-abc.md)和796实交语义合同，将CT模型接缝推进为有限的真实执行：`CT查询 → 已检查callee HIR → 798共享MIR → ABC → AV → 冻结结果 → consumer推导/HIR重验`。首个可执行用例同时接真实store、AutoDown、阶段计量和内存预留；不是把ModelCt换个名字。

本次仅修订文档，已有准备/在途成果保留；与[818](818-ac-shared-mir-native-parity.md)、797/798及拆分的依赖/单写面见[批次并行合同](attachments/ac-execution-batch-20261019/parallel-contract.md)。T-01可独立设计，T-02起必须满足D-798及D828和单写交接，不复制在途VM。818和797不是本计划前置。

## 1. 目标

1. 实现有限i32/bool的实际CTE调用、查询DAG调度、递归依赖环诊断和受控失败；callee检查先于执行，不要求consumer所在全程序先final Checked。
2. 复用796语义身份/类型与可移植值准入，以独立版本化的真实执行envelope记录recipe、执行引擎、结果及依赖；不放松796模型准入。
3. 真实冻结结果回灌consumer的有限推导与HIR常量节点，重走准入，不通过表达式格式化/源码重parse执行CTE。
4. A05结果缓存完整覆盖helper body、签名、参数、Meta集合/负依赖、版本/profile/权限及目标语义；跨进程/工程命中不执行AV，变化时正确重算。
5. 正式AutoDown对实际producer/consumer判定，从首例保留端到端trace、内存峰值、I/O和重算计数，规定编辑路径P95<1000ms。

范围：新独立 `experimental/ac-cte-probe/` workspace（contract/core/probe按职责分包）、`test/cases/ac-cte/`、817脚本/报告/附件；正式runner具名recipe及最小注册/分派。只读796/798/783/ac-core，不接生产默认编译入口。

非目标：Auto完整元编程语法、类型/声明生成、type-as-value执行、用户泛型实例化、runtime反射调用、text/record运行值、任意IO/环境/时间/随机/网络/FFI、JIT、发布器、全面前端/缓存生产化、自举、MIR→native新后端。只读Meta描述可作参数来源，不等于AV已能操作record/类型对象。

## 2. 架构方案

输入为版本化的**CTE探针recipe IR**：callee源码/模块身份、显式签名与常量参数、有限查询节点、consumer语义模板/HIR绑定、profile及预算。它是测试/查询接口，不新增Auto语言语法。普通scalar函数源码复用既有前端；consumer中的待求值位置由新recipe的具名CtRef表示，不把Unknown交给796求解器。

```text
请求快照 / 查询DAG
  → 796当前签名/Meta查询 → CT调用实参
  → 检查callee可执行闭包（不含待求值consumer）
  → 798 HIR→MIR→ABC→AV → portable冻结值+依赖manifest → 783 A05 producer
  → 在受控consumer模型里替换CtRef
  → 796 infer + owner/type校验 → HIR常量回填 → 既有verify
  → 当前结果/来源诊断；必要例经旧native路径观察
```

contract/core默认依赖闭包不得有UI/Man/旧VM/Cranelift；driver可消费现有ac-core及798 probe公开adapter、783 store。只从实际合入API调用AV，若798未来变动与本草案观察不同，在T-02记录映射并复验，不在798代码里加第二套推导/执行器。

## 3. 技术栈

Rust独立Cargo workspace+跟踪Cargo.lock；796 `ac-sema-contract`/`ac-sema-core` 公共类型，798 `auto-av-core` 的验证/执行服务和HIR adapter，783 `ManagedStore`，正式AutoDown probe/run及788 contained执行/原始bytes/父侧判定。标量语义不依赖宿主位宽；参考设备Windows x64，其他target不据本计划授信。

拟新增 `contract/src/{request,frozen,admission}.rs`、`core/src/{schedule,dependencies,consumer}.rs`、`probe/src/{av_driver,store,worker,observe}.rs` 及 `scripts/verify-ac-cte-817.ps1`。这些是未来路径，不是已实现入口；T-02实现 `-Scope first-case`，T-08聚合 `admission|semantics|cache|resources|autodown|turnaround|acceptance|closeout`。

拟议实验预算（非用户指定值/永久语言上限）：recipe+source/HIR总输入≤4MiB；每请求≤128查询节点、≤128层查询依赖、遍历≤100万步；结果页≤512KiB、单次元数据IPC≤4MiB、stdout/stderr各≤1MiB。AV fuel/frame/slot/wall不能大于D-798已冻结cap，权限更严；每请求墙钟≤10s。失败均带typed resource/cancel/infra，不截断为合法值。

受管总额硬门≤32MiB（hot 8、请求/编码/索引暂存16、AV槽8；跨库同时持有的796对象计入相应分区，不重复领预算）。worker自身private bytes峰值硬门≤384MiB；整个测试进程树采样并单报，CTE请求路径峰值≤512MiB；native参考进程另报，不从worker峰值中任意扣掉前端。预算申请须先于clone/decode/编码，运行时采样含瞬态峰值；超标不填target_met=true。

## 4. 需求分析与背景调查

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

初稿授权记录（2026-10-19）：用户要求草拟本计划及818，明确797/798/拆分并行、首例cache/AutoDown/亚秒/内存。该轮只调查和写计划，不创建实施树/代理、不安装工具、不修改canonical Spec或他方实现，不联系其他会话。

依据[overview](../specs/overview.md)、[796语义合同](../specs/auto-semantics/semantic-contract.md)、[查询cache](../specs/auto-semantics/query-cache.md)、[阶段合同](../specs/auto-hir/stage-contract.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[测试架构](../specs/auto-test-case/architecture.md)。Design42/43为目标；auto-cte/auto-mir/auto-av Spec尚未在当前主检出完整交付，D-798后重读实际Spec。

调查主检出 `13367d127584461449ee2f151c0c5987f0740044`，796已delivered/archived，787/788已交付；797 executing r1、798 executing r2（现场快照，work须重新核验）。逐文件/分支blob hash见[inputs.json](attachments/ac-execution-batch-20261019/inputs.json)。

实际接缝：796 `core/src/reflect.rs::{CtRequest,CtResult,PortableValue,ModelCt}`；模型默认wrapping sum不是新CTE数值oracle。`CtResult::admit_as_model`拒vm_execute，不得删除；真实执行使用新schema/producer envelope，不给模型添信任。796 `InferSession::infer_function`、`SemaInput`和store/query为consumer事实来源。ac-core `compile_parts_with/verify*`、公开HIR表可用于callee和consumer；其源码adapter发射内部Atom HIR是现存机制，不等于允许把CT表达式格式化回Auto源码解释。

只读在途 `plan-798-dev`观察到 `CheckedMir::module`、`execute_entry(LoadedModule,limits,cancel,args)` 和i32/bool槽/ABC；分支不是已交付依赖。797原生ABI实现不在本任务调用链。814–816拆SDK/UI/工具协议可能迁移AutoDown编排位置，注册阶段必须核当时唯一owner及当前模块路径，不趁机修改SDK/VM产品包。

**D-798**=最终review pass、冻结门禁、实际合入、Spec及merge收据齐全（不只frontmatter翻状态）。T-01只读决策/独立golden可先做，T-02起所有依赖MIR/AV的代码/构建等D-798；不复制branch接口、保持长期mock或在假AV上交付。797/818无前置；公共测试注册串行，不制造互等。

## 5. 详细设计

### 5.1 首版CTE执行及consumer合同

值级支持i32/bool参数/返回、pure直接函数调用、有限控制流及D-798已支持精确算术/trap；普通函数递归由AV fuel管，不等同CT查询依赖环。CTE默认无import，连测试trace标记也不是普通CT权限；查询前审查闭包所需能力，拒trace/任意宿主服务在av_execute=0处完成。未知指令/profile或只存在模型producer不得执行。

查询节点只有：字面量、已完成query引用、`Call(decl,args)`，及显式能力下的 `FieldCount(record)`/`HasField(record,name)`。后两项通过796当前签名/字段投影读取，分别返回i32/bool，再作为实际AV调用实参；它们本身记录meta_query计数，不冒称AV执行。不存在record的查询报解析/语义错误，已存在record中缺字段返回false并记录集合负依赖。Meta能力默认不授予，需具名 `cte.meta.fields/v1`；不允许文件/环境broker或任意宿主函数扩面。

consumer推导面严格限796已交付的core i32/bool操作（含+/*/<、局部与直接调用/条件约束），不因callee由798支持更多scalar操作就默默扩796求解器。consumer输入仅为有限语义模板+与HIR常量位置的owner/type绑定（CtRef属于新recipe IR）。CT结果投影成796 InputExpr字面量，才进行局部推导；HIR模板对应位置回填 `Constant` 后重新verify，带双向绑定/source origin，错误绑定、重复绑定、类型错和跨owner都拒绝。T-01冻结该映射/Schema与例子，T-02证明结果确实改变推导事实和最终HIR/native可观察行为。不能只计算值输出JSON就称回灌完成；也不称完整Auto源码`#`/comptime已接入，生产语法留后续。

首例计算sum(20,22)=42：真实AV返回I32(42)、consumer推导为i32、HIR含42，独立expected及既有native自检出口0；bool另有分支正例。首次真实执行例即完成存盘→新worker恢复→必要验证→结果复用、正式AutoDown一次运行和trace/峰值，不将cache/runner拖到后几项才第一次出现。

### 5.2 请求、调度和失败

请求key含签名/实际callee identity、常量参数、查询算法/profile、引擎及MIR/ABC/冻结codec版本、权限/目标语义等；callee只能来自当前796声明和经过验证的执行闭包。调用者传来的deps字符串只作输入提示，不能代替生产者记录依赖。不同包同名及方法owner必须区分。

调度用显式查询栈/worklist、InProgress/Completed/Failed及当前snapshot；A→B→A等无解依赖环在调用链定位，不能靠“全程序先Checked”绕过。普通递归callee可在已检查签名下执行；查询取消/超限及旧snapshot迟到不发布当前结果。重复纯查询可single-flight，但失败/取消不缓存成永久语言错。

compiletime已执行溢出/trap→有CT调用链的 `cte.*`语义诊断；未执行分支trap保持潜伏；fuel/内存/取消/宿主崩溃→资源/基础设施，不能当预期语言负例。诊断投影用已支持stage/type+精确code和来源，不因CTE另造未知阶段并被runner忽略。

### 5.3 冻结、信任和cache

新增版本化CteExecuted/Frozen envelope，值只支持i32/bool，携带声明/预期类型、完整request身份、依赖manifest、执行语义版本、origin sidecar和实际producer。可引用796 PortableValue，但保留其原有模型准入规则；单写marking=vm_execute、输入自带trusted或仅有内容hash都不能证明适用于当前请求。新执行producer的创建路径绑定真实AV成功结果；恢复是数据校验+新鲜性检查，不反序列化进程信任。

复用783独立namespace/合法通用载体、内层A05 Schema；不改旧producer Future覆盖、不复制796 store数据库。类型/常量等消费必须存活的数据是required边，helper body及来源/失效记录不永久pin全部AST。字面常量native物化后无需保留AV活堆。

至少记录实际helper body/签名/调用闭包、CT实参、读取Meta集合/候选负依赖、算法/Schema/AV版本、配置/权限/target相关语义。实现可保守依赖可执行闭包全部body，但必须声明粒度，不能声称动态读集最小化。上次manifest必须先验证完整新鲜，源码不变但helper/字段集/权限变化仍失效；同名异包、好frame错地址/owner/key、旧嵌入类型都拒绝或受控miss。

cache hit：ct_execute=0、模型计数不增加，实际冻结consumer仍重新准入/比较；通过禁调AV的反控证明。miss正控ct_execute>0且是AV真实调用，不能命中旧运行PASS。成功值跨进程/工程共享；瞬态resource/cancel不持久成程序错；失败发布保留旧提交，释放lease/暂存。

### 5.4 AutoDown、资源与周转

拟正式recipe `cte/scalar-query-v1`（v2 probe/run，atom-data源，steps=query/edit-source/restart）使用独立worker；沿用受控生命周期/工具身份/父侧判定，不借 `sema/query-v1` 名字混入新语义。首次T-02就接最小注册+独立父侧adapter；T-06完善所有正反例。不得另写.ad parser、case携shell或自造PASS。原v1、788 native/syntax、796/已合入797/798 recipes兼容，未知字段/recipe拒绝。

公共注册/分派在797/798和814–816当时写面owner交接后以短窗口单写；只追加本recipe和独立文件，完整executor复用现成设施。若正式接口无法表达已定源/步骤，带具体反例显式修订合同，不在T-02后保留一个长期私有runner。T-02尚未正式运行AutoDown不得勾选。

规模：独立脚本流式生成≥32MiB真实冻结页/manifest、512次**不同输入**修订和真实query/重键/结果断言，同store承载冷数据及GC；活lease期间强引用保活、释放后按783水位收敛，用户哨兵不变。不让128单请求query上限阻止跨请求造数。测试须用真实不同产物推动同store超过高水位，确保至少一次物理删除并在活集低于低水位时实测收敛；不以未触发GC的低占用算通过。依赖链128边界两侧、合法DAG/重复子查询、坏嵌入引用、深层输入/解码/释放全路径有控制。

T-01冻结参考Windows机器/工具、256KiB有意义callee/consumer输入（100函数规模）、元信息规模及局部编辑配方。每场景≥30原始样本：fresh每次新store、同进程hot、每次真新进程restart、每次新版本body/helper编辑、字段/负依赖编辑、真实并发GC；路径必须与其名字一致。

硬门：warm session局部CT helper body编辑及Meta集合编辑的“请求准备/序列化→IPC/排队→新鲜性/CT执行或命中→冻结/consumer推导/HIRverify→父侧准入/比较”单墙钟P95<1000ms。包含真实必要等待，不扣序列化/哈希/比较；数值、trace和计数逐样本绑定当轮新hash，不删失败。完整native参考另报，此门是CTE查询端到端，不冒称完整应用编译/链接或热重载亚秒。内存硬门与时间同轮验，低内存不能靠漏计进程/编码暂存达标；无cache结果/消费观察=cache路径语义基线，字节复现另定。

### 全链路增量兼容与828优先排期（r2）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：CTE helper body/Meta/负依赖进入真实query，命中不执行AV；仅重算受影响query/consumer；只读复用828会话和函数接口，不整模块重建。

T-11 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-11 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 执行排期与单写交接（2026-10-21）

优先批次的CTE消费线；不等818或820–827全部完成。只读公共语义/MIR/AV，独立CTE查询/冻结/consumer实现可与818、827消费层并行；公共recipe/dispatch等写面须取得窗口。

所有后继在D828前仅作只读准备，现有实现与证据保留。共用[执行顺序与交接合同](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)：T-01准备写面/owner/消费者映射，T-02在D门满足后刷新基面、版本、预算与接口并记录窗口接受；既有兼容/coverage工作记录定向结果，独立review核记录，merge补实际合入与窗口释放。公共核心默认单写，公共注册追加也串行；不同worktree不豁免。不能持窗等未交付依赖、复制WIP或改变他方在途合同；无公共写面者可并行消费稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-cte/project.md、scalar-query-profile.md | 模型→有限真实AV查询/能力/consumer回灌 | 不冒称全语言comptime | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-cte/frozen-cache.md | A05真实producer、依赖/marking/恢复/资源/周转 | 正确跳执行和冻结消费 | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-semantics/semantic-contract.md | 796模型不变，追加指向真实CTE版本域/消费接缝 | 不让旧模型mint执行结果 | AC-02/04 |
| SD-04 | modify | docs/specs/auto-ac/artifact-contract.md | 追加817 producer有限A05实现，旧783 producer仍Future | 不扩大历史授信 | AC-04 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 追加正式CTE recipe/worker身份/观察，旧recipe不变 | 单一裁判 | AC-07 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-11 |

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

拟新test族 `cte_first_`、`cte_schedule_`、`cte_admit_`、`cte_freeze_`、`cte_cache_`、`cte_resources_`、`cte_turnaround_`、`cte_adapter_`，由相应T创建；命令例 `cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml cte_cache_`、新verify脚本相应Scope。当前不可运行，不算证据。

| Case | AC/invariant | 输入/反控 | owner/真实入口 | 必须观察/边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 真实闭环 | sum=42、bool、CT结果改变consumer类型/分支 | T-02/03 worker→AV→infer/HIRverify/native参考 | 独立expected+实际AV及消费者行为，非只JSON；不支持类型生成 |
| V-02 | AC-02/03 顺序/阶段 | 未走trap、已走溢出、helper调用、consumer未完成 | T-03 schedule/AV | 未走不trap，已走CT诊断含调用来源；不先要求全程序Checked |
| V-03 | AC-03 环/取消 | query小环/长环、合法共享DAG、普通递归、fuel/取消 | T-03 scheduler/AV | 查询环定位，合法DAG/函数递归不误拒；资源/infra分态 |
| V-04 | AC-02/04 信任 | model_compute结果/伪vm标记/错预期类型、错owner、未知权限/导入 | T-02/04 生产与消费准入 | 模型无权冒领真实结果；不合法输入AV计数0；合法反控通过 |
| V-05 | AC-04 真缓存 | fresh→hit→退出→新worker/另一工程；AV禁调 | T-02/04 store/AV/consumer | hit ct_execute=0、miss>0，每次consumer重准入/比较，不缓存PASS |
| V-06 | AC-04 依赖 | caller源不变改helper body/签名、FieldCount/HasField新增成员、schema/profile/caps/target语义 | T-04真实query | 相关值失效且结果=无cache，未读面按声明粒度复用；负依赖有效 |
| V-07 | AC-04/05 坏块/生命周期 | 自洽异址、好frame错内层/页key/嵌入ref、冲突/取消发布、GC/lease | T-04/05 真实读取和消费 | 受控拒绝/miss、旧提交不坏、活数据保活、无输入unavailable |
| V-08 | AC-05 规模内存 | 32MiB页、512真实修订、128链边界/错误引用、瞬态spike | T-05资源/峰值采样 | 32MiB受管/384MiB worker/512MiB树硬门，全路径有界；spike反控真红 |
| V-09 | AC-06 周转 | 六场景×30，helper/Meta编辑，阈值=0及空/负/NaN样本反控 | T-07实际runner/harness | 端到端P95<1s；当轮不同hash、raw可重算；阈值0/坏样本gate非0 |
| V-10 | AC-07 runner | 未知recipe/worker身份、坏actual好expected、restart及旧协议族 | T-02/06 auto-case lint/run/stale | 真dispatch/父判、错误不能绿，resource控制断言外层ERROR非语言PASS |
| V-11 | AC-08 并行/依赖 | D-798、zero-product core tree、797/818零依赖、SDK迁移路径 | T-08边界审计 | 无复制VM/改旧sema模型、无根SDK/存储热点；输入hash/SD齐 |

V-01–11全部为必验区分性控制；独立expected不由AV/新freeze生成，反控实际变异有hash/bytes证据；required缺例/skip/工具缺失不绿。实现填每项入口/命令/commit/输入hash/raw位置，旧证据可复用须说明代码/依赖/配置等价。

唯一预定树 `D:/autostack/.wt/lang-817/auto-lang`、分支 `plan-817-dev`（尚未创建）。实施/自检后同树独立上下文review，冻结提交、暂停写入；未指定模型/人员，不创建代理。work只相关定向测试和必要check，不跑根整档；最终review pass写merge gates pending，merge按实际注册触面跑独立workspace全套、runner定向兼容，根Rust变更按AGENTS补裸t及tv/tt；无相关aavm/UI/生成器改动不跑taa/tu/docs_gen，tf仅到期主检出单实例。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-12 | AC-11 / T-11 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-11 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

## 7. 验收标准

| ID | owner_stage | 可验证结果 |
|---|---|---|
| AC-01 | work | T-01决策及T-02首例真实执行/cache/正式AutoDown/trace齐；V-01/05/10 |
| AC-02 | work | 值级调用及有owner/type来源的consumer回灌真实改变事实/HIR；V-01/02/04 |
| AC-03 | work | 调度/查询环/能力与trap、资源/取消分态；V-02–04 |
| AC-04 | work | 冻结身份/依赖/新鲜性、跨进程命中跳AV和坏块消费准入；V-04–07 |
| AC-05 | work | 配额先预留、真实峰值/32MiB规模/512churn/生命周期硬门；V-07/08 |
| AC-06 | work | 正式runner路径两编辑P95<1s及六场景raw/失败反控；V-09 |
| AC-07 | work | 正式recipe/stale/父侧oracle/旧通道兼容，无私有runner；V-10 |
| AC-08 | work | 依赖/SD/源码边界和支持表完整；V-11 |
| AC-09 | review | 同树独立逐AC/V/SD及遗漏/延期核验，绑定提交；无开放finding |
| AC-10 | merge | 冻结门禁、Spec/ledger、归档合入与guard清理实际完成，10/10 |
| AC-11 | work | D828与新增能力保持全链路增量，V-12 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

| Task | owner_stage | 前置 | 文件/交付 | 验证/AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施指令 | 只读D-798接口/CT consumer映射/权限/预算设计，固定recipe Schema、256KiB负载/独立oracle/资源归属，contract-decision.md | 设计与V映射可审；AC-01；不编译在途接口。[✅ 已完成] 2026-10-20：`experimental/ac-cte-probe/contract-decision.md`（worktree 提交 2c38609eb）冻结 §1 身份/Schema 表（recipe IR `ac817/cte-recipe-ir/v1`、envelope `ac817/cte-executed/v1`、层 `ac817/a05-cte-exec/v1`、producer `plan817/cte-probe`）、§2 D-798 接口只读观察（plan-798-dev@072cb4e14：`execute_entry/LoadedModule/ExecLimits`+全冻结 cap 常量；796/783/ac-core master@08ba8ca2b 实际 API；未编译在途接口）、§5 consumer 双向绑定（let 锚点→InputExpr 字面量→infer；HIR `Constant{v}`/`0<n` bool 子树回填+重verify；全拒绝项清单+首例 sum=42/bool 分支例）、§6 冻结 envelope/依赖 manifest/cache 行为、§7 分区预算（hot8+暂存16+AV槽8≤32MiB；worker 384MiB/树 512MiB；AV/验证 cap ≤D-798）、§8 参考负载 load256cte/闭式 oracle/六场景/编辑配方/GC 冷集≥80MiB、§9 AutoDown 五点注册映射、§10 V-01–11 映射。验证=文档审查（Category A doc-only，无 cargo）；T-02 复验义务已记入 §2 |
| [x] T-02 | work | T-01、D-798、公共注册窗口、D828排期门、当前单写窗口交接 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； 新workspace/lock、实际AV首例/新schema冻结+store恢复+consumer最小回灌+正式AutoDown首例+trace/峰值；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记 | cte_first_/cte_admit_/auto-case；V-01/04/05/10；AC-01/02。[✅ 已完成] 2026-10-23：code b249767d0（worktree plan-817-dev，base d46e03e8a=828 归档后 master HEAD）。**门核验**：D-798 满足（798 archived，merge 收据 d58a77a56/fe1eb28a2，Spec auto-mir/auto-av 齐）；**D828 排期门满足**（828 archived 02af08442，merge cleaned d46e03e8a）；单写窗口经实际 diff 核验（master 注册表=sema/abi/ac-av；816 分支不触 crates/auto-test-case 与 test_cases；818 未合入暂停在案——按并行合同 817 先注册，818 rebase 后复验）。**T-01 合同刷新**：contract-decision r1.1 重读合入 API（ABC wire /2、ABC_SEMANTICS=2、新增 SLOT_CELL_BYTES/MAX_VERIFY_ANALYSIS_BYTES、828 session/formal_worker 面不在 817 消费面；消费 API execute_entry/ExecLimits/LoadedModule/verify_abc_bytes/lower_checked 签名零漂移），预算口径不变。交付：新 workspace（contract/core/probe+跟踪 lock）；首例 sum(20,22)=42 真 AV（wrapper entry cte817_call 内联常量实参调真 callee，ct_execute=1）；consumer 双向回灌（let 锚点→SemaInput 字面量→infer i32/bool；HIR 原位 Constant/复用 0<n 操作数槽回填→重verify 过）；native 可观察（回填 CheckedModule 直入 build_stages，无源码重parse；exit 42；bool 例分支翻转 2→1）；冻结 envelope ac817/cte-executed/v1（783 LayoutData 载体、全字段恢复准入+新鲜性、错 producer=受控拒绝非 miss）；跨进程恢复（真新 worker 进程 hit、ct_execute=0）；AV 禁调反控（miss 受控失败/hit 照常完成）；跨工程命中；正式 AutoDown（REGISTERED_PROBE_RECIPES+=cte/scalar-query-v1、Options.probe_cte、end-with-query、ac_cte.rs 适配器 containment/回显/父侧 edit+hash 复验/restart 分块真新进程、stale cte 轴）+语料 2 case；正式运行 2/2 绿（cache published+exit 42/1）+错误不绿反控（错 exit→Fail）+auto-case lint ok。trace/峰值：Obs spans+mempeek PeakSampler 逐步采样已接（断言 peak>0）。测试：workspace 19/19 绿（cte_first_ 6+cte_admit_ 9+deck 4）。**写面登记**：repo 根 Cargo.toml exclude+="experimental"（修 cargo 嵌套 workspace 新鲜解析 bug，814 面 owner 已归档，818 rebase 受益）；contract/suite/mod/stale/tests 为合同允许的追加式注册 |
| [ ] T-03 | work | T-02 | core/schedule/consumer、CT片段检查/纯调用/Meta参数broker/环与取消 | cte_schedule_、V-01–04；AC-02/03 |
| [ ] T-04 | work | T-03 | 完整依赖/冻结/恢复/跨工程/坏块/root/lease，model与真结果隔离 | cte_freeze_/cte_cache_、V-04–07；AC-04 |
| [ ] T-05 | work | T-04 | 真实页/512修订/GC、全路径资源和峰值采样/spike反控 | cte_resources_、V-07/08；AC-05 |
| [ ] T-06 | work | T-05、公共单写窗口 | AutoDown完整suite/worker/兼容与身份/stale、ERROR反控 | cte_adapter_、V-10；AC-07 |
| [ ] T-07 | work | T-06 | 正式runner六场景及两编辑硬门、raw报告/反控 | cte_turnaround_、V-09；AC-06 |
| [ ] T-08 | work | T-07 | 新verify全Scope、coverage/SD、docs/reports/plan817、边界审计 | acceptance V-01–11；AC-01–08；execution_done=8/10 |
| [ ] T-11 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-12 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-11；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-09 | review | T-08冻结提交、T-11 完成 | 同树独立review/规范及遗漏审查，反控可重放 | AC-09；reviewed=10/11，merge pending |
| [ ] T-10 | merge | T-09 pass | 冻结提交门禁/Spec/ledger/归档合入/批量到期判定/wt-guard后清理 | AC-10；实际11/11 |

## 9. 复审记录

2026-10-21 stage:new | PLAN-817 r2→r3 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-817 r1→r2 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-11 / V-12四子case / SD-06 / T-11；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 10→11，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

2026-10-19 stage:new；PLAN-817 r1；outcome:pass（规划契约预检，非实现验收）；next:work（需用户实施指令；T-02起等D-798）。新增T-01–10/AC-01–10/V-01–11/SD-01–05；drafting 0/10。未实施/编译/跑性能或创建树；来源及预检见[planning-preflight.md](attachments/817/planning-preflight.md)。预算/支持/接口范围改变应显式revision，不以待绑定或模型绿替代完整验收。

2026-10-20 stage:work；PLAN-817 r1；outcome:blocked（T-01 完成 1/10；T-02 起 blocked 于 D-798，非设计阻塞）| worktree `D:/autostack/.wt/lang-817/auto-lang` | branch `plan-817-dev` | base `08ba8ca2b`（=master HEAD）| code_commit `2c38609eb`（experimental/ac-cte-probe/contract-decision.md，doc-only）| dependency_revisions：796/783/788=master 已交付；798=在途 plan-798-dev@072cb4e14（只读观察，未编译）| task_ids：T-01 | evidence：见 §8 T-01 行内收据与 contract-decision.md §1–11；主检出代码路径零 WIP（preflight 已核）| blockers：D-798（798 final review pass+冻结门禁+实际合入+Spec/merge 收据）+ 公共注册短窗口（797/798 合入、814–816 边界交接后）| next:work（T-02：同步新 master，按 §2 复验义务重取 798 实际 API 映射，建 workspace+首例真实执行/冻结/正式 AutoDown）

2026-10-23 stage:work；PLAN-817 r3（contract-decision r1.1）；T-02 完成（2/11）| worktree `D:/autostack/.wt/lang-817/auto-lang` | branch `plan-817-dev` | base `d46e03e8a`（828 归档后 master HEAD）| code_commit `b249767d0`（37 文件 +5697/−9）| dependency_revisions：796/783/788=master 交付；798=master 合入（ABC /2 已复验）；**828=archived（D828 满足）**；797=archived | task_ids：T-02 | evidence：§8 T-02 行内收据；workspace 19/19 绿、正式语料 2/2 绿+错误不绿、auto-case lint ok | blockers：无（T-03 起无外部依赖）| merge-owned 门禁 pending：裸 cargo t + tv + probe 消费面定向（最终 review pass 后冻结提交执行）| next:work（T-03：schedule/consumer 完整化、CT片段检查/Meta参数broker/环与取消；cte_schedule_）。**并行调和记录**：本轮实施期间发现 817 计划文件工作树存在并行规划会话未提交的 r2→r3 排期修订（2026-10-20/21 用户裁定）；本收据将 r3 内容与 T-02 work 收据一并提交（r3 记录见上方 2026-10-21/20 stage:new 两行），其 818/820–826 等他文件 WIP 未触碰、留归其会话。

## 10. 待澄清事项

无待用户重新决定的排期问题。2026-10-21用户已明确优先顺序，保持D828门，不提前绕过；原§9收据中的等待D798/待裁定为历史。D798已交付，D828仍须按实际最终review、冻结门禁、合入、Spec/收据核验。

已有T-01准备保留。D828后先同步已交付基面，刷新contract-decision的API/版本/预算及实际注册owner，记录窗口接受并定向重验；满足T-02后继续原任务。（2026-10-23 work：D828 已核验满足（828 archived）；contract-decision r1.1 已刷新；注册窗口已接受并登记。**818 交接注记**：817 已追加 cte/scalar-query-v1 注册并改 repo 根 exclude+=experimental（修嵌套 workspace 解析 bug）；818 恢复时 rebase 后同步该注册状态并复验其 native recipe 追加。）独立消费层可与其他优先线并行，公共定义/注册必须串行交接，不要求817/818互相全部完成。

原有限profile、CTE/native/增量/AutoDown/P95/内存验收均不缩减。若需改变支持/预算/公共合同，带证据回new；完整语言、复合CTE/native、产品默认、AJ/发布及自举另案。
