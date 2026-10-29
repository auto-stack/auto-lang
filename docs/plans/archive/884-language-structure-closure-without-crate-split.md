---
plan_id: PLAN-884
status: archived
feature_name: 语言内部结构收口（保留crate粒度与既有行为）
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
supersedes_spec_components: [docs/specs/language-sdk/architecture.md, docs/specs/language-sdk/project.md, docs/specs/auto-lang/runtime/overview.md, docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/language-sdk/structure-closure.md]
touched_goals: []
affects: [language-sdk, auto-lang/runtime, auto-lang/trans, auto-lang/vm, stdlib, test-infrastructure]
depends_on: [PLAN-869, PLAN-872, PLAN-873]
current_step: 10
total_steps: 10
---

# [PLAN-884] 语言内部结构收口：运行桥接、stdlib装配与Rust转译职责

> 整Plan前置 G884 = U869 ∧ U872 ∧ U873；本轮已只读核验其完整交付收据。
> 源码写入另须 X884（§5.1实际写面交接），本次授权仅立项，**不实施、不建树、不创建agent**。
> 用户已确定按两个大工作包收口；本Plan为第一包，后继UI/Man结构包仍未编号，全部实施任务等待U884。

## 0. 变更摘要

873已实测并裁定**暂不细拆crate**：真实函数编辑check中位约语言6.9s、UI4.6s、Man1.2s；编辑后no-run约27.4/22.9/4.9s（后两组n=2）。这些不是拆分收益实测。本Plan据[范围决策](../reports/post-873-structural-work-packages-2026-10-29.md)保持当前成员与默认特性，以职责/装配/状态单源收口843旧残项和869登记热点。

交付范围顺序：当前职责和消费对账 → Core运行桥接 → stdlib装配 → call_inner顺序审计及主题提取 → Rust转译主管线；HTTP入口已壳化部分只核实际残项。真正的产物是生产调用切到职责明确的内部模块，旧重复装配/桥接实际收敛、全部原行为和消费契约保持，不能仅提交目录规划或文件切片。

## 1. 目标

- G1：范围和owner清楚。逐项区分843原目标/实际交付、869旧残项/已解决部分及本Plan新增结构目标；测量pin与开工基线差额可解释。
- G2：同步API到专用执行线程/LocalSet/异步执行的同构桥接单源，入口策略显式；签名、返回、线程亲和、栈、stdin、path、deadline、panic和trap分类保持。
- G3：stdlib按域组织注册、装配和实现，872名字/ID/覆盖序/生效实现及六族差异保持；fingerprint随源码owner移动。
- G4：call_inner按可审计的调用形状组织，首匹配、early-return、错误和写出顺序保持；主管线只负责阶段编排，算法归具名主题模块。
- G5：SDK/Core、上层兼容入口和UI/Man/ACE实际消费者在固定版本继续可用，测试ID零未解释缺口、新增红=0；规范与最终交接闭环。

**非目标**：不拆crate、不改变default/features/manifest依赖或ABI、不实施873 A1（默认ui-iced调整）、不承诺提速或净删≥2,000；不重写parser/typecheck/VM算法、不迁ACE主体/引擎、不切默认、不删除被遮蔽内建、不修历史行为债、不重建UI/Man/OS架构、不跑新平台移植或全应用治理。保持原支持资源边界，不增加新的源码长度/深度/集合大小限制，也不承诺无限资源。发现旧缺陷按owner登记，改变语义/支持面需单独裁定。

## 2. 架构方案

保留 `auto-lang-core` 单一语言实现、`auto-lang`组合/再导出和既有中立基础包。复用当前 `entry.rs`、`ffi/stdlib/*`、`trans/rust/*`；新增内部主题模块由T-02冻结名字和完整职责，明确为**拟新增路径**，不创建common万能包、第二份类型系统或第二套转译器。

1. 入口层：公开函数维持旧策略，复用最小内部桥接helper；只合并已证明相同的机制，不把staged trap通道与普通错误通道合成不同语义。
2. 内建层：同一NativeInterface继续catalog→inventory→具名override的实际装配序；域文件持有真实shim/辅助算法，入口持有受审计的组合，不改变依赖/初始化副作用。
3. 转译层：同一个RustTrans、TypeStore/Database和Sink；shape helper的“不适用”“已处理”“错误”必须可区分，不能把旧early-return转换成继续走后续臂。`call`外层收集引用后写出整个临时缓冲的顺序保留。
4. 发布层：源码输入manifest/loader与source pin测试随模块迁移；公开再导出路径恒等。Core正常闭包不新增UI/Man依赖。HTTP保持已完成的server_*单源，本轮只作核销与必要路径适配。

机制选择为有界实施决策；T-02须用基线oracle证明最小全路径可行后，才进入对应生产提取。不允许先机械切块再用批量改golden自证正确。

## 3. 技术栈

现有Rust/cargo、Tokio LocalSet/专用执行线程、inventory/rust_fn、NativeInterface、Trans/Sink、stdlib_assembly manifest/loader；Python仅用于已有度量/台账或必要局部复算。复用867测试ID规则、872绑定探针和现有SDK/三仓consumer。不上新依赖、调度器、链接器或缓存后端。

## 4. 需求分析与背景调查

### 4.1 授权、基线与来源

2026-10-29用户明确要求“根据这个范围，为第一个语言结构工作包建立正式计划，明确验收标准和与ACE在途计划的交接边界，暂不实施”。本轮只允许主检出计划/设计/路线簿记；保留其他WIP，不改产品/canonical/ledger，不取第二包编号、不建实施树/agent、不运行Cargo/GUI，不推送。未来work仍等待明确启动指令；本Plan不将规划授权升级为实施授权。

**执行授权（2026-10-29，/goal 完成计划884）**：用户经 /auto-plan-goal 目标下达明确执行指令——G884 视为已取得，授权链覆盖 work（建树/探针/测试/实施）、review、有界修复与 merge。权威树 `D:/autostack/.wt/lang-884/auto-lang`（分支 `plan-884-dev`，基面实际重绑 `d9718cc63`，主检出代码路径预检零 WIP）。X884 仍按 §5.1 逐写面核记录，不以本授权替代交接记录。

读取基面：语言master `643d58d5b`；UI master读取时 `a9e8d70`（生产主体包含873 pin `4adf2d03`，随后为计划归属簿记）；Man `e6cc06f`；down `c3464d0`只读。work须重新绑定实际HEAD、锁、配置及修改范围，不消费主检出未提交docs或他方WIP接口。

| 来源 | 本轮事实/限制 |
|---|---|
| [Spec总览](../../specs/overview.md)、[索引](../../specs/INDEX.md) | 导航中有搬仓前历史描述；以现行language-sdk/UI/Man Spec及实际交付校正，不按总览旧目录写代码 |
| [language-sdk架构](../../specs/language-sdk/architecture.md) | 单owner、入口组合、公开路径恒等；call_inner须先审early-return，不能机械分片 |
| [runtime](../../specs/auto-lang/runtime/overview.md)、[trans](../../specs/auto-lang/trans/overview.md)、[VM](../../specs/auto-lang/vm/overview.md) | 一些行数/旧Scope描述滞后；本轮记录差额，merge仅回写本范围，不改无关历史 |
| [869](869-crate-inventory-and-subtraction-batch-2.md) | 已交付清单及七热点材料；S3实际主要在core/entry；S1对U871的全局绑定不能沿用 |
| [872](872-builtin-semantics-dual-track-ruling.md)、[stdlib Spec](../../specs/stdlib/project.md) | 六族DISTINGUISH、覆盖序和字节/字符/错误差异为保持对象，不是本Plan统一目标 |
| [873](873-build-cost-and-member-split-decision.md)、[基线](../../specs/repository-boundaries/build-cost-baseline.md) | 8/8已交付，54f4ae6e9/ledger v127/643d58d5b guard收尾；不拆、A1另签核、缺测限制仍有效 |

读取内容SHA-256：language-sdk/architecture `1ce06d43be93e4764cb5305118fc058626ef5963e02e804307a22239c15559a4`；language-sdk/project `21d4d83176d7834f1ca7bd5d024c7ba1caa3f3180906bed2249d6f343175c419`；trans/overview `77eed843470b9026793eaec549d67dcfb808556dc87cddb3d7ba54fd465b85f9`；runtime/overview `c3df7257c7ba707d453b7eda98ce27a227dbe108cb07db240ce13f3a95425228`；VM/overview `a0c6154934923054975b6809cd517d025e37bc4995639ae041199330187efc5d`；stdlib/project `6b4818b589e2681720ceaad4a78c1409af36e2d01df817d056631a63d967d3a0`；873基线 `061464e7ee8a4f223fe680dd5f6a1c9c8e20e7045135e2646e63c33a37867dd6`。

### 4.2 实际源码观察（静态，不是未来验证通过）

| 已存在路径（auto-lang仓） | 观察与本Plan边界 |
|---|---|
| `crates/auto-lang-core/src/entry.rs` | 3263总行；run/path/staged/bytecode/meta/input/deadline、join_execution、run_autovm族；还含config/FFI/文件测试等无关职责，不将整个文件重写 |
| `crates/auto-lang-core/src/lib.rs` | `mod entry; pub use entry::*`，并有execute_autovm*真实异步入口；仅必要模块/再导出适配，执行算法不重构 |
| `crates/auto-lang/src/lib.rs` | 上层公开再导出和测试注册，属于与ACE可能相交的组合根，默认只读 |
| `crates/auto-lang-core/src/vm/ffi/stdlib.rs` | 8183总行、已有8个域；RequestFileRootGuard/线程本地根与真实shim仍活跃，按职责收口，不强追行数 |
| `crates/auto-lang-core/src/vm/native.rs` | production/register_std_shims/build_from_inventory定义装配；允许最小路径适配，注册算法、顺序、名字/ID不能变 |
| `crates/auto-lang-core/src/trans/rust/call_emit.rs` | 7553总行；`call`先向Vec发射，收集引用，再写out；call_inner巨型分支含提前返回/多层写出，必须逐块审计 |
| `crates/auto-lang-core/src/trans/rust.rs` | 7782总行；RustTrans+Trans::trans及后处理/收集helper，已含6个子模块；按阶段延伸既有边界 |
| `crates/auto-lang-core/src/vm/ffi/http_server.rs` | 5145总行含内联测试，869样点生产约186行；先核实壳化及测试归属，不按总行数判未收口 |
| `crates/auto-lang-core/src/stdlib_assembly/{manifest,loader}.rs` | 已存在指纹输入/装配实现（T-01按真实文件核具体符号）；迁移需增补输入，不改变缓存/信任政策 |

当前物理行数仅作定位，不是目标阈值；最终用同口径生产/测试/生成/第三方分列及职责闭合判断。843旧自设行数目标如需变更，T-01必须在对账中显式处理，不静默宣称旧合同已完全达标。

## 5. 详细设计

### 5.1 G884、X884与ACE实际交接边界

G884覆盖全部实施准备/建树/探针/测试。本轮上游收据已齐，**执行授权仍未给出**。未来T-01在指定执行后可只读调查并建立本Plan权威树；T-02前取得X884，未获得的共享写面只等待该scope，不持窗等待无关上游。

| 写面/任务 | 884权属 | ACE相关owner及交接要求 |
|---|---|---|
| core/entry运行桥接、ffi/stdlib及其子模块、trans/rust主题模块 | 本Plan结构写集R；已有算法/公开身份保持 | 863/865/871/881不因“native/函数”同名获得这些产品实现的写权；若实际调用/路径迁移相交，逐符号登记后先交付者释放→后者刷新接受 |
| experimental/ac-core/source、HIR/descriptor/verify、auto-mir-lower、ac-av-probe的AV/session/cache | **排除写入**（S域）；不接管profile/预算/会话协议 | 863 r2、865 r2、881 r2及871删除窗沿[ACE合同](../reports/ace-wave-handoff-2026-10-28.md)继续单写；884只读其正式接口，不复制WIP/类型/lowering |
| experimental/ac-mir-native、ac-case-worker及native CLI | **排除写入**（N/driver域） | 871 r4与881按各自G-COV/G-DELETE/方法窗交接；U871/U881不是G884全局前置，884也不替它们认领旧native能力 |
| `crates/auto-lang-core/src/test_cases/**`、auto-test-case协议、`ace_m1_tests.rs`、binding.tsv、公共worker/schema | **默认只读，禁止顺手改oracle**（H域） | 863/865/871/881的真实H写面按窗口；本Plan兼容回放优先复用已注册用例，不改ACE expected/NA/accepted-diff/来源身份。发现必要适配先列具体符号/owner并修改X884记录；改变协议/语义超范围则needs_replan |
| core/lib.rs与root auto-lang/lib.rs的模块/再导出/测试注册 | 仅必要最小适配（H/R交界） | 新测试优先归所属内部模块或独立integration文件，避免争抢公共mod块；确需同块写入时核原owner已完成review/repair、合入和释放后再接受；不能凭Git可自动合并认定安全 |
| stdlib_assembly manifest/loader、源码hash/SDK包清单、共享测试台账 | 迁移输入更新属本Plan必要写面I | 与消费源码指纹的ACE SDK/生成/发布owner列真实producer/consumer并集；迁移后的文件/路径/hash/版本可消费后释放，后合者重核指纹与相应case。不得把旧hash保持当兼容，漏新文件也不得假绿 |
| master/Cargo.lock/manifest/规范索引/受管ledger | master/ledger仅merge有序写入；Cargo依赖/feature默认不改 | 后合者核源码/依赖/配置/测试输入并集，代码基面变化使证据失效则受影响review/门重跑。不能覆盖其他路线WIP |

**X884交接记录的最小字段**：前owner/Plan修订与范围 → 已合入SHA、源/锁/接口及profile或指纹版本 → 具体释放文件/符号 → 884接受SHA和差额 → 下游真实消费者/兼容用例 → 剩余限制、证据失效与恢复方式。没有owner响应的纯读路径不视为已取得写窗；无共享写集时允许“无重叠+路径依据”的明确记录。交接仅经现有计划/受管记录或用户授权通信，不自行创建或向其他聊天发送消息。

在途快照：863/865/871/881均drafting，864/866/870已归档；不依据状态头假设没有实施写入，T-01须再核所有权威树/diff及最新合同。881默认S队列864→866→881→871删除就绪插窗→865→863不被本Plan重排。R域可独立推进；只在实际R/H/I重叠处短窗串行，跨域同时取所需窗，不持一个等待另一个。

**边界反例**：871退役 `experimental/ac-core/src/native.rs` 的HIR→CLIF；884整理 `crates/auto-lang-core/src/trans/rust/**` 的Auto→Rust源码生成。前者完成不意味着后者可删，前者未完成也不阻塞全部884；旧AutoVM/a2r仍承担生产。

### 5.2 当前基线与有界决策（T-01/T-02）

T-01用已提交新基面重读§4范围与873 pin差额，冻结：run入口逐函数策略表、stdlib定义/注册/消费者图、call_inner**全部顶层分支及嵌套early-return**位置→形状/顺序/状态写入/错误/输出图、trans主管线阶段图、HTTP壳/测试归属、API/ID/源码指纹/实际依赖边。产物为Markdown摘要与本地机读清单，不用源码#[test]计数替代测试枚举。

T-02对桥接、注册、call_inner→完整生成/执行三条真实路径各做最小可恢复探针，先冻结未改基线观察再提取；复用现有用例，缺少的区分性用例由T-02/T-03..07补为长期测试。现有错误行为原样保留，计划不裁定更好的语义。对不能安全提取的块须给具体机制/证据并保持任务未完成，必要时needs_replan；不能以“保留巨型链”免除G4。

### 5.3 运行桥接（T-03）

至少覆盖当前run_autovm/run_autovm_capture、staged path、bytecode/meta及path/input/deadline变体；run/run_with_capture的ExecutionEngine选择逻辑保持。冻结表需包含：capture、path所有权/相对导入、stdin None/Some空/非空、deadline None/已到/运行中到期、专用线程/LocalSet、栈策略、结果/元数据、panic→错误、trap typed code及TLS取走时点。

共同机制只建一份，不改变`vm_thread_stack_size`默认/覆盖值、不把非Send VM移出原线程、不引入嵌套runtime/block_on。保留`join_execution`和公开路径，`StagedRunFailure.run_trap_code`来自原typed VM trap通道而非解析错误文本；panic和基础设施错误仍为None。helper不吞错、不将deadline变成功、不承诺新增即时取消能力。异常和正常退出都恢复请求本地root/输入，原线程/句柄释放合同保持。

本范围不要求搬走entry.rs中无关Py/Rust FFI、配置和文件测试引擎；其调用桥若发生实质改变则纳入受影响消费者，不以同文件为由略过。

### 5.4 stdlib装配（T-04）

按域放置真实shim/数据和配套注册，将已有8域先复用，入口仅保留受审计的状态声明、轻包装与组合。RequestFileRootGuard/TLS、静态表和全局map仍各一个owner；函数模块可见性仅最小pub(super)，不扩公网pub，不复制第二份表。

正常production、catalog和inventory的不同实际路由全部保留；对872绑定指纹不能只比较名字集合，还要逐名ID、同ID覆盖序、最终handler/结果/错误一致。遮蔽math/parse_int/time候选仍不退役；str.length/len及CALL_SPEC差异不统一。宏展开来源路径变化若影响生成名/顺序，必须修回原行为而不是更新基线接受漂移。

### 5.5 call_inner与主管线（T-05/T-06）

- 先完整审计再分主题，顶层`call`保持缓冲→UTF-8转换→serialize_positional_args→collect_core_reference→out.write_all的当前顺序；其失败时哪些状态或输出已产生也保留。
- 每个新helper必须表达未命中/已处理/失败区别；不能用`Ok(())`同时表示未命中和已发射，不能在已发射后继续调用通用fallback。保借用/类型/导入上下文以及对RustTrans标志的写入时序，不额外求值实参。
- 具有重叠匹配条件的族保首匹配序；如裸diff/frame函数、外来Value构造/泛型、type方法、常规函数、泛型单态化及通用fallback，按基线全部分支登记覆盖。每个迁移块有原文/规范化差异核对及形状反例；不可达/尚无oracle块不能静默丢失。
- 主管线复用现有expr/stmt/decl/ty/project_fixups，提取真正的类型/导入/模块壳/后处理职责；同一算法仅一份，原大入口不保一份旧实现作永久fallback。类型、借用、错误阶段、source map、生成源码/导入/资源/引用证明保持。
- HTTP仅核旧残项已解决/仍需最小装配适配；不改路由、字节级reply、端口、超时或并发政策。不为缩总行数单独外移全部HTTP测试。

### 5.6 守恒、消费者与健康（T-07/T-08）

公开fn/类型/字段/再导出路径集和状态owner不变；新helper默认私有，既有必要域可见性变更逐项解释。旧实例不引入复制/move/释放差异。各入口仅保轻装配/派发，算法模块职责须在owner表逐项有唯一归属，不用include!分片、空re-export壳、巨型common或仅按行切块“收口”。不新增专为绕过审查的fallback。

所有移出源码进入`stdlib_assembly`实际输入闭包：删除/旧路径引用清理、新路径清单与fresh/变更后cache失效及恢复decode/校验沿原合同；不改全局cache策略。API/SDK consumer、测试ID迁移/retired账本、golden与原始oracle保持；不删/ignore/改expected使门变绿。新增红为0，既存红逐ID/环境有证复核，必需正控未执行不能pass。

健康：只对本变更新增/触面的警告要求清零，预存警告冻结归因不冒称全仓无warning；fmt/diff clean、无stray debug print、遗漏/延后/workaround扫描写KNOWN-DEBT，合同内缺项不能仅转债宣布完成。代码/依赖/配置/输入与证据绑定冻结SHA。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/structure-closure.md | 无本次有限结构收口合同→最终职责/入口/owner、行为守恒与ACE边界 | 记录实际完成范围及未改语义 | AC-01..AC-07 |
| SD-02 | modify | docs/specs/language-sdk/architecture.md | 843残留与旧路径/行数→新职责DAG、唯一状态及已完成/保留项 | 以结构事实沉淀，不伪造性能收益 | AC-02..AC-06 |
| SD-03 | modify | docs/specs/auto-lang/runtime/overview.md | 重复run桥/历史lib位置→实际桥接owner与错误/线程/guard合同；只改本面滞后内容 | 可追溯生产生命周期 | AC-02, AC-06 |
| SD-04 | modify | docs/specs/auto-lang/trans/overview.md | call_inner暂留→经审计的shape/主管线边界及输出契约 | 不误认871退役A2R | AC-04, AC-05, AC-07 |
| SD-05 | modify | docs/specs/auto-lang/vm/overview.md；docs/specs/stdlib/project.md | 旧装配路径→新owner/指纹输入，872生效绑定与语义保持 | 模块迁移不改注册行为 | AC-03, AC-06 |
| SD-06 | modify | docs/specs/language-sdk/project.md | 结构描述/THR-D1决策链接→884实际职责及消费者交接链接 | 连接873不拆结论与结构交付 | AC-01, AC-06, AC-07 |

SD均为proposal，work不发布canonical；review核其是现行交付描述，merge落地。touched_goals为空：本Plan不重定义GOAL或认领ACE/UI架构新能力。

## 6. 测试设计

实现与内部自检由未来被指定执行者完成；final review需独立上下文、同权威树只读核验，内部pass仅表示可送审。用户未指定模型/agent，本Plan不创建或联系任何agent。merge负责冻结门/沉淀/合入/归档清理；review不代修代码。

| Case ID | AC/invariant | 输入/反例 | 真实入口与oracle | 覆盖边界/owner_stage |
|---|---|---|---|---|
| V-01 | AC-01/07 范围与交接 | 当前四域、已解决HTTP壳、空写集正控；缺owner/未合入定义/跨窗漏记录负控 | T-01对账+T-02 X884记录，逐路径/符号与实际diff；不得开始未释放写面 | 全部实际写集；work |
| V-02 | AC-02 入口结果/通道 | 基线成功值+stdout、path相对use、bytecode/meta、input None/空/非空 | 公开core API及root再导出；冻结程序的结果/输出/元数据逐字相同、input区分保持，原基线有文件case直接复用 | 受影响入口逐一映射，不以一条run代全族；T-02/03 work |
| V-03 | AC-02 线程/失败/释放 | cooperative deadline已到/运行中；实际执行线程panic；staged typed trap与parse/基础设施失败；先失败后正常请求 | 原生产线程及LocalSet路径，join_execution/StagedRunFailure；失败类别/消息/typed code、线程结束和请求本地root恢复与基线同值。超时有测试自身截止，不能以未触发poll的空程序冒领；对照不同path串行请求 | 不新增即时取消语义，不实际造OOM/spawn耗尽；该类不可注入限制+相同调用序源码审计；work |
| V-04 | AC-03 绑定全量 | 872全量绑定/六族正控；在本树临时反控交换覆盖顺序或漏名字 | 复用plan872_builtin_binding_probes/semantics_tests、JSON指纹守卫；在各实际catalog/production/CALL_SPEC路由核名字/ID/生效实现/返回错误；反控至少一项变红且复位hash | 不把peek当所有路由；T-02/04/07 work |
| V-05 | AC-04 顺序/副作用 | T-01所有迁移分支；至少一个可命中两候选的优先序对照、一个未命中fallback、一次有副作用实参 | 真RustTrans::call→call_inner；基线生成bytes、状态标志与调用轨迹一致；早命中后未继续fallback、每参求值次数/顺序保持。安全隔离反控反转一处匹配序或强制漏handled应红 | 逐迁移块有审计映射；已有源码不可达者由源码保持证明+限制，不省去块；T-02/05 work |
| V-06 | AC-04/05 失败与写出 | 有效调用、发射返回错误、最终writer受控失败/部分写入、引用收集失败（有真实可构造路径时） | `call`临时Vec→UTF-8→args/reference→writer，比较原错误阶段/已写prefix/副作用及输出；不可构造失败不能伪造成功，记录可验证代码序与限制 | 不改原输出事务语义；T-05/06 work |
| V-07 | AC-05 A2R全链 | 每个形状族至少1有效源码，重叠/构造/方法/泛型/正常fallback/导入资源组合及非法输入 | 使用同一原源码、独立expected；trans_rust[_with_session]/Sink生成bytes、导入/资源清单与baseline等；代表生成工程真实Rust编译运行核stdout/exit。非法面保持拒绝阶段/诊断，不以parse成功代替可运行 | 复用现有a2r语料/golden，新增场景T-02定；不新许诺任意外来库；work |
| V-08 | AC-03/05 HTTP与guard | plan755字节契约+plan781预算/请求root相关定向面；HTTP壳引用变化时正/错误/关闭请求 | 真实接口输出/状态/headers/body及guard恢复一致，只有HTTP实际触面才增加相关TCP正控 | 未修改功能不新增平台门；T-04/06/07 work |
| V-09 | AC-06 指纹与测试守恒 | 新模块源码更改/漏输入反控、fresh/hit/旧缓存恢复、测试路径移动 | manifest/loader实际读源码，plan738/716相关钉源测试+SDK生产consumer；漏掉一个迁移源必须可被清单或旧fingerprint错误观察拦住；测试枚举ID逐项对应，选择0非pass | 迁移源全清单，hash变化不能改expected掩盖；T-07 work |
| V-10 | AC-06/07 外部消费与ACE兼容 | Core独立/上层组合/三仓固定consumer、ACE已交付adapter代表源码/错误case | language-sdk-consumer、three-repository-consumer和受影响ACE现有命名case；签名/类型单源、stdout/诊断/报告身份与旧值相同，零新反依赖；ACE WIP不参与证明 | 受影响接口逐条owner映射；不额外验全部M1或所有组合；T-07 work |
| V-11 | AC-08/09 健康与关闭 | 全任务/AC/SD、实际提交/依赖/收据；忽略样本/合同缺口负控 | 独立review逐条重构+merge最终态读回；未验项不勾、merge门pending明确；最后guard clean与真实remove/分支状态 | review/merge分阶段，不用全档替代定向证据 |

**未来验证命令**（不是本轮已执行证据）：

- work最小检查：`cargo check -p auto-lang-core -p auto-lang`；`cargo check -p auto-lsp`在公开再导出触面时；受影响源码执行rustfmt检查和`git diff --check`。
- 既有定向锚：`cargo t plan872_builtin`、`cargo t plan755`、`cargo t plan781`、`cargo t plan738`、`cargo t plan716`、`cargo t plan746`、`cargo t plan608`，仅跑实际受影响者，记录选择数>0；新用例归owner inline模块或拟新增`crates/auto-lang/tests/plan884_language_structure_contract.rs`，独立integration显式`cargo test -p auto-lang --test plan884_language_structure_contract`，不能因裸t只选lib而漏掉。
- Core独立consumer：`cargo test --manifest-path test/fixtures/modularity/language-sdk-consumer/Cargo.toml`；三仓consumer同路径替换为`three-repository-consumer`，仅在固定组与资源准备齐时执行；必须核真实crate身份，不能借其他计划target/rlib。
- A2R work用已有`--features test-trans,test-cases`的**命名filter**或所属单元测试，禁止用裸tt整档代定向oracle；具体真实fixture/命名case与命令由T-02冻结且先证选择数，不虚构已跑测试名。

**最终门禁**：T-09 pass后T-10在冻结提交统一裸`cargo t`＋`cargo tv`＋`cargo tt`（运行桥/VM与trans均触面）。新增integration及实际消费者的定向门不得因整档通过而漏掉；相同源/锁/config/input/选面证据可复用并证明适用。UI/Man源码只读不跑两仓整档，但真实consumer/必要定向兼容仍须在work完成。tb仅真实book触面；语言tu不存在，UI生成器零写面不触tu；taa仅AGENTS列出的AAVM路径触发，本计划默认排除；docs_gen不因写Spec而触发。th依实际HTTP触面单独判定。tf批量按到期由主检出单实例，不在本树重复开重档。新增红须归因修复并受影响重审，新冻结后跑受影响门；既存红/环境限制实名留证，不更改原始退出码。

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | T-01四域/843旧残项/HTTP已解决部分/873输入差额逐项有结论，现有成员/默认特性/生产支持面保持；V-01及X884真实写集无未取得交接项 |
| AC-02 | work | run入口策略表全部受影响路径有V-02/03证据；同构机制单源、公开签名/结果/stdout/meta/错误及typed trap分类、线程栈/LocalSet、input/path/deadline/退出guard保持；非同构策略不误合 |
| AC-03 | work | stdlib算法/注册按域归属，入口只装配；V-04/08成立，872全绑定指纹/路由和优先级不变、六族零行为变化，状态/guard唯一，零未授权RETIRE |
| AC-04 | work | call_inner全部原块/early-return审计映射齐且生产分派按职责实拆，首匹配/handled/error/fallback及输出/副作用顺序保持；V-05/06正反控有效，无机械切块或旧算法永久副本 |
| AC-05 | work | trans主管线主题职责明确、真实消费者接线，V-07生成bytes/资源与运行oracle保持；HTTP壳核销与必要V-08闭合；没有把输出漂移转golden更新或将未完成项无授权延期 |
| AC-06 | work | V-09/10、API/状态/依赖与ID映射保持；迁移指纹无漏、新增红0、必需正控均执行；fmt clean、触面新增warning0、无debug输出；迁移/净删/helper分列，不凭行数宣称提速 |
| AC-07 | work | ACE释放/接受/回交与并集记录齐，保护域无越界；SDK/消费者指纹和SD-01..06/handoff冻结，内部核查逐项有可审证据；不改ACE语义/分母/默认、不借WIP |
| AC-08 | review | 独立终审在同树冻结SHA重验AC-01..07、V-01..10、SD及遗漏/workaround/债；结论pass且明确merge-owned门pending；无法检查的必需面不能pass |
| AC-09 | merge | 冻结裸t+tv+tt及必要定向/触面门通过或按已确认基线具名归因；并集变化受影响复审/重验，实际合入/规范ledger读回/归档/guard清理齐才U884；给后继正式版本及限制清单 |

## 8. 执行步骤

10项=8 work＋1 review＋1 merge；全部目前未执行。G884对所有行有效，源码/注册/fixture写入须T-02完成X884。每个探针属于本Plan，不另开编号。

| 状态 | ID | owner_stage | 依赖 | 路径/交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G884；用户明确执行指令 | ✅ 2026-10-29 五路只读审计落 attachments/884/t01-{baseline-freeze,audit-entry,audit-stdlib,audit-call,audit-rust-http-api}.md；873差额=0（643d58d5b..d9718cc63 钉定路径全空）；X884=881树diff实测零R域交集+863/865/871 drafting 无树（无重叠+路径依据记录）；权威树 lang-884 @ d9718cc63，兄弟钉 auto-ui d5c7b86/auto-down c3464d0a/auto-man e6cc06f detached 只读 | AC-01, AC-07 |
| [x] | T-02 | work | T-01；X884实际释放/接受 | ✅ 基线探针全绿于未改基面：plan746:9/872:13/738:75/716:14/608:2/755:4/781:13/a2r-golden:405；plan884 oracle（crates/auto-lang/tests/plan884_language_structure_contract.rs 11例，先冻结基线观察后钉断言）；模块设计冻结=run_bridge单源+stdlib五新域文件+注册按域拆分+call_emit按深度0语句边界22族tri-state+rust.rs主题模块 | AC-01..AC-05, AC-07 |
| [x] | T-03 | work | T-02 | ✅ e358830d2：entry/run_bridge.rs 单源 G1/G2/G3（capture/path/deadline/input 显式参数）；G4 选择器/G5 staged 独立不合并（TLS take 留闭包内）；验证 plan746:9+oracle:11 绿，check 零新警告 | AC-02 |
| [x] | T-04 | work | T-03 | ✅ 10ba01657：stdlib.rs 8184→2820行；request_root/storage/exit_audit/http_async/rust_stdlib_dispatch 五新域+域全局归位；register_stdlib_ffi 582语句行→19域helper保序调用；指纹闭包（loader+manifest）增补；V-09递归守卫测试新增。反控：NC-1漏列→守卫红点名/复位绿；NC-2d变异生效shim→plan872_math红/复位绿；序互换实测中性（catalog与inventory fn身份已同一，native.rs:1700注释为前史）。验证 872/738/755/781/746 全绿+a2r 405 | AC-03 |
| [x] | T-05 | work | T-04；call形状审计已冻结 | ✅ 82499c0cb：call_inner 7436行按**深度0语句真实边界**（扫描器验证，审计B14/15/16实为一Bina守卫内阶段、B20嵌于Dot块）拆22族入 call_emit/ 九主题文件；CallShape{Miss,Handled} 显式三态；体逐字仅改return形态；指纹闭包增补；plan716源扫描面随owner扩展（843先例）。反控 NC-3 换序→a2r红/复位绿。验证 a2r 405/405+716:14+608:2+872:13+oracle:11 | AC-04 |
| [x] | T-06 | work | T-05 | ✅ rust.rs 7783→~4290行（编排留驻）：post_fixes(27 fix+derive mask)/ast_scans(扫描族+共用收集器)/merged_fixes(merged后处理+regex包)/decl_emit(4遗留发射器)/http helper→call族；收集器去重=spec_params逐字同+str_params仅注释差→单份，param_types**语义分叉**（merged多merge-mut收集）保持双份入KNOWN-DEBT；HTTP壳T-01审计核销零迁移。指纹闭包增补（守卫红→绿）。验证 a2r 405/405+全定向族绿+警告=基线131 | AC-05 |
| [x] | T-07 | work | T-06 | ✅ SDK consumer 5/5、三仓 consumer 3/3（auto-man钉e6cc06f）、verify-deps.sh PASS；上层路径消费者 plan705:19/707_cancel:5/442:5/046:11 绿（再导出面保持）；测试ID零改名（plan716扫描面扩展有注记）；ACE在途863/865/871 drafting/881已归档均零R域交集 | AC-06, AC-07 |
| [x] | T-08 | work | T-07 | ✅ 全AC/V自检+KNOWN-DEBT 4条登记+SD-01..06 proposal 就绪（merge落地）；冻结提交=工作树clean；diff无新增调试输出；触面新增警告0（总量=基线131）；merge-owned门 pending（裸t+tv+tt+integration 显式门） | AC-01..AC-07 |
| [ ] | T-09 | review | T-08；同树冻结 | 独立上下文复审所有AC/SD/V和X884；仅相关定向反例/必要check，不以整档替代；findings回work同树修复并重绑 | AC-08 |
| [x] | T-10 | merge | T-09 pass | ✅ 最终冻结 `fa0282c85`（两轮 rebase：a74fea967→219fd5258，12e8050a9→1c2e1149e→fa0282c85；range-diff 5/5、6/6 全等）；并集核验=881/002（d9718cc63..a74fea967）零文件交集、886（a74fea967..12e8050a9，仅 vm/codegen.rs+docs）零交集；裸 t 3204/3204 + tv 163/163 + tt 3575/3575 + integration 11/11 + guard 1/1 + SDK consumer 5/5 + 三仓 consumer 3/3 + verify-deps PASS；ff-only 落地（无 merge commit）→ master tip=fa0282c85；ledger v131→v132（4 条，guarded offline projection）；canonical SD-01..06 已随交付合入并读回；归档+索引再生+源读回+guard/清树完成；限制清单见 §9 merge 收据 | AC-09 |

**未来实施树**：`D:/autostack/.wt/lang-884/auto-lang`、分支`plan-884-dev`，唯一语言写树。按需要同组UI/Man/down为固定SHA detached只读树，不借他方WIP或隐式主检出；不创建junction/symlink。实际base/HEAD/锁/根路径写回T-01；code/check/build/test全部在权威树。计划簿记留主检出，review暂停写入且仍用同树。新跨仓产品改动超本Plan边界，先具体needs_replan，不自行把只读依赖改为写仓。

证据：原始日志/探针/JSON/截图/执行转储本地保存，attachments仅提交Markdown摘要（命令/退出码/选择数/关键观察/路径+hash）。需长期复跑的测试/小工具放受管源码目录；不能改目录名将运行日志伪装产品入库。守原大文件门禁。未授权push、清共享cache、历史过滤、分支更名或默认feature改变。

## 9. 复审记录

2026-10-29 / stage: new / plan_id: PLAN-884 / plan_revision: 1 / outcome: **pass（合同静态预检，非实现/启动放行）** / authorization: 用户本轮明确立项、暂不实施 / progress: drafting 0/10 / scope: T-01..T-10、AC-01..09、V-01..11、SD-01..06 / checks: 0–10单一章节与任务表、任务依赖无环、编号884唯一、AC/V/SD引用与本地链接存在、受影响文档diff/空白/围栏检查通过；G884上游收据已核，X884不假定已释放 / next: 供用户审阅；明确执行指令后work T-01，T-02需真实X884；本轮未创建agent或worktree、未执行任何未来验收、Cargo/GUI/产品改动、canonical/ledger发布或push，未提交。

2026-10-29 / stage: work / plan_id: PLAN-884 / plan_revision: 1 / outcome: **pass → execution_done** / worktree: D:/autostack/.wt/lang-884/auto-lang / branch: plan-884-dev / base: d9718cc63 / code_commits: b215f8616 (oracle) → e358830d2 (T-03) → 10ba01657 (T-04) → 82499c0cb (T-05) → T-06 收口提交 / dependency_revisions: auto-ui d5c7b86 / auto-down c3464d0a / auto-man e6cc06f（同组 detached 只读树）/ task_ids: T-01..T-08 / evidence: 五路审计+基线探针全绿（746:9/872:13/738:75/716:14/608:2/755:4/781:13/a2r:405）于未改基面；T-03..T-06 逐任务定向复验全绿（a2r 405/405 于 T-04/05/06 三次复跑）；plan884 oracle 11/11；V-09 递归守卫 1/1；SDK consumer 5/5、三仓 consumer 3/3、verify-deps PASS；上层路径消费者 705:19/707:5/442:5/046:11；反控 NC-1（漏列→守卫红）/NC-2d（生效shim变异→872红）/NC-3（call派发换序→a2r红）各红后复位绿；NC-2a/b 序互换实测中性=catalog/inventory fn 身份已收敛（884-N2 注记）；触面新增警告 0（总量=基线 131）；diff 无新增调试输出 / blockers: 无 / 净变化: 冻结区间 34 文件 +18265/−17313（净+952；R884-F1 修正，原记为 vs 推进后 master 的 91 文件数）（R域四热点收敛：entry 同构单源、stdlib 壳 8183→2278（R884-F2 修正）、call_inner 22 族 tri-state、rust.rs 7783→~4290；收集器部分去重 param_types 分叉留债 884-N1；死码保留 884-N4） / merge-owned pending: 裸 `cargo t` + `cargo tv` + `cargo tt` + `plan884_language_structure_contract` 显式 integration 门 + SDK/三仓 consumer 定向门 + fresh guard/清理；master 已推进 12 提交（881 交付 db0773130 归档、886 立项），合并前须并集核验 / next_responsible_role: review（T-09，独立上下文同树冻结 SHA 复审）

stage: goal | plan_id: PLAN-884 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-884/auto-lang | commit: 82499c0cb^..T-06收口 | next: review

## 10. 待澄清事项

| ID | 事项 | 归属与当前处理 |
|---|---|---|
| D-01 | 新helper的具体名字与模块划分 | T-02有界设计/最小全链探针；语义/支持面/既有资源边界不变时属机制选择，不需重复授权；不可行则报告具体needs_replan |
| D-02 | 开工时ACE真实owner及R/H/I交集 | T-01读回权威树/最新合同，T-02取得X884；本轮未向其他会话发消息，未假定任何窗口已释放 |
| D-03 | 某个基线失败/错误路径不可稳定复现 | T-02冻结具名现状/独立oracle和可验证范围；必需反例不可验证不能仅记限制放行，先解oracle或具体重规划 |
| D-04 | 873 A1或细拆crate能否加入本Plan | 明确不加入；独立签核/重启条件另议，不是884启动或交付前置 |

2026-10-29 / stage: review / review_scope: final / reviewer: 独立 agent 上下文（非实施会话；只读树，冻结核验 status=clean HEAD=8796cf8b6，区间 d9718cc63..HEAD 5提交；兄弟钉 auto-ui d5c7b86/auto-down c3464d0/auto-man e6cc06f 复核 detached+clean）/ plan_id: PLAN-884 / plan_revision: 1 / outcome: **pass** / reviewed_commit: 8796cf8b6 / base_commit: d9718cc63 / acceptance_results: AC-01..07 全 pass（逐项命令+检查证据）；保真抽样超额完成——call_inner 22/22 helper 与 base 审计段 token 级同一（仅允许的三类改写：CallShape return 形态/委托 emitter .map/pub 可见性），T-06 四新文件 80/80 函数对 base 逐字同（8 残差=提取边界+rustfmt 重排，零语义差），注册 215 调用 API+名+序全同，基面 4 热点文件 421 个函数名在 HEAD 零丢失；定向复验 plan872:13/plan738:75/plan746:9(contract 11/11 内含)/a2r-golden 405/405(37.2s)/guard 1/1；公开面 core/lib.rs+auto-lang/lib.rs diff=空、entry.rs 57/57 签名同；警告=基线131、fmt clean、dbg! 增量0、eprintln/println 增删平衡；SD-01..06 目标现行状态核读（merge 落地，未提前改） / findings: R884-F1 MINOR（§9净变化数字修正→已按复审值落正：34文件+18265/−17313）；R884-F2 INFO（T-04行壳行数→已修正8183→2278）；R884-F3 INFO 预存边界（entry.rs 本不在指纹闭包，与基面一致非回归）/ pending_closeout_gates: 裸 cargo t + cargo tv + cargo tt + --test plan884_language_structure_contract 显式门 + SDK/三仓 consumer 门 + verify-deps + master 12新提交并集核验（881 db0773130/886）+ canonical/ledger/索引读回 + 归档 + wt-guard 清理 / evidence: attachments/884/t01-*.md 五件 + 复审报告全文（本收据即摘要）；KNOWN-DEBT 884-N1..N4 复核为真实且范围准确 / next: merge（T-10）

stage: goal | plan_id: PLAN-884 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-884/auto-lang | commit: 8796cf8b6 | next: merge

2026-10-29 / stage: merge / plan_id: PLAN-884 / plan_revision: 1 / receipt: **PLAN-884:r1** / outcome: **pass**

| checkpoint | evidence |
|---|---|
| prepared | reviewed baseline r1 / `reviewed_commit=8796cf8b6`（5 提交，基面 `d9718cc63`）；SD-01..06 冻结于复审；交付=纯文档后代 `4f409ff90`（首轮 rebase 后，range-diff 5/5 全等）。并集：`d9718cc63..a74fea967`（14 提交，881 交付 `db0773130` + 002 沉淀）与 R 域**文件交集=0**（实测 Compare-Object）；其后 master 再推进 `a74fea967..12e8050a9`（886：`vm/codegen.rs` + docs）再 rebase，range-diff 6/6 全等 → 链 `eb0c297eb..fa0282c85`。证据绑定最终冻结提交 `fa0282c85` |
| landed | `git merge --ff-only plan-884-dev` → master tip = **`fa0282c85` = 交付提交**（无 merge commit）。两轮 rebase 的 old→new 映射：`8796cf8b6→219fd5258`（onto a74fea967）、`219fd5258→1c2e1149e`（onto 12e8050a9）、`4f409ff90→fa0282c85`；两轮 range-diff 均逐提交 `=`。master 上代码/规范落地读回：`entry/run_bridge.rs`、13 个 stdlib 域文件、`call_emit/*` 9 文件、`post_fixes/ast_scans/merged_fixes`、`docs/specs/language-sdk/structure-closure.md` 均在位 |
| gates（merge-owned，绑定 `fa0282c85`） | 裸 `cargo t` **3204/3204 pass**（419.2s）、`cargo tv` **163/163**（6.2s）、`cargo tt` **3575/3575**（475.8s）；显式 `cargo test -p auto-lang --test plan884_language_structure_contract` **11/11**；`plan884_provider_input_coverage` **1/1**；`language-sdk-consumer` **5/5**；`three-repository-consumer` **3/3**；`verify-deps.sh` **PASS**（git bash；无 UI/重复实现，单源计数合规）。日志：`attachments/884/t10-gates-final.log`、`t10-explicit-final.log`（本地证据，不入库） |
| 环境归因（不是回归） | 同一冻结代码在**四仓并发构建**下首轮曾红：t 20 timeout + 3 CTE fail、tt 12 timeout，全部落在 ACE worker 族（`ac_av_probe/ac_case/ac_cte/ace_m1`＝H 域、881 代码，与本 Plan 零文件交集），命中 nextest 60s/120s 墙；逐条隔离复放全绿（`cte_probe` 5/5、`ac_av mixed_recipe` 1/1、`ace_m1` 7/7，tt 隔离段 0 FAIL）；master 空闲基线裸 t 3203/3203 pass；工作树空闲复跑 3204/3204（多 1＝本 Plan 新增 V-09 守卫）+ 3575/3575。未改任何原始退出码，未以 golden/ignore 掩盖。测试顺带重写了 `docs/reports/plan8{18,27}/*.json` 等易变报告：已备份本地并 `git checkout` 还原，未入库 |
| ledger_refreshed | `.autoos/specs.json` **v131→v132**（提交 `624644107`）。4 条：`designs/P884-STRUCTURE-CLOSURE`（file=`docs/specs/language-sdk/structure-closure.md`，含 source-sha256）、`tests/P884-TESTS`（file=`crates/auto-lang/tests/plan884_language_structure_contract.rs`）、`reviews/P884-REVIEW-FINAL`、`reports/P884-DEPOSIT`（file=归档 Plan）；每条带 `file`+`tags`（PLAN-884/r1/reviewed:8796cf8b6/delivery:auto-lang:fa0282c85/guarded-offline-projection）。**通道=guarded offline projection**（store writer 不可达：`127.0.0.1:8080` 连接被拒）：字节级备份 `.autoos/specs.json.bak-p884-merge`(2330475B)、程序化 RMW、仅 4 条追加 + version+1（diff=99+/1-）、读回校验六段 id/字段完成度/file+tags/既有条目零扰动 |
| archived | 本文件 @ `docs/plans/archive/884-language-structure-closure-without-crate-split.md`，`status: archived`；任务 10/10（8 work+1 review+1 merge）全部 `[x]`；D-01..D-04 均为机制/边界记录非未决阻塞；相对链接已按归档位置重写（`../../specs/`、`../reports/`、同级 `869/872/873`）；索引 `python scripts/spec-index.py` 再生（内容无变化）；`spec-lint.py` 对本 Plan 无新增错误（唯一 ERROR=`design: 缺 project.md` 为预存） |
| 交付限制 | 见下节"交付限制与后继指针" |
| cleaned | 见文末 cleaned 收据（guard/树/分支/组目录/构建中间产物） |

**交付限制与后继指针**

- 保留限制（在册债务，详见 `docs/specs/language-sdk/structure-closure.md` §8 + KNOWN-DEBT）：**884-N1**（`collect_fn_param_types` project/merged 语义分叉仍双份，参数化合并需重做两侧 oracle）、**884-N2**（`vm/native.rs:1700-1706` 注释为前史，catalog/inventory 对 872 族已 `fn` 身份收敛）、**884-N3**（stdlib 壳仍驻留跨域 `mod tests` ~600 行 + musk extern 尾段 ~90 行）、**884-N4**（`call_emit/` 三处审计定死码逐字保留）。`entry.rs` 本不在源码指纹闭包内（与基面一致，非回归）。
- 未做且明确不属本 Plan：不拆 crate / 不改 default 特性与依赖 / ABI 不变（873 THR-D1 结论继续有效）；不承诺提速或净删指标（规模数字仅为记录）；不迁 HTTP 服务壳文件（审计确认 `server_*` 单源已成立）；UI/Man 侧结构包仍未编号。
- 后继指针：UI/Man 结构工作包须另立编号（本 Plan 未取号）；ACE 队列（881 已交付，863/865/871 各自窗口）+ 886 诊断面在途，R/H/I 交集按 §5.1 逐符号交接后才写；批量回归 `cargo tf` 按到期待触发（`docs/plans/.last-batch-regression.json`），不在本 Plan 重复。
- 环境备注：CLAUDE.md 中 `cargo build -p auto` 对本仓已失效（`auto` CLI 已迁 auto-man，本仓无同名 package）；本仓部署面=测试二进制与 consumer fixtures，均已在落地提交上重建复验。

## spec-sync 回写记录

| SD | 落地文件 | 内容 |
|---|---|---|
| SD-01 (add) | `docs/specs/language-sdk/structure-closure.md` | 有限结构收口合同：范围/守恒原则、运行桥接单源（G1..G5）、stdlib 域装配与指纹闭包、call_inner 22 族三态、Rust 主管线主题模块、规模事实、测试锚、保留债务、ACE 写面边界 |
| SD-02 (modify) | `docs/specs/language-sdk/architecture.md` | 状态行+模块 DAG（run_bridge）+ owner 表（FFI 13 域/转译 9 主题/运行桥接行）+ "先审计再切分"规则取代 843 "不做手术"+ 规模事实（stdlib 2278/rust 4293 达标，http_server 4156 为合法核心） |
| SD-03 (modify) | `docs/specs/auto-lang/runtime/overview.md` | 结构节 + 新增 "run 入口桥接合同（PLAN-884）"：单源 helper、G4/G5 刻意分离、栈/spawn/join panic/TLS trap/root 恢复 |
| SD-04 (modify) | `docs/specs/auto-lang/trans/overview.md` | 结构节改写（编排 4293 + 9 主题、收集器去重与 884-N1）+ 新增 "`call_inner` 形状族派发（PLAN-884）"（22 族/CallShape/首匹配序/call 事务序/死码保留） |
| SD-05 (modify) | `docs/specs/auto-lang/vm/overview.md`；`docs/specs/stdlib/project.md` | vm：核心结构 13 域、872 绑定序新增第 5 条（19 个域 helper 有序、指纹双清单+守卫）；stdlib：新增 "装配 owner 与指纹输入（PLAN-884）" |
| SD-06 (modify) | `docs/specs/language-sdk/project.md` | 结构半部条目改写为 as-built（四类职责单源 + structure-closure 链接）；模块 `plans.md` 追加 884 行（language-sdk / runtime / trans / vm / stdlib） |
