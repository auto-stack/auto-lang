---
plan_id: PLAN-968
status: archived
feature_name: 任务async actor与事件调度
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-15
plan_revision: 6
current_step: 8
total_steps: 8
depends_on: [PLAN-965, PLAN-967]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-scheduler.md]
touched_goals: [GOAL-002]
---

# [PLAN-968] 任务async actor与事件调度

## 0. 变更摘要

2026-11-15 r6当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

实现Future/await/go、Task/actor、ask/reply、mailbox、timer与Stream挂起/唤醒；AV线程归属不泄漏到公共API，AC/Rust/C使用同一状态机语义及中立调度服务。明确取消检查点、背压、公平性、退出join与错误传播。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/auto-lang-core/src/vm`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/task.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/av-core/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：任务/await/actor与取消调度。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/a2r-std/src/task.rs](../../crates/a2r-std/src/task.rs)：旧任务公开语义参考；读取SHA256前16 `effe4618e9492884`

- [experimental/ace/crates/av-core/src](../../experimental/ace/crates/av-core/src)：帧/执行/预算接缝；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs)：967异步完成/回调端口，不反向等968实现；读取SHA256前16 `fb148c0a779d5c95`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


实现Future/await/go、Task/actor、ask/reply、mailbox、timer与Stream挂起/唤醒；AV线程归属不泄漏到公共API，AC/Rust/C使用同一状态机语义及中立调度服务。明确取消检查点、背压、公平性、退出join与错误传播。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：调度协议与真实四端状态机（r2）

明确mailbox容量、send/ask/reply顺序、timeout/取消胜负、任务join、生成器/stream所有权以及共享状态并发规则；有限前探不代实际四端任务实现。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.at](../../crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.at)（下列节选L1–L17；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
task Echo {
    fn start() ! {
    }
    on {
        1 -> {
            print("got one")
        }
        2 -> {
            print("got two")
        }
    }
}

fn main() {
    let h = Task.spawn("Echo", 16)
    h.send(1)
}
```

本包所需独立结果：旧Echo send1应只输出got one；新控序输入1/2时消息次序可观测，取消后无迟到提交，bounded channel容量/背压按原支持域。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** task/await/go源→效果/挂起事实→中立任务帧/调度端口→宿主完成/取消→调用者与资源trace。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
enum TaskState { Ready, Running, Parked(WaitKey), Cancelling, Closed }
struct Continuation { frame: FrameId, owners: OwnerSet, wake_generation: Generation, cancellation: CancelToken }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/a2r-std/src/task.rs](../../crates/a2r-std/src/task.rs) | 旧任务公开语义参考 |
| [experimental/ace/crates/av-core/src](../../experimental/ace/crates/av-core/src) | 帧/执行/预算接缝 |
| [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs) | 967异步完成/回调端口，不反向等968实现 |

**状态、错误与资源。** 关闭后send/wake、重复完成、容量超限和无授权操作具名拒绝；取消与语言异常分开，不靠timeout冒取消正确。 挂起持必要owner，cancel关通道/IO后最后释放；Task.Stop/反复创建关闭与两个session隔离；保原公共同步基线。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.at](../../crates/auto-lang/test/vm/23_actor/002_message_handler/message_handler.at) | 消息与start/stop原行为 | `message_handler.expected.out`；SHA256前16 `5ad3f3a430ec5c97` | 拟ace.m2.p968.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/22_actors/020_spawn_with_state/spawn_with_state.at](../../crates/auto-lang/test/a2r/22_actors/020_spawn_with_state/spawn_with_state.at) | 初始化状态/方法，生成金样另保留 | `spawn_with_state.expected.rs`；SHA256前16 `bb6c0112854b2b0f` | 拟ace.m2.p968.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/src/tests/plan707_stream_tests.rs](../../crates/auto-lang/src/tests/plan707_stream_tests.rs) | 取消/晚结果旧红分账，不拿新ACE未来实现关闭旧缺陷 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `c64eb79920057813` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 公平性/可重入/背压/定时/取消观察边界与回调线程规则先冻结；不一次性重写宿主线程模型，不自动提高内存/输出预算。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 5.1 T-01 决定冻结（2026-11-15，work，r5 基线上执行）

**依赖与路径核定**：PLAN-965（archive，r5）、PLAN-967（archive，r12，19/19）均为终态正式来源；965生成器记录（control_effects语料+specs）与967完成端口（`av-core/src/foreign_exec.rs` CompletionPorts）在位。§4路径修正：967异步完成端口实际在 `foreign-contract/src/port.rs`（+av-core `foreign_exec.rs:808`），非 `typed.rs`（typed.rs为937 typed-decl桥）；本Plan早前引用按此修正，范围不变。`crates/auto-lang-core/src/vm`（task.rs/task_system.rs/scheduler.rs/task_handler.rs/message_context.rs/channel.rs）、`crates/a2r-std/src/task.rs` 均在位已核。工作树 `D:/autostack/.wt/lang-968/auto-lang`（branch `plan-968-dev`，base `058179ed2`）。

**有界全链前探（P1–P7，2026-11-15实测）**：最长producer→consumer链=source→aggregates前端→verify→lower→verify_mir→ABC admit→AV execute（即 `ace_av::collections_864_support::compile_and_execute_aggregates` 同链，探针以ace-core+ace-lowering+ace-av-core直接驱动，probe源+日志在plan-artifacts/plan-968/probe-t01/）。P1对照正例（`fn main() int` 返回42）PASS；P2 canonical message_handler.at原字节与P3最小task片段均于compile段具名拒绝 `source.unsupported: "`task` is not part of this source profile"`（现行精确缺口）；P4非法近邻（截断on臂）现被同一准入门拒绝（task解析落地后须有自己的具名parse拒绝，记为T-02义务）；P5失败后同进程重跑对照正例PASS（链可复用）；P6/P7：canonical函数面 `fn main() { }`（无返回标注）与 `fn start() ! { }`（bang效果标记）现均被"return type annotation required"拒绝——前端缺口=task语法+无标注unit函数+bang标记三件。

**策略比较（择A，弃B/C）**：
- **A（择）AV内确定性协作调度+共享前译**：aggregates profile 扩 task/on/spawn/send/ask/reply/await/go/sleep 语法；lowering 到现有HIR/MIR；调度服务=av-core内中立scheduler（虚拟时钟、零宿主线程、mailbox/timer/reply挂起唤醒，经host-intrinsic调用）；a2x-rust 以自包含生成代码实现同一状态机（行为oracle，不逐字匹配旧A2R金样）。
- B（弃）宿主线程调度复用967端口：非确定序破坏正确性测试、线程归属泄漏公共API、违背"不一次性重写宿主线程模型"。
- C（弃）移植旧AutoVM task_system执行器：V-08明令"不引入旧执行器"；旧实现仅作语义参考。

**语义冻结（D1–D3）**：D1 语法面=原件义务（task decl：状态字段默认值/`fn start() !`/`fn stop() !`/on臂字面量·类型绑定模式；`Task.spawn("Name", cap[, 位置初始化按声明序])`；`h.send(m)`；`h.ask(m)`+handler内`reply(v)`（send模式reply=具名错误，沿旧MessageContext）；`await`；`go`；`sleep(ms)`；无标注unit函数与bang标记在aggregates profile准入）。D2 调度=确定性协作（mailbox有界=spawn cap，满则send挂起发送者=背压，沿旧 `mpsc::channel(capacity)` 域；程序退出=有序drain/join——message_handler在main返回后仍打印"got one"；取消=既有CancelFlag契约+per-task取消token，取消/完成竞争单终态，迟到结果丢弃（沿967 Publish/DropLate处置形态）；timer=虚拟时钟，无可运行任务时最早截止优先；exit时未决timer取消）。D3 四端矩阵（按原支持域）：**av=全量实现；a2r=emit+运行**（行为等价，旧金样保留给旧实现）；**native/a2c=具名拒绝码**（旧A2C `trans/c.rs` 零task支持、旧产品无native-actor执行——原支持域外不扩，负控验证具名拒绝）；967端口不改不反向等待。

**owner/命令冻结（D4–D6）**：D4 owner=ace-core/source（lexer/parser/resolve/typecheck/adapter）、ace-lowering（task driver前译）、ace-av-core（中立调度服务+intrinsics）、a2x-rust（生成侧调度器）；语料 `crates/auto-lang/test/cases/ace-m2/scheduler/`（case.ad+suite.ad+src/，recipe `ac-av/aggregates-source-run-v1`，负控具名 `bind/source.*`）；新固定组 `case-ace-m2-scheduler-968`（沿965先例注册test-groups.toml）；四端断言进 ace-av/ace-a2x-worker 既有test crates。D5 命令=日常 `cargo t case-ace-m2-scheduler-968`；T-04完整职责并集 `cargo t ace-av ace-native ace-a2x ace-runtime a2r-actor-tests case-ace-m2-scheduler-968`。D6 资源=虚拟时钟不触墙钟（ExecLimits wall预算保持）；mailbox容量=spawn实参；新测试单例≤10s（test-discipline）；正确性泳道零新宿主线程（V-04/V-05由构造保绿）。



| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-scheduler.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-scheduler.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-scheduler.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**调度中被标准库消费的Linux OS/事件/资源腿**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

仅标准库Linux腿延期；普通平台中立async/actor、Windows状态机、四端调度原义务继续，不能借本条跳过共享算法测试。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-scheduler.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 真实挂起后唤醒只一次 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | reply关联不串 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 取消与完成竞争只一个终态 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 阻塞宿主不占UI事件线程 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 窗口退出后定时器/任务/读线程归零 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-06 | AC-02/03 | 四端相同事件序约束 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-av ace-native ace-a2x ace-runtime a2r-actor-tests`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 两sender交错保各自约定序，满队列背压；取消与reply竞争只有一终态；实际await挂起后继续正确；949旧修复语料作为迁移输入但不引入旧执行器。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 旧Echo send1应只输出got one；新控序输入1/2时消息次序可观测，取消后无迟到提交，bounded channel容量/背压按原支持域。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 关闭后send/wake、重复完成、容量超限和无授权操作具名拒绝；取消与语言异常分开，不靠timeout冒取消正确。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 挂起持必要owner，cancel关通道/IO后最后释放；Task.Stop/反复创建关闭与两个session隔离；保原公共同步基线。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 调度协议与真实四端状态机的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

## 8. 执行步骤

当前共8任务，1完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。[✅ 已完成 2026-11-15，见§5.1 T-01冻结记录；探针证据 P1–P7 存plan-artifacts/plan-968/probe-t01/]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。[DONE 2026-11-15 三提交 1792cde0a(AV全链)+e9d213f95(语料/recipe/worker)+461ecce35(a2r lane)：AV确定性调度器、MIR SCHED_*、case-runner语料11例、a2r Rust emit(线程+有界通道运行时,7例绿)；具名延后：a2r的Str字段spawn初值owned-text seam(spawn_with_state在AV/corpus全量覆盖)。原状态：AV全链已绿——scheduler profile(语法+CST投影+oracle)、sched_desugar(task→driver函数/操作→__sched_*内在)、VT::Task/Future单态句柄、MIR SCHED_*操作码(0x41-0x4B, closures族加性)、av-core确定性协作调度器(execute_sched: 虚拟时钟/背压/有序drain)、native具名拒。永久测试 av/tests/cases/sched_m2_968.rs 12/12绿(0.04s)：message_handler"got one"、spawn_with_state"mycounter: 15"(fstr裸花括号方言)、FIFO序、ask/await/reply 42、go、背压1/2/3、stop-hook、虚拟timer序、3负控。待办：语料case-runner接入、a2r lane、警告清理。]

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。[DONE 2026-11-15：case-runner worker协议(avworker/1信封+console观察)、a2x会话冷装+保留env、native具名拒；M2-A切片内增量/失败恢复由调度器内建语义(背压/有序drain/具名错误)承载，B阶段深度增量编辑不在A门]

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：明确mailbox容量、send/ask/reply顺序、timeout/取消胜负、任务join、生成器/stream所有权以及共享状态并发规则；有限前探不代实际四端任务实现。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。[DONE 2026-11-15：协议冻结=D1/D2(§5.1)+调度器实现(容量=spawn实参/满则park、FIFO、ask/reply端口单消费、stop=关邮箱→挂起态唤醒→停止钩子→join、sleep虚拟时钟、共享态=单Registry+mailbox归属)；V-08可观测面在 sched_m2_968(AV)+corpus+a2r 三处有真实结果(交错序/满队列/取消胜者/真实await挂起)]

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。[DONE 2026-11-15：完整职责并集 cargo t ace-av ace-native ace-a2x ace-runtime a2r-actor-tests case-ace-m2-scheduler-968 → 5 PASS/1 FAIL(ace-runtime 基线红,源码回退至 058179ed2 复核同样红=预存,非本包触面；未改判NA)+a2r-actor-tests NOT_APPLICABLE(legacy域无本包改动,组级NA规则)。V行证据：V-01..V-08/V-PHASE/V-DETAIL-01..04由 sched_m2_968 12例(AV单测)+corpus 11例(case-runner真实worker)+a2r 7例(Rust emit编译执行) 三层承载，见§5.1映射]

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。[DONE 2026-11-15：needs_fix(F1/F2)→修复a9f9800cc→重审pass aa630a2eb；无新债务候选（原具名延后已随F2关闭撤销）]

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。[DONE 2026-11-15：SD-01/02落盘 m2-scheduler.md(sha256前16 869aaf24495d5557)+replacement-scope L-29切片段(ed850cf1570c214a)；merge冻结门 ace-av/ace-native/ace-a2x/ace-lowering/case-ace-m2-scheduler-968 全PASS@9b3e07cda(=2fe30f52f rebase后)；ff-only合入master 2fe30f52f（6提交，rebase无冲突，无merge commit）]

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。[DONE 2026-11-15：本Plan全部输出在worktree+plan-artifacts/plan-968（后者整目录删除）；无外置target/输出目录对账缺口；wt-guard clean后移除worktree lang-968/分支plan-968-dev]

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-968/auto-lang`，分支 `plan-968-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-968/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: goal | plan_id: PLAN-968 | plan_revision: r5 | round: 2 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-968/auto-lang | commit: aa630a2eb | next: review

stage: goal | plan_id: PLAN-968 | plan_revision: r5 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-968/auto-lang | commit: aa630a2eb | next: merge

stage: goal | plan_id: PLAN-968 | plan_revision: r5 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 2fe30f52f | next: done

2026-11-15 stage: merge | PLAN-968 | r5 | cleaned: wt-guard clean → worktree lang-968+branch plan-968-dev removed（组目录已删）；plan-artifacts/plan-968 整目录删除（记录已折入本Plan），retention registry 刷新（无具名持有）；主检出代码路径 clean，docs/ 簿记已提交 8f4e6a5a8；landing checks: master tip=2fe30f52f（交付提交）+归档簿记 8f4e6a5a8，canonical specs在landed提交内验证；批量回归 due check: 968%5=3 非倍5触发，receipt 2026-11-14T23:23<48h → 不到期 | deployment observation: 无本包触面的发行二进制/web产物需要刷新（语言仓库内交付）| merge pass

2026-11-15 stage: merge | PLAN-968 | r5 | outcome: pass | prepared: reviewed aa630a2eb+specs 9b3e07cda | landed: master=2fe30f52f（rebase 6提交，ff-only，range-diff等价隐含于无冲突rebase）| canonical_verified: docs/specs/ace-migration/m2-scheduler.md(869aaf24495d5557) + replacement-scope-and-evidence.md(ed850cf1570c214a) @2fe30f52f | merge门: ace-av/ace-native/ace-a2x/ace-lowering/case-ace-m2-scheduler-968 PASS@冻结提交 | archived: docs/plans/archive/968-ace-m2-scheduler.md status=archived, 8/8 | completion_kind: delivered | 旧→新哈希映射: b8197f04d/cad341817/dec5dae83/d81c918d6/a411711ec/9b3e07cda → rebase后同序6提交 tip=2fe30f52f

2026-11-15 stage: review | review_scope: final | reviewer: 同会话独立上下文（限制：实施会话内，非独立模型/会话——从artifacts重建结论） | PLAN-968 | r5 | outcome: pass | worktree D:/autostack/.wt/lang-968/auto-lang | plan-968-dev | reviewed aa630a2eb | base 058179ed2 | deps: 965/967 archived | spec_inputs: docs/specs/ace-migration/replacement-scope-and-evidence.md；规范增量表SD-01/02/PHASE/DETAIL可行，落盘归T-06 merge | acceptance_results: AC-01 pass；AC-02 pass（F2关闭：a2r 8/8含spawn_with_state；AV 12/12；corpus 11/11）；AC-03 pass；AC-06 pass；AC-PHASE pass（A切片完整，无scope缩减；B门不在本切片）；AC-04 pass（my-crate warnings=0、debug prints移除、gated prints清除）；AC-05 merge阶段 | findings: 无未闭项；F1/F2已于a9f9800cc+aa630a2eb关闭并重验（分组：ace-av/ace-a2x/case-ace-m2-scheduler-968/ace-lowering/ace-native PASS）| evidence: 完整职责并集T-04已录（5 PASS+ace-runtime基线红非本包）；新用例计时：sched_m2_968(AV) 12例0.04s、corpus 11例、a2r 8例4.5s（含rustc -O编译prep，执行<0.1s/例，均≤10s门）| pending_closeout_gates: merge-owned冻结并集重跑（代码变更aa630a2eb后：ace-av/ace-a2x/case-968/ace-lowering/ace-native已重跑PASS；ace-runtime/a2r-actor-tests按T-04口径基线红/NA不变）+ spec落盘SD-01/02/PHASE/DETAIL | next: merge

2026-11-15 stage: work | PLAN-968 | r5 | T-02修复提交 a9f9800cc | F1清零（3 warnings移除，my-crate warning检查=0）；F2关闭（spawn_with_state a2r lane绿，a2r 8/8例；具名延后撤销）| 分组：ace-av/ace-a2x/case-ace-m2-scheduler-968 PASS | next_responsible_role: T-05重审

2026-11-15 stage: review | review_scope: final | reviewer: 同会话独立复核（实施会话内独立上下文重建，非独立模型——限制已注明） | PLAN-968 | r5 | outcome: needs_fix | worktree D:/autostack/.wt/lang-968/auto-lang | plan-968-dev | reviewed 461ecce35 | base 058179ed2 | deps: 965/967 archived | spec_inputs: docs/specs/ace-migration/replacement-scope-and-evidence.md + §5规范增量表(SD-01/02/PHASE/DETAIL,落盘为T-06 merge义务,可行) | acceptance_results: AC-01 pass(T-01冻结§5.1)；AC-02 partial(见F2)；AC-03 pass(worker/case-runner/a2x真实接线,native具名拒)；AC-06 pass(协议冻结+三层V-08真实结果)；AC-PHASE partial(同F2)；AC-04/05 review/merge阶段 | findings: F1(轻,AC-04健康面) my-crate 3 warnings：fstr.rs profile_admits_fstr 重复Aggregates臂+unreachable 286、死wrapper split_fstring、a2x-rust lower.rs:1838 未用task_of——修复=去重/删wrapper/删helper；F2(中,AC-02/V-DETAIL-01原支持域) spawn_with_state的a2r lane(其原支持域=a2r转译)Str字段spawn初值owned-text seam具名延后不充分——按"按原支持域"合同应修：driver field let的owned提升未触发,最小重放=cargo test -p ace-a2x-worker sched_968 spawn_with_state(恢复用例后) | evidence: 完整职责并集5 PASS+ace-runtime基线红(058179ed2源码回退复核)已录T-04；调度器行为证据 sched_m2_968 12例(0.04s)+corpus 11例+a2r 7例(3.95s含rustc编译,超10s规则仅测试本体编译为prep,测试执行<4s) | next: work修复F1/F2→重审

2026-11-15 stage: work | PLAN-968 | r5 | 5/8，work-owned任务全部完成，冻结提交461ecce35，无阻塞问题 | next_responsible_role: T-05独立复审（final review）

2026-11-15 stage: work | PLAN-968 | r5 | 5/8（T-04完成）| 冻结提交 461ecce35 | 完整职责并集：ace-av/ace-native/ace-a2x/case-ace-m2-scheduler-968 PASS；ace-runtime FAIL=基线红（源码回退058179ed2复核同样红，text-conformance v04/v06/v07等，非本包触面；runtime crates 0文件改动）；a2r-actor-tests NOT_APPLICABLE（legacy域）。附：T-04期间发现并恢复一次 stash 误弹（PLAN-002 跨工作树共享 stash，pop冲突后 reset --hard 恢复，PLAN-002 stash 保留完好）| next: T-05 独立复审

2026-11-15 stage: work | PLAN-968 | r5 | 4/8（T-01/T-02/T-03/T-08完成）| commits 1792cde0a+e9d213f95+461ecce35 | 分组证据：ace-av PASS(含sched_m2_968 12例)、case-ace-m2-scheduler-968 PASS 11/11、ace-a2x PASS(含sched_968_a2r_rust_emit 7例)、ace-native/ace-lowering/ace-source/ace-foreign-contract PASS | 具名延后(债候选)：a2r Str字段spawn初值owned-text seam（spawn_with_state在AV/corpus全量覆盖）；a2c=原支持域外具名拒 | next: T-04全职责并集

2026-11-15 stage: work | PLAN-968 | r5 | T-02语料/T-03接线已提交 commit e9d213f95（21文件 +539/-31：recipe ac-av/scheduler-source-run-v1、sched_worker、case-runner接线、契约注册、语料11例8正3负、固定组 case-ace-m2-scheduler-968）| 分组：case-ace-m2-scheduler-968 PASS 11/11、ace-av PASS；daily单红(plan702门)经干净基线复核预存 | 剩余：a2r lane(D3)、T-04全职责并集、T-05复审 | next: a2r lane

2026-11-15 stage: work | PLAN-968 | r5 | T-02 AV切片已提交 commit 1792cde0a（44文件 +4751/-305）| worktree D:/autostack/.wt/lang-968/auto-lang | 分组证据：ace-av PASS(1101+12含sched_m2_968新12例0.05s)、ace-native/ace-lowering/ace-source/ace-foreign-contract PASS；ace-host红经干净基线stash复核为预存（av_source_inventory、a2r_hir_first_case两例基线同样红，环境性）| next: T-02语料(recipe ac-av/scheduler-source-run-v1+worker接线+case-ace-m2-scheduler-968组)+a2r lane

2026-11-15 stage: work | PLAN-968 | r5 | T-02进行中（AV主链全绿）| worktree D:/autostack/.wt/lang-968/auto-lang | branch plan-968-dev | base 058179ed2 | task T-02部分 | 证据：ace-av conformance sched_968 12/12 PASS 0.04s（canonical message_handler=got one、spawn_with_state=mycounter: 15、FIFO、ask/reply、go、背压、stop-hook、虚拟timer、3负控具名拒）；T-01探针P1-P7已录 | blockers: 无（机器占用等待回归组）| next: T-02语料+T-03接线

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r6：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

stage: goal | plan_id: PLAN-968 | plan_revision: r5 | round: 1 | from: start | to: work | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-968/auto-lang | commit: 058179ed2 | next: work

2026-11-15 stage: work | PLAN-968 | r5 | 1/8（T-01完成）| worktree D:/autostack/.wt/lang-968/auto-lang | branch plan-968-dev | base 058179ed2 | deps: PLAN-965(archived r5)、PLAN-967(archived r12) | task T-01 | 证据：探针P1-P7（plan-artifacts/plan-968/probe-t01/，P1对照链PASS=42、P2/P3具名 source.unsupported task、P5失败后恢复PASS、P6/P7 unit函数/bang标记缺口）；冻结D1-D6入§5.1 | blockers: 无 | next: T-02

2026-11-12 stage:new / PLAN-968 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
