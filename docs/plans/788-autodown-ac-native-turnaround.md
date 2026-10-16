---
plan_id: PLAN-788
status: executing
feature_name: AutoDown AC 原生测试接入与增量周转观测
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
current_step: 5
total_steps: 10
supersedes_spec_components:
  - docs/specs/auto-test-case/project.md
  - docs/specs/auto-test-case/architecture.md
  - docs/specs/auto-test-case/profile-v1.md
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-ac/artifact-probe.md
new_spec_components:
  - docs/specs/auto-test-case/profile-v2-native.md
touched_goals: []
affects: [auto-test-case, auto-ac, auto-syntax, auto-cache]
---

# [PLAN-788] AutoDown AC 原生测试接入与增量周转观测

## 0. 变更摘要

承接[Design35](../design/35-autodown-test-cases.md) B阶段，给AutoDown TestCase增加AC source/HIR检查、真实Windows原生构建/执行和787语法观测；将783真实缓存命中/失效、各阶段测量及端到端周转放入同一份可复验报告。先引用现有源码/HIR/oracle，不大搬语料，不删除Rust内省测试或原CLI门禁。

本计划是AC/ACC后续能力的测试基础，不实现新语言特性、类型推导/泛型/meta、AVM编译期执行或生产ABI。上述架构要求分别消费[Design39](../design/39-ac-incremental-syntax-and-language-platform.md)、[Design40](../design/40-compiler-artifacts-and-cache-lifecycle.md)、[Design41](../design/41-interface-metadata-and-foreign-abi.md)、[Design42](../design/42-semantic-metadata-inference-and-comptime.md)。

787当前executing r1、4/9；协议与现有AC/native接入可先独立实施，T-06的syntax接入必须等787独立复审、门禁、实际合入与Spec沉淀后再绑定API，不从其在途树搬代码。全部必验完成才可execution_done。

## 1. 目标

1. 冻结严格版本化的TestCase v2增量，旧v1原样接受/拒绝；不解析后忽略未知字段。
2. 同一AutoDown用例可声明源码或原始HIR，分别检查诊断/规范HIR、真实构建、实际产物执行。
3. 保持真实actual产物交接、路径只读、截止/输出上限/全树回收和明确基础设施失败。
4. 通过现有AC core/scalar及783统一store，验证fresh、热点、重启/跨工程命中与变更失效，断言每次重新比较。
5. 接入787的两个有限grammar、CST/诊断与edit观测；不把语法树称为native编译结果。
6. 从每个新check首例记录阶段与端到端耗时/等待/命中/重算/I/O/内存；对代表局部编辑路径实际执行P95<1s门禁。
7. 逐例对账旧入口与新case/check/oracle，保留既有能力和门禁。

非目标：Rust generated compile-run、全部旧语料迁移/可视编辑器、通用性能实验平台、生产cache迁移/新GC、ABI定稿/FFI、HIR→ABC/CT执行、MIR/优化pass、语言扩面或替换生产parser。新native执行器首批只支持已合入core/scalar、Windows x86_64-pc-windows-msvc；text/record/text-record仍由旧门禁守护，不谎称本次全部导入。

## 2. 架构方案

保持依赖单向：auto-test-case负责文档/合同/比较/报告，auto-lang::test_cases及auto-case负责编排；新增独立 `experimental/ac-case-worker/` 消费ac-core、ac-artifact-probe和合入后的ac-syntax。auto-lang不静态依赖整套实验编译器/Cranelift，协议核心不依赖执行后端或UI。

worker使用有界版本化request/response，返回结构化诊断、真实产物manifest、执行观察和trace。父runner重新核request身份、原始字节哈希/路径/产物集合及oracle，不信任旧PASS。worker不执行自定义shell命令，不加载expected决定如何编译。

同进程热点用显式有界worker会话/批次证明，不能每次新起进程却标same-process。会话最多256个请求、总生命周期≤180s，每请求仍受§3预算约束；初次启动及排队计入相应请求，预热另记。重启场景必须真建新worker，不能只重置统计。T-01冻结会话消息/清理协议，完整生命周期沿用contained设施，不建立无限常驻测试daemon。

必要时对ac-core公开前端/执行器、783 driver增加窄的结构化观察/分阶段入口；旧API保留，零语言/ABI改变。不能复制一套编译管线、另造私有cache或用正则从stderr猜阶段。787 runtime零产品依赖红线保留，syntax adapter放worker侧。

## 3. 技术栈

沿用pinned vendored AutoDown parser与auto-test-case核心；serde JSON用于worker IPC，v2报告沿用协议模型和父runner投影，原生输入/输出/产物保持bytes。核心crate保持无backend/UI依赖，不为IPC拉入实验编译器。worker独立workspace/锁版本，复用ac-core::link已有受控启动/Job/deadline设施、783身份/store和实际编译接口。

当前link执行器返回lossy String且无输出硬cap。T-03须在同一contained lifecycle实现中增加原始bytes/有界采集与资源失败，旧String API作兼容wrapper；不能用read_to_end无限增长或有损UTF-8冒充bytes。worker、linker、PE均在完整进程树和总截止范围内；不允许Job建立失败后降级裸跑。

首批拟议执行预算：每次worker请求build≤60s、run≤5s、total≤70s；stdout/stderr各≤1MiB，IPC元数据≤4MiB、单产物≤64MiB、check实际产物总量≤128MiB。T-01冻结可配置字段/硬cap与边界测试，不能事后放宽换绿。它们是测试安全资源策略，不是语言源码/运行时间上限。case输入沿用v1已有admission限额。

## 4. 需求分析与背景调查

依据[Spec overview](../specs/overview.md)、[TestCase project](../specs/auto-test-case/project.md)、[v1合同](../specs/auto-test-case/profile-v1.md)、[测试架构](../specs/auto-test-case/architecture.md)、[AC](../specs/auto-ac/project.md)、[783交付](../specs/auto-ac/artifact-probe.md)。Design是目标态，Spec和代码是当前事实；不提前登记native/syntax已支持。

调查主检出master `67b44e0409fb11ef45b8d01b0ed17e2cc0ec3c05`；787在途树当时HEAD `8b1920c33`、r1/4步。逐文件SHA-256见[planning-inputs.tsv](attachments/788/planning-inputs.tsv)，包括本会话尚未提交的设计更新，work须核实际合入文档/接口。

| 实际入口 | 当前事实与缺口 |
|---|---|
| auto-test-case/src/{contract,suite,ir,compare,report}.rs | v1仅Adapter::{Avm,A2r}、Run/Emit，source语法auto/auto-script，Stage parse/type/run；无原生退出码/新观测支持 |
| auto-lang/src/test_cases/{mod,materialize,stale}.rs | 已有先选择后执行、actual产品交接、输出域审计/身份stale，必须复用 |
| auto-lang/src/bin/auto-case.rs | 现有统一CLI，feature test-cases，gate 0/1/2与被测进程exit分离 |
| auto-lang/test/cases/pilot/*.case.ad | 773引用式pilot可作旧v1兼容控制，裁判不改 |
| ac-core/source/adapter.rs::compile_parts_with | 真parse/resolve/typecheck/发射/bind在一入口；返回Diagnostic，但阶段细分/trace需窄观察入口 |
| ac-core/atom_text.rs::Diagnostic、verify::*、native::lower_object | 有真实结构化诊断/准入/lowering，不应把上层String当完整类型化错误 |
| ac-core/link.rs::run_deadlined_split | 有suspended→Job→resume、单截止含收集；stdout/stderr目前lossy转String，无capture限额 |
| ac-artifact-probe/driver.rs::vertical、StageCounters、VerticalCfg | core/scalar真实纵线/缓存/计数已交付；错误仍多为String，toolchain_fp调用方传入，不可直接继承为稳定native协议 |
| 787 runtime/grammar/adapters（在途） | 已有包布局，但未交付接口不能冻结为依赖；新syntax观测须等D-787 |

用户授权本轮规划并说明下一计划；继承其元信息/类型/CT、ABI、统一cache和亚秒指标要求。只调查/草拟，不实施、不创建树、不联系代理/其他聊天；没有指定实现模型/个人或时间预算。本文资源值为拟议合同，不冒充用户给定数值。

现存785/781在途改动与本计划实现范围不同；主检出债务/其他计划WIP不改。未来worker观察不把Design42未实现的infer/CT阶段填0ms；明确unsupported/not-applicable。

D-787仅阻止T-06和最终完整验收：需787合入/Spec/最终审查及merge收据，实际API/grammar版本重新绑定。T-01–05基于现有AC/783独立开展。783交付只证明其既有探针能力，本计划重新验证adapter的真实消费路径。

## 5. 详细设计

### 5.1 严格v2与观察支持表

新增 `autolang.case/2`、`autolang.suite/2`、`autolang.test-report/2`；v1入口和合法/非法集合保持不变。共享内部模型可扩，但按文档schema门控；v1不得因此接受native、budget、metrics等新字段。v2可继续用avm/run、a2r/emit，行为不变；显式默认继承/required覆盖/选择规则继续成立。

v2最小新增面：source syntax明确auto与hir-atom（虚拟后缀.at/.atom），adapter `ac` 和 `syntax`；固定profile/target/entry、构建/运行预算、cache模式、类型化阶段/原生exit、trace/性能断言及有界edit序列。字段名/decoder/错误路径与IPC Schema在T-01决策稿冻结，必须有合法/非法Atom文件；不扩大为任意recipe/shell/options透传。

| adapter/action | 输入 | 可承诺观测 |
|---|---|---|
| ac/check | auto-source或原始HIR，core/scalar | 真实前端/绑定/verify结果、规范HIR bytes、类型化诊断/来源；无PE运行声明 |
| ac/build | auto-source或HIR | Checked→真实COFF/PE、manifest/来源与trace；产物集合明确 |
| ac/run | 本次有效build产品，或声明完整source→build→run | 正常进程完成、原生exit code、stdout/stderr原始bytes、阶段trace |
| syntax/parse | 787有限core或interface-mini grammar | 当前revision CST/诊断/outline/原文重建的规定Schema投影 |
| syntax/edit | 同一base及有界非重叠edit序列 | fresh与incremental规范结果、实际重扫/重解析/页复用和修订发布状态 |

不支持观察一律admission ERROR。语法negative是语法诊断，不能获得语义Checked；不支持profile/target不会被SKIP掩盖。本计划不追加任意语言的native/CT支持。

### 5.2 阶段分类、产物链与比较

保留Diagnostic的code/span/来源及真实阶段，补调用边界origin区分lex/parse/resolve/typecheck/bind/verify/capability/backend/link；内部adapter错误、IO、工具缺失、panic/非法退出、超时/超输出、失配worker响应归Infrastructure。链接器工具失败不满足语法/type负例。stage错误分类不依赖render字符串。

ac/run进程正常退出与exit code分开：程序ExitProcess(70)可作为精确运行oracle，不等于runner gate70，也不据此猜为VM trap诊断。崩溃/被杀/Windows异常不伪装用户期望exit。短路/求值序/溢出已有fixture为语义来源。

product_from按真实producer manifest交接：request/source/profile/target/entry/worker/compiler/toolchain hash、每项实际路径/长度/hash、对象/可执行及依赖绑定。比较FAIL但有效产物仍可供后续执行；执行失败/无产物才BLOCKED。消费者不重编source替代坏产品，更不能从expected取可执行代码。有效frame替换、错target/过期manifest/越界路径均受控拒绝。

不要求PE原始bytes跨构建相同；规范HIR可按bytes比对，obj/PE身份绑定本次实际字节，正确性看真实运行观察。source/expected始终只读；普通run不更新oracle，proposal仍只生成候选并记录消费者。

### 5.3 IPC、路径与生命周期

唯一run/case/check scratch；输出域与suite/fixture/source/oracle根双向不相交且全链reparse审计。worker只接受本次有界request和owned scratch的response路径，响应绑定nonce/请求hash与实际工具身份；未知schema、nonce错、缺响应、旧响应或越界产物为ERROR。比较器在父侧，worker不能靠返回PASS授信。

选中检查之前不建scratch、不启动worker或编译工具；取消/截止覆盖编译、运行、stdout/stderr收集及全树reap。两路并发排空，达到上限立刻终止并报告，不返回截断成功。非法UTF-8输出仍按bytes比较；另提供安全展示，不改oracle字节。

扩展已有contained executor并重放其后代持管/快速spawn/Job失败反例，不另写第二份生命周期。缺工具链明确ERROR，required native在不支持平台不能绿；非Windows下纯协议测试可运行，native gate保持不满足而非假pass。worker自身测试不允许清空任意用户cache/temp路径。

### 5.4 cache/增量与报告身份

复用783 ManagedStore和分阶段产物，fresh/reuse/read-only模式明确：fresh禁止读取旧编译结果但可按安全合同发布；reuse有效命中否则真实重算；read-only缺失明确cache miss/基础设施状态，不假装语言错误。所有模式每次重新读取当前oracle并比较，**不缓存PASS**。

使用实际source/HIR/grammar内容、依赖、profile、target/ABI、entry、runtime/support库、后端/优化选项、真实工具文件/版本及锁文件/worker recipe构造key，不能沿用未验证的调用方常量toolchain_fp。当前仅验证已实现的输入轴；未实现meta/generic/CT反射依赖明确future，保留扩展字段需schema而不是无效空表。

必验fresh、同进程热点、新进程磁盘热、跨工程相同内容、body编辑、profile/工具链配置变化、损坏/缺块重算；同签名body变化必须miss对象。HIR命中重验，obj命中跳过相应前端/verify/lower后真实链接运行；计数绑定真实API，并以“若调用该阶段则失败”的反控验证，不能只改统计。

783库入口可以增加阶段/观察回调，但既有store/GC/codec合同不弱化；发现新缺陷保留反例、明确归属后修订计划，不把它藏在runner重试/吞错。GC并发/预算继承783，不以本计划实现生产调度。root/lease与失败退出清理须可观察，不把每次test scratch变成永久强pin。

### 5.5 全阶段周转从首例记录

v2报告分开功能结果与性能结果，缺测为gap，未实现阶段not-applicable而非0ms。每请求关联trace，阶段span覆盖admission/materialize、排队/IPC、parse/resolve/typecheck/bind/verify、cache读取/校验/重算/写入、lower/link/run、诊断/结果发布；计wall/等待、可得CPU/I/O/峰值内存及真实计数，未能取得的指标标明且必需项缺失使对应性能门失败。嵌套span不重复归因，不把各阶段P95简单相加。

时钟从runner接收本次编译/edit请求开始，包含本次materialize、worker进程生发、校验/排队/发布，到正确诊断/可运行产物/真实run分别结束。报告AutoDown载入/工具构建准备与进程整体时间，工具准备可单列；不能扣掉用户请求实际经过的IPC/等待。尚无热重载，明确不宣称新代安全生效。

参考Windows工作站在T-01记录CPU/RAM/OS/工具链/电源与负载，固定release工具、预热策略。六组每组≥30样本：fresh、热点、重启磁盘热、跨工程、局部body编辑、GC并发。以现有p04及“循环界3改4”的独立exit3/4 oracle作代表原生周转；另测787的局部edit负载并复用其预算/复用不变量，不事后提高指标。

**本计划硬性能验收**：规定参考环境下，原生热点/局部body编辑的请求→正确run结果P95<1000ms；787规定局部edit的请求→当前正确syntax结果P95<1000ms。其余场景报告P50/P95/P99/峰值和长尾，不泛化为任意仓库亚秒。每场景功能与计数必须正确，超时/取消/失败样本不得丢掉。T-01据基线冻结阶段预算分配/测量开销/样本配方，不得改变上述总阈值；实测不达标则明确未达并给证据，不能取消AC或扩大预算。

持续编辑采用至少50次连续revision请求（固定到达间隔100ms），报告旧任务取消/过期、队列积压、最新revision完成时间与发布顺序，禁止错版本发布；该负载用于暴露排队/取消，不能用完成样本筛掉其余请求伪造P95。性能门在冻结参考环境运行，不对每个普通单测在竞争机器上加毫秒断言。

### 5.6 首批用例与覆盖账本

新 `test/cases/ac-native/` 包含v2 suite、case与来源映射；fixture_root可审定为仓根以引用experimental源码/HIR，输出/cache必须在该根之外的专属owned temp目录。admission保留路径/输入上限，不因仓根扩大允许任意source路径或删除。

至少24个独立语义case（动态check数另计），来源账本包含legacy path+test entry/selector→case/check→oracle→新旧gate。必须含：
- core p01/p04/p12/p13/p15/p16：优先级、循环、未执行trap、实际overflow、命名参数顺序、trap前可观察输出；fixture oracle源于既有独立测试。
- scalar除零/短路/条件for至少各一例，复用现行scalar支持面。
- source invalid的换行/未闭合/未定义名/类型错，以及原始HIR结构/类型/引用负例，至少各两例，不能只从正常source生成“非法HIR”。
- valid HIR→check/build/run，build比较FAIL但实际产品仍被run消费的反控，以及good expected+bad actual必红。
- 787两个grammar的合法/非法/原文保真及edit/fresh等价例；unsupported profile或观察显式ERROR。

24是最低导入规模，不替代§6协议/生命周期/性能补充矩阵。还未实现的profile仍标未导入，原门禁不退役；不得为漂亮统一格式自动重生成期望。

### 规范增量

以下拟于merge沉淀，不在规划时编辑canonical Spec。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/profile-v2-native.md | v1无native→明确v2支持表/阶段/产物/bytes/预算/cache/trace/性能 | 不静默扩v1 | AC-01–06 |
| SD-02 | modify | docs/specs/auto-test-case/profile-v1.md | 保留v1合同，增加v2导航/版本拒绝边界 | 兼容可复验 | AC-01、07 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | A阶段→B阶段worker/单向依赖/typed IPC/containment与比较分工 | 执行可信 | AC-02、03、07 |
| SD-04 | modify | docs/specs/auto-test-case/project.md | 增补已实现v2/native/syntax支持、profile/平台边界与入口 | 事实导航 | AC-01–07 |
| SD-05 | modify | docs/specs/auto-ac/project.md | 补AutoDown入口/导入范围，旧语言面/CLI门禁不变 | 不冒称完整AC | AC-02、07 |
| SD-06 | modify | docs/specs/auto-ac/artifact-probe.md | 补真实consumer/观察扩展、依赖身份与预算继承，非生产cache | 统一产物接线 | AC-04、06 |

## 6. 测试设计

以下入口由T-01–08物化，并非已执行证据。工作期使用 `cargo test -p auto-test-case <filter>`、`cargo test -p auto-lang --features test-cases <filter>`、worker独立manifest的定向filter及必要check。新 `scripts/verify-autodown-ac-788.ps1 -Scope acceptance` 聚合全部必验并拒绝缺例/skip/失败；`-Scope closeout`供merge冻结提交执行完整触面独立套件。

| ID | AC / 不变式 | 输入/状态 | 实际/拟新增入口与work owner | 区分性oracle/边界 |
|---|---|---|---|---|
| V-01 | AC-01 版本 | v1 pilot、v1塞v2字段、v2错误类型/未知项、坏fence/CRLF | contract_v2_ / T-02，auto-case lint | v1行为不变，严格定位拒绝；payload原字节 |
| V-02 | AC-02 阶段 | source lex/parse/resolve/type错，HIR bind/verify错，工具缺失 | ac_stage_ / T-03、04 | 真边界code/span；infra永不满足语言negative |
| V-03 | AC-02 原生 | p01/04/12/13/15/16、scalar例、原始HIR | ac_native_ / T-04 | 真COFF/PE，exit/stdout/stderr逐项对照；非VM/A2R替代 |
| V-04 | AC-03 交接 | build金样FAIL/真产物可用、坏实际exe/好expected、错target/hash/nonce | ac_product_ / T-04 | 消费本轮actual且独立判定；伪造/过期产品拒绝 |
| V-05 | AC-03 执行 | 卡死/输出超限/双管、快速后代持管/Job失败、异常退出 | process_budget_ / T-03 | 到期杀全树并回收，bounded capture，不能截断PASS；infra≠exit70 |
| V-06 | AC-03 原始bytes | stdout含无效UTF-8/NUL/CRLF、空与未声明流 | native_bytes_ / T-03、04 | 比较原bytes，lossy展示不改变结果 |
| V-07 | AC-03 只读 | selector零/未知、未选check、越界/reparse输出、哨兵source/oracle | native_admission_ / T-02、04 | 先拒绝/未选零副作用，全部哨兵hash不变 |
| V-08 | AC-04 命中 | fresh→热点→新进程HIR/obj hit、跨工程 | native_cache_ / T-05 | 相应真实计数/禁调反控，HIR重verify；oracle每次重比较 |
| V-09 | AC-04 失效 | body/entry/profile/工具链/target改变、坏/缺块、改expected | native_invalidation_ / T-05 | 逐层应hit/miss明确；bad块拒绝/重算；改oracle不复用PASS |
| V-10 | AC-04 生命周期 | worker失败/取消、GC并发读、lease释放 | native_store_ / T-05 | live不误删，成功/失败根可追踪，旧临时根不无界增长 |
| V-11 | AC-05 syntax | 两grammar/坏源码/partial/error-missing/200步既有replay | syntax_case_ / T-06 | 当前树/来源、lossless及fresh等价；不授Checked、不调旧parser冒充 |
| V-12 | AC-06 周转 | 六场景≥30样本、参考环境、IPC/排队/发布 | turnaround_ / T-05、06、08 | §5.5两项P95<1s；完整raw样本/计数/资源，阶段和端到端分开 |
| V-13 | AC-06 修订 | 50次100ms间隔请求、逆序完成/取消 | revision_trace_ / T-06、08 | 最终发布属于最新revision，取消/积压可见，不删样本 |
| V-14 | AC-07 对账 | ≥24case及全部旧pilot，旧CLI门禁控制 | coverage_、compatibility_ / T-07、08 | 映射到实际checks/oracles；required缺失/skip/0checks非零 |
| V-15 | AC-01、07 报告 | stale的worker/工具链/源/oracle/协议/grammar，JSON/console | native_report_ / T-02、08 | 任一相关轴变化STALE；功能/性能分列且总体gate不能假绿 |

每项必验有真实执行正控和能区分错误实现的反控；不能只看报告退出码代替消费者校验。结构化观察入口需要mutation时证明实际变更发生。

角色：用户未指定实施者/审查者模型，不创建代理/聊天。work自检后由独立review在同一权威树顺序核验，暂停实施写入并绑定commit/依赖；自检不替代最终review。唯一预定树 `D:/autostack/.wt/lang-788/auto-lang`，分支 `plan-788-dev`，尚未创建；计划簿记留master，禁止链接。

工作只跑相关反例/模块与必要check，不能触发整档。最终独立review pass后，merge在冻结提交执行根 `cargo t`、触及共同test_cases/AVM/A2R分派的 `cargo tv`/`cargo tt`，以及worker完整套件、ac-core执行器/观察变更的独立触面套件和783/787受影响独立套件；实际触面由收据钉明。无aavm/UI/book/Schema文件修改则不跑taa/tu/tb/docs_gen；若范围修订命中则重判。tf仅按批量回归到期规则在主检出单实例，不作为本计划复审门禁。

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | v2显式严格扩展，v1不变，支持表/资源/报告拒绝准确；V-01/07/15 |
| AC-02 | work | core/scalar源码与原始HIR的真实检查/构建/执行、阶段与字节oracle正确；V-02/03/06 |
| AC-03 | work | actual交接、路径/selector、nonce/manifest、硬截止/输出限额/全树回收均有反控；V-04–07 |
| AC-04 | work | 真实统一store复用/失效/重验和生命周期正确，不缓存PASS；V-08–10 |
| AC-05 | work | D-787后双grammar/current revision/edit观测与fresh等价接入；V-11/13 |
| AC-06 | work | 每阶段trace/计数/资源齐全，§5.5两项参考负载P95<1s及原始样本可复验；V-12/13 |
| AC-07 | work | ≥24语义case/动态checks对账，v1/旧门禁保留，SD完整；V-14/15 |
| AC-08 | review | 同树独立逐条验AC/SD/遗漏延期/旁路，零未处理阻断，commit绑定；merge-owned pending明确 |
| AC-09 | merge | 冻结提交实际门禁、Spec/索引沉淀、合入归档、guard后清理有收据；真实计数10/10 |

## 8. 执行步骤

T-01是有限whole-path可行性门，不可用研究结论替代后续实现。D-787未满足不能跳过T-06后宣称交付。

| 状态 / ID | owner_stage | 依赖 | 操作/路径与结果 | 验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户执行指令 | 核树/基面/工具链，新增attachments/788/contract-decision.md与最小worker实验；冻结v2/IPC字段/支持表、阶段映射/预算/性能配方、787接缝清单 | [✅ 2026-10-16] 契约冻结见[contract-decision.md](attachments/788/contract-decision.md)（基面2f6b06c74/参考环境i5-13600KF+31.8GB/Win11/rustc1.98.0；787禁触面=ac-syntax/**+probe codec/schema/store_mod）；whole-path实验`experimental/ac-case-worker/tests/feasibility_t01_.rs` 3/3 PASS（source→COFF→PE→run exit3 oracle；typed bind.dangling-ref+span拒绝；fresh/hit计数1,1,1,1,1→0,0,0,1,1）。提交7e6d64fcb | AC-01–06 |
| [x] T-02 | work | T-01 | auto-test-case/src/{contract,suite,ir,compare,report,discovery,admission}.rs的版本门控与v2模型/比较/stale身份；旧v1恒定 | [✅ 2026-10-16] v2三schema精确分派+全字段面（hir-atom/ac×{check,build,run}/syntax×{parse,edit} decode受理·执行待D-787/typed stages/ac options含冻结预算硬cap/exit/perf/hir.atom-only artifacts/suite v2 defaults值校验+apply_defaults合并/NativeRecord报告）；v1接受/拒绝集字节不变（contract_decode 19 绿+wrong_schema改用case/3）。contract_v2_ 17例+v1回归30+19全绿；auto-lang --features test-cases 编译零错+test_cases_tests 26绿。提交a5e6e66a9（7文件1453行） | AC-01 |
| [x] T-03 | work | T-01、02 | 新experimental/ac-case-worker/{Cargo.toml,Cargo.lock,src/}；ac-core/source观察与link rawbytes/cap兼容入口；IPC/资源边界 | [✅ 2026-10-16] ac-core：`run_deadlined_captured/_with`（bytes内核+CaptureCaps+OutputCapExceeded立即杀树，旧split=lossy wrapper）、`SourceObserver/compile_parts_with_observer`（同管线边界归因，观察为parse/resolve/typecheck唯一来源）、VERSION常量；worker：`acworker/1` JSON-lines会话（begin/request/end，256请求/180s硬限）、request admission（schema/kind/profile/budget≤冻结cap/scratch绝对+拒reparse）、check/build/run全真实路径+canonical HIR产物+manifest哈希。`link_capture_` 7/7（deadline杀树/输出cap立即终止+精确边界/无效UTF-8·NUL·CRLF字节级round-trip对照lossy wrapper/contained拒绝重放）；`protocol_t03_` 10/10（三前端边界归因、typed bind.dangling-ref+span、真实死循环程序budget.run杀、admission负例、经真实二进制的完整IPC往返）；受影响ac-core lib/native_execution/cli 37绿；t01 3/3不变。提交5791ad12e | AC-02、03 |
| [x] T-04 | work | T-02、03 | 新auto-lang/src/test_cases/ac.rs，mod/materialize/stale及bin/auto-case.rs接分派/产物DAG/CLI；零静态Cranelift依赖 | [✅ 2026-10-16] ac.rs：worker定位（AC_CASE_WORKER env→仓布局release构建+进程内缓存）、单发IPC+父侧nonce/hash复核+墙钟杀树、父侧全判定（typed stage/code/message/exit/原始bytes/canonical HIR金样/声明perf）、manifest重核+128MiB cap+产物持久化注册；mod.rs：selector收ac/syntax、分派ac×{check,build,run}、syntax=点名D-787的admission ERROR（不SKIP）、ac/run product_from消费=manifest核验后**运行**上游exe（consume_and_run，不重编译）；Verdict增native记录→v2报告；ResolvedOracle把exit/perf纳入内容绑定身份（改期望exit即失效旧PASS）；stale增ac-case-worker轴；worker增HIR按名入口回退。`ac_case_tests` 7/7（exit oracle+红兄、typed stage归属PASS/错stage FAIL、真wrapper.atom HIR check+run exit5、syntax未绑定ERROR exit2、产物交接run+红控、v1载入拒ac、零匹配selector）；test_cases_tests 26+auto-test-case 84绿。提交7ea0ac0f9（8文件，格式噪音已剥离） | AC-02、03 |
| [x] T-05 | work | T-04 | worker的cache/trace adapter，必要的783 driver窄观察入口，不改GC；关联request→result计时与实际计数 | [✅ 2026-10-16] probe driver.rs窄观察入口：`DriverCacheMode{Reuse,Fresh,ReadOnly}`+`vertical_observed/vertical_and_run_observed`（同一参数化管线 vertical_inner——Fresh跳结果读取/ReadOnly跳全部put+roots/artifact列表标read-only标记；实测阶段span frontend/hir-restore/verify/lower/link/run/total），旧`vertical`保持Reuse wrapper；store/GC/codec/schema零改动（787禁触面）。worker：请求增 store_root/cache/project/include_hir，auto-source build/run经真实store路径（toolchain_fp=lld文件sha256+ac-core版本，非调用方常量），canonical HIR金样走仅观察不发布的前端、run腿留在worker保预算/cap/原始bytes，语言失败typed重放（driver渲染串仅展示）、计数按键合并。ac.rs：per-run store根（AC_CASE_STORE钉持久根）、cache模式随check options、NativeRecord承载hits/counters。`cache_t05_` 7/7（fresh→object-hit计数1,1,1,1→0,0,0,1且link/run仍真、fresh模式热库强制重算、body编辑全miss exit4、read-only缺根infra error、read-only命中消费+miss永不发布、typed重放无cache记录、每请求trace含run/link=PASS从不缓存）；ac_case_tests +跨check复用例 8/8；783回归 native_resume_5+invalidation_3+identity_3+concurrency_4+store_boundary_5 绿；test_cases_tests 26绿。提交54e6a46ca | AC-04、06 |
| [ ] T-06 | work | T-05、D-787 | worker/syntax adapter及v2edit观察，合入787后绑定真实grammar/API，当前revision/取消/重播/trace | syntax_case_、revision_trace_、syntax局部性能；AC-05、06 |
| [ ] T-07 | work | T-04–06 | 新test/cases/ac-native/*.case.ad、suite、legacy-map.tsv及独立oracle审计；至少24case，旧fixtures/pilot不改 | coverage_、v1/CLI兼容；AC-07 |
| [ ] T-08 | work | T-07 | 新scripts/verify-autodown-ac-788.ps1、docs/reports/plan788/矩阵/性能raw数据/命令、attachments/788/spec-delta.md | acceptance V-01–15、turnaround_；SD/范围/债务；完成→execution_done=8/10 |
| [ ] T-09 | review | T-08 | 同树独立最终审查，绑定commit/worker/toolchain/787依赖，重验反例与SD/遗漏 | AC-08；pass→reviewed=9/10，merge-owned pending |
| [ ] T-10 | merge | T-09 pass | 冻结提交跑§6门禁与closeout；沉淀SD/模块导航与ledger、合入归档、批量回归到期判定、guard后清理 | AC-09；实际收尾结束才10/10 |

代码修复改变冻结提交需重审受影响内容再执行受影响门禁。独立实验的完整套件归merge不意味工作期可以缺少V矩阵证据。范围外语言/ABI/CT或生产cache改动回规划，不以方便测试为由扩大。

## 9. 复审记录

2026-10-16：stage:new，PLAN-788 r1，outcome:pass（规划契约预检，非代码/性能验收），next:work；T-01–05可独立，T-06前核D-787。新增T-01–10、AC-01–09、SD-01–06、V-01–15；0/10，未执行实施/测试，未创建树。

### 2026-10-16 work handoff — T-01–T-05 完成（用户授权的独立部分），T-06 阻塞于 D-787

stage:work | PLAN-788 | r1 | outcome:pass-scoped（T-01–05 全绿；**非 execution_done**——T-06–08 未做，全部必验完成才可 execution_done） | worktree `D:/autostack/.wt/lang-788/auto-lang` | branch `plan-788-dev` | base `2f6b06c74` | code `54e6a46ca`（T-01 `7e6d64fcb`/T-02 `a5e6e66a9`/T-03 `5791ad12e`/T-04 `7ea0ac0f9`/T-05 `54e6a46ca`，历史干净无格式噪音） | deps: ac-core/ac-artifact-probe@master 基面+本树扩展、auto-down 兄弟 worktree detached `c39ac7f`（组内解析用，零改动） | tasks: T-01✅ T-02✅ T-03✅ T-04✅ T-05✅（5/10） | evidence: worker `feasibility_t01_` 3/3 + `protocol_t03_` 10/10 + `cache_t05_` 7/7；ac-core `link_capture_` 7/7 + 受影响 lib/native_execution/cli 37 绿；auto-test-case 84（含 contract_v2_ 17）绿；auto-lang `ac_case_tests` 8/8 + `test_cases_tests` 26 绿；783 回归 native_resume_/invalidation_/identity_/concurrency_/store_boundary_ 20 绿；契约冻结 attachments/788/contract-decision.md（含 §9 实施修订：hash回显、project/include_hir 字段、driver String 流不参与、本批 V-06 非 ASCII 生产者随 text 批、跨进程重启归属） | blockers: T-06 需 D-787（787 独立 review pass+门禁+实际合入 master+Spec 沉淀+merge 收据；其 needs_fix 重开中）——解锁后绑 syntax adapter 与 V-11/13，再做 T-07 ≥24 case/T-08 验收聚合 | next: D-787 满足后 work 继续 T-06（同树）；merge-owned 门禁（裸 t、tv、tt 触面、worker/probe/783 独立套件、closeout）全部 pending——按 2026-10-14 时序裁定，最终 review pass 后由 merge 在冻结提交统一执行

来源/预检见[planning-preflight.md](attachments/788/planning-preflight.md)。契约决策见[contract-decision.md](attachments/788/contract-decision.md)。亚秒门槛是本计划拟验收合同，T-05 交付的是 trace/计时接线；六场景×30样本实测与 P95 判定归 T-08（参考环境已冻结于决策稿 §1/§8）。

## 10. 待澄清事项

无阻断起草的用户决策。T-01需在既定范围内冻结准确v2字段/worker IPC、真实阶段边界观察、执行器bytes/cap扩展、参考设备/预算分配与样本配方。总阈值/支持面/安全边界不能暗改；失败带最小反例回规划。

D-787是依赖实施门，不能以其在途API、截图或局部测试替代实际合入。后续独立工作包仍包括Design42语义meta/推导/CT、有限AutoABI/FFI、语言enum/List/use；本计划只为它们提供统一可信测试、缓存和周转观测入口。
