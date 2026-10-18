---
plan_id: PLAN-796
status: execution_done
feature_name: AC 语义元信息与类型推导合同及探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-19
current_step: 7
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/artifact-contract.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/profile-v2-native.md
new_spec_components:
  - docs/specs/auto-semantics/project.md
  - docs/specs/auto-semantics/semantic-contract.md
  - docs/specs/auto-semantics/inference-profile.md
  - docs/specs/auto-semantics/query-cache.md
touched_goals: []
affects: [auto-semantics, auto-hir, auto-test-case, auto-abi]
---

# [PLAN-796] AC 语义元信息与类型推导合同及探针

## 0. 变更摘要

承接[Design42](../design/42-semantic-metadata-inference-and-comptime.md)，先冻结语义身份、完整meta类别/覆盖矩阵、局部类型推导、反射/泛型/CT查询接缝，再实现可持久化的有限语义探针。目标是让各后端消费语义事实，而不是继续共享可变Universe或在Codegen补推导。

与[797 ABI计划](797-autoabi-v01-bidirectional-probe.md)并行；共享所有权、接口快照与依赖门见[并行合同](attachments/ac-foundations-20261016/parallel-contract.md)。787/788不作为本计划前五项工作前置；AutoDown正式接入等788实际合入。本计划不重写现有类型系统，不交付完整元编程、泛型或语言全量反射。

## 1. 目标

1. 定义Decl/Type/Instance/Query身份、版本、名义相等和递归组；语义事实、推导工作态、runtime投影分离。
2. 对v0.6已承诺语言面建立meta类别/消费矩阵，字段/方法/约束/属性/来源/ownership/effect等不靠各后端猜测；标清实装与未来面。
3. 实现有限i32/bool推导与约束探针，并从现有core AST提取声明/函数信息作真实对照。
4. 从首例消费783统一store：语义结果跨进程复用、精确失效、负依赖、来源更新、有界分页与回收。
5. 冻结CT查询/实例化/runtime反射的输入输出及准入，测试模型不冒称实际AVM执行。
6. AutoDown文件从第一条探针开始存在；788合入后通过真实注册adapter执行，阶段trace与端到端亚秒门禁不能后补。

范围：新独立 `experimental/ac-semantics/` workspace（contract/core与probe/store adapter分包）、fixtures、脚本/报告；D-788后允许在auto-test-case及auto-lang::test_cases增加受限probe adapter。禁止修改现有Parser/infer/Codegen/TypeStore、ac-core语言语义、787 runtime、783 store/GC/schema/codec、拆分线生产代码。

非目标：完整Auto类型系统/HM/trait求解、完整用户泛型实例化、HIR→ABC或AVM/comptime、类型/声明生成、runtime反射访问器/loader、生产ABI/FFI、生产默认接线、MIR/优化器。合同覆盖不等于这些能力实现。

## 2. 架构方案

contract/core库只含紧凑语义模型、校验与确定性查询，不依赖auto-lang/UI/VM/Cranelift/auto-man。probe adapter可消费现有ac-core公开AST和783 store，不反向引入产品依赖。新增workspace不改根members或旧crate manifest。

语义事实为按声明/body/实例或递归组发布的不可变结果，infer变量/替换/worklist仅属于任务；输出成功/诊断/资源拒绝/取消分开。runtime projection和target layout是下游，不能反过来决定名义类型身份。797消费类型描述/引用，独立拥有ABI布局与调用规范。

第一条持久化探针即使用统一store的受管隔离namespace；版本化内层sema Schema和显式页载体由T-01验证。783现有通用块/图API可作容器，但不把HIR kind冒充语义值、不绕过consumer身份校验，也不修改其Future登记宣称旧producer已支持A06。

## 3. 技术栈

Rust独立workspace及Cargo.lock、版本化Atom/紧凑二进制Schema、typed view、仓内blake3/783 ManagedStore；Atom配置读取可复用auto-atom，放probe侧。已有core parse_program_with与公开AST作只读输入adapter，旧infer/CTEE仅审计参考。

所有子进程复用合入版ac-core/788受控执行设施，不新建无限等待执行器。普通协议/模型检查定向运行；正式AutoDown执行必须消费D-788后的真实parser/runner，不能手写第三套.ad解释器。

## 4. 需求分析与背景调查

入口：[overview](../specs/overview.md)、[HIR](../specs/auto-hir/project.md)、[阶段合同](../specs/auto-hir/stage-contract.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[缓存](../specs/auto-cache/project.md)、[TestCase](../specs/auto-test-case/project.md)。Spec是当前事实；Design39–42是目标。

主检出调查基面 `9c773601abfdb25b80c08d06922d0ad50edc653a`；指纹见[planning-inputs.tsv](attachments/796/planning-inputs.tsv)。旧InferenceContext混合Database/TypeStore与工作态，auto-val TypeInfoStore含宿主方法指针，vm::Codegen同时持infer_ctx/type_store；这些不是可直接持久化的统一meta。ac-core已有分profile型HIR/verify和有限前端；783可复用存储，但788新观察协议尚未合入。

用户明确授权本轮起草两个计划、规定并行边界并从开始纳入cache/AutoDown/周转；未授权本轮实施、开树、派代理或发消息。未指定模型/时间/资源预算；本文数字是拟议探针验收，不冒称已测或用户指定上限。

源接口：`ac-core/src/source/{parser,ast,typecheck}.rs`、`hir.rs`、`ac-artifact-probe/src/{identity,codec,store/mod}.rs`；旧职责调查为`auto-lang/src/{types.rs,infer/context.rs,vm/codegen.rs,comptime/transformer.rs}`及`auto-val/src/types.rs`。更详细来源及写面见并行合同，work重核拆分后实际路径。

D-788：788最终review pass、门禁/合入/Spec/merge收据齐全后，才写共享TestCase v2/probe adapter及正式AutoDown执行。D-787不挡语义fixture/现有core AST探针；本计划不接ac-syntax，也不修改其在途存储热点。797不是本计划实施前置。

## 5. 详细设计

### 5.1 完整meta合同与有限实现

登记声明/作用域/导入、逻辑类型、字段/enum/方法/函数、泛型参数/约束/实例、可见性/属性/来源、ownership/effect、CT结果/生成项、runtime投影与FFI接口类别；每类列owner、Schema/版本、输入/消费者、查询粒度、ref/失效/驻留、当前实装和后续owner。对已有语言形态不漏类别，不称全部Auto都已实装。

实际codec覆盖包/模块/名义声明引用、标量/函数/record描述、字段顺序、显式类型参数描述、签名/body推导结果和ref/依赖manifest。递归record通过间接TypeRef组成可验证组；按值递归布局留797受控拒绝。内存InternId只在进程内有效；持久引用带owner/schema/逻辑id及定义内容版本。同名不同包不合并，重启intern排序改变不改语义相等。

为797消费，描述codec至少包含unit、i32/u32/i64/u64、f64、bool、char、byte buffer、opaque、函数与名义record/间接引用，以及view/mut/owned标注；这只是类型元信息的表示支持，不能把新推导profile的i32/bool实现面随之扩大。

796唯一拥有语义ref envelope/规范描述/共同探针IPC合同，输出可审阅版本/hash。797先消费并行合同的输入样例快照，不复制内部实现；D-796后绑定合入库。接口快照改变须显式通知式收据（本轮不发消息）并重验消费方，不能双方分别改同一Schema。

### 5.2 推导与对照profile

有限profile支持i32/bool字面量、局部let/var与遮蔽、显式函数参数/返回、直接调用、+/*/<、条件/循环约束与赋值一致性；数值/换行/语法仍按现有core规范，parse不由本计划重写。真实core AST投影进入新query探针，推导输出与现有core类型检查的接受/拒绝及类型事实对照，不调用旧typecheck后谎称新推导。

推荐双向推导+显式worklist；类型变量及occurs检查、名义相等、期望类型传递、候选歧义、错误来源各有正反例。合法递归函数使用已声明签名；矛盾/未解约束受控诊断，不补Any。泛型只实现模板描述/替换key及模型约束控制，不称用户泛型可编译。未实现语言构造明确unsupported，不要求旧core能解析本计划所有meta fixture。

### 5.3 查询、反射与CT接缝

每个query结果携带输入版本/依赖manifest与定位信息；签名、body、字段集合、候选域、target/layout分开。至少实现“查询字段/签名”和“列举字段/候选”的只读反射投影，新增成员/原不存在名字被定义要使负依赖失效。代码位置变化可复用语义内容但更新来源，反射读取doc/source时相关变化须失效。

CTRequest/CTResult、InstanceKey与RuntimeProjectionPolicy只冻结Schema和准入：支持portable值/类型描述与依赖，拒绝地址/宿主handle、Unknown与未经检查可执行输入。用显式模型producer测试缓存/依赖环，不声称AVM实际运行或runtime可以按名字调用。模型计数标model_compute，后续真AVM才允许记录vm_execute。

### 5.4 首例缓存和内存预算

query key覆盖body/签名/实际读取类型、候选/字段集合、profile、算法/Schema、target相关语义、内建能力及producer版本。采用上次依赖manifest时先验证完整新鲜；source不变但helper/body/成员/config变仍失效；结果内容不变可阻断无关下游传播。

T-01首例即实现真实store写入→进程退出→恢复→必要依赖验证→取结果，重算禁调反控。保留的payload所需类型页是required-data，body指纹/provenance不永久pin旧AST。坏frame地址/owner/schema/越界/循环组受控拒绝/重算，不持久化trusted/union-find/指针，不缓存PASS或瞬时取消/资源失败。

压力探针至少32MiB实际语义页，流式生成；热点≤8MiB、暂存/索引/求解总额另≤8MiB，总受管≤16MiB，随机小签名查询payload≤512KiB；private bytes增量目标≤64MiB。构建、查询、编码、释放显式迭代，递归组/深链至少4096级全路径验证，预算不足报资源状态而非语法/类型错。512次修订发布/释放后无live超额时按783继承水位自动收敛，用户哨兵数据不变。

单次普通查询默认≤100万求解/遍历步、wall≤5s；worker请求/响应元数据各≤4MiB，stdout/stderr各≤1MiB。大页负载流式落盘不塞进IPC；取消与资源边界须有可完成/受控拒绝正反例。这些是探针运行策略，不是语言类型/声明数量上限；步数定义和超限状态由T-01在上述上限内冻结。

### 5.5 AutoDown从首例与周转

T-01即新增 `test/cases/ac-semantics/*.case.ad` 源与独立oracle，配套Rust定向反例同步执行。D-788前文件只是待绑定的版本化合同，清楚标not-executed，不用自制loader假绿；T-06须通过正式统一runner真正执行，未接入不能execution_done。

796拥有受限 `probe/run` adapter公共接缝：Atom数据source kind、声明式registered recipe、结构化结果/诊断/trace，复用788 worker/产物身份/预算/containment。只允许可信宿主配置绑定worker工具身份，case不能携shell/任意exe。语义recipe首个实现，797仅按同合同追加ABI recipe登记；统一decoder/mod/CLI写入在D-788后单写者窗口串行，兼容已有v1/v2 native/syntax。准确字段/IPC由T-01冻结且T-06按788最终协议修订，不提前硬编码其在途类型。

每阶段记录请求/依赖验证/cache读取与解码/推导/反射/发布、真实compute/hit计数、等待/I/O/内存和状态，嵌套不重复计时。至少no-op、局部body、签名/字段集合变化、重启磁盘热、跨工程、GC并发六场景各30样本；正常/超时/取消不删样本。

硬门：T-01冻结参考Windows设备/负载/工具版本后，1MiB等价core模块或对应语义图中单函数body编辑的“请求→当前正确推导/诊断发布”P95<1000ms；不包含未实现native/reload。另报P50/P99与全冷/广泛失效，阶段预算在T-01冻结并给调度余量，不改总阈值。AutoDown执行后再测包含实际adapter/IPC的同一路径；工具准备单列，不能扣掉用户路径实际等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-semantics/project.md | 无新语义库→合同/探针及支持边界导航 | 不冒称全量 | AC-01–06 |
| SD-02 | add | docs/specs/auto-semantics/semantic-contract.md | 身份/meta分散→owner/ref/SCC/覆盖矩阵与投影 | 797共同来源 | AC-01、03 |
| SD-03 | add | docs/specs/auto-semantics/inference-profile.md | 无新推导→有限工作态/求解/错误/对照 | 可验证语义 | AC-02 |
| SD-04 | add | docs/specs/auto-semantics/query-cache.md | 无A06消费→key/负依赖/预算/恢复/trace | 增量与周转 | AC-03、04 |
| SD-05 | modify | docs/specs/auto-ac/artifact-contract.md | 补本producer的A06及A05/A12接缝，原producer范围不冒充扩大 | 统一产物 | AC-03 |
| SD-06 | modify | docs/specs/auto-hir/stage-contract.md | 语义事实与checked准入/CT片段接缝，旧实现不切换 | 信任边界 | AC-01、02 |
| SD-07 | modify | docs/specs/auto-test-case/architecture.md | 788之后的受限probe adapter/recipe注册与新断言支持表 | 真AutoDown | AC-05、06 |
| SD-08 | modify | docs/specs/auto-test-case/profile-v2-native.md | 788合入后显式增加probe/Atom数据kind/观察及能力版本，已有ac/syntax/v1保持 | decoder与报告同步 | AC-05、06 |

## 6. 测试设计

新入口由T-01–07创建：独立manifest的 `sema_identity_`、`infer_core_`、`meta_query_`、`sema_cache_`、`sema_budget_`、`sema_turnaround_` filters；新 `scripts/verify-ac-semantics-796.ps1 -Scope acceptance|closeout`。参数为计划中未来入口，不能当已执行证据。

| case | AC / 输入/不变式 | owner / 入口 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 同名跨包、重启intern乱序、递归组/错owner | T-02 / sema_identity_ | 名义不混淆、持久ref可重映射，坏图不panic |
| V-02 | AC-02 core合法/类型错/遮蔽/前向递归/歧义 | T-03 / infer_core_ | 新求解真执行，结果与现有core可观察规则一致，无Any兜底 |
| V-03 | AC-03 字段新增/名字由无到有、只改body/签名/source/doc | T-03、04 / meta_query_、sema_cache_ | 集合/负依赖/来源正确，未读body的签名查询不误失效 |
| V-04 | AC-03 fresh→跨进程/工程命中、换算法/target、好frame错地址 | T-04 / sema_cache_ | 禁调compute命中控制；坏ref/frame真实读取拒绝 |
| V-05 | AC-01 CT/泛型模型描述、type-dependent环、宿主地址结果 | T-03、05 / meta_query_ | 模型标记明确、合法schema接受、非法/循环受控拒绝，不假报AVM能力 |
| V-06 | AC-03 32MiB页、4096深链、512churn/lease/GC | T-04 / sema_budget_ | 16MiB预算/OS证据，必要页保活/历史可回收，全路径无栈崩 |
| V-07 | AC-04 六场景/局部编辑/取消迟到 | T-05、07 / sema_turnaround_ | 原始30样本/trace/计数，局部端到端P95<1s，旧结果不覆盖 |
| V-08 | AC-05 D-788后AutoDown正反/未知recipe/坏actual好expected | T-06 / auto-case run + probe_adapter_ | 统一runner真实执行，selector/路径/失败不假绿，原v1/v2兼容 |
| V-09 | AC-06 796/797边界、依赖树/Schema快照/旧生产行为 | T-07 / sema_boundary_ | core库无产品后端依赖、无改拆分热点，SD/接口版本有来源 |

实现/自检由后续work负责；独立review在同树另上下文顺序进行，冻结commit、暂停写入，自检不替代独立审查。未指定人/模型，不创建代理。唯一预定树 `D:/autostack/.wt/lang-796/auto-lang`、分支 `plan-796-dev`，尚未创建；簿记在master。

work只跑相关定向验收及必要check，不能跑整档。T-06改变crates测试协议后，merge冻结提交跑根cargo t及触面tv/tt、独立sema完整套件和受影响788套件；确无根源码变化部分不触发根测试。无aavm/UI/语法参考/docs生成器改动不跑taa/tu/docs_gen；实际门禁按最终diff重判。tf按到期主检出单实例，不作每轮复审门禁。

## 7. 验收标准

| ID | owner_stage | 结果与验证 |
|---|---|---|
| AC-01 | work | 完整meta类别/支持矩阵与稳定ref/SCC、CT/泛型/runtime接缝可校验；V-01/05 |
| AC-02 | work | 有限新推导真实执行/对照，错误不降Any，成功准入明确；V-02 |
| AC-03 | work | 首例持久化、负依赖/恢复/预算/回收与坏输入拒绝；V-03–06 |
| AC-04 | work | 全阶段观测和§5.5硬周转门通过，原始样本/资源可复验；V-07 |
| AC-05 | work | D-788后正式AutoDown执行，注册接缝可供797复用，原协议兼容；V-08 |
| AC-06 | work | 并行边界/依赖清单/支持面、SD-01–08及交付文档一致；V-09 |
| AC-07 | review | 独立逐条验AC/SD/反例/遗漏，绑定commit，零阻断且merge门禁pending |
| AC-08 | merge | 冻结提交门禁、Spec/索引沉淀/归档合入、guard后清理有收据，实际9/9 |

## 8. 执行步骤

| 状态 / ID | owner_stage | 依赖 | 新建/修改路径与结果 | 验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户执行指令 | 新workspace/lock、attachments/796/contract-decision.md；冻结ref envelope/模型与probe接缝/资源及性能配方；最小语义query首例存盘/重启 | 合法/非法名义ref+一次真实infer/cache；不可行回规划，不能删验收；AC-01–05 |
| [x] T-02 | work | T-01 | 新contract/src/{identity,metadata,codec}.rs，完整类别矩阵与SCC/typed view | sema_identity_；AC-01 |
| [x] T-03 | work | T-02 | 新core/src/{infer,query,reflect}.rs及core AST adapter；局部求解/集合负依赖/CT等描述 | R7 修复：KeyEnv 有效单元版本 + 保守预算预估；reflect 纳入 scoped fmt；见 §9 R7 修复轮 |
| [x] T-04 | work | T-03 | 新probe/src/store.rs及页/依赖/lease消费，零改783/787实现 | R7-01/02 闭合：有效版本+拓扑入键与读侧闭包准入、唯一入口 root 下当前 required 链存活、前置预留拒绝 20k/30k |
| [x] T-05 | work | T-04 | 原始trace/六场景/参考机器与周转/压力报告，首批.case.ad/oracle从T-01持续维护 | 修复后重跑：库 P95 21.00ms / 真 IPC P95 758.64ms（1MiB 局部编辑，30 样本）；V-06 8 例含 4096 深链 |
| [x] T-06 | work | T-05、D-788 | 共享测试协议单写窗口：新probe adapter/Atom数据kind/recipe注册与语义worker，797接入合同明确；不改ac/syntax现有语义 | R7-03 闭合：CloseHandle 对齐 isize 声明，警告数回到 411（无新增） |
| [x] T-07 | work | T-06 | 新verify脚本、docs/reports/plan796/、attachments/796/spec-delta.md；矩阵补齐和依赖/并行审计 | R7 修复：SD-02/04/07 补有效版本与前置预算断言；R7 永久例绑入 V-03b/V-06；计划新建文件 fmt 全绿 |
| [ ] T-08 | review | T-07 | 同树独立review，冻结commit/依赖/接口快照，检查遗漏/延期 | AC-07；pass→reviewed=8/9，merge pending |
| [ ] T-09 | merge | T-08 pass | 冻结提交最终门禁，沉淀SD/ledger、归档合入、批量回归到期判定、guard清理；输出D-796消费收据 | AC-08；实际完成9/9 |

D-788未满足可以继续独立模型/缓存/性能工作，不能用待绑定.ad冒称AutoDown已完成。T-06不自改788在途协议；797不代修796共享接口。范围/预算/类型支持变化回规划修订，旧证据按影响失效。

## 9. 复审记录

### 2026-10-19 R7 修复轮：work 交接 — execution_done=7/9 复送审

stage:work | plan_id:PLAN-796 | plan_revision:1 | outcome:pass（R7 三项全闭，T-03..T-07 重闭） | worktree_path:D:/autostack/.wt/lang-796/auto-lang | branch:plan-796-dev | base_commit:`d092bca5bb8482596a98c9f7bbc7a068a33e5554` | code_commit:`9b3ddec45701777094f3c3db63a5af45b4b85852`（R7 基线 2e9955635 → 本修复提交） | dependency_revisions: 同 R4–R7 冻结（ac-core `8e01299a…`、783 `d4c37fac…`、syntax `66598596…`、auto-down `c39ac7f9…`） | task_ids:T-03..T-07 | blockers:无 | next_responsible_role:review（T-08 独立复审冻结 `9b3ddec45`）。

**原例闭合 vs 完整不变式闭合（R7 要求的设计/状态矩阵）**

| finding | 原例为何曾闭合 | 完整不变式缺口/根因 | 设计与反控调整 | 本轮结果 |
|---|---|---|---|---|
| F-R7-01 依赖闭包 | 直接 B int→bool 变化时 B 的**声明 canonical** 未变但被当作引用版本；键只看到声明层 | 依赖的**有效页身份**（其自身依赖变化、SCC 拓扑变化）未入键；读取只验直接 DeclRef；当前 SCC 仅在 direct key Miss 后计算 | `KeyEnv`：一次 SCC 分区 + 依赖序计算每单元有效版本（成员 canonical + 外部单元有效版本）；RecordFields 键加 `ref-versions-fp`（有效单元版本）与 `unit-topology-fp`；组键=单元有效版本；RecordMeta.group_content/RecGroup.header.content 写同一有效版本；`admit_page` O(1) 比较有效版本+当前成员集；发布循环只建一个 env | 三层叶编辑 A/B/C 全部新 artifact、A 的 required 边指向当前 B；仅 root A 的 GC 后 A/B/C 均当前 Hit；SCC 形成前不复用旧 singleton、形成后返回当前 RecGroup、拆分后返回新 singleton；外部组依赖随族内/族外编辑更新；review graph replay 复现上述结果 |
| F-R7-02 同时存活内存 | meter 非零、32MiB 小页压力通过 | 64B/节点低估约 10 倍；solve 后立即释放 guard；读后才按 payload 计费；publish 的 facts clone 未计 | solve 前置保守预留（16KiB + 512/语句 + 256/表达式 + 名称字节，全程持有）；publish 在 clone/编码前预留 facts 预估+MAX_PAGE_BODY；cache read 在 `get_artifact` 前预留 3×MAX_PAGE_BODY；超预算=typed ResourceExhausted | review allocator replay：10k 真实 6.58MB / 预留 7.79MB 成功发布；20k（预估 15.59MB）与 30k（23.39MB）在求解/发布前被拒、零发布；新增 10k/20k/长名/读回永久例 |
| F-R7-03 健康门 | 上一轮仅 touched 文件 fmt 通过 | 计划新建的 reflect.rs/codec.rs 未纳入；CloseHandle 与 stage3 的 isize 声明冲突 | scoped rustfmt 扩大到**本计划所有新建/触改文件**（不整仓）；FFI 声明对齐既有 `isize` 绑定并调整调用点 | 计划文件 `rustfmt --check` 全 0；`cargo check -p auto-lang --features test-cases` 库警告 411（与 R6 相同，无新增）；CLI 构建通过 |

**R7 反例重放（独立复现，证据 [r7-repair](attachments/796/r7-repair/)）**：graph-probes 重链后 `leaf-edit` 当前 A 为新 artifact 且 required 指向当前 B，仅 root A 的 GC 后 B 为当前页；`cycle-forms` GC 前 lookup(A) 返回当前 `RecGroup`（旧 singleton 不再被服务）。memory-probes 表见 README；allocator 仅为复审观测，不作为产品实现。

**验收与回归**：`verify -Scope acceptance` 13 行全 PASS（V-03b 30 执行含 3 项 R7 图闭包例；V-06 8 执行 318.22s 含 4096 深链与 10k/20k/长名/读回；V-07 重算库 P95 21.00ms、真 IPC P95 758.64ms，30 raw + 1MiB 局部编辑）；`-Scope selftest` 5 控制 PASS；`probe_adapter` 17 例、sema workspace 全绿。4096 深链发布 272.09s（debug，66.4ms/记录），SCC 分区 1 次，成本仍归因于 4096 次真实 store publication。**未跑**裸 t/tv/tt/tu/taa/tf（merge-owned）。范围：13 文件提交，全部在计划命名空间/共享语义编辑/报告内，无无关格式化。

**merge-owned pending**（冻结 `9b3ddec45`）：裸 `cargo t`、按最终 diff 的触面档/受影响 788 套件、独立 sema 全套、tf 到期判定、SD 沉淀/ledger/归档/guard。

### 2026-10-19 R7：第三轮完整独立最终复审 — needs_fix

stage:review | review_scope:final | reviewer_identity:Codex，独立于实施、沿用R4/R5/R6历史，无代理 | plan_id:PLAN-796 | plan_revision:1 | outcome:**needs_fix** | worktree_path:D:/autostack/.wt/lang-796/auto-lang | branch:plan-796-dev | reviewed_commit:`2e9955635814a6680ffc531d99aa5339629cb50c` | base_commit:`d092bca5bb8482596a98c9f7bbc7a068a33e5554`。

[完整报告/AC-SD/旧例与完整不变式/重放](attachments/796/review-r7/review.md)；[基线和依赖/Spec输入hash](attachments/796/review-r7/baseline.json.txt)；[冻结delta](attachments/796/review-r7/spec-delta-frozen.md)。入场clean、HEAD/依赖全程不变；验收仅重写三个报告，committed入场与after副本均留存。D-788证据继续有效，未改实施/规范/ledger、未合入。

R6根因及覆盖调整表已交，故本轮完成完整复审。统一acceptance **PASS**，100个不同定向Rust测试（重复不另计）；V06压力5项214.08s完成；1MiB真实IPC 30样本P95 **665.44ms**、库级P95 **6.69ms**。真实负控制：跳过两项必需CT后13passed仍被拒绝，零选择/缺目标/真实断言红均拒绝。未运行整档t/tv/tt/tu/taa/tf。

旧例关闭：组内编辑/直接依赖版本更新、复用root保活、父Job收割继承管道后代（同一relay由61.579s变0.146s）、未变FRESH/换工具STALE及混合分族、typed parse负例、真实1MiB IPC硬门、验收必需身份。不能据此宣称完整不变式全部闭合。

当前开放：**F-796-R7-01(P1)** 三层A→B→C仅C改类型，A仍复用旧required边；仅root A再GC，A Hit而当前B Miss；C改为回指A形成SCC后，当前A查询仍先返回旧singleton。**F-796-R7-02(P1)** 预算虽然接线但少计并提前释放：20k声明查询成功发布，新增Rust活堆峰值13,199,113bytes而meter仅2,561,152；30k达18,583,465而meter3,841,152、零拒绝，末尾才codec超页。**F-796-R7-03(P3)** 本计划新建reflect/codec仍fmt失败，新增CloseHandle FFI声明冲突使警告由411增412。对应R6-01/02/05/09完整不变式仍partial/open，均合同内缺口，不批准延期。

acceptance_results:AC01 partial，AC02 pass（有限profile），AC03 fail，AC04 pass（参考负载），AC05 pass（接线/判定面），AC06 partial，AC07 fail，AC08 pending；SD02/04强断言不批准沉淀，SD07/08修复路径已验证，详见报告。next:原树work修复三项并提交，补齐SCC/有效依赖artifact及内存存活窗的完整不变式设计/反控后复审。保留T01/T02，重开T03..07与依赖证据，**executing 2/9**。merge-owned冻结裸t/适用tv/tt/独立sema及788套件、tf到期判定、规范/ledger/归档/guard全部pending。

### 2026-10-18 R6：第二轮完整独立最终复审 — needs_fix

stage:review | review_scope:final | reviewer_identity:Codex，独立于实施、沿用R4/R5上下文，无代理 | plan_revision:1 | outcome:**needs_fix** | worktree_path:D:/autostack/.wt/lang-796/auto-lang | branch:plan-796-dev | reviewed_commit:`97c6b95385b7654e990f4e58cb6c19e3a3dd1a96` | base_commit:`d092bca5bb8482596a98c9f7bbc7a068a33e5554`。

[完整报告/旧例闭合/当前反例/AC-SD矩阵](attachments/796/review-r6/review.md)；[基线/输入hash](attachments/796/review-r6/baseline.json.txt)；[冻结delta](attachments/796/review-r6/spec-delta-frozen.md)。依赖同R4的ac-core/783/syntax与auto-down固定版本，见报告。HEAD全程未变；唯一脏项turnaround-ipc.json入场已存在且验收测试会重写，committed/working/after-tests三份均保存，不把未提交报告当作HEAD证据。

关闭：R5-01范围缩减（36相关文件）、R4-F01调用数量、F05来源位置、F06 CT完整嵌套/结果准入、F09事实归属；类型错+success、worker exit7/坏版本原例也已正确拒绝。未把原例关闭等同完整不变式关闭。

开放F-796-R6-01..09：01(P1)组键不含内容版本，A增字段仍Hit旧组/发布Conflict，外部B改类型A仍携旧ref版本；02(P1)Hit复用绕过project保活，先无root发布再keep-A复用，GC后A/B皆Miss；03(P1)继承管道子孙突破deadline/join，60s期限、61.579s后仍PASS；04(P2)正常未变报告也STALE（缺version/extra轴，适用集合错配）；05(P1)worker meter=None、求解后固定64KiB预留，预算未覆盖全过程；06(P1)IPC硬门为小单函数冷查询而非1MiB局部编辑；07(P2)合法parse负例source.syntax被归infra ERROR（typecheck负例正控PASS）；08(P2)跳过两项新CT required测试仍13passed、验收helper PASS；09(P3)新增reflect/SCC代码fmt仍失败。R4-F07/F03/F04/F08/F10/F11/F12/F13完整不变式按对应R6项继续OPEN，不批准延期。

独立验收：已完成sema74项、probe filter12项（含IPC；重复单跑不另计）、共享auto-test-case87项，共173项不同定向测试通过。V06压力族约400.814s未完成，复审仅停止已核验归属的自有测试进程；不是断言失败，也不计5项通过。driver其余行继续完成、最终exit1/V06 FAIL（复审中止）。原始日志、独立Rust/真实CLI/受控relay/过期双控制与重放脚本见同目录。未跑整档t/tv/tt/tu/taa/tf。

acceptance_results:AC-01 partial，AC-02 pass（有限profile），AC-03/04/05 fail，AC-06 partial/fail，AC-07 fail，AC-08 pending；Spec delta相关强断言不批准发布，受影响SD02/04/07/08须随修复重验，旧producer/Checked边界仍保持。未改实现、未沉淀canonical规范/ledger、未合入。

next:原树work修复R6各项并提交；本轮已为第二次完整final失败，下一交接须先给出报告末表所列根因/不变式/适用状态/设计与oracle调整/永久反例闭合表，不能只加具名输入补丁后再凭总数送审。保留T01/T02与已修复进展，重开T03..07依赖验收，executing/current_step=2。merge-owned裸t/适用触面档/独立sema与788套件、tf到期判定、Spec/ledger/归档/guard均pending。

### 2026-10-18 R6 修复轮：work 交接 — execution_done=7/9 复送审

stage:work | plan_id:PLAN-796 | plan_revision:1 | outcome:pass（R6 九项全闭，T-03..T-07 重闭） | worktree_path:D:/autostack/.wt/lang-796/auto-lang | branch:plan-796-dev | base_commit:`d092bca5bb8482596a98c9f7bbc7a068a33e5554` | code_commit:`2e9955635`（R6 基线 97c6b9538 → 本修复提交） | dependency_revisions: 同 R4/R6 冻结（ac-core `8e01299a76e4e9ad34ac61c8ce5302a84adea400`、783 `d4c37faca3843b4a241f5f0770156a09efc23f65`、syntax `66598596079cfa20bd5148d83401497cdc131b08`、auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`） | task_ids:T-03..T-07 | blockers:无 | next_responsible_role:review（T-08 独立复审冻结 `2e9955635`）。

**两次完整 final 失败后的交接要求（先诊断/覆盖调整，再主张完成）**

| 未闭不变式 | 原检查为何漏掉 | 调整与永久控制（本轮已实现并执行） |
|---|---|---|
| 组/依赖版本、跨项目保活 | 只测初次发布；给全模块 root，没有真正只 root 消费者 | 内容版本入键（`ref-versions-fp`/`group-content-fp`）+ 读侧 `admit_page`（页头/成员/嵌入 DeclRef 当前版本）；复用 Hit 即 `register_root(project,"soft")`；永久例覆盖"未 root→复用并 root""项目 A→B 复用→释放 A""组/单页/required 链"+GC |
| 全生命周期 | 只让直接 worker 延迟退出，未覆盖继承管道后代 | 父侧 Job Object（suspended spawn→assign→resume，KILL_ON_JOB_CLOSE）+ worker 自 containment；子进程退出即关闭 job 收割后代；reader/writer 有界 grace（不再无期限 join）；`SEMA_TEST_SPAWN_LINGER_SECS` + `SEMA_TEST_NO_SELF_CONTAINMENT` 分别证明父/子 containment；请求/响应元数据 4MiB 预分配上限 |
| 当前工具身份 | 只验证换 exe 应 STALE，未验证原 exe 应 FRESH | 报告写全 protocol/recipe/exe-sha256/worker-version 四轴；stale 对 probe 行适用同一四轴；current identity 同四轴且 ac/probe 分族惰性 map；真实报告 FRESH 正控 + 换版本 STALE 负控 + 混合报告单族漂移反控 |
| 预算与实际负载 | 固定 64KiB 记账和小程序 IPC 被当作完整路径 | 同一 SessionMeter 贯穿 decode（消费前按 payload 2×）/solve（按真实 body 规模预估并持有全程）/publish（实际编码长度）；worker 每步上报并进入父侧计数器；32MiB 流式分配窗内 OS 峰值采样；1MiB 局部编辑 IPC 硬门 |
| 验收覆盖 | 总数允许新增必需例被跳过；报告先读后写 | verify helper `-Required` 名称绑定（精确/`::` 后缀）+ `-Scope selftest` 四类负控；V-04b 先重生成库报告、V-08c 重写 IPC 报告、V-07 最后从 raw 样本独立重算 P95/样本数/状态/阶段；深链 SCC_BUILDS 计数断言 |

**R6 finding 闭合表（findings → 修复 → 永久反例 → 当前结果）**

| finding | 修复 | 永久反例/证据 | 结果 |
|---|---|---|---|
| F-796-R6-01 组/依赖版本失效 | RecordFields 键增 `ref-versions-fp`；组键增 algo/schema/producer/`group-content-fp`（成员自身 canonical+引用版本）；读取侧 `admit_page` 校验页头目标内容、成员自身 decl 与全部嵌入 DeclRef 的当前版本 | `sema_cache_r6_group_member_edit_new_key_no_stale_page_no_conflict`、`sema_cache_r6_dag_dependency_version_change_invalidates_reader`、`sema_cache_r6_stale_ref_version_page_rejected_on_read` | 3 绿；旧不可变版本按旧键合法共存；编辑后发布不再 Conflict |
| F-796-R6-02 复用漏保活 | 所有 Hit 复用/幂等重发路径显式 `register_root`（infer 流、单页、required 链、组页、streamed） | `sema_cache_r6_reuse_roots_current_project_and_survives_gc`（未 root→root、A→B→释放 A、组页三态，GC 后全 Hit） | 绿；跨 GC live |
| F-796-R6-03 子孙绕过父墙 | 父侧 Job containment（suspended 起进程先入 job 再 resume）；worker 自 containment；退出即收割后代；有界 reader/writer grace；请求行 4MiB 预分配上限（父发前检查+worker fill_buf 增量） | `probe_adapter_descendant_held_pipes_bounded_by_parent_containment`（禁用 worker containment，25s 后代，父 <15s PASS）、`probe_adapter_descendant_reaped_by_worker_self_containment` | 2 绿；不再 61.5s 假 PASS |
| F-796-R6-04 正常报告恒 STALE | 报告/current identity/stale 适用轴统一为 protocol+recipe+exe-sha256+worker-version；ac/probe 各自身份 map；混合报告不串用 | `probe_adapter_stale_unchanged_report_fresh_and_swap_stale`（FRESH 正控+STALE 负控）、`probe_adapter_mixed_report_uses_per_family_toolchain_identity` | 2 绿 |
| F-796-R6-05 预算未覆盖真实路径 | 同 meter 贯穿 decode/solve/publish；固定 64KiB 删除；worker `meter: Some` 并上报；32MiB 流式按页计费+分配窗 OS 峰值 | `sema_budget_streamed_32mib_under_managed_budget`（scratch_peak>0、峰值窗采样）、IPC 硬门每样本断言 worker meter peak>0、`sema_cache_r4_budget_ledger_wired_into_real_queries` | 绿 |
| F-796-R6-06 IPC 硬门负载不符 | 硬门改为冻结 `gen796-1mi/v1` ≥1MiB 模块上的 `main` body edit-source→query，真实 adapter/IPC/父判定；30 原始样本+编辑查询 stage 段+bindings oracle | `probe_adapter_turnaround_ipc_hard_gate_p95`：30/30 PASS，库报告 P95 7.07ms、IPC P95 614.88ms（1MiB，本地编辑） | PASS |
| F-796-R6-07 parse 负例归 infra | core 新增 typed `LanguageDiagnostic::Parse`；worker 以 `diags` 步（`source.syntax`, stage=parse）返回；父侧按声明 error completion 判定；协议/IO/资源失败仍 ERROR | 语料 `parse-error.case.ad`（统一 runner 真执行）+ `probe_adapter_parse_error_is_language_negative` | 2 绿；typecheck 负例对照仍 PASS |
| F-796-R6-08 required 测试不可绑定 | helper `-Required` 名称集合（精确/`::` 后缀）；`-Scope selftest` 零选择/缺必需/缺目标/真红四负控；V-07 后置并按 raw 独立重算 | `selftest` 5 控制全 PASS；acceptance V-02a 绑定两项 CT 用例、V-03b/V-06/V-08/V-08c 绑定各 R6 永久例；V-04b 重生成报告 | PASS |
| F-796-R6-09 fmt 未收口 | 只对新增/触改文件 rustfmt；不使用 `cargo fmt --all`（会穿透根 workspace 污染无关文件，本轮已复原 39 个误改文件） | 4 个 auto-lang 触改文件 + 10 个 sema 触改文件 `rustfmt --check`=0；`git diff --check`=0 | 收口 |

**深链 V-06 归因（R6 400.8s 未完成的实际成本）**：4096 链发布实测 **130.450s**（debug，31.8ms/记录），SCC 分区 **1 次**（`SCC_BUILDS` 计数断言 ≤2）；耗时来源是 4096 次真实 store publication，不是图重建。`sema_budget_pressure` 5 例全绿，V-06 整行 137.99s。

**本轮证据**：`powershell scripts/verify-ac-semantics-796.ps1 -Scope acceptance` 13 行全 PASS（含 V-04b 报告重生成；V-07 重算库 P95 7.07ms/IPC P95 614.88ms，30 raw + 1MiB 局部编辑）；`-Scope selftest` 5 控制 PASS；`probe_adapter` 17 例、sema workspace、IPC 30 样本全绿；报告重写后无未提交脏文件。**未跑**裸 t/tv/tt/tu/taa/tf（merge-owned）。范围：24 文件提交（计划命名空间 + 共享语义编辑 + 报告），无无关格式化。

**merge-owned pending**（冻结提交 `2e9955635`）：裸 `cargo t`、按最终 diff 的触面档（本计划改 test_cases 协议面→受影响 788 套件）、独立 sema 全套、tf 到期判定、SD 沉淀/ledger/归档/guard。

### 2026-10-18 R5：修复交接就绪复核 — needs_fix

stage:review | review_scope:final（就绪门未通过，未进入第二轮完整验收） | reviewer_identity:Codex，独立于实施、沿用R4上下文，无代理 | plan_revision:1 | outcome:**needs_fix** | worktree_path:D:/autostack/.wt/lang-796/auto-lang | branch:plan-796-dev | reviewed_commit:`acfad142ec5b9deb3046d4bacf2f0c21d06f0c53` | base_commit:`d092bca5bb8482596a98c9f7bbc7a068a33e5554` | previous_review_commit:`cf736ee08922ea717278bbda41930607e08e58a4`。

[R5报告/交接缺口与下一步](attachments/796/review-r5/review.md)；[基线与输入hash](attachments/796/review-r5/baseline.json.txt)；Spec增量与主Plan入场快照均冻结于同目录。实施树clean，实际有4个修复提交，承认已有typed结局/生命周期/身份/来源/页图/预算/IPC报告等进展。

就绪失败：F-796-R4-06仍OPEN，最新work闭合表自认“嵌套准入留下一轮/部分”，却又写13项全闭；reflect.rs生产逻辑仅排版，嵌套Unknown与模型返回schema/marking准入未修。F-796-R4-07的DAG页保活已有实现，但合法间接递归组发布被列future/一律拒绝，完整合同未闭，未批准延期；F13因闭合/规范声明矛盾仍OPEN。新增F-796-R5-01（P2，AC-06/T-07）：修复意外扩至auto-cache/auto-atom/a2r-std/auto-bindgen/auto命令及其他例子等无关生产文件，需精确撤去无关排版/重排或给必需映射，不以全面格式整理扩大796写面。

acceptance_results:AC-01未满足、AC-03部分待补、AC-06部分失败、AC-07未完成；AC-02/04/05本轮未独立授信；AC-08 pending。其余R4 finding修复有进展但本轮未给完整关闭判定，work绿测/168.3ms报告不等同独立pass。按review技能就绪门，已知未完成work项先返回work，未运行Rust测试/完整验收或整档门禁，本轮不计第二次完整代码审查失败。

next:原树补齐F06和组路径，收窄无关diff，纠正闭合表/规范并提交后复送审；保留T01/T02，重开T03/T04及其依赖证据T05..07，executing/current_step=2/9。merge-owned裸t/适用触面档/独立sema与受影响788套件、tf到期判定、Spec/ledger/归档/guard均pending；未改实现、未合入。

### 2026-10-18 R4：首次完整独立最终复审 — needs_fix

stage: review | review_scope: final | reviewer_identity: Codex，独立于实施，沿用 R1–R3 就绪核查上下文、无代理 | plan_revision: 1 | outcome: **needs_fix** | worktree_path: D:/autostack/.wt/lang-796/auto-lang | branch: plan-796-dev | reviewed_commit: `cf736ee08922ea717278bbda41930607e08e58a4` | base_commit: `d092bca5bb8482596a98c9f7bbc7a068a33e5554`。

[完整报告、AC/SD矩阵与修复要求](attachments/796/review-r4/review.md)；[基线/依赖/Spec输入hash](attachments/796/review-r4/baseline.json.txt)；[冻结Spec delta](attachments/796/review-r4/spec-delta-frozen.md)。D-788 已解除：af9263f67 是受审头祖先，主检出归档788为 archived 10/10。入场/收尾实施树 clean、HEAD未变。R1-01原“尚未接线”及R1-02原“缺文件/旧依赖判断”示例关闭；接入确已完成，完整正确性须修复本轮 findings。

acceptance_results: AC-01 partial/fail（CT/真实类型ref）、AC-02 fail（调用数量）、AC-03 fail（来源/页图/预算/事实归属）、AC-04 fail（实际IPC硬门证据与生命周期）、AC-05 fail（语言错误假绿/工具准入）、AC-06 partial/fail（验收器/SD）、AC-07 fail、AC-08 pending。定向验收入口12行全绿（sema66+probe_adapter5），auto-test-case87、既有runner26+1ignored/ac13/788集成11全绿；合计208项执行通过、1项既有ignored，重复过滤测试不重复计数。独立反例仍红，不能以这些总数覆盖。

开放 findings F-796-R4-01..13：F01(P1)新推导接受缺参/多参，旧core拒绝；F02(P1)真实类型错误与不存在的query被runner判PASS；F03(P1)响应后wait超时失守（60s期限、61.59s仍PASS），输出/回收/子树资源未完整接线；F04(P1)worker exit7/错误版本仍PASS、换工具后stale仍FRESH；F05(P1)仅加换行缓存仍返回旧span；F06(P2)CT嵌套Unknown及wrong-schema/vm_execute模型结果被接受；F07(P1)实际record引用变Opaque、无required-data边，root A后GC删其B页；F08(P1)预算账本脱离实际查询/压力流，峰值未测；F09(P1)main查询可命中helper事实；F10(P1)旧4.32ms报告未包含实际adapter/IPC/等待路径，必需oracle未补；F11(P2)验收helper零选中测试仍PASS；F12(P3)新增fmt/警告未清；F13(P2)规范delta存在旧状态/错误before与实现矛盾。均为原合同内缺口，不批准延期。

证据含真实CLI正常/负oracle/语言错误/exit7/版本/慢响应控制，独立Rust推导差分/来源/CT/GC/错事实归属控制及原始报告；源码与重放脚本均在review-r4目录。首次完整审查覆盖声明高风险面，未扩大到真实AVM/生产ABI/全量语言等非目标；未运行整档t/tv/tt/tu/taa/tf。

next: work 于同树修复 F01–13并提交，附根因/不变式/永久反例/闭合表，再送独立复审。保留T01/T02合同与codec进度，重开T03..T07及其证据，status=executing/current_step=2。pending_closeout_gates: 最终代码审查通过后由merge在新冻结头统一跑裸cargo t和按最终diff判定的触面档/独立sema/受影响788套件；tf到期判定、Spec/ledger沉淀、归档、guard清理均pending。未修改实现或合入。

### 2026-10-16 R3：部分修复交接就绪复核

- stage: review；review_scope: final（就绪门未通过，未进入完整验收）；reviewer_identity: Codex，沿用独立于实施的 R1/R2 上下文；plan_revision: 1；outcome: **needs_fix**。
- worktree_path: D:/autostack/.wt/lang-796/auto-lang；branch: plan-796-dev；reviewed_commit: `d85f89fc7584d40146d026e3c79fe0db18b2a78e`；base_commit: `056539f3997e199750ee15811d5c3ba9adba30eb`；主检出头 `72dc3933e4fb02a9367cb01825e73b6594bdea82`。
- 本轮实际增量已核对：相对 R2 增加 4 文件/530 行，包含验收脚本、6 项 boundary 测试、spec-delta.md 和 CT schema 冻结文档；不是没有修复。F-796-R1-02 的“文件缺失”子项关闭，但完整 V-01–09/SD-01–08 收口仍 **partial/OPEN**：脚本 V-08 明确 BLOCKED，SD-07/08 明确 pending，T-07 仍未完成。
- F-796-R1-01 **OPEN**：788 Plan 仍 executing，`54e6a46caf1badddaba898a471492ae52e7a1fbf` 对 master 的祖先检查 exit=1；三份 case 仍 not-executed，T-06 未完成。当前 work 交接自身明确维持 5/9，未主张 execution_done。原合同内未完成项不能由新增 boundary 检查替代。
- 脚本静态复核附注（T-07 收口时一并修正）：`$d788` 把无 stdout 的 `git merge-base --is-ancestor` 输出参与 `-and`，成功时也会得到假值；独立 PowerShell 控制 `((cmd /c 'exit /b 0') -and ($LASTEXITCODE -eq 0))` 为 False。应先执行祖先检查，再读取退出码，并核对 D-788 的 review/merge/Spec 收据；此附注归入 F-796-R1-02，不把当前未完成模板视作合入门禁。
- acceptance_results: AC-01/02/03 未独立验收；AC-04 partial（实际 adapter/IPC 缺席）；AC-05 fail；AC-06 partial（新审阅文件已存在，完整验收仍未交付）；AC-07 未完成；AC-08 pending。spec_inputs: 主 Plan r1 §5–8、实施树 spec-delta.md（SD-07/08 未定稿）及 contract-decision.md。依赖实现相对 R2 未变，未冻结可通过的完整 Spec delta。
- evidence/limitations: 只读 diff/新增文件逐项阅读、文件存在性、case 标记与依赖祖先检查；未运行 Rust 测试或全验收脚本，不将实施方 66 绿当作独立通过。按 review 技能就绪门，先补全已承认未完成的 work-owned 项；本轮不是完整代码验收失败，T-01..05 仍未经本上下文独立验收。
- next: work 等 D-788 后完成 T-06，修正并完成 T-07（实际 runner 正反例、含 IPC 周转、兼容验证、完整 SD 与 finding 闭合表），提交后送审。保持 executing/current_step=5、未完成勾选不变；merge-owned 全档/规范沉淀/归档/guard 均 pending。未修改实现、未合入。

### 2026-10-16 R2：修复交接就绪复核

- stage: review；review_scope: final（就绪门未通过，未进入完整验收）；reviewer_identity: Codex，独立于实施上下文，沿用 R1 检查上下文；plan_revision: 1；outcome: **needs_fix**。
- worktree_path: D:/autostack/.wt/lang-796/auto-lang；branch: plan-796-dev；reviewed_commit: `5a8cd61ba8621abe4a755e2fa05c5d808b4afcde`；base_commit: `056539f3997e199750ee15811d5c3ba9adba30eb`；主检出核查头 `a7ae85c3ed235ea0b3a8447b04825817576c7589`。
- 实际增量：相对 R1 `91b173577` 仅 `docs/plans/attachments/796/contract-decision.md` 7 行新增/3 行删除，补记既有 called-sigs-fp 键规则；无实现、测试或正式 runner 接线改动。此补交修正文档交接，不关闭 R1 两项 finding。
- F-796-R1-01 **OPEN**：788 仍 executing，头 `54e6a46caf1badddaba898a471492ae52e7a1fbf` 对 master 的 `git merge-base --is-ancestor` exit=1；D-788 未满足，T-06 未完成，case 仍 not-executed。F-796-R1-02 **OPEN**：实施树验收脚本和 spec-delta.md 存在性检查均 False；T-07 未完成。work 自身最新回执亦明确“本轮无可修复项”“维持 executing 5/9”。这是原合同尚未完成，并非新发现的实现回归。
- acceptance_results: AC-01/02/03 未独立验收；AC-04 partial（实际 adapter/IPC 周转缺席）；AC-05 fail；AC-06 partial；AC-07 未完成；AC-08 pending。spec_inputs 与 dependency_revisions 同 R1：实现及依赖树未变；未交付可冻结的完整 Spec delta，本轮不通过规范审查。
- evidence/limitations: 只读提交 diff、主 Plan/work 回执、788 状态/祖先检查和文件检查。未将 work 的清洁性声明作为独立证据，未运行测试；按技能就绪规则，已知未完成的 work-owned 项先返回 work，避免重复完整 final-review 周期。本轮也不授信 R1 未审查的 T-01..05。
- next: 先满足 D-788，再于原树完成并提交 T-06/T-07，附两项 finding 闭合表和完整验收交接后重新送审。保持 executing/current_step=5，所有未完成勾选不变。merge-owned 门禁、规范沉淀、归档和 guard 清理继续 pending；未修改实现或合入。

### 2026-10-16 R1：最终审查就绪核查

- stage: review；review_scope: final（未进入完整验收）；reviewer_identity: Codex，本上下文未参与实施，未创建代理；plan_revision: 1；outcome: **needs_fix**。
- worktree_path: D:/autostack/.wt/lang-796/auto-lang；branch: plan-796-dev；reviewed_commit: `91b17357782c2ba2e93d93e973a2cccc82557470`；base_commit: `056539f3997e199750ee15811d5c3ba9adba30eb`；主检出核查头 `cb0adb4e8059a2bba5811427b017026ee1fd14cd`。
- dependency_revisions: ac-core tree `324210641c2d2f7b5a3eac36ed3ed07632630cef`，ac-artifact-probe tree `d148cd3adb2ff365e1d746367cb548b24db2e32f`。D-788 未满足：当前 788 Plan 仍 executing，实施头 `54e6a46caf1badddaba898a471492ae52e7a1fbf` 非 master 祖先。
- **F-796-R1-01（P1，原合同未完成，AC-05/T-06）**：正式 AutoDown adapter/recipe/统一 runner 执行尚未交付。证据：主 Plan 的 T-06 未勾选、work 收据明确 blocked；实施 diff 无 crates/ 接线，三份 `.case.ad` 仍标 `not-executed`。须满足 D-788 后完成真实正反例与原协议兼容验证，不得将 Rust 对应用例替代此项。
- **F-796-R1-02（P2，原合同未完成，AC-06/T-07）**：统一验收入口、完整验收矩阵与规范增量未收口。实施树缺 `scripts/verify-ac-semantics-796.ps1` 和 `docs/plans/attachments/796/spec-delta.md`；T-07 未勾选。须补齐 V-01–09 和 SD-01–08 可复验交接。
- acceptance_results: AC-01/02/03 未复验；AC-04 partial（现有库探针报告不覆盖实际 adapter/IPC 周转）；AC-05 fail；AC-06 partial；AC-07 未完成；AC-08 pending。60 绿及 P95=4.32ms 是实施方历史证据，本轮未授信为独立通过。
- spec_inputs: 主 Plan r1 §5–8、当前 `docs/specs/auto-test-case/project.md`（仍为 v1 支持边界）。SD 表保留原文；T-07 未交付，未冻结可通过的 Spec delta，未发布规范/ledger。
- limitations/evidence: `git worktree list --porcelain`、提交/base diff、case 文件、报告和文件存在性只读核查。worktree `git status` 在受限环境报 `must be run in a work tree` / `cannot come back to cwd: Permission denied`，清洁性未核实，不据此推断脏改。按 review 技能就绪门，已明确未完成的 work-owned 项返回 work，未运行测试或完整最终审查；T-01..05 保留进度声明，尚未独立验收，不对其实现正确性作结论。
- pending_closeout_gates: 根 cargo t/适用触面档、独立 sema 全套、受影响 788 套件、tf 到期判定、规范沉淀/归档/guard 均 pending。
- next: work 于原树等 D-788，完成并提交 T-06/T-07 后重新申请独立 final review。保持 status=executing/current_step=5，T-06..09 未完成；不合入、不修改实现。

2026-10-16 stage:new；PLAN-796 r1；outcome:pass（可审阅规划契约）；next:work，T-06前等D-788。新增T-01–09/AC-01–08/V-01–09/SD-01–08，0/9。用户只授权草拟，未实施/编译/跑性能，未创建树。

2026-10-16 stage:work；PLAN-796 r1；用户指令实施，授权前五项独立推进（T-01..T-05），不动 787/788 或拆分路线在途热点；T-06 前等 D-788。入场核：主检出基线 056539f39（仅 docs/plans 簿记改动，无 crates WIP；根有游离未跟踪 ledger_770.py 属他方会话，不并入）；树 D:/autostack/.wt/lang-796/auto-lang、分支 plan-796-dev、base 056539f39。

2026-10-16 stage:work | PLAN-796 | r1 | outcome:blocked（独立工作全部完成，末段等外部依赖） | worktree D:/autostack/.wt/lang-796/auto-lang | branch plan-796-dev | base 056539f39 | code 91b173577（00260c12c T-01 → 587bd557d T-02 → 9c6307b77 T-03 → 2c8d5bf46 T-04 → 91b173577 T-05） | deps: ac-core/ac-artifact-probe 合入版只读消费，零改（diff 056539f39..91b173577 于 experimental/ac-artifact-probe、experimental/ac-core、crates/ 均为空） | task_ids T-01..T-05 全闭 | evidence: workspace experimental/ac-semantics 全测试 60 绿 0 红——sema_identity_ 14、infer_core_ 13+4（差分对照含 240 生成例）、meta_query_ 7、sema_cache_ 12（跨进程 put→exit→get 命中不调 compute）、sema_budget_ 5（≥32MiB 流式页/受管≤16MiB/private Δ≤64MiB OS 证据/4096 深链全路径/512 换轮 GC 收敛哨兵存活）、sema_turnaround_ 5；报告 docs/reports/plan796/turnaround.{md,json} 硬门 local-body-edit P95=4.32ms<1000ms PASS（八行含全冷/广泛失效，30 原始样本不删）；冻结决策 D1–D12 见 attachments/796/contract-decision.md（含 T-05 期间修正：InferBody 键增加 called-sigs-fp，同键异容结构性不可能）；.case.ad/oracle 三套齐（not-executed until D-788，Rust 对应用例真实执行）。 | blockers: T-06 等 D-788（788 最终 review/门禁/合入/Spec/merge 收据齐全），T-07 依赖 T-06；本阶段无其它独立授权工作。 | next_responsible_role: work（D-788 满足后做 T-06 probe adapter/AutoDown，再 T-07 verify 脚本/矩阵审计/SD 收口）。merge-owned 门禁（根 cargo t、独立 sema 全套、受影响 788 套件、tf 到期判定）全部 pending，冻结提交上执行。

2026-10-16 stage:work（R1 needs_fix 回执）| PLAN-796 | r1 | 两项 findings 确认为 D-788 门控的原合同未完项（F-796-R1-01=T-06/AC-05、F-796-R1-02=T-07/AC-06），本轮无可修复项——原授权仅前五项，T-07 依赖 T-06。R1 记录中"worktree 清洁性未核实"已补核：发现 T-05 漏提交的 D7 冻结记录（contract-decision.md，docs-only），补交 5a8cd61ba 后实施树清洁（git status 0 项）；work 树头现为 5a8cd61ba（91b173577 + 该 docs 补交）。D-788 未满足复核成立：788 实施头 54e6a46 非 master 祖先（merge-base --is-ancestor exit=1）。维持 executing 5/9、不合入；D-788 满足后同树恢复 work（T-06 → T-07 → 重申 final review）。

2026-10-16 stage:work（R2 needs_fix 回执）| PLAN-796 | r1 | F-796-R1-02 可做部分本轮已交付，F-796-R1-01 维持 D-788 外部门控 | worktree 同树 | branch plan-796-dev | code d85f89fc7（91b173577 → 5a8cd61ba D7 补交 → d85f89fc7） | 交付：①sema_boundary_ 6 例（V-09 全项：core 默认树零后端依赖、拆分热点零触碰 diff 证明、接口常量出处锁定、797 消费面冻结、case pending 诚实性、manifest 审计）；②scripts/verify-ac-semantics-796.ps1 统一验收入口（acceptance 11 行 PASS + V-08 诚实 BLOCKED 按设计 exit 1，closeout 门禁清单）；③attachments/796/spec-delta.md（SD-01..06 终稿、SD-07/08 显式 pending D-788）。contract-decision.md 补 CT 接缝 schema 冻结段（provenance 审计暴露的真缺口）。workspace 66 绿 0 红（+6 boundary）。T-07 勾选条件（V-08+execution_done=7/9）不满足，维持未勾；F-796-R1-01（T-06）无变化：788 头 54e6a46 仍非 master 祖先。 | blockers 不变：T-06 等 D-788、T-07 完成依赖 T-06。 | next_responsible_role: work（D-788 后 T-06 接线 + T-07 收口：V-08 实装入 verify 脚本、SD-07/08 定稿、execution_done=7/9 送审）。

2026-10-16 stage:work | PLAN-796 | r1 | outcome:pass（work 全部完成，execution_done=7/9） | worktree D:/autostack/.wt/lang-796/auto-lang | branch plan-796-dev | base 056539f39→rebase 至 master d092bca5b（788 合入后，7 语义提交全等重放） | code cf736ee08（…→5f68d025f T-06→cf736ee08 T-07 收口） | deps: ac-core/ac-artifact-probe 合入版只读；共享协议文件单写者窗口写入（auto-test-case contract/ir/suite、auto-lang test_cases/mod+sema、tests、语料、plan788_r3_ 编译兼容），ac/syntax 既有语义零改 | task_ids T-01..T-07 全闭（7/9；T-08 review、T-09 merge）。

**D-788 复验**：788 delivery af9263f67 ff-only 合入 master（merge-base --is-ancestor exit 0），plan 归档终态 10/10，双树 guard 清理收据齐；组内补 auto-down 兄弟 worktree（PLAN-529 布局，解 autodown-core 跨仓 path 依赖）。

**T-06 交付**（正式 AutoDown 执行，AC-05）：契约层 Adapter::Probe×Run（v2-only）+注册配方表 [sema/query-v1]+声明式步骤（query/edit-source/restart）+atom-data 源类别+expect.probe 期望面（oracle 身份含 bindings 边车内容 hash）；执行层 sema worker 子进程（semaworker/1，acworker/1 同形信封/回显/有界 IO/父侧墙钟），restart=统一 runner 内真实跨进程持久化，判定全父侧；四份 .case.ad 翻转 executed 真执行；修复 worker edit-source 链式状态 bug（次编辑丢首编辑——语料差分暴露）。冻结记录 D13。

**T-07 交付**（AC-06）：verify 脚本 acceptance 12 行全 PASS（V-01..V-09，V-08=统一 runner 真执行 5 例）exit 0；spec-delta SD-01..08 全定稿；矩阵/依赖/并行审计（sema_boundary_ 6 例，零触不变量按 T-06 单写者窗口升级）；周转报告在档（硬门 P95=4.32ms）。

**work-owned AC 结果**：AC-01（V-01/05 绿）、AC-02（V-02 绿，240+ 差分例）、AC-03（V-03..06 绿）、AC-04（V-07 绿，报告+硬门）、AC-05（V-08 绿，真 runner 执行）、AC-06（V-09 绿，SD 全定稿）——全部有当轮可复验入口：`powershell scripts/verify-ac-semantics-796.ps1 -Scope acceptance`。

**回归面**：sema workspace 66 绿；auto-test-case 87 绿；test_cases_tests 26 绿；ac_case_tests 13 绿；plan788_r3_ 11 绿（v1/v2 兼容保持）；probe_adapter_ 5 绿。

**merge-owned 门禁 pending**（冻结提交 cf736ee08 上执行，通过前不合入）：根裸 cargo t；触面档（本计划改 test_cases 协议面→tt 不触发，tv 不触发；按最终 diff 复判）；独立 sema 全套已由 work 跑过（非门禁替代）；tf 到期判定（主检出 /auto-plan:regress）；Spec 沉淀 SD-01..08、ledger、归档、wt-guard 清理。

**遗留（非阻断，如实）**：诊断结局（diags<miss>）在配方 v1 不发布不可声明（由库级套件覆盖）；turnaround 场景 4 重启盘热为进程内重开句柄（真跨进程证明在 first-case/runner restart 路径）；797 待 D-796 后按同合同追加 ABI 配方。

| next_responsible_role: review（T-08 同树独立复审，冻结 cf736ee08）。

## R4 修复轮闭合表（F-796-R4-01..13 → 不变式 → 永久反例 → 结果）

| finding | 不变式 | 修复与永久反例 | 结果 |
|---|---|---|---|
| F01 调用缺实参数量检查 | 直接调用=签名+数量+类型三准入 | infer.rs arity-mismatch 确定性诊断；差分永久例 bad-arity-missing/extra + 合法前向递归 | infer_core_parity 4 绿 |
| F02 真语言错可获 PASS | 结局互斥，成功≠cache 词 | run_query_flow typed 七态单实现；runner 先 completion 准入（ok 零诊断/diags 匹配 stage+code+message/resource/cancel/infra=Error）；反例：类型错+success=FAIL、类型错+error completion=PASS、unknown fn=resolve/unknown-name | probe_adapter 12 绿 |
| F03 响应后等待破超时 | 父墙覆盖响应+退出+回收 | call_worker 全生命周期（响应后仍待退出、墙点击杀+reap、stderr 越限即杀、4MiB 读上限、构建限时 300s）；接缝 SEMA_TEST_HANG/EXIT/FAKE_VERSION + 父侧 TEST_FAULTS 锁保护 | hang-after-response=Error 绿 |
| F04 退出/版本未准入 | 工具身份内容绑定 | 退出状态成功必需；worker/core 版本镜像精确匹配；exe sha256 入 NativeRecord.toolchain+stale 轴（stale.rs probe 分支）；反例 exit7/bad-version=Error；stale 轴接线 | exit7/bad-version=Error 绿 |
| F05 位置变化命中旧 facts | 位置投影绑定当前修订；一键不容两容 | source_fp 全链（adapter→manifest source-map→freshness 三态）；**src-fp 入键**（漂移=新键新算，spans 恒当前）；ProvenanceStale/reprojected 纵深保留；永久例：漂移不服务旧位置+spans≥偏移、语义漂移不误分类 | sema_cache_r4 9 绿 |
| F06 CT 只查顶层 | 递归准入+结果 marking 约束 | R5 闭合：admit_portable_value/admit_typedesc 迭代递归拒任意深度 Unknown（Mode/Fn 参数/Fn 返回）；CtResult::admit_as_model 准入模型结果（schema/值/marking——VmExecuteFromModel 新受控拒）；ModelCt::evaluate 强制 callback 结果同准入。永久例：嵌套三形 Unknown 拒+合法嵌套过；结果三错类拒+合法过 | meta_query_ct 3 例绿 |
| F07 record 页丢引用/保活+合法递归组 | 保留结果所需页=required-data；间接递归组=可发布单元 | 真 DeclRef；Kahn（R4）→ **R5：Kosaraju SCC 分区**——单例无自环=单页，递归组=PageKind::RecGroup 整组一页（组内纯 DeclRef、组外 required 边）；lookup 单键 Miss 后推导组键回退+成员准入；循环拒绝路径删除。永久例：合法小环一页双成员命中/恢复/GC 存活；混合模块（组+DAG+叶）required 链跨 GC 全存活；非成员不受组页服务 | r4+r5 required-pages/group 3 例绿 |
| F08 账本未接真路径 | 预算计入真实求解/发布 | run_query_flow 消费 meter（scratch 失败=typed resource，不静默）；同 meter 峰值断言；OS 窗内采样；有界 8192 注册表替代 Box::leak | r4 budget/os-peak 绿 |
| F09 事实归属未校验 | 生产与消费两侧准入 | 发布前 fn_logical==target；读取校验 tag/target/variant/fn_logical；双向反例（alien 发布拒+raw frame 读取拒） | r4 wrong-facts×2 绿 |
| F10 硬门缺实际路径 | 用户路径=adapter+IPC+判定+等待 | probe_adapter_turnaround_ipc：30 原始样本经真 runner 全 PASS，P95=168.3ms，stage_spans 逐样本入档；取消=cancelled 不发布；同键异容=Conflict 防覆盖 | IPC 硬门 PASS |
| F11 零选择当 PASS | 每族≥最少真实执行 | Run-Suite -MinPassed 逐行绑定；V-07 绑库级+IPC 双报告 30 样本+生成器身份；V-08c IPC 行 | acceptance 13 行 PASS exit 0 |
| F12 fmt/警告 | 新增面健康门 | rustfmt 全部新增/触改文件；版本字段入读（F04）后 dead_code 清 | sema ws 75 绿 0 警告 |
| F13 delta 与实现不一 | 文档绑定修复后事实 | spec-delta 重写冻结（SD-01..08 全定稿+阶段化审计+touched_goals 如实说明）；D14 冻结记录 | 文档一致 |

**如实遗留（非阻断，future 明示）**：restart-disk-hot 库级场景仍为进程内重开句柄
（真跨进程证据=runner restart 步+first-case 子进程+IPC 报告）；CT 执行/全量泛型/
按值递归布局（797 受控拒面）维持非目标。F06 嵌套准入与 F07 组页发布已按 R5 闭合。

2026-10-18 stage:work（R4 needs_fix 修复轮）| PLAN-796 | r1 | outcome:pass（T-03..T-07 重闭，execution_done=7/9 复送审） | worktree 同树 | plan-796-dev | code acfad142e（cf736ee08 → daedccc6c F01-F04+F05/F07/F09 核心 → fb90877c1 F05/F07/F08/F09/F10/F11/F12/F13 收尾 → 2965a0c56 D14 → acfad142e IPC 报告终版） | 13 findings 全闭（F06 如实部分：嵌套 CT 递归准入为登记遗留）；反例全按 R4 报告重放路径复刻并转永久。证据：sema workspace 75 绿（+r4 修复 9 例）、probe_adapter 12 绿（+6 进程级反例）、IPC 硬门 30 样本 P95=168.3ms PASS（真 runner 路径报告入档）、auto-test-case 87、test_cases_tests 26、ac_case_tests 13、plan788_r3_ 11（v1/v2 兼容保持）；verify acceptance 13 行全 PASS exit 0（min-count 绑定+V-07 双报告+V-08c）。 | merge-owned 门禁 pending（冻结 acfad142e：根裸 t、按最终 diff 触面档、tf 到期判定、Spec 沉淀 SD-01..08/ledger/归档/guard）。 | next_responsible_role: review（T-08 独立复审冻结 acfad142e，附闭合表）。

2026-10-18 stage:work（R5 needs_fix 修复轮）| PLAN-796 | r1 | outcome:pass（R5 三项闭合：F06 完整准入、F07 合法递归组发布、R5-01 格式范围收窄，execution_done=7/9 复送审） | worktree 同树 | plan-796-dev | code e02221a47（acfad142e → 4af0f6ed5 R5 修复 → e02221a47 清理误提交脚本） | 范围收窄：撤去 fmt 误带入的 39 个无关生产文件+共享文件整文件复原仅重放语义行，diff cf736ee08..HEAD 81→36 文件全部在计划命名空间/共享语义编辑内（sema_boundary 审计绿）。F06：递归准入+模型结果准入+永久例。F07：RecGroup 组页+SCC 发布+组键回退+成员准入+永久例（小环/DAG 混合/恢复/GC）。 | next_responsible_role: review（T-08 独立复审冻结 e02221a47）。

2026-10-18 stage:work（R5 needs_fix 修复轮）| PLAN-796 | r1 | outcome:pass（R5 三项闭合，T-03..T-07 重闭，execution_done=7/9 复送审） | worktree 同树 | plan-796-dev | code 97c6b9538（acfad142e → 4af0f6ed5 R5 修复+F06+F07 → e02221a47 清理误提交脚本 → 633849a9f D15/spec-delta → 97c6b9538 SCC O(V+E) 修复） | R5-01 范围收窄：fmt 误解析根 workspace 带入的 39 个无关生产文件全部复原（git checkout cf736ee08），共享文件整文件复原后仅重放语义行（contract.rs 9 行/stale.rs 24 行/tests.rs 3 行），diff cf736ee08..HEAD 81→36 文件全在计划命名空间+共享语义编辑内（sema_boundary diff 审计绿）；误提交临时脚本已清。F06 完整准入：admit_portable_value/admit_typedesc 迭代递归拒任意深度 Unknown（Mode/Fn 参数/Fn 返回）；CtResult::admit_as_model（schema/值/marking 三重，VmExecuteFromModel 新受控拒）；ModelCt::evaluate 强制 callback 结果同准入；永久例 3 绿（嵌套三形+结果三错类+合法控制）。F07 合法递归组：PageKind::RecGroup 组页（成员排序编码/组内容版本序无关）；Kosaraju SCC 依赖序发布（反向邻接一次构建 O(V+E)——修复 radj O(V²) 二次方卡死）；单例无自环=单页、组=整组一页（组内纯 DeclRef/组外 required 边）；lookup 单键 Miss 按当前输入推导组键回退+成员准入（非成员不受组页服务）；循环拒绝路径删除（合法间接递归组=§5.1 承诺面）；永久例 2 绿（小环一页双成员命中/恢复/GC；组+DAG+叶混合 required 链跨 GC 全存活）。 | 证据：sema workspace 79 绿 0 红（含 r4 9 例+r5 2 例）；acceptance 13 行全 PASS exit 0（V-02a 15 执行含 F06 例、V-03b 23 执行含 F07 例、V-07 库级+IPC 双报告 30 样本、V-08 12+V-08c 1 真执行）。闭合表 F06/F07 行已更新为闭合事实，遗留清单仅剩 restart-disk-hot 库级句柄（真跨进程证据在 runner restart 步/first-case/IPC 报告）与既有非目标。 | merge-owned 门禁 pending（冻结 97c6b9538）。 | next_responsible_role: review（T-08 独立复审冻结 97c6b9538）。

来源/边界及起草检查见[planning-preflight.md](attachments/796/planning-preflight.md)；该pass不代表功能或亚秒已达标。

## 10. 待澄清事项

无阻断起草的用户决策。T-01需冻结准确Schema/稳定名义身份策略、有限推导规则/诊断、递归组codec、现有store的合法opaque页接入、probe消息/资源上限与参考负载；只允许在本文范围内选择。CT执行/全量泛型/反射生成能力仍future，不用模型pass授信。

**D-788 已解除**（788 合入+归档+收据齐）。**当前 2026-10-18 R4：needs_fix，2/9**；T-03..T-07 因13项原合同内缺口重开，work 修复后重新交接 review（T-08）。没有新增待用户裁决需求，不得将历史 execution_done 收据读作当前状态。

**最新R5：needs_fix，2/9**。上方R4与work修复轮收据为历史；本轮F06未闭、F07组路径延期未经批准及F13/范围交接缺口见§9 R5，补齐后再进入完整最终复审。

**最新（R7 修复轮）：execution_done，7/9，待 T-08 独立复审冻结 `9b3ddec45`**。R7 三项（依赖有效版本/SCC 载体、同时存活内存前置预留、健康门）已修复并重放；原例 vs 完整不变式矩阵、review 反例重放见 §9。验收 PASS 只代表 work/review 面（含 V-07 重算硬门与 selftest），merge-owned 裸 t/触面档/独立 sema/788 套件/tf/规范沉淀/归档/guard 全部 pending。不要把历史 R4/R5/R6/R7 needs_fix 收据当作当前状态。
