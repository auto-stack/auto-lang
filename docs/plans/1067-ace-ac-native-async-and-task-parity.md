---
plan_id: PLAN-1067
status: executing
feature_name: ACE M2-B AC原生Future异步与Task调度补齐
author: [Codex]
created_at: 2026-11-16
updated_at: 2026-11-17
plan_revision: 1
current_step: 1
total_steps: 10
depends_on: [PLAN-967, PLAN-968, PLAN-970, PLAN-972, PLAN-973, PLAN-978, PLAN-1065]
supersedes_spec_components: [docs/specs/ace-migration/m2-scheduler.md, docs/specs/ace-migration/m2-control-effects.md]
new_spec_components: [docs/specs/auto-ac/native-async-scheduler.md]
touched_goals: [GOAL-002]
affects: [ace, auto-ac, stdlib]
---

# [PLAN-1067] ACE M2-B AC原生Future异步与Task调度补齐

## 0. 变更摘要

承接968已交AV/A2R而AC明确拒绝的调度差额；在M2-B沿原语义/IR补AC真实机器执行，作为三端一致性的具名producer，交1068及985/B。A出口不反等本包。

用户2026-11-16裁定：M2最终AV/AC/A2R能力一致，A2C完整对齐为增强项；M2-A允许AC具名差额，在M2-B结合AAC真实工程逐步补齐。本轮仅研究与规划，无源码/fixture/构建/运行/合入；全部0/10。归档968等不重开、不改历史收据。

## 1. 目标

补~T异步计算、await/go、Task/actor/send/ask/reply、timer/mailbox、取消/背压/有序关闭以及owned/错误结果跨挂起点的AC实现。原生程序实际执行Auto业务与状态机，不嵌AV解释来冒AC。

同一目标平台的公共语言/库/领域语义对齐，不要求相同内部布局或调度实现。A2C保已有代码/用例/支持事实，缺完整对齐不阻M2；既有A2PY标准库义务、AV单执行器三provider、Linux1063延期规则保持。Linux写代码和完整用例，当期不测试；不能用Windows证据升Linux verified。

## 2. 架构方案

复用core源码解析/类型/调度desugar、原SCHED_*与资源事实；在native lowering/runtime接实际状态机/调度与OS等待/完成事件。优先复用受检Auto/MIR运行逻辑，OS薄层按目标ABI；不把Tokio/Rust std或C库网络/文件实现悄然当默认Auto库，不为AC复制语言语义。预编译必要runtime/机器stub的来源、ABI与内存责任必须列明。

本包M2-B实施门为985/T-A05已落主M2A-08收据及本包执行授权；允许A出口保留有owner/输入/独立expected/关闭条件的AC差额。frontmatter不写整个985，防985/B→本包→985全包环；也不要求AAC1054整包或Gen2/Gen3先完成。

## 3. 技术栈

既有Schema/CST→共享绑定/Checked HIR→MIR→AV/native Cranelift/A2X-Rust；Auto默认库＋受检OS原语，显式外库经967稳定ABI。未来唯一树`D:/autostack/.wt/lang-1067/auto-lang`、分支`plan-1067-dev`，材料唯一`D:/autostack/plan-artifacts/plan-1067/`。纯规划在主检出；跨仓只有真实修改需求与对应owner接续时加入同组树，禁止链接。

## 4. 需求分析与背景调查

授权为当前目标/阶段边界的设计和计划修订，未要求立即启动实现或新会话，无新增预算。起草读取HEAD `2961a2aa24cb8f6158e638786cb0972b812dd589`；来源为[总览](../specs/overview.md)、[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[AAC设计](../design/59-aac-native-bootstrap-and-compiler-subject.md)、[调度现状](../specs/ace-migration/m2-scheduler.md)及985。归档965/966/968与真实代码只证明其原支持范围，不代三端全能力。

已实读：`experimental/ace/crates/native/src/lower.rs`含`native.sched-unsupported`以及platform.net拒绝；`av-core/src/abc.rs`有SCHED_SPAWN/AWAIT等；`core/src/source/sched_desugar.rs`、`av/src/scheduler_run.rs`/`sched_worker.rs`、`a2x-rust/src/sched_rt.txt`与两侧`sched_m2_968.rs`为原实现/判据来源。968现规范明确AV/A2R支持、native/A2C拒绝；这是补齐起点，不改写为已支持。

开工T-01重绑所有源码/规范/工具/ABI/SDK身份；源或语义变化只重验实际影响行，不能复用失效PASS。使用既有TestCase/AutoDown/worker/固定组，不造新注册器或另一个编译前门。

## 5. 详细设计

逐项保留968原case/check与源码、类型/返回域、错误/owner、平台/预算；新增一般~T与嵌套Option/Result组合的源级义务，不能只复刻单态Task整数ID。共享类型事实区分Future<T>与Task/Iter，不将所有~修饰擦成同一i32。合法表面按现行规范/grammar核定，不在规划中发明新糖。

| 边界 | 本包合同 |
|---|---|
| 创建/挂起/恢复 | 函数/块产生异步计算，结果跨参数/返回/容器/泛型边界保类型；await交值及错误，不重复副作用 |
| Task与mailbox | 沿原FIFO/容量/发送背压/ask-reply单终态与停止规则，AC真实执行handler |
| 事件/计时 | 等待不忙等、不阻塞唯一执行器；Windows公开OS API，Linux分支及原例交1063，当前不实跑 |
| 取消/退出 | 挂起缓冲保活；取消/完成竞争仅提交一次，迟到结果不复活旧实例，finally/close/最后owner恰清一次 |
| 错误边界 | Result业务错误可处理；OS/资源/取消/基础设施分类按现合同，不吞为成功空值，也不将所有失败当语言反例 |
| CTE/宿主 | 编译期显式AV服务可保留；目标运行期Auto业务不得转AV解释或先转C/Rust源码 |

T-02有界先证一条Future<Result<owned>>调用→AC状态机→受检OS完成→取消/清理全链后，才扩其余面。具体native ABI、pin/帧布局、唤醒队列与等待原语在原owner内决定，不新建第二资源ledger；对外语义变化须正式修订。

### T-01 决策冻结（2026-11-17，work，r1）

**阶段门与授权**：985/T-A05已落主、M2A-08成立（[m2-certification.md](../specs/ace-migration/m2-certification.md) §门级行，收据 `95c7fbad8`/`0db949754`，B批解锁）；本轮goal即本包执行授权，无需再询。基线 `f0db9bd0f`；唯一树 `D:/autostack/.wt/lang-1067/auto-lang`、分支 `plan-1067-dev`；材料 `D:/autostack/plan-artifacts/plan-1067/`（主检出中他人docs WIP未触碰）。

**冻结的调度面（沿968 D1–D6，不改语义）**：11条`SCHED_*` MIR（0x41–0x4B）、scheduler源profile（closures超集）、每task一个driver `__task_<Name>(__init…) int`（start/stop钩子＋`while true`收/分派环）、handle/future为i32、mailbox容量=spawn实参、ask端口1基单消费者、退出有序drain（置stop→缓冲消息先投递→stop钩子→join→全释放；drain期sleep立即完成）。语料=`crates/auto-lang/test/cases/ace-m2/scheduler/`（8正例+3反例），AV oracle组`case-ace-m2-scheduler-968`。

**native车道方案（T-01择定，T-02已实证）**：
- 新no_std kernel32 runtime crate `experimental/ace/crates/sched-runtime`（ABI `mn.sched.runtime/1`）：task=真实OS线程（`CreateThread`），mailbox=`SRWLOCK`+`CONDITION_VARIABLE`阻塞（满则park发送者=背压；无忙等、不占唯一执行器）；ask/reply端口单终态提交（迟到reply丢弃）；TLS当前task；`mn_sched_drain`置stop→join→逐块释放并以`LIVE_ALLOCS`残值审计（非0=具名`72 sched.leak`退出，绝不冒成功）；语言错误`mn_sched_fail`写冻结消息到stderr后`ExitProcess(70)`（=native Trap类）。
- 降级：`LowerMode::Sched`=Numeric值面＋console/1 `HostCall`＋SCHED发射；每个callee一份unit-local spawn thunk（`ac1067tk.<owner>.<unit>.<idx>`）按callee参数种类解包后调driver自己的unit符号；参数/消息用三字`SchedVal`（tag/payload/spare），为T-03的typed/owned值预留形状；`platform.*`与owned text跨挂起**具名出切**（T-05/T-03）。
- 支撑库约束：两个Rust no_std staticlib同链接会在panic/unwind符号上冲突，故scheduler runtime以依赖方式**内嵌**owned-text runtime（text/alloc ABI＋编译器生成的mem*），identity同时哈希两者源码与`Cargo.lock`（沿用typed/collections prepare的内容绑定纪律）。
- 组与平台：本包日常`cargo t ace-native`（新增test target `sched_m2_1067`）；AV oracle `cargo t case-ace-m2-scheduler-968`；Linux实现照写、完整用例NOT_RUN归1063，不用Windows证据升verified。

### T-02 首个垂直切片实证（2026-11-17，work，r1）

已跑通**7/8正例的完整生产链**（scheduler源→共享前门→closures低层→verify→unit split→SCHED发射→真PE运行→drain审计），stdout与AV契约逐字节相同：

| case | 观测stdout | 观测exit/stderr |
|---|---|---|
| message_handler | `got one\n` | 0 / 空 |
| mailbox_fifo | `got two\ngot one\n` | 0 / 空 |
| ask_reply | `reply:\n42\n` | 0 / 空 |
| go_join | `go:\n7\n` | 0 / 空 |
| backpressure | `1\n2\n3\n` | 0 / 空 |
| stop_hook | `one\nstopped\n` | 0 / 空 |
| virtual_timer | `main\nlater\n` | 0 / 空 |
| neg_send_after_stop | — | 70 / `sched: send on a closed task\n` |
| neg_unknown_task | — | 前门 `bind/source.sched-task` |
| neg_malformed_task | — | 前门 `text/source.unsupported` |

永久证据：`experimental/ace/crates/native/tests/sched_m2_1067.rs`（4 tests，2.4s：逐字节正例＋零残值drain审计、send-after-stop具名错误、前门具名拒绝、重复运行独立）。**未闭**：`spawn_with_state`（Str初始化）按T-03补，取消/迟到终态与`platform.*`计时按T-05补；`Future<Result<owned>>`整链在T-03完成后才算T-02的完整谓词，故本轮T-02保持未勾。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-ASYNC | modify | docs/specs/ace-migration/m2-scheduler.md | native named rejection→经真实验证的AC调度/语义及原错误资源合同 | M2最终三端一致 | AC-01/02/03 |
| SD-NATIVE | add | docs/specs/auto-ac/native-async-scheduler.md（新） | 无AC状态机→已验帧/完成/取消/资源/OS/依赖闭包 | 可部署native | AC-02/03/04 |

## 6. 测试设计

下表均为未实施合同；原T任务将其接到真实入口，保旧输入规模/判据/资源义务。三端分别对独立expected，不仅相互比较；确定性输出逐字节，异步合法交错按因果/完成/清理合同判，不强求同一时钟值或线程排布。

| V / AC / work owner | 实际输入与入口 | 可区分oracle与负控 | 覆盖边界 |
|---|---|---|---|
| V-01 / AC-01 / T-01、06 | 968原11case及实际补例，同一普通源码AV/AC/A2R | 原值/byte、FIFO和独立因果trace；AC仍恒拒绝或实际AV fallback须红 | 不用旧A2C零支持缩AC目标 |
| V-02 / AC-02 / T-02、03 | 泛型异步函数携Option/Result/owned文本，跨调用/返回/两次await及yield近邻 | 标记副作用次数/独立返回值与析构账；错误类型、错误await对象在执行前拒绝 | 实际拼写由现规范冻结 |
| V-03 / AC-02/03 / T-04、05 | mailbox容量边界、ask/reply、真实完成/延迟/取消/关闭竞争 | 仅一次回复/提交、FIFO/背压、停止后无dispatch、任务/缓冲/handle归零；不能只测取消flag | 允许合法交错，不承诺即时抢占 |
| V-04 / AC-03/04 / T-05、07 | timer、可取消外部完成事件、失败后再次正常任务、仓外PE | OS真实等待/时间单位/恢复；链接闭包和调用点审计，无AV/旧VM/隐式libc或Rust std IO | Linux代码/完整case NOT_RUN归1063 |
| V-05 / AC-01..04 / T-08 | 原组完整影响集及错ABI/坏源/错代际/缺资源/emit-only控制 | 各自独立expected，拒绝位置正确；stub替真身或仅生成未运行须整体失败 | 真实标准库网络扩面另由1068协调原owner |

现有候选固定组：`cargo t ace-source ace-semantics ace-lowering ace-av ace-native ace-a2x ace-runtime`，调度用`case-ace-m2-scheduler-968`，库/FFI/SDK按真实触面加入`ace-stdlib stdlib ace-foreign-contract ace-abi ace-artifacts`。T-01核实际组ID/选择，新增native用例注册到原组；新增AAC/GUI入口必须由相应任务/产品owner落实后记录，不将拟名当现成命令。普通组不带Linux/performance；正式性能仅显式专项，保原成本和必要集成门。

## 7. 验收标准

| ID | owner_stage | 完成谓词 |
|---|---|---|
| AC-01 | work | 原调度义务＋新增Future类型组合逐行有实现/来源与独立正反例，V-01/05闭合 |
| AC-02 | work | AC真执行异步状态机/Task和typed结果，V-02/03满足，同源AV/A2R按合同等价 |
| AC-03 | work | 挂起/事件/取消/错误/close/退出资源满足V-03/04，无迟到提交或泄漏 |
| AC-04 | work | Windows仓外native及默认依赖审计真实通过；Linux完整移交1063，不冒verified |
| AC-REVIEW | review | 独立上下文沿真实producer/consumer核全部AC/V/SD/差额，不用本包自报PASS；缺项回同树修复 |
| AC-LAND | merge | 已审源码/Spec/来源正式合入，收据供985/B；归档及实际所有输出scope清理、wt-guard/树/分支收尾完成 |

## 8. 执行步骤

全部0/10，原型/用例均待实施；当前不启动工作树或产品。

- [x] **T-01｜work｜AC-01**：核985/A阶段门和原源码/规范/组，冻结~T/Task/Iter与每个旧新义务/预算、SCHED/native/platform接面、Linux移交。 — 2026-11-17 ✅（见§5 T-01冻结：M2A-08门、11 SCHED面、native线程runtime方案、单staticlib约束、组/平台名单）
- [ ] **T-02｜work｜AC-02/03**：在原native/runtime有限先证typed Future<Result<owned>>完整生产→挂起→完成/取消链，冻结一个可实现方案；不拿手填MIR代普通源。 — 进行中：7/8正例逐字节＋send-after-stop具名错误已真PE跑通（`sched_m2_1067` 4 tests）；Str owned跨挂起（spawn_with_state）与取消终态待T-03/T-05。
- [ ] **T-03｜work｜AC-02**：沿共享类型/IR补AC异步帧、await/go及泛型/Option/Result跨挂起值和所有权，落V-02正反例；依T-02。
- [ ] **T-04｜work｜AC-02/03**：原SCHED_*补AC Task/actor/mailbox/send/ask/reply/stop真执行及FIFO/背压，保旧源码；依T-02/03。
- [ ] **T-05｜work｜AC-03**：接OS等待/timer/967完成端口、取消/late/异常/退出清理，补V-03/04并核真实资源；依T-03/04。
- [ ] **T-06｜work｜AC-01/02**：同源AV/AC/A2R完整原调度语料与新Future组合各对独立oracle，保非法近邻/原条件；依T-03..05。
- [ ] **T-07｜work｜AC-04**：978式仓外候选真PE运行和OS/库依赖审计，Linux代码/原完整case归1063；依T-03..06。
- [ ] **T-08｜work｜AC-01..04**：跑冻结影响组/必要consumer，核V-01..05、原规模/未验项与准确收据，交1068；依全部work。
- [ ] **T-09｜review｜AC-REVIEW**：同树独立复核真实native调用/差额/所有权/错误/源级组合，缺陷回同树有界修复。
- [ ] **T-10｜merge｜AC-LAND**：review pass后按变化补门，Spec/源码正式合回、收据交1068/985-B，折叠证据、清scope/树/分支并核实际路径。

T-01先冻结依赖与未决；T-02在依赖实现前先证全链，T-03..07按各自已交接缝执行；T-08汇总全部work，T-09独立review→T-10 merge串行。发现超出独立评审规模的新增实现域时，必须在本包正式修订中逐原行迁交具名后继并取得实际交付，不能仅写“以后修”后勾整包完成。

## 9. 复审记录

2026-11-17 stage: work | PLAN-1067 r1 | T-01 完成 + T-02 垂直切片实证 | worktree `D:/autostack/.wt/lang-1067/auto-lang`（branch `plan-1067-dev`，base `f0db9bd0f`，code `88999db74`）| task_ids: T-01[x]、T-02(进行中)

- 门/身份：985 M2A-08 成立（[m2-certification.md](../specs/ace-migration/m2-certification.md)），本 goal 即执行授权；基线时主检出仅他人 docs WIP（未触碰）。
- 实现：新增 `experimental/ace/crates/sched-runtime`（no_std kernel32, ABI `mn.sched.runtime/1`）与 `native/src/sched.rs`＋`sched_runtime.rs`，`lower.rs` 增 `LowerMode::Sched`/SCHED 发射/spawn thunk，`emit.rs` 增调度准入与 `emit_sched_unit_coff`；单 Rust staticlib 约束下内嵌 text runtime；wrapper 带 drain 且残值→退出 72。
- 观测（真 PE，非 AV 解释）：`message_handler`=`got one\n`、`mailbox_fifo`=`got two\ngot one\n`、`ask_reply`=`reply:\n42\n`、`go_join`=`go:\n7\n`、`backpressure`=`1\n2\n3\n`、`stop_hook`=`one\nstopped\n`、`virtual_timer`=`main\nlater\n`，全部 exit 0/空 stderr；`neg_send_after_stop` exit 70 + `sched: send on a closed task`；`neg_unknown_task`/`neg_malformed_task` 分别为前门 `bind/source.sched-task`、`text/source.unsupported`。
- 运行：`cargo test -p ace-native --test sched_m2_1067` 4/4 PASS（2.4s，单 case ≤3.1s）；`cargo t ace-native` PASS 303 tests（53.7s，19 skipped；一次并行跑 `closures_incremental::tests::audit1027_utf_02` 瞬时红，单独重放与整组重跑均绿=并行争用归因，非本 diff）；AV oracle `cargo t case-ace-m2-scheduler-968` 基线 PASS。
- 未闭（按计划原任务）：`spawn_with_state` 的 owned Str 初始化（T-03）、取消/迟到终态与 `platform.*` 计时（T-05）、native 语料进 case-runner 与三端对照（T-06）。T-02 完整谓词（`Future<Result<owned>>` 整链）待 T-03 后勾。
- 阻塞：无。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r1：用户已裁定三端最终对齐及A→B延期边界；本稿可供规划审查，未实施、未测试、未签M2/AAC。静态核验记录另补，实际work仍核阶段门和来源。

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 88999db74 | next: work

## 10. 待澄清事项

原生调度的帧/等待策略及资源上限由T-01/02依据现合同先证；本次不预定私有Rust ABI、任意线程数或无穷规模。超出原语义的并发模型/平台扩面不得借本计划引入。

日常等价实现选择由T-01/02有界调查决定，不为已授权方向重复申请确认。未提供平台环境保持NOT_RUN；计划结构检查不能代实际运行证据。
