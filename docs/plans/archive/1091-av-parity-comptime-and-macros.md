---
plan_id: PLAN-1091
status: archived
completion_kind: delivered
feature_name: AV编译期执行与宏兼容补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1085, PLAN-1088]
supersedes_spec_components: [docs/specs/ace-migration/m2-metaprogramming.md, docs/specs/auto-lang/comptime/overview.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1091] AV编译期执行与宏兼容补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`comptime/evaluation-macros.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-13**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

合并已交装配折叠和真实AV CTE服务的普通语言接面，补旧已支持宏语义并保确定性/预算。

本包交付`IF-AV-PARITY-CT`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- 已交#if/#for/#is/#in目标选择与#{expr}共同普通工程，CT引用合法函数/泛型/模块/值，显式target/host事实不能混读。
- 实际CT求值经现有source-cte/AV服务，回灌常量与已批准值域、无运行期残留；文本/复合值扩面先有限可行性探测，不能从旧AST臆造任意CT承诺。
- 既有#debug/#info/#warn/#error的可观察行为经同一语义路由；普通#name(args)函数糖与日志兼容分账。未知宏采用现行具名拒而不继承静默忽略，不发明#macro定义/hygiene语言。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1091/auto-lang`，分支`plan-1091-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1091/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-21：用户以 `/goal 完成计划1091` 授权按 r3 执行、独立复审、合同内修复、合入、归档和清理。首次执行树 `D:/autostack/.wt/lang-1091/auto-lang`，分支 `plan-1091-dev`，基面 `196ce23f735e367500e065318c4259dbc701271c`，只读兄弟依赖 auto-down `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。材料区已登记为 in-flight。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

966装配谓词有限，936/958已有真实scalar AV CTE；旧表达式#{expr}多为运行期伪CT、#for数组和宽值回写不完整。旧#name只debug/info/warn/error路由Log，未知宏静默忽略，并不是任意文本宏系统。

既有来源与边界：

- [source_ct_936.rs](../../../experimental/ace/crates/core/tests/cases/source_ct_936.rs)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：旧CTEE Rust tests/未接入的comptime示例按source逐项核；#日志宏可达consumer；936/958既有CT用例。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [transformer.rs](../../../crates/auto-lang-core/src/comptime/transformer.rs) | `0ca29705f668c03602aa1e87a3e51b92709b93910a619a5a36dbb381f7ede8a1` |
| [stmt_emit.rs](../../../crates/auto-lang-core/src/vm/codegen/stmt_emit.rs) | `c2779a620090f5423731282a0e546029b08939ddda5e2ac00dca7ac07d4d448d` |
| [binding.rs](../../../experimental/ace/crates/core/src/source/packages/binding.rs) | `aa2cee1daf9ce0912f2de4952bd9618070c77807e0af8dcf9b5a2e36c0483f6d` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |

硬依赖：PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1085, PLAN-1088；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [transformer.rs](../../../crates/auto-lang-core/src/comptime/transformer.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [stmt_emit.rs](../../../crates/auto-lang-core/src/vm/codegen/stmt_emit.rs)：本包旧行为/独立oracle来源，只读，不修改旧VM语义。
- [binding.rs](../../../experimental/ace/crates/core/src/source/packages/binding.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [lib.rs](../../../experimental/ace/crates/source-cte/src/lib.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/comptime/evaluation-macros.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-metaprogramming.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-lang/comptime/overview.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1091-原文档清单)静态主承接 **0份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1091.real-cte-call` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1091.assembly-condition` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1091.macro-function-sugar` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1091.unknown-target-fact` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.cte.in-name | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `CT-NOT-RUNTIME` | AC-03/AC-04 | work / T-04..05 | 终态无HashExpr未服务/无运行期callee执行；改CT依赖body/target/cap/provider使相关结果失效，坏CT保旧并恢复首读与冷编一致。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `CT-RESOURCE` | AC-03/AC-04 | work / T-04..05 | 非法IO/撤权、CT预算边界与无限递归受控拒绝；1035资源值支持不能自动当可portable-freeze CT值域。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: real-cte-call
id: "av.parity.1091.real-cte-call"
tags: ["av-language-parity", "plan-1091"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn sum(a int, b int) int { return a + b }
fn main() int { let n = #{ sum(20, 22) }; return n }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr", "ac.source.cte.scalar/v1"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
同时观察CT实际dispatch和最终运行MIR常量替换，单看42不能证明CT发生。
`````

`````autodown
## Case: assembly-condition
id: "av.parity.1091.assembly-condition"
tags: ["av-language-parity", "plan-1091"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
#if true { fn answer() int { return 42 } } else { fn answer() int { return 0 } }
fn main() int { return answer() }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr", "ac.source.cte.scalar/v1"]
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
## Case: macro-function-sugar
id: "av.parity.1091.macro-function-sugar"
tags: ["av-language-parity", "plan-1091"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn add(a int, b int) int { return a + b }
fn main() int { #add(19, 23); return 42 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr", "ac.source.cte.scalar/v1"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
这是966已有普通函数糖；日志宏另按原可达provider输出oracle验，不称任意文本宏。
`````

`````autodown
## Case: unknown-target-fact
id: "av.parity.1091.unknown-target-fact"
tags: ["av-language-parity", "plan-1091"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
#if sys.unregistered_fact { fn answer() int { return 42 } }
fn main() int { return 0 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr", "ac.source.cte.scalar/v1"]
stage: "resolve"
code: "bind/source.cte.in-name"

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
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-comptime ace-av ace-artifacts
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
| T-02 | work | T-01 | 已交#if/#for/#is/#in目标选择与#{expr}共同普通工程，CT引用合法函数/泛型/模块/值，显式target/host事实不能混读。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 实际CT求值经现有source-cte/AV服务，回灌常量与已批准值域、无运行期残留；文本/复合值扩面先有限可行性探测，不能从旧AST臆造任意CT承诺。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 既有#debug/#info/#warn/#error的可观察行为经同一语义路由；普通#name(args)函数糖与日志兼容分账。未知宏采用现行具名拒而不继承静默忽略，不发明#macro定义/hygiene语言。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-CT。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：基面、旧源角色与八个普通入口探针已冻结，见 §9。
- [x] **T-02** work：共享 Auto CST 装配和包目标 context 已接通。单文件、包导入、泛型和常量检查通过。
- [x] **T-03** work：已接真实 source-cte 与受检 AV。父侧看到实算一次、运行期调用为零，负对照运行期调用为一。
- [x] **T-04** work：日志双流、普通函数宏、授权/值域/预算具名诊断、取消实达 AV、失败保旧与缓存重开均通过定向检查。
- [x] **T-05** work：20 条 canonical AV 检查全绿，七种语料负控和源身份对账完成；旧义务见 31 行映射。
- [x] **T-06** work：冻结并集完成并归因两项首红，余项无失败，见 §9。
- [x] **T-07** review：独立冻结复审通过，见 §9；待 T-08。
- [x] **T-08** merge：已快进落主、核验四份规范和实际 AV 结果，完成归档及受控清理。

代码与规范已落主，复审、归档及受控清理完成。8/8 项任务和 10/10 项验收已核验。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- 2026-11-21 T-01 输入核对。五个硬依赖 PLAN-1078/1079/1080/1085/1088 档案与普通 recipe、AV 语料目标选择、包闭包与实例化入口在基面可定位。旧 CTEE 和 Log 发射源码 SHA256 与 §4 相同；共享 resolve 已前移，基面 SHA256 为 c7b722e6f7d2e1d84b74fdfeec6843bbc2a7e5f3202311dc9656daae843dda6d。无主承接旧 VM 文档。另核 16 个无注册检查的 `test/comptime` 示例、四个 cookbook 日志/外部 provider 文档各 VM/Rust 两个原义务、四个旧 CTEE Rust 断言、三个旧 parser AST 断言。逐项源/角色/预期/owner 见 `legacy-map.tsv` 新 31 行。原 AutoVM truthiness、外部 log/env_logger/tracing 和仅运行期伪 CT 的示例预期各有独立角色，不能将新 AV 源码检查视为原检查已执行。
- T-01 可行性基线。`cargo build --manifest-path experimental/ace/Cargo.toml -p ace-av --bin av-worker` debug 37.62s/退出 0。普通入口 `av-worker case-serve` 八个探针分别 0.039–0.420s，已观察同一份源普通函数返回 42，四个设计用例原先均在语法门拒绝。报告 `baseline-output.json` SHA256 `0de16d30651c2b05a09eda18cb2235de7302a0a30cae1e60813278f3d67df3e6`。现有 966 结构选择与 936 的标量冻结都需接至普通入口；验证 CT 调用、进程重开、运行期调用次数分离。按已交标量合同保 int/bool。字符串与 List 经完整入口探测，旧服务没有对应的可持久页 tag；普通 CT 给带 span 的具名值域拒绝。保留原始语料及外部 provider 的独立义务。

- 2026-11-21 T-02..T-05 冻结检查点 `67b1ec500a4107195d8cad2d039b1836ed703bad`。初始提交 `1646ed712de6d64072564be81e44da194b19429e` 重放至 master `483fe91d3458b39ca583ba92651b82fa4ed4dbe6`；两个候选 tree 唯一不同为 PLAN-1047 归档文档，源码、验证配置、依赖和 Case 未改。拟沉淀四项 Spec，提案 `spec-proposal.md` SHA256 `961a31a3370f06e611cc3f33182295e04be939119d8f4cd95d6d18d1b8b76f45`。
- 普通 AV 观测：`real-cte-call` 返回 42，`ct_execute=1,runtime_calls=0`；其动态调用对照反转两项计数。`macro-function-sugar` 返回 42，`ct_execute=0,runtime_calls=1`，额外带输出的函数宏控制实际打印 42。`compile-time-io-denied` 三项 CT/运行时/host 调用均 0，`compile-time-frame-budget` CT 实达一次并以具名资源错误结束。预算/授权/未知名通过父侧对源码 span 的准入。异目标上下文的选择值为 42，本地上下文为 1，包输入身份不同。改坏依赖触发真实 CT 陷阱后，同一旧源首次重开命中旧页且返回 42。修复版首算 43，冷编也为 43。授权变化使原页失效，撤权使 CT 与页命中均为 0。取消负控从短运算改为真实循环达到 AV 轮询点；前两次失败分别为短运算未轮询和误用旧 `while` 写法，均保留。
- 选择控制使用相同源和普通 Case 父端。错 AV 预期 exit 1（1 FAIL）；缺有效 AV 或整 AV ignore exit 2；重复用例名 exit 2。错非 AV oracle 在 #ignore 时为 2 SKIP，取消阶段标记后在 AV 视图为 2 NOT_RUN，两边均 1 AV PASS/exit 0。源码等值改动后报告 source SHA256 由 `79f9ad48…` 变 `b8f5c091…`，预期身份保持 `4991e03e…`。没有执行非 AV 检查。控制结果 SHA256 `e3beb67f9fd798d836aa68ee90a3771d0ce3fe6837ad9273cd46fd5392ac9062`。

| 本轮定向范围 | 判决 | 准备 | 运行总墙钟 |
|---|---|---|---|
| `auto-case lint --suite language.suite.ad` | 18 文件、445 Case、引用准入通过 | debug 父工具 195.379s | 1.650s |
| `auto-case run --target av` 同字节 canonical 子套件 | 20 PASS、0 FAIL，exit 0 | 当前 debug AV worker 6.811s | 1.962s |
| 同一父端的 7 种选择控制 | 逐项判决在上方，均符合控制目的 | 复用当轮工具 | 单项 0.025–0.466s |
| `cargo nextest run -p ace-av -p ace-source-cte --lib` 定向 CT | 42 PASS（4 新、38 既有），20 未选，exit 0 | 构建和发现未独立取时 | 整命令 21.043s，执行 3.056s |

20 项 Case 测量含父侧工作、子进程和清理。秒数依 canonical 文件顺序分别为 0.616 / 0.604 / 0.536 / 0.614 / 0.575 / 0.613 / 0.670 / 0.689 / 0.521 / 0.599 / 0.572 / 0.529 / 0.647 / 0.575 / 0.567 / 0.666 / 0.674 / 0.498 / 0.655 / 0.660。新增原生用例 `explicit_package_target_changes_ct_selection_and_identity` 0.103s，`ct_cancel_reaches_the_av_and_preserves_the_demand_span` 0.123s，`ct_dispatches_once_and_leaves_no_runtime_call` 0.206s，`ct_cache_reopens_after_failed_edit_and_uses_current_source_and_grants` 0.356s。未单独测出的发现或清理阶段未由其它计数代填。Case 报告 SHA256 `405426a57e5dfcbbc9c62285efb5a9111df944f06c711c1264dde2c5b2bd8465`，native 日志 SHA256 `8380c4eaa4d1ff2994debF1ebc043f6e578f3c23016f9d340071092ed06e7ab6`。

- 2026-11-21 T-06 最终候选 `c4a307d36c6be6a6454c5e665490e907707f0eaa`。普通入口发现 `42i32` 已合法投影为 Int 类型的 TypedLit，旧折叠误归为越域。新增负数 `#{ -42i32 } + 84` 红转绿，19 条对照与其余 1 条旧验收仍通过。21 条 canonical AV 检查全绿（2.043s / exit 0）。新用例单项 0.600s。父报告 SHA256 `31c16f69ca6177f93285b754c486011dffe48b4cda4c4b3d15e9d78dbdc60545`。纯文本和 List 返回维持带源 span 的冻结值域拒绝。
- 完整 AV 驱动准备成功，`cargo t case-av-language-parity --prepare-only --out <PLAN-1091 scope>/case-final-prepared` 只选择并准备 debug `case-parent` 和 `av`，prepared.json SHA256 `890425de36b34faa4fe896da3f411a13dba98445edb9fe8a971ae2643b7f484f`。本轮冻结验收的执行等待机器空闲。相邻 UI worktree 的 cargo test/build 和编译器 PID、创建时间及在途命令见本计划 scope 的占用日志。混合固定组的 AV-only/windows 选择从原组与实际 mixed 用例角色导出，只排除非 AV 实际执行、Linux/WSL 和原正式性能，前门和 AV 验证保留。

- T-06 公共语料在最终候选通过。正式 `cargo t case-av-language-parity --run-prepared …` 父子 exit 0，整命令 47.512s，Case 父端实际执行 42.729s。594 检查中 446 PASS、148 SKIP，0 FAIL/ERROR，21 个本包 AV 检查全部通过，单项 0.522–1.111s。准备阶段的 debug `case-parent` 41.34s，worker 编译及准备细项见准备收据。门禁结果 SHA256 `151e195ea7e07a944b03c760423ad5c4dc9c6dda6b3d5a06980b30a6eca8a3a2`，父报告 SHA256 `b5ea53f37e8a0b848da648f1b211aacc7b472898a382d85f1c47d4a74c974ede`。AV worker SHA256 `a2df020203866d41c1417c4a5472f71eee8113b9c9d985f10f830fb2f598f8e9`。该独立 Case 命令消费已准备工具，统一 Rust 固定组等待可用窗口后单独准备。

- 冻结 Windows AV 并集准备 143.328s（就绪窗口等待 11.561s），七个归档与工具通过源/配置/哈希核验。混合固定组保原职责，按测试职责过滤非 AV 执行子项与 Linux/WSL，未准备 AC/MN/A2X 工具。第一次执行在比对子组派生配置和仓库原配置 SHA 的外层检查处退出，未运行用例。重放分别校验两个 SHA 和相同的 prepared 条目，工具和归档未重编。真实并集 83.953s，顶层 exit 1。原始门报告 SHA256 `6126a0115124facc2aef10c55e15fddea5dedd652c084f862fc702b68230a619`，prepared SHA256 `766717d6f255ec5e3d1d0be858db1bc0643987b59a9e08fb086d4728eaeb0558`。表内发现与命令实测分列，日志只报告集成清理时间，单独清理未取样。

| 固定职责组选集 | 实际结果 | 排除 | 发现 (s) | 执行命令 (s) |
|---|---|---|---:|---:|
| ace-source | 932 PASS / exit 0 | 1 原条件 | 0.564 | 8.548 |
| ace-av | 1484 PASS、1 原红 / exit 100 | 53（29 混合角色，23 正式性能，1 Linux） | 1.888 | 37.515 |
| ace-syntax | 159 PASS / exit 0 | 3 正式性能 | 0.986 | 3.425 |
| ace-semantics | 295 PASS / exit 0 | 10 正式性能 | 2.643 | 6.644 |
| ace-comptime | 136 PASS / exit 0 | 17 正式性能，2 原条件 | 2.016 | 13.594 |
| ace-artifacts | 42 PASS、1 首红 / exit 100 | 6 正式性能，1 原条件 | 0.468 | 4.616 |
| ace-lowering | 9 PASS / exit 0 | 0 | 0.391 | 0.391 |

- AV 原红为 `ace-av-core::layout::tests::layout_rejects_non_record_and_empty`。实际空记录布局为 size 0、align 1，旧断言要求拒绝。隔离同归档重放 exit 100（测试 0.011s，整命令 1.821s）。PLAN-1088 提交 `6f26279ab` 已删除空记录拒绝，基面与本候选的 `layout.rs` 同 blob `2c9adfefb425a7bf56f1fc6d3355d41b6962d991`。AV-core、仅本仓依赖 foreign-contract、锁文件、根 Cargo 配置及 nextest 断言配置 diff 均空。该原红归 AV layout/PLAN-1088，保原 verdict。日志 SHA256 `044d208e2e9ddea56ec2fee36daae034c950ae37b396c0bceacad84fda5a1df8`。
- Artifacts 首红为 `review_r3::r3_f01_concurrent_staging_budget_is_atomic`，并发运行中 3 成功/1 拒，既有 PLAN-1098 同症状。相同归档单项隔离重放 exit 0，测试 1.212s、整命令 1.741s，日志 SHA256 `b4191d9bc8949896b51f37434d51756e4bd3c66ad1d32970dc4a5f7887f961de`。首红和重放绿分列。无本包引入失败，原例、采样及判决保持。
- `stage: work | plan_id: PLAN-1091 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1091/auto-lang | branch: plan-1091-dev | base: 483fe91d3458b39ca583ba92651b82fa4ed4dbe6 | code_commit: c4a307d36c6be6a6454c5e665490e907707f0eaa | auto-down: 45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | task_ids: T-01..T-06 | next: review`。同一工作树已暂停写入并交复审。拟交接口 `IF-AV-PARITY-CT` 使用普通 `ac-av/language-source-run-v1` / `d_main`，AV `av.closures/1`，worker `avworker/1`，MIR lowering 7。输入身份含源码字节与包图、目标和宿主事实、cap/provider 及执行 ABI。实际证据见上，合入规范与运维清理均待 T-08。
- `stage: goal | plan_id: PLAN-1091 | plan_revision: r3 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1091/auto-lang | commit: c4a307d36c6be6a6454c5e665490e907707f0eaa | next: review`。

### T-07 最终复审

在实施会话内换至冻结提交，以源码、原始报告和重新选择的输入重建判决。无单独 reviewer 会话。工作树 `D:/autostack/.wt/lang-1091/auto-lang` / `plan-1091-dev` 清洁。核验提交 `c4a307d36c6be6a6454c5e665490e907707f0eaa`，基面 `483fe91d3458b39ca583ba92651b82fa4ed4dbe6`，auto-down `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。复审期间代码无写入。

| 验收 | 判决 | 独立核对 |
|---|---|---|
| AC-01 | PASS | 父侧逐 Case 固定值与双流，21/21。另组合未选 #if 分支、嵌套 CT、定向 #for 与普通函数宏，核值、stdout 与运行计数。 |
| AC-02 | PASS | 拒绝 code、stage、原路径与实际 span 取样。非法外部作用和可变全局 CT 次数为零。存储打开失败具名 `cte.infra/store`，原文件字节一致。 |
| AC-03 | PASS | 对照区分 CT 执行/运行期调用。取消实达 AV，发布计数为零。1024 次装配展开返回 1024，1025 次拒绝。旧源失败修复与冷编，以及包 target/host 身份对照具备独立断言。 |
| AC-04 | PASS | 旧义务映射 31 行保所有原角色与条件；主承接旧 VM 文档为零。L 行对账如下，本包 AV required 21 条全部有实际结果。 |
| AC-05 | PASS | 追踪 `language_source_worker` → Auto CST/assembly → 共享实例化/绑定 → `prepare_auto_ct` / source-cte → HIR 验证、MIR/ABC 准入后 AV。命令和消费者链中无 legacy VM 路由或根据 Case ID 改变实现。 |
| AC-06 | PASS（两项首红有归因） | 复核 3059 次原生执行和 594 个父侧检查；两项失败按原红和隔离重放保留。现有格式/告警基线与打开文件无新增未处理告警或调试输出。 |
| AC-AV-ONLY / AC-FC | PASS | 复算目标选择、四种拒绝/变异控制、源变更、原检查角色及唯一作者。非 AV 工具未准备、检查未执行。新增 25 条永久义务均低于 10s，完整测时见 T-06。 |

| 分母行 | PLAN-1091 的 AV 见证 | 当前 owner/状态 |
|---|---|---|
| L-32 装配项与语句 | `assembly-condition`、`assembly-statement-for-and-is`、`assembly-namespace-package`、复审 `statement-demand` | PLAN-1091 Windows PASS |
| L-33 表达式替换 | `real-cte-call`、`concrete-generic`、`readonly-module-value`、布尔/容器元素/型宽、复审两层调用；文本和组合结果显式拒绝 | PLAN-1091 Windows PASS（标量合同） |
| L-34 宏 | `macro-function-sugar`、`function-sugar-effects`、`runtime-log-streams`、`unknown-macro` | PLAN-1091 Windows PASS；cookbook foreign provider 继续原检查 |
| L-35 元数据属性 | `metadata` 校验既有 CT 字段查询。声明属性由原 owner PLAN-1092、971、980/982 承接 | 本包 CT 查询 Windows PASS；声明属性未选 |
| L-36 平台事实 | `unknown-target-fact`、`assembly-namespace-package`、显式目标与宿主不同的 Rust 断言 | PLAN-1091 Windows PASS；仅已注册事实 |

复审单独运行 `review-holdouts.py`，使用同一冻结 worker，6 条均符合断言、exit 0、整命令 0.556s。嵌套 CT 与未选分支 0.080s、循环选中的函数宏 0.052s、可变模块状态拒绝 0.068s、1024 展开 0.125s、存储失败 0.047s、语句位 CT 0.084s。探针进程和收尾均包含于实测，未单独取样。先前连续使用同一存储的探针复跑返回 42 和 2 次命中，达冷执行计数断言前已触发已知热命中，改用新目录后通过。冷探针报告 SHA256 `0c4dc07bdb6015a94476659d0e43ae2048e36df41b32cf0f360d5ef89b9b3ea9`。复用 T-06 同提交的公共 Case 与 Rust 固定组证据，复审未再跑整组。

遗漏、延后和临时兼容扫描未发现本包欠账。现有空记录布局旧断言作为 D-1088-LAYOUT-01 登记至已知债，artifacts 同症状沿用 PLAN-1098 首红与重放区分。已审核 SD-01/02/03/FC 四份实际文档内容和各自当前源身份。复审清单 `reviewed-specs.json` SHA256 `038a95470858e41a12e6ffaba0a2d85078ad97fd700be37fbdbf5501345e4337`。原 `supersedes_spec_components` 四路径、`new_spec_components: []` 和 `touched_goals: [GOAL-002]` 保持。GOAL-002 沿现行部分达成状态。

- `stage: review | scope: final | plan_id: PLAN-1091 | plan_revision: r3 | outcome: pass | reviewed_commit: c4a307d36c6be6a6454c5e665490e907707f0eaa | base: 483fe91d3458b39ca583ba92651b82fa4ed4dbe6 | dependency: auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | findings: no in-scope open items | pending: landing, canonical verification, archive, guarded cleanup | next: merge`。
- `stage: goal | plan_id: PLAN-1091 | plan_revision: r3 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1091/auto-lang | commit: c4a307d36c6be6a6454c5e665490e907707f0eaa | next: merge`。

### T-08 合入收据（PLAN-1091:r3）

- `prepared`。冻结复审为 `c4a307d36c6be6a6454c5e665490e907707f0eaa`，auto-down 为 `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。准备提交 `2ee399ed0ce78e8b5fb520572f5edc57aa59babf` 仅追加四份已审规范，28 行。核对规范原 SHA256、评审后 SHA256 和 HEAD 父提交。复审后的固定组选集、语料、工具、依赖及配置未发生字节变化，沿用 §9 的同内容实跑及两项原始判决。
- 合入阶段仅复用 debug 工具，无额外准备构建。冻结官方 Case 报告的本包用例实测如下。每格为该检查的执行时间，父端已验独立值、双流或错误；全量准备、发现、运行、占用等待与四个新 Rust 用例实测见上文。

| 新 Case | ms | 新 Case | ms | 新 Case | ms |
|---|---:|---|---:|---|---:|
| real-cte-call | 775 | assembly-condition | 601 | macro-function-sugar | 580 |
| unknown-target-fact | 585 | assembly-statement-for-and-is | 676 | assembly-namespace-package | 632 |
| concrete-generic | 636 | readonly-module-value | 759 | fold-list-element | 794 |
| fold-bool-condition | 674 | metadata | 924 | runtime-log-streams | 693 |
| function-sugar-effects | 529 | unknown-macro | 606 | capability-revoked | 579 |
| compile-time-io-denied | 958 | compile-time-frame-budget | 980 | assembly-loop-budget | 713 |
| compile-time-text-boundary | 1111 | compile-time-list-boundary | 840 | fold-typed-i32 | 522 |

- `landed`。`git rebase master` 无新增重写。`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 D:/autostack/.wt/lang-1091/auto-lang` clean（1.627s）。`git merge --ff-only plan-1091-dev` 从 `483fe91d3458b39ca583ba92651b82fa4ed4dbe6` 到 `2ee399ed0ce78e8b5fb520572f5edc57aa59babf`。四份工作文件与该提交 blob 一致。原始实施基面重放见 T-02..T-05。
- 主检出真实消费。`auto-case run --suite <master>/crates/auto-lang/test/cases/language.suite.ad --target av --case av.parity.1091.<name> --out <scope>/landing-<name>` 绑定原冻结父端 `0c2cd34f88750fa365b5251878530bb577ea8fcc5d956d310145b82bbc1e8fbb` 与前述 worker，复跑 `real-cte-call`、`assembly-namespace-package`。两次均 exit 0，分别 0.442/0.460s 子检查，均返回 42，`ct_execute=1`、`runtime_calls=0`。每次报告 1 PASS、148 SKIP、445 NOT_RUN。报告 SHA256 分别为 `ff3237c0ee708d510ccfba3f561c27ee931c452a184f821cb8d96f2171ec4635`、`f2a7cb33eeb78ee275e163fcd74a64c0f789b4f69e8ac590f79f3006bbbf2e63`。两条父命令与工具核验共 4.38s。
- `canonical_verified`。交付 `IF-AV-PARITY-CT` 普通 `d_main` 入口与上述冻结协议。主检出 SHA256 逐字匹配评审清单，Git blob 与落地主提交匹配。

| 正式来源 | SHA256 |
|---|---|
| [AV 语言齐平](../../specs/auto-av/language-parity.md) | `bed74d28a69c79f2ed9d49b4befde35cf2174dd4a340ca613625892b7ebd3725` |
| [ACE 元编程](../../specs/ace-migration/m2-metaprogramming.md) | `bf467d70fab335ee8ace133e19b39929d349e3f101e5c6d315bdd0488c79395e` |
| [Auto 编译期执行](../../specs/auto-lang/comptime/overview.md) | `3e6e2511bc87f331d4c95fbc4183b433520ad820d4b0ac2b961b034b4b58e236` |
| [按特性组织的语料](../../specs/auto-test-case/feature-organized-corpus.md) | `1cabd39c2b9f941c81069ed5bfb7e139e7a8a8e09d42193b9872c00b4d0b7731` |

- [公开套件](../../../crates/auto-lang/test/cases/language.suite.ad) 引用唯一 [编译期与宏作者](../../../crates/auto-lang/test/cases/language/comptime/evaluation-macros.case.ad)。[旧义务映射](../../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv) 记录 31 项的原角色、预期与 owner。
- 输出对账。开发探针、准备归档、测试输出、子进程临时根、复审与落主报告均在 `D:/autostack/plan-artifacts/plan-1091`，清点 9025 文件、306198968 字节。调试 target 在专属 auto-lang 检出的 `target` 与 `experimental/ace/target`，含检出共 89126 文件、15834983487 字节。组内只另有无本计划改动的 detached auto-down，1318 文件、27021855 字节。逐轮输出未发现外置 target、安装根或其它证据目录。主检出中既有父工具仅作只读兼容检查，输出均显式指向本计划 scope。清点时无指向本组或本 scope 的活动构建/测试进程。
- 归档路径为 `docs/plans/archive/1091-av-parity-comptime-and-macros.md`，当前 7/8 项任务、9/10 项验收。清理前保留 T-08 与 AC-08 待办。实际归档、引用与受控清理结果见下一个收据。

### 归档与清理回证

- `archived`。归档提交 `a74ef71c2e31b0bb47ba3058fd50188c364a90f9` 的唯一父提交为交付提交。实际归档已核 `status: archived`、`completion_kind: delivered`、任务表与勾选、22 个出站链接及锚点、7 个入站链接（四份规范、两张设计导航、债务登记）、唯一 canonical 作者和四份提交源。主检出并行簿记由各会话持有。
- `cleaned`。临移除对 `D:/autostack/.wt/lang-1091/auto-lang` 及同组 `auto-down` 分别重跑 PowerShell reparse 闸门，均 clean。两个干净检出分别 22.156s、0.183s 移除，随后删除已合入的 `plan-1091-dev` 与仅有的空组目录。两仓 Git 注册与磁盘路径均核为空。
- 归档提交后，核对唯一材料目录的索引 owner、实际绝对路径、无活动占用和 reparse clean。PowerShell 字面路径删除 `D:/autostack/plan-artifacts/plan-1091` 1.843s，复核目录不存在。常规构建产物随树清除。已折叠材料 9025 文件、306198968 字节。共享索引 `D:/autostack/plan-artifacts/.retention/policy.json` 该属主条目在 2026-11-21T01:53:58Z 更新为 `deleted-at-merge`，无具名材料持有。
- 交付范围为源码和规范。主检出现有 release `av-worker.exe` 构建于 2026-11-19T03:10:13Z，早于本交付。批量回归收据 2026-11-20T00:00:00Z，覆盖号 1098，核验时约 33.9 小时，无超过覆盖号的 %5 归档，未到期。
- `stage: merge | plan_id: PLAN-1091 | plan_revision: r3 | outcome: pass | checkpoints: prepared, landed, canonical_verified, archived, cleaned | delivery_commit: 2ee399ed0ce78e8b5fb520572f5edc57aa59babf | archive_commit: a74ef71c2e31b0bb47ba3058fd50188c364a90f9 | archive: docs/plans/archive/1091-av-parity-comptime-and-macros.md | tasks: 8/8 | acceptance: 10/10 | cleanup: verified | next: done`。
- `stage: goal | plan_id: PLAN-1091 | plan_revision: r3 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 2ee399ed0ce78e8b5fb520572f5edc57aa59babf | next: done`。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
