---
plan_id: PLAN-864
status: archived
feature_name: AV集合与变体源码闭环（R2-L 前半）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-av/collections-variants.md
  - docs/specs/auto-mir/collection-place-profile.md
  - docs/specs/auto-test-case/ace-m1-conformance.md
new_spec_components:
  - docs/specs/auto-ac/source-collections-variants.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 5
total_steps: 9

# /auto-plan:work 结束时填写：
work_status: execution_done
---

# [PLAN-864] AV集合与变体源码闭环（R2-L 前半）

## 0. 变更摘要

PLAN-822 已交付执行侧 `av.collections/1`（profile id 4、AVAD v3）：受控 List、Option/Result、有限名义 Variant、Index Place 的 typed 存取/转移/清理，但**明确排除 source 支持面（fixture-only）**（[auto-av/project](../specs/auto-av/project.md)、[collections-variants](../specs/auto-av/collections-variants.md)）。827 的源码前门记录“av source front door rejects List（`source.unsupported`）”。

本Plan补齐**源码到执行**这一段：真实Auto源文件中的 `List<T>`、`?T`、`Result<T,E>`、带载荷有限 `enum`、`is` 模式与 `for-in` 迭代，经前端 → Checked HIR → Typed MIR → ABC → 受检验证 → AV 全链执行，并关闭 852 语料 **L01–L07 / V01–V07** 的源码层缺口行。

不新增执行语义、不改 profile 4 的已冻结指令/布局；新增的是**源码域 + lowering 覆盖 + 消费者绑定**。

## 1. 目标

- 新源码 profile 支持：标量`List`、`List<Str>`及`List<非递归拥有型type>`（含`Str`字段）；构造/`push`/`insert`/`get`/`set`/`remove`/`len`/`clear`，`?T`/`Some`/`None`、`Result`、有限 `enum` 声明与带载荷构造、`is` 分支与载荷绑定、`for x in <List>` 迭代。
- 把852原L/V承诺正例变为真实源码执行；L06等原负例按冻结诊断/trap判定。拥有型L04/L05必须成功，不以带owner的拒绝替代。超出明确子集的输入才具名拒绝，不把typed fixture通过当作源码行关闭。
- 冻结 **SEM-02**（List 负下标/越界/空容器语义）与集合/变体的错误边界（trap vs Result vs 资源失败），并写入 Spec。
- 源码编辑走既有 828 函数级增量会话形状（未改函数身份/生产工作=0），不新造平行会话。

## 2. 架构方案

| 层 | 本Plan动作 | 边界 |
|---|---|---|
| 源码前端 | 新 `SourceProfile`（含 862 入口语义）+ 集合/变体/模式/迭代语法与推导、诊断 | 不引入 Map/Set/JSON、用户泛型、record 容器字段、List 嵌 List（822 非目标保持） |
| HIR | 新 descriptor 与精确可失败三元组；继承 822 的名义 Variant/Index Place 事实 | 不改旧 descriptor |
| MIR/lowering | 复用 822 集合指令与 Index Place；`for-in` 降为显式索引循环；`is` 降为受检分支+载荷投影；无新指令即不新增指令 | 不在 lowering 里偷偷做元素拷贝/借用扩展 |
| 执行 | `av.collections/1` 原样消费，不新增 profile id | 不改 822 指令语义/布局 |
| 增量 | 接 828 单元接口（source 侧会话，822 `collections_session` 同型扩源） | 授权预检/身份/失效沿用 822 合同 |

## 3. 技术栈

- 源码：`experimental/ac-core/src/source/{lexer,parser,resolve,typecheck,adapter,ast}.rs`、`src/{hir,descriptor,verify}.rs`。
- lowering/会话写面：`experimental/auto-mir-lower/src/**`、`experimental/ac-av-probe/probe/src/{collections_session,session}.rs`；只读消费`core/src/{mir,mir_verify,abc,abc_verify,av}.rs`的822合同。若发现执行核心旧合同缺陷，登记原owner独立修复/交接，不在此暗改语义。
- 语料/判定：`crates/auto-lang/test/cases/ace-m1/{src/L*.at,src/V*.at,binding.tsv}`、`crates/auto-lang/src/tests/ace_m1_tests.rs`、`scripts/verify-ace-m1-852.ps1`。
- 拟建 `scripts/verify-av-864.ps1`（Scope=targeted/compat/source/acceptance），测试前缀 `av_collections_864`；当前不存在。

唯一实施树 `D:/autostack/.wt/lang-864/auto-lang`，分支 `plan-864-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）。事实基础：

- [collections-variants](../specs/auto-av/collections-variants.md) 与 [collection-place-profile](../specs/auto-mir/collection-place-profile.md)：List/Option/Result/Variant 的 typed 存取、Index Place、转移/清理已冻结；`av.collections/1` 语义版本 1/1/1。
- 852 语料 L01–L07、V01–V07 已有**真实 `.at` 源**（`src/L01.at` … `src/V07.at`），当前 AV 列判为 `gap:av-unsupported-list`/`wait:av-variants-822`；`src/L01.at` 形如 `var xs = List<int>.new()` + `push`/`get`，`src/V01.at` 形如 `fn maybe(x int) ?int` + `is x { Some(v) -> … None -> … }`。
- 能力矩阵 C-15/C-16/C-17（List/Option-Result/enum 载荷与 `is`）源码层为“—”，typed 层为 T；C-09（for-in/range/迭代器）为“—”，本Plan只认领 `for x in List` 的有限迭代。
- SEM-02：旧 VM 保留 List 负索引（null-family），822 负下标先拒 → 必须显式冻结，不能拿两种语义直接对拍。
- 827/852 纪律：typed fixture 通过不等于源码支持；改源/删 import 只能作为新变体 ID，不能把原 AP 标绿。

本次授权与输入：用户明确要求修订波次/实际写面、补871旧能力覆盖及删除门、让864支持List与拥有型type元素；允许动作仅计划/设计/战略文档修订，不实施代码、不调整预算、不改868。基面路径、内容SHA-256、旧稿快照及静态预检见[修订收据](attachments/ace-wave-20261028/revision-summary.md)；观察到的current Spec/调用点与未来拟建入口分开记录。

## 5. 详细设计

### 5.0 波次、实际写面与交接（本次修订）

按[波次交接合同](reports/ace-wave-handoff-2026-10-28.md)§1–2执行：S为共享source/HIR/lowering/MIR/ABC/AV及会话接口，N为ac-mn，H为公共driver/SDK适配/测试注册与binding；分别单写，跨窗同时取窗。共享lowering的未来写面为D868后的`experimental/auto-mir-lower/src/**`，旧`probe/src/hir_adapter.rs`仅作搬前调查，不能再落新实现。T-02及接线前核验实际已合入revision、Spec/API/版本、owner释放与消费者兼容证据并记录接受；review/repair未完不交WIP。

第2波占S（前端/HIR/共享lowering及collections源码会话），H绑定另串行交接；已交付822 MIR/ABC/AV合同只读消费，不改opcode/layout/资源语义。866可并行做私有resident模块，默认本Plan先释放S再由866接会话/runtime公共接口；这不增加866对D864的语义依赖。871如需删除ac-core必须另取S，不能与本Plan同时写该公共合同。

首例即按既有文件化AutoDown/TestCase、cache fresh/hit/restart/body-edit、未改函数生产工作=0与现行P90≤2000ms口径验证；亚秒目标及P95/P99另列，不能伪称当前硬门已是亚秒。内存/资源峰值、live/回收及cache容量在T-02沿上游预算交集冻结，记录命令、样本和反控，不擅自提高上限。共享核心/SDK/测试迁移先确认拆分owner实际位置与释放；产品默认入口、旧AutoVM/a2r和868合同/进度不在本次修订写面。

### 5.1 前置与写面

硬前置：`D862`（入口/Unit）、`D822`（已具备）、`D828`（增量会话）、**`D868`（HIR→MIR lowering 出库）**——集合/变体的源码 lowering 必须落在共享 crate，否则后续 native 迁移要二次搬移。写面见 §3；语料只**追加行与新源文件**（若需为某行新增变体，必须同时保留原行并注明理由）。

### 5.2 源码语义冻结（T-02 决策件）

`attachments/864/work/collections-source-contract.md` 逐项冻结：

| 决策项 | 要求 |
|---|---|
| 支持子集 | 必含标量List、List<Str>、List<非递归拥有型type>（含Str字段）、既有Option/Result/有限Variant；逐项列源码形态。排除record容器字段、嵌套List、借用str元素、循环图及任意用户泛型；合法承诺输入不得以unsupported替代 |
| 推导 | `List<int>.new()`/`let xs = …` 的推断规则；`?T` 与 `Result` 的构造/返回；元素类型不隐式转换 |
| 方法签名 | 构造/push/insert/get/set/remove/len/clear逐条冻结参数、返回、失败；复用822合法指令或已证明等价序列，不新造opcode语义。get按Copy或受检只读view/Index Place；remove转移拥有权 |
| SEM-02 | 负下标明确拒绝（沿 822）并具名诊断；越界 trap 码固定；空 List 的 `get` 语义与 `len` 边界写死；旧 VM 差异登记为 profile 差异，不静默统一 |
| 模式 | `is` 分支穷尽性判定、载荷绑定形式、guard 是否在首批；失败诊断码；不得走“首个匹配即成功” |
| 迭代 | for-in求值次数/顺序固定；拥有型元素用不逃逸的受检只读view，不隐式move/clone；迭代中结构修改首版拒绝并反控，不做快照副本 |
| 资源 | push/insert/set的prepare→commit、remove转移、显式clone及全出口once-release沿822；失败前容器与未提交输入拥有权不变，禁止隐式深clone。T-02冻结预算交集与峰值测量 |
| 增量 | 哪些编辑走单函数道、哪些触发精确失效（容器接口变化 vs body 变化），与 822/828 同判据 |

### 5.3 lowering 与执行接缝

- `for-in` → 索引循环（不新增迭代器对象）；边界条件使用 List 长度读取一次或每轮读取必须固定并测。
- `is` → 受检分支 + 载荷投影（822 已有 checked branch projection）；穷尽性缺失在编译期拒。
- 构造 `Some/None/Ok/Err`/Variant 载荷走 822 既有转移/清理语义，不在 lowering 引入额外 clone。
- 与820/821组合的List<Str>/List<拥有型type>已由用户确认纳入；822排除的是record容器字段及嵌套List，并非拥有元素本身。L04/L05原源中的Item(name Str, code int)、push(x.move)、get字段投影、remove返回拥有值必须真实成立；不能改源为标量或fixture。

### 5.4 测试框架接入（既有 TestCase 协议，不新造框架）

L/V 两族目前只有 **typed 语料**走协议（`ac-av/collections-run-v1` + `av.entry: "collections:<id>"`，实质期望在父侧 `COLLECTIONS_CASES` 冻结表）。本Plan把**同源真实源码**接入：

- case 形态不变：在 `crates/auto-lang/test/cases/` 追加 source 行（复用 `ace-m1/src/L*.at`、`V*.at`），一份 source × 多 check（av 主判 + legacy/a2r 按实际适用性，ac-native 仍为 `gap:mn-scalar-only`）。
- recipe：既有 `collections-run-v1` 的 entry 语义是 fixture id，**不改造它**。T-02 在两者间冻结：(a) 新注册 `ac-av/collections-source-run-v1`（三缝 append-only + worker 源码域分支，推荐）；(b) 若既有 av binding 已有可用的 source/profile 字段则追加该字段（要求旧 case 逐字节不变）。二者都不得让 case 命名可执行文件。
- 期望落点：源码行的`expect.exit`/错误`stage,code`写在case文件（L06保留边界trap负例，不能冒作正例）；资源台账（alloc/free/live）若仍需父侧冻结表，按 822 `judge_*` 形状扩展，但“case 声明了什么就必须被判什么”，禁止声明后忽略。
- `binding.tsv` 只改状态/owner 列（`gap:*`→实测或换 owner），不改 12 列 schema；852 base suite 不被悄悄扩缩。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-collections-variants.md | 无 → 标量/Str/拥有型type的List源码、转移/只读view/显式clone/失败清理、模式/迭代/SEM-02 | 源码闭环 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/collections-variants.md | 追加 source 绑定与实测覆盖，typed-only 边界改写 | 事实纠正 | AC-02/05 |
| SD-03 | modify | docs/specs/auto-mir/collection-place-profile.md | 追加 source lowering 约束（for-in/is 不新增指令） | IR 合同 | AC-02/03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | L/V 行重绑定与实测映射 | 证据落点 | AC-05 |
| SD-05 | modify | docs/design/strategy/av-language-capability-matrix.md | C-15/16/17/C-09 源码列按实测更新 | 路线事实 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 源码面 | L01–L07/V01–V07原源逐行分类；正例构造/方法/模式/迭代；L06等冻结负例另判 | 正例手算值与852 expected一致，L04/L05必须42；负例保持原trap/诊断，源不改写 | T-03/04 |
| V-02 | AC-02 边界与SEM-02 | 负下标、越界、空容器、非穷尽 `is`、错型构造、迭代中修改 | 各自具名诊断/trap 码；把负下标当合法必红 | T-03/04 |
| V-03 | AC-03 资源 | 元素转移/失败路径清理事件（alloc/free 台账，once release） | 台账精确；多释放/漏释放/深拷贝冒充必红 | T-04 |
| V-04 | AC-04 增量 | 单函数 body 编辑、元素类型/接口编辑、强制整备反控 | 未改函数生产阶段=0/身份不变；退化必红 | T-04 |
| V-05 | AC-05 语料绑定 | 852 binding 行更新 + 独立 oracle + SKIP 审计 | 分母不缩；`wait:` 走完关闭或留 owner；改 expected 必红 | T-05 |
| V-06 | AC-01/03 拥有型List | 原L04 get借用字段、L05 remove转移；新增List<Str>源码变体：push hello、push 中、remove(0)，返回removed.len()*10+xs.get(0).len() | 原L04/L05=42；新变体=51（Unicode标量），独立手算；禁止替换原case，隐式复制拥有权必红 | T-03/04/05 |
| V-07 | AC-03/04 拥有资源与增量 | 拥有元素insert/set/remove/clear、显式clone后独立修改；分配失败前后/迭代结构修改/视图逃逸；fresh/hit/restart/body与元素接口编辑 | 失败不丢原拥有权、once-release精确；无隐式clone；非法借用具名拒绝；恢复重验、未改函数工作=0、内存回落及预算交集成立 | T-04/05 |
| V-08 | AC-04/AC-06 写面/兼容 | T-02及公共接线交接摘要；已交付消费者并集与窗口清单 | 真实D门/释放/接受齐全；缺交接或消费WIP不准接线；受影响profile兼容验证及首例cache/周转/内存证据齐全 | T-02冻结；最终work任务执行；review复核 |

定向命令：两独立 workspace 定向 `cargo test`；`powershell -File scripts/verify-av-864.ps1 -Scope <targeted|compat|source|acceptance>`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 承诺源码正例全成功，包含List<Str>及List<拥有型type>和L04/L05原源；原负例如L06按冻结trap/诊断成立，不以owner拒绝替代合法输入；V-01/06 |
| AC-02 | work | SEM-02 与边界/诊断矩阵成立，无静默降级；V-02 |
| AC-03 | work | 拥有元素get受检view/remove转移/显式clone/失败原子性与once-release、峰值/live/清理按预算成立；V-03/06/07 |
| AC-04 | work | 源码编辑走828增量，fresh/hit/restart重准入、未改函数零重做；拥有元素接口编辑精确失效，周转/内存样本齐全，退化反控必红；V-04/07/08 |
| AC-05 | work | 852 binding/独立 oracle/分母审计落地，typed 通过不冒领源码行；V-05 |
| AC-06 | review | 独立核 AC/V/SD、lowering 是否私加拷贝、profile 4 语义零漂移、warnings/fmt；核V-08交接与首例指标 |
| AC-07 | merge | 冻结提交裸 `cargo t` + `tv`（改 VM/编译器触面）通过 |
| AC-08 | merge | Spec 沉淀、矩阵更新、ledger、归档、guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D862、D822、D828、D868 | 核822类型/Place/资源合同，L04/L05拥有型原源及其余L/V正负例，确认List<Str>/List<拥有型type>为已批准必含范围；产出baseline | AC-01/02/03 | [✅] |
| T-02 | work | T-01 | 冻结源码/资源合同与有界全链可行性探针（L04/L05、List<Str>、失效view/失败清理），先接受S/H交接；T-03首例完成探针后再扩面，失败返回本步调整策略，不缩支持面；冻结方法/view/SEM-02/增量；继承822值数据8MiB/受管32MiB/worker增量128MiB与MAX_LIST_ELEMS=65536并取其他上游预算交集；不得排除拥有型List或擅自放宽上限；冻结fresh/hit/restart/body-edit、周转/内存口径与反控 | AC-01..05 | [✅] |
| T-03 | work | T-02 | 先以原L04/L05及List<Str>打通最小源码→共享lowering→既有822执行的全链可行性探针并核资源门，通过后扩前端语法/推导/诊断/descriptor；V-01/02/06，未取S窗不接线 | AC-01/02 | [✅] |
| T-04 | work | T-03、S释放/接受 | 共享lowering的for-in/is/构造与拥有List方法 + 会话；复用822执行语义；V-02/03/04/06/07 | AC-01/02/03/04 | [✅] |
| T-05 | work | T-04、H释放/接受 | 原L04/L05及新增List<Str>变体文件化绑定/oracle/资源与增量反控；V-05/06/07；提交代码及证据摘要，原始证据本地保留；执行V-08首例cache/文件化/周转/内存与兼容验证 | AC-01/03/04/05 | [✅] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-06 | [✅] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/矩阵/ledger/归档/guard | AC-08 | [ ] |
| T-09 | merge | T-08 | 交接 865（闭包源码）与 866（在线重载）实际基面与限制 | AC-08 | [ ] |

## 9. 复审记录

`stage: work | plan_id: PLAN-864 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/lang-864/auto-lang | branch: plan-864-dev | base_commit: 08ea8b35d | code_commit: 38dc10e33..work-head | dependency_revisions: auto-down@c3464d0 (detached sibling), auto-ui@154a396 (detached sibling) | task_ids: T-01..T-05 | evidence: 见下 | blockers: 无（merge-owned 门禁 pending：裸 cargo t + tv 于冻结提交） | next_responsible_role: review`

2026-10-29 work T-01..T-05 完成（S窗按波次合同队首 862→864 取得；863/865/866/871 均 drafting 无争用）。逐任务证据：

- **T-01 基线**：822 合同审计（mir.rs 指令面/mir_verify 类型与 lattice/av.rs 执行与 LIFO 清理/SEM-02 单陷阱码 av.trap.list-index；入口约定零参 i32）+ L/V 语料逐行核读；确认 List<Str>/List<拥有型type> 为已批准必含。合同冻结件 [collections-source-contract.md](attachments/864/work/collections-source-contract.md)（C1..C10）。
- **T-02/T-03 可行性**：前端垂直切片（lexer `?`/`->`、?T/List<T>/Result<T,E> 文法、enum/is/for-in 语句、泛型接收器、List 方法面、ElemView/ListFieldGet 融合、is→guard链/for-in→索引循环 resolve 降级、descriptor schema collections.atom、verify_collections 含 view-escape/scrutinee/穷尽规则）— `cargo test --manifest-path experimental/ac-core/Cargo.toml --test collections_864`（L01/L04/V01 真源编译+验证 3/3）。
- **T-04 lowering+会话**：`lower_checked_collections`/`lower_function_collections_unit` 全部落 822 既有指令（无新 opcode；clear=尾删循环展开；变体槽写走 VARIANT_SWITCH；unit 效果语句无槽 NO_SLOT 约定）；owned 元素投影融合（标量场 ListFieldGet；Str 场 remove→exact-type FieldGet(handle-share)→View→StrLen→BorrowEnd→insert-back）。split_units 修复内嵌类型操作数闭包（Aggregate/ListNew/VariantPack/Dyn*）。会话=CollectionsSourceSession（822 CollectionsUpdateSession 同型扩源，未新造平行会话）：内容 fp 选择重降级，未改函数走 carry 且 production counters=0（计数器实证，非推断）。
- **T-05 文件化+指标**：recipe `ac-av/collections-source-run-v1`（contract.rs 三缝 append-only 注册+mod.rs 派发+RecipeFamily::CollectionsSource+worker 源码分支）；17 行文件化套件（L01–L07/V01–V07 按 852 冻结分母值 + L06 SEM-02 trap 行 + 追加 N01 List<Str>=51/N02 for-in copy=42/N03 for-in owned 投影=53，新源文件追加不改原行）；binding.tsv 14 行重绑 ok:P864-source（仅 status/owner 列）；`scripts/verify-av-864.ps1`（targeted/source/compat/acceptance）。**acceptance GREEN**：定向族 38 测试（3 前端编译+12 负例诊断+4 全链+16 语料+3 会话）、17/17 文件化行经真 worker 全绿、852 基套件 97 检查零漂移。**V-08 首例指标**：17 行 × 3 次 auto-case 真实运行，P90=362ms ≤ 2000ms 现行开发硬门（中位 218ms，max 432ms；含 worker spawn+全链）；未改函数生产工作=0 由会话计数器实证（body-edit 精确重降 1 单元/carry 1；纯 hit 0 重降/2 carry）；once-release：全部正例 live_after=0。
- **负例矩阵（AC-02）**：12 个具名拒绝（pattern-non-exhaustive/duplicate/unknown-arm、view-escape、iterate-mutate、move-required、list-value-context×3、type-ambiguous、borrowed-str、record-methods）逐项测试通过。

写面纪律：S 窗（ac-core source/hir/descriptor/verify + auto-mir-lower + probe session/driver/worker 接口）+ H 窗（contract.rs/ac_av.rs/mod.rs 派发/ace_m1_cases/binding.tsv）+ probe cache.rs split_units 修复（共享单元缝的内嵌类型操作数闭包，820 起潜伏，collections 源单元浮出；822 三元组回归绿）。probe crate 的 pre-existing fmt 债（data_driver/session/owned_driver/bench bins/820 测试文件的纯格式化行合并）未触碰语义。已交付 822 消费面零改动（fixture case 文件逐字节未动，822 三元组回归绿）。merge-owned 门禁 pending：裸 `cargo t` + `cargo tv`（VM/编译器触面）于冻结提交执行。`stage: goal | plan_id: PLAN-864 | plan_revision: r2 | round: 1 | from: drafting | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-864/auto-lang | commit: 见上 | next: review`

2026-10-29 独立最终 review（T-06/AC-06，独立上下文于同一树，非二次检出）：**PASS** @ `38dc10e33`（复审后修复提交 `938a2ac7f`）。八项核对全过并逐项引用 file:line 证据：(1) AC-01 真源正向全绿且 14 个 852 源字节未动（diff 空），oracle 手算一致（L02=42/L03=0/V03=3/N01=51/N03=53）；(2) AC-02 单陷阱码 av.trap.list-index 冻结面无新编译期下标校验，12 具名拒绝逐项断言；(3) AC-03 profile-4 零漂移——core/{mir,mir_verify,abc,av,contract,ownership}.rs diff=0，lowering 新增行仅既有 822 指令（无隐藏 clone；clear=尾删循环展开）；(4) AC-04 会话计数器实证（fresh=2/body-edit=1+carry=1/hit=0+2/接口编辑精确失效），split_units 修复为真悬空类型索引修复且旧 profile 零漂移（822 三元组 3/3）；(5) H 面 append-only（0 删行；av-data/822 diff 空；binding.tsv 仅 status/owner 列、分母值列未动）；(6) 健康零新警告、无调试残留；(7) 写面披露（9 个文件在收据字面清单外，均为新 HIR 变体的编译必需穷尽臂/身份注册，旧行为保守）记录为 P2；(8) Spec delta 与实现一致且分支未动 docs/specs。acceptance 门由复审者重跑 GREEN。发现：P1×1（pattern-scrutinee 缺负例）已在修复提交闭合（13 具名拒绝全测）；P2×5（session 族入 verify 脚本——已闭合；fn_hir_codec tag-11 decode 不对称——latent 且响亮失败，记 866 后继；SD-05 目标不在 frontmatter——merge 沉淀时补；corpus 注释——已清；SEM-02 负下标无字面源码行——结构性覆盖充分，不阻塞）。merge-owned 门禁 pending：裸 `cargo t` + `cargo tv` 于冻结提交。

`stage: goal | plan_id: PLAN-864 | plan_revision: r2 | round: 2 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-864/auto-lang | commit: 38dc10e33 (review) / 938a2ac7f (repairs) | next: merge`

`stage: goal | plan_id: PLAN-864 | plan_revision: r2 | round: 3 | from: review | to: merge | status: archived | outcome: pass | delivery: merge commit on master (feat(auto-av): collections/variants SOURCE closure) | specs: docs/specs/auto-ac/source-collections-variants.md (new) + collections-variants.md + collection-place-profile.md + ace-m1-conformance.md + capability-matrix C-15/16/17 delivered & C-09 partial | ledger: .autoos/specs.json PLAN-864 upsert | merge gates: cargo t 3186/3186 + tv 163/163 green | cleanup: worktree guarded+removed | next: done`

2026-10-29 merge 收据：T-07 门禁（裸 cargo t 3186/3186、tv 163/163）于冻结提交通过；首次跑出的 cte worker 超时/失败归因为 pre-existing worker-build 竞态（master 同样复现 pass/fail 交替），非本计划回归（ac-sema-core 投影适配器已补受控臂并复绿）。T-08 Spec 沉淀 SD-01..SD-05 + ledger upsert + spec-index 刷新。T-09 交接：865（闭包源码）基面=collections 源前端已就绪（is 链降级/generic 接收器分类/ProjectionView 融合可复用；闭包 opcode 0x32..0x34 在 core 已存在）；866（在线重载）基面=CollectionsSourceSession 计数器/carry 语义 + fn_hir_codec tag-11 decode 不对称待补（latent，响亮失败，P2 记录在案）。

2026-10-28 r1→r2（用户授权修订波次/写面，864纳入拥有型List、871补旧能力与删除门）：仅修订执行合同，不实施。原status/current_step/total_steps、任务勾选与历史记录保留；D868合同和进度不变。V-08补交接/消费者兼容与首例cache、文件化测试、周转及内存核验；详细依据与静态预检见[波次合同](reports/ace-wave-handoff-2026-10-28.md)和[修订收据](attachments/ace-wave-20261028/revision-summary.md)。本记录只说明文档修订，未执行AC、性能测试或独立实现review，merge-owned门禁仍待实施后完成。

2026-10-28 new r1：`stage: new | plan_id: PLAN-864 | plan_revision: 1 | outcome: pass（等 D862，合同可预检） | next: D862 后 work T-01`。全部 AC/V/T 未执行；子集清单、诊断码、SEM-02 具体形态待 T-02 冻结；未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- 已裁定：List<Str>与List<拥有型type>进入首批，含Str字段的非递归type；T-02只细化既有822合同下的源码表示、诊断和预算，不再作为是否纳入的开放问题。
- 首版迭代中结构修改明确拒绝；拥有元素采用受检只读view。以后若要放宽快照/写迭代语义另立合同，不阻塞本Plan按现有范围推进。
- `for-in` 是否同时支持 range（`0..n`）/数组/Tuple：默认只 `List`，其余留 R2-L 后续切片。

2026-10-28 D868 前置解除：PLAN-868 已交付落 master（delivery 7754c4e4c）——HIR→MIR lowering 唯一实现为 `experimental/auto-mir-lower/src/**`（纯搬移等价已证，旧 `probe/src/hir_adapter.rs` 已删除），本计划 D868 硬前置视为满足；lowering 相关 T 任务的写面以该 crate 为准（边界合同见 [docs/specs/auto-mir/hir-lowering-boundary.md](../specs/auto-mir/hir-lowering-boundary.md)）。
