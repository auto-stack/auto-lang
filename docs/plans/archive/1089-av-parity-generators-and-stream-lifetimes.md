---
plan_id: PLAN-1089
status: archived
completion_kind: delivered
feature_name: AV生成器与迭代生命周期补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1083, PLAN-1086, PLAN-1087]
supersedes_spec_components: [docs/specs/ace-migration/m2-control-effects.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1089] AV生成器与迭代生命周期补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`generators/yield-close.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-11**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

扩生成器在普通工程中的真实可用组合，补合法借用参数与close后的错误/资源语义。

本包交付`IF-AV-PARITY-GENERATOR`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- ~Iter<T>与普通函数/泛型/枚举/资源值跨模块、yield前后locals、间接返回、连续next/耗尽、for-in drain/提前break。
- 借用形参生命周期先基于跨挂起活跃性与escape判定：不跨yield合法用例支持；真正跨yield/逃逸依语言借用规范，不用复制owner掩盖借用。
- close幂等、finally恰一次、close抛错后状态稳定、取消/异常退出释放；流式库API本体留标准库owner。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1089/auto-lang`，分支`plan-1089-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1089/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

965已有惰性next/yield/close/finally和间接返回，不重做状态机；借用生成器参数即使不跨yield也一律拒绝是当前真实限制。旧生成器只有4例，不能以总数小当完整覆盖。

既有来源与边界：

- [indirect_return.case.ad](../../../crates/auto-lang/test/vm/22_generator/004_indirect_return/indirect_return.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：22_generator；cookbook/iter中真实语言迭代角色；Rust generator/drain/close回归。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [ownership.rs](../../../experimental/ace/crates/av-core/src/ownership.rs) | `2d2799149932028861c15c2107e5dd0b44b4330da2ccf25c01b511a2481044d7` |
| [av.rs](../../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1083, PLAN-1086, PLAN-1087；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [ownership.rs](../../../experimental/ace/crates/av-core/src/ownership.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/generators/yield-close.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-control-effects.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1089-原文档清单)静态主承接 **4份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1089.indirect-generator` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=6; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1089.close-finally-once` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1089.no-borrow-across-yield` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1089.yield-neighbor` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.yield-outside-generator | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `GEN-CLOSE-STATE` | AC-03/AC-04 | work / T-04..05 | 显式close/耗尽/提前break/取消/close语言错误后再次close各终态，不能仅一次yield正确便关闭所有生命周期。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `GEN-RESOURCE` | AC-03/AC-04 | work / T-04..05 | Str/List/variant payload在producer→yield→consumer三边的own/move逐个验，至少2次yield与早停后parent live=0。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: indirect-generator
id: "av.parity.1089.indirect-generator"
tags: ["av-language-parity", "plan-1089"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn counter() ~Iter<int> { yield 1; yield 2; yield 3 }
fn make() ~Iter<int> { return counter() }
fn main() int { var n = 0; for x in make() { n = n + x }; return n }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 6

#### stdout

```text
```

#### stderr

```text
```

### VM result
#ignore
check-id: "legacy-reference"
final-newline: false

```text
6
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: close-finally-once
id: "av.parity.1089.close-finally-once"
tags: ["av-language-parity", "plan-1089"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
var closed = 0
fn values() ~Iter<int> {
 try { yield 19; yield 23 } finally { closed = closed + 1 }
}
fn main() int {
 for n in values() { break }
 if closed == 1 { return 42 }; return 0
}
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
## Case: no-borrow-across-yield
id: "av.parity.1089.no-borrow-across-yield"
tags: ["av-language-parity", "plan-1089"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type P { v int }
fn values(p P) ~Iter<int> { let n = p.v; yield n }
fn main() int { let p = P(42); var sum = 0; for n in values(p) { sum += n }; return sum }
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
## Case: yield-neighbor
id: "av.parity.1089.yield-neighbor"
tags: ["av-language-parity", "plan-1089"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int { yield 42; return 0 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "resolve"
code: "bind/source.yield-outside-generator"

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
诊断阶段/码是本路线目标合同；新码或现有family包装差异由本包T-01核对/冻结，修正必须保持同一错误类别、源位置与拒绝时机，不能降为any-Err。
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
| T-02 | work | T-01 | ~Iter<T>与普通函数/泛型/枚举/资源值跨模块、yield前后locals、间接返回、连续next/耗尽、for-in drain/提前break。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 借用形参生命周期先基于跨挂起活跃性与escape判定：不跨yield合法用例支持；真正跨yield/逃逸依语言借用规范，不用复制owner掩盖借用。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | close幂等、finally恰一次、close抛错后状态稳定、取消/异常退出释放；流式库API本体留标准库owner。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-GENERATOR。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：冻结源、原预期、消费者、实际阶段和高风险实现边界。见 §9 基面与用例冻结。
- [x] **T-02** work：四份旧生成器原样通过。跨模块记录模板 List/枚举负载、分支间接返回、循环局部与四次 next 检查通过。
- [x] **T-03** work：直接 for-in 的借用前段用普通受检形参，跨挂起及逃逸在 bind 分开拒绝；只读、mut 写回、循环挂起相邻例均通过。
- [x] **T-04** work：关闭前未启动、正常耗尽、提前 break、嵌套 finally、关闭抛错后重复 close 与 next、父端账本与取消控制通过。
- [x] **T-05** work：唯一作者载入与四条 VM 迁移映射经生产发现验证。故意红、目标隔离与结构准入七条对照通过。
- [x] **T-06** work：冻结实现及定向证据。§9 交 IF-AV-PARITY-GENERATOR，按仓规定独立复审后执行冻结并集，AC-06 在实跑后关闭。
- [x] **T-07** review：按冻结提交重建 AC、语义和语料证据，核 SD-01、SD-02、SD-FC，见 §9。
- [x] **T-08** merge：冻结组对账、规范落主、归档与守卫清理均已完成，见 §9。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- `stage: goal | plan_id: PLAN-1089 | plan_revision: r3 | round: 1 | from: work entry | status: executing | authorization: /goal 完成计划1089 | worktree: D:/autostack/.wt/lang-1089/auto-lang | commit: c941db401 | next: work`。

### 2026-11-20 r3 · T-01 基面与用例冻结

- 四项依赖在基面 c941db401 核对。PLAN-1078 普通 AV recipe、公共 suite 与检查级选择，PLAN-1083 枚举和模式、PLAN-1086 owned 列表/借用准入、PLAN-1087 普通入口错误展开均已落主。读取历史 §9 与当前实现，沿用 ac-av/language-source-run-v1 和 portable av.closures/1。
- 四份 VM 文档逐案核对源码 sidecar、标准输出 oracle 及末尾换行。源 SHA256 前缀依次为 2815e1baa84fb291（sum 6）、b0588f713fe99695（10,20,30,）、610f4a480e693aa6（abc）、ae40e0d7d21f4f28（间接返回 6）。同源迁入 generators/yield-close.case.ad，旧 VM ID、源路径、check 和 oracle 保持，旧 VM 读者经 CanonicalVmCases 取源。AP-13 是 PLAN-827 冻结清单的存在性及指纹行，保留 sidecar 供历史核查。004_indirect_return/indirect_return.wrong.out 是故意红的独立对照。
- Rust 角色实查 ace-av/tests/cases/aggregates_m2_965.rs 的惰性、多次 next、幂等、早停、资源与取消、关闭错误、双实例与相邻拒例，另有 plan1086_ownership.rs 资源病例。保原窄轮廓作为兼容见证，普通轮廓增对应断言。cookbook 无 ~Iter 或 yield 作者，rayon 迭代属于 Rust 外接并行所有权。旧 a2r/21_generators 两条为 Rust emit 金样，第二条使用 ~Stream 与 while 旧拼写，独立保留原源码和后继 owner，本 AV 阶段未运行。
- 按基面源码准备的 debug 父端与 worker 运行 auto-case run --suite probe-input/generators.suite.ad --target av。首轮 3 PASS、5 FAIL、5 SKIP，扩展探针修正实际错误码后 6 PASS、8 FAIL、5 SKIP（父退出 1）。两个旧文本用例分别在 bind 缺整数 .to_str()、text 拒 ~Iter<str>。普通全局关闭例补模块授权后仍因包绑定拒流类型。跨模块 payload 也触及包面准入。yield 邻例 source.yield-outside-generator 在 bind、main.at 16..24，父端阶段选择 resolve 接收 bind。借用不跨挂起正例在 bind、main.at 27..30 被现有参数形状门拒绝。关闭轨迹内层错误跳过外层 finally，双次关闭还暴露 mir.use-after-move。
- 借用机制边界。生成器第一次 next 时才执行体，保留懒启动与调用者写回。借用值须留在调用者可证明存活的作用域，经受检借用调用交给首次执行步。测试挂起后的控制流、finally 重放和返回或存储逃逸。T-03 负责接线和大小反例。
- 基线 debug 预备工具准备 112.867s（父端）与 34.941s（worker）。清单 SHA256 4493c8e723c75fe91c01bb0caf725ccdb7e36f2088464788ff14f1068d687262，扩展探针日志 SHA256 0de965328ea773423c190d8dbb6d35b695e6e38023cf0ca6e679aac436f1ff56，均在 PLAN-1089 材料 scope。源比较与探针为施工验收输入。

### 2026-11-20 r3 · T-02 至 T-05 定向候选验证

代码候选 94be8bc4ddd042e45ea5d95537598a3db9cb05b2，基面 c941db401，auto-down 45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5。标题保留 r3 原 AC，借用边界限直接 for-in 可证明的调用者生存区间。生成器借用实参经普通受检调用访问前段 original owner，可写形参完成真实 write-back。移出之前检查跨挂起本地引用，只将存活的拥有值/标量放进状态记录。体内取值如 p.v 作为数据持久化。逃逸以及跨界视图按原源区间在绑定拒绝。生成器的纯 owned 生产者仍保持惰性，闭合后的 state 为 -1。

普通模块绑定给 stream type/payload、yield 和枚举经现有 AST pass 分配有来源的符号；泛型实参利用原模板具体化。原 .to_str() 与 ~Iter<str> 由普通面复用已有 display 和拥有文本容器规则。仍经过共享 HIR、MIR verify、ABC verify 与 AV，历史窄面由原验证组核。异常边先在 caller 保留已末止记录，再转交原 record 给 close，以可重用本地状态保护实际写回 ABI。悬挂 finally 逐层执行并保留后发错误覆盖顺序，释放后传递错误。原显式 fail 和移出位以验证器的实际传播为证。

| 定向范围 | 结果 | 准备 | 执行 |
|---|---|---:|---:|
| 普通 AV 语料 25 检查 | 25 PASS，5 VM SKIP（四迁移、一新参考），0 非 AV 执行 | 预备清单 iteration9-prepared/debug，源格式化未改变 token | 父命令 3.793s，逐项 0.766–0.959s |
| 具名 GEN-RESOURCE/GEN-CLOSE-STATE 父侧观测 | 3 PASS，7 本次未选 | debug 已绑定 rlib，1.030s | 0.170s，Rust 测试体合计 0.07s |
| 生产 legacy-inventory | 4/4 VM ID 经 canonical 读取，仅原 vm-stdout | 同一父工具 | 3.103s |
| AV 选择/版本对照 | 7 个对照符合 exit，见下 | 同一父工具/worker | 单个 0.021–0.442s |

父侧生命周期三例：payload-owner-early-stop 中连续两份实际文本、List 与 Variant 所有权事件，断言真实创建和 alloc/free ID 等集/live=0；cancel-during-suspension 第二个输出的宿主取消，断言两条独立双流输出、Cancelled、0 语言 handler 和所有 owner 释放；close-throws-then-terminal 两次 handler 进入（close 内接收与 caller 捕获），再次 close 不进入，最后 next 耗尽、42，alloc/free 等集。原文本 Str 组合普通 AV 有到场的 typed list 读取现象，错误 oracle 和假早停分别会使值 42 / 内容断言变红。

迁移：错误 VM oracle 处于 #ignore 或已打开但 AV 未选时均父退出 0、AV receipt 1。故意改 AV oracle 或源均退出 1。重复 ID、忽略全部有效 AV 和缺 AV required 各退出 2、AV receipt 0。相同源前门只提交已选 AV 工作，旧 VM 检查无执行收据。读者对照文件哈希 70636324d7f357e088bf78a682874b12e7d7fb093993511dcd0fe459b667f7a8，AV 候选日志 b6029118aec74bb08ccb04b611a9fa246e2b7511d617b2b5366e1fb378e97064，父侧资源日志 c74ca12b9fc4db63e5bd5ccdd86181ce77067d8808c99f8e94c2d3d80e2892ee，七对照摘要 cc27f02707960e03880f11ad45c139884fd00072bb85d354859d5cc72fede3e2。材料均登记在唯一 scope。

编译器迭代中 ACK 差异。初次 close 的重放 loop ID 重复，改为每次回放词法重编号。用 MIR 复现 Close 使用的 owned mut ABI 正常边才写回，失败边显式已末止记录解决。已拒绝将借用 owner 复制入生成器状态。错误 AST 试例和实际模板实例化缺口分开处理，原用户草案/旧 oracle 保留。组冻结工作 T-06 待跑。旧 rust emit / 非 AV obligations 保原独立责任与未执行状态。

### 施工冻结前的独立反例

F-1089-01（AC-01/03/06，T-02/04/06）。候选 94be8bc4d 的两个同型流各含 finally。同一普通工程依次拉取 19/23，按顺序关闭后应得 42 且轨迹 [1,2]。实际父端退出 1，源码转 Atom 时校验发现 duplicate loop id q1，未进入 MIR。复现 review-input/review.suite.ad 中 two-finally-producers-interleaved，auto-case run --target av。根因是同一合成 next 定义合并各生成器的源局部循环号，close 已做的词法重编号没有覆盖 next。另一个独立探针 nested-close-errors-from-calls 通过，证实真实调用抛错后外层 finally 执行并覆盖消息，后续 close/next 仍终结。F-1089-01 保持开启直到修复提交上的相同反例及相邻例转绿。

### 2026-11-20 r3 · 冻结、定向修复与独立复审

F-1089-01 在 e72afc9e43155c71a229462cb092a7919fe10c75 关闭。合成 next 为每个复制状态的局部循环重编号，状态分派跳转保存独立身份。原反例两个同类型生成器各保独立挂起状态，返回 42 且关闭顺序留下 [1,2]；外层 finally 覆盖内层调用错误的相邻例也通过。定向 27 PASS、5 VM SKIP、0 FAIL，dev 准备 32.035s，父命令 3.284s。两条后加检查分别 0.58042s 和 0.95649s。构建替换前一次旧 worker 回读保留首红，不计此轮实际。

第一次准备到 94be8bc4d 的 store-gc 时发现通用配置使用 release，终止仅归本包的准备进程。203.841s，退出 1，未执行用例。新准备移除没有实际消费者的 CLI，保留内置 GC 测试。新选择具名排除 27 个非 AV 检查和 Windows 下一个 Linux 检查，使用 case-parent、AV worker、execution-fixture 三项 debug 准备。冻结组待合入门，AC-06 保持待实跑。

IF-AV-PARITY-GENERATOR 冻结来源 e72afc9e43155c71a229462cb092a7919fe10c75，Windows x86_64。普通 Auto 源经共享受检 HIR/MIR/ABC 与既有 av.closures/1，作者 crates/auto-lang/test/cases/language/generators/yield-close.case.ad，四 VM 映射逐案追踪原 oracle。recipe ac-av/language-source-run-v1，普通 console.stdout/stderr，工程文件经现有模块能力。程序错误按语言通道交付，取消与权限失败独立。三个规范源与最终运行身份在合入门对账。

stage: work | plan_id: PLAN-1089 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1089/auto-lang | branch: plan-1089-dev | base_commit: c941db401 | code_commit: e72afc9e43155c71a229462cb092a7919fe10c75 | dependency: auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | next: review

stage: goal | plan_id: PLAN-1089 | plan_revision: r3 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1089/auto-lang | commit: e72afc9e43155c71a229462cb092a7919fe10c75 | next: review

同会话独立核验阶段重建判定，无额外 reviewer。冻结差异 git diff --check 通过。auto-case lint 核公共 suite 的 13 个文件、315 个 Case 完整引用和结构，退出 0、1.335s。F-1089-01 已由同源重放和两条永久用例关闭。新例计时与资源守恒断言见定向记录。

| AC | 独立核验 |
|---|---|
| AC-AV-ONLY、AC-FC、AC-04 | 生产 legacy-inventory 四条均经 CanonicalVmCases 取唯一作者，仅 vm-stdout 保持原条件。原源、stdout、换行、check ID 对账。七元控制实际退出和 worker 数符合隔离。 |
| AC-01 | 普通 recipe 的原源码、间接返回、两个同型流、跨包模板与枚举、只读与可写借用、资源值与循环局部全走受检执行。父端独立判值与双流。 |
| AC-02 | yield 邻例 bind 16..24，借用跨挂起 33..40；逃逸定位返回调用。宿主第二次输出后取消且未落语言 catch。预算、授权与宿主分类沿原路径。 |
| AC-03 | 两项后提前退出、正常耗尽、未启动关闭、抛错后重关的 finally 次数和顺序均有断言。父端三条独立核 Str/List/Variant 真实创建、alloc 与 free ID、arena live、取消。 |
| AC-05 | 从源码 CST、模板实例化、共享 HIR/MIR/ABC 到 AV 实际调用。生产无 case ID 分派和新 parser。 |
| AC-06 | 新例计时均低于 10s，已冻结八组实际选择，最终执行待合入门。 |
| AC-07 | 三个正式 Spec 逐项确认，GOAL-002 与实现一致。 |

遗漏、延后、workaround 核查。借用生存区间直接 for-in 可证明，流内不容纳借用 owner；普通 mut 用原受检写回。外部流式库与非 AV 义务保原 owner。已有语义债 D-1037-01 保原责任。三项观察只运行进程内 Auto 编译，worker 在测试前准备。F-1089-01 已闭。复审 Spec delta SHA256 f47158efe3d39150339a4cda663531884b1ca0187c58acf84270c969e4f60c48。修复重放日志 SHA256 0f4504eb0d13108562a7fb85d05530a6a12bfaea5e34ea36311fda1fac67671c。

stage: review | review_scope: final | reviewer: 同会话独立核验 | plan_id: PLAN-1089 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1089/auto-lang | branch: plan-1089-dev | reviewed_commit: e72afc9e43155c71a229462cb092a7919fe10c75 | base_commit: c941db401 | dependency: auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | findings: F-1089-01 已闭 | pending_closeout_gates: 冻结八组、规范落主、归档与清理 | next: merge

stage: goal | plan_id: PLAN-1089 | plan_revision: r3 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1089/auto-lang | commit: e72afc9e43155c71a229462cb092a7919fe10c75 | next: merge


### 2026-11-20 r3 · 冻结固定组与落主关口

冻结代码 e72afc9e43155c71a229462cb092a7919fe10c75，auto-down 45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5。固定入口 `python av_phase.py case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering ace-artifacts auto-test-case --prepare-only --out final-prepared` 与同入口 `--run-prepared final-prepared/prepared.json --out final-run`，均在专属工作树执行，输入/输出全路径位于唯一材料区。脚本调用仓内 `scripts/fixed-tests.py`（`cargo t` 的实际入口），仅应用 AV 阶段筛选和工具准备集合。选择适用的完整公共语料、源码、语法、语义、AV 及 lowering/artifacts/test-case 消费组。保原 ignored、资源门和性能命名空间。准备阶段 8 个 invocation、3 类 debug 工具，外层 61.650s（内部 61.244s）、退出 0。分离运行外层 116.649s（内部 115.591s），父退出 1 保留原红。为等本机并发任务退出，准备与执行前分别空等 523.008s、82.505s，不计测试运行。公共语料原执行临时目录由父端成功清理，持久证据副本合入后统一清除。

| 固定组 | 择后列出 | PASS | FAIL | 排除/跳过 | 发现 s | 命令 s |
|---|---:|---:|---:|---|---:|---:|
| case-av-language-parity | 455 | 315 | 0 | 140 个检查级 SKIP | 0.000 | 26.644 |
| ace-source | 933 | 932 | 0 | 1 原 ignored | 0.546 | 9.985 |
| ace-syntax | 159 | 159 | 0 | — | 0.986 | 3.143 |
| ace-semantics | 295 | 294 | 1 | — | 4.922 | 6.205 |
| ace-av | 1478 | 1478 | 0 | 27 非 AV + 1 Linux 平台过滤 | 1.896 | 48.470 |
| ace-lowering | 9 | 9 | 0 | — | 0.341 | 0.474 |
| ace-artifacts | 44 | 43 | 0 | 1 原 ignored | 0.522 | 4.748 |
| auto-test-case | 157 | 157 | 0 | — | 1.713 | 2.228 |

公共语料一共 455 个检查。属于本计划的 27 个 AV 检查逐项判 PASS，墙钟 0.484–0.792s，五个 VM 检查 SKIP。三个父侧资源/取消/错误状态观测也在冻结 ace-av 固定组实际通过（0.067、0.069、0.072s）。原 `ace-source`/ `ace-artifacts` ignored 仍未执行。运行器标记全仓发现未完成，实际验收覆盖所选八组，所有所选 nextest 记录均有子项终态。语义组原始 nextest 汇总同时报告 10 项过滤 SKIP，未计 PASS。仅有既有 D-1037-01 失败（0.028s）。其断言及对应 AC 语义文档的 Git 内容与基面一致，历史 PLAN-1087 复现相同 `cache-roundtrip.document.case.ad must carry the executed status`。责任仍归 PLAN-1062，验收记录 294 PASS + 1 继承 FAIL，保留父/子退出 1/100。零本包新红。证据含实际输出准入 PASS 与 Rust 子进程清单核验。

AV 阶段从固定 AV 模块选择过滤 27 项非 AV 实际执行及 1 项 Windows 不跑的 Linux 测试。Linux/WSL 归专用 Linux 环境；不存在通过记录。具名过滤为 `assembly_944::rust_assembly_effect_and_restored_first_body_edit_use_real_fragment_producers`、`assembly_944::two_consumer_host_publishes_one_root_and_publication_failure_preserves_both_executables`、`assembly_944::provider_bytes_are_owner_and_consumer_scoped_and_restored_first_edit_executes_new_implementation`、`assembly_944::speculative_owner_never_evicts_current_facts_before_an_independent_first_edit`、`bstr_1064::bstr_1064_v01_large_64k_a2r`、`console_load_1041::vload_1041_in_axis_with_io_runs_on_a2py_both_targets`、`plan980_state_face::p980_native_driverless_compile_observes_states_and_texts`、`b1_slice_975::json975_char_to_str_b1_slice_oracles`、`module_sources_950::modules_950_qualified_consumer_restore_first_edit_and_next_read`、`module_sources_950::modules_950_user_shadow_consumer_restore_first_edit_and_next_read`、`plan982_c_multi::plan982_new_a2c_lowers_a_qualified_cross_module_call`、`plan982_c_multi::plan982_new_a2c_infers_the_cross_module_return_type`、`stdlib_contract_975::json975_packages_coverage_av_mn_a2r`、`stdlib_contract_975::json975_packages_coverage_a2c`、`stdlib_contract_975::json975_packages_coverage_a2py`、`stdlib_list_946::stdlib_list_946_native_no_call_inline`、`stdlib_list_946::stdlib_list_946_a2py_end_sum_find`、`v10_layers_972::v10_core_alloc_layers_mn`、`stdlib_list_946::stdlib_list_946_rust_end_sum_find`、`stdlib_list_946::stdlib_list_946_rust_end_pop`、`stdlib_list_946::stdlib_list_946_rust_end_owned_str`、`stdlib_list_946::stdlib_list_946_c_end_sum_find`、`stdlib_list_946::stdlib_list_946_c_end_pop`、`stdlib_list_946::stdlib_list_946_c_end_owned_str`、`stdlib_list_946::stdlib_list_946_native_end_sum_find`、`stdlib_list_946::stdlib_list_946_native_end_pop`、`stdlib_list_946::stdlib_list_946_native_end_owned_str`、`stdlib_assembly_1043::stdlib_assembly_1043_linux_target_ignores_missing_windows_sdk`。

准备 manifest SHA256 73721e60a11aa6d8784b47b0f99137f889819ff52c2fbf627c9d98c54adebc68，执行结果 SHA256 60e9726d1b1af6e5b6b101c4e91a588b053621b02a6f29f3d37624342d8526d1，选择 SHA256 4b85d0d09d484bbfc2c0062dfe3840c09708bc41eae8daccf1402847a661a47f，阶段筛选脚本 SHA256 68f607ed40fa474455e68158e443942edb6f2666d97b1aaf51c699ccec16ae80，配置 SHA256 69165441f188d5dde0a17a1291a0a5e2b0748f778104a7e2b027632a76611fbf，冻结源码身份 e03cb3ebe8f3ad15698e55b6256279b9392709ea6c037d15746f96fad350b3e0。本轮交付不是 %5 触发计划，批量收据 2026-11-20T00:00:00Z 距本轮不足 48h；全仓批量未到期。

- `prepared`：专属树清洁、master 基面 c941db401fffe08abae1e1f198238adec46d4f5c。冻结代码后的增量仅是三个已审 Spec 与两条设计导航。归档前守卫扫描通过。
- `landed`：`git merge --ff-only plan-1089-dev` 在 master 退出 0，a003e23213ed912a4dafcf95c4fb8c89d8f679ce。
- `canonical_verified`：主检出三个 Spec 与 Git 对象相等，两个索引均引用本计划归档路径；未写旧 specs.json。从主检出的 canonical suite 用已核对身份的冻结 debug 父端与 AV worker 重跑交错生产者，退出 0，PASS=1、SKIP=140、NOT_RUN=314，实际值 42，1.906s，report SHA256 6c8d19f7c5f011dff6d6954ce67bfe5135899d5aa3884778cccbbb2002a1de4e。本轮未发布程序；主检出 debug `auto-case.exe`（2026-11-20T04:43:43Z）、debug `av-worker.exe`（2026-11-20T04:44:11Z）与 release `av-worker.exe`（2026-11-19T03:10:13Z）早于落主，release `auto-case.exe` 缺席。
- [language-parity.md](../../specs/auto-av/language-parity.md)，SHA256 9cfd3b40b9d8a90f6359e80009680d51750c3c1e1b2210c3c5992d5a06f62212。
- [m2-control-effects.md](../../specs/ace-migration/m2-control-effects.md)，SHA256 b3bde1eb9eb8508ac229ee573fbef0530da97da90e087ceccbb7379922607459。
- [feature-organized-corpus.md](../../specs/auto-test-case/feature-organized-corpus.md)，SHA256 f6d4b11f00b233b6732569f5994600bb02f3a2f520647881f52babb36cd4ab81。

- `archived`：计划身份 r3、`completion_kind: delivered`。下面补记材料和工作树清理的实际收据。


### 2026-11-20 r3 · 清理收据

2026-11-20T23:40:58+08:00 确认交付树和归档提交已在 master、两工作树无修改、归属明确、无本计划执行进程。PowerShell `wt-guard.ps1` 对两工作树与唯一材料 scope 均输出 clean，未发现 reparse point。按原命令和 manifest 对账根/ACE Cargo target、nextest 预备与运行、AV worker 临时项目、父端探针输入/输出、日志和清单。工作树内的正常 target 随树清除，临时项目及证据位于唯一材料区。无需额外迁移项，无挂起持有。

| 已清除路径 | 文件数 | 文件字节总和 |
|---|---:|---:|
| D:/autostack/.wt/lang-1089/auto-lang | 85,736 | 18,236,777,437 |
| D:/autostack/.wt/lang-1089/auto-down | 1,318 | 27,021,855 |
| D:/autostack/plan-artifacts/plan-1089 | 9,121 | 524,564,404 |

两次 `git worktree remove` 与 `git branch -d plan-1089-dev` 退出 0。空组目录和材料 scope 已用 PowerShell 删除并复查不存在；两个仓库的 worktree 清单均没有该组。证据先折 Plan、归档提交 a503d7ab23208a0496c35637303dd8cc8c4e4d51，再执行删除。登记表按本计划删除收据刷新。首门遗留原红 D-1037-01 保持原 owner。

stage: merge | plan_id: PLAN-1089 | plan_revision: r3 | outcome: pass | reviewed_commit: e72afc9e43155c71a229462cb092a7919fe10c75 | delivery_commit: a003e23213ed912a4dafcf95c4fb8c89d8f679ce | archive_commit: a503d7ab23208a0496c35637303dd8cc8c4e4d51 | completion_kind: delivered | cleanup: removed | next: done

stage: goal | plan_id: PLAN-1089 | plan_revision: r3 | round: 3 | from: merge | to: done | status: archived | outcome: pass | commit: a003e23213ed912a4dafcf95c4fb8c89d8f679ce | next: done

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
