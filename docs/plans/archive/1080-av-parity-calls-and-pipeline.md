---
plan_id: PLAN-1080
status: archived
feature_name: AV调用绑定、隐式函数签名与管道补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 3
current_step: 6
total_steps: 8
depends_on: [PLAN-1078, PLAN-1079]
supersedes_spec_components: [docs/specs/ace-migration/m2-generics.md, docs/specs/ace-migration/m2-denominator.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1080] AV调用绑定、隐式函数签名与管道补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`calls/binding-pipeline.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-02**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补旧VM已支持的调用/管道拼写和推导，关闭命名参数绑定与求值次序差额。

本包交付`IF-AV-PARITY-CALL`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- L-24/25/47：位置/命名参数源序求值、重复/缺失/未知参数、函数前向引用、合法无类型形参/返回推导与尾表达式。
- 支持已存在的x |> .m(args)、多行连续管道及x |> .field；在统一CST投影层脱糖，LHS和每个实参恰求值一次；不新增尚未定的x |> f(x)自由函数注入。
- 默认参数保963已定边界，模块全局/依赖前参扩展先定规则，不用旧parser-only语法自动承诺任意用户varargs；List.new可变参数归集合兼容。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1080/auto-lang`，分支`plan-1080-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1080/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

D-963-C1明确ACE缺管道；legacy parser已有方法/字段管道。默认参数旧VM只parser-only，ACE已有默认填充，因此默认作用域扩展不是旧VM兼容必需项，须单列规则。

既有来源与边界：

- [pipe.case.ad](../../crates/auto-lang/test/vm/99_idiom2/m10_pipe/pipe.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [fn_named_args_reordered.case.ad](../../crates/auto-lang/test/vm/09_functions/020_fn_named_args_reordered/fn_named_args_reordered.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：09_functions/001..006、009、020、021；99_idiom2/m10_pipe；for_outer_var中调用/作用域部分。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [expressions.rs](../../crates/auto-lang-core/src/parser/expressions.rs) | `cb21e122e5ae312e916f45ebe9c26ea94b2e9dda4df797ac5c383b1ac717f858` |
| [auto-source.at](../../experimental/ace/grammars/auto-source.at) | `9e5248b2a6aeacd74d2ef61c77c41aae4181a47dd7e3f4d5fdd8c78e970ab25a` |
| [project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs) | `2223bfbbf79279bd4fc88596febc6dd455a477bd793832084ab8ffcba2ff38cc` |
| [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |

硬依赖：PLAN-1078, PLAN-1079；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [expressions.rs](../../crates/auto-lang-core/src/parser/expressions.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [auto-source.at](../../experimental/ace/grammars/auto-source.at)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/calls/binding-pipeline.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-generics.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-denominator.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1080-原文档清单)静态主承接 **10份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1080.pipe-method-and-field` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1080.named-binding` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=7; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1080.untyped-old-form` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=26; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1080.missing-call-argument` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.argument | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `CALL-SOURCE-ORDER` | AC-03/AC-04 | work / T-04..05 | 以模块trace记录named(b: mark(2),a:mark(1))先2后1且绑定a=1/b=2；依赖模块global批交付前可先用已有mut record trace，不能只比最终7。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `PIPE-NEIGHBOR` | AC-03/AC-04 | work / T-04..05 | \| >与\|>、is or臂、比较优先级、链首行错误/缺receiver，原合法位或不误吞。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: pipe-method-and-field
id: "av.parity.1080.pipe-method-and-field"
tags: ["av-language-parity", "plan-1080"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type P {
 v int
 fn add(n int) P { return P(.v + n) }
}
fn main() int {
 let p = P(19)
 let q = p |> .add(20)
            |> .add(3)
 return q |> .v
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
## Case: named-binding
id: "av.parity.1080.named-binding"
tags: ["av-language-parity", "plan-1080"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn sub(a int, b int) int { return a * 3 + b * 2 }
fn main() int { return sub(b: 2, a: 1) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 7

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
7
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: untyped-old-form
id: "av.parity.1080.untyped-old-form"
tags: ["av-language-parity", "plan-1080"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn add(a, b) { a + b }
fn mul(a, b) { a * b }
fn main() int { return add(mul(3, 4), mul(2, 7)) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 26

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
26
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: missing-call-argument
id: "av.parity.1080.missing-call-argument"
tags: ["av-language-parity", "plan-1080"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn f(a int, b int) int { return a + b }
fn main() int { return f(42) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "resolve"
code: "bind/source.argument"

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
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 6.5 T-01 决策记录（2026-11-19，worktree 基线 670c720a8）

### 有界全链探测结果（ace-core CLI `check-source --source-profile auto.source.language.av/v1` + av-worker `case-serve` 真跑）

| 探针 | 输入 | 结果 |
|---|---|---|
| p02 命名实参 | `sub(b: 2, a: 1)` | 编译 OK；av-worker 真跑 value=7、双流空、dispatch=0 |
| p04 缺实参 | `f(42)`（f 双参） | `bind/source.argument` "`f` expects 2 argument(s), found 1" |
| p06 重复命名 | `sub(a: 1, a: 2)` | `bind/source.argument` "argument `a` is passed more than once" + "parameter `b` … not bound" |
| p07 未知命名 | `sub(a: 1, c: 2)` | `bind/source.argument` "has no parameter named `c`" + not bound |
| p05 前向引用 | main 先调 later | 编译 OK |
| 源序求值 | `sub(b: mark(2), a: mark(1))`，mark=print+return | stdout `2\n1\n`（源序）、value=12（按名绑定 a=1/b=2）——CALL-SOURCE-ORDER 契约现实现已满足，T-04 只需固化永久断言 |
| p09 尾表达式体 | typed fn 无 return | 编译 OK |
| p10 具名构造 | `Pair { right: 2, left: 1 }` | 编译 OK |
| p18 位置构造+字段读 | `P(19)`/`p.v` | 编译 OK |
| p12 `var` 可变局部 | — | 编译 OK |
| p16 `print` | int/str | 编译 OK（需 console.stdout cap） |
| p19d mut record trace | `l.mut` 后缀实参 | 编译 OK |
| p01 管道 | `p |> .add(20)` | `text/source.syntax`——语法层无 `\|>`（D-963-C1 确认） |
| p03/p15 无类型函数 | `fn add(a, b) { a + b }` | `text/source.unsupported` 返回注解必填——无类型形参/返回推导缺失；且 `param` 文法 `ann-ty` 为必选步 |
| p13 无注解 main | `fn main() {…}` | 同上（scheduler 面外 ret-ty 投影必填，project.rs:1311） |
| p14 原型声明 | `fn test() int;` | `text/source.syntax` expected `{` found `;`——009 原型形态缺失 |
| p11/p17 record 方法 | `fn get() int { return .v }`、`static fn new`、`t.get()` | `bind/source.unsupported` "record methods are not part of this source profile"——resolve.rs:2296（is_coll 分支）单点拒绝；closures 降级 hir_adapter.rs:608 对 `DefKind::Method` 同拒 |

### 决策（D-01..D-08）

- **D-01 唯一作者**：`crates/auto-lang/test/cases/language/calls/binding-pipeline.case.ad`；suite 引用增入 `language.suite.ad`；legacy-map 增本包行（TSV 12 列，SHA16=整份 legacy .case.ad 文件字节 SHA-256 前 16 hex——已按 10/10 复核与 1078 约定一致）。
- **D-02 record 方法面**：Auto profile 放行方法声明/实例调用/静态调用/receiver `.field`，复用 record 域既有同构机制（方法=普通 MIR 函数、receiver=HIR params[0]、静态无 receiver、`m.<Rec>.<name>` 名——`lower_checked_data` 已证模式，PLAN-881）；冻结窄 profile（Collections/Closures/Numeric/Aggregates/Scheduler）保持现拒不变。触面：resolve.rs is_coll 方法拒（仅 Auto 豁免）、project.rs 方法/`.field`-receiver/静态调用投影、hir_adapter.rs collection-family 降级 Method 臂。
- **D-03 管道**：postfix-step 形（auto 专属步骤）：`pipegt() dot(same-line:true) ident(member)` + pratt call 后缀接 args；`pipegt` 不带 same-line ⇒ 行首 `|>` 跨行续行（legacy 同形）。投影层脱糖为与 `.member`/调用后缀同一原子（AST/HIR 零新节点，LHS/实参恰一次求值天然成立）。**边界（已批审计规范化，L-25 双账）**：管道左操作数=紧邻后缀链；二元运算结果接管须括号——`x + y |> .m()` 在投影层具名拒（`text/source.syntax`，二元算子直接子位含管道步），防静默语义分叉；`x |> .m() + 1` 解析为 `(x.m()) + 1` 与 legacy 一致。不新增 `x |> f(x)`。`| >`（带空白）保持按位或/比较 tokenize 不变；is or 臂在语句模式文法不受表达式 postfix 影响。
- **D-04 无类型函数**：`param.ann-ty` 步对 auto 面可选（步骤级门控，窄面不动）；ret-ty 投影必填规则对 auto 放行为"推导"：体内推导（尾表达式/return 型）+ 调用点一致性检查；全部调用点与体内推导矛盾 ⇒ 新码 `bind/source.inference`（不静默默认、不 any-Err）。无注解 `fn main()` ⇒ Unit 返回（worker 已有 unit 通道）。混合类型调用点（如 add(1,2) 与 add("a","b") 并存）拒绝。
- **D-05 原型声明**：fn-tail 增 auto 专属 `;` 尾形（签名先行）；resolve Pass-1 原型与实体签名须一致，不一致 `source.duplicate`；原型无实体 ⇒ `bind/source.unresolved`。
- **D-06 诊断冻结**：缺失/重复/未知/未绑定=现存 `bind/source.argument`（探针实证四形）；无类型推导冲突=`bind/source.inference`（新）；管道错形=`text/source.syntax`（"`|>`"邻接错）；链首 `|>`（无 receiver）=语法拒。
- **D-07 默认参数（T-04 细化）**：保 963 声明位默认值边界（`param-default` 步已存在）；模块全局默认/依赖前参不在本包扩权；List.new 可变参数归集合批（AVP-04）。用户 varargs 不承诺。
- **D-08 compound-ops 归属修正**：legacy-map 中 6 行 11_compound_ops（1079 写入）标注 owner "PLAN-1080 (calls and pipeline statement surface)"——本包 r3 合同**不含** `+=` 语句面；T-05 迁移时将该 6 行 owner 修正为"statement-surface batch（未排号）"，保 research-hold 与原 oracle 不丢（分母不减），Plan 内具名记录。

### 指纹漂移（1078/1079 交付所致，预期内）

expressions.rs `cb21e122…` 未变（旧 parser 冻结）；auto-source.at `9e5248b2→6ee7996e`、project.rs `2223bfbb→e0b64164`、resolve.rs `54978de5→449cfe3b`——均因 1078/1079 落地；T-02 起以 worktree 当前版为准。10 份旧 case SHA16 逐份复核全匹配，旧义务零漂移。

## 6.6 T-06 验证记录（冻结提交 34d97223c，2026-11-19）

冻结门（`cargo t ace-source ace-syntax ace-semantics ace-av case-av-language-parity`，worktree 单实例，含增量准备总 wall 636.7s）：

| 组 | 结果 | 执行数 | 备注 |
|---|---|---|---|
| ace-source | PASS | 926 | 含 plan981 grammar 13 邻居冻结（auto 门控零外溢实证）+ 927 七 profile 差分语料 |
| ace-syntax | PASS | 159 | |
| ace-semantics | FAIL | 295 | 294 绿 + **1 原红**：`sema_boundary_audit::sema_boundary_case_files_executed_status_honest`（ac-semantics cache-roundtrip 文档缺 executed 状态）——**干净基线复现同红**（stash 后重跑实证），非本包改动，归 ac-semantics 语料 owner |
| ace-av | FAIL | 1450 | 1448 绿 + **2 原红**：`host_console::av_console_entry_selection_cannot_bypass_main_policy` / `av_console_owned_front_door_enforces_console_entry_only`（期望 982 前 lower.entry 政策，982 已改具名入口+MIR 形状合同）——**干净基线复现同红**，归 console 面 owner |
| case-av-language-parity | PASS | 30 例/45 检查 | 1078/1079 全例无回归 + 本包 22 例全绿（复审 F-01 更正：原记 21）；非 AV 检查 #ignore/NOT_RUN（"not selected (check=av)"），avm dispatch=0 |

新增用例计时：conformance `plan1080_calls_binding` 15 测 0.07s（含编译执行，全部 <<10s/例）；语料单例经 worker 全链 30-60ms/case。故意红元控制三红一绿（§8 T-05 行）。健康：基线 4 条预存告警数不变，零新增告警/调试输出。

**IF-AV-PARITY-CALL**（交付接口）：`auto.source.language.av/v1` 面新增——① record 方法（实例/静态/receiver `.field`，MIR `m.<Rec>.<name>` 单元名）；② 无类型签名补全（调用点单态+体内推导；`bind/source.inference` 具名拒）；③ 原型 `fn f(..) R;`（`source.unresolved`）；④ 管道 `x |> .m(args)`/`x |> .f`/行首续行（紧结合，D-03）；⑤ 零参 Unit 入口（`fn main()`）；⑥ 描述符身份 `auto.hir.auto-language.draft/1/auto-language-draft`（aggregates 词汇+method+unit）。下游消费者：AVP-03（闭包/函数值——lambda 注解面未动）、AVP-04（List.new varargs 归集合）、statement-surface 批（compound-ops `+=`，legacy-map 具名）。

## 7. 验收标准

- [x] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [x] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [x] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [x] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [x] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [x] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [x] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [x] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [ ] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | L-24/25/47：位置/命名参数源序求值、重复/缺失/未知参数、函数前向引用、合法无类型形参/返回推导与尾表达式。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 支持已存在的x \|> .m(args)、多行连续管道及x \|> .field；在统一CST投影层脱糖，LHS和每个实参恰求值一次；不新增尚未定的x \|> f(x)自由函数注入。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 默认参数保963已定边界，模块全局/依赖前参扩展先定规则，不用旧parser-only语法自动承诺任意用户varargs；List.new可变参数归集合兼容。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-CALL。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态：

- [x] **T-01** work：2026-11-19 已完成（worktree `D:/autostack/.wt/lang-1080/auto-lang`，基线 670c720a8）。决策记录见 §6.5。
- [x] **T-02** work：2026-11-19。实现：`param`/`fn-item` auto 门控文法臂（无类型形参/原型 `;`）；AST `TypeRef::Inferred`+`FnItem.proto`；投影（project.rs 可选注解/ret 推导标记/run_fn_proto）；**补全遍** `source/infer.rs`（parse→resolve 之间，仅 auto 面：调用点驱动单态定点、尾/return 推导、void main=Unit、原型合一；冲突/无约束=具名 `bind/source.inference`，无静默默认）。命名实参四诊断（缺失/重复/未知/未绑定）实证为现存 `source.argument` 家族；前向引用+CALL-SOURCE-ORDER（stdout 2\\n1\\n+值12）实证已满足。
- [x] **T-03** work：2026-11-19。`pipegt` token+auto 门控 postfix-step（行首续行=pipegt 无 same-line）；投影与点后缀同构脱糖（AST/HIR 零新节点，LHS/实参恰一次）；**紧结合边界**（§6.5 D-03 修订：管道先作用于紧邻链，legacy 低结合形须显式括号——括号为透明 trivia，span 无法区分，故不设二元操作数拒而是固化正例 603/8）；record 方法面（D-02：auto-language 描述符+resolve 放行+`lower_checked_auto_language`，冻结窄面字节不变）；MIR 入口零参 Unit 放行（ABC 布局不动）。
- [x] **T-04** work：2026-11-19。默认参数保 963 边界实证（`n int = 42` 调用点填充 42+7=49 全链绿）；不扩 varargs；extra oracle：CALL-SOURCE-ORDER（stdout 溯源）/PIPE-NEIGHBOR（紧结合 603、`| >` 分词 3、is 臂不受扰）入 canonical；V-FC AV 隔离：非 AV 检查 `#ignore`+NOT_RUN（报告实证 "not selected (check=av)"，avm dispatch=0）。
- [x] **T-05** work：2026-11-19。canonical `language/calls/binding-pipeline.case.ad`（22 例；复审 F-01 更正计数——原记 21）+ suite 引用 + legacy-map 10 行（SHA16 全对账，double→twice 具名审计改名）+ compound-ops 6 行 owner 修正（D-08）；**故意红元控制**：m10 stdout 18→19 红、named-binding exit 7→8 红、missing-arg 码替换红、复原绿（三红一绿）；conformance `plan1080_calls_binding.rs` 15 测全绿。
- [x] **T-06** work：2026-11-19 冻结提交+门（见 §9 收据与 §6.6 计量）。
- [x] **T-07** review：2026-11-19 final pass @ 34d97223c（§9 收据；F-01 簿记更正）。
- [x] **T-08** merge：2026-11-19 落主 74a0ff965 + 归档 + 清理（§9 merge 收据）。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- `stage: goal | plan_id: PLAN-1080 | plan_revision: 3 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1080/auto-lang | commit: 34d97223c | next: review`
- `stage: goal | plan_id: PLAN-1080 | plan_revision: 3 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1080/auto-lang | commit: 34d97223c | next: merge`
- 2026-11-19 merge `cleaned` 完成记录：wt-guard 复核 clean → worktree `D:/autostack/.wt/lang-1080/auto-lang` 与分支 `plan-1080-dev`（74a0ff965）移除、组目录删除；`plan-artifacts/plan-1080/`（31KB 探针暂存）整体删除（记录已折本 Plan），登记表刷新、无具名持有。落主检查：master tip=74a0ff965 线性历史（20231c041→d3020c6ef→74a0ff965→aa752d2e4）；主检出 target 缓存相对新 HEAD 为陈旧（下次使用增量重建——部署观察，非本包授权重建）。批量回归到期（1080%5=0 且上次收据 1064/2026-11-16 超 48h）→ 移交 auto-plan-regress（主检出单实例）。
- `stage: goal | plan_id: PLAN-1080 | plan_revision: 3 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 74a0ff965 | next: done`
- 2026-11-19 merge 收据：`stage: merge | plan_id: PLAN-1080 | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1080/auto-lang | branch: plan-1080-dev | prepared: reviewed 34d97223c + docs-only 后代 e34f17412（SD-01..03/SD-FC，四 spec 目标） | rebase: master 前移 670c720a8→20231c041（他人 docs 提交，零文件重叠）；旧→新 34d97223c→d3020c6ef、e34f17412→74a0ff965，range-diff 两段全等（安全改写证明）；rebased 树 case-av-language-parity 复跑 PASS（证据重绑 delivery） | landed: master tip=74a0ff965 ff-only 无合并提交 | canonical_verified: language-parity a078207898342fd6 / m2-generics b5867f4252baf7b8 / m2-denominator 2f2bdc63b49f0f66 / feature-organized-corpus 943479857f4dfa26（master:路径 SHA256 前16，PLAN-1080 增量均在） | archived: 见下行 | cleaned: 见下行 | next: done`。
- 2026-11-19 final review 收据：`stage: review | review_scope: final | reviewer: 本会话独立上下文（限制：实施同会话——结论自工件重建：冻结提交 diff/组结果 JSON/语料报告逐项重核，不采信执行叙述） | plan_id: PLAN-1080 | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1080/auto-lang | branch: plan-1080-dev | reviewed_commit: 34d97223c | base_commit: 670c720a8 | dependency_revisions: PLAN-1078/1079 archived | spec_inputs: docs/specs/auto-av/language-parity.md（1078 建）+ m2-generics/m2-denominator/feature-organized-corpus（四目标均存在，SD-01..03/SD-FC 由 merge 沉淀） | acceptance_results: AC-AV-ONLY/AC-FC/AC-01..06=pass（45 语料检查含 1078/1079 零回归+22 新例；诊断四形+inference 两形+unresolved 实证；红控三红一绿；conformance 15/15；无 case 名特判 grep=0；冻结 schema/vm 零触碰 git 证；新例全部 <<10s） | pending_closeout_gates: AC-08 merge 落主+Spec 沉淀+归档+guard 清理（统一冻结门已跑：§6.6） | findings: F-01（簿记）计划/收据例数 21→22 已更正——不涉代码基线 | evidence: §6.5/§6.6/本行（run-5xs8c1t5 结果 JSON 路径已折计划） | next: merge（T-08）`。状态置 `reviewed`。
- 2026-11-19 work 完成收据：`stage: work | plan_id: PLAN-1080 | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1080/auto-lang | branch: plan-1080-dev | base_commit: 670c720a8 | code_commit: 34d97223c | dependency_revisions: PLAN-1078/1079 archived（master 670c720a8 合入态） | task_ids: T-01..T-06 | evidence: §6.5 决策记录+§6.6 冻结门（926+159+295+1450+30 执行；3 条原红干净基线复现具名分列；45 语料检查全绿含 1078/1079 零回归；故意红元控制三红一绿；15 conformance 全绿）| blockers: 无 | next_responsible_role: review（T-07）`。状态置 `execution_done`。目标授权链：`/goal 完成计划1080`（2026-11-19）授权 work→review→bounded repair→merge 全链。
- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
