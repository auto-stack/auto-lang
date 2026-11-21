---
plan_id: PLAN-1090
status: archived
completion_kind: delivered
feature_name: AV任务、异步块与Actor语言补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1081, PLAN-1083, PLAN-1085, PLAN-1086, PLAN-1087, PLAN-1089]
supersedes_spec_components: [docs/specs/ace-migration/m2-scheduler.md, docs/specs/ace-migration/m2-control-effects.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1090] AV任务、异步块与Actor语言补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`async/tasks-actors.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-12**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

在968已有scheduler基础上补旧合法异步/TaskRef存储与跨函数调用，并使组合不受main-only限制。

本包交付`IF-AV-PARITY-SCHEDULER`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- ~{}、.await/.go与前缀await/go的实际旧合法拼写，~T函数、future传播/容器/返回、Task.spawn位于普通函数/handler及TaskRef参数/record字段。
- task/on/start/stop、typed/variant/string/guard/else消息、初始化参数、ask/reply及错误、FIFO/有界背压、关闭/late/取消语义。
- 系统/FFI/异常/生成器与任务组合按已交接口接入；不把host权限转为catch，不承诺所有线程调度轨迹相同或新增无限并发。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1090/auto-lang`，分支`plan-1090-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1090/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-21 用户以 `/goal 完成计划1090` 授权 r3 现有合同的执行、独立复审、合同内修复、合入、归档与清理。既有 AV-ONLY 边界继续适用。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

968/1067已有真实调度，不是空白；当前Task.spawn限main、任务/future按task单态句柄，以及Scheduler独立profile，是相对旧TaskRef/async程序的明显组合差额。AV虚拟timer与旧Tokio实现不要求同内部调度。

既有来源与边界：

- [handle_struct_field.case.ad](../../../crates/auto-lang/test/vm/23_actor/021_handle_struct_field/handle_struct_field.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [scheduler.document.case.ad](../../../crates/auto-lang/test/cases/ace-m2/scheduler/scheduler.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：23_actor；cookbook/concurrency及asynchronous的纯语言角色；旧Task/Msg/channel/future Rust tests。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [ast.rs](../../../experimental/ace/crates/core/src/source/ast.rs) | `e1977531e25c897cec99db418887de7ec51845b04c74b568543b0b464a6c8eb3` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [adapter.rs](../../../experimental/ace/crates/core/src/source/adapter.rs) | `915cc3e6c62d37f5bd55477d198a68279531760ed72573bcf7d9a1728f6ee5a3` |
| [av.rs](../../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1081, PLAN-1083, PLAN-1085, PLAN-1086, PLAN-1087, PLAN-1089；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [ast.rs](../../../experimental/ace/crates/core/src/source/ast.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [adapter.rs](../../../experimental/ace/crates/core/src/source/adapter.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [task_system.rs](../../../crates/auto-lang-core/src/vm/task_system.rs)：本包旧行为/独立oracle来源，只读，不修改旧VM语义。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/async/tasks-actors.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-scheduler.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-control-effects.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-FC | modify（本特性消费/来源） | `docs/specs/auto-test-case/feature-organized-corpus.md` | 按特性唯一作者、同源多端检查、旧义务映射和本包迁移/观察来源；实际交付后沉淀，当前不宣称已支持 | 公共机制由1078生产一次，本包保原语言/目标义务 | AC-FC、V-FC-SOURCE/MIGRATE/TARGET |

`language-parity.md`由AVP-00首次创建，后续串行增量维护；它是当前规范的能力/来源映射，不是第二注册器。实施前本包文档只提议，真实merge后按独立复审结果沉淀，不提前将Spec写成已支持。


## 6. 测试设计

### V-AV-ONLY / V-IGNORE-LIFECYCLE（未执行）

原T-03..06落实、T-07复核：同一Case AV已选，AC/A2R检查加#ignore；制造该目标缺工具/运行失败/错误oracle时，目标prepare/dispatch计数为0，AV仍按独立预期通过，忽略行不充PASS。将AV oracle改错必须使AV门红，全部有效AV检查被ignore或零选中也必须红。移除某非AV阶段ignore并进入其后端选择后，该目标失败必须在该后端门显现，而AV视图仍不选择它；移除ignore不能同时删原平台/ignore理由。公共格式/重复ID/AV必需shared规则出错不被这一阶段策略豁免。

### FC 测试组织与迁移必验增量（未执行）

| V-ID | owner_stage / 原任务 | 必须观察的结果 |
|---|---|---|
| V-FC-SOURCE | work / T-02..06 | 本轮同一source/project仅选AV真实执行/按规定阶段拒绝；改公共body使相关旧证据陈旧，非AV行保持#ignore/未选与后续owner，不要求本轮运行。 |
| V-FC-MIGRATE | work / T-01/02/05/06 | 原case/check/条件/判决角色→canonical映射完整；非等价邻例、错误/资源/恢复角色不丢；suite、worker/内联/工具读者均切唯一活跃作者，旧副本仅具名历史。 |
| V-FC-TARGET | work / T-04..06 | AV oracle变异、重复活Case ID/AV旧活入口残留、零条有效AV/缺AV required检查必红；AC/A2X oracle不符或缺工具在#ignore/未选时不进入AV门。非AV完整目标oracle/emit-vs-run控制留给其启用阶段。 |

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1090-原文档清单)静态主承接 **13份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1090.ask-reply` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1090.spawn-outside-main` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1090.async-block-value` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1090.fifo-capacity` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout='19\n23\n'; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `TASK-HANDLE` | AC-03/AC-04 | work / T-04..05 | 原021_handle_struct_field原源原stdout，两实例/跨参数/跨函数返回；不能把TaskRef字段改成外部全局registry绕过语言类型。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `SCHED-FAILURES` | AC-03/AC-04 | work / T-04..05 | closed task/未回复/重复消费future/cap=0、跨await owned locals、取消/late reply/stop drain；实际park/唤醒与资源观察，不用sleep猜时序。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: ask-reply
id: "av.parity.1090.ask-reply"
tags: ["av-language-parity", "plan-1090"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
task Q { on { n int -> { reply(n + 1) } } }
fn main() int { let h = Task.spawn("Q", 1); let n = await h.ask(41); h.stop(); return n }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: spawn-outside-main
id: "av.parity.1090.spawn-outside-main"
tags: ["av-language-parity", "plan-1090"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
task Q { on { n int -> { reply(n + 1) } } }
fn request() int { let h = Task.spawn("Q", 1); let n = await h.ask(41); h.stop(); return n }
fn main() int { return request() }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: async-block-value
id: "av.parity.1090.async-block-value"
tags: ["av-language-parity", "plan-1090"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int { let f = ~{ 40 + 2 }; return f.await }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: fifo-capacity
id: "av.parity.1090.fifo-capacity"
tags: ["av-language-parity", "plan-1090"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
task Echo { on { n int -> { print(n) } } }
fn main() int { let h = Task.spawn("Echo", 1); h.send(19); h.send(23); h.stop(); return 42 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
19
23
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-semantics ace-av ace-lowering
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [x] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [x] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [x] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [x] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [x] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [x] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [x] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [x] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [x] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [x] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | ~{}、.await/.go与前缀await/go的实际旧合法拼写，~T函数、future传播/容器/返回、Task.spawn位于普通函数/handler及TaskRef参数/record字段。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | task/on/start/stop、typed/variant/string/guard/else消息、初始化参数、ask/reply及错误、FIFO/有界背压、关闭/late/取消语义。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 系统/FFI/异常/生成器与任务组合按已交接口接入；不把host权限转为catch，不承诺所有线程调度轨迹相同或新增无限并发。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-SCHEDULER。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态：

- [x] **T-01** work：已核七项归档依赖、普通 AV 入口、13 份旧 Actor 文档、Rust / Cookbook 语言角色及 L-29/L-31/L-43 承担路径。
- [x] **T-02** work：候选提交 1d56fed19 走真实普通 AV 执行。~{}、~T、.await/.go 和前缀形式、Future 返回/容器、TaskRef 字段/跨函数/跨 handler 已有同源实际。
- [x] **T-03** work：候选提交上已观测原值/双流、生命周期、类型/隐式及显式变体、guard/else、容量和关闭/晚回复。余下验收由 T-04..06 核销。
- [x] **T-04** work：普通链组合生成器和程序错误，实际撤权、宿主失败、取消与预算分终态；已交 AV F0 provider 在等待链实跑，撤权零派发。
- [x] **T-05** work：41 项唯一活动 AV 检查，13 份旧文档与四个原有语言输入保源和各自预期；父侧 8 项隔离、身份与判决控制通过。
- [x] **T-06** work：冻结普通 AV 及共同职责组。AV 组保留一个基面原红，两个墙钟预算首次失败已用同归档重放通过。
- [x] **T-07** review：逐 AC 复核代码、实际子项、源身份和审定 Spec 增量，最终复审 pass。
- [x] **T-08** merge：真实落主、归档校验和专属路径清理已核实。

本计划 T-01..08 已完成。周期批量回归按独立到期规则执行。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- `stage: goal | plan_id: PLAN-1090 | plan_revision: r3 | round: 1 | from: work entry | status: executing | authorization: /goal 完成计划1090 | worktree: D:/autostack/.wt/lang-1090/auto-lang | commit: 196ce23f7 | next: work`。伴随只读 auto-down 树为 `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。


- 2026-11-21 工作检查点 r3 位于 `plan-1090-dev@1d56fed1903cefd1c79c5631f6230584276af2da`；基面 `196ce23f7`。普通 Auto 通过共享 CST、绑定/Checked HIR、普通 MIR、固定 ABC 和现有 AV scheduler。复用生产者，无旧 VM 执行。新 lazy future 保持拥有环境，消费时执行；挂起错误携带原位置，经现有 Invoke 边交语言 catch。取消、资源终态分列。SCHED_FUTURE/DETACH 固定 wire 以及父子降级版本 8 一并核验。
- 受检语料已落 `crates/auto-lang/test/cases/language/async/tasks-actors.case.ad`。13 份旧 Actor 文档保原 ID、源码与 VM stdout，历史原件含 1066 签章来源不改。PLAN-348 三条 task22 源码与 Cookbook 002 的纯语言输入原样复用。L-29 对应延迟体、后缀/前缀、保值与隔离捕获，L-31 对应 ask/reply、未回复/复用/晚回复，L-43 对应 task/on 各种消息、生命周期、句柄携带。需要 Tokio 原生运行、线程池、信号、文件服务或 HTTP 宿主的非语言提供方检查保持原 owner，现场 AV 无授予。旧 Phase3 Rust AST 单测仍由旧 parser 持有。
- `CARGO_PROFILE_DEV_DEBUG=0 cargo build --manifest-path experimental/ace/Cargo.toml -p ace-av --bin av-worker --locked` 与父侧 `cargo build --locked --offline -j 2 -p auto-lang --bin auto-case` 成功。固定 ABI 镜像随版本同步。子侧真实逐例 40/40，含 17 条旧源复用；逐例范围 0.039–0.156s，材料 `scheduler-cases.json` SHA256 `1233d015d0e0f1fb4289328851e09de2b74d4a2b1793b245a008428e8416060a`。
- 四项受检原生观察测试全绿，分别 0.049s, 0.046s, 0.046s, 0.047s。真实条件为 mailbox send park/wake=1/1；等待及取消路径在清理后均无 live owned text，指令预算从真实已挂起状态发出 Resource。材料 `scheduler-test-results.json` SHA256 `327d11abc6826e1df130d679db398fad61962d70abda418cd3d1b9f5a21bb50e`。
- 父侧 6 条区分性控制全通过。AV PASS+非AV SKIP、错 AV 预期 FAIL、零有效 AV 和重复 ID 拒绝、移除非AV ignore 后 AV 视图保 NOT_RUN、单选非AV传入缺失 worker ERROR。首轮控制的 suite.id 误用点号，第二轮发现父子降低版本镜像差异，以及 AC/MN 覆盖变量选错触发本地 release worker 准备，已终止。有效重跑使用 `AC_MN_CASE_WORKER` 缺路径，无非AV执行。材料 `fc-controls.json` SHA256 `3e078e9d212ee5906daf472111bfa32c7a137acd9741e5d941a55f49992056cb`。
- 首轮冻结候选 `f25e759ce3da240d056245fb4483028538865d7e`。8 个 invocation 的 debug 准备 334.892s，执行 175.451s。新增 41 项均通过父侧独立预期。完整语料发现 5 个既有 `!T` 反例退化，固定 AV 的 1083 两个交叉检查也抓到相同问题。Actor hook `!` 抢先匹配普通 `!T` 返回注解，修复使用 AAC 已有的完整返回类型优先规则。本轮空记录用例原红见 PLAN-1091，布局源码 `2c9adfefb425a7bf56f1fc6d3355d41b6962d991` 与 1088 引入该差异后的基面一致，失败仍归 `D-1088-LAYOUT-01`。其它组执行 PASS。保留首红，等待最终候选重放。
- 同一普通异步体的 `console.stderr` 撤权对照已在真实 worker 得 `verify/host.unauthorized`，原 `main.at` 字节 17..33，宿主派发、catch 和两条流均为零。实际宿主 I/O 失败的 conformance 维持 `Infra`，不改变程序 catch。补充父侧源变异与旧活跃文件重复控制后 8/8；源变异使 source identity 改变且 oracle identity 相同，exit 1，两个文件同时声明同 ID 以 exit 2 判失败。
- `a74ef71c2e31b0bb47ba3058fd50188c364a90f9` 包含并行落主的 PLAN-1091。专属树重放、保留其 suite 和迁移表以及 worker 的 CT 测试，修复候选 `8b35174783217d3e34f7a865767088a092e13aa2` 待冻结回归。源 `use.rs f0_text` 经现有授权和生成 shim 运行；首次组合探针暴露闭包先占 foreign 函数索引，复用原延迟重定位规则修复。既有 `execute_foreign`/`execute_console_foreign` 在实际存在 scheduler 指令时带同一份已授权调用表进入合作调度，其余既有执行保持。外部探针 debug 冷/增量 71.000s/10.800s，显式 Rust F0 fixture 准备 0.444s，复用冻结提供方单次运行 0.042s。值 9，park/wake=1/1，已授权实际 ABI 派发 1 次。源撤权和运行绑定缺失分别验证派发零。普通 Case worker 仍使用仅 console provider 的入口。
- 追上 PLAN-1091 收尾文档 `282e6de9b` 后冻结候选 `eb6c9a528d393059acff54784edaa850d8fe6900`。`git range-diff` 对两项实现均为 `=`。09:58:25 桌面空闲启动准备；最后一项 AV worker 重建与提前运行探针发生 Windows 可执行文件锁冲突，AV 工具准备在 27.033s 处以 OS error 5 终止，未执行该轮固定组。六份 Rust 投放归档已准备成功。重试对同 HEAD、配置与命令的真实归档重核哈希后复用，等其它任务退出，再串行完成工具准备和执行。`auto-test-case` 复用首轮，根 lock/manifest、模块及 auto-atom/AutoDown parser 输入和固定组配置均无差异。
- 已绑定 `eb6c9a528d` 的冻结 debug worker 与实际 `auto-case run --target av --case <ID>` 重放首次五个 `!T` 退化用例。`result-fn-str-error`、`err-enum-payload`、`multi-error-family`、`nested-result-option` 与 `legacy.16_option_result.020_result_propagate` 均 PASS/exit 0，分别 596/410/406/400/406ms。另一次准备的默认配置启动辅助 `store-gc --release`，检查发现后终止本计划所属 PID 17788/29808 和其编译子进程。最终准备以现有 `configured_tools(debug_tools=True)` 约束本轮四个实际工具，已准备 Rust 归档均为 test；正式 AV 原子结果仍待整门。

### 冻结验收记录

`eb6c9a528d393059acff54784edaa850d8fe6900`，基面 `282e6de9b3674937f3b48d19a868b9b3ef36f15e`。`av_phase.py` 调仓内 `scripts/fixed-tests.py`，按 AV 阶段保留 `case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering ace-artifacts`。先 `--prepare-only --out final-debug-prepared`，再 `--run-prepared final-debug-prepared/prepared.json --out final-debug-run`，路径皆在唯一材料区。六个 Rust test 归档按 source_state、固定配置、原 Cargo 命令、退出码和哈希复用。最终运行工具 `case-parent`、`av`、`store-gc`、`execution-fixture` 均为 debug。追加准备 3.174s，完整父运行 200.321s。启动时无竞争编译；运行期间出现 AutoUI 工具构建、UI 105 测试和 PLAN-1055 准备进程。

| 固定组/范围 | 实际执行 | PASS | FAIL | 原条件/过滤跳过 | 子命令墙钟 s |
|---|---:|---:|---:|---:|---:|
| case-av-language-parity | 487 | 487 | 0 | 161 | 40.548 |
| ace-source | 932 | 932 | 0 | 1 | 9.774 |
| ace-syntax | 159 | 159 | 0 | 3 | 5.309 |
| ace-semantics | 295 | 295 | 0 | 10 | 15.976 |
| ace-av（首次冻结执行） | 1492 | 1489 | 3 | 51 | 77.826 |
| ace-lowering | 9 | 9 | 0 | 0 | 2.367 |
| ace-artifacts | 43 | 43 | 0 | 7 | 16.593 |
| auto-test-case（原冻结执行复用） | 157 | 157 | 0 | 0 | 4.210 |

准备清单 SHA256 `e9ec3e9583a64ffe5d42603cd2caa34fc5542dd8b29ea69dbb857af38396e6d2`，执行结果 `40547bc2dd25ddc85c433d04535582ae24079774125d021532636bd8974764e4`，筛选/准备脚本 `82055e8ceacc55a9221f77c7d44fdf7a4b0c464d18b10b7dbc479f470056e5cf`。27 条非 AV 入口和 1 条 Linux 专属排除沿用 [PLAN-1089 具名清单](1089-av-parity-generators-and-stream-lifetimes.md#9-复审记录)，性能和原适用规则照常；排除项不计 PASS。本计划 41 条 AV 检查单项 548–736ms，13 条既有非 AV 文档检查为 SKIP。五条新增真实调度观察分别为：取消与清理 302ms、发送挂起/唤醒 259ms、owned 捕获迁移 206ms、宿主 I/O 失败 365ms、指令预算 366ms。未增加性能或系统平台门。

冻结 AV 组的 `D-1088-LAYOUT-01` 为已登记旧失败。`kv_zip_1064::v03_zip_negative_corrupt_bomb_and_preserve_av` 和 `::v03_kv_wire_oracle_and_reopen_av` 分别在 21.227s、21.287s 因 `ExecutionWall: platform call exceeded wall budget` 失败，较早的冻结轮为 7.680s、15.340s PASS。用同一 `ace-av` nextest 归档的原命令加两项精确筛选和 `--test-threads 1` 重放。保原输入、平台、oracle 和预算。两项分别 2.964s、9.222s PASS，子退出 0，整命令 17.867s；其它编译仍在运行，重放子集无 AV 组内并发。原失败按墙钟资源争用留档，汇总有效结果 1491 PASS + 1 原 FAIL。归档 SHA256 `95d2c3780b8caa50552a6a4039df8d631179a6f9c284794a4ba93e051f4cda80`，重放 stderr `b312c31163104dbee3f77f0e7a5c8105abfe383cbd284e42dcb80e6410483a29`。nextest 自建临时解包目录核验已不存在。

stage: work | plan_id: PLAN-1090 | plan_revision: r3 | outcome: pass | status: execution_done | worktree: D:/autostack/.wt/lang-1090/auto-lang | branch: plan-1090-dev | implementation_commit: eb6c9a528d393059acff54784edaa850d8fe6900 | work_tasks: T-01..06 | next: review


### 2026-11-21 r3 · 最终复审

在原专属树以冻结提交、真实归档和父侧逐例报告重新核验。独立性限制为同一会话内重建证据，无额外模型或审查树。base `282e6de9b3674937f3b48d19a868b9b3ef36f15e`，reviewed commit `eb6c9a528d393059acff54784edaa850d8fe6900`，AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。七个硬依赖均取 master 已归档正式来源，按 1078/1081/1083/1085/1086/1087/1089 顺序 Git blob 为 `5e9f7c6361e860284eeb2eb3054615e2c2b41bc2`、`0cc492cd24ed519a40e2dcfc56a7fe583f809c93`、`c96f9260bd17672ed5c2ae6c3c833fb0356c914f`、`11b5578a16096ec123c8ca484fe9cde4342cc958`、`91d21f78eb7b4367f5651c89b28a87eb34ac7224`、`bf09af4cc46537e35d0f09f7d0f7a22441841cce`、`fff6c352683b893182de0c88c98d1c1d3d4c2e15`。

| 验收 | 判定 | 源、实际对象与核验 |
|---|---|---|
| AC-AV-ONLY / AC-FC | pass | 自 suite include 到唯一 async 文档，核 41 个普通 AV 检查、13 个历史 check SKIP 和迁移表 17 条来源。八个父侧控制观察固定 oracle/source 身份、重复活 Case、零有效 AV、未选目标和缺 worker。忽略或未选状态未冒实际执行。 |
| AC-01 | pass | 四个原计划正例、旧源码与例外状态均经正式普通 AV 父侧判定。typed/variant/guard、前缀/后缀、存储、嵌套创建和 FIFO 使用受检运行结果。 |
| AC-02 | pass | 实际诊断保留原路径和字节区间。权限拒绝零宿主派发/零 catch，程序错误由等待链交现有 Invoke。宿主 I/O、取消、预算保真终态，真实 Rust F0 只执行一次。 |
| AC-03 | pass | TASK-HANDLE 跨参数、返回、字段和实例独立。SCHED-FAILURES 具名反例与 conformance 同源，真实 park/wake 和 text ownership 账归零。 |
| AC-04 | pass | 13 个旧文档 Git 对象、Auto 主体、VM fixture 与 AV 独立 stdout 逐份复核。独立历史 VM/A2R 条件及各端原 owner 留在迁移表，Rust 内联和 Cookbook 原义务保输入和预期。 |
| AC-05 | pass | 从 schema/投影、任务 desugar、受检签名与 binder 到 MIR、ABC 验证、AV 执行完整贯通。调度不按 Case ID 判断，旧 VM 不派发，类型校验先于 scheduler handle 编号。 |
| AC-06 | pass | 复核冻结配置、已准备入参、子终态和两条重放。同一包的新例耗时均小于 10s。既有 warning 均有原位置，无新增 debug 输出；`git diff --check`、新增 conformance 和 actor desugar 的 rustfmt 校验通过。保留 D-1088-LAYOUT-01 原失败。 |

两项额外反例直接请求冻结 AV worker `2642812332cef84438a970b9d1638d9a77d574b6562b7dd139caf69f6122258a`。将文本发至 `TaskRef<int>` 的跨函数 handle，在 bind 得 `source.sched-argument`、76..81、派发 0，0.0442s。对同一 future 两次 `.go`，run 得 `lang.error` 与原位置，已准入的控制台副作用共 1 次，0.0446s。材料 SHA256 `993326108eb28ca1c281a76ebf880adffc0c7ad00fc97c8df748d3744f849fd2`。复审复用完整固定组和既有八项父控，无新增构建。

SD-01/02/03/FC 与实际语义和四个目标对应，目标基面 blob 依次为 `8cb93813bf3111cac6dd956d1f4107ccdb727520`、`0b9e2ff8fb66715b1e594aba25008343a233174c`、`6084c3971b56ab2e0bb40675f4110ff5ee245363`、`232f87dbc88233f161560031fae6483914b9179c`。审定规范增量摘要 SHA256 `176520a46d8441b1bbebe51064e3a80faf18a9021d958cd463d036f77ba23384`。不含超合同后移或本包未关闭失败。既有 D-1088-LAYOUT-01 仍归布局 owner，通用外部库扩展归 PLAN-1093。发布 Spec、落主 smoke、实际归档校验、删除核对和到期批量回归留给合入。

stage: review | review_scope: final | reviewer: Codex 同会话、独立证据重建 | plan_id: PLAN-1090 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1090/auto-lang | branch: plan-1090-dev | reviewed_commit: eb6c9a528d393059acff54784edaa850d8fe6900 | base_commit: 282e6de9b3674937f3b48d19a868b9b3ef36f15e | acceptance_results: AC-AV-ONLY、AC-FC、AC-01..07 pass | next: merge


### 2026-11-21 r3 · 交付与归档

- `prepared`：冻结实施 `eb6c9a528d393059acff54784edaa850d8fe6900`，规范及导航提交 `c4ecdf12abb7be08d78ae51160a872ccf17dc91c`，二者间只增四份审定 Spec 和两处路线导航。PowerShell 对专属树提前扫描为 clean。最终冻结组沿用复审证据，无重复构建。
- `landed`：`git merge --ff-only plan-1090-dev` 在主检出退出 0，master 正常快进至 `c4ecdf12abb7be08d78ae51160a872ccf17dc91c`。本包发布接口 `IF-AV-PARITY-SCHEDULER` 对应此源码身份。
- `canonical_verified`：四份 Spec、canonical 语料及迁移表的工作文件均与 master Git blob 相同。主检出用冻结 parent/worker 重放同源码，`typed-handle-return` 实得 42、631ms，`denied-async-console` 实际授权诊断匹配、450ms，均 PASS/exit 0。父报告 SHA256 分别为 `cb5ef839c034de229a05e74cde8e33ba41eda5e88abd2225b4c2f7f8e67e85b3`、`2ee64a53d979a8aa55836ae405e0edacd7bba75c4e531664abd00d67c15146ff`。
- Canonical 源 SHA256：[language-parity.md](../../specs/auto-av/language-parity.md) `01b8c7eb858ab7d846afd6de0c41dbdd206a8c62aa57e3a49f7cdf1de29930cb`，[m2-scheduler.md](../../specs/ace-migration/m2-scheduler.md) `5d93fedd59901c43c1eb9b210095ce361d4ba7f7a7d9e0f27022193e80141437`，[m2-control-effects.md](../../specs/ace-migration/m2-control-effects.md) `aa4859b5add8f74fd01eda5ce4d28411780a363b95959daf58a25b3ec1fd9ac2`，[feature-organized-corpus.md](../../specs/auto-test-case/feature-organized-corpus.md) `abfd09c7454258cd99c1cf28ab99a1407006b0796baecab75a280677e1dc1675`。语料和迁移表分别为 `fe0aaee2b5e51bc28ecf5a234752e2c985ada37861d9207aff379a5d7e52c2c2`、`f8bd4e0e8d273a3561a5a95c51f6ac7c5cb305a8e02ae10826a1f44c4fbbc37d`。
- 主检出 debug `auto-case.exe`（2026-11-20T04:43:43Z）、debug `av-worker.exe`（2026-11-20T04:44:11Z）和 release `av-worker.exe`（2026-11-19T03:10:13Z）早于交付，release `auto-case.exe` 缺席。交付验证采用冻结的 debug 工具。工作树与材料清理仍由 T-08 收尾。

- `archived`：实际路径 `docs/plans/archive/1090-av-parity-tasks-async-and-actors.md`，r3、`completion_kind: delivered`。复核 7/8 任务、9/10 验收，T-08 与 AC-08 待清理。独立解析各出链和本包六个导航/Spec 入链，均指向实际文件。其它会话中的 PLAN-1113 草稿仍用原活跃相对链接，归其 owner 更新。

### 2026-11-21 r3 · 清理收据

交付提交 `c4ecdf12abb7be08d78ae51160a872ccf17dc91c` 和归档提交 `6030a26de5f441e49ffe7f296ad3ae141362f1fd` 均在 master。按原命令、归档 manifest 和实际路径核对了根/ACE target、脚本辅助 target、FFI 专用 target/源/提供方、父子运行报告和归档证据。nextest 自带临时解包目录已自动清理。两个工作树、组目录和唯一材料区都完成绝对路径及归属校验。

| 已删除路径 | 文件数 | 文件字节总和 |
|---|---:|---:|
| D:/autostack/.wt/lang-1090/auto-lang | 126677 | 20635770936 |
| D:/autostack/.wt/lang-1090/auto-down | 1318 | 27021855 |
| D:/autostack/plan-artifacts/plan-1090 | 9442 | 2933553619 |

所有路径在 PowerShell `wt-guard.ps1` 输出 clean 后清除，两个 `git worktree remove`、分支删除及材料根移除均成功。2026-11-21T10:26:29+08:00 确认工作树清单、分支、组目录和材料 scope 均已清除。构建中间产物随树清除；证据摘要先写入归档，无待保留件。批量回归按合入后 %5 到期触发，单独从主检出执行。

stage: merge | plan_id: PLAN-1090 | plan_revision: r3 | outcome: pass | reviewed_commit: eb6c9a528d393059acff54784edaa850d8fe6900 | delivery_commit: c4ecdf12abb7be08d78ae51160a872ccf17dc91c | archive_commit: 6030a26de5f441e49ffe7f296ad3ae141362f1fd | completion_kind: delivered | cleanup: removed | next: batch due check

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
