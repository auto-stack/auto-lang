---
plan_id: PLAN-1084
status: archived
completion_kind: delivered
feature_name: AV动态值、对象键与null语义补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 3
current_step: 7
total_steps: 8
depends_on: [PLAN-1078, PLAN-1079, PLAN-1082, PLAN-1083]
supersedes_spec_components: [docs/specs/ace-migration/m2-dynamic.md, docs/specs/auto-lang/vm/design/null-family.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1084] AV动态值、对象键与null语义补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`dynamic/objects-null.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-06**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补旧VM真实动态对象能力以及null正常/脚本边界，保安全资源规则。

本包交付`IF-AV-PARITY-DYNAMIC`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- 异构object字段、动态读写、键域string/int/bool、成员与下标一致、嵌套对象/列表、身份/别名与值装箱/查询/恢复；保序和缺键规则先写清。
- null/nil/None分别按语法/Option/Script规范接入；判空/??/truthy与错误类型，正常静态模式不因script放宽为任意非法算术。
- 环/共享/引用：先用实际旧case判定可达承诺；无旧合法循环承诺时保具名cycle拒。若确有required可运行循环，T-01先探测可回收方案并在本批拆子计划，不准以leak或无限GC承诺结案。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1084/auto-lang`，分支`plan-1084-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1084/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-20 施工授权：用户发出 `/goal 完成计划1084`，授权在 r3 原合同内执行、独立复审、有限修复、合入及清理。专用树 `D:/autostack/.wt/lang-1084/auto-lang`（`plan-1084-dev`），启动基线 `92b3f1842ae1fb66253c2ee5f764da2c22f5dd29`，同组只读 `auto-down@72b097006f34db3a871449f10e213820d51abef6`。材料 scope `D:/autostack/plan-artifacts/plan-1084/` 已登记。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

964源码对象字段同型；旧对象支持异构、字符串/整数/bool键与深层对象。typed Dyn和portable树已有不等于完整源码对象。旧null家族统一中有历史哨兵/异常行为差异，要对现行规范而非照nanbox布局。

既有来源与边界：

- [object_field_bool_key.case.ad](../../../crates/auto-lang/test/vm/07_objects/003_object_field_bool_key/object_field_bool_key.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [nested_object.case.ad](../../../crates/auto-lang/test/vm/07_objects/005_nested_object/nested_object.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：07_objects；13_collections中Map/Set实际API的语言派发；99_script_err的Auto对象分支；旧null guards/Dyn Rust tests。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [av.rs](../../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |
| [ownership.rs](../../../experimental/ace/crates/av-core/src/ownership.rs) | `2d2799149932028861c15c2107e5dd0b44b4330da2ccf25c01b511a2481044d7` |
| [object_data.rs](../../../crates/auto-lang-core/src/vm/object_data.rs) | `c14f3f5f1e58ad0e1f60623df38e51b4241e8610fa53bffce5037c5fcfbd965d` |

硬依赖：PLAN-1078, PLAN-1079, PLAN-1082, PLAN-1083；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

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
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [ownership.rs](../../../experimental/ace/crates/av-core/src/ownership.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [object_data.rs](../../../crates/auto-lang-core/src/vm/object_data.rs)：本包旧行为/独立oracle来源，只读，不修改旧VM语义。
- [virt_memory.rs](../../../crates/auto-lang-core/src/vm/virt_memory.rs)：本包旧行为/独立oracle来源，只读，不修改旧VM语义。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/dynamic/objects-null.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-dynamic.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-lang/vm/design/null-family.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1084-原文档清单)静态主承接 **11份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1084.heterogeneous-object` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1084.bool-and-int-keys` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1084.nested-object-alias` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1084.null-check` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `DYN-MISSING` | AC-03/AC-04 | work / T-04..05 | 不存在键、错误键型、拿错Dyn标签时禁止返回裸ID/默认0；普通错误交错误批，不把host/resource/cancel吞成catch。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `DYN-IDENTITY` | AC-03/AC-04 | work / T-04..05 | 同一对象别名写后可见、两个实例隔离，循环拒绝/批准支持的相应反例，portable freeze身份和预算保原合同。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

补充DYN-IDENTITY还必须用**同一对象**中的整数`1`、布尔`true`、文本`"1"`核key身份/重复key规则（先按旧Key与现行规范冻结），不能分别造三张map而漏掉跨类型key碰撞；异构List和object/List互嵌的真实旧可达样例也由T-01逐项补入，不能只补对象字段。

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: heterogeneous-object
id: "av.parity.1084.heterogeneous-object"
tags: ["av-language-parity", "plan-1084"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 var o = { n: 40, title: "ok", enabled: true }
 o["n"] = 42
 if o["title"] == "ok" && o["enabled"] { return o["n"] }
 return 0
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
## Case: bool-and-int-keys
id: "av.parity.1084.bool-and-int-keys"
tags: ["av-language-parity", "plan-1084"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let a = { true: 19, false: 23 }
 let b = { 1: 19, 2: 23 }
 return a.false + b[1]
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
## Case: nested-object-alias
id: "av.parity.1084.nested-object-alias"
tags: ["av-language-parity", "plan-1084"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 var o = { inner: { x: 19, y: 20 } }
 o.inner.y = 23
 return o.inner.x + o.inner.y
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
## Case: null-check
id: "av.parity.1084.null-check"
tags: ["av-language-parity", "plan-1084"]
entry-mode: "program"

### Auto
path: "main.as"
syntax: "auto-script"

```auto
fn main() int {
 let a = null
 if a == null { return 42 }
 return 0
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
| T-02 | work | T-01 | 异构object字段、动态读写、键域string/int/bool、成员与下标一致、嵌套对象/列表、身份/别名与值装箱/查询/恢复；保序和缺键规则先写清。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | null/nil/None分别按语法/Option/Script规范接入；判空/??/truthy与错误类型，正常静态模式不因script放宽为任意非法算术。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 环/共享/引用：先用实际旧case判定可达承诺；无旧合法循环承诺时保具名cycle拒。若确有required可运行循环，T-01先探测可回收方案并在本批拆子计划，不准以leak或无限GC承诺结案。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-DYNAMIC。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：已锁 11 项主承接加 8 项 1082 移交、原检查和处理层，历史正文及 owners 见 §9。
- [x] **T-02** work：对象、标签化键、异构列表、别名及装箱取回通过实际 AV；同一程序保持三个同形键的类型区分。
- [x] **T-03** work：nil/null、Option、真假与惰性求值在普通链通过；静态诊断和 .as 捕获有原 span 与独立输出 oracle。
- [x] **T-04** work：循环、缺键、错键/标签、执行配额、取消、清理和恢复在各自边界实测，保既有 portable 限制。
- [x] **T-05** work：40 份活跃作者、19 项原文与双检查映射、改源 stale 和 AV 选择负控核完。
- [x] **T-06** work：在 680188280 冻结正式 AV 并集，保留四项继承红及一次隔离重放记录。
- [x] **T-07** review：按冻结提交、原结果及独立反例核 r3 各 AC 与四条规范增量，PASS。
- [x] **T-08** merge：按已审 Spec 增量落主（`cf033adc4`）并归档（`907d558ac`）；专属材料 scope 已移除、登记已刷新，关闭。

交付已合入并归档；专属材料目录 `D:/autostack/plan-artifacts/plan-1084` 已按已核归属移除并刷新登记，T-08 关闭。

## 9. 复审记录

- 2026-11-20 `stage: merge | key: PLAN-1084:r3 | outcome: cleaned | checkpoints: artifact_scope_removed, registry_refreshed, t08_closed`。用户授权手动移除专属材料 scope `D:/autostack/plan-artifacts/plan-1084`：删除前核 9,086 个普通文件、629,949,585 字节（与 15:27 受阻记录一致）、无进程占用、无 reparse point；递归删除 2.8s 完成，删除后目录与 `plan-artifacts` 父级条目均不存在（D: 剩余 307.7 GB）。登记 `.retention/policy.json` 该条目置 `deleted-at-merge`、`cleanup_status: cleaned`、`closed: 2026-11-20`、`remaining_bytes: 0`，`last_refresh` 同步。AC-08/T-08 关闭，无具名持有。

- 2026-11-20 `stage: merge | key: PLAN-1084:r3 | outcome: blocked | delivery_commit: cf033adc4defcd6b4194ce180f6f973abed87cc2 | archive_commit: 907d558ace582f6a31404a89858a80b8aec015c3 | checkpoints: prepared, landed, canonical_verified, archived, worktrees_removed | pending: plan-artifacts deletion and final registry refresh`。主检出仍有其它会话文档改动，未纳入本计划提交。两个 worktree 的绝对路径、注册/提交、清洁状态与各自 `wt-guard.ps1` 均核为 clean，借用的 auto-down 只有 detached 依赖身份。主/伴随 worktree、plan-1084-dev 分支和已空的组目录均已移除；树内 target/.pdb/可执行文件随树移除。

  路径对账：已归档的 TestCase 系统临时运行根 `C:/Users/zhaop/AppData/Local/Temp/tmcase-ahqle6_x` 核实不存在。首次固定组输出审计在 `C:/Users/zhaop/AppData/Local/Temp/fixed-admission-18zxid_3` 留下 3 个普通文件、784 字节。其 JSON 写明唯一 1084 worktree 的 suite 与本计划 out；三文件及空目录已定向移除。最终固定组的临时、准备归档、源码探针、编译日志与结果均在唯一材料 scope 或已移除的工作树中。

  专属 scope `D:/autostack/plan-artifacts/plan-1084` 的登记 owner、绝对路径和 reparse guard 已核，目录共 9,086 个普通文件、629,949,585 字节，材料摘要已折叠。工具自动审批拒绝该精确范围的递归删除，随后也拒绝经文件校验后逐项移除的方案，出具 `blocked by policy`。两次命令都未执行，目录仍在，不属于实施问题持有。恢复时只需移除该目录、验证不存在、刷新登记，加盖 `cleaned` 并关闭 AC-08/T-08。旧台账资料未写入。

  主检出的 release AV worker 是 2026-11-19 11:10 的制品。验收和落主 smoke 使用绑定冻结代码的 debug worker。批量收据覆盖 1098，2026-11-20 00:00，核对到本次落主时没有超出该收据的整五 Plan 已归档，且不足 48 小时，批量门未到期。

- 2026-11-20 `stage: merge | key: PLAN-1084:r3 | reviewed_commit: 680188280bd5f333facc39e89bab74f6f33d7c16 | delivery_commit: cf033adc4defcd6b4194ce180f6f973abed87cc2 | checkpoints: prepared, landed, canonical_verified`。7 个实现提交和 4 个 Spec 的文档子提交线性快进到 master，交付签名与 T-06 代码、配置及输入相同。Git 更正 Spec 的换行表示，按 LF 字节重核原审文字无改；规范增量 SHA256 `c961089d6796079cb8c2af3572ecce05f3401c2d9728ab18ad727a351623d2cb`。逐 Spec 校验 worktree、主检出和 Git 对象一致。

  | canonical Spec | 落主 SHA256 |
  |---|---|
  | [AV 普通语言](../../specs/auto-av/language-parity.md) | `daba77b427598021209b7b5fcd090fb23e444a69c19dab9d6c8875e93cee1892` |
  | [ACE 动态值](../../specs/ace-migration/m2-dynamic.md) | `660e882ca4e0754e328d196c25e6463a04febbd9103820b84a38f26f87123be2` |
  | [Null 家族](../../specs/auto-lang/vm/design/null-family.md) | `87658359c01f145f14261b7fc9df0f99ba25334789da0b9ce58a3a4d9a56e432` |
  | [共享语料](../../specs/auto-test-case/feature-organized-corpus.md) | `9628ad2eb4a21b9aadeb7bf57e5a836110e8cc841da1f8f0ee2cebd8acb17044` |

  IF-AV-PARITY-DYNAMIC 入口为 `ac-av/language-source-run-v1`（`avworker/1`，`av.closures/1`，MIR lowering 5）。对主检出 `language.suite.ad` 用已冻结父端和 AV worker 实际 lint 和点选三个 smoke。类型化键与装箱 nil 捕获各为 42，缺键给 `av.dyn.object-missing`，三次 scoped exit 0。归档已核本地引用与前置任务，清理 checkpoint 待确认。

- 2026-11-20 `stage: review | review_scope: final | reviewer_identity: 同会话读档独立上下文 | plan_id: PLAN-1084 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1084/auto-lang | branch: plan-1084-dev | reviewed_commit: 680188280bd5f333facc39e89bab74f6f33d7c16 | base_commit: 384609645cfc092da66e7d005ee1b052e0a547f2 | dependency_revisions: 1078/1079/1082/1083 archived | next: merge`。复审基于冻结 diff、独立输入、固定组原终态及原始作者文本。同一会话承担 work 与 review，未调用外部评审者。清洁树与 HEAD 重核一致，七个代码提交接基等价。F-1084-01 关闭。四项继承红均有父树证据，Windows 深链见 KNOWN-DEBT，ZIP 首红与隔离通过分别保留。

  | AC | 复核证据与判决 |
  |---|---|
  | AC-01 | PASS。父报告 42 份本包活跃用例含原四例、异构列表、键标签、nil 核对及 19 份原样源，结果和双流有独立 oracle。 |
  | AC-02 | PASS。绑定、.as TypeError、缺键/错标签/循环均有原虚拟文件和半开区间。复择右侧盒装 nil 的减法，实测左操作数打印一次，catch 精确 TypeError，结果 42。 |
  | AC-03 | PASS。冻结 Rust 的增长预检、帧与 arena 清空、取消与同 ABC 重试、间接环及 v1/v2 往返证据逐项核验。 |
  | AC-04 | PASS。实际运行原 legacy reader 的只读探针，原源、检查 ID、结果、stdout/stderr 和 completion 19/19 等价，实际路径为 canonical 文档。 |
  | AC-05 | PASS。diff 逐段指向共享前门、受检 HIR/MIR、ABC 与 AV 执行，分支依据类型、模式及 recipe。 |
  | AC-06 | PASS。8 个受影响组有冻结结果，四项继承红及一次隔离重放分账。42 份作者检查和 5 个新 Rust 见证均 <10s。 |
  | AC-FC | PASS。对读原源和原 oracle、改正文触发 stale、重命名检查仍按真实目标选择，缺 required/全 ignore/重复 ID/格式坏均红。 |
  | AC-AV-ONLY | PASS。复择用例的 AC 缺 worker 加阶段 ignore，报告 1 AV PASS、1 SKIP，stale 只核所选 AV。 |
  | AC-08 | T-08 落主、规范和清理。 |

  独立反例报告 SHA256 `6779ae27d70c969b5fa78f48a7034d53b5037c501b4dad855f1ab6ec7bad5ddc`。19 项 legacy reader 只读探针日志 SHA256 `d245fd2c793879b5d07a4d855fa8808451bd1d5eb72eff192492b795c847d61b`。规范对读精确四文件见 frontmatter，冻结增量 SHA256 `155bbc7a625b362bf83771a24325a59cda70c50aa6341acc5a250bb63bd103dd`。无新增未处理告警、临时路径依赖或未完成合同内义务。收尾需复核代码与规范同版落主、归档及清理。

- 2026-11-20 `stage: work | plan_id: PLAN-1084 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1084/auto-lang | branch: plan-1084-dev | base_commit: 384609645cfc092da66e7d005ee1b052e0a547f2 | code_commit: 680188280bd5f333facc39e89bab74f6f33d7c16 | task_ids: T-01..T-06 | next: review`。主线 AGE 语法两提交前进后七条 range-diff 等价。正式 AV 阶段使用既有 `fixed-tests.py` 准备并用 `--run-prepared` 复放，保原包参数与样本，具名排除混合原生检查和 Windows 不适用腿。显式工具为 debug case-parent/av 和通用 debug execution-fixture。准备 58.578s，等待占用 72.8s（准备）与 48.6s（执行）。执行和对账合计 138.600s。原始并集 exit 1，SHA256 `7754f9d2eb5f89424ec8aef7568f730ae261232c7ecbdd93aff7b68817f3b8f0`。

  | 固定组 | 原始实际结果 | 准备归档 | 发现 | 执行 |
  |---|---|---:|---:|---:|
  | case-av-language-parity | 153 PASS、60 SKIP，本包 42/42 | 显式工具准备 | 随清单预检 | 16.477s |
  | ace-source | 930 PASS、1 ABORT、1 条件排除 | 5.422s | 0.507s | 16.956s |
  | ace-syntax | 157 PASS | 5.117s | 1.735s | 5.235s |
  | ace-semantics | 294 PASS、1 FAIL、10 选择排除 | 4.470s | 3.484s | 7.594s |
  | ace-av | 首轮 1456 PASS、3 FAIL、53 选择排除，ZIP 单例隔离重放 PASS | 12.152s | 2.244s | 70.115s |
  | ace-lowering | 9 PASS | 3.897s | 0.531s | 0.456s |
  | ace-artifacts | 43 PASS、1 条件排除 | 2.207s | 0.554s | 6.477s |
  | auto-test-case | 157 PASS | 2.528s | 1.710s | 2.570s |

  清理包含于进程和全命令 wall，未单独计时。四项继承红包括 `scalar_source::deep_not_chain_reaches_checked_bundle`，原 20,000 层样本在候选与父提交均为 Windows `0xc00000fd`，2 MiB 观察在 parse 完成前溢出，父树日志 SHA256 `7322ac1c0057e243868723e15e782df1c340bdc25aa75a5bd4f621091df6dc5a`；另三项沿用 1082/1083 同输入父树收据，为 PLAN-1062 的 `sema_boundary_case_files_executed_status_honest` 和 `host_console` 的 `av_console_entry_selection_cannot_bypass_main_policy`、`av_console_owned_front_door_enforces_console_entry_only`。同候选 `kv_zip_1064::v03_zip_negative_corrupt_bomb_and_preserve_av` 首轮为 `ExecutionWall`（11.712s），同 nextest archive 单实例原样通过（2.991s），重放日志 SHA256 `ea72cb81c5d3238a8e4558fc263295551f6e59ffa4b08be6932f4eb527f084d0`。

  本包 42 例最大 1.062s（duplicate-int-key），list-object-cycle 与 boxed-nil-arithmetic-script-catch 均 0.932s。新增 Rust 见证为间接环 0.171s、对象增长预检和清理 0.224s、便携标签回放 0.112s、v1 兼容 0.071s、装箱列表克隆与恢复 0.135s。无新增未处理告警或临时调试输出。

- 2026-11-20 冻结准备期补充反例 `F-1084-01`（AC-02/03，T-03/04/05）：`51e1a2a4f` 在 `.as` 从对象读取 nil 后执行加法，实际先发生 `av.trap.dyn-type`（原 87..101），跳过语言 catch。停止当时本 Plan 的准备进程（未执行并集），在同树修复 `fde0ec968`，将 Nil 标签检查放在算术的 DynTake 前并保操作数源序。原反例经真实 worker 返回 42（0.042s），同检查的 source-static/未装箱用例继续由 canonical 语料验证。独立重复整数键控制在原 45..50 绑定拒 `source.object-duplicate`（0.037s）。两条加入同一活跃作者，最终本包 42 份 Case。准备重启前机器另有 AutoUI 示例 build 与 PLAN-1046 AAC prepare-native，按工作技能等待占用退出。

- 2026-11-20 T-01..05 定向收据。异构列表的初始化、追加、嵌套对象和共享写实际返回 42；自插和 object/List 间接回路用真实普通 worker 给 `av.dyn.object-cycle`。无已承接的合法环源义务；v1/v2 portable 冻结支持的纯值树，继续拒共享图、活 provider 和嵌套 Dyn 包装，普通源通过受检 DynPack/DynIs/DynTake 装箱与取回。Rust `public_dynamic::tests` 35/35（0.23s），含错标签恢复完好的原值、深复制解除共享、v1/v2 标签与冻结回放。独立增长用例 `ordinary_object_growth_keeps_resource_stop_outside_script_catch` 1/1（0.04s），13,640 字节门给 `SlotBytes` 且停止于对象写预检，帧数和 Str arena 存活数各为 0；预取消为 Cancelled，重试同 ABC 得 79。首试受语法形和 root frame 先触发影响，修订测件使反例确实跨过写前门。构建日志 SHA256 `9e53c484967ea42a351067bb59319dc24bcc3e4023abbbd7d1dc1d1ca939ae5d`。
- 2026-11-20 T-03/T-05：公共语料 151 PASS、60 SKIP、exit 0（40 份本包 canonical Case）。同一算式的 `main.at` `source.nil-op` 与 `main.as` TypeError 均为原 23..31；p4c 的装箱 nil 调用路径 `p4c_call_dyn_null.as` 161..165，stdout `got\n`，随后语言错误。p5b/p9 捕获文字及 p7 两行 `None` 由父侧 exact oracle 判定。普通型别 `None` 依 Option 判别（直接以 `o == None` 对比旧整枚举表示的 p13 是历史观察，当前用类型化 `None` 和 `??` 见证）。初次 .as 作者用例被父接面旧 .at 白名单拒，按真实 .as 源身份修正后通过。报告 SHA256 `6038c199414dd5b3645808fe5503b7879262bac4cfef4b9f72377420d4d0fc76`。
- 2026-11-20 V-FC-SOURCE/MIGRATE：按完整原 Auto 围栏字节、SHA16、原检查 ID、映射以及共享 source SHA256 核 19/19，旧检查状态 SKIP，新 AV 检查 RUN。主动改一处共享正文使原报告通过正式 stale 命令观测 `case_file`+`source` 陈旧，exit 1；未修改的原报告 exit 0。首次 stale 误走 sema worker 的既存默认分支，停掉本 Plan 发出的备用构建后补语言 worker 身份路由，重放修正。迁移/陈旧账 SHA256 `5dc4e3aee0d687bc6af68db8948c4dee0c349b839525d6f81ff8b5f33f45190b`。当前工作树先接到 master `dcd4fdafd`，五个代码提交 range-diff 全等，后收 staleness 补丁，冻结 `51e1a2a4f`。冻结固定组仍待执行。

- 2026-11-20 在途提交 `d2f5f749c`：引用对象和列表的写入在改动前检查间接可达环；低层真指令验证过两节点环。实际 `ace-av --test conformance dynamic_objects_964` 13 绿（0.06s），`plan1082_collections_places` 17 绿（0.25s）。已核的 07_objects 和 1082 移交 15_nested_mutation 源没有合法循环义务。
- 2026-11-20 在途提交 `2ea9cd167`：普通源 Dyn 装箱/检查提取、带标签对象、混合容器索引、脚本末值和共享作者读取已通。19 个原始 Auto 源保持原独立期望，实际 18 成功（含精确文本 `auto`/`Bob`、有序对象含换行以及 assert 后 stdout），1 个按原预期给 `av.trap.list-index`。逐项 JSON SHA256 `5043e5aa21acf787cfa7cb9bd197fe115d62eb1046f06b7576fb5e0aadd366c2`。99_script_err.05 的 Python/宿主捕获完整行交原 1094 owner，HashMap/HashSet 的容器 API 行仍由原标准库容器职责承接。
- 2026-11-20 T-05 在途：正式 `case-av-language-parity` 在冻结准备/仅运行路径实跑 137 PASS、60 SKIP，工具只有 `case-parent` 和 `av`，执行 10.482s，exit 0，报告 SHA256 `15dfae6e5aaeaf6800af1266524b4870575e20cf6cbee3121769364050a2a450`。首次父出口因旧整套分流规则把 60 未选记为 NOT_RUN 而 exit 1，显式 target scope 重放已按所选判定。负控实际分别得到转命名 AV + 缺 native worker/错 oracle 的检查为 SKIP（exit 0），解除 native ignore 仍 NOT_RUN（exit 0），改错 AV oracle 报 mismatch（exit 1），全部忽略、缺 required、重复 ID、公共格式坏均 exit 2。负控 JSON SHA256 `317ed0d52397cbdfb72ed6ca9ee383ace861cc641bbcbd283d58d1c37af62c55`。`cargo check --locked --offline -p auto-lang --tests` exit 0（1m13s），验证 canonical legacy reader 可编译。完整并集及收尾未执行。

- 2026-11-20 T-02 类型化键增量 `44f62edba`（仍在途）：同一普通 AV 对象中 int `1`、bool `true`、text `"1"` 分别判等；保源序和字符串借用读取，写入新文本键才拷入 arena。原 07.002、07.003 精确源分别实际返回 4，草案 `bool-and-int-keys` 与 `nested-object-alias` 返回 42；canonical 新 `int-bool-text-key-identity` 含负键与定向替换，实际 42、双流空，0.075s。错误浮点键真绑定拒 `source.object-key`，缺键 `av.dyn.object-missing`。T-02 worker 直跑 `t02-key-final.json` SHA256 `a7cc33538f64dede80c998587f11f8300fea9df0fecc42244fa0b4475d733a5d`。
  `ace-av-core --test conformance public_dynamic::tests::` 34 绿（执行 0.14s，进程 wall 0.171s），v1 原样字符串对象往返 0.0112s，v2 标签/顺序/嵌套对象往返与伪造 v1 拒绝 0.0772s；`ace-av --test conformance dynamic_objects_964` 12 绿（执行 0.06s，准备 57.65s）；`ace-core --test conformance` 定向 `plan981_grammar_scope::p981_grammar_ids_match_the_recorded_frozen_values` 1 绿（0.32s）。`--report-time` 在 stable 被拒，改为计时实际准备好的单测可执行文件；grammar `--exact=false` 参数被拒，按准备好的可执行文件及全 ID、`--exact` 重放成功。真实模块并集待 T-06。新 portable v2 仅当含非文本键发出，旧 v1 布局保留；当前可移植对象值仍限可自描述的叶/对象，异构容器与整返回观察未完成。

- 2026-11-20 T-01/T-02 在途收据。基线 `92b3f1842`，本树 `cargo build --locked --offline -p ace-av --bin av-worker`（`RUSTC_WRAPPER=''`、`CARGO_INCREMENTAL=1`，debug 32.09s、exit 0；首试 sccache 拒 incremental，exit 101，已按 wrapper 配置恢复）。在同树 `av-worker case-serve` 直跑 `ac-av/language-source-run-v1`，46 项真实观察；11 份主承接原文档只有 `.010_map_bracket_write` 返回 600，原 `.001/.006_inline_map` 拒异构，`.002/.003` 词法/语法拒，`.007/.008` 原输入在语法拒。4 份草案只有 `null-check` 返回 42。`null ?? 42` 与条件位 null 绑定拒；普通 `null + 1` 为 `source.nil-op`，`-1 + 43` 返回 42。缺键 control 为 `av.dyn.object-missing`。材料 `t01-av-baseline.json` SHA256 `a9454d654daac0ef1248a76e2c40481af0ed664a7398ecabbb0eaec282dce66`。
  交叉来源核验：PLAN-1082 移交八份 `15_nested_mutation` 对象义务（001/003/006/007/008/011/012/014），与 11 份主承接逐项保原 oracle；`13_collections.001–013` 依赖 HashMap/HashSet 标准库定义，PLAN-1082 已交独立 stdlib 容器 owner；`99_script_err.05` 的 `use.py builtins: list` 与 `try/catch` 原样综合例归 PLAN-1094，1084 核 Auto 对象本地分支并保综合引用。旧 `ValueKey` 以 `Str/Int/Bool` 枚举分别判等，禁止将 `1`、`true`、`"1"` 合并。检查的旧文档未找到合法循环用例；964 已交 `av.dyn.object-cycle` 防御，当前运行层仅检测直接同 cell，间接环与跨容器环仍待核。
  首个部分提交 `c99b36efa`：普通 Auto 对象点读、点写复用 Checked HIR/MIR/ABC 对象指令，增加 canonical `av.parity.1084.nested-object-alias`。工作树重建 worker debug 5.85s，真实复放 005=10、006_nested=20、009=600、010=600、15.001/006=30、草案 alias=42（0.039s），缺键原 trap 保持。`t02-member.json` SHA256 `271ed64db70031d71297958fadebc9d7518609e83bfefa7024241f0edabdeb65`。固定组未运行；T-01 输入/资源审计仍在途，T-02 未完成。

- 2026-11-20 `stage: goal | plan_id: PLAN-1084 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1084/auto-lang | commit: 92b3f1842 | next: work`。用户整链授权见 §4；依赖 1078/1079/1082/1083 均已归档，进入 T-01，进度 0/8。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
