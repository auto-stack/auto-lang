---
plan_id: PLAN-965
status: execution_done
feature_name: 完整控制流错误传播与生成器
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-14
plan_revision: 5
current_step: 5
total_steps: 8
depends_on: [PLAN-962, PLAN-964]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-control-effects.md]
touched_goals: [GOAL-002]
---

# [PLAN-965] 完整控制流错误传播与生成器

## 0. 变更摘要

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

补真实try/catch、.?、??、is表达式、嵌套break/continue/return、yield及捕获环境退出；共享效果/清理图区分语言错误、trap、资源失败与宿主取消；挂起点保留清理责任。

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
- `crates/auto-ast/src/ast.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/core/src/source`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/lowering/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：控制流/错误传播/生成器清理。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：循环目标、模式、传播返回类型；读取SHA256前16 `24441ead81f35f8f`

- [experimental/ace/crates/lowering/src](../../experimental/ace/crates/lowering/src)：共同控制/清理降级；目录职责已核，确切符号/闭包开工再核

- [crates/auto-ast/src/ast.rs](../../crates/auto-ast/src/ast.rs)：旧语法义务输入，非新侧parser；读取SHA256前16 `e8820f13f329630f`

### 2026-11-14 goal 授权记录

用户指令"完成计划965"（/auto-plan-goal 目标）授权整链：work（含建专属树）、review、合同内有界修复、merge（规范沉淀、归档、清树、落地核验）。授权范围不改变目标/验收/预算/仓库集；不授权绕过worktree规约、证据政策或阶段停止条件。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


补真实try/catch、.?、??、is表达式、嵌套break/continue/return、yield及捕获环境退出；共享效果/清理图区分语言错误、trap、资源失败与宿主取消；挂起点保留清理责任。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### T-01 有界前探与冻结决定（2026-11-14）

- **前探验证与基面确认**：在 `D:/autostack/.wt/lang-965/auto-lang` 验证 `cargo test -p ace-core --no-run`（exit 0，编译通过）；核查 `experimental/ace/grammars/auto-source.at`（`r-try`/`r-catch`/`r-yield`/`question`/`is` 已保留）、`experimental/ace/crates/core/src/source/{parser.rs,resolve.rs,adapter.rs}`（所有 `SourceProfile` 已走统一 CST，当前 `Aggregates` 为 M2-A 超集前端）、`experimental/ace/crates/lowering/src/hir_adapter.rs`、`experimental/ace/crates/av-core/src/{av.rs,contract.rs}`（`release_frame_owned_lifo`/`cleanup_owned_frames`/`host.unauthorized` dispatch 前置拦截）、`experimental/ace/crates/native/src/` 及 `crates/a2x-{core,rust,c}/src/`（A2X 已具备 `Family::Collections`、`Record`/`Variant`/`List`/`MutBorrow` 降级能力）。
- **冻结决定 D1（前端 Profile 与文法门控）**：沿 `SourceProfile::Aggregates`（`"auto.source.aggregates.draft"` / `av.aggregates/1` -> `HirDomain::Closures` -> `AV_CLOSURES_PROFILE`）扩展，保持 `Core..Numeric` 全部冻结 profile 逐字节不变。`auto-source.at` 仅在 `aggregates` profile 门控新增：`tilde`（`~Iter<T>`）、`dotquestion`（`.?`/`.?(default)`）、`questionquestion`（`??`）、`r-finally`（`finally`）、`!T` 返回/类型简写（默认 `Result<T, Str>`）、`is` 表达式形态与单表达式分支 `-> expr` / 字面量+`else` 分支、`try { ... } [catch [(e)|e] { ... }] [finally { ... }]`、`yield [expr]`，以及内建可控抛出原语（如 `rethrow(e)` / `fail(msg)`，用户同名函数优先）。
- **冻结决定 D2（共享效果与清理图 `resolve.rs` -> HIR -> MIR）**：单一真相源设于 `resolve.rs` + `hir_adapter.rs`，不另建平行类型检查器或平行 VM。
  - 每个 `try/catch/finally` 降级为显式控制/退出分派状态机：正常退出、`return v`、跨层 `break`/`continue`、`.?` 错误传播、`rethrow/fail` 沿退出路径**仅执行一次**沿途内层到外层的 `finally` 块与词法作用域 `Drop`。
  - `.?` 分派：`Err(e)` 若在活动 `try/catch` 内则绑定载荷至 `catch` 并按 `finally` 顺序收尾；否则若在兼容 `Result<_, E>` 函数内运行清理并返回 `Err(e)`；若在非 `Result`/`?` 函数内无 `catch` 则编译期报 `source.propagate-incompatible`；若在顶层脚本未捕获传播，触发语言错误终止（保留载荷与阶段，后续语句不执行，绝不返回旧 VM `4000000` 堆地址）。`None.?` 仅在返回 `?U` 的函数内提前返回 `None`（不进 `catch`），否则编译期拒绝（或脚本顶层报未捕获语言错误）。
  - `??` 与 `is` 表达式：`lhs ?? rhs` 与 `is` 表达式按控制流分支降级——`??` 对 `Some(v)`/`Ok(v)` 直接解包，**仅在 `None`/`Err` 分支惰性求值 `rhs`**（V-02）。
- **冻结决定 D3（四分类错误/资源/宿主边界，V-04 / V-08 / T-08）**：
  - `ExecOutcome` 显式增加 `ExecOutcome::Error { message, span }`（配合 `MirInstr::LangFail`），将**语言错误**与 `Trap`（`DivByZero`/`InvalidHandle`/…）、`Resource`（fuel/wall/frames/slot_bytes/list-elems）、`Cancelled`（`CancelFlag`）、`Infra`（`host.unauthorized`）彻底分账。
  - 编译期错误（`source.*`/`verify.*`）在运行前拒绝，永不进入 `catch`；底层 `Trap`、`Resource`、`Cancelled` 及 `host.unauthorized` **均不可被语言 `catch` 吞掉**；未授权宿主/平台调用在检查 `provider` 时直接阻断，保持 `dispatch = 0`。
- **冻结决定 D4（生成器 `~Iter<T>` 与 `yield` 状态机降级，V-03 / V-08）**：
  - 在 `resolve.rs` 将每个 `fn gen(...) ~Iter<T>` 降级为惰性状态机记录 `__Gen_<id>`（`__state: i32`, 参数, 跨 `yield` 存活的局部包装为 `?LocalTy`）及模块内统一枚举壳 `Iter<T>`（支持跨函数间接返回如 `get_counter()` 与多生成器同型并存）。
  - 惰性启动：调用 `gen()` 仅构建 `__state=0` 帧；每次 `for n in g` 调用 `__iter_next(mut g)` 推进到下一 `yield v` 返回 `Some(v)`（**挂起期间 owned 局部留在状态记录内，不提前 drop**）；耗尽或 `return` 设 `__state=-1` 并返回 `None`；循环正常结束/提前 `break`/`return`/`Err` 退出作用域或未尽生成器被 drop 时触发幂等关闭 `__iter_close(mut g)`（仅活跃时运行一次 `finally` 并清理，后续 drop 见 `__closed` 不重复释放）。无限生成器按步/时间消耗正常 `ExecLimits`，超限报 `Resource` 并由 `cleanup_owned_frames` 无泄漏回收。
- **冻结决定 D5（四端适用表与永久验证）**：
  - **AV (`ace-av` + `av-worker`)**：全量原语 via `av.closures/1` + `ExecOutcome::Error` + `StrArena` `live=0` 核验。
  - **Native (`ace-native`)**：复用 `collections_lower` / `collections_driver`（`Variant`/`Record`/`MutBorrow`/作用域清理）+ `LangFail` 非零退出承载 `try/catch/finally`、`.?`、`??`、`is` 表达式与状态机生成器真实 PE 编译执行。
  - **A2X (`ace-a2x-rust` + `ace-a2x-c`)**：放开 `auto.source.aggregates.draft` 走共享 `collections_facts` + `lower_coll` 通道，补齐 `Variant` `MutBorrow` 参数及 `LangFail` 运行时中断。
  - **Language TestCase**：新增 `crates/auto-lang/test/cases/ace-m2/control_effects/` 并在 `scripts/test-groups.toml` 注册 `case-ace-m2-control-effects-965` 接入 `ace` 域。

### 2026-11-14 实施期边界（T-02/T-03/T-08；合同内细化，不改目标/AC/范围/仓库集）

- **生成器借用参数（D4 细化）**：状态记录按值保存参数（D4 原文），因此借用无法跨 `yield` 保留：`mut` 记录参数会丢失写回，记录视图/`str` 会越过 owner 生命周期。实施前述 `mut` 参数在 AV 端确实静默丢失写回（native 因共享句柄看似可见），因此改为**具名拒绝** `source.generator-borrow-param`（Bind 阶段）；owned 句柄（`List`/`Object`/`Variant`/owned `Str`）与标量参数继续支持，关闭效果经 `List` 句柄观测。旧语法分母无生成器借用参数用例（`test/vm/22_generator/*`、`test/a2r/21_generators/*` 均为无参/标量参数），故不构成分母缩减；证据与反例见 §9。
- **opcode 归属**：`Terminator::LangFail` 使用 `0x40`（指令空间 `0x01–0x3F` 已满，`m2-dynamic` 冻结 `0x3D/0x3E/0x3F`）；不得复用 `0x3D`（OBJECT_NEW）。
- **case lane 阶段词表**：AV probe 判定的 run 阶段语言错误（`ExecOutcome::Error`）此前被 `map_diag_stage` 落入 Verify；补 `"run" => Stage::Run`，使 run 阶段 diag 与 verify 拒绝可分账（对既有语料无影响：既有 run 期望均为 trap 结局）。
- **清理错误策略未决**：drain 时 close `finally` 抛 `fail` 会传播（AV `Error`/native `Trap(9)`），但 for-in 提前 `break` 触发的 close 抛错会被丢弃（两端一致但可疑）。登记 §10，交 T-05 裁定，不并入本条完成声明。

### 本次复核补充：控制流与资源跨出口组合（r2）

补catch/finalization、错误传播跨模块返回、循环continue/break与生成器drop；宿主权限失败可否被语言捕获必须继承原政策，唯一不可放宽的是未授权dispatch=0。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/22_generator/001_sum/sum.at](../../crates/auto-lang/test/vm/22_generator/001_sum/sum.at)（下列节选L1–L12；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    print(sum)
}
```

本包所需独立结果：counter yield1/2/3真实求和6；Some(42)??0为42；Err传播保payload/阶段且后续效果不发生，不把旧4000000机器表示当语言数值。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** try/传播/循环/yield源→共享效果与清理CFG→挂起帧/错误值→四端执行/释放trace。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
enum FlowOutcome { Value(ValueRef), Return(ValueRef), Break(LabelId), Continue(LabelId), Yield(FrameId), Throw(ErrorRef) }
struct CleanupEdge { owner: OwnerId, exit: ExitKind, target: BlockId }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | 循环目标、模式、传播返回类型 |
| [experimental/ace/crates/lowering/src](../../experimental/ace/crates/lowering/src) | 共同控制/清理降级 |
| [crates/auto-ast/src/ast.rs](../../crates/auto-ast/src/ast.rs) | 旧语法义务输入，非新侧parser |

**状态、错误与资源。** 不合法break目标、传播到不兼容返回、错误分支多执行/漏执行、无限生成资源超限必须区分语言错误/预算/取消。 return/break/continue/throw/yield/drop各清理一次；宿主权限失败不因catch放行，未授权dispatch=0；取消/late不得复活帧。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/22_generator/001_sum/sum.at](../../crates/auto-lang/test/vm/22_generator/001_sum/sum.at) | yield序列/求和6，补早drop不重复 | `sum.expected.out`；SHA256前16 `2815e1baa84fb291` | 拟ace.m2.p965.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/16_option_result/011_coalesce_some/coalesce_some.at](../../crates/auto-lang/test/vm/16_option_result/011_coalesce_some/coalesce_some.at) | coalesce公开结果42 | `coalesce_some.expected.result`；SHA256前16 `e939335e8c71efad` | 拟ace.m2.p965.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/16_option_result/010_propagate_err/propagate_err.at](../../crates/auto-lang/test/vm/16_option_result/010_propagate_err/propagate_err.at) | 旧expected.result=4000000，需解读原协议并补语义oracle | `propagate_err.expected.result`；SHA256前16 `d4ff4b6bc192bf38` | 拟ace.m2.p965.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/09_option_result/011_question_propagate/question_propagate.at](../../crates/auto-lang/test/a2r/09_option_result/011_question_propagate/question_propagate.at) | 传播生成形转实际运行/负控 | `question_propagate.expected.rs`；SHA256前16 `652f63109d322252` | 拟ace.m2.p965.source04：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** try/catch/finalization与.?/??/is等真实拼写、各退出清理与generator关闭合同，按旧源及最新规范逐形冻结；不猜异常ABI。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-control-effects.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-control-effects.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-control-effects.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 嵌套try提前return清理一次 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | ??右支无需求不执行 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | yield暂停不提前析构，close释放 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 编译错误不进入运行时catch；宿主拒绝按冻结错误合同可捕获或不可捕获，未经授权dispatch始终为0 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 四端trace严格保源求值顺序 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 同一owned值经return、throw、yield-close、cancel各一次释放；合法catch处理可捕获错误，编译错误不进入运行时catch；非法跳转具名诊断。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | counter yield1/2/3真实求和6；Some(42)??0为42；Err传播保payload/阶段且后续效果不发生，不把旧4000000机器表示当语言数值。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 不合法break目标、传播到不兼容返回、错误分支多执行/漏执行、无限生成资源超限必须区分语言错误/预算/取消。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | return/break/continue/throw/yield/drop各清理一次；宿主权限失败不因catch放行，未授权dispatch=0；取消/late不得复活帧。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 控制流与资源跨出口组合的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。
  - 证据（`plan-965-dev`，已 resync 至 master `cd5f2e568`）：`source/agg_lower.rs`（新，生成器状态记录/`__ace_iter_next`/`__ace_iter_close` 与 close 清理）、`source/resolve.rs`（try/catch/finally、`rethrow(e)`、`.?`/`.?(d)`、`??`、`is` 表达式/语句、`LangFail` 分派与四分类退出、生成器借用参数具名拒绝）、`av-core`（`Terminator::LangFail`=opcode `0x40`、`ExecOutcome::Error`、MIR/mir_verify/ownership/ABC/compact_abc/instr_spec）、`lowering/hir_adapter`、`native/{collections_lower,collections_driver,driver,fingerprint,lower,closures_lower}`、`a2x-{rust,c,python,core,worker}`。正反例结果见 T-04 收据（AV 11 / native 5 / A2X 2 / 语言 TestCase 10，全绿）。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。
  - 证据：四端消费同一共享 MIR（AV interpreter、native PE、A2X-Rust、A2X-C），无 legacy fallback；host/权限保持既有 dispatch 合同（`host.unauthorized` dispatch=0，AV 例 4 断言）。新增语言侧 TestCase 组 `case-ace-m2-control-effects-965`（`scripts/test-groups.toml`）与 `crates/auto-lang/test/cases/ace-m2/control_effects/`。新增 `source.generator-borrow-param`：生成器状态记录按值保存参数，借用（`mut`/记录视图/`str`）无法跨 `yield`，改为具名拒绝而非静默丢失写回（详见 §9 与 §10）。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：补catch/finalization、错误传播跨模块返回、循环continue/break与生成器drop；宿主权限失败可否被语言捕获必须继承原政策，唯一不可放宽的是未授权dispatch=0。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。
  - 证据：V-01（嵌套 try/finally 每帧一次 + 循环 break/continue）、V-02（`??`/`.?(d)` 右支惰性；计数器 hits=0）、V-03（惰性启动、显式 close 幂等一次、for-in 提前 break 关闭；用 owned `List` 句柄观察，见 AV 例与语言 case `generator-sum-close`）、V-04（编译错误不进运行时 catch；Trap/Resource/Cancelled/host.unauthorized 不可捕获、dispatch=0）、V-08（owned 值经 return/throw/drain-close 各清理一次；drain 时 close `finally` 抛 `fail` → AV `ExecOutcome::Error("closed965")`、native `Trap(9)`、语言 case `generator-close-fail`）。宿主策略与四分类分账按 D3 保持。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。
  - 证据（`plan-965-dev@edae44f0b`，resync 至 `master@e9e67496a`，冻结树单实例）：
    - 语言 TestCase：`cargo t case-ace-m2-control-effects-965` → **PASS**，10/10 checks，`gate_passed=true`，wall 5s（av-worker release 复用；`target/test-groups/run-8ezgeg3w`）。
    - 固定组并集：`cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime`（wall 1058.8s，`run-intz138c`）：ace-lowering **PASS**(5/5)、ace-a2x **PASS**(238/238)；ace-source 523/524、ace-av 1126/1128、ace-native 273/276、ace-runtime 65/84。**红集精确等于** `master@e9e67496a`（及 `cd5f2e568`）既有红：6 例 `text.char-multi`（基线命令见 §9）+ `ace-runtime` 19 例 text shim 链接（`_fltused`/`__CxxFrameHandler3`，主检出同组复现）。本包 diff 引入的红为 0。
    - 新增永久用例：AV `aggregates_m2_965` 12 例、native `control_effects_965` 6 例、A2X `aggregates_965_control_effects_{rust,c}` 2 例；隔离与组内均绿。V 行→用例映射见 §9 收据。
    - 无法 PASS 的具名项（未改 NA）：§10-1（early-break close 清理抛错策略）、§10-2（默认分支既有 6+19 红）、§10-3（生成器借用参数边界待复核）。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-965/auto-lang`，分支 `plan-965-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-965/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

### 2026-11-14 work 收据（T-02/T-03/T-08/T-04；goal round 2）

`stage: work | plan_id: PLAN-965 | plan_revision: r5 | outcome: pass | worktree: D:/autostack/.wt/lang-965/auto-lang | branch: plan-965-dev | base_commit: e9e67496a（两次 resync：9073b5fd2 → cd5f2e568 → e9e67496a） | code_commit: edae44f0b | dependency_revisions: auto-down@58ea33f（detached，未变） | task_ids: T-02,T-03,T-08,T-04 | blockers: none（外部既有红具名于 §10-2） | next_responsible_role: review(T-05)`

**实施与修复（本包预算内）**
- 承接上一轮未提交实施面（63 改 + 4 新，约 6k 行；交接时无 commit）：先落本地 commit 再两次 `git rebase master`（均干净文本合并）。第一次 rebase 需补 `ast::Item::UseRs`（967 `use.rs` 前门）在 `agg_lower::collect_agg_iter_records` 的空臂；第二次 29 提交漂移（967 T-10/T-11、1021 ase-m0）后 rebase 干净，编译与并集重跑通过。
- `LANG_FAIL` opcode 从 `0x3D` 改为 **`0x40`**：`0x3D/0x3E/0x3F` 已由 `m2-dynamic` 冻结（OBJECT_NEW/GET/TEXT_DISPLAY）；ABC 与 compact 双编码同步；`instr_spec` V-11 计数钉 56→57（`av_data_instruction_contract_round_trip_all_opcodes` 相应更新）。
- `patch_foreign_sentinels` 的 `unsafe` 裸指针栈遍历改为**安全递归**（覆盖与原实现逐分支一致；本包 AV 生成器/关闭用例全绿）。
- 生成器间接返回（D4 明示能力，legacy `22_generator/004_indirect_return`）：`~Iter<T>` 且无 `yield`、`return g()`/尾表达式形式的函数按普通函数降级（此前被当状态机，触发 `verify.block-structure` 不可达块）。AV+native 各一永久例。
- 生成器借用参数（D4 原文“参数按值存入状态记录”的细化）：`mut`/记录视图/`str` 借用无法跨 `yield`，改为具名拒绝 `source.generator-borrow-param`（AV 端曾静默丢失写回）；owned 句柄与标量继续支持（§10-3 交复审确认）。
- case lane 词表：`crates/auto-lang-core/src/test_cases/ac_av.rs::map_diag_stage` 补 `"run" => Stage::Run`（run 阶段语言错误此前被归为 Verify）；既有语料无 run 期望 diag，无行为回退。
- WSL/musl 载体 `mn-linux-av-runner` 补 `ExecOutcome::Error` 分支（此前 `cargo t ace-native` 的 linux_elf 腿因非穷尽匹配编译失败；修复后该例 91s 通过）。
- 语料修正：`rethrow`→`rethrow(e)`/`rethrow()` 真实拼写、`is Some(1)` 非穷尽反例、生成器 close 观测改 owned `List` 句柄 + 新增 `generator-close-fail`（drain 期 finally 抛错传播）；语言 case 10 例。

**验证矩阵（提交 `edae44f0b` 冻结树）**

| 入口 | 命令 | 结果 |
|---|---|---|
| 语言 TestCase | `cargo t case-ace-m2-control-effects-965` | PASS 10/10 checks，gate=true，wall 5s（run-8ezgeg3w） |
| 固定组并集 | `cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime` | wall 1058.8s（run-intz138c）：lowering 5/5 PASS、a2x 238/238 PASS；source 523/524、av 1126/1128、native 273/276、runtime 65/84 — 红集=master 既有 |
| 基线复核（新 master） | 主检出 `e9e67496a`：`cargo nextest run -p ace-core -p ace-av -p ace-av-core -p ace-native --config-file ../../.config/nextest-full.toml -E '<6 例过滤>'` | 7 run: 1 passed, 6 failed（`text.char-multi`，与本包无关） |
| 基线复核（runtime） | 主检出 `master@37b197a5c`：`cargo t ace-runtime` | 65/84（19 例同一 text shim 链接失败），与本包无关 |
| 新增永久用例 | AV 12 / native 6 / A2X 2 / 语言 case 10 | 全绿 |

**V 行 → 永久用例映射**
- V-01 → AV `m2_v01_nested_try_finally_runs_once_per_frame_and_loop_ctrl`、语言 `try-catch-finally`、native `mn_965_try_catch_finally_nested_break_continue_return`。
- V-02 → AV `m2_v02_lazy_short_circuit_coalesce_and_unwrap_or`、`m2_oracle_coalesce_some_and_none`、native `mn_965_coalesce_propagate_and_is`、语言 `coalesce-propagate-is`。
- V-03 → AV `m2_v03_generator_lazy_suspend_finally_and_idempotent_close`、`m2_oracle_indirect_generator_return_and_coexisting_instances`、native 对应 2 例、语言 `generator-sum-close`、`generator-mut-borrow`。
- V-04/V-08 → AV `m2_v04_ac06_v08_limits_trap_cancel_host_not_caught`、`m2_v08_close_fail_during_drain_propagates_as_language_error`、native `mn_965_uncaught_fail_maps_to_non_zero_trap`、`mn_965_generator_close_fail_on_drain`、语言 `uncaught-fail`、`generator-close-fail`。
- V-05（求值顺序）→ 以 V-02 惰性断言（AV `hits==0`、native `ticks==0`）与 V-03 惰性启动覆盖，见 §10-5 交复审确认。
- 反例：AV `m2_negative_compile_controls`（propagate/yield/rethrow/非穷尽 `is`/生成器借用参数）、语言 `propagate-incompatible`/`yield-outside-generator`/`rethrow-outside-catch`/`nonexhaustive-is`。

**未决（交复审）**：见 §10-1..§10-5；其中 §10-1 为语义策略问题（两端一致，非实现漏项），§10-2 为默认分支既有红，§10-3 为新增边界的确认。

**基面漂移记录**：本轮工作期间默认分支前移两次（`9073b5fd2` → `cd5f2e568` → `e9e67496a` → 收据时 `master=37b197a5c`）。`code_commit: edae44f0b` 基于 `e9e67496a`；merge（T-06）必须在当时 `master` 上 resync 并重跑受影响并集（不得沿用本收据的并集证据作为 merge 后代码的证明）。

`stage: goal | plan_id: PLAN-965 | plan_revision: r5 | round: 2 | from: work | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-965/auto-lang@plan-965-dev | commit: edae44f0b | next: review`

### 2026-11-14 goal 轮次收据

`stage: goal | plan_id: PLAN-965 | plan_revision: r5 | round: 1 | from: goal | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-965/auto-lang@plan-965-dev | commit: 9073b5fd2 | next: work`

首轮：goal 授权已记录于§4；主检出仅 `docs/plans/` 与 `docs/design/` 簿记 WIP（无代码路径 WIP）；前置 `PLAN-962`/`PLAN-964` 均已归档在 `master@9073b5fd2`；专属树 `D:/autostack/.wt/lang-965/auto-lang`（`plan-965-dev@9073b5fd2`）与 `auto-down` 兄弟（`detached@58ea33f`）及唯一材料目录 `D:/autostack/plan-artifacts/plan-965/` 已建，进入 work 阶段（T-01 有界前探）。

2026-11-12 stage:new / PLAN-965 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

### 2026-11-14 work 阶段登记的具名问题（交 T-05 复审裁定）

1. **清理期抛错策略（early-break close）**：for-in 提前 `break` 触发的生成器 close 中，`finally` 的 `fail`/`rethrow` 被丢弃（AV 与 native 一致地返回正常值），而 drain 触发的 close 会传播（AV `ExecOutcome::Error`、native `Trap(9)`，见 AV `m2_v08_close_fail_during_drain_propagates_as_language_error` 与 native `mn_965_generator_close_fail_on_drain`）。最小复现：把 `crates/auto-lang/test/cases/ace-m2/control_effects/src/generator_close_fail.at` 的 drain 循环改为 `if x == 20 { break }`，两端均返回 `part=30`。关闭条件：T-05 裁定“传播/显式文档化”之一，并补对应永久用例；不并入本条工作完成声明。
2. **默认分支既有红（非本包改动）**：`master@cd5f2e568` 上 6 个既有测试红，根因同一族（CST lex-facts 对非 ASCII 字符字面量误报 `text.char-multi`，如 `eprint('界')`）：`ace-core host_console::av_console_typed_symbols_and_capabilities`、`ace-av host_console::av_console_exact_source_bytes_and_result`、`ace-av av_closures_source::av_closures_909_char_cell_write_back_observed`、`ace-native console_lane::native_console_i32_bool_char_say_lanes`、`ace-native shadow_tier::shadow871_text`、`ace-native data_owned_tier::mn_data_owned_three_path_differential`。已在主检出 `master@e9e67496a`（6 例）与 `master@37b197a5c`（ace-runtime 19 例）用同一入口复现（见 §9 基线收据），非本包 diff 引入；需独立 fix/Plan 处理，关闭条件：该 6 例在 `master` 转绿。
3. **生成器借用参数边界**：`source.generator-borrow-param` 为实施期新增具名拒绝（§5 边界）；复核需确认它没有与 959 分母/Design55 的既有生成器语义冲突，并决定是否在未来以引用字段承载借用（当前状态记录只能按值保存）。
4. **opcode 归属**：`LANG_FAIL=0x40` 不占用 `m2-dynamic` 的 `0x3D..0x3F`；复核需确认 V-11 计数钉（56→57）与 `m2-dynamic`/`m2-aggregates` 规格记录一致。
5. **V-05 映射**：本轮以 V-02 的惰性/顺序断言（AV `hits==0`、native `ticks==0`）与 V-03 的惰性启动覆盖“求值顺序”；复核需确认是否足以关闭 V-05，或要求补独立四端 trace 顺序用例。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
