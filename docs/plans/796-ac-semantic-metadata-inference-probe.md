---
plan_id: PLAN-796
status: executing
feature_name: AC 语义元信息与类型推导合同及探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
current_step: 5
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
| [x] T-03 | work | T-02 | 新core/src/{infer,query,reflect}.rs及core AST adapter；局部求解/集合负依赖/CT等描述 | infer_core_、meta_query_；AC-01、02 |
| [x] T-04 | work | T-03 | 新probe/src/store.rs及页/依赖/lease消费，零改783/787实现 | sema_cache_、sema_budget_；AC-03 |
| [x] T-05 | work | T-04 | 原始trace/六场景/参考机器与周转/压力报告，首批.case.ad/oracle从T-01持续维护 | sema_turnaround_；AC-04 |
| [ ] T-06 | work | T-05、D-788 | 共享测试协议单写窗口：新probe adapter/Atom数据kind/recipe注册与语义worker，797接入合同明确；不改ac/syntax现有语义 | 正式AutoDown与probe_adapter_、v1/v2兼容；AC-05 |
| [ ] T-07 | work | T-06 | 新verify脚本、docs/reports/plan796/、attachments/796/spec-delta.md；矩阵补齐和依赖/并行审计 | acceptance V-01–09、SD；AC-06；execution_done=7/9 |
| [ ] T-08 | review | T-07 | 同树独立review，冻结commit/依赖/接口快照，检查遗漏/延期 | AC-07；pass→reviewed=8/9，merge pending |
| [ ] T-09 | merge | T-08 pass | 冻结提交最终门禁，沉淀SD/ledger、归档合入、批量回归到期判定、guard清理；输出D-796消费收据 | AC-08；实际完成9/9 |

D-788未满足可以继续独立模型/缓存/性能工作，不能用待绑定.ad冒称AutoDown已完成。T-06不自改788在途协议；797不代修796共享接口。范围/预算/类型支持变化回规划修订，旧证据按影响失效。

## 9. 复审记录

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

来源/边界及起草检查见[planning-preflight.md](attachments/796/planning-preflight.md)；该pass不代表功能或亚秒已达标。

## 10. 待澄清事项

无阻断起草的用户决策。T-01需冻结准确Schema/稳定名义身份策略、有限推导规则/诊断、递归组codec、现有store的合法opaque页接入、probe消息/资源上限与参考负载；只允许在本文范围内选择。CT执行/全量泛型/反射生成能力仍future，不用模型pass授信。

**进行中阻断（work 收据 2026-10-16）**：T-06/T-07 等 D-788（PLAN-788 最终 review pass、冻结提交门禁、实际合入、Spec 沉淀/merge 收据齐全）。只读在途原型不解锁。解除后恢复 work 于同树 D:/autostack/.wt/lang-796/auto-lang。
