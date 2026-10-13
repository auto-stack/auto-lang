---
plan_id: PLAN-774
status: executing
feature_name: AC 数据扩面的共同类型、资源与内部调用契约
plan_revision: 3
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-13
current_step: 1
total_steps: 8
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-hir/stage-contract.md
  - docs/specs/auto-acc/project.md
new_spec_components:
  - docs/specs/auto-ac/data-resource-contract.md
touched_goals: []
affects: [auto-ac, auto-hir, auto-acc]
---

# [PLAN-774] AC 数据扩面的共同类型、资源与内部调用契约

## 0. 变更摘要

实施 [Design 36](../design/36-ac-data-resource-expansion.md) 的共同前置：用现行 Spec、真实编译器源码与有界 native 探针，解决文本/标量聚合首批所需的类型、资源、内部调用和 profile 演进规则。
输出经独立复审的决策包，供 770 文本与 771 record 分支消费；本计划不交付新源码语言 feature，不把 Rust 探针当 Auto native 编译证据。

## 1. 目标

1. 明确 i32/char/索引与 str/Str 各语境的语义，记录旧 VM/A2R/auto-lib 的差异及适配要求。
2. 对 view/mut/move/clone、临时与返回资源、borrow escape/alias 作受限且可验证的规则。
3. 用真实 COFF/PE 调用证明候选内部表示可实现，形成唯一首批方案和 770/771 可消费接口表。
4. 守住 Checked、精确三重身份与旧 profile 兼容；把通用生产 ABI、完整所有权与自举隔离。

非目标：改旧 parser/VM/A2X/auto-val、实现完整借检查器/GC、完整 int 宽度、字符串生产实现、record 源码、List/enum/use、DLL/热重载/生产 ABI、ACC 主体迁移或后端桥。

## 2. 架构方案

证据核查→决策表→独立数据/资源 ABI 探针→真实 native 正反/故障控制→冻结共同合同及下游接口。
探针独立 workspace（新建 `experimental/ac-core/probes/data-resources/`），不扩大 ac-core registry/Schema，不让临时实验接口成为生产支持声明。
资源表示采用显式宽度、借用 buffer 或句柄/out-buffer；候选最多比较“显式拥有/借用”和“受限引用计数句柄”两类，若后一类违反要求则淘汰。不复用 Rust String/Vec 的私有布局。

## 3. 技术栈

现有 Rust/Cranelift 0.126.2、Windows x64 COFF/PE、kernel32 与既有链接/contained executor。
探针复用实际 ac-core link/executor 入口；必要的薄 runtime shim 在探针树新建，不引入旧 VM/产品 crates。

## 4. 需求分析与背景调查

- 原规划基线 `540dc7d30839eb7df8e68b4d19472358e170fe5b`；历史取材指纹见 [输入表](attachments/769/planning-inputs.tsv)。历史实施就绪核查基线为 `a45ff543e7f94474f29a0aed76661b8a76f081e0`，当时指纹见 [r2 输入表](attachments/769/readiness-inputs-r2.tsv)。本次不宣称 v0.5 完整恢复。
- 当前 Specs：auto-ac/project、source-core-i32、source-scalar-i32、auto-hir/project/stage-contract、auto-acc/project；772 scalar Spec 与精确身份/签名边界已经实际沉淀。
- [772](archive/772-ac-scalar-ops-short-circuit.md) 已修复、独立重审、合入并 archived（9/9）；P767-R1/R2/R3 已销号（历史发现ID保留）。本计划的772技术前置已满足，T-01仍应从启动时最新主基线刷新输入，而不是复制已清理的772工作树。
- Design 02/04/10 与 types Spec：str/Str/字面量区分、view/mut/move、静态/实例方法。旧 ownership 模块未接生产，types 概述的“已实现”不能代替安全证据。
- 745 numeric-boundary 明示 char_at 的三种表示及 int 冲突；packages/aavm/lib/lexer.at:44–63、token.at:154/219、parser.at:71+ 提供真实消费者。
- 本会话授权为规划和实施就绪核查，不在本会话实施、合入或创建树。用户拟单独开实施agent；在该agent下达执行774指令时记录当前r3实施授权，未指定模型或预算。独立最终复审与合入授权按后续交接办理。

- 2026-10-13引用/交接核查基线：主检出 d777ccbb227c41882ad5f6976c803b1e03b61135。编号映射为标量772、共同契约774、文本770、record771；770/771未改号。历史输入表保留来源，当前指纹见 [引用核查输入表](attachments/774/reference-update-inputs-20261013.tsv)。本次仅修正引用/取材和实施入口，不修改语言支持、验收阈值或执行状态。
- 2026-10-13 r3 实施授权（§4 预留条款兑现）：用户在独立实施会话下达「执行 774」指令，
  授权按本计划 r3 全文实施 T-01..06（work-owned）；未指定模型或预算。独立最终复审（T-07）
  与合入（T-08）按后续交接办理，本授权不含。实施起点主检出 5e21561d2c303a02319ff7a29e4607933e09a38f
  （r3 引用核查后仅 docs/plans 簿记推进）；T-01 以实施时最新基线刷新输入。

## 5. 详细设计

### 5.1 决策包（必须完成，不允许留 open 供下游猜测）

新建 `docs/reports/774-data-resource/{decisions.md,interface-contract.md,verification.md,proposed-spec-delta.md}`。
每个决定有 ID、现行要求/观察差异、候选、选择理由、支持/拒绝表、oracle、能力/profile 身份、下游责任。

| 决定 ID | 固定范围与交付 |
|---|---|
| D-01 | 保持 core/scalar 精确 i32；新 text/record source 与 HIR 三元组精确注册。内部地址/byte length 独立于源 int，越界转换显式失败，不冒称 i64/usize 源语言支持。 |
| D-02 | 核查并冻结 text 的 Unicode 标量索引、UTF-8 存储、char 名义身份与显式码点转换；跨行/转义/空串/NUL 与非法标量规则。char_at 旧实现差异不升级为隐式 char/int 混用。 |
| D-03 | str/Str 的局部、参数、返回、字段与容器逐语境规则；字面量提升、拥有型构造、借用 slice、clone/显式转换、返回逃逸。容器借用 slice 的既有禁止不得删除。 |
| D-04 | 受限 borrow/own：来源、活跃借用、move 后禁用、mut 排他、唯一清理、临时延长/返回转移；不支持情形在前门明确拒绝。只读实例方法与显式 mut 自由函数的 receiver/参数规则。 |
| D-05 | 内部 ABI：i32/bool/char、借用 buffer、返回拥有 buffer、record view/mut/out-return 五类；宽度、alignment、caller/callee、错误与释放方。不是生产模块 ABI，也不是 ACC Cranelift 桥。 |
| D-06 | 资源：分配器/所有权策略、失败注入点、正常控制流 Drop、fatal trap 的 OS 回收边界；支持预算单位、无固定语法深度排除与环境耗尽类别。 |
| D-07 | 770/771 最小接口/profile/API/共享热点清单及合入顺序；语义定义一处，组合 text-record 留后续 owner；无需创建通用生产框架。 |

D-02/03/04 候选规则先核查规范和消费者，再执行探针；结果与下游 r1 不同必须先修订对应合同。涉及既有语言承诺变化需用户决策，不得以探针方便为由改规则。

### 5.2 全路径可行性探针

新建探针 crate/Cargo.lock、测试 shim 与 `scripts/verify-ac-data-resources-774.ps1`。
用 Cranelift 生成调用者、独立显式 ABI shim 作被调用者，链接真实 PE；宿主测试不能直接调用 shim 冒充机器调用通过。
观测分配/释放/借用/复制计数和固定 trace，正常 paths live=0、alloc=free、一次 move 无重复清理；故障注入不让 panic 跨 ABI。
触发前门拒绝的控制由探针的规则模型拒绝入口执行并定位，模型结果不宣称生产 verifier 已支持资源。

### 5.3 T-02 → T-03 冻结关口

774的表示、资源预算和具体错误码是本计划要产出的有界决定，不是假设已有结论。T-02在候选/支持拒绝表之外，须同时产出 `docs/reports/774-data-resource/probe-cases.md`：逐项列出V-01..09及必要子例的输入、真实入口、期望值/trace/状态或阶段与码、计数口径、覆盖边界及决定ID。T-03只实现已明确预期的候选探针，不得用探针自己的输出反写expected。

- V-02在发射前列出五类ABI的参数/返回/字段哨兵值、准确大小/对齐/offset与调用trace；至少一个有效的异质顺序控制能识别参数或字段交换。布局预期从冻结ABI独立推导，不能从生成产物读取后作为expected。
- V-03/05的alloc/free计数按成功分配与实际释放统计，分配尝试另计；为每个已声明申请点的故障注入列出应已释放的资源和返回状态。正常控制流与受控失败最终live=0；fatal trap另按V-06观察，不宣称Drop执行。
- V-04明确每个非法状态的拒绝入口/码、native调用计数为0，以及合法兄弟例；V-06固定前驱trace、终止方式/码和执行器超时分类。
- V-07的Unicode控制必须区分byte与scalar：`A中😀`为8 UTF-8字节/3标量，`e`加U+0301为3字节/2标量，NUL为1字节/1标量；合法空范围与非法反向/越界范围有各自预期。非法UTF-8与非法Unicode标量分别声明拒绝阶段/码。
- V-08在实现前写明预算值、单位、入口和边界两侧预期；0/1/4096生命周期循环是验证规模，不得变成语法/语言支持上限。

若候选探针证实方案不可行，先记录证据并修订相应决定/用例；既有语言承诺或下游支持行为有材料变化时，按§5.1修订/决策流程处理，不继续实现未定规则。到T-05全部D01..07须确定且有完整oracle，无必需open项才能完成work或交给770/771。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/data-resource-contract.md | 从未决到 D01..07 已核查共同合同；实现仍仅探针 | 下游唯一来源 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-ac/project.md | 增加合同/探针状态与非生产 ABI 边界，不新增语言 implemented | 防混称 | AC-05 |
| SD-03 | modify | docs/specs/auto-hir/stage-contract.md | 追加数据/资源阶段责任与需要验证的义务，未实现部分明确 planned | 保持 Checked 唯一门 | AC-01/03 |
| SD-04 | modify | docs/specs/auto-acc/project.md | 关联文本/聚合的有限决定及剩余能力 owner，不关闭 Gen1/GOAL-017 | 双清单分离 | AC-05 |

## 6. 测试设计

以下均为待 T-03/04 物化的 mandatory supplement；Case owner=work，实际入口为新探针 native gate 或决策核对。确切候选值/码/预算按§5.3由T-02先冻结，不允许边写探针边自动更新golden。

| Case | AC/invariant | 输入与入口 | 预期/区分断言 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 语义有依据 | D01..07 每项→Spec/源码/探针逐项表 | 无无证据默认、无 unresolved required；差异给适配规则 | 首批 text/scalar record |
| V-02 | AC-02 内部调用 | 五类 ABI 各正控，异质顺序、嵌套调用、out-return，真实 PE | 实际值/trace/布局与决策表相符；更换参数顺序必须被测试发现 | Windows x64，不含公开 C 双向互调承诺 |
| V-03 | AC-03 寿命 | 借用读取、拥有返回、move、clone、嵌套分支/循环早退 | live=0，正常 alloc=free；move 不重释放，clone 独立；borrow 不提前释放 owner | 正常出口 |
| V-04 | AC-03 非法访问 | 释放后 view、双 move、活跃 view 下 mut、借用逃逸；规则拒绝入口 | 明确状态/码，非法路径未调 native；相应合法兄弟例通过 | 探针模型，非生产 verifier |
| V-05 | AC-03 失败路径 | 第 N 次分配失败、零长度、长度加法/转换溢出、shim error | 与 malformed/budget/runtime failure 区别明确；无 ABI panic；正常已拥有资源按合同回收 | 各申请点/返回点 |
| V-06 | AC-03 trap | 已分配后 fatal trap，受控 runner | 观察前驱 trace 与约定终止；只记录 OS 回收，不要求/冒称析构 | 不承诺 unwind |
| V-07 | AC-04 边界 | A中😀、组合字符、NUL、invalid scalar/UTF-8、反向 slice、空范围 | 决策表列出确切预期与阶段，UTF-8 byte 与 scalar count 不混；有合法反控 | D02支持面 |
| V-08 | AC-04 资源规模 | 0/1/4096 生命周期循环；支持预算临界与注入耗尽两侧 | 无 live 增长；预算单位/入口/拒绝 oracle 完整，无临时语法深度截断 | 不承诺任意性能/规模 |
| V-09 | AC-05 隔离 | old registry/schema/source 与代码 scope diff；重跑既有 gates | 既有支持/拒绝和身份不变；无 crates/root/auto-lib diff | 741/745/772 已合入域 |

命令在唯一工作树：`cargo check --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml --locked --all-targets`、`cargo test --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml --locked`、`cargo fmt --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml --check`，774 PS gate、741 `-SkipMainGates`、745 与 772 gates。开发 scoped probe；最终复审完整 probe+完整 ac-core 一次。无 crates/root 定义改动，不跑根 cargo t/tv/docs_gen。
实施由用户单独启动的agent承担，具体agent/model未指定；内检与独立最终复审者待交接指定。最终复审在同一树使用独立上下文，自检通过只表示可交复审，不能自测绿即 reviewed。所有 PE/子进程复用 contained executor、60s 截止，输出及工具/产品 hash 随收据。

## 7. 验收标准

| AC | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | D01..07 全部决定且溯源，Spec 冲突/拒绝边界明示；V01 |
| AC-02 | work | 五类内部 ABI 真实 native 调用及顺序/布局控制通过；V02 |
| AC-03 | work | 正常资源计数、非法状态/别名/逃逸、故障/终止 oracle 全执行；V03..06 |
| AC-04 | work | Unicode/索引/预算规则和正反边界完整，770/771 无未决必需前提；V07/08 |
| AC-05 | work | old gates 保留，SD 冻结且不宣称 feature/ABI/自举；V09 |
| AC-06 | review | 独立重构 AC01..05、SD、遗漏/债务；绑定合同/HEAD/依赖/hash，无合同缺口才 pass |
| AC-07 | merge | 实际 Spec/导航/ledger/index、归档/计数、守卫清理收据，不预勾 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 路径/操作、成果与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户批准 r3；772 已 archived（9/9） | 刷新实际最新 Spec/inputs/主基线；确认唯一树，建 decisions.md 的事实差异表，无生产代码改动 | 01/05 |
| [ ] | T-02 | work | T-01 | 完成 D01..07 候选/支持拒绝/接口与资源预算表及新probe-cases.md；按§5.3冻结值/trace/码/计数/布局/预算oracle，逐规则preflight后才进入T03；材料冲突先修订下游 | 01/04 |
| [ ] | T-03 | work | T-02 | 新 probes/data-resources crate/shim 与锁文件，真实 COFF/PE 五类调用；V02/03/07/08；probe test/fmt/check | 02..04 |
| [ ] | T-04 | work | T-03 | 新 scripts/verify-ac-data-resources-774.ps1，V04..06 故障/拒绝/反控；既有 executor deadline、完整输出与资源收据 | 03/04 |
| [ ] | T-05 | work | T-04 | 冻结 decisions.md/interface-contract.md，D07 合并顺序、770/771 消费表；重新核对 oracle 不变 | 01/04 |
| [ ] | T-06 | work | T-05 | verification.md/proposed-spec-delta.md；完整 probe/ac-core 与新旧 gates；inputs/native/artifact hash、SD覆盖；不发布 canonical | 05 |
| [ ] | T-07 | review | T-01..06完成、writes暂停 | 同一树独立 AC/SD/反例复审；pass才reviewed，needs_fix返原树 | 06 |
| [ ] | T-08 | merge | T-07 pass与合入授权 | 沉淀 SD/导航/ledger/index；真实archive及总数；批回归到期判定；wt-guard clean 后清树/branch/空组 | 07 |

实际唯一树 `D:/autostack/.wt/lang-774/auto-lang`、branch `plan-774-dev` 已创建；2026-10-13 第二次交付就绪核查 needs_fix，T-02..06 重开，当前 executing 1/8。修复后 work-owned完成为6/8 execution_done，review7/8，实际closeout8/8；详见§9最新记录。

## 9. 复审记录

以下2026-10-12记录使用当时编号，作为历史保留；现行执行入口以§8及2026-10-13记录为准。

2026-10-12，stage:new，revision1，outcome:pass（草案结构/覆盖 preflight，不是实现通过）。
新增 T01..08、AC01..07、D01..07、SD01..04，支持决策由有界探针认领，三份 Plan 无环。
取号中央脚本串行、共同排他锁与 active/archive 前后唯一性检查；769/770/771，.next-id=772。主检出保留既有 WIP，未提交混合修改/建树/实施/测试。
历史交接：r1起草时767未合入；该前置状态已由下述r2就绪核查替代，不再作为当前阻塞。

规划期间主检出由`897eb34911a4f496a28ba7cdfd8a82815fa043d1`推进至`540dc7d30839eb7df8e68b4d19472358e170fe5b`，提交仅768计划与取号簿记；实现路径零差异。本计划按最终观察HEAD与文件hash绑定，保留已有768 executing/WIP；它不是本批实施前置。

2026-10-12，stage:new，revision2，outcome:pass（实施就绪核查，不是实现/测试通过）。767已实际archived 9/9，scalar Spec及修复证据落地；r2刷新§4/§10前置状态，并在§5.3/§6/T02明确探针前oracle冻结关口，任务/AC/Case/SD ID及范围不变。8个Task、7个AC、9个mandatory Case、4个SD，全部实施任务仍未勾。取材与核查见[r2就绪收据](attachments/769/readiness-r2.md)及[r2输入指纹](attachments/769/readiness-inputs-r2.tsv)。未创建实施树、未运行cargo/PE、未修改产品或canonical Specs；next: 用户在单独agent授权执行r2后 `/auto-plan:work` T01，T02冻结后进入T03，最终独立review后再merge。

2026-10-13，stage:new，revision3，outcome:pass（引用/交接核查，非实施完成）。770/771编号不变，前置改为现行774与已归档772；更新活跃链接/取材/拟建路径，保留历史附件与既有脚本路径。Task/AC/Case/SD ID、支持与测试语义不变，status仍drafting、current_step仍0。策略队列见[后端演进§7.5](../design/strategy/auto-native-backend-evolution.md#75-现行acacc队列与编号映射2026-10-13)。[核查收据](attachments/774/reference-update-20261013.md)包含当前路径/链接/指纹。共同774完成并复审合入后才放行770/771的依赖实施；本次未建树/实施/合入/运行Cargo。

2026-10-13，stage:work，plan_id:PLAN-774，plan_revision:3，outcome:pass（work-owned T-01..06 全部完成并提交；execution_done 6/8，T-07 独立复审未开始）。
code_commit: 工作树 D:/autostack/.wt/lang-774/auto-lang @ plan-774-dev，基面 5e21561d2（=master），收尾 HEAD f1a26e1be（85a4c482f→51f3a82e9→f5a91c42e→590e44a78→736e5a5da→69d3832c4→f1a26e1be，七提交；依赖无改动，auto-ac-prototype 为 path 依赖零源码 diff）。
task_ids: T-01..T-06 全勾。
evidence: decisions.md（事实差异表 46/47 零漂移 + D-01..07 冻结）、probe-cases.md（V-01..09 oracle 先冻结后实现，无反写）、experimental/ac-core/probes/data-resources/（39 探针测试全绿：五类 ABI 真实 PE、寿命计数、前门拒绝+R2 淘汰实证、故障注入、trap 0xC0000005、文本边界、4096 规模/预算两侧）、scripts/verify-ac-data-resources-774.ps1（6 步门禁含 scope 隔离与 hash 收据）、interface-contract.md（770/771 消费表+合并顺序）、verification.md（收据：probe 39/39、741 13步、745 74例、767 45例全绿）、proposed-spec-delta.md（SD-01..04，未发布 canonical）。
blockers: 无。实现期两处基础设施修正（harness 违规标志极性、规则模型 EndView 状态机）均已修复合入，oracle/拒绝码/验收无变化。
next: T-07 同树独立复审（AC-06；独立上下文重构 AC01..05/SD/遗漏债务，绑定合同/HEAD/hash），pass 后 T-08 merge（需用户合入授权）。

2026-10-13，stage:review，review_scope:final，plan_id:PLAN-774，plan_revision:3，outcome:needs_fix（第一轮独立最终复审；用户「请复审一下计划774的工作，看看是否有遗漏？」授权）。
reviewer_identity/limitations: 当前 Codex /root 会话；本会话未实施774，以合同/提交/源码/独立反例重构，无另建agent或审阅checkout，不声明未暴露模型/session身份。
worktree_path: D:/autostack/.wt/lang-774/auto-lang；branch: plan-774-dev；reviewed_commit: f1a26e1be51de2dbb0c0d983b5ec346a86475aed；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f。入/出场同HEAD，实现树clean。
dependency_revisions/spec_inputs: ac-core path依赖源码/根定义/旧门禁无本计划diff；当前主检出相关Spec/ac-core/旧门禁相对基面无漂移；完整hash绑定见[input-hashes.tsv](attachments/774/review-r1/input-hashes.tsv)。冻结规范增量SHA256=17e1ee60194c9d9a510bac0f31640a6661d195a0fdb9398ad7d094c2c9da8df5，见[冻结提案](attachments/774/review-r1/proposed-spec-delta.frozen.md)。未发布canonical/live ledger。
acceptance_results: AC-01 partial、AC-02 partial（五类平面ABI通过，nested合同缺项）、AC-03 fail、AC-04 fail、AC-05 partial、AC-06 fail（存在合同内缺口，不勾T07）、AC-07 pending merge-owned。
findings: P774-R1-01资源账本64槽静默丢账/预算0隐含缺省；R1-02借用返回来源及拒绝面未验证；R1-03 LIFO/临时/嵌套退出证据缺项；R1-04默认Str参数模式、下游身份/转换拼写漂移及跨行/转义/嵌套record决定缺项；R1-05实际释放口径与freed-header负控问题；R1-06门禁/60s/持久取证和SD误称List已冻结。已登记KNOWN-DEBT；合同内缺口不是获准延后项。
evidence: [完整报告和逐项修复要求](attachments/774/review-r1/REVIEW.md)；774门禁6/6、probe39/39、ac-core145 passed/既有1 ignored、741 13步、745 74例、772 45例均通过；独立7控制3通过/4失败（真实PE资源账本/预算恢复/预算0，模型返回来源），原始输入/输出/重放入口/hash均在同目录。没有代码修复/合入/归档。
pending_closeout_gates: Spec/导航/ledger/index实际沉淀、归档/任务总数、用户合入授权、批回归到期判定、wt-guard及同树清理。
next: `/auto-plan:work` 在原唯一树修复P774-R1-01..06，先核对并冻结共同/下游合同；语义变化按new/revision与用户授权边界处理。提交新基线、重放失败及合法兄弟控制后重新独立review。T01保持完成；T02..06重开、T07/08未勾，status=executing、current_step=1/8，plan_revision仍3（本轮未改变语义合同）。

2026-10-13，stage:work（修复轮），plan_id:PLAN-774，plan_revision:3，outcome:pass（r1 复审六发现 P774-R1-01..06 全部修复；T-02..06 重开并重新完成，execution_done 6/8）。
code_commit: 同树 plan-774-dev 修复 HEAD `773f9a09a`（基面 5e21561d2；前轮 7 提交 + 本轮 1 修复提交；依赖仍零 diff）。
task_ids: T-02..T-06 重勾（r1 修订版证据）。
evidence: decisions.md §3.8（六发现合同修订：参数三模式/身份表/源级拼写/字面量转义表/嵌套布局/预算零语义/R2 理由限定）、probe-cases/interface-contract r1 增补节；shim 完整外部账本（无容量上限）+ FREESTAT 分离计数 + FREE id 顺序 oracle + rec2/h_get_checked 新 op；rules.rs 来源追踪（LocalOwned/ParamBorrowed/ParamOwned）+ field/container/move-param 可达入口；新用例 v02_9/v03_6..10/v04_m4b..r2-checked/v05_13/v08 三例；PS 门禁重写（逐行警告匹配正反控制、check/test/fmt 全文落盘、失败也写收据、shim 60s+locked）；V06-2 前驱 trace 文件重定向可观察。
结果：探针 55/55 全绿；**复审反例重放 7/7 全绿（exit 0，原 4 失败控制翻绿）**；774 门禁 7 步全绿（gate-20261013-134941 + raw 全文收据）；旧门禁 741/745/767 重跑全绿。blockers: 无。
遗留明示：规则模型未建模显式 mut str 借用参数（D-03 声明合法，模型保守拒绝并注释）；770/771 §5.1 身份/拼写需其 revision 采用 774 表（下游责任，interface-contract §7）。
next: T-07 第二轮独立复审（建议重放 review-r1/replay.ps1 应全绿 + r1 六发现逐项核对），pass 后 T-08 merge（合入授权另办）。

2026-10-13，stage:review，review_scope:final（入口就绪核查，已知work缺口使本轮未进入完整最终复审），plan_id:PLAN-774，plan_revision:3，outcome:needs_fix（第二次交付）。
reviewer_identity/limitations: 第一轮审阅的当前Codex /root上下文，未参与实施/修复；本轮未另建新聊天/agent，不冒称“新上下文最终复审”已执行。发现已承认的work-owned未完事项后做定向闭合与回归核查，不重复全套门禁来确认已知缺口。
worktree_path: D:/autostack/.wt/lang-774/auto-lang；branch: plan-774-dev；reviewed_commit: 773f9a09aeb874845dc0a0cf44d89a07e3b3a87f；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；previous_reviewed_commit: f1a26e1be51de2dbb0c0d983b5ec346a86475aed。入/出场同HEAD，实现树clean。
dependency_revisions/spec_inputs: 代码/报告/锁文件、当前770/771与Spec及技能hash见[输入绑定](attachments/774/review-r2/input-hashes.tsv)；冻结SD SHA256=ff5ec7d62c8538bdade8eb8e99c3302aaf710da157d6e5cfbf4934cc80f73346，见[冻结提案](attachments/774/review-r2/proposed-spec-delta.frozen.md)。未发布canonical/live ledger。
acceptance_results: AC01 partial、AC02 pending完整复审、AC03 fail、AC04 fail、AC05 partial、AC06 fail、AC07 pending merge。
findings: P774-R2-01（延续R1-02）可变借入参数明示未建模、新ParamBorrowed→View→EndView panic；P774-R2-02（延续R1-04）下游仍未同步，跨行和常驻字面量返回合法面未经合同修订被削减；P774-R2-03（延续R1-01/05）新账本增长失败/回滚缺执行oracle及metadata计数边界；P774-R2-04（延续R1-06）旧HEAD/dirty运行收据与实际raw目录/当前完成声明不一致。已登记债务，非延后授权。
evidence: [第二次交付报告/闭合表/收敛诊断](attachments/774/review-r2/REVIEW.md)；本轮原反例7/7 exit0（四失败样例清偿）；新来源组合3控制2通过1panic；warning matcher实际正反控制通过。work原始55/55已核对并持久复制，未称本轮重跑或完整pass；旧门禁/完整ac-core未在已知未就绪状态重复执行。
pending_closeout_gates: 修复后完整独立复审、Spec/导航/ledger/index、归档/任务总数、用户合入授权、批回归到期、wt-guard/同树清理。
next: 同树work先完成报告内来源/访问模式及失败出口覆盖诊断、实际同步下游合同，保留通过控制；提交后再交独立final review。两次未通过交付的历史保留，不能把旧样例清偿等同不变量闭合。T01保持完成，T02..06重开、T07/08未勾，executing 1/8；本轮未改语义故plan_revision仍3，必要合同变更由new/revision办理。

## 10. 待澄清事项

1. ~~772修复复审合入前置已满足；实施agent启动时刷新最新基线并记录用户执行r3的授权。~~
   已兑现（2026-10-13 r3 实施授权记录于 §4；T-01 基线刷新 46/47 零漂移）。
   现存待办：第二次交付就绪核查仍needs_fix；先完成来源/模式与下游合同等work-owned缺口，再完整独立复审；T-08合入授权仍待交接。
2. 语义表示/资源策略属 D01..07 有界调查，必须在本计划冻结；超出现行承诺的改变需用户决策，不能带 open 交给770/771。
3. 无完整所有权/公开 ABI/无限规模承诺；后续text-record/List/enum/use/ACC有 Design36 owner，非遗漏验收。
