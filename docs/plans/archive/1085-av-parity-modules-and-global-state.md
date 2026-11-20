---
plan_id: PLAN-1085
status: archived
completion_kind: delivered
feature_name: AV模块、全局状态与初始化补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1081, PLAN-1083]
supersedes_spec_components: [docs/specs/ace-migration/m2-modules.md, docs/specs/ace-migration/m2-generics.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1085] AV模块、全局状态与初始化补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`modules/imports-globals.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-07**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

将完整语言值接入模块包/全局状态与会话生命周期，关闭owned全局转移和普通顶层程序差额。

本包交付`IF-AV-PARITY-MODULE`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- L-37/38/41/44普通模块部分：多文件use/dep/alias/pub/reexport、包同名隔离、前向/环依赖、私有成员与模块全局初始化源序/once。
- 全局值包括Str/List/record/variant/closure/dynamic，显式move后的状态、赋值替换释放、两session隔离、失败初始化回滚/重试与卸载。
- 顶层脚本和fn main的入口规则明确；配置输入只消费IF-982-CONFIG-ACE及已有SDK，通用语言问题修同一源前门，982不反等本包整包。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1085/auto-lang`，分支`plan-1085-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1085/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-20 用户通过 /goal 完成计划1085 授权 r3 全链实施、独立复审、有界修复、合入、归档与清理。专属树 D:/autostack/.wt/lang-1085/auto-lang（plan-1085-dev，master 基线 dcd4fdafd412be911dc44ba1fee911a916d9c198），只读兄弟依赖 auto-down@72b097006f34db3a871449f10e213820d51abef6。材料 D:/autostack/plan-artifacts/plan-1085 已登记。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

960已有包图、once/diamond与失败恢复；AV有GlobalGet/Set，但D-963-C4资源record全局.move仍缺。982的配置文档有自己的生产owner，不在本包复制配置解释器。

既有来源与边界：

- [a2r-project.document.case.ad](../../../crates/auto-lang/test/cases/pilot/a2r-project.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：03_variables；17_modules；99_bootstrap中的use/store/state；语言Rust模块/初始化回归。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [binding.rs](../../../experimental/ace/crates/core/src/source/packages/binding.rs) | `aa2cee1daf9ce0912f2de4952bd9618070c77807e0af8dcf9b5a2e36c0483f6d` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [config_document.rs](../../../experimental/ace/crates/core/src/source/config_document.rs) | `20af68d92691837dc9f197b85d3d2137e4923c1bb0b3b272a58ccc58ba11abb1` |
| [packages_store.rs](../../../experimental/ace/crates/source-query/src/modules/packages_store.rs) | `e9eefe4d61215b467d53717452ecac435bf17c3d237f14fdb0ce4810d1760823` |

硬依赖：PLAN-1078, PLAN-1081, PLAN-1083；实际外部接口：IF-982-CONFIG-ACE（仅实际配置消费者）。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [binding.rs](../../../experimental/ace/crates/core/src/source/packages/binding.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [config_document.rs](../../../experimental/ace/crates/core/src/source/config_document.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [packages_store.rs](../../../experimental/ace/crates/source-query/src/modules/packages_store.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/modules/imports-globals.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-modules.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-generics.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1085-原文档清单)静态主承接 **9份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1085.multifile-call` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1085.module-global-once` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1085.owned-global-move` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1085.missing-import` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.module-missing | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `MODULE-DIAMOND` | AC-03/AC-04 | work / T-04..05 | 同一叶模块被左右两路导入只初始化一次；以实际trace验证顺序，模块同名不同包版本不得串缓存。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `GLOBAL-STATE` | AC-03/AC-04 | work / T-04..05 | 两个会话、初始化中throw、坏编辑保旧→恢复→首次读取、owned全局.move后重读具名拒/已定空态、卸载owner归零。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: multifile-call
id: "av.parity.1085.multifile-call"
tags: ["av-language-parity", "plan-1085"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use lib.answer
fn main() int { return answer() }
```

#### lib.at

```auto
pub fn answer() int { return 42 }
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
## Case: module-global-once
id: "av.parity.1085.module-global-once"
tags: ["av-language-parity", "plan-1085"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use state.bump
use state.read
fn main() int { bump(); return read() }
```

#### state.at

```auto
var n = 40
pub fn bump() { n = n + 2 }
pub fn read() int { return n }
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
## Case: owned-global-move
id: "av.parity.1085.owned-global-move"
tags: ["av-language-parity", "plan-1085"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type Packet { text Str }
var packet = Packet("answer".clone())
fn take(move p Packet) int { if p.text == "answer" { return 42 }; return 0 }
fn main() int { return take(packet.move) }
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
## Case: missing-import
id: "av.parity.1085.missing-import"
tags: ["av-language-parity", "plan-1085"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use absent.answer
fn main() int { return answer() }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "resolve"
code: "bind/source.module-missing"

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
cargo t case-av-language-parity ace-source ace-semantics ace-av ace-artifacts
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
| T-02 | work | T-01 | L-37/38/41/44普通模块部分：多文件use/dep/alias/pub/reexport、包同名隔离、前向/环依赖、私有成员与模块全局初始化源序/once。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 全局值包括Str/List/record/variant/closure/dynamic，显式move后的状态、赋值替换释放、两session隔离、失败初始化回滚/重试与卸载。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 顶层脚本和fn main的入口规则明确；配置输入只消费IF-982-CONFIG-ACE及已有SDK，通用语言问题修同一源前门，982不反等本包整包。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-MODULE。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

执行状态：

- [x] **T-01** work：冻结九份旧文档与原检查、13 份基线源输入、四项草案和本包诊断，见 §9。
- [x] **T-02** work：包图、别名、重导出、隔离与分模块原诊断已实跑。
- [x] **T-03** work：常规 owned 全局、显式移取、替换、回滚和释放已实跑。
- [x] **T-04** work：脚本和 fn main、预算/取消、恢复与边界已实跑。
- [x] **T-05** work：九个原义务迁移、作者和目标隔离控制均已核。
- [x] **T-06** work：冻结固定组实际终态、继承红与样本时间见 §9。
- [x] **T-07** review：独立复放、合同逐项与规范核对，见 §9。
- [x] **T-08** merge：代码与规范线性落主、归档、guard 及材料清理均核，见 §9。

交付、归档与清理均已核验。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- 2026-11-20 `stage: goal | plan_id: PLAN-1085 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1085/auto-lang | commit: dcd4fdafd412be911dc44ba1fee911a916d9c198 | next: work`。用户整链授权见 §4。1078/1081/1083 已归档且在 master；0/8，进入 T-01。

- 2026-11-20 T-01 基线探测（`dcd4fdafd`，debug/incremental worker 构建 35.27s、exit 0）：直接走 `av-worker case-serve`，原 03 变量例 001/002/003/005 返回 3/2/11/41，006 精确拒 `verify.immutable-write`，004/008 语法拒，007 重声明拒。17.001 和四份草案在单源入口未连通。13 源观察 `t01-av-baseline.json` SHA256 `17c3d97f17809edb45c2a1cc6d58cc22f7e01c96700852694c0f3a9b1da2498f`。模块原用例的仓内 import 源已定位到 `crates/auto-lang/auto/greet_mod.at`；原独立 VM stdout 为 `world\n3`。982 配置 producer 的正式 phase fold `auto-lang@0b6d3198d` 已核为 master 祖先，原 982 §9 round 17 的产品消费者 `auto-man@39bc016`，仅当前包实际配置路径消费。
- 2026-11-20 T-02 在途提交 `af7dca224`：普通 Auto 包图使用共享 Schema/CST、包绑定、签名推断、受检 HIR/MIR 和重新准入的 ABC linked units；父侧以完整虚拟路径/字节闭包 SHA256 校验观测。缺模块与私有导入实测 `bind/source.module-missing` / `bind/source.module-private`，原 span 分别 0..17 和 0..14，父侧阶段均为 resolve。首轮固定入口 `cargo t case-av-language-parity --run-prepared` 实际环境走了预备构建并以 5 个新项目 source-hash 旧比较报 ERROR，111 PASS、41 NOT_RUN，子 exit 2，8.424s。修正父侧共用校验后，显式预构建 `cargo build --locked --offline -p auto-lang --bin auto-case` 53.47s；使用同树源绑定 debug worker 与父工具按 `auto-case run --suite …/language.suite.ad --check av --out …/t02-direct-suite` 实跑 116 PASS、41 NOT_RUN、0 ERROR/FAIL，11.104s。直接父工具因部分选择返回 exit 1。新增 5 例按上表顺序及新增私有/重导出分别实测：multifile-call 0.712s、module-global-once 0.897s、missing-import 0.768s、dependency-reexport-alias 0.816s、private-import 0.768s。报告 SHA256 `d4b50243ea9f7a92237eaacf94629e55f2a58653156da70217bcdacbb390e950`。原 17.001 通过项目 worker 观察 `world\n3\n`；VM oracle/作者迁移仍待。T-01/T-02 未勾，全局 move、会话、旧脚本剩余输入和冻结并集均未结案。


- 2026-11-20 T-03/T-04 实施中间证据（整合 1084 前）：八个原脚本正/反例原文实测全符，t03-script-verify.json SHA256 26cc17620027355ac63c6698f2a4778cf728baa8b5204260c5937e82206efac8。trace 探针先报缺 mark 接入，修后钻石实际事件 a,b,a，叶只 init 一次。全局原草案 record .move=42、移后重读 av.trap.global-moved、回填=47，Str/List/variant/closure/object 正例同时实测。t03-global-take.json SHA256 4be480d9d6db7e835549458f0bf4b7bc22e138b07213eafa5651da9962a478fc；父侧部分选择 127 PASS 的报告 SHA256 e7e4353d85dddb60b3fda6dce29310d02adb4ed419386e17e91796441c257436。包运行入口覆盖坏编辑旧代继续、冷恢复的新代首次 42、版本隔离、取消与指令耗尽保持未初始化、释放旧代 arena 归零，命名 Rust 包用例 3/3（0.14s）。脚本工程、非入口脚本和双入口拒绝以及前向初始化、环初始化在 t04-entry-modes.json 具实际观察。原源码、AV 判定及模块诊断继续以整合后的冻结候选择重核。
- 2026-11-20 T-05 整合前选择证据：显式 debug/incremental 预构建 cargo t case-av-language-parity --prepare-only，46.530s、exit 0，只准备 av/case-parent。--run-prepared 23.859s、exit 0，141 PASS、50 SKIP，老 VM 未派发。报告 SHA256 270d150ad123be097a742b6f83e2417b70782d7e224cdf8beca85ed166d93834。21 个新 AV 用例与 9 个旧义务的每例 0.644–1.314s。目标控制 SHA256 71b88aee0185906cf81f678388c37c0707574151fcf0d05ae1676ee346d6abc5：改名的 AV 仍入选；不可用 native worker/错非 AV oracle 分别得到 SKIP 和 NOT_RUN；错 AV oracle exit 1；无有效 AV、所需检查缺失、重复 ID、非字符串虚拟路径均 exit 2。首次以非法路径拼写作公共结构负控进入 AV 拒绝而 exit 1，随后改用非字符串类型命中真正的 schema 路径。只改依赖文件正文使先前通过证据陈旧，stale 由 exit 0→1。产品 legacy-inventory 只读路径 exit 0（SHA256 ea9d42dbf2e15790818e205bc67652c61ac5b2787222977f531a46ede7916936）。根 cargo check --locked --offline -p auto-lang --tests exit 0，84.845s。
- 2026-11-20 同步 1084 到 907d558ac 后的首次联合 AV 实跑，157 PASS、69 SKIP、25 ERROR、1 FAIL，父工具 exit 2，命令内执行 26.159s。报告 SHA256 3a91047c74d6dca3d28705b821d70f957d6a7afe5342921d5285d1f257585c8c。25 ERROR 源于组合后的 span 报告形状/依赖文件跨度准入。唯一 FAIL 是 1085 object-global 的类型写入，1084 将普通对象统一为 Dyn 元素，旧 1085 全局推断仍按同质元素。后续候选修复两处接缝并加强异质对象与依赖文件位置测试。首次结果保留作回归归因。


- 2026-11-20 整合定向重放。普通包 worker 名控六项：本模块移后读 main.at 194..200、依赖模块移后读 lib.at 183..189 都是原标识符 `packet` 与 `av.trap.global-moved`。非入口脚本 lib.at 31..33、缺模块 main.at 0..17。异构对象整格替换实得 42，钻石真实轨迹 a,b,a。命名运行各 0.054–0.288s，JSON SHA256 `e16a184ba23c1bd1fa76e632868ae8cbf3be4aa5c16dc68eca8df6a9759ac5ea`。第一次扩写的对象属性写落在既有模块 store-place 边界，正式例核原合同要求的整格赋值和多标签读取。
- 2026-11-20 包定向单测以 `cargo test --locked --offline -j 2 -p ace-av --lib modules::packages::tests --no-run` 显式准备（59.36s），直接运行其 test harness 3/3（0.26s）。初始化先取得 Str 再除零的同 target 连续两次重试中，无 published 状态、帧归零且 arena.live()=0。冷恢复、版本隔离、坏编辑保旧、预算/取消与关闭保持同真执行证据。新增较深聚合的关闭单测随最终固定组核验。
- 2026-11-20 T-05 同源审计 `t05-exact-audit.json` SHA256 `94c777fa295f29aaf5379ef708221fb8469299610c50758a6f9d85e5d5354247`。9/9 原语言围栏、虚拟入口、原 VM 检查体与独立预期字节相同，canonical 仅加阶段 #ignore，17.001 的 greet_mod 依赖源文本也字节相同。22 个新增 AV 对照与 9 个继承角色共用 feature author。`auto-case legacy-inventory --test-root crates/auto-lang/test` 发现 1060 条、executed=0，九条 VM 保持原 ID 和检查并指向唯一 canonical 作者。旧历史文档未变。按一次扫描复用语料视图，清单从 35.272s 降到 2.765s，两次 SHA256 均为 `a2ae078e1f71cf6c18f9b93dc3f0bd0b06fcddc4498a9939a19fb909cc280f5c`。候选根 `cargo check --locked --offline -j 2 -p auto-lang --tests` exit 0，8.667s。
- 2026-11-20 冻结候选 `ee23e6c0609f6331c7a1085b2cd1df72c6e27cb8`（base `de373d9d2db07881eb0977d2314387330d2efa65`，依赖 auto-down 为 §4 锁定版本）。拟议 Spec 增量四项对应 §5，材料 `spec-delta-proposal.json` SHA256 `49397321c74684db8e4d651cef4f696d993ac35b3cf069a49c6f0327217cbacb`。无新增 spec 组件，部分推进 GOAL-002。


- 2026-11-20 T-06 首轮联合门 `ee23e6c0`，前置 debug/incremental 157.813s，正式 `--run-prepared` 整命令 81.741s、exit 1，八个组的 receipts SHA256 `f22e05e92c5dca3af0a0a9313ceacdac600e38309afe2bcd4df0eef8b634bea6`。AV 官方报告 184 PASS、69 SKIP、0 ERROR/FAIL，全部 31 项本包义务 PASS（386–784ms），父报告 SHA256 `3db5abf7d311eb1b57026799623992c82ec51a419369b46129351ae68af758cd`。冻结父/worker 上重做目标负控，所有判决维持，控制 SHA256 `56f71e5f538ae95228f147c5bf0ab3aa9b5648931e36a95b672fcab947f09dd7`。深聚合关闭测件 0.124s，普通包 owned 代测件 0.215s。
- 2026-11-20 T-06 首红归因及修复。新增 `auto-packages` 与 `GLOBAL_TAKE` 的固定清单需同时更新。整包 MIR 的预验将宿主 import 的单元上限误用于整个包，23 个路径在 `host.duplicate` 失败（其中 1 个是阶段误选的 Windows Linux 绑定见证 `stdlib_sys_973::sys973_t09_target_selection_linux_binds`）。恢复逐个 portable ABC 单元的原准入边界，保留跨单元全局类型/ordinal 复核。Windows 排除 token 改为明确的 `linux|wsl`，核此首轮仅误选该具名例，且在宿主声明准入阶段拒绝，未达到 Linux 目标执行。四项继承红单列。修复提交 `2947e6640b0b54ce66b6093fec6c3581ceb6be92` 仅动 AV 包目标及两个清单断言，重验 `case-av-language-parity ace-syntax ace-av`，复用其余职责组不受本三文件影响的证据。


- 2026-11-20 T-06 修复并集。预构建 13.306s、exit 0；已准备归档正式重放 61.675s、exit 1（仅两个已知 host_console 红），回执 SHA256 0b77768fad2896ac95b013662e3d30bf5f7db4c744617dea06594a66b4ee39dc。准备 manifest SHA256 184330322104cfa7719f4ac7312acab7540c8589e757bf2bff5cd67c6367ad77，冻结 source state d01fe9189afed9bc4e2c810e37a96e5a8c05c5d6fdc35ca4d69b80f84c495723，debug AV worker SHA256 4e89ea713b7ed61c1162609c59c009272cfff2ea6588f4dc10962231fb22a089，协议 avworker/1、av.closures/1、MIR lowering 5。修复回执与其余五组同依赖/夹具口径合成以下门禁。均有真实终态，无缺失。清理含在父进程统计内，未单计。

  | 固定组 | 已执行的结果及跳过 | 发现 | 执行 |
  |---|---|---:|---:|
  | case-av-language-parity | 184 PASS、69 阶段 SKIP | 预构建期的清单 | 13.988s |
  | ace-source | 930 PASS、1 继承 ABORT、1 条件排除 | 0.517s | 11.570s |
  | ace-syntax | 157 PASS、3 原有过滤 | 0.955s | 3.484s |
  | ace-semantics | 294 PASS、1 继承 FAIL、10 原有过滤 | 2.364s | 5.095s |
  | ace-av | 1458 PASS、2 继承 FAIL、54 条不入选（含 1 具名 Linux，首轮另计） | 1.738s | 38.131s |
  | ace-lowering | 9 PASS | 0.330s | 0.407s |
  | ace-artifacts | 43 PASS、1 原条件排除 | 0.512s | 6.076s |
  | auto-test-case | 157 PASS | 1.389s | 2.230s |

  四项继承红的终态与 [PLAN-1084](1084-av-parity-dynamic-objects-and-null.md) §9 及 [已知债务](../KNOWN-DEBT-AND-RISKS.md) 对齐：ace-source 的 scalar_source::deep_not_chain_reaches_checked_bundle（20,000 层，Windows 0xc00000fd）、ace-semantics 的 sema_boundary_case_files_executed_status_honest（1062 文档状态），ace-av 的 host_console::av_console_entry_selection_cannot_bypass_main_policy 及 host_console::av_console_owned_front_door_enforces_console_entry_only（帮助函数入口绕过既有政策）。修复后 ZIP、BSTR 与平台 I/O 所有入选项通过。保各原资源限制、条件与独立失败结果。Linux/WSL 属专用 Linux 环境，最终受影响门中没有执行这条绑定见证。新 Rust 用例：owned 代隔离/恢复/释放 0.272s，20,000 层 owned 聚合关闭 0.020s。

  AV 独立报告 SHA256 b1f9f3d074031e14206ac503565f435ce45fca5c6a57180eaaffce8f30af1baf。本包下列 31 项均真执行 PASS，耗时单位 ms（前缀 av.parity.1085. 或 legacy. 从显示中省略）：multifile-call 605、module-global-once 573、missing-import 561、dependency-reexport-alias 567、private-import 527、diamond-initializer-order 619、same-module-name-in-two-packages 524、cyclic-import 433、owned-global-move 895、read-moved-global 631、read-moved-dependency-global 550、reassign-moved-global 576、string-global-replacement 573、list-global 573、variant-global 583、closure-global 598、object-global 561、03_variables.001_var 601、03_variables.002_var_assign 569、03_variables.003_var_mut 563、03_variables.004_var_if 560、03_variables.005_let_binding 533、03_variables.006_let_asn_error 518、03_variables.007_var_reassignment 527、03_variables.008_simple_block 559、17_modules.001_use_fn 530、script-import 593、dependency-script-refusal 555、competing-entries 600、forward-initializer 529、initializer-cycle 443。MODULE-DIAMOND 的实轨 a,b,a，依赖移动后读取 lib.at 183..189。同 case 内原 VM 检查分别记 SKIP。正式运行命令 cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering ace-artifacts auto-test-case --run-prepared <prepared.json>。阶段准备复用脚本既有固定包/测试参数，总配置 SHA256 443d4580de083384b42072c97c55e0932c24112ec9ceb8cc1ecd740b9513f0e4。ace-av 排除 PLAN-1084 已冻结非 AV 混合命名腿，Windows 条件过滤 (^|::|_)(linux|wsl)(_|::|$)，准备工具仅 case-parent、av 与 shared execution-fixture。

- 2026-11-20 `stage: work | plan_id: PLAN-1085 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1085/auto-lang | branch: plan-1085-dev | base_commit: de373d9d2db07881eb0977d2314387330d2efa65 | code_commit: 2947e6640b0b54ce66b6093fec6c3581ceb6be92 | task_ids: T-01..T-06 | next: review`。T-01..T-06 关毕。审核待核四个 Spec 增量，冻结提议 SHA256 e49a2004679c62a269537cac5f94f421862e05267727eaae334e86d5f6f5578f，待交 IF-AV-PARITY-MODULE。候选先结束写入，在同一工作树移交独立读档复审。
- 2026-11-20 `stage: goal | plan_id: PLAN-1085 | plan_revision: r3 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1085/auto-lang | commit: 2947e6640b0b54ce66b6093fec6c3581ceb6be92 | next: review`。


- 2026-11-20 `stage: review | review_scope: final | reviewer_identity: 同会话独立读档及新反例 | plan_id: PLAN-1085 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1085/auto-lang | branch: plan-1085-dev | reviewed_commit: 2947e6640b0b54ce66b6093fec6c3581ceb6be92 | base_commit: de373d9d2db07881eb0977d2314387330d2efa65 | dependency_revisions: auto-down@72b097006f34db3a871449f10e213820d51abef6 | next: merge`。实施与复审同会话，未另派 agent。复核时按 §7 核真行为、冻结终态、原作者和独立反例。候选和兄弟依赖均为 clean，HEAD 重核未变。

  | AC | 独立复核 |
  |---|---|
  | AC-01 | PASS。断言从实际 canonical source 和父报告读出。四个草案例、钻石、两包同模块、owned 取出与回填、脚本及八份原变量例都达到源值、流或拒绝预期。异构对象真实切换值后为 42。 |
  | AC-02 | PASS。正式报告中的缺模块、私有、环、非入口脚本、竞争入口和移后读均核真阶段及原路径/范围。新增复择覆盖直接与传递重导出隐私、重复模块身份、显式 console grant 与同源无 grant。 |
  | AC-03 | PASS。读取真实 trace a,b,a 和程序代测试终态。坏编辑保旧、冷恢复首次读取和失败重复均有独立值。确认初始化的 guard 在错误、预算、取消前退出且不发布，并核 20,000 层 owned 关闭及 arena 归零。 |
  | AC-04 | PASS。九条旧 VM 全源和 check 逐字节审计，生产清单 1060 行、executed=0，原 VM 项仍指 canonical。L-37/38/41 的本包模块、变量与状态义务齐。Parser/emit 的 bootstrap 与工程组合按路线由 1095 接手。 |
  | AC-05 | PASS。逐段核 source closure→共享 CST/包绑定/Checked HIR→MIR→逐单元 ABC 准入→跨单元全局检查→实际 AV。保护原有宿主 import 容量边界，未增加二次解析器或 case 名分支。活动配置仍复用 982。 |
  | AC-06 | PASS。八个职责组原终态和修复后范围对账。22 个新文档正/反例及两个新 Rust 测件均 <10s。四项已有红按实际失败保留，修复后无本包新增红。预构建与执行分列，未见隐式构建、release 验收、调试打印或新增编译告警。 |
  | AC-FC | PASS。映射保原 ID、检查、字节、阶段 ignore。源单轴变动令旧报告陈旧。运行证据证明真正的目标选择、错 AV oracle 失败、共享格式/重复 ID 和 required 空集失败。 |
  | AC-AV-ONLY | PASS。正式工具和调用只准备 AV 与共同依赖。原 VM/非 AV 阶段检查 SKIP / NOT_RUN。首次误选的 Linux 见证单列，最终门排除并核配置。 |
  | AC-07 | PASS。逐项合约、证据与规范对齐。 |
  | AC-08 | pending。合入后重核当前规范、交付接口、归档与保护清理。 |

  复择真实 worker 六例 0.009–0.067s，无编译准备复用冻结工具。公开重导出返回 42，使用私有重导出得 `source.module-private`（调用者 main.at 0..14），显式声明私有重导出得 `source.module-reexport-private`（main.at 0..26）。首次复择把后者错误套给传递引用，按 960 的调用/声明位置还原两个输入并分别留证。根目录及 auto/ 别名撞同一模块身份给 `source.module-identity`。同工程授权 stdout 真显示 observable 且返回 42，去掉授权在 admission 给 `host.unauthorized`、零输出。JSON SHA256 `20164eedd720234c9d4dff9bdb020aa0b69072bff23285e3d1919b4937e93c8c`。清单重放 2.414s、与 §9 原字节一致，作者 lint 0.845s，184 cases 全引用准入。

  源规范 SHA256 按前言四路径依次为 `be25b0c69f2f8cc064045558534188a449f1f40d2ae63e827b1f2b6074adae84`（m2-modules）、`8624240d9aa34e2236e65d8c399b2a1eff511a1271bc73c11d5a9387f6ce2ce9`（m2-generics）、`daba77b427598021209b7b5fcd090fb23e444a69c19dab9d6c8875e93cee1892`（language-parity）、`9628ad2eb4a21b9aadeb7bf57e5a836110e8cc841da1f8f0ee2cebd8acb17044`（feature-organized-corpus）。提议 SHA256 `e49a2004679c62a269537cac5f94f421862e05267727eaae334e86d5f6f5578f`。四项增量以实现所示的范围落盘。无本包未关闭需求，四项既有失败保原 owner。Merge 需对冻结组证据作再准入、确认落主和清理。

- 2026-11-20 `stage: goal | plan_id: PLAN-1085 | plan_revision: r3 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1085/auto-lang | commit: 2947e6640b0b54ce66b6093fec6c3581ceb6be92 | next: merge`。


- 2026-11-20 `stage: merge | key: PLAN-1085:r3 | reviewed_commit: 2947e6640b0b54ce66b6093fec6c3581ceb6be92 | delivery_commit: 03ccfff2aa48916921287220a0fd2a9247d95fa8 | checkpoints: prepared, landed, canonical_verified`。复审候选后的唯一交付后继是四项规范文档提交，源码、测试配置和依赖身份未动。`git merge --ff-only plan-1085-dev` 于 de373d9d2 → 03ccfff2a 线性落主。八组冻结证据通过其边界复用，四项已有红保各自状态。两工作树落主前 reparse guard clean。主检出受影响代码路径无未提交文件，其他会话文档改动仍在。

  四项规范落主 SHA256 已与冻结提案和 Git 对象逐内容复核。`auto-av/language-parity.md` 为 `d5c1047a451d3446a1dfa89bc923291225e52822e0ad3adb60966083684cf567`，`ace-migration/m2-modules.md` 为 `d92aaff0078a9f9975b5a948fce23fd31f2859fd81a39745fbead4f444f7062b`，`ace-migration/m2-generics.md` 为 `45de75a13ae5746774196761f3dc6d2a07c1e6e2044915ef48e7416ae624220e`，`auto-test-case/feature-organized-corpus.md` 为 `8e9cf38a1090e466a7742efec4853714532a778d16a3d1bb3584b2ef74a8eecd`。验证用时 0.209s。旧账本未写入。

  `IF-AV-PARITY-MODULE` 的正式入口为 `ac-av/language-source-run-v1`，源码 package object 和依赖闭包以 SHA256 绑定，worker 协议 `avworker/1`，AV `av.closures/1`，MIR lowering `5`。单文件包选择使用 `ac.source.modules.packages/v1`。对主检出语料使用同一份已准备 debug 父/AV worker。lint 0.715s，4 项独立 T-08 smoke 精确 PASS：多文件调用 1.135s、跨文件移后重读 1.122s、异构对象全局 1.136s、原 17.001 1.118s。报告摘要 SHA256 `f94b15f936755ed66158775d5913fb4aff591ac209106aef2cebd937937f79af`。首次归档 smoke 助手用 check 名统计，把其他用例的 NOT_RUN 一并计入，改由 case+check 的绑定键核验。

  归档路径 `docs/plans/archive/1085-av-parity-modules-and-global-state.md`。迁入后校验本计划真实相对链接和四个发布规范链接，路线路径与原语料表改指归档。T-08 等待合法清理完成。主检出的 release AV worker 文件时间 2026-11-19 11:10:13，当前交付通过绑定源码的 debug 验收。


- 2026-11-20 `stage: merge | key: PLAN-1085:r3 | archive_commit: 274f86183bfb5dce7118e95ccbd517c392873bdd | checkpoints: archived, worktrees_removed | pending: artifact_scope_removed, registry_refreshed, T-08`。归档已在 master，17 个从归档向外的路径及四个规范和两处导航均核对。主、伴随工作树属于 lang-1085，清洁、身份符合并在删除前分别通过 `wt-guard.ps1`。普通主工作树 101,669 文件、22,225,226,257 字节，伴随工作树 1,294 文件、26,841,767 字节，目录和中间产物已随树移除，合入分支与空组目录已移除。

  路径对账：T-02 早期系统临时执行根 `C:/Users/zhaop/AppData/Local/Temp/tmcase-wwhk0o_3` 核实不存在。早期显式工具准备根 `C:/Users/zhaop/AppData/Local/Temp/fixed-admission-62ikuv6t` 的 JSON 指向唯一 1085 worktree，确认无 reparse point 后移除 5 文件、71,625 字节，路径不存在。其余各轮源码探针、AV store、Cargo/nextest 日志、运行输出、准备归档及临时产物在 Plan 专属材料或已移除的工作树。材料目录 owner 和在途登记确认，保护扫描 clean，15,477 文件、362,329,193 字节，即将按政策全删。

- 2026-11-20 `stage: merge | key: PLAN-1085:r3 | outcome: pass | delivery_commit: 03ccfff2aa48916921287220a0fd2a9247d95fa8 | archive_commit: 274f86183bfb5dce7118e95ccbd517c392873bdd | checkpoints: prepared, landed, canonical_verified, archived, cleaned`。两工作树及合入分支、空组目录均验证不存在。注册材料作用域 `D:/autostack/plan-artifacts/plan-1085` 在同一路径和 owner 校验且无占用后重跑 reparse guard clean，15,477 文件、362,329,193 字节于 3.051s 删除，目录不存在。无具名保留。收尾登记 `deleted-at-merge`、`cleaned`、remaining_bytes=0，以本结案提交的归档 SHA256 为索引。

- 2026-11-20 周期批量预检。上份收据为 2026-11-20T00:00Z、最高已落主 ID 1098、覆盖提交 949fdc5e1。本次 1085 虽是较小 ID，但在该覆盖提交之后落主，满足 `%5` 周期触发。按 `auto-plan-regress` 归独立主检出单实例。档后检查发现其它会话的 AutoUI `cargo rustc ... --example ace_node_desktop_window` 仍在编译，并有范围外的设计工作中改动，当前未启动批量运行，也未前移收据；该周期任务不改变本 Plan 的受影响组与归档结论。

- 2026-11-20 `stage: goal | plan_id: PLAN-1085 | plan_revision: r3 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 03ccfff2aa48916921287220a0fd2a9247d95fa8 | next: done`。T-01..T-08 完成，归档、规范和路径见上。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
