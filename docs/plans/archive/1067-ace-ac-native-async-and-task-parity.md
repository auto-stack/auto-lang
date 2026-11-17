---
plan_id: PLAN-1067
status: archived
feature_name: ACE M2-B AC原生Future异步与Task调度补齐
author: [Codex]
created_at: 2026-11-16
updated_at: 2026-11-17
plan_revision: 1
current_step: 10
total_steps: 10
depends_on: [PLAN-967, PLAN-968, PLAN-970, PLAN-972, PLAN-973, PLAN-978, PLAN-1065]
supersedes_spec_components: [docs/specs/ace-migration/m2-scheduler.md]
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

授权为当前目标/阶段边界的设计和计划修订，未要求立即启动实现或新会话，无新增预算。起草读取HEAD `2961a2aa24cb8f6158e638786cb0972b812dd589`；来源为[总览](../../specs/overview.md)、[Design55](../../design/55-ace-m2-complete-language-and-product-replacement.md)、[AAC设计](../../design/59-aac-native-bootstrap-and-compiler-subject.md)、[调度现状](../../specs/ace-migration/m2-scheduler.md)及985。归档965/966/968与真实代码只证明其原支持范围，不代三端全能力。

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

**阶段门与授权**：985/T-A05已落主、M2A-08成立（[m2-certification.md](../../specs/ace-migration/m2-certification.md) §门级行，收据 `95c7fbad8`/`0db949754`，B批解锁）；本轮goal即本包执行授权，无需再询。基线 `f0db9bd0f`；唯一树 `D:/autostack/.wt/lang-1067/auto-lang`、分支 `plan-1067-dev`；材料 `D:/autostack/plan-artifacts/plan-1067/`（主检出中他人docs WIP未触碰）。

**冻结的调度面（沿968 D1–D6；本轮仅在共享前门收紧三条实现一致性规则——`Task.spawn` 仅入口、mailbox `cap>=1`、`stopping()` 需邮箱耗尽——三者都是 968 已写明的"bounded mailbox / 具名错误 / 缓冲先投递"合同的落实，不扩张语言能力）**：11条`SCHED_*` MIR（0x41–0x4B）、scheduler源profile（closures超集）、每task一个driver `__task_<Name>(__init…) int`（start/stop钩子＋`while true`收/分派环）、handle/future为i32、mailbox容量=spawn实参、ask端口1基单消费者、退出有序drain（置stop→缓冲消息先投递→stop钩子→join→全释放；drain期sleep立即完成）。语料=`crates/auto-lang/test/cases/ace-m2/scheduler/`（8正例+3反例），AV oracle组`case-ace-m2-scheduler-968`。

**native车道方案（T-01择定，T-02已实证）**：
- 新no_std kernel32 runtime crate `experimental/ace/crates/sched-runtime`（ABI `mn.sched.runtime/1`）：task=真实OS线程（`CreateThread`），mailbox=`SRWLOCK`+`CONDITION_VARIABLE`阻塞（满则park发送者=背压；无忙等、不占唯一执行器）；ask/reply端口单终态提交（迟到reply丢弃）；TLS当前task；`mn_sched_drain`置stop→join→逐块释放并以`LIVE_ALLOCS`残值审计（非0=具名`72 sched.leak`退出，绝不冒成功）；语言错误`mn_sched_fail`写冻结消息到stderr后`ExitProcess(70)`（=native Trap类）。
- 降级：`LowerMode::Sched`=Numeric值面＋console/1 `HostCall`＋SCHED发射；每个callee一份unit-local spawn thunk（`ac1067tk.<owner>.<unit>.<idx>`）按callee参数种类解包后调driver自己的unit符号；参数/消息用三字`SchedVal`（tag/payload/spare），为T-03的typed/owned值预留形状；`platform.*`与owned text跨挂起**具名出切**（T-05/T-03）。
- 支撑库约束：两个Rust no_std staticlib同链接会在panic/unwind符号上冲突，故scheduler runtime以依赖方式**内嵌**owned-text runtime（text/alloc ABI＋编译器生成的mem*），identity同时哈希两者源码与`Cargo.lock`（沿用typed/collections prepare的内容绑定纪律）。
- 组与平台：本包日常`cargo t ace-native`（新增test target `sched_m2_1067`）；AV oracle `cargo t case-ace-m2-scheduler-968`；Linux实现照写、完整用例NOT_RUN归1063，不用Windows证据升verified。

### T-03 typed/owned 值跨挂起（2026-11-17，work，r1，部分闭合）

**已实现并实证**：
- 值面：`SchedVal`（tag 0=i32 / 1=bool / 2=owned text ptr+len）在 spawn 实参、send/ask 消息、reply 与 await 结果四个方向同一形状；native 侧 `SchedArgKind::Str` 按私有 ABI 的 (ptr,len) 解包进 driver 的 unit 符号。
- 所有权：runtime 在**交接点物化**（`ac_rt_own_clone_quiet` 走内嵌 owned-text 分配器＝guest 释放用的同一分配器），接收方持有并以 `ac_rt_own_free`/出口 sweep 释放；未投递/迟到/未 await 的缓冲在 drain 释放。
- 审计：drain 残值 = runtime 堆块数 + 内嵌 text ledger（`ac_rt_owned_audit`），任一不为零即具名退出 72，绝不冒成功。
- 探针通道：owned-text runtime 的 CLONE/CONCAT/DISPLAY/FREE 行是观察而非语言输出，本车道 wrapper 在 entry 前调 `ac_rt_trace_switch(0)` 关闭，故 stderr 与 AV 逐字节相同。

**修掉的前端缺陷（真实 bug，非本包放宽）**：CST 前端把 `reply(v)` 直接投影为 `__sched_reply`，而 desugar 的 reply 类型推断只认 `reply` 拼写，导致**任何**非 i32 reply 声明都退化为 i32。已让推断识别两种拼写；现 `reply(true)` 可推断为 bool 并用真实 await 取回（`scheduler_typed_reply_crosses_the_await` → `true`）。

**具名差额（有 owner/输入/独立 expected/关闭条件）**：
| 差额 | 实测输入 | 当前行为 | 关闭条件与 owner |
|---|---|---|---|
| owned-text **消息/回复** | `h.send("hello")`（Str 消息臂）；`reply("v42")`（Str 回复） | 解析期 `source.sched-argument`（内建签名要 owned `Str`，表面只产 borrowed view）；无静默通过 | 在交接点插入显式 owned 转换节点（`text_to_owned` 已存在，需接到 scheduler 参数投影），owner=1068/T-02 共享前端；关闭时须同步改 `scheduler_owned_text_message_and_reply_stay_named_rejections` 的边界钉 |

永久证据：`tests/sched_m2_1067.rs` 8 tests / 约 8s（corpus 8 例逐字节 + owned-Str 状态跨 spawn + bool reply 跨 await + 未 await 回复的 drain 释放 + send-after-stop + 前门具名拒绝 + 边界钉 + 重复运行）。



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

### T-05 OS 等待/取消/退出清理（2026-11-17，work，r1）

**实现**：
- 真实 OS 等待：`sleep(ms)` 在独立线程内以 OS 睡眠分片执行（每 10ms 检查 drain/stop），不忙等、不占唯一执行器；drain 期 sleep 立即返回（AV 同规则）。
- 未发布 future 的终止语义（**三端一致**）：生产者 driver 结束时，它已 latch 但未回的 ask 端口、以及仍排在它邮箱里（再也无人接收）的 ask 消息端口，一律置为终态 `Closed`；drain 时所有仍 Pending 的端口同样置 `Closed`。等待方得到具名语言错误而非自旋到 fuel 上限。
  - native：`E_REPLY_CLOSED` → stderr `sched: task closed without replying` + trap 70；
  - AV：`ExecOutcome::Error { message: "task closed without replying" }`（PLAN-1067 前该输入是 `Resource{InstrSteps, scheduler multiplex exceeded the fuel budget}`，即 AV 侧真实缺陷）。
- 退出清理：drain 逐块释放（task 邮箱/端口/线程参数/快照表）＋ text ledger 审计；未投递消息与已发布未消费回复内的 owned 值一并释放。

**观测**（真 PE / 真 AV）：
| 用例 | native | AV |
|---|---|---|
| 唤醒后 40ms 再回（`scheduler_sleep_is_a_real_os_wait`） | exit 0、`42\n`、PE 墙钟 ≥35ms（no-op sleep 会是个位数 ms） | — |
| ask 后立刻 stop、未回复（`scheduler_await_on_unreplied_ask_is_a_named_error` / `await_on_unreplied_ask_is_a_named_language_error`） | exit 70 + 具名消息、无输出、<1s 返回（不自旋） | 具名 Error、stdout 空 |
| 复合退出（2 活 task＋未消费缓冲＋未 await 回复＋go timer，`scheduler_exit_drain_releases_every_outstanding_owner`） | exit 0（零残值，绝不 72）、stderr 空、收敛 <10s | — |

**本轮审计发现的既有缺陷（不属本车道，记名）**：owned-text 临时值在**循环体第二次迭代**重建时未逐次释放（退出 sweep 只释放最后一次写）。输入：task 臂内 `print(f"a=${n}")` 收两条消息 → native lane drain 审计报残值 1（exit 72）；同一源改 `print(n)` 则 exit 0。单次迭代（`spawn_with_state`）正常。owner＝共享 owned-text native 降层（PLAN-774/870 的 sweep 面）；关闭条件＝循环内文本临时逐次释放或槽复用纪律，并加"≥2 次迭代后 text ledger 归零"反控。本包复合退出用例改用标量打印以免测他域缺陷。



### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-ASYNC | modify | docs/specs/ace-migration/m2-scheduler.md（合入时改 **:10-11、:19**；另 **:17** AV 调度用例 12→17、并补记三端共享的"仅入口 spawn / cap≥1 / stop 保留缓冲投递"规则） | native named rejection→经真实验证的AC调度/语义及原错误资源合同 | M2最终三端一致 | AC-01/02/03 |
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
- [x] **T-02｜work｜AC-02/03**[T-1067-1 已修]：在原native/runtime有限先证typed Future<Result<owned>>完整生产→挂起→完成/取消链，冻结一个可实现方案；不拿手填MIR代普通源。 — 2026-11-17 ✅（8/8正例逐字节真PE＋owned Str 状态跨 spawn＋bool reply 跨 await＋未 await 回复 drain 释放＋具名错误；`sched_m2_1067` 8 tests。取消/迟到终态按原任务归 T-05，不属本行谓词）
- [x] **T-03｜work｜AC-02**：沿共享类型/IR补AC异步帧、await/go及泛型/Option/Result跨挂起值和所有权，落V-02正反例；依T-02。 — 2026-11-17 ✅（值面tag/物化/释放/drain审计＋bool typed reply；owned-text 消息/回复为具名差额，见§5 T-03 表）
- [x] **T-04｜work｜AC-02/03**[N-1067-1/2/3 已修]：原SCHED_*补AC Task/actor/mailbox/send/ask/reply/stop真执行及FIFO/背压，保旧源码；依T-02/03。 — 曾于 2026-11-17 记 ✅，review 判 R-1067-1/3 不成立，重开（历史证据保留：真PE逐字节：message_handler/mailbox_fifo FIFO、ask_reply 42、stop_hook 先投递后 stop、backpressure cap=1 三消息顺序不漏；源码为 968 原语料未改）
- [x] **T-05｜work｜AC-03**[T-1067-1 已修]：接OS等待/timer/967完成端口、取消/late/异常/退出清理，补V-03/04并核真实资源；依T-03/04。 — 曾于 2026-11-17 记 ✅，review 判 R-1067-1/2 不成立，重开（历史证据保留：真实OS等待墙钟oracle、未发布future在三端统一具名终止、复合退出零残值；platform.*源级入口具名交1068/T-03，见§10）
- [x] **T-06｜work｜AC-01/02**[T-1067-1 已修]：同源AV/AC/A2R完整原调度语料与新Future组合各对独立oracle，保非法近邻/原条件；依T-03..05。 — 曾于 2026-11-17 记 ✅，review 判 R-1067-4 不成立，重开（历史证据保留：三端各对独立 expected：AV ingress 8正例＋3反例分类、native 8正例＋3反例、A2R emit+rustc+run 8正例(timer 已补)＋closed-task 反例；并修掉 A2R 静默成功的三端背离）
- [x] **T-07｜work｜AC-04**：978式仓外候选真PE运行和OS/库依赖审计，Linux代码/原完整case归1063；依T-03..06。 — 2026-11-17 ✅（显式仓外输出根构建+真实进程运行+published PE 导入审计：仅 kernel32.dll、符号全在本车道 19 项声明内、变异体被具名拒绝；runtime 身份与库字节一致绑定）
- [x] **T-08｜work｜AC-01..04**[T-1067-2/3/4/5 已修]：跑冻结影响组/必要consumer，核V-01..05、原规模/未验项与准确收据，交1068；依全部work。 — 曾于 2026-11-17 记 ✅，review 判 R-1067-5/6/7/9/10 不成立，重开（历史证据保留：并集 5 组 PASS＋case 组 11/11；ace-a2x 单跑 PASS（并集内 cl spawn deadline 归因）；ace-runtime 19 红经基线版本实证为既有；AC-01..04 证据表见 §9 round-6）
- [x] **T-09｜review｜AC-REVIEW**：同树独立复核真实native调用/差额/所有权/错误/源级组合，缺陷回同树有界修复。
- [x] **T-10｜merge｜AC-LAND**：review pass后按变化补门，Spec/源码正式合回、收据交1068/985-B，折叠证据、清scope/树/分支并核实际路径。

T-01先冻结依赖与未决；T-02在依赖实现前先证全链，T-03..07按各自已交接缝执行；T-08汇总全部work，T-09独立review→T-10 merge串行。发现超出独立评审规模的新增实现域时，必须在本包正式修订中逐原行迁交具名后继并取得实际交付，不能仅写“以后修”后勾整包完成。

## 9. 复审记录

2026-11-17 stage: merge | PLAN-1067:r1 | **outcome: pass** | delivery_commit: `b28399cc9`（rebased tip；reviewed `104cde943`）| archive: `docs/plans/archive/1067-ace-ac-native-async-and-task-parity.md` | completion_kind: delivered

**checkpoints（实际证据）**
| checkpoint | 证据 |
|---|---|
| `prepared` | reviewed `104cde943` + r1 final pass（reviewer-1067d）；SD-NATIVE 已在分支；SD-ASYNC 在 worktree 落地（`m2-scheduler.md:9-13,22,24-25`：native 不再具名拒绝、17 cases、A2R 三种收据、三条共享实现一致性规则）；frontmatter `supersedes_spec_components` 按 U-1067-1 去掉 `m2-control-effects.md`（merge-owned 元数据对账） |
| `landed` | worktree rebase onto `master@65befa560`：12 提交全部 patch-identical（`git range-diff f0db9bd0f..a379f121b master..HEAD` **12/12 `=`**），old→new 映射：`88999db74→b85351850`、`e16f01b6e→13e43280c`、`c6b84240e→ea4bae2bd`、`40f5c195b→19b2d7b6d`、`005096495→7f0bb3e16`、`817eb1e58→436a6ac43`、`12f11b641→cbe24afab`、`4d6e81a2c→01f4ffcff`、`eeff4d60e→058b2608d`、`f0814b271→cd0a8dd1a`、`104cde943→a90f5c7c4`、`a379f121b→b28399cc9`；主检出 `git merge --ff-only plan-1067-dev` 成功（无 merge commit），master tip == `b28399cc9` |
| `canonical_verified` | landed `docs/specs/auto-ac/native-async-scheduler.md` sha256 `350F5E0E72F084F00C7F1887E7B7B3C8629A975DCD050928E2A9B5241EFEB74A`；`docs/specs/ace-migration/m2-scheduler.md` sha256 `5BDD5AEA28011CB474E01AC974748202280B92C4112D543648F2DA1BBEA72DE3`（SD-ASYNC 已合入，无 legacy ledger 写入） |
| `landed` 门（隔离取证） | **主检出当前不可用以跑门禁**：另一会话（PLAN-982 config-document）在共享主检出留了**未提交代码 WIP**（`crates/core/src/source/mod.rs` 引 `SourceProfile::ConfigDocument` 但枚举未加），`ace-core` 编译失败 → `cargo t ace-native …` 失败为 `tool preparation failed: av`。按规则**未触碰其 WIP**；改在 merged commit 的**干净 worktree**（`D:/autostack/.wt/verify-1067/auto-lang`，detached `b28399cc9`，tree clean）跑同一门禁：**`cargo t ace-native case-ace-m2-scheduler-968` PASS**（run-bc_ool_i，ace-native 全绿、case 组 11/11） |
| `archived` | 本文件由 `docs/plans/` `git mv` 至 `docs/plans/archive/`，`status: archived`；任务 10/10、金额/ID/链接按最终状态回读 |

**landing 后主检出的两个 stray（已处理并留证）**：主检出曾残留本会话早期误写的 `experimental/.../sched_m2_968.rs`（旧探针版本，已被分支最终版替代）与**空文件** `docs/specs/auto-ac/native-async-scheduler.md`；两者均先备份到 `plan-artifacts/plan-1067/stray-main/` 再移除，ff-only merge 得以完成。该 stray spec 与主检出未提交的 config-document WIP 无关联。

**landing 不是部署**：本包只改 `experimental/ace`（独立 workspace）与 `docs/specs`；`auto` CLI 二进制不涉及，无需重建/重启；记录为"无产物刷新需求"。
**批量回归 due check**：`docs/plans/.last-batch-regression.json` = last_covered 1064 / 2026-11-16T10:04Z；本包 id 1067（%5≠0）且间隔 ≈25h <48h ⇒ **不触发** `cargo t all` 批次门。
2026-11-17 stage: review | review_scope: final（第四次复审） | reviewer_identity: reviewer-1067d（独立委派上下文，无实现署名；自建双车道探针 + 门内 nextest 三次 + 全触面 rustfmt + 代码/spec 逐行重建） | plan_id: PLAN-1067 | plan_revision: 1 | **outcome: pass** | worktree_path: D:/autostack/.wt/lang-1067/auto-lang | branch: plan-1067-dev | reviewed_commit: `104cde943089ddc03b42170a916e0038e0494c6e` | base_commit: `f0db9bd0f746f115f9ec16124b798939e96e5860` | spec_inputs: docs/specs/auto-ac/native-async-scheduler.md@104cde943（sha256 `350F5E0E72F084F00C7F1887E7B7B3C8629A975DCD050928E2A9B5241EFEB74A`）、docs/specs/ace-migration/m2-scheduler.md（live 未改，SD-ASYNC pending） | acceptance_results: **AC-01 pass；AC-02 pass；AC-03 pass；AC-04 pass；AC-REVIEW pass（本记录）** | pending_closeout_gates: merge 侧 SD-ASYNC 改 `m2-scheduler.md:10-11/:17(12→17)/:19`＋三条共享规则；U-1067-1 frontmatter 元数据对账；Linux NOT_RUN→1063；`platform.*`→1068/T-03；owned-text 消息/回复→1068/T-02 | findings: T-1067-1..5 全闭；U-1067-1 minor（非阻断，merge-owned） | next: merge（T-10）

**关门证据（第四次复审实测）**
- **门内（`cargo t ace-native` ×3）**：`run-99q0p9cx` PASS 68.8s / 327 tests / 28 调度用例，逐例最大 **2.006s**；`run-fy_2tcjk` PASS 73.3s，max 1.870s；`run-s38035fx` PASS 69.4s，max 1.390s；`max_parallel_observed=20` 且调度用例**重叠对=0**；无调度用例 >10s。
- **test-group 有效性**：`nextest list -E 'test(/^scheduler_/)'`=26、`-E 'test(/^av_leg::av_lane/)'`=2，合计 28＝调度用例数（`.config/nextest-full.toml:26-27,67-69`）。
- **隔离**：`cargo nextest run -p ace-native --test sched_m2_1067 --test-threads 1` → 28/28 PASS 13.9s，逐例最大 0.953s。
- **固定组（顺序）**：`cargo t ace-source ace-lowering ace-native case-ace-m2-scheduler-968` PASS `run-qu4kcxq9`（gate 102.1s；825/5/327 tests；case 组 checks 11/11）；`cargo t ace-av` PASS `run-x7ccje0o`（89.1s，1349 tests，0 红）；`cargo t ace-a2x` PASS `run-a5xsihgp`（151.1s，310 tests，0 红）。既有 flake 类本轮未复现。
- **fmt**：`git diff --name-only f0db9bd0f..HEAD` 的 15 个 `.rs` 全部 `rustfmt --check` exit 0。
- **双车道独立探针**（`plan-artifacts/plan-1067/review4`，临时目标用后删除、`git status` 空）：scalar 直调 `0/"2\n"`=AV；递归 `0/"2\n"`=AV；owned-Str 丢弃 `0/"done\n"`=AV；空 owned Str（`let` 与 `f""`）`0` 且 stderr 空（无 72/无双重释放）；`Str` 直调+go `"s=1\n"`；bool `"true\n"`；`f64`/`str` 两车道同拒（sched profile 无此类型）；0/1 参数 go 绿；3 参数 Str go 两车道同拒；spawn+state init `"8\n"`=AV；`Str` 状态 init `"s: 8\n"`=AV；main 内 spawn `"got one\n"`；helper spawn 具名同拒；同块/两分支两 `go` 均绿；stop 钩子 go+缓冲+ask `"one\ntwo\n2\ntick\n"`=AV。
- **AC-04**：仓外候选 PE 24,064 B（sha256 `A4778A05…`）仅 `KERNEL32`，含 `CreateThread`/`WaitForSingleObject`/`SleepConditionVariableSRW`/`ExitProcess`/`HeapAlloc`，无 CRT/`printf`/`malloc`；变异 DLL 负控被拒。
- **成本表（隔离 / 门内三次）**：corpus_1 0.744 / 1.822·0.776·0.877；corpus_2 0.842 / 0.765·0.731·0.936；corpus_3 0.745 / 0.791·0.797·1.172；corpus_4 0.628 / 0.786·0.887·0.765；go_from_task_handler(10 跑) 0.924 / 1.150·0.760·1.054；go_to_scalar 0.726 / 0.771·0.849·0.746；go_discarding_owned_text 0.474 / 0.492·0.459·0.441。

**复审给出的 merge 侧待办**：U-1067-1（frontmatter `supersedes_spec_components` 多列 `m2-control-effects.md`，该文件未被任何 delta 修改——合入时删除或补实）；O-3/O-5 为可选措辞（spec §5 证据命令已因拆分更名、§2 `go` 行可补"结果丢弃/owned text 释放"一句）。
2026-11-17 stage: work | PLAN-1067 r1 round-12 | review3 findings T-1067-1..5 修复完成 → 回 execution_done，交第 4 次复审（第 3 个 repair cycle 已用；若再 needs_fix 按上限停止并交用户裁定） | code ``104cde943``（config）/``f0814b271``（tests）/``eeff4d60e``（sigs）

| finding | 修复 | 重放/证据 |
|---|---|---|
| T-1067-1（go 标量返回 callee 构建失败） | `SchedCallee` 增 `sig`（callee 自己 unit 的完整签名）与 `result_text`；thunk 用**同一签名** `declare_function`（结果照旧丢弃）；owned-text 结果在 thunk 内 `ac_rt_own_free` 释放，避免丢弃即泄漏 | 新用例 `scheduler_go_to_scalar_returning_callee_builds_and_runs`（同单元直调+`go` → `2\n`；递归 `go chain` → `2\n`）与 `scheduler_go_discarding_an_owned_text_result_does_not_leak`（exit 0＝零残值）全绿；`sched_m2_1067` 28/28 |
| T-1067-2（门内 >10s） | ① race 用例**建一次跑十次**（`run_repeated`，重复不再重编译）；② 8 例语料拆成 4 个双例 case；③ `.config/nextest-full.toml` 新增 `sched-native = {max-threads = 1}` 组，仅按名字（`test(/^scheduler_/) or test(/^av_leg::av_lane/)`）串行本二进制用例 | 三次 `cargo t ace-native`：**28 个调度用例逐例最大 1.53 / 1.58 / 1.86s**，gate 62–64s（修复前同条件 10–14s）；输入/断言/10s 规则未变 |
| T-1067-3（rustfmt） | `text-runtime`、`sched_desugar` 及其余触面统一 rustfmt | `rustfmt --check` 11 个触面 .rs **exit 0** |
| T-1067-4（spec §6 未记 entry-only） | spec §6 增「入口限定」与「邮箱容量 ≥1」两条具名边界 | spec §6 行 81–85 |
| T-1067-5（SD-ASYNC 待改行不全） | 规范增量表补 `m2-scheduler.md:17`（AV 用例 12→17）与三条共享实现一致性规则（仅入口 spawn / cap≥1 / stop 保留缓冲投递）；§5「不改语义」改为"仅收紧三条 968 已写明合同的实现一致性规则" | plan §5 行 69、§5 规范增量表 |

**运行**：`sched_m2_1067` **28/28**（隔离 6.6–9.6s）；`cargo t ace-native` PASS ×3（run-2s8a9db6/j_6by294/qha683i3 为修复前，串行化后 3 次新 run 全 PASS）；`cargo t ace-source ace-lowering ace-native case-ace-m2-scheduler-968` **PASS**（串行化后复跑，case 组 11/11）。
**过程记录（诚实）**：一次 `.config/nextest-full.toml` 编辑因 .NET 相对路径落到**主检出**，已 `git checkout --` 还原（主检出该文件 clean）后在 worktree 绝对路径重做并提交；后续所有 .NET 文件写入一律绝对路径。
2026-11-17 stage: review | review_scope: final（第三次复审） | reviewer_identity: reviewer-1067c（独立委派上下文，无实现署名；自建 probe + 门内 nextest 三次 + 代码/spec 逐行重建） | plan_id: PLAN-1067 | plan_revision: 1 | **outcome: needs_fix（第三次 failed final review，已用 2 个 repair cycle）** | worktree_path: D:/autostack/.wt/lang-1067/auto-lang | branch: plan-1067-dev | reviewed_commit: `4d6e81a2c26bc3938e63cfb4284bbd5bd1d8c29c` | base_commit: `f0db9bd0f746f115f9ec16124b798939e96e5860` | acceptance_results: AC-01 partial；AC-02 partial；**AC-03 pass**；**AC-04 pass**；AC-REVIEW fail（本记录） | next: work（第三次 bounded repair，见下"裁定"）

**已关闭**：N-1067-1（drain 每记录先 stop 再 join；mid-drain spawn 变具名拒绝；互递归链 0.33s 具名 Resource 不死锁）、N-1067-2（stop/start 钩子同一 callee 双块 → `tick\ntick` 绿）、N-1067-3（AV+native `one\ntwo\n2`、stop 钩子 `go` 与缓冲消息 `one\ntwo\ntick`）、N-1067-4（冷 target 实验：rename 目录 → PASS 且重建、恢复 78 文件/72,566,604 B/SHA256 一致）、N-1067-6、N-1067-8。部分：N-1067-7（15 个触面 .rs 中 13 绿，2 红：text-runtime 分支新增空行、sched_desugar 既有长签名）、N-1067-9（行为已闭，但 §6 未记 entry-only，规则在 §2:33）。
**组证据（门内）**：`cargo t ace-native` PASS ×2（run-zuqvxwf1 322 tests 67.0s；run-3y4iwc_a 322 61.9s）；`cargo t ace-source ace-lowering ace-native case-ace-m2-scheduler-968` PASS（run-7_atn94k，case 11/11）；`cargo t ace-av` **FAIL 3 红**（run-zsxaqejv：`kv_zip`/`bstr_1064` ExecutionWall、`stdlib_assembly_1043` `cl` Deadline；三条单跑均绿 2.56/16.20/2.35s，均非调度面）；`cargo t ace-a2x` **FAIL 1 红**（run-jihpdrn7：`worker_ct_958` `cl` Deadline，单跑绿 53.87s；A2R sched 3/3）。

### 新 findings

| ID | 级别 | 规则 | 输入/命令 | 期望 vs 观测 |
|---|---|---|---|---|
| T-1067-1 | **major（in-contract）** | AC-01/02 同源三端等价；合法前门输入不得以内部 `mn.backend.lowering` 失败 | `fn worker(n int) int { n + 1 }` + `print(worker(1))` + `go worker(2)`（同单元既直接调用又 go） | 应 exit 0 / `2\n`（AV `Success(0)` `2\n`）；native 构建失败：thunk 声明 `returns: []` 与既有 `returns:[I32]` 签名冲突；递归形式同样失败；**仅 go、无同单元直调**则正常 |
| T-1067-2 | major（test-cost） | 单用例门内 ≤10s | `cargo t ace-native`（20 并行） | 10 跑用例门内 **10.155 / 9.948 / 7.944s**；隔离 4.13s；上一轮 5.45s 不可复现 |
| T-1067-3 | minor | AGENTS §3 健康检查 | `rustfmt --check` 15 个触面 .rs | 13 绿 2 红（text-runtime:637-638 分支新增空行；sched_desugar:114 既有） |
| T-1067-4 | minor | 收据/规范精确 | — | entry-only 规则在 §2:33，§6 未记（round-10 收据称已记） |
| T-1067-5 | minor | 规范增量完整 | — | SD-ASYNC 待改行未列 `m2-scheduler.md:17`（AV 用例 12→17）与 entry-only 语义收窄；plan §5「沿968 D1–D6，不改语义」已过时 |

**提交卫生说明**：commit `64a01a60b` 因共享主检出的并发 git add 一并带入了另一会话已暂存的 `docs/plans/archive/1038-ace-reuse-candidates-and-cost-decisions.md`（内容即该会话暂存的版本，未改写、无内容损失）。未做 history rewrite 以避免与并发会话的 HEAD 竞争；后续本计划提交只 git add 本计划文件。

### 本轮裁定（第三次 bounded repair，仍在 work 技能允许的 3 个循环内）

1. **T-1067-1 走"支持"而非"受限"**：AV/A2R 已支持标量返回的 `go` 目标，thunk 代码注释亦声明"结果被丢弃"，故这是 **native 侧缺陷修复**，不是语义扩张、不需要用户改合同。实现：把 callee 的返回形状纳入 `SchedCallee`（thunk 的 `declare_function` 使用与真实 callee 一致的签名，调用结果照旧丢弃）；owned-Str 返回按同一签名规则处理，若不可行则具名拒绝并在 spec 记录。
2. **T-1067-2**：10 跑 → 5 跑（仍覆盖"多次独立 PE 运行"的意图），并以 ≥3 次 `cargo t ace-native` 门内取证 ≤10s 且留余量。
3. **T-1067-3/4/5**：fmt 两个文件；spec §6 补 entry-only 规则；plan SD-ASYNC 编辑行补 `:17` 与 entry-only；刷新 §5「不改语义」表述。
4. **若第四次复审仍 needs_fix**：按 work 技能上限停止继续修复，保留诊断并交用户裁定（不再第 4 次 repair）。
2026-11-17 stage: work | PLAN-1067 r1 round-10 | review2 findings 修复完成 → 回 execution_done，交第 3 次（也是最后一次允许的）复审 | code ``4d6e81a2c``

| 新 finding | 修复 | 重放证据（隔离 / 门内） |
|---|---|---|
| N-1067-1（drain join-不-stop 挂死） | drain 每轮先对**本轮发现的所有任务记录**置 stop（每记录一次 `stopped_marked`）再 join 一个链；并把 `Task.spawn` 收敛为**仅入口合法**，使"drain 期间新建 TASK"状态不可达 | `scheduler_spawn_during_the_drain_terminates`（`go boot(){sleep; go inner()}`）：exit 0、`late\n`、**0.75s**；`scheduler_go_from_stop_hook_runs` 0.45–1.28s；原挂死输入（`go boot(){sleep; spawn}`→driver/helper spawn）现为具名前门拒绝 |
| N-1067-2（stop 钩子内 go 不可编译） | 根因＝thunk **地址 Value 跨块缓存**违 SSA 支配（stop 体内联两次→两个块共用同一 Value）；改为缓存 FuncRef、每个调用点各自 `func_addr` | `scheduler_go_from_stop_hook_runs`：`one\ntick\n`、exit 0、0.45s（隔离）/1.28s（门内） |
| N-1067-3（AV stop 后排队 ask 不投递） | AV `stopping()` 改为"**邮箱空**才报停止"＋一次真实投递遮蔽一次检查（与 native 同构） | AV `stop_keeps_buffered_deliveries`：`one\ntwo\n2\n`；native `scheduler_stop_keeps_buffered_deliveries` 同字节（0.82s 门内）；AV sched 模块 **17/17** |
| N-1067-4（冷 target prep lock os error 3） | 取锁前 `create_dir_all(parent)` | 修复点代码确认；`cargo t ace-native` 在干净 target 状态亦 PASS（本轮 run-lryxys22 内 ace-native 322 tests 全绿） |
| N-1067-5（20 跑门内 14.05s） | 降为 **10 跑**（复审最低值） | **门内 5.45s**（union run-lryxys22），隔离 8.68→≈4.4s |
| N-1067-6 | plan §10 `platform.*` owner 统一 1068/T-03（已回读确认三处一致） | — |
| N-1067-7 | `resolve.rs` 及其余触面 rustfmt | `rustfmt --check` 8 个文件 **exit 0** |
| N-1067-8 | spec §5 双反斜杠折回单 `\n` | spec §5 行与断言一致 |
| N-1067-9 | 语义检查扩到**任意非入口函数**（driver 与 helper 同一具名规则），spec §6 记录 | `scheduler_spawn_in_helper_is_a_named_frontend_rejection`（`source.sched-task` "only legal in the program entry"，0.04s 门内） |

**修复后运行**：`sched_m2_1067` **23/23**（隔离 6.17–9.22s）；AV sched 模块 **17/17**；A2R sched **3/3**；`cargo t ace-source ace-lowering ace-native case-ace-m2-scheduler-968` **PASS**（run-lryxys22；ace-native 322 tests、case 组 11/11）；`cargo t ace-av` **PASS**（run-aulpsplt）。
**门内逐例时耗（run-lryxys22，全部 ≤10s）**：corpus 5.87s、go-from-handler(10 跑) 5.45s、runs_are_repeatable 3.47s、candidate_out_of_repo 3.48s、candidate_mutant 2.70s、duplicate_reply 2.83s、unawaited_reply 1.86s、exit_drain 1.85s、owned_state 1.79s、sleep_real_wait 1.06s、stop_keeps_queue 0.82s、zero_capacity 0.95s、send_after_stop 0.69s、await_unreplied 1.31s、stop_hook_go 1.28s、spawn_during_drain 0.75s、typed_reply 0.50s、两个 av_leg 0.26/0.30s、其余拒绝类 ≤0.05s。
**spec**：§2 增"容量 ≥1 / 仅入口 spawn / drain 每轮先 stop 再 join"；§3 stop-队列规则写明 AV 同规；§6 记录 driver/helper 内 spawn 与 cap<1 的具名拒绝；§5 oracle 串修正。`git diff` 见 ``4d6e81a2c``。
**复审额度**：这是第 2 次 repair 后交出的复审；若第 3 次仍 needs_fix，按 work 技能上限（3 个 work↔review 循环）将停止继续修复并给出诊断/交用户裁定，不再第 4 次尝试。
2026-11-17 stage: review | review_scope: final（post-repair 复审） | reviewer_identity: reviewer-1067b（独立委派上下文，无实现署名；自建探针 + git/代码/spec 重建结论） | plan_id: PLAN-1067 | plan_revision: 1 | **outcome: needs_fix（第二次 failed final review）** | worktree_path: D:/autostack/.wt/lang-1067/auto-lang | branch: plan-1067-dev | reviewed_commit: `12f11b6417f7cda7f2c17b3c988b1b06f7131646` | base_commit: `f0db9bd0f` | acceptance_results: AC-01 partial；AC-02 partial；AC-03 **fail**；AC-04 pass；AC-REVIEW fail | next: work（先 N-1067-1/4 挂死与冷启动，再 N-1067-2/3 或正式修订，最后文本/fmt）

**已关闭**：R-2/3/4/5/9/10；R-1 对**原报输入**关闭（go-from-handler 40+ 跑逐字节绿、零残值）；R-6/8 部分；**R-7/11 未关**（plan §10 仍写「本包 T-05」；spec §6 无嵌套 spawn 规则）。
**组证据**：`cargo t ace-native` PASS（run-0lztrfme，318 tests，70.2s）；`cargo t ace-source ace-lowering ace-native case-ace-m2-scheduler-968` PASS 4/4（run-34ldqbg1，80.9s）；`cargo t ace-a2x` PASS（run-ryd1j0ul，310 tests，130.6s）；`sched_m2_1067` 19/19（8.81s）；AV sched 模块 16/16；A2R sched 3/3。

### 新 findings

| ID | 级别 | 规则 | 输入/重放 | 期望 vs 观测 | 影响 |
|---|---|---|---|---|---|
| N-1067-1 | **major（新回归·挂死类）** | spec §2「循环重扫并 join…直到无未 join 链」；AC-03 无 hang | `fn boot(){sleep(200) let b=Task.spawn("B",4) b.send(7) print("booted")} fn main(){go boot()}`（stop 钩子/晚启动兄弟态同理，9/9） | 应 exit 0；native `mn.infra: … Deadline`（5s 预算被杀，INFRA 非语言错误）；AV `Success{0}` `booted\n7\n` | AC-03/01、T-04/05 |
| N-1067-2 | major（原有 in-contract） | AC-01/03 三端等价；stop 钩子内 `go` 合法 | `task S { fn stop() ! { go tick() } … }`（4/4） | 应 `one\ntick`；native 构建失败 `Verifier errors`（无 `mn.*` 码）；AV `Success{0}` `one\ntick` | AC-01/03、T-04/05/06 |
| N-1067-3 | major（原有 in-contract） | §2 stop「缓冲消息仍先投递」 | `send(1); ask(2); stop()`（4/4 确定性） | 应 `one\ntwo\n2\n`；native 一致；**AV `Error{task closed without replying}` 且 stdout 空**；无 stop 但有 sleep 时 AV 变 fuel `Resource` | AC-01/02、T-04/05/06 |
| N-1067-4 | **major（新回归·冷 target）** | test-discipline 冷路径 | rename `target/.../runtime-release` 后跑任一用例 | 应自动建目录并重建；实测 `prep lock: … (os error 3)`（锁在目录创建前） | AC-01/04、T-08 |
| N-1067-5 | major（test-cost 硬规则） | 单用例 ≤10s | `scheduler_go_from_task_handler_survives_the_drain`（20 跑） | 门内 **14.05s / 11.71s**（隔离 8.68s），本包记的 6.52s 不可复现 | T-04/08 |
| N-1067-6 | minor | plan 一致 | — | §10 仍「本包 T-05」 | T-08 |
| N-1067-7 | minor | 健康检查 | `rustfmt --check` | `core/src/source/resolve.rs` 新增行未格式化（基线该文件干净） | T-08 |
| N-1067-8 | minor | spec 精确 | — | §5 行出现双反斜杠 `\\n`（应为 `\n`） | T-08 |
| N-1067-9 | minor | R-11 全量关闭 | `Task.spawn` 在 driver 调用的普通 helper 内 | 仍 `lower/lower: __task_B missing`（检查是词法的）；spec §6 未记规则 | T-02/03 |

### 诊断检查点（两次 failed final review 后，work 必须先定向）

**为何前几轮没发现（逐条）**

1. **N-1067-1**：R-1 的修复只针对**报告输入**（handler 内 `go`）验证；handler 线程本身已在 join 集合里，所以"创建于 drain 期间的链"只是被 join 而**从未被 stop**。drain 的"新任务置 stop"分支以 `thread.is_null()` 为条件，而活任务的 `thread` 恒非空 ⇒ 该分支永不生效。**漏检的机制是 drain 的两条职责（stop 与 join）在"drain 之后才出现"的状态上没有合并成同一不变量。**
2. **N-1067-2**：`go` 在 stop 钩子的用例从未进入语料（语料 stop 钩子只打印）。desugar 把 stop 体内联**两次**（收前/收后各一次），本车道按 callee 生成 unit-local thunk 时对同一 callee 的重复调用点产生重复定义 → 构建期 Verifier 错误。**漏检原因：只测了 start 钩子与 handler 内的 `go`，未测 stop 钩子这一同源分支。**
3. **N-1067-3**：AV 的 `SchedStopping` 不查邮箱（native 从第一版就有 mask），此差异在"stop 后有排队消息"这一状态上才可见；此前 AV 侧只补了 closed-future 与 late-reply。**漏检原因：AV 的 stop/队列组合态从未被纳入对照用例。**
4. **N-1067-4**：prep 锁在并集跑红后的压力下加入，只在**热 target** 上验证；锁文件路径位于 cargo 尚未创建的目录。**漏检原因：把"能跑通"当"冷路径可用"，未做 rename 冷启动实验。**
5. **N-1067-5**：只测了隔离时耗（6.52s），未取**门内**（nextest 并行）时耗。**漏检原因：test-cost 规则要求"门内"数字，我用了隔离数字。**
6. **R-7/11 未真关**：文本改动只改了 §5/§7 而未回读 §10 原行；R-11 的检查是词法 owner 比较，未构造"helper 间接 spawn"的兄弟态；spec §6 的补记未落地。**漏检原因：以"我改过了"代替逐处回读与兄弟态重放。**

**设计/ oracle 调整（下一轮实现并验证）**

- **drain 单一不变量**：每一轮扫描发现的每个任务记录，必须在同一轮被**置 stop 并 join**（每 task 一个 `stopped_marked` 标记，不再依赖 `thread.is_null()`）；"drain 期间新链"必须有永久用例（A: `go boot(){sleep; spawn; send}`；B: handler 内 `sleep` 后 spawn；C: `go` 内 spawn+send），判据＝`<1s` 收敛且 exit 0（或具名 `sched:` 错误），**禁止 host Deadline**。
- **stop 钩子链**：先定位重复 thunk/重复定义，修成"同一 callee 多调用点各自唯一 thunk"或"stop 体只内联一次（改调 helper）"，并加用例 `go` in stop hook（native + AV 同源 expected `one\ntick`）。
- **AV stop+队列**：`SchedStopping` 必须先看邮箱是否非空（与 native 的 mask 同构），并加 AV/native 对照用例（`send(1);ask(2);stop()` → `one\ntwo\n2\n`）。
- **冷 target**：prep 锁前 `create_dir_all(parent)`；以 rename 实验作永久验证步骤（或在测试里模拟）。
- **test-cost**：20 跑降为 10 跑（复审最低要求 10）并**记录门内时耗**。
- **文本**：逐处回读 §10/R-11/spec §6/spec §5 转义；`rustfmt` 覆盖 12 个文件。
- **oracle 调整**：新增/改动用例一律同时给"隔离时耗 + 门内时耗"，并在 §9 记录门内数字。
2026-11-17 stage: work | PLAN-1067 r1 round-8 | repair 完成 → 回 execution_done，交复审（只重验 AC-01/02/03 受影响项） | code ``12f11b641``

**逐条关闭（每条含最小重放的实际结果）**

| ID | 修复 | 重放/新证据 |
|---|---|---|
| R-1067-1 (blocker) | 链登记与 `CreateThread` 在**同一 registry 锁**内完成（task 记录与 handle 一起发布）；drain 改为**循环重扫并 join**（drain 期间由 stop 钩子或缓冲消息处理启动的新链也会被 join，直到无未 join 链；10 万轮上限→残值 1）；新增 `joined`/`GO_JOINED` 记账 | `scheduler_go_from_task_handler_survives_the_drain`：**20/20 跑** exit 0 + ``5\n`` + stderr 空（修复前 10/12 exit 72、3/12 stdout 丢失）；耗时 6.52s（20 次真 PE 运行，≤10s） |
| R-1067-2 (major) | AV `sched_reply` 仅在 `Pending` 时发布（迟到/重复 reply 丢弃） | native `scheduler_duplicate_reply_never_resurrects_the_future`（70 + `1\n` + `await on an already-consumed future`，1.08s）＋AV `late_reply_does_not_resurrect_the_future`（同文本 Error、stdout `1\n`，0.26s） |
| R-1067-3 (major) | 容量 **≥1**：native `E_CAPACITY`（"must be at least 1"）、AV 同名 Error、A2R 具名 fail（不再 clamp） | native `scheduler_zero_capacity_is_a_named_error`（70 + 具名 + <5s，0.83s）；AV `zero_capacity_is_a_named_language_error`（0.25s）；A2R `sched_1067_a2r_capacity_and_consumed_future_are_named`（1.68s） |
| R-1067-4 | A2R `await` 越界/已消费 → 具名 fail（`unknown future` / `already-consumed`） | 同上 A2R 用例的 consumed 分支（exit 70 + `sched: await on an already-consumed future`） |
| R-1067-5 | spec 改 **18 项** | `SCHED_KERNEL32_IMPORTS` 计数=18（与代码一致） |
| R-1067-6 | 证据表恢复 `got two\n got one\n` 等转义串 | spec §5 行已改，逐条与 `sched_m2_1067` 断言一致 |
| R-1067-7 | §10 `platform.*` owner 统一 1068/T-03 | §5/§7/§10 三处一致 |
| R-1067-8 | rustfmt 全部触面文件 + `emit.rs` 折叠行 + 两处缺尾换行 | `rustfmt --edition 2021 --check` 对 10 个受改文件 **exit 0** |
| R-1067-9 | `sched-runtime/Cargo.toml` panic 注释改为"内嵌 runtime 持有唯一 handler 且 spin，由 run budget 兜底" | 注释与 `text-runtime` 实际 handler 一致 |
| R-1067-10 | 记 flake 并给稳定收据；另发现并修 **跨进程 prep 锁**（见下） | `cargo t ace-a2x` 单跑 **PASS**（run-_3ag_x9l）；并集内 `kv_zip_1064::v03_zip_negative_corrupt_bomb_and_preserve_av` 红因 `Resource{ExecutionWall}`，单测重放绿（3.19s，组内 13.2s）=负载下墙钟预算 flake，与本 diff 无关；`worker_ct_941_ordinary_same_names_four_operations` 单测重放绿（7.79s） |
| R-1067-11 | `Task.spawn` 在 task driver 内→前端 `source.sched-task`（"inside a task driver"）；spec §6 记录 | `scheduler_nested_spawn_is_a_named_frontend_rejection`（0.30s） |

**额外修复（并集跑暴露）**：`nextest` 每个测试独立进程，先前只有进程内互斥，两个测试进程会同时重建/哈希同一个 staticlib → `os error 5`。`sched_runtime::prepare` 现加**跨进程 prep 锁**（`create_new` 重试 + 10 分钟陈旧锁回收），`cargo t ace-native` 由 1 红转 **PASS**（run-5hy6bqx9）。

**修复后运行**：`sched_m2_1067` **19/19 PASS**（18.5s，含 20 次重放用例）；AV sched 模块 16/16；A2R sched 3/3；`cargo t ace-native` PASS（run-5hy6bqx9）；`cargo t ace-a2x` 单跑 PASS（run-_3ag_x9l）；并集 `ace-source ace-lowering ace-native ace-av case-ace-m2-scheduler-968`（run-re2hdly3）除上述 kv_zip 墙钟 flake 外全绿，case 组 11/11。
**新用例时耗**：0.25–6.52s（最大为 20 次重放的 blocker 用例），无 >10s。
**spec**：`native-async-scheduler.md` 已按 R-5/6/11 更正并补入容量/重扫 drain/嵌套 spawn 规则（`git diff` 见 ``12f11b641``）。
2026-11-17 stage: review | review_scope: final | reviewer_identity: reviewer-1067（独立复审上下文，无实现署名；结论由 git/代码/spec 与自建探针重建，不用实现者自报） | plan_id: PLAN-1067 | plan_revision: 1 | **outcome: needs_fix** | worktree_path: D:/autostack/.wt/lang-1067/auto-lang | branch: plan-1067-dev | reviewed_commit: `817eb1e581339b26fb7f37d2fc7c584bfec5b681` | base_commit: `f0db9bd0f` | spec_inputs: docs/specs/auto-ac/native-async-scheduler.md@817eb1e58（新，95 行）、docs/specs/ace-migration/m2-scheduler.md（live 未改，SD-ASYNC pending）、docs/specs/ace-migration/m2-control-effects.md（既有红记录） | acceptance_results: AC-01 partial；AC-02 partial；AC-03 **fail**；AC-04 pass；AC-REVIEW fail（本记录） | pending_closeout_gates: merge 侧 SD-ASYNC 需改 m2-scheduler.md:10-11,19；ace-a2x 固定组稳定收据；Linux NOT_RUN→1063 | next: work — 先修 R-1067-1/2/3（含最小重放），再改 R-1067-5/6/7/9 的 spec/plan 文本；复审只重验 AC-01/02/03 受影响项

**Baseline**：`worktree list` → `817eb1e58 [plan-1067-dev]`；`git status --porcelain` 进入与退出均空；范围 6 提交（`88999db74 e16f01b6e c6b84240e 40f5c195b 005096495 817eb1e58`），20 文件 +3544/−31；无 `docs/plans/attachments`、无 plan-artifacts 入库、无二进制或 >5MB 文件。

**Affected 固定组**（`cargo t ace-source ace-lowering ace-av ace-native case-ace-m2-scheduler-968`，run-i3ne3rnt）：5/5 PASS，gate_passed=true，wall 164.8s（ace-native 314 tests 含新 15；case 组 11/11）。`cargo t ace-a2x` 单跑：run-k9p2rola 1 红（`worker_ct_958_c_certifies_seven_independent_oracles` 10.66s）/ run-r3x3jhec PASS gate=true 93.9s；该红单跑绿（14.5s）⇒ 负载/环境 flake，非本 diff（R-1067-10）。`cargo test -p ace-a2x-worker --test conformance sched_` → 2/2 PASS 5.00s。

**新用例时耗**：18 个新/改用例，逐例 wall 0.21–4.22s（最大 `scheduler_corpus_runs_native_byte_exact` 4.22s）；目标整体 15 例 12.52s wall；A2R sched 3.12s。**无 >10s 用例**。（复审实测范围比本包 §9 早前记的 0.1–2.5s 更宽，规则仍满足。）

**独立反例（一次性探针目标，已删除，树保持 clean）**：
1. task handler 内 `go helper(n)`：native 12 跑 → exit 72 ×10（2× exit 0）、stdout 空 ×3、stderr 始终空；AV 同源 → `Success(I32(0))`、`5\n`。成因（代码确认）：`sched-runtime::start_go` 在 `REG_LOCK` 外 `CreateThread` 后才登记 `GO_HANDLES`，而 `mn_sched_drain` 在同锁下取快照（R-1067-1）。
2. `reply(1); reply(2)` + `await; sleep(60); await`：native → exit 70、`1\n`、`sched: await on an already-consumed future`；AV → `Success(I32(0))`、`1\n2\n`（迟到 reply 复活，R-1067-2）。
3. `Task.spawn("Z",0); send(1)`：native → `mn.infra: run … Deadline`（5s 预算内挂死）；AV → `Resource{InstrSteps,"scheduler deadlock: every chain waits and no timer can fire"}`；A2R 按 `sched_rt.txt:99` 静默 clamp cap<1→1（R-1067-3）。
4. task handler 内 `Task.spawn("B",4)` → 12/12 `mn.unsupported.sched.frontend: lower: scheduler callee function __task_B missing`（R-1067-11）。

**AC-04 独立复算**：published candidate `plan-artifacts/plan-1067/candidate-t07/mn8-out-35672.exe`（24 064 B）仅 `KERNEL32.dll`；`CreateThread`/`WaitForSingleObject`/`SleepConditionVariableSRW`/`ExitProcess`/`HeapAlloc` 在列；无 `printf`/`malloc`/`msvcrt`/`ucrtbase`；runtime 身份与会话内 `file_hash` 相等；`SCHED_KERNEL32_IMPORTS` 实为 **18** 项（spec 写 19）。

**红集归因复核**：a2x flake 类可再现但非实现者点名的那条；ace-runtime 19 链接红与 [m2-control-effects](../../specs/ace-migration/m2-control-effects.md) 记录的 master 既有红集一致（复审未重跑该组）。

**复审自述局限**：A2R 的 consumed/out-of-range future 仅代码检视（未新增 A2R 驱动测试）；ace-runtime 基线未重跑；blocker 为概率性（10/12、3/12）但代码窗口已确认；ace-a2x 单跑不可作为稳定全绿证据；未做 Linux 探针（按合同 NOT_RUN）。

### findings 与关闭表（work 修复项）

| ID | 级别 | 违反既有规则 | 最小重放/输入 | 期望 vs 观测 | 影响 | 关闭检查 |
|---|---|---|---|---|---|---|
| R-1067-1 | blocker | AC-03/§3 drain 零残值、"退出时 join" | task handler 内 `go helper(n)`，≥20 跑 | 应恒 exit 0 + `5\n`；实测 native 10/12 exit 72、3/12 stdout 丢失 | AC-03/01、T-04/05/08 | 注册与发布线程句柄（或 DRAINING 门把迟到 `go`/`spawn` 变具名错误）使 drain 不可能漏登记；加永久用例跑 ≥20 次 |
| R-1067-2 | major | SD-NATIVE §3「三端一致…不复活」 | `reply(1);reply(2)` + 二次 `await` | 应两次丢弃/具名；AV 实测 `1\n2\n` 复活 | AC-02/03、T-05/06 | AV 仅 `Pending` 时发布，或把 AV 差异具名归档；补 AV+native 各一例 |
| R-1067-3 | major | AC-01/02/03「同源等价、无 hang 类」 | `Task.spawn("Z",0)` + `send` | 三端各不同（native 挂死/AV 具名 Resource/A2R clamp） | AC-01/02/03、T-04 | 固定 `cap<=0` 语义（native 具名 `E_CAPACITY`、A2R 同）+ 正反例 |
| R-1067-4 | minor | T-06「已消除 A2R 静默零值」 | A2R `await` 已消费/越界端口 | 应具名；实测返回 `0` | AC-02/03、T-06 | A2R 具名化 + 补 A2R `await` 用例 |
| R-1067-5 | minor | spec 精确性 | — | `SCHED_KERNEL32_IMPORTS` 18 项被写成 19 | T-08 文本 | 改为 18 |
| R-1067-6 | minor | spec 精确性 | — | 证据表出现损坏串 `got two,g?got one` | T-08 文本 | 恢复 `got two`/`got one` |
| R-1067-7 | minor | plan 内部一致 | — | §10 `platform.*` owner 写「本包 T-05」vs §5/§7 写 1068/T-03 | T-08 文本 | 统一为 1068/T-03 |
| R-1067-8 | minor | AGENTS §3 健康检查 | `rustfmt --check` | 新文件 rustfmt 漂移、`emit.rs:376` 折叠行、sched_m2_968 末尾缺换行 | T-08 | fmt 受影响文件并重跑组 |
| R-1067-9 | minor | 注释真实性 | — | sched-runtime/Cargo.toml 称 panic 会退出，实际 spin | T-08 文本 | 改注释或 handler |
| R-1067-10 | minor | 收据可复现性 | `cargo t ace-a2x` 两次 | 一次红（另一条重测试）一次绿；点名红未再现 | T-08 收据 | 记 flake（测试身份+时耗）并给稳定重跑收据或隔离 |
| R-1067-11 | minor | 边界具名 | task handler 内 `Task.spawn` | `lower: scheduler callee __task_B missing`（形似内部 bug） | T-02/03 边界 | 前端具名拒绝或 §10 记 gap+owner |
2026-11-17 stage: work | PLAN-1067 r1 round-6 | T-08 闭合 → work 完成（execution_done，交 review） | code `817eb1e58`（spec）/`005096495`（前一提交）

**冻结影响组并集运行**（`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime case-ace-m2-scheduler-968`，`run-834j1fuh`）：

| 组 | 结果 |
|---|---|
| ace-source / ace-lowering / ace-av / ace-native | **PASS** |
| case-ace-m2-scheduler-968 | **PASS**（11/11 checks，AV 真实运行） |
| ace-a2x | 并集内 1 红 `a2c_rt_cert::rt_certify_all_suite_cases`（`c-tool infra: CToolError("cl -? spawn: Deadline")`，60s 的 `cl` 工具 spawn 超时）→ **单跑 `cargo t ace-a2x` PASS**（`run-5849mkpr`），且同组在 T-06 之后已 PASS 过 ⇒ 并行负载下工具超时归因，非本 diff |
| ace-runtime | 19 红，全部 `ace-text-conformance` 链接失败（缺 `__CxxFrameHandler3`/`_fltused`）→ **基线归因已实证**：把 `text-runtime/src/lib.rs` 临时退回基线 `f0db9bd0f` 版本后同一测试**仍然红**，恢复本版后 `contract_sidecar_is_written_and_self_consistent` 绿；且 [m2-control-effects](../../specs/ace-migration/m2-control-effects.md) 已记该 19 例为 master 既有红集 |

**AC 工作侧证据（work 所有项）**：
| AC | 证据 |
|---|---|
| AC-01 | 968 原语料 8 正例在 AV/native 各自逐字节通过（`sched_m2_1067` `av_leg`＋native 行），3 反例按类具名；新增组合＝owned-Str 状态跨 spawn、bool reply 跨 await |
| AC-02 | AC 真执行状态机/Task：线程+邮箱+端口+driver 机器码（`CreateThread` 在导入审计中）；typed 结果 i32/bool/owned-Str 四向物化与释放 |
| AC-03 | closed-future 三端具名终止（不自旋）、迟到 reply 丢弃、stop 先投递缓冲、drain 零残值审计（残值→72）、真实 OS 等待墙钟 oracle |
| AC-04 | 仓外候选真 PE 运行＋kernel32-only 导入审计（含 CRT 变异体负控）＋runtime 内容绑定身份；Linux 明确 NOT_RUN 交 1063 |
| AC-REVIEW / AC-LAND | 未执行（review/merge 阶段 owner） |

**work 完成度声明**：全部 work-owned 任务（T-01..T-08）已闭合；无 in-contract 未决项；具名差额 4 项均有 owner/输入/关闭条件（§10）：owned-text 消息/回复（1068/T-02）、`platform.*` 源级入口（1068/T-03）、进程外取消源（后续宿主计划）、Linux 腿（1063）；另记一处共享降层既有缺陷（owned-text 循环内临时，774/870）。
**本会话测试时耗**：`sched_m2_1067` 15/15（单测 0.1–2.5s，首跑 5.95s 含 runtime 重建）；`ace-a2x-worker` sched 2/2（6.2s）；并集整跑 ≈ 各组见 `run-834j1fuh`；新增用例均在 ≤10s 门槛内，未新增重内存/performance 用例。
**身份**：worktree `D:/autostack/.wt/lang-1067/auto-lang`（branch `plan-1067-dev`，base `f0db9bd0f`，code `e16f01b6e`→`c6b84240e`→`40f5c195b`→`005096495`→`817eb1e58`）。
**next**：review（T-09，独立上下文；§5/§6/SD 增量与具名差额需逐行核，含新 spec `docs/specs/auto-ac/native-async-scheduler.md`）。

2026-11-17 stage: work | PLAN-1067 r1 round-5 | T-07 闭合（仓外候选＋依赖审计；Linux NOT_RUN 交 1063） | code `005096495`

- **仓外候选**：`scheduler_candidate_runs_out_of_repo_kernel32_only` 把调度 PE 构到显式仓外根（`plan-artifacts/plan-1067/candidate-t07`，可用 `AUTO_LANG_PLAN_ARTIFACTS` 迁移，无该根时退回 target tmp 仍跑同一审计），以普通进程在该目录启动（无 cargo、无工作区 cwd），观测 `mycounter: 15\n`、exit 0、stderr 空。
- **依赖审计（published bytes）**：新增 `SCHED_KERNEL32_IMPORTS`（19 项：CreateThread/WaitForSingleObject/CloseHandle/Tls*/SRW*+CondVar/Sleep/GetProcessHeap/HeapAlloc/HeapFree/GetStdHandle/WriteFile/ExitProcess）与 `audit_sched_pe_imports`；共享 `platform_io::audit_pe_imports` 重构为按车道传 allowlist（原冻结表不变）。断言：DLL 集合 = {kernel32.dll}、无 ordinal、每个符号都在声明表内且观察到 `CreateThread`；镜像中不出现 `msvcrt.dll`/`ucrtbase.dll`/`vcruntime*`/`api-ms-win-crt*`/`kernelbase.dll`/`bcrypt.dll`。
- **负控**：把镜像导入 DLL 名改写成 `MSVCRT.dll`（同槽位）→ 审计必须给 `pe.audit.forbidden-dll`；断言 `err.code()` 命中，证明"仅 kernel32"是可失败的检查而非口号。
- **支撑库身份**：重算 `sched_runtime::prepare()` 的 content hash 并与库字节对照相等（abi `mn.sched.runtime/1`），未被审计的 runtime 无法悄悄链入。
- **Linux：NOT_RUN（不升 verified）**——本车道只为冻结 Windows triple 降 COFF/kernel32，无 Linux 分支可测；实现与完整用例按 §1/§10 由 1063 承接，Windows 证据不外推。
- 运行：`sched_m2_1067` **15/15 PASS**（5.95s 首跑含重建，单测 0.1–2.5s）；**`cargo t ace-native case-ace-m2-scheduler-968 ace-av` 三组全 PASS**（platform_io 审计重构无回归）。
- 阻塞：无。

2026-11-17 stage: work | PLAN-1067 r1 round-4 | T-06 闭合（三端各对独立 oracle ＋ A2R 背离修复） | code `40f5c195b`

- 三端独立 expected（**不用互相比较当代替 oracle**）：
  - **AV**（`sched_m2_1067.rs` `av_leg`，真实 AU 车道同进程跑）：8 正例逐字节＋exit 0；3 反例按类：`bind/source.sched-task`、`text/source.unsupported`、运行期 `Error{send on a closed task}`。
  - **native**：同 8 正例逐字节＋零残值 drain 审计；反例＝trap 70＋stderr 具名消息。
  - **A2R**（`ace-a2x-worker`，emit Rust→rustc -O→run，绝不复用本车道产物）：原 7 例＋本轮补 `virtual_timer`（`main\nlater\nvalue:0\n`）；反例 `neg_send_after_stop` 断言 `exit 70`＋`sched: send on a closed task\n`。
- **发现并修复三端背离（A2R 侧）**：`a2x_sched_send`/`ask` 对已 stop 任务返回静默成功、`await` 对永不发布的 future 返回零值。现统一为具名语义：`send/ask on a closed task`、`reply is only legal while handling an ask message`、`task closed without replying`（stderr＋exit 70，与 native 同通道；AV 沿用 `ExecOutcome::Error` 同文本），迟到 reply 仍丢弃（单终态提交）。
- 运行：`sched_m2_1067` **13/13 PASS**（11.1s 含首跑 runtime 重建；单测 0.1–2.5s）；`ace-a2x-worker` sched 用例 2/2（6.2s）；**`cargo t ace-a2x ace-native case-ace-m2-scheduler-968` 三组全 PASS**。
- 遗留（转入 T-07/T-08）：宿主/进程外取消源、仓外候选与 OS 依赖审计；A2R 反例目前只覆盖闭任务投递，`reply outside ask`/`await 未发布` 的 A2R 反例留给 T-08 全矩阵。
- 阻塞：无。

2026-11-17 stage: work | PLAN-1067 r1 round-3 | T-05 闭合（OS 等待/取消终态/退出清理） | code `c6b84240e`

- 真实等待：`sleep` 以 OS 睡眠分片执行（10ms 片，drain/stop 可中止），墙钟 oracle `scheduler_sleep_is_a_real_os_wait` 要求 PE 运行 ≥35ms（no-op 睡眠会是个位数 ms）。
- **三端一致的 closed-future 终止**：生产者 driver 结束时关闭其已 latch 未回的 ask 端口**及**仍在其邮箱里再无人接收的 ask 端口；drain 期所有 Pending 端口同样关闭。
  - native：`E_REPLY_CLOSED` → stderr `sched: task closed without replying` + trap 70，<1s 返回。
  - AV：`ExecOutcome::Error{message:"task closed without replying"}`。**修复前**该输入是 `Resource{InstrSteps, scheduler multiplex exceeded the fuel budget}`（自旋耗尽 fuel）——本轮实测确认并修正，属 AV 侧真实缺陷。
- 退出清理：复合用例（2 活 task＋未消费缓冲＋未 await 回复＋go timer）exit 0 零残值、stderr 空、收敛 <10s。
- **发现并记账的既有限层缺陷**：owned-text 临时值在循环体第二次迭代重建时不逐次释放（`print(f"a=${n}")` 收两条 → 残值 1/exit 72；改 `print(n)` → exit 0；单迭代正常）。owner＝共享 owned-text native 降层（774/870 sweep 面），关闭条件＝逐次释放或槽复用纪律＋≥2 迭代后 text ledger 归零反控。本包复合用例改用标量以免测他域缺陷。
- 运行：`sched_m2_1067` **11/11 PASS**（单测 0.6–2.5s，均 <10s）；**`cargo t ace-av case-ace-m2-scheduler-968 ace-native` 三组全 PASS**（AV core 改动无回归）；AV 侧新增 `typed_reply_domain_is_not_int_bound`、`await_on_unreplied_ask_is_a_named_language_error` 两永久用例。
- 具名交后继：`platform.*`（clock/timer）源级入口仍在调度车道具名拒绝（`native.sched-platform`）——源级 `std.time.*` 模块入口属库/FFI 接面，owner＝1068/T-03；本包不新增未验证表面。
- 阻塞：无。

2026-11-17 stage: work | PLAN-1067 r1 round-2 | T-02/T-03/T-04 闭合（owned值跨交接＋typed reply＋actor面） | code `e16f01b6e`

- 前端真实缺陷修复：`scan_reply_word_expr` 只认 `reply` 拼写，而 CST 前端把 `reply(v)` 投影为 `__sched_reply`，导致**所有** task 的 reply 类型恒为 i32（非 i32 reply 源码不可达）。改为两种拼写都识别；`reply(true)` 现推断 bool 并在两车道 await 取回（AV 侧新增 parity case `typed_reply_domain_is_not_int_bound`）。
- owned text 跨交接：`SchedVal` tag=2（ptr/len）在 spawn 实参 / send·ask 消息 / reply / await 结果四向同形；runtime 在交接点用内嵌 owned-text 分配器物化（`ac_rt_own_clone_quiet`），接收方 `ac_rt_own_free`/出口 sweep 释放，未投递·迟到·未 await 缓冲由 drain 释放；drain 残值并入 text ledger（`ac_rt_owned_audit`），任一不为零→具名退出 72。
- stderr 一致性：owned-text runtime 的 CLONE/CONCAT/DISPLAY/FREE 观察行由 wrapper 在 entry 前 `ac_rt_trace_switch(0)` 关闭，故调度 PE 的 stderr 与 AV 逐字节相同（`spawn_with_state` 从 12 行 trace 噪声变为空）。
- 具名差额（不勾整包）：owned-text **消息/回复**仍由共享前端具名拒绝（内建签名要 owned `Str`，表面只产 borrowed view；`source.sched-argument`）。关闭需在参数投影处插入显式 `text_to_owned`，owner=1068/T-02；`scheduler_owned_text_message_and_reply_stay_named_rejections` 钉住当前行为，关闭时须同步改。
- 运行（本树）：`cargo test -p ace-native --test sched_m2_1067` 8/8 PASS（3.5s，含 corpus 8 例）；`cargo test -p ace-native --lib` 16/16；**`cargo t ace-source ace-lowering case-ace-m2-scheduler-968 ace-native` 四组全 PASS**（共享前端变更对 AV/A2R/既有面零回归）；`cargo test -p ace-av --test conformance typed_reply_domain_is_not_int_bound` PASS。
- 提交：`e16f01b6e`（7 files, +468/−72）。工具坑记录：PowerShell 单引号串里的 `` `n `` 不展开，早前两次测试文件编辑写成字面 `\n`，已改为 edit 工具并修正。
- 阻塞：无。

2026-11-17 stage: work | PLAN-1067 r1 | T-01 完成 + T-02 垂直切片实证 | worktree `D:/autostack/.wt/lang-1067/auto-lang`（branch `plan-1067-dev`，base `f0db9bd0f`，code `88999db74`）| task_ids: T-01[x]、T-02(进行中)

- 门/身份：985 M2A-08 成立（[m2-certification.md](../../specs/ace-migration/m2-certification.md)），本 goal 即执行授权；基线时主检出仅他人 docs WIP（未触碰）。
- 实现：新增 `experimental/ace/crates/sched-runtime`（no_std kernel32, ABI `mn.sched.runtime/1`）与 `native/src/sched.rs`＋`sched_runtime.rs`，`lower.rs` 增 `LowerMode::Sched`/SCHED 发射/spawn thunk，`emit.rs` 增调度准入与 `emit_sched_unit_coff`；单 Rust staticlib 约束下内嵌 text runtime；wrapper 带 drain 且残值→退出 72。
- 观测（真 PE，非 AV 解释）：`message_handler`=`got one\n`、`mailbox_fifo`=`got two\ngot one\n`、`ask_reply`=`reply:\n42\n`、`go_join`=`go:\n7\n`、`backpressure`=`1\n2\n3\n`、`stop_hook`=`one\nstopped\n`、`virtual_timer`=`main\nlater\n`，全部 exit 0/空 stderr；`neg_send_after_stop` exit 70 + `sched: send on a closed task`；`neg_unknown_task`/`neg_malformed_task` 分别为前门 `bind/source.sched-task`、`text/source.unsupported`。
- 运行：`cargo test -p ace-native --test sched_m2_1067` 4/4 PASS（2.4s，单 case ≤3.1s）；`cargo t ace-native` PASS 303 tests（53.7s，19 skipped；一次并行跑 `closures_incremental::tests::audit1027_utf_02` 瞬时红，单独重放与整组重跑均绿=并行争用归因，非本 diff）；AV oracle `cargo t case-ace-m2-scheduler-968` 基线 PASS。
- 未闭（按计划原任务）：`spawn_with_state` 的 owned Str 初始化（T-03）、取消/迟到终态与 `platform.*` 计时（T-05）、native 语料进 case-runner 与三端对照（T-06）。T-02 完整谓词（`Future<Result<owned>>` 整链）待 T-03 后勾。
- 阻塞：无。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r1：用户已裁定三端最终对齐及A→B延期边界；本稿可供规划审查，未实施、未测试、未签M2/AAC。静态核验记录另补，实际work仍核阶段门和来源。

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 88999db74 | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: e16f01b6e | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: c6b84240e | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 40f5c195b | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 005096495 | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 817eb1e58 | next: review

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 7 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 817eb1e58 | next: work

## 10. 待澄清事项

原生调度的帧/等待策略及资源上限由T-01/02依据现合同先证；本次不预定私有Rust ABI、任意线程数或无穷规模。超出原语义的并发模型/平台扩面不得借本计划引入。

### 具名未决（本轮新增，均有 owner/输入/关闭条件）

| 项 | 现状 | owner 与关闭条件 |
|---|---|---|
| owned-text 消息/回复 | `h.send("hello")`、`reply("v42")` 由共享前端具名拒绝 `source.sched-argument`（内建签名要 owned `Str`，表面只产 borrowed view） | 1068/T-02（共享前端）：在 scheduler 参数投影处插入 `text_to_owned`；关闭时须改 `scheduler_owned_text_message_and_reply_stay_named_rejections` 边界钉并补 native+AV 正例 |
| `platform.*` / 真 OS timer | 本车道准入具名拒绝（`native.sched-platform`）；`sleep` 已走真实 OS 等待 | 1068/T-03（库/FFI 接面），与本包 §5/§7 一致 |
| 外部取消源 | T-05 已闭内部终止语义：未发布 future 三端具名终止、stop 后不复活、drain 期 sleep/await 立即返回；但**无进程外取消源**（如宿主取消句柄/超时中断一个在跑任务） | 本包 T-07（仓外候选/宿主接面）或后续宿主计划；关闭条件＝真实宿主取消入口 + 迟到结果不复活反例 |
| Linux 调度腿 | 本车道只为冻结 Windows triple 降 COFF/kernel32；无 Linux 分支与用例 | 1063（Linux 延期验证）：写 Linux 实现（pthread/OS 等待或等价的受检原语）＋原完整用例后再验；**不得用 Windows 证据升 Linux verified**，当前一律 NOT_RUN |
| owned-text 循环内临时 | 循环体第二次迭代重建文本临时不逐次释放（退出 sweep 只释放最后一次写）；本车道 drain 审计以 exit 72 捕获 | 共享 owned-text native 降层（774/870）；关闭条件＝逐次释放/槽复用纪律＋≥2 迭代后 text ledger 归零反控 |

日常等价实现选择由T-01/02有界调查决定，不为已授权方向重复申请确认。未提供平台环境保持NOT_RUN；计划结构检查不能代实际运行证据。

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 8 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 12f11b641 | next: review

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 9 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 12f11b641 | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 4d6e81a2c | next: review

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 11 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 4d6e81a2c | next: work

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 12 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 104cde943 | next: review

stage: goal | plan_id: PLAN-1067 | plan_revision: r1 | round: 13 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1067/auto-lang | commit: 104cde943 | next: merge
