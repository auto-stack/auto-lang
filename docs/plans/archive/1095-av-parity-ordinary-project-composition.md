---
plan_id: PLAN-1095
feature_name: AV普通工程、会话与领域语言组合收口
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-22
plan_revision: 3
current_step: 8
total_steps: 8
status: archived
depends_on: [PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1081, PLAN-1082, PLAN-1083, PLAN-1084, PLAN-1085, PLAN-1086, PLAN-1087, PLAN-1088, PLAN-1089, PLAN-1090, PLAN-1091, PLAN-1092, PLAN-1093, PLAN-1094]
supersedes_spec_components: [docs/specs/ace-migration/m2-facade.md, docs/specs/ace-migration/m2-modules.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1095] AV普通工程、会话与领域语言组合收口

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`composition/ordinary-projects.case.ad`、`domain/node-values.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-17**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

消除功能只能分profile运行的组合墙，在真实普通AV入口证明模块/语言/库/领域描述同源工作。

本包交付`IF-AV-PARITY-PROJECT`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- 普通Auto profile统一生产路径：类型/数值/闭包/动态/泛型/错误/CT/任务/FFI/标准库在同包、同session可组合；不凭内容轮流试profile，不调用旧parser/VM。
- persistent REPL/在线编辑只验语言状态、初始化/失效/恢复与已交970/969入口，完整产品替换仍归982–987。
- Node语句/表达式、属性值/事件handler/配置文档消费普通Checked HIR与公共语义；复用979/981/982producer，只验CPU值/业务轨迹与前门，不启动UI/GPU，也不声称全部demo通过。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1095/auto-lang`，分支`plan-1095-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1095/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

SourceProfile当前有Core到Scheduler、Age*/ConfigDocument等显式裁剪；多个单独profile通过不等同一工程可组合。979/981/982已有领域producer，各自不应再写一套普通表达式语义。

既有来源与边界：

- [plan981_original_g01.rs](../../experimental/ace/crates/core/tests/cases/plan981_original_g01.rs)（既有输入/oracle；当前适用证据须按身份重核）。
- [a2r-project.document.case.ad](../../crates/auto-lang/test/cases/pilot/a2r-project.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：99_bootstrap；99_idiom_probe；99_idiom2；99_misc；99_p588debt；99_plan230；99_plan231；99_plan664；99_slice；composition；vm_engine_face；剩余普通语言Rust/外部切片。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [ace_entry.rs](../../crates/auto-lang-core/src/ace_entry.rs) | `6933cdd96917cc234ebe666a4b2b5783e59a4fd7217d109492dd08621b93aa51` |
| [adapter.rs](../../experimental/ace/crates/core/src/source/adapter.rs) | `915cc3e6c62d37f5bd55477d198a68279531760ed72573bcf7d9a1728f6ee5a3` |
| [age_package.rs](../../experimental/ace/crates/core/src/source/age_package.rs) | `347f31c8b91f32ee09633412089811f4a798a1a6a968a82ec559818bd6f7ca27` |
| [config_document.rs](../../experimental/ace/crates/core/src/source/config_document.rs) | `20af68d92691837dc9f197b85d3d2137e4923c1bb0b3b272a58ccc58ba11abb1` |

硬依赖：PLAN-1078, PLAN-1079, PLAN-1080, PLAN-1081, PLAN-1082, PLAN-1083, PLAN-1084, PLAN-1085, PLAN-1086, PLAN-1087, PLAN-1088, PLAN-1089, PLAN-1090, PLAN-1091, PLAN-1092, PLAN-1093, PLAN-1094；实际外部接口：969/970普通会话、979/981领域、982 CONFIG各实际接口。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [ace_entry.rs](../../crates/auto-lang-core/src/ace_entry.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [adapter.rs](../../experimental/ace/crates/core/src/source/adapter.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [age_package.rs](../../experimental/ace/crates/core/src/source/age_package.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [config_document.rs](../../experimental/ace/crates/core/src/source/config_document.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [packages_store.rs](../../experimental/ace/crates/source-query/src/modules/packages_store.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/composition/ordinary-projects.case.ad`、`crates/auto-lang/test/cases/language/domain/node-values.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-facade.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-modules.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1095-原文档清单)静态主承接 **129份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1095.module-closure-generic` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1095.task-language-composition` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1095.cte-and-ordinary-library` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1095.node-ordinary-coexist` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `PROJECT-RESTORE` | AC-03/AC-04 | work / T-04..05 | 同项目切换实际函数body/依赖/属性/CT/target，坏编辑→旧可用→恢复首次运行与冷编一致；两个session不能串state/cache。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `DOMAIN-NO-DUPLICATE` | AC-03/AC-04 | work / T-04..05 | 领域原源通过统一普通表达式checker；CPU字段/事件调用真实AV业务，原SDK/权限/错误来源保持；不等待UI1075整包，不代签其GPU/完整产品。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

领域语言完整性：T-01除Node Hello外，还要对旧`Expr::Node`/`Stmt::Node`实际可达输入核通用Node名称、参数/属性值、子节点层级、Node值的绑定/传参/返回与现行AutoData/领域承接；不得把单个UI schema的text例子当所有Node语言已支持。保979/981/982真实producer，OnEvents/Scene/NavCall旧parser-only部分按L表分账，不借此新增renderer。

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: module-closure-generic
id: "av.parity.1095.module-closure-generic"
tags: ["av-language-parity", "plan-1095"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use lib.id
fn twice(n int) int { return n * 2 }
fn main() int {
 let f = twice
 let xs = [id(19), id(23)]
 if f(xs[0]) + f(xs[1]) == 84 { return 42 }; return 0
}
```

#### lib.at

```auto
pub fn id<T>(x T) T { return x }
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
## Case: task-language-composition
id: "av.parity.1095.task-language-composition"
tags: ["av-language-parity", "plan-1095"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
task Q { on { n int -> { reply(n + 1) } } }
fn get() int {
 let h = Task.spawn("Q", 1)
 let v = await h.ask(41)
 h.stop()
 return v
}
fn main() int {
 let meta = { label: "ready", enabled: true }
 try { if meta.enabled { return get() }; return 0 } finally { }
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
## Case: cte-and-ordinary-library
id: "av.parity.1095.cte-and-ordinary-library"
tags: ["av-language-parity", "plan-1095"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn scale<T>(n T) T { return n + n }
fn main() int {
 let seed = #{ scale(21) }
 let xs = [seed - 1, seed]
 return xs.filter(x => x == 42)[0]
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
## Case: node-ordinary-coexist
id: "av.parity.1095.node-ordinary-coexist"
tags: ["av-language-parity", "plan-1095"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn label() str { return "你好" }
node Hello { view { text label() {} } }
fn main() int { return 42 }
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
不只检查main=42：父侧消费979/981已核CPU描述符，必须得到同一源的文本“你好”；本例不运行renderer。
`````

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering ace-artifacts ace-host ace-m2-build
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
| T-02 | work | T-01 | 普通Auto profile统一生产路径：类型/数值/闭包/动态/泛型/错误/CT/任务/FFI/标准库在同包、同session可组合；不凭内容轮流试profile，不调用旧parser/VM。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | persistent REPL/在线编辑只验语言状态、初始化/失效/恢复与已交970/969入口，完整产品替换仍归982–987。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | Node语句/表达式、属性值/事件handler/配置文档消费普通Checked HIR与公共语义；复用979/981/982producer，只验CPU值/业务轨迹与前门，不启动UI/GPU，也不声称全部demo通过。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-PROJECT。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：冻结旧/新实际输入、规范差异与边界；129 份旧文档清单核角色；canonical 位置定案；有界全链可行性探测（见 §9 work 收据）。
- [x] **T-02** work：普通 Auto profile 统一生产路径：同包同 session 组合（工程 fn 值、record 字面量、try/finally、task、CT+filter）；建 canonical 作者与 legacy-map 行（见 §9）。
- [x] **T-03** work：PROJECT-RESTORE 补充 ID：坏编辑→恢复冷等值、双 store 隔离（见 §9）。
- [x] **T-04** work：Node 面接入普通链 + 979 producer CPU 观察 + `nodes` oracle（C4 全链；见 §9）。
- [x] **T-05** work：§6 草案落实为唯一活跃作者（composition/domain 两文件 + suite）；129 行迁移；故意红元控制（见 §9）。
- [x] **T-06** work：冻结候选提交；固定组执行并核子项/资源/源身份；规范增量四份；IF-AV-PARITY-PROJECT（见 §9）。
- [x] **T-07** review：独立逐 AC 核验通过，见 §9 复审收据。
- [x] **T-08** merge：落主/归档/清理完成，见 §9 merge 收据。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- `stage: goal | plan_id: PLAN-1095 | plan_revision: r3 | round: final | from: merge | to: done | status: archived | outcome: pass | worktree: 已移除 | commit: f70320f5c（delivery）+ 12aeafcd6（goal 收据） | next: done`。用户 `完成计划1995→1095` 全链授权闭环：work（T-01..T-06）→ review（final pass）→ merge（landed/canonical_verified/archived/cleaned）。交付 commit f70320f5c 在主检出默认分支线性快进落主；四份规范增量（language-parity/m2-facade/m2-modules/feature-organized-corpus §PLAN-1095）逐字读回核验；计划归档 `docs/plans/archive/1095-av-parity-ordinary-project-composition.md`（status archived，8/8）；wt-guard clean 后工作树/分支/组目录移除，plan-artifacts/plan-1095 整目录按 delete-at-merge 删除，登记表 `.retention/policy.json` 刷新；批回归不触发（1095<1109 且收据<48h）。
- 2026-11-22 merge 收据：`stage: merge | plan_id: PLAN-1095 | plan_revision: r3 | outcome: pass | delivery_commit: f70320f5c | archive_commit: 65ce06b0e | next: done`。
  - `prepared`：复审基线 580b6d185 + 卫生 f70320f5c（语料尾随空白，语义零改动）；规范增量四份已在 worktree 提交。
  - `landed`：master 未前进（仍 b260b6e65），`git merge --ff-only plan-1095-dev` 线性快进至 f70320f5c（无 merge commit）。
  - `canonical_verified`：四份规范落主，SHA-256 前16：language-parity.md `f81641af…`、m2-facade.md `4fb6836f…`、m2-modules.md `475e7695…`、feature-organized-corpus.md `ae12efb8…`（均含 PLAN-1095 增量节，逐字读回确认）。
  - `archived`：`docs/plans/archive/1095-av-parity-ordinary-project-composition.md`，status archived，current_step 8/8（T-01..T-08 全 done；AC 全勾选），delivered。
  - `cleaned`：wt-guard clean 后 `git worktree remove` + `git branch -d plan-1095-dev`；`D:/autostack/plan-artifacts/plan-1095/` 整目录按 delete-at-merge 删除（证据已折入本计划 §9 各收据；构建中间产物随 worktree 删除，不回收）。登记表 `.retention/policy.json` 刷新。
  - 批回归：`.last-batch-regression.json` last_covered_plan_id 检查（见清理后核验）。
  - 落主后主检出仅存其它会话的 docs 簿记 WIP（未触碰）。
- 2026-11-22 final 独立复审（同实施会话，从工件重建结论）：`stage: review | review_scope: final | plan_id: PLAN-1095 | plan_revision: r3 | reviewed_commit: 580b6d185（+ 卫生 f70320f5c：语料尾随空白修剪，语义零改动） | base_commit: b260b6e65 | worktree: D:/autostack/.wt/lang-1095/auto-lang | branch: plan-1095-dev | outcome: pass | next: merge`。复审者在实施会话内，按工件重建（不采实施摘要）：
  | AC | 结果 | 核验 |
  |---|---|---|
  | AC-AV-ONLY | pass | 四个新 case 仅 av 检查真实执行；非 AV 检查在语料中不声明（既有 SKIP(IGNORE) 机制分列）；AV required 行全部有 actual。 |
  | AC-FC | pass | 唯一活跃作者（composition/domain 各一文件，suite 单引用）；129 行迁移表 12 列格式全对、owner=PLAN-1095、原 oracle/条件/SHA16 保留；无双活作者。 |
  | AC-01 | pass | 四正例经真实 case-runner 4/4 PASS（module-closure-generic 工程 fn 值、task 组合、CT+filter、node+main）；值 42、双流空。 |
  | AC-02 | pass | 工程 fn 值反例（`let f = main`）`source.fn-as-value`；CT 除零坏编辑 `cte.trap/` 带源 span；错误 nodes oracle 精确红（`nodes[texts] mismatch`）。 |
  | AC-03 | pass | PROJECT-RESTORE 真实断言（坏编辑→恢复缓存命中、值/hash 冷等值、双 store ct_execute=1/ct_hits=0 隔离）；DOMAIN-NO-DUPLICATE 由 node 观察 + 变异臂（改函数体→观察文本随之变）覆盖。 |
  | AC-04 | pass | 129 行映射全部 owner=PLAN-1095、状态 mapped、canonical 按语言族；AV required 行均有 actual。 |
  | AC-05 | pass | 共享前门（compile_language_source / compile_package_product_auto_prepared）＋CST 投影；node 观察复用 979 producer；无 case 名特判/第二 parser；legacy dispatch=0。 |
  | AC-06 | pass | 固定组：ace-syntax 159、ace-semantics 295、ace-lowering 9、ace-artifacts 43、ace-m2-build 全 PASS；ace-source 977（5 条 text_record_source=1092 已登记基面红 D-1055/D-1088，清单逐字一致）；ace-av 1581（12 条与 1092 继承红清单逐字一致，非本包）；ace-host 预存编译阻塞（master 同失败）。零新增红；5 条新永久回归 ≤0.3s。`git diff --check` 干净；无新增告警/调试输出。 |
  | AC-07 | pass | 本复审逐 AC 对源码/提交/消费者；health check 通过。 |
  | AC-08 | — | merge 阶段（T-08）落主后核来源身份。 |
  遗漏/延后扫描：无未批准缩减。C3 无类型闭包归一化为 typed lambda（1081/1082 面规则既有规范化，plan §6 Notes 已记录）；ace-host 编译阻塞为主检出预存（Plan 1084 Selection.target 未同步测试文件），不阻本包 AV 出口。规范增量四份（language-parity/m2-facade/m2-modules/feature-organized-corpus §PLAN-1095）在 worktree 提交内，与 supersedes_spec_components 一致；merge 落主后按来源核验。
- 2026-11-22 work 完成收据（T-01..T-06）：`stage: work | plan_id: PLAN-1095 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1095/auto-lang | branch: plan-1095-dev | base: b260b6e65 | commits: b0474f13b / 991f3769f / 580b6d185 | task_ids: T-01..T-06 complete（6/8） | next: review`。授权：用户 `完成计划1995→1095` 指令即全链授权。
  **T-01**：指纹漂移复核（ace_entry `a5e6f2ca…`、adapter `3253da90…`、age_package `6c79e6a5…`、config_document `d3591d08…`、packages_store `0f740319…`）；129 份旧 VM 文档清单逐项核角色；canonical 定案 `language/composition/ordinary-projects.case.ad` + `language/domain/node-values.case.ad`；有界探测（探针已删，输出 `D:/autostack/plan-artifacts/plan-1095/t01-probe-faces.txt`）定位四个正例的阻塞点（C2 原样通过；C1=工程绑定 fn 值缺口；C3=无类型闭包面规则；C4=node 未入普通面）。
  **T-02**：`packages/binding.rs` Auto-language 通道允许本地自由 fn 作值（工程/单源一致，CreateClosure 通道）；multi-file+import+fn 值+泛型+列表+索引 全链 42（`p1095_project_local_free_fn_as_value_with_import`）。C3 按 1081/1082 既有规范化把无类型闭包归一为 `(x int) =>`（面规则：无 lambda 参数推断）。
  **T-03**：`p1095_project_restore_cold_equal_and_sessions_do_not_share`——坏编辑（CT 除零）具名 trap，恢复缓存命中且与冷编值/hash 一致；第二 store 各自冷执行（ct_execute=1、ct_hits=0），双 session 不串 state/cache。
  **T-04**：`auto` profile 接纳 H0 node（grammar age-node-item/head/body/text-item 门控增 auto + contextual name 增 auto + age-text-item 双 pratt 臂：wide-ops vs aggregates-ops）；CST 投影在 auto 面产出 `Item::Node`（名字+span，体不重投影）；手写 parser 同步承认（平衡花括号，差分 oracle 用）；worker `step.nodes` 经 979 producer（`compile_age_h0`+共享执行链）真执行 view 文本；父侧 `nodes` oracle（author/contract/ir/judge 接线）。**故意红元控制**：错误 nodes oracle → `nodes[texts] mismatch: expected 世界 actual 你好` 精确红，恢复后绿。
  **T-05**：canonical 两文件 + `language.suite.ad` 引用；legacy-map.tsv 增 129 行（owner=PLAN-1095，原 case/check/条件/oracle/SHA16 保留，按语言族映射 composition/domain 及既有 canonical 面）；四个正例经真实 case-runner 全 PASS（`av.parity.1095.*` 4/4），非 AV 检查 SKIP(IGNORE) 分列。
  **T-06 固定组**（`--prepare-only --run-prepared`，共享语法/绑定/IR 改动组）：ace-syntax 159、ace-semantics 295、ace-lowering 9、ace-artifacts 43、ace-m2-build 全 PASS；ace-source 977（5 条 text_record_source 为 1092 已登记基面红 D-1055/D-1088，清单逐字一致）；ace-av 1581（12 条与 1092 继承红清单逐字一致：av_method_modes/av_method_owned/av_owned_first_case/av_owned_recipe/av_test_batch/component_event_sessions_1106/resource_915×3/av-core layout；首轮另有 kv_zip 2 条=既有墙钟归因，重放绿）；ace-host 预存编译阻塞（master 同失败：Selection.target 由 1084 引入后 native_protocol.rs 未更新，非本包）。零新增红。新永久回归 `plan1095_ordinary_composition.rs` 5 条全部 ≤0.3s（10s 门内）。grammar 身份：age-h0/age-component/age-package 因共享门控表增 auto 逐字节漂移，`plan981_grammar_scope.rs` 按 2026-11-22 具名重冻结（AgeH0 62332a74…、AgeComponent e310202b…、AgePackage 3555cdf0…），重冻结后单测绿。
  **规范增量（worktree 内已提交）**：language-parity.md §PLAN-1095、m2-facade.md §PLAN-1095、m2-modules.md §PLAN-1095、feature-organized-corpus.md §PLAN-1095。待交接口 `IF-AV-PARITY-PROJECT` 绑定普通源/工程、node CPU 观察（979 producer）、`avworker/1.0`、`av.closures/1`、MIR lowering 9、4 条 canonical case 与逐 case 真实结果。
- 2026-11-22 work 阶段收据（T-01）：`stage: work | plan_id: PLAN-1095 | plan_revision: r3 | outcome: in-progress | worktree: D:/autostack/.wt/lang-1095/auto-lang | branch: plan-1095-dev | base: b260b6e65 | task_ids: T-01 complete（1/8） | next: T-02`。授权：用户 `完成计划1995→1095` 指令即全链授权（work/review/merge）。依赖 1078–1094 全部 archived；`case-av-language-parity` 固定组（test-groups.toml:690）与 `ac-av/language-source-run-v1` recipe（language_source_worker.rs）均已交付。
  **输入冻结**：§4 五指纹全部漂移复核——ace_entry.rs `a5e6f2ca145d067f`、adapter.rs `3253da907e8ad081`、age_package.rs `6c79e6a5cd2bf4f3`、config_document.rs `d3591d088b231555`、packages_store.rs `0f7403194f20f982`。129 份旧 VM 文档清单（legacy-case-map §PLAN-1095）逐项属 T-05 迁移，legacy-map.tsv 当前无 1095 行（T-05 增行）。canonical 定案：`language/composition/ordinary-projects.case.ad`（C1/C2/C3）+ `language/domain/node-values.case.ad`（C4 及领域）。
  **可行性探测**（探针 `experimental/ace/crates/av/tests/cases/plan1095_probe_tmp.rs`，临时，T-05 前删除；输出 `D:/autostack/plan-artifacts/plan-1095/t01-probe-faces.txt`）：
  - C2（task-language-composition）**原样 42 通过**——record 字面量 `{ label: "ready", enabled: true }` + `meta.enabled` + try/finally + task spawn/ask/await/stop 同源组合全绿。
  - C1（module-closure-generic）：多文件工程+`use lib.id`+泛型 `id`+列表+索引均通；**唯一阻塞**＝工程(package)路径 `packages/binding.rs:3216` 拒绝本地函数作值（`source.module-symbol`: function is not a value），而单源路径 `let f = twice` 通过（1081 basic-ref 同形）——同包组合真缺口，T-02 修复（工程模式与单源一致）。
  - C3（cte-and-ordinary-library）：`let seed = #{ scale(21) }` 与列表字面量均通（`Y-let-ct-call` 42）；**唯一阻塞**＝无类型闭包 `x =>`（该面规则：无 lambda 参数推断，1081/1082 已裁定 typed-lambda 规范化）→ C3 草案归一化 `(x int) =>`，引擎零改动（`Y-c3-typed` 42）。
  - C4（node-ordinary-coexist）：`node` 声明不在普通 `auto` profile（grammar auto-source.at:349 仅 age-h0 门控 age-node-item）→ 普通链 parse 拒。**age-h0 面已接受 fn+node+main 同源**（`H0-C4` ok nodes=["Hello"]，compile_age_h0 全通）。T-04 实施：把 node 项纳入普通 auto 链（grammar 门控 + 投影 + checked 事实 + worker CPU 观察），复用 979/981/982 producer（age_h0 观察链），不启 renderer，不凭内容轮流试 profile。
  - T-03 依据：970/969 会话/恢复接口已交付（m2-sessions、query_entry.rs），PROJECT-RESTORE 骑其真实机制。
- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
