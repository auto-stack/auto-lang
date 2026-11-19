---
plan_id: PLAN-1082
status: archived
feature_name: AV数组、集合语法与嵌套place补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1081]
supersedes_spec_components: [docs/specs/ace-migration/m2-aggregates.md, docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1082] AV数组、集合语法与嵌套place补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`collections/list-places.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-04**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补数组/List旧合法构造与索引、深层写回和高阶调用的语言接面，不另造容器库。

本包交付`IF-AV-PARITY-COLLECTION`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- L-16/19/48：数组字面量、空数组推导、裸索引读/写、链式field/index place与复合赋值，所有base/index/RHS源序恰一次。
- List.new(a,b,c)按旧可达构造提供受检兼容糖或具名已批准差异，不新增任意用户函数varargs语法；map/filter/reduce/find/any/all接972真实公共实现。
- 整元素view/显式move/复制规则与资源批协调；负索引沿现行SEM-02批准差异分账，不能用get替换原源后声称裸索引兼容。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1082/auto-lang`，分支`plan-1082-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1082/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

962已有聚合和nested place，但裸xs[i]读仍是D-962-C1，旧List.new(a,b,c)与新List<T>.new()+push不等价；972已有map/filter，964旧手写loop不能代原HOF调用证据。

既有来源与边界：

- [list_chain_pipeline.case.ad](../../crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [v01_nested_clone.at](../../experimental/ace/crates/core/tests/fixtures/aggregates_962/v01_nested_clone.at)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：06_arrays；13_collections的List语言/操作面；15_nested_mutation；29_list_shims；09_functions/010..015；for_outer_var。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [auto-source.at](../../experimental/ace/grammars/auto-source.at) | `9e5248b2a6aeacd74d2ef61c77c41aae4181a47dd7e3f4d5fdd8c78e970ab25a` |
| [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [hir_adapter.rs](../../experimental/ace/crates/lowering/src/hir_adapter.rs) | `7e629f79e171b9fa0f7a284eae1e23bca972b3c6d2dabd07449c0135eb9c4d85` |
| [av.rs](../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1081；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [auto-source.at](../../experimental/ace/grammars/auto-source.at)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [hir_adapter.rs](../../experimental/ace/crates/lowering/src/hir_adapter.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [list.at](../../stdlib/ace/asm/list.at)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/collections/list-places.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-aggregates.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-stdlib-core.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1082-原文档清单)静态主承接 **62份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1082.bracket-read-write` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1082.legacy-list-new` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1082.hof-real-call` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1082.nested-place` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `PLACE-SIDE-EFFECT` | AC-03/AC-04 | work / T-04..05 | base()/index()/rhs()分别记trace，写回不得丢兄弟字段；数组对象混合嵌套、remove时借用与迭代中结构修改反例。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `BOUNDS` | AC-03/AC-04 | work / T-04..05 | 0/len-1/len、空表、负索引现行规则与超界类型化错误；错误由错误批接入catch，不能改成默认0。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: bracket-read-write
id: "av.parity.1082.bracket-read-write"
tags: ["av-language-parity", "plan-1082"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 var xs = [19, 20]
 xs[0] += 1
 xs[1] = xs[1] + 2
 return xs[0] + xs[1]
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

### VM result
#ignore
check-id: "legacy-reference"
final-newline: false

```text
42
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: legacy-list-new
id: "av.parity.1082.legacy-list-new"
tags: ["av-language-parity", "plan-1082"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int { let xs = List.new(19, 23); return xs[0] + xs[1] }
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
## Case: hof-real-call
id: "av.parity.1082.hof-real-call"
tags: ["av-language-parity", "plan-1082"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let xs = [1, 2, 3, 4]
 let ys = xs.filter(x => x % 2 == 0).map(x => x * 3)
 return ys.reduce(24, (acc, x) => acc + x)
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
## Case: nested-place
id: "av.parity.1082.nested-place"
tags: ["av-language-parity", "plan-1082"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type Row { xs List<int> }
fn main() int {
 var r = Row([19, 20])
 r.xs[1] += 3
 return r.xs[0] + r.xs[1]
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

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-semantics ace-av ace-lowering ace-stdlib
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [ ] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [ ] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [ ] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [ ] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [ ] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [ ] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [ ] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [ ] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [ ] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | L-16/19/48：数组字面量、空数组推导、裸索引读/写、链式field/index place与复合赋值，所有base/index/RHS源序恰一次。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | List.new(a,b,c)按旧可达构造提供受检兼容糖或具名已批准差异，不新增任意用户函数varargs语法；map/filter/reduce/find/any/all接972真实公共实现。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 整元素view/显式move/复制规则与资源批协调；负索引沿现行SEM-02批准差异分账，不能用get替换原源后声称裸索引兼容。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-COLLECTION。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：已完成 2026-11-20。冻结记录+决策D-1082-01..09见§9；探针O1–O29观察矩阵真实运行（`plan1082_collections_places.rs`）。canonical位置冻结：`crates/auto-lang/test/cases/language/collections/list-places.case.ad`（新collections目录，1078 suite只增引用）；判等条件=FC合同（同一Auto源/工程闭包，AV检查+非AV `#ignore`/NOT_RUN分列）。
- [x] **T-02** work：已完成 2026-11-20，提交 `eb7d7796c`。place-assign-stmt（auto面独占，置于path/index-assign后仅承新形）；复合赋值`+ - * /`；数组字面量rvalue放宽（auto面，sink提升）；空`[]`语法收录+具名拒`source.array-lit-empty`；嵌套字面量Move包裹。962冻结面（aggregates init-only）字节保持（`aggregates_t01_probe::array_literal_negative_controls`复绿）。回归：ace-source/syntax/semantics/comptime/lowering PASS；ace-av 1468例仅2项**基线继承红**（host_console entry policy两项，经base复测确认先于本包存在，证据`plan-artifacts/plan-1082/logs/base-host-console.txt`）+2项当时待实现T-03探针。
- [x] **T-03** work：已完成 2026-11-20。`List.new(e1..en)`受检兼容糖（elem取e1+同质校验+同push admission；零参具名拒`source.list-new-empty`；影子局部/record/variant优先）；HOF六件套`map/filter/reduce/find/any/all`方法面→冻结`stdlib/ace/asm/list.at`（include_str身份）经`compile_language_source`按需文本graft（`__std_list_`mangle、`pub`剥离、方法形不动、字节稳定单测）+`CallFn`直调——同源同编译器，无第二算法；非int元素具名拒`source.hof-elem-domain`。
- [x] **T-05** work：已完成 2026-11-20，提交 `5278a3399`。canonical唯一作者 `language/collections/list-places.case.ad`（25 Case：§6四件+PLACE-SIDE-EFFECT+BOUNDS五面+nested/record place+List ops族+tuple解构+复合语句族+HOF族+int-shim+三个具名反例；25条av检查全PASS，legacy VM检查34条带`#ignore`在`--check av`视图NOT_RUN不阻AV门）；suite只增canonical引用；legacy-map.tsv新增62行（33 mapped带审计normalization/29 research-hold具名owner：AVP-06×8、AVP-05×1、AVP-08×1、stdlib容器×19）+跨批关闭statement-surface 6行（mapped复合语句族）+21_conv 2行重定向；`fixed-tests.py`修复per-group `check`选择器经prepared清单丢失的infra缺陷（run-prepared丢`--check av`致全量套件对滤后期望判定）；故意红元控制：单oracle变异42→43使scratch套件FAIL=1/exit1（`plan-artifacts/plan-1082/meta-red*`）。
- [x] **T-06** work：已完成 2026-11-20，冻结提交 `5278a3399`（树clean）。§6固定组并集实测（`plan-artifacts/plan-1082/logs/t06-frozen-union.json`）：`case-av-language-parity` PASS（67 av检查）、`ace-source` PASS（927）、`ace-lowering` PASS（9）、`ace-stdlib` PASS（2）、`ace-semantics` FAIL=**1项继承红**（`semantics-store::sema_boundary_case_files_executed_status_honest`：PLAN-1062文件`cache-roundtrip.document.case.ad`缺executed标记，base复测同红，证据`logs/base-sema-audit.txt`）、`ace-av` FAIL=**2项继承红**（host_console entry-policy两项，base复测同红，证据`logs/base-host-console.txt`）。继承红具名分账不判本包、不充PASS；本包触面0红。

  **IF-AV-PARITY-COLLECTION 收据（T-06交付接口）**：
  - 语言面：数组/List字面量任意rvalue位（auto面）、`List.new(e1..en)`受检varargs糖（零参具名拒）、复合赋值`+= -= *= /=`（裸名/字段链/索引链/mixed链，base/index/RHS源序恰一次）、place链写回（field×index、index×index，句柄写穿透）、负索引/OOB/空容器统一`av.trap.list-index`（SEM-02）。
  - HOF方法面：`map/filter/reduce/find/any/all/pop` on List<int> → 冻结`stdlib/ace/asm/list.at`同源直调（`__std_list_` graft，字节稳定单测），非int元素具名拒`source.hof-elem-domain`。
  - canonical语料：`crates/auto-lang/test/cases/language/collections/list-places.case.ad` + suite引用 + legacy-map 62行/跨批8行；AV视图固定组`case-av-language-parity`（67检查绿/34 NOT_RUN隔离）。
  - 诊断面：`source.array-lit-empty`、`source.list-new-empty`、`source.place-elem-field`、`source.compound-elem-type`、`source.hof-elem-domain`、既有`source.iterate-mutate`/`source.move-required`边界保持。
  - Rust语义pin：`experimental/ace/crates/av/tests/cases/plan1082_collections_places.rs`（18例绿：O-观察矩阵+§6+extra+negative）。

- [ ] **T-03** work：见上表唯一任务合同。
- [ ] **T-04** work：见上表唯一任务合同。
- [ ] **T-05** work：见上表唯一任务合同。
- [ ] **T-06** work：见上表唯一任务合同。
- [x] **T-07** review：已完成 2026-11-20，final pass（见§9复审记录）。
- [x] **T-08** merge：已完成 2026-11-20（delivery 6bda0f03b，见§9合并收据）。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- 2026-11-20 **work完成交接（T-01..T-06全绿）**：`stage: work | plan_id: PLAN-1082 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1082/auto-lang | branch: plan-1082-dev | base_commit: da458f671 | code_commit: 5278a3399（前序 eb7d7796c T-02、2da4e761b T-03/T-04）| dependency_revisions: PLAN-1078/1079/1080/1081 archived | task_ids: T-01..T-06 | evidence: §6固定组并集 t06-frozen-union.json（corpus 67av绿/34隔离；ace-source 927绿；ace-lowering 9绿；ace-stdlib 2绿；继承红3项具名分账：host_console×2 + sema-boundary-audit×1，base复测同红）；18例Rust语义pin绿；故意红元控制绿 | blockers: 无 | next: review (T-07)`
  Work-owned AC结果：AC-01 pass（§6四件42/42/42/42 + 全oracle）、AC-02 pass（具名反例+SEM-02 trap分账）、AC-03 pass（PLACE-SIDE-EFFECT/BOUNDS真实断言非exit0推断）、AC-04 pass（62行映射33mapped/29hold守恒）、AC-05 pass（共忍受检前门+AV dispatch，无case特判/第二parser）、AC-06 pass（固定组并集+计时记录；继承红具名非本包）、AC-AV-ONLY pass（非AV #ignore NOT_RUN零dispatch不阻AV，#ignore不充PASS）、AC-FC pass（唯一作者+迁移表+suite引用+故意红）。开放项：无未决in-contract发现。

- 2026-11-20 **goal完成收据**：`stage: goal | plan_id: PLAN-1082 | plan_revision: r3 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: （已移除，guard clean）| commit: 6bda0f03b（delivery，master tip）/ 409f2ce2b（归档簿记）| next: done`
  全链核验：delivery commit 在 master（ff-only，无 merge commit，range-diff 等价证明）；四 canonical Spec 路径随 delivery 落主；计划移入 `docs/plans/archive/` 且 status archived、docs 已提交（409f2ce2b）；worktree/branch/组目录经 clean wt-guard 移除；plan-artifacts scope 整删、登记表刷新（无具名 hold）；主检出净（仅其它会话在途 docs，未触）；批量回归不到期（receipt 覆盖 1098/今日）。继承红 3 项具名分账在案（非本包触面）。材料/计时：本包新增用例 25 corpus（最慢 834ms）+ 18 Rust pin（0.24s）全≤10s；§6 并集 ace-source 927/ace-lowering 9/ace-stdlib 2/corpus 67 绿。

- 2026-11-20 **T-08合并收尾**（`stage: merge | plan_id: PLAN-1082 | plan_revision: r3 | outcome: pass`）：
  - `prepared`：reviewed 5278a3399 + docs-only Spec 沉淀（SD-01 `docs/specs/auto-av/language-parity.md`、SD-02 `m2-aggregates.md`、SD-03 `m2-stdlib-core.md`、SD-FC `auto-test-case/feature-organized-corpus.md`，+43 行）= 87346a093。
  - `landed`：master 因并行会话 PLAN-1107 前移（1afa94897）→ rebase 零冲突，`git range-diff` 四提交全 `=`（等价证明）；旧→新映射 eb7d7796c→8c1045197、2da4e761b→396ad6038、5278a3399→091ea923c、87346a093→**6bda0f03b（delivery commit）**；rebase 后语料组在 6bda0f03b 重验 PASS（`t08-rebased-corpus.json`）；main `git merge --ff-only plan-1082-dev` ✓ tip==delivery，无 merge commit。
  - `canonical_verified`：四 Spec 路径随 delivery 落 main（docs-only 后代，实现/依赖未变）；KNOWN-DEBT D-962-C1 具名关闭（残留裸 `hold` 拼写改记 C1a，owner 不变）。
  - 继承红分账（非本包触面，base 复测同红证据 `logs/base-host-console.txt`、`logs/base-sema-audit.txt`）：host_console entry-policy ×2（owner：host-console 面）、sema_boundary_case_files_executed_status_honest ×1（PLAN-1062 文件缺 executed 标记，owner：1062 收尾）。
  - `archived`/`cleaned`：见下（本收据随归档提交落主）。

- 2026-11-20 **T-07最终复审（final）**：`stage: review | review_scope: final | reviewer_identity: 同会话独立上下文（限制已声明：结论自工件重建——冻结提交5278a3399上独立重跑pin17例+graft稳定1例、TSV守恒独立脚本核对62/62、语料组复用t06-frozen-union（同冻结基线）、diff特殊化扫描零命中） | plan_id: PLAN-1082 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1082/auto-lang | branch: plan-1082-dev | reviewed_commit: 5278a3399 | base_commit: da458f671 | dependency_revisions: PLAN-1078/1079/1080/1081 archived | spec_inputs: docs/specs/auto-av/language-parity.md, ace-migration/m2-aggregates.md, m2-stdlib-core.md, auto-test-case/feature-organized-corpus.md（SD-01..04+SD-FC提案审读有效，落盘归merge）| acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass、AC-04 pass、AC-05 pass、AC-06 pass、AC-AV-ONLY pass、AC-FC pass（AC-07本记录、AC-08 merge-owned）| pending_closeout_gates: merge落主后冻结并集重验（同一基线t06证据可复用）；KNOWN-DEBT D-962-C1读面已交的具名更新；1081名下09_functions.010/013 stdlib-HOF行可由其owner按本包HOF面关闭（观察项非义务）| findings: F-R1过程事故已闭环（共享stash栈误pop旧PLAN-1066条目入本树→UU冲突，树已恢复clean、原stash条目完好零丢失；教训：stash栈全仓共享，clean树不得盲pop）；F-R2文档微瑕（收据24→25 Case已改正）；3项继承红具名分账（host_console×2、sema-boundary-audit×1，base复测同红证据在案，非本包触面）| evidence: 本记录内嵌命令/结果；logs/t06-frozen-union.json（67av绿/34隔离；ace-source 927绿；lowering 9；stdlib 2；继承红3具名）；新用例计时：语料av最慢834ms（25例均值~800ms）全≤10s；Rust pin 17例0.24s；无新增未处理告警 | next: merge (T-08)`
  AC→task→工件→证据映射：AC-01→T-02/03/05→§6四case(42)+pin→corpus run+pin重跑；AC-02→T-02/04→5具名反例+4 trap面→pin/corpus；AC-03→T-04→place-side-effect/bounds→pin+corpus真实断言；AC-04→T-01/05→legacy-map 62行(33mapped/29hold,15 canonical)→独立守恒脚本；AC-05→T-02/03→diff扫描零case-id特判、单一parser/graft按源内容→diff+graft单测；AC-06→T-06→t06-frozen-union计时在案；AC-AV-ONLY→T-03..06→34条#ignore NOT_RUN零dispatch→corpus run;AC-FC→T-01/02/05→唯一作者+suite只增+故意红FAIL=1→meta-red证据。

- 2026-11-20 goal轮2：`stage: goal | plan_id: PLAN-1082 | plan_revision: r3 | round: 2 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1082/auto-lang | commit: 5278a3399 | next: review`

- 2026-11-20 goal授权：`/goal 完成计划1082`（auto-plan-goal全链授权：work→review→有限repair→merge；目标/验收/范围/预算不变）。worktree `D:/autostack/.wt/lang-1082/auto-lang`（branch `plan-1082-dev`，base `da458f671`）已建；材料scope `D:/autostack/plan-artifacts/plan-1082/` 已登记（`.retention/policy.json`）。主检出在途docs改动（00-intro/58-/ace-route-plan-map/1046/1107草案）属其它会话，不触不并。
  `stage: goal | plan_id: PLAN-1082 | plan_revision: r3 | round: 1 | from: start | to: work | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1082/auto-lang | commit: da458f671 | next: work`

- 2026-11-20 T-01冻结（work）：探针文件 `experimental/ace/crates/av/tests/cases/plan1082_collections_places.rs`（挂接conformance.rs），观察矩阵O1–O29真实跑通于普通AV链（`compile_language_source`→HIR→MIR verify→ABC admit→AV execute），关键观测：
  - **已绿**：O1/O2数组字面量(let/var init)、O3裸索引读、O4裸索引写、O10–O13越界/负下标=统一`Trap ListIndex`（SEM-02现行冻结正确）、O15 `List<int>.new()`、O17 `arr[0].x`读、O25/O26 record List字段经`.push`/`.set`写穿透（句柄共享语义成立）、O27/O28 tuple字面量+解构。
  - **缺口**：O5 `+=`复合赋值（text拒）、O6链式field+index place（text拒）、O7裸`List.new(varargs)`（bind `source.undefined`）、O8 HOF方法面（bind：现合法表仅new/push/insert/get/set/remove/len/clear）、O9空`[]`（text拒）、O14/O29 tuple `.N`位置读（text拒）、O16/O21/O22/O24 数组字面量仅init位（`source.array-lit-position`）、O18/O19/O20 place链写（text拒）、O23 record方法/`mut fn`/`self`（`source.unsupported`，非本包面）。

  **T-01决策（本Plan内冻结）**：
  - **D-1082-01**：裸`xs[i]`读写在auto面已绿（962 L-48 + 964 postfix所交）；本Plan r3文字"裸读仍是D-962-C1"过时——KNOWN-DEBT D-962-C1待merge时具名更新（读面已交，非`.get`独占）。
  - **D-1082-02**：复合赋值语句`=扩展 op∈{+,-,*,/}`（与legacy 11_compound_ops族同集），place目标（裸名/字段链/索引链），base/index/RHS源序各恰一次；一并关闭1079遗留的6条statement-surface research-hold行（跨批行更正具名记录）。
  - **D-1082-03**：新语句规则`place-assign-stmt`（[aggregates,auto,scheduler]）`ident (.f | [e])* (op)= e`，置于path-assign/index-assign之后仅承旧规则不匹配之形（混合链/多级索引/复合op）；`arr[0].x=v`走既有MIR `LIST_FIELD_SET`，`m[0][1]=v`走`ListSet`(base=ListGet链)，record字段链写穿透按O26句柄语义。closures面`cell[k]=v`旧规则字节不动。
  - **D-1082-04**：数组字面量放宽到一般rvalue位：resolve语句级临时提升（同一ListNew+push脱糖，init位快速路径保留）；空`[]`维持具名拒`source.array-lit-empty`（不引入新推导机制；空表=显式`List<T>.new()`），legacy `[]`/零参`List.new()`规范化为`List<int>.new()`（oracle只观察len/空，不变）。
  - **D-1082-05**：`List.new(e1..en)`受检兼容糖：elem VT取自e1（与数组字面量同同质校验/同push admission），按位脱糖ListNew+push；零参`List.new()`具名拒（elem不可推导，用`List<T>.new()`）；不新增任意用户varargs语法。
  - **D-1082-06**：HOF方法面`xs.(map|filter|reduce|find|any|all)(...)`仅List<int>接收者，脱糖为对冻结`stdlib/ace/asm/list.at`同名pub fn的直接调用——按需把该冻结源（include_str身份）的函数编入同一模块函数表（mangled `__std_list_*`，仅在源码用到时 graft，零漂移）；同源同编译器=972真实公共实现，不写第二算法。非int元素/`for_each`（972集无此函数）具名拒/留owner。
  - **D-1082-07**：tuple位置读`.N`不加语法（FIRST集机制风险、无§6消费者）；legacy 06_arrays.010规范化为解构读（oracle语义不变）。
  - **D-1082-08**：record方法/`self`（15_nested_mutation.015）→ research-hold归records/方法owner（AVP-05线）；O23实证`source.unsupported`。
  - **D-1082-09**：hashmap/hashset/StrBldr载体（13_collections.001–019、21_conv.003/004）与对象字面量深链（15_nested_mutation.001/003/006/007/008/011/012/013/014之对象部分）→ research-hold分别归stdlib容器owner与AVP-06动态对象面（L-20）；本包canonical面=typed record/index链。
  - 负索引（06_arrays.002）：SEM-02批准差异分账——canonical按现行trap行为断言，legacy行记差异不冒绿。
  `stage: work | plan_id: PLAN-1082 | plan_revision: r3 | outcome: in-progress(T-01 done) | worktree: D:/autostack/.wt/lang-1082/auto-lang | branch: plan-1082-dev | base: da458f671 | evidence: plan1082_collections_places.rs O1–O29观察矩阵（cargo test -p ace-av --test conformance plan1082_collections_places::t01 pass 1） | next: T-02`

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
