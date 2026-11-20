---
plan_id: PLAN-1086
status: archived
completion_kind: delivered
feature_name: AV借用、move与资源值组合补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 5
current_step: 9
total_steps: 9
depends_on: [PLAN-1078, PLAN-1081, PLAN-1082, PLAN-1083, PLAN-1085]
supersedes_spec_components: [docs/specs/ace-migration/m2-aggregates.md, docs/specs/auto-test-case/resource-lifecycle-observations.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1086] AV借用、move与资源值组合补齐

## 0. 变更摘要

### 2026-11-20 r5 · 接续用例裁定

用户批准 08_strings/016_heaprc_string_pool 保留 2,000 轮、50000 原预期、100 万指令及现行墙钟预算，优化可复用的 AV 文本操作。记录列表 .pop() 移出末项，空列表报告具名越界。含旧 str 字段的记录只在旧式无显式类型的只读 let 绑定中复制为独立值。三项沿原任务登记与 AV-ONLY 约束继续验收。

### 2026-11-20 r4 · collections profile 承载 owned `Str`（用户裁定，仅规划）

用户在 105 复工评估中裁定：**1086 施工时顺带把 collections profile 的 `Str` 类型/值接上**，作为 105 解除
`lower.type-use: type kind Str is outside the av.collections/1 source surface` 的前置。范围限于 `av.collections/1` MIR
的类型/值面（元素与状态位的 owned 文本、`clone`/拼接/`len` 等最小值语义），不扩张本包其它非目标；本包仍 AV-ONLY，
非 AV 检查按 §5 #ignore 处理。

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`ownership/borrows-moves.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-08**，r4，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/9，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补旧合法view/mut/move/Hold与存储值的组合，执行可观察的所有权和释放合同。

本包交付`IF-AV-PARITY-OWNERSHIP`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。范围增量（r4）：`av.collections/1` MIR 承载 owned 文本 `Str`（元素/状态位类型与值、`clone`/拼接/`len` 最小值语义），以解除组件面文本列表的 lowering 阻断。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- L-06..09：view/mut/move/take兼容、参数/返回/闭包/容器/全局、临时生命周期、路径重借、Hold结束后可再次访问。
- 资源record/List/Str/variant/closure/object的move/use-after-move、深clone/共享操作按公开规范实现或保已批准拒绝；typed MIR clone不冒source能力。
- storage/Box/Arc/裸pointer操作先核old真实运行面；旧parser-only Box/Arc不作为legacy硬门，ACE已经承诺的独立Box安全回归保留。默认不承诺新的裸地址unsafe语言。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1086/auto-lang`，分支`plan-1086-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1086/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-20 用户通过 /goal 完成计划1086 授权 r4 的施工、独立复审、合同内修复、合入归档和清理。当前范围沿用 r4 与 AV-ONLY。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

962/1035已有资源record闭包判定、显式move和typed clone；源码List.clone/资源record clone仍拒，旧borrow若未执行写回不能冒正确语义。需要保守地对合法程序补能力、对旧bug明确分账。

既有来源与边界：

- [move_basic.case.ad](../../../crates/auto-lang/test/vm/14_borrow/004_move_basic/move_basic.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [v08_hold_alias.at](../../../experimental/ace/crates/core/tests/fixtures/aggregates_962/v08_hold_alias.at)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：14_borrow；15_nested_mutation资源部分；99_plan231；旧pointer/storage/closure borrow Rust回归。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [ownership.rs](../../../experimental/ace/crates/av-core/src/ownership.rs) | `2d2799149932028861c15c2107e5dd0b44b4330da2ccf25c01b511a2481044d7` |
| [mod.rs](../../../experimental/ace/crates/av-core/src/mir_verify/mod.rs) | `7c281df8c81c7242d58e1b61ced74b52a06ace8951d4f8820ea847c46d508ecf` |
| [av.rs](../../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1081, PLAN-1082, PLAN-1083, PLAN-1085；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

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
- [mod.rs](../../../experimental/ace/crates/av-core/src/mir_verify/mod.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [borrow.rs](../../../crates/auto-lang-core/src/ownership/borrow.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [hir_adapter.rs](../../../experimental/ace/crates/lowering/src/hir_adapter.rs)：**r4 新增触面**——`av.collections/1` 的类型面（`scalar_shape_of_use` / `scalar_type_idx_of_reserved` 目前只认 i32/bool）需承载 owned `Str`；先核 `MirType`/值表示/`clone`/拼接/`len` 的最小接线面。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/ownership/borrows-moves.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-aggregates.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-test-case/resource-lifecycle-observations.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-FC | modify（本特性消费/来源） | `docs/specs/auto-test-case/feature-organized-corpus.md` | 按特性唯一作者、同源多端检查、旧义务映射和本包迁移/观察来源；实际交付后沉淀，当前不宣称已支持 | 公共机制由1078生产一次，本包保原语言/目标义务 | AC-FC、V-FC-SOURCE/MIGRATE/TARGET |

`language-parity.md`由AVP-00首次创建，后续串行增量维护；它是当前规范的能力/来源映射，不是第二注册器。实施前本包文档只提议，真实merge后按独立复审结果沉淀，不提前将Spec写成已支持。


## 6. 测试设计

### V-AV-ONLY / V-IGNORE-LIFECYCLE（实际结果见 §9）

原T-03..06落实、T-07复核：同一Case AV已选，AC/A2R检查加#ignore；制造该目标缺工具/运行失败/错误oracle时，目标prepare/dispatch计数为0，AV仍按独立预期通过，忽略行不充PASS。将AV oracle改错必须使AV门红，全部有效AV检查被ignore或零选中也必须红。移除某非AV阶段ignore并进入其后端选择后，该目标失败必须在该后端门显现，而AV视图仍不选择它；移除ignore不能同时删原平台/ignore理由。公共格式/重复ID/AV必需shared规则出错不被这一阶段策略豁免。

### FC 测试组织与迁移必验增量（实际结果见 §9）

| V-ID | owner_stage / 原任务 | 必须观察的结果 |
|---|---|---|
| V-FC-SOURCE | work / T-02..06 | 本轮同一source/project仅选AV真实执行/按规定阶段拒绝；改公共body使相关旧证据陈旧，非AV行保持#ignore/未选与后续owner，不要求本轮运行。 |
| V-FC-MIGRATE | work / T-01/02/05/06 | 原case/check/条件/判决角色→canonical映射完整；非等价邻例、错误/资源/恢复角色不丢；suite、worker/内联/工具读者均切唯一活跃作者，旧副本仅具名历史。 |
| V-FC-TARGET | work / T-04..06 | AV oracle变异、重复活Case ID/AV旧活入口残留、零条有效AV/缺AV required检查必红；AC/A2X oracle不符或缺工具在#ignore/未选时不进入AV门。非AV完整目标oracle/emit-vs-run控制留给其启用阶段。 |

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1086-原文档清单)静态主承接 **28份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1086.mut-reborrow` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1086.owned-transfer` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1086.hold-scope` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1086.use-after-move` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | verify: mir.use-after-move，原读位置，执行派发 0 | T-01 从原拟绑定码冻结为共享 MIR 所有权诊断 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `OWN-EXITS` | AC-03/AC-04 | work / T-04..05 | 正常/return/break/continue/fail/捕获后继续/取消/close逐路实际owner账；owned循环至少2次以暴露漏释放，不只退出一次。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `BORROW-ESCAPE` | AC-03/AC-04 | work / T-04..05 | 相同类型合法局部view与非法return/capture-escape反例；错误阶段不能靠不相关类型不匹配抢先证伪。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `COLL-STR` | AC-09 | work / T-09 | `av.collections/1` 下 owned 文本 `Str` 作**列表元素/状态位**：`List<T>` 的 `Str` 元素、状态函数返回 `Str`、`clone`/拼接/`len` 不报 `lower.type-use`；与本包既有 `Str` move/clone 语义一致，不引入第二套文本表示。 | 组件面文本列表 lowering 必须从「类型不在 profile 面」变为真实可执行（013 形状的最小复现）。 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: mut-reborrow
id: "av.parity.1086.mut-reborrow"
tags: ["av-language-parity", "plan-1086"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type P { n int }
fn bump(mut p P) { p.n = p.n + 2 }
fn main() int { var p = P(40); bump(p.mut); return p.n }
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
## Case: owned-transfer
id: "av.parity.1086.owned-transfer"
tags: ["av-language-parity", "plan-1086"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type P { text Str }
fn consume(move p P) int { if p.text == "ok" { return 42 }; return 0 }
fn main() int { let p = P("ok".clone()); return consume(p.move) }
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
## Case: hold-scope
id: "av.parity.1086.hold-scope"
tags: ["av-language-parity", "plan-1086"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type P { v int }
fn main() int {
 var p = P(40)
 let hold p as q { p = P(q.v + 2) }
 return p.v
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
## Case: use-after-move
id: "av.parity.1086.use-after-move"
tags: ["av-language-parity", "plan-1086"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let a = List<int>.new()
 let b = a.move
 return a.len()
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "verify"
code: "mir.use-after-move"

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

- [x] **AC-09｜owner_stage: work**：`av.collections/1` 的 MIR 类型面承载 owned 文本 `Str`（元素/状态位、`clone`/拼接/`len` 最小值语义）；`COLL-STR` 的最小复现（组件面文本列表）从 `lower.type-use` 拒绝变为真实可执行，且不新增第二套文本表示、不弱化既有 AV `Str` 所有权/释放合同。原 T-09。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | L-06..09：view/mut/move/take兼容、参数/返回/闭包/容器/全局、临时生命周期、路径重借、Hold结束后可再次访问。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 资源record/List/Str/variant/closure/object的move/use-after-move、深clone/共享操作按公开规范实现或保已批准拒绝；typed MIR clone不冒source能力。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | storage/Box/Arc/裸pointer操作先核old真实运行面；旧parser-only Box/Arc不作为legacy硬门，ACE已经承诺的独立Box安全回归保留。默认不承诺新的裸地址unsafe语言。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-OWNERSHIP。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |
| T-09 | work | T-03 | r4 增量：`av.collections/1` 类型/值面承载 owned `Str`（`scalar_shape_of_use`/`scalar_type_idx_of_reserved` + `MirType`/值表示 + `clone`/拼接/`len`）；以组件面文本列表最小形状验证不再 `lower.type-use`，并与 T-03 的 `Str` move/clone 语义对齐。 | AC-09 |

任务状态（2026-11-20 r5 冻结检查点）：

- [x] **T-01** work：已核四个依赖类型入口、主承接和接续分母及用户兼容裁定，见 §9。
- [x] **T-02** work：旧借用、写回、捕获/逃逸和只读记录副本经普通 AV 验证。
- [x] **T-03** work：owned 传递、临时复制和移后拒绝按运行及诊断断言验证。
- [x] **T-04** work：释放、故障、预算、取消与显式 close 的真实 Rust 见证已复验。
- [x] **T-05** work：28 条主承接与 11 条接续保原源码和原 VM 预期迁入唯一作者。普通 AV 已实跑。
- [x] **T-06** work：冻结八组，修正原聚合文本探针后完整 AV 组通过，具名保留一条基线语义文档失败。齐备 IF-AV-PARITY-OWNERSHIP。
- [x] **T-07** review：已在原工作树按冻结提交重核行为、反例、迁移、测试与 Spec 增量，原聚合探针纠正后回归通过。
- [x] **T-08** merge：快进落主、核对四份规范和导航、归档并完成工作树和材料路径核销。
- [x] **T-09** work：collections-str 从共享源码前门走到 AV，返回 42，覆盖 owned 文本状态函数、列表转移和移出、克隆、拼接、Unicode 长度和内容比较。

主承接与接续共 39 条已通过 AV。交付已合入 `master` 并归档，专属树与材料路径已核销。

## 9. 复审记录

- 2026-11-20 修订 r4：`stage: new`（范围增量）；用户裁定把「collections profile 承载 owned `Str`」并入本包施工（`AC-09`/`T-09`/`COLL-STR`，触面 `crates/lowering/src/hir_adapter.rs`），用于解除 105 的 `lower.type-use` 阻断；`plan_revision: 3 → 4`、`total_steps: 8 → 9`。本修订**仅规划**：所有 0/9，未执行任何施工。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- 2026-11-20 开工：r4，主检出代码路径 clean，保留其它会话文档改动。专属树 D:/autostack/.wt/lang-1086/auto-lang，plan-1086-dev，基面 89f70228289c314ac24b74ff96b89f15d1360167；只读 auto-down 兄弟树 72b097006f34db3a871449f10e213820d51abef6。1078/1081/1082/1083/1085 已归档，先核 T-01。证据唯一路径 D:/autostack/plan-artifacts/plan-1086（已登记 in-flight）。

- 2026-11-20 基线定向探测。av-worker case-serve 的 42 条观察绑定到基面 89f70228289c314ac24b74ff96b89f15d1360167 与 auto-down@72b097006f34db3a871449f10e213820d51abef6。debug worker 预构建 28.139s、exit 0。t01-av-baseline.json SHA256 397e4b06334f2a36e8278a6ef6b26fc5bfcb0e77c2e410ca901ff9f5191013f7。owned-transfer 与 hold-scope 返回 42；mut-reborrow 首次因 mir.field-type 拒。使用已存在的 MIR 所有权数据流判决移后读，父侧现断言 stage=verify、mir.use-after-move 和原 span，仍在执行前拒绝。L-08 旧 VM 的 Box/Arc 属 parser-only，证据见 M2 分母对应行；ACE 递归 Box 安全腿已重放。
- 2026-11-20 施工检查点 c05684f748f8553994d2e80c80c1e9d8a69bb3ec。记录可变参数及方法复用写回包装，Unit 结果只承载写回字段；显式 mut fn 和实际写 self 的旧记录方法通过。只读接收者、同次调用重复可变借用拒绝。普通 AV 支持 owned Str 借作 str 形参、具名文本视图、分支与循环离开后的释放和调用者根返回。本地根返回精确拒 mir.borrow-escape，main.at:63..64、host dispatch 0。collections 类型走原 Str/StrView 与原 clone/concat/len 指令。状态函数/文本列表最小形状实跑 42。AutoUI 105 完整产品消费者未在本包工作树执行。
- 主承接 28 条均登记，21 条已切 canonical 并取得 AV 实际结果，7 条待齐。原 14_borrow/017 的 VM oracle first 保留在检查级 #ignore 下，AV 活跃借用期间移动受 mir.mut-conflict 拒绝。另接续 08_strings/013 及 99_idiom2/m01、m08、m09 四条。非 AV 跳过不计通过，Windows 排除 Linux/WSL。
- 本轮实跑（debug/incremental）：

| 范围 | 结果 | 准备 | 执行墙钟 | 命令墙钟 |
|---|---|---:|---:|---:|
| cargo t case-av-language-parity --run-prepared | 225 PASS / 94 SKIP / 0 FAIL，子退出 0 | 5.873s | 父工具 19.008s | 22.130s |
| cargo t ace-syntax ace-lowering --run-prepared | 157 + 9 PASS，两个子退出 0 | 31.092s | 4.972s / 1.348s | 10.662s |
| cargo test --locked --offline -j 4 -p ace-av --test conformance --no-run 后独立执行两个 plan1086_ownership exact case | 2 PASS，alloc/free ID 相等、live=0、frame=0 | 4.234s | fuel 后同模块恢复 0.058s；循环两次跨 continue/break/return 0.048s | 每个 exact 同左 |
| 三条已有 Box/take conformance exact | 3 PASS | 17.192s | 每条 0.044s | 每个 exact 同左 |
| COLL-STR AutoDown 三个正反控制 | 42 PASS、expected 43 实际 42 FAIL、所有 AV 检查被忽略 exit 2 | 复用已准备 worker | 0.434s / 0.414s / 0.016s | 每次独立命令同左 |

  发现及清理阶段无独立计时。已确认固定组临时输出 tmcase-w_jypgk0 清理。首次公共组 215 PASS/88 SKIP/1 FAIL（copy 是保留字），随后定向发现短路未执行分支的借用结束错误，修复后全绿。首次 Rust 生命周期形状触发已有不可达块诊断，调整为两个实际循环迭代，保留 continue、break、return、真实 owner 账。首次控制输出与输入树重叠被父工具拒绝，移到同 scope 的兄弟目录后复跑。
  公共组结果 SHA256 4394195846e063b5c0f91e23566dcf900e0dbf895b5d3693c7430f8dc937192e；共享组 dfede988459267885284c41a6e47708d99134fe85997b43e6763980a4296396e；控制 145dbc09dabfe7bccff40f79fa6909580e9b575711684daf74c8418ed6a07056。报告位于已登记 PLAN-1086 scope。
  16 个新增文档 AV 检查父侧耗时为 mut-reborrow 633ms、owned-transfer 621ms、hold-scope 693ms、use-after-move 653ms、collections-str 674ms、owned-argument 601ms、named-view 663ms、borrow-block 608ms、mut-resource 566ms、mut-return 577ms、method-write 569ms、method-readonly-receiver 562ms、mut-alias-rejected 568ms、return-external-view 649ms、return-local-view 622ms、borrow-loop 615ms。各新增检查均低于 10 秒；完整 ace-source ace-semantics ace-av 及必要下游冻结门待施工齐后执行。
- stage: work | plan_id: PLAN-1086 | plan_revision: r4 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1086/auto-lang | branch: plan-1086-dev | base_commit: 89f70228289c314ac24b74ff96b89f15d1360167 | code_commit: c05684f748f8553994d2e80c80c1e9d8a69bb3ec | task_ids: T-09 pass; T-01..T-06 incomplete | blockers: §10 legacy compatibility decisions | next_responsible_role: user then work。
- stage: goal | plan_id: PLAN-1086 | plan_revision: r4 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1086/auto-lang | commit: c05684f748f8553994d2e80c80c1e9d8a69bb3ec | next: user。用户 /goal 完成计划1086 的阶段授权见 §4。

- 2026-11-20 第二个实施检查点 ec1131fa64634ec8d2fd520249960779278799db。99_plan231/003–005 各保原源码、ID、VM 独立 oracle 和检查级 #ignore。单语句 is 臂经原 statement projector，Option 与 None 比较复用受检的 tag 读，Some(int)/None 插值按所选臂格式化并保原 payload。003 实际 total: 6，004 实际 got: 2 / got: 1，005 实际 v1: 2 / v2: 1 / v3: None。原源码 stdout 比对 r2-options.json SHA256 143afe7aca88fe5d7d01c7d9b9e6cc850834ac0b460c002b84cf33383f007124。三条迁移实跑父侧耗时 003_simple_for_option 668ms、004_for_option_no_is 708ms、005_mutfn_no_loop 669ms。
- OWN-EXITS 补强：普通 Auto 前门到 ABC 后运行，验证捕获后的继续执行结果 42、未捕获语言错误保留消息、已经进入 stdout provider 的执行取消、显式 close 释放悬挂的 owned 文本并传播 finally 抛出的错误。每条检查至少一个已实际分配对象、alloc/free ID 对账、live=0 和 frame=0。重新实跑先前 fuel/loop 两例、PLAN-1081 直接借用闭包合法与逃逸拒绝两例、PLAN-965 显式和 break-close 传播例、host cancel/io 分类例。10 个 exact 都通过。新增四例逐测 0.046s、0.042s、0.078s、0.052s（按上述顺序）；整体范围 0.037–0.078s，准备 17.862s。独立发现/清理无单独计时。r2-final-rust-run.log SHA256 8eb5ac7ddb7b709035c662956a7dd72ba15f2b652a6faca68703ab4922bf7d4d。
- 公共 AV/语法/lowering 首次并集通过，准备 23.846s、执行整命令 33.843s。首次三条迁移的 VM 小节随 target AV 未选，随后补齐检查级 #ignore，并复用生产源未变的 157 语法和 9 lowering 通过。最终公共组 228 PASS / 97 SKIP / 0 FAIL，独立准备 2.398s，命令墙钟 21.940s，r2-final-case-run/results.json SHA256 5e40ff7fb537c8aa5887f9e94c4b163937f2ba4e440752bf962fe42468a49ad7。Windows 的 Linux/WSL 及非 AV 域仍排除；完整冻结组在剩余验收齐后进入。
- stage: goal | plan_id: PLAN-1086 | plan_revision: r4 | round: 2 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1086/auto-lang | commit: ec1131fa64634ec8d2fd520249960779278799db | next: user。已继续完成可独立实证的三条旧输入及生命周期腿；继续交付受 §10 兼容裁定影响。

- 2026-11-20 第三个实施检查点 c0a9cadb810e12d6ad67771a0891f5245e81a5b8。接续用例 legacy.15_nested_mutation.015_self_list_push 的首阻断为显式类型构造 List<int>.new([]) 的空字面量推导。原源码进普通 AV 后端，由显式元素类型构出空列表，随后三次 mut fn 写回及只读 count 从同一记录取得 3。基线误拒与许可对照有据，显式 List<bool>.new([]) 返回 0，无类型 List.new([]) 仍在 bind 以 source.array-lit-empty 拒绝。生产 legacy inventory 确认该 wrapper 解析到 canonical 文档，VM 原检查仍在 #ignore。AV 检查父侧耗时 572ms。历史 TSV 的 184ebeaca0a66a03 为整作者文档 SHA16，保存在迁移说明中，并把来源字段归正为原 Auto 源 SHA16 8a63fa51485c94d58。
- 生产 legacy-inventory 静态对账 28 条主承接行，24 条 canonical 源、原 VM oracle 与换行设定完全相同，四条待裁定仍引用原作者。legacy 查询实际执行 0，后续重查也命中新接续作者。审计耗时 2.608s / 2.432s。main-denominator-audit.json SHA256 41f35b1d7d85a41f504a3e7aeb54f98e6c325356e7a7ac4d6166b8365183c9d8。8 条 V-FC 定向控制通过，包含同源 AV 通过与 VM 跳过、改错忽略 VM oracle、空 AV 选区、套件另有 AV 时单 Case 缺 required、重复 Case ID、同源 fresh、改源 stale、改源同 oracle 重放。单控制墙钟 0.018–0.401s。首次空集的错误为选择器零命中，增加一个独立有效 AV 后才触及单 Case required 防线。corrected/observations.json SHA256 13c4b06f70d6d5c6077f551affc63b8bedff032dc4a6ffc60b7449361283f708。
- 独立准备 cargo t case-av-language-parity ace-semantics --prepare-only 19.224s。run-prepared 整命令 48.707s，发现/清理无单独计时。公共 AV 子退出 0，18902.913ms，229 PASS / 98 SKIP / 0 FAIL。语义子退出 100，15790.842ms，294 PASS / 1 FAIL / 10 SKIP。失败为 PLAN-1062 已登记的 sema_boundary_case_files_executed_status_honest，缺 executed 状态的 cache-roundtrip.document.case.ad 对照 master blob 一致，上一已归档 PLAN-1085 §9 同失败。Windows Linux/WSL 条件未扩张。r3-groups-run/results.json SHA256 4296a2c7367fb63dc38c32651b808a1be2d253ffb17be2d4a0c1c60d1cd348c2。
- 其余 11 条历史待项定向回放发现四条主承接仍分别受旧 str 存储、列表携带借用、文本 .mut 写回规则影响。另七条接续在本轮回放中有独立诊断，其中 self-list-push 已闭合。余六条涉及旧无空格范围、字符串方法、旧小写 str 存储和 slice/std API 边界，实际诊断不充通过。open-obligations.json SHA256 8daca1e603d2a418179313c451a41adccc7c7ed9a8a9e3f5ebb9eabf4dbd9789。
- stage: goal | plan_id: PLAN-1086 | plan_revision: r4 | round: 3 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1086/auto-lang | commit: c0a9cadb810e12d6ad67771a0891f5245e81a5b8 | next: user。三轮均见相同兼容语义决策缺失。四条主承接的 AV 兼容判决需明确。完成其余入口与最终阶段依赖此裁定。

- 2026-11-20 第四个实施检查点 19e5ef77297598ac28db04426a3ca857855e1983。按用户三项裁定迁入原样主承接四条。文本 .mut 以词法绑定定位可变 owner，append 经受检拼接和原位写回，原 VM 预期 hello 保留在检查级 #ignore，AV 实际 hello world。脚本尾列表用原文本复制指令发布 ["hello", "world"]。旧 SSE record/List/Option 存储规范化为 owned Str，普通借用参数仍是 str；两条旧源码均输出 events: 1 及各自完成行。旧 List<str> 出口把持有的文本复制进 owned 存储，显式 List<Str> 的无 .move 入列仍以 source.move-required 拒绝。普通未发布的借用列表、不可变 owner、活跃 .view 与写回冲突、.mut 绑定逃逸分别见 mir.type.list-elem、source.immutable、mir.mut-conflict、source.view-escape。兼容对照 r4-controls.json SHA256 c355d23c9518465a97914fca2e8e287b1e95d6e1437c1bcbcdea0150844302f4。
- 已闭合三个接续用例 08_strings/017_sub_semantics、018_loop_concat_churn 和 99_slice/001_slice。旧 sub/slice 是排他端点，substr 使用受限长度；无空格 .. 保留为独立记号，方括号文本切片复用现有 checked slice。只读文本 helper 复制需要跨拥有参数边界的文本。遍历列表里的拥有文本沿已有 PLACE_GET 和文本复制产生独立计算值。原样 stdout 分别为 el/ell/el/空行/o、res *.tmp|res *.bak|res *.log|、99_slice: all assertions passed，双流均有显式检查。尚待三项接续见 §10，保原分母与旧源码。原输入观察 r4-observations.json SHA256 52e57db5c137c7b4892996d25e70e4f4a814996d0104f8de22ed8a86a987506e。
- 生产 legacy-inventory 退出 0、耗时 5.548s、legacy 执行 0。28/28 主承接 wrapper 解析到唯一 canonical 作者，逐字节比对原 Auto 正文及 VM 检查，历史 VM 换行条件及 oracle 不变。r4-main-denominator.json SHA256 fb583b000e2275c4e654715b299f1376e900e74c6d2f357ce3f06d2410010b5e。r4-legacy-inventory.json SHA256 54cff472856417c2dbc4e9517cf0274acc26fbe38133195e81e0876fa669144b。
- cargo t case-av-language-parity ace-syntax ace-lowering 显式准备 28.893s；run-prepared 整命令 34.851s，exit 0。公共组 236 PASS、105 SKIP、0 FAIL，子墙钟 22.047s；语法 157 PASS、5.152s；lowering 9 PASS、1.422s。语法/lowering 未出现条件排除，发现/清理无独立计时。AV 公共组准备只含 av、case-parent；旧 VM 与其他目标未运行。上述七条迁移的 AV 父侧耗时依次为 671、603、705、709、760、690、756 ms；VM 七条各 SKIP。r4-checkpoint-run/results.json SHA256 1df141232135857eda5161c4be4183af646479acb626015e2219fa6322f518fb。最终完整组和独立复审仍待。
- 共享组首轮在 19e5ef772 准备 45.839s、执行 43.662s，总退出 1。ace-source 子进程 14.549s、929 PASS、1 新红、1 继承 ABORT、1 条 Windows 排除。新红为 collection_function_keeps_view_escape_and_reference_bounds_checks，证明普通 Auto 的文本元素复制误进窄 collections 检查入口。继承 ABORT 为 Windows 上 deep_not_chain_reaches_checked_bundle 的 0xc00000fd。ace-semantics 子进程 14.689s、294 PASS、1 继承 FAIL、10 条件跳过。继承 FAIL 为 PLAN-1062 已登记的 sema_boundary_case_files_executed_status_honest。准备工具仅 case-parent 与 av，Windows 未执行 Linux/WSL。r4-source-run/results.json SHA256 f93496481f0fce7f9520f6f8c7c5021c8b6404f85f980564330393d7ef6d2c31。
- 修复提交 90ceb0be61ac489a62179bccc65c5e4497164c98 让普通单文件和包入口显式使用同一 Auto HIR 验证入口，窄 profile 的模块及单函数路径使用原规则。准备 25.734s。正式 nextest archive 上重放原窄面反例 1 PASS，测试 0.131s、执行 0.149s、退出 0，临时提取目录已核销。修复后二条生产 AutoDown AV 定向为原样 018 循环文本 4.603s 及 PLAN-1085 多文件 package 2.479s，均 1 PASS、退出 0，所选 VM/其它目标实际执行 0。三份修复证据 SHA256 依次为 673be826866ed18db1188a08296d76761204c705be0234e25e4d4b0ef17830fe、528d68d190747ecc2ace227a2c3c4d0d69627a1897e41224903b47dd04758838、9d54a82dbd972d7edd95e631984a335288b4f124f77b78ea4319e80e8616e782。完整冻结并集留待三项接续裁定落实。当前 worktree clean。
- stage: goal | plan_id: PLAN-1086 | plan_revision: r4 | round: 4 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1086/auto-lang | commit: 90ceb0be61ac489a62179bccc65c5e4497164c98 | next: user。原三项裁定已落实，主承接 28 条已闭合。接续三项的预算与语义裁定见 §10。
- 2026-11-20 r5 检查点 7e7a9bf6393e8a1e8ab77736ff4d4ea3ad175f15。用户追加三项裁定后完成剩余接续。29_list_shims/002 原样输出 2、alpha、beta、beta、1，空列表在 run 以 av.trap.list-index 拒绝，移出由既有 LIST_REMOVE 完成。10_types/036 原样输出 array field ok 和 second id ok。替换源列表元素后副本仍读到 before，列表读到 after。显式 Str 记录和 var 绑定仍在 resolve 给 source.view-escape。文本字段复制使用校验 ListPlace、PlaceGet、FieldGet 与已有 Clone，原列表保持持有其字段。
- 普通 AV 新增能力 av.text.scan/1，携带 STR_FIND 与 STR_TRIM。扫描对可提前结束的 ASCII 路径按访问字节计费，UTF-8 扫描事先按输入计费，文本分配按复制字节计费。文本查找保留原 Unicode 标量位置，未命中为 -1 或按已选语境包装 None。去边界空白保持空格、制表符、CR 和 LF。未发布的切片临时值避免重复复制，去空白后只取长度走标量结果。字面量 split 预先求边界，每轮仍在原操作位置创建列表与拥有文本。字节计费、取消与墙钟使用同一执行器预算，窄 profile 不取得新准入。
- 016 第一次带优化的诊断样本在扩展的诊断上限下需 1,360,055 条指令，融合去空白和长度后降到 1,050,049。最终保留一个搜索结果供 Option 和旧数值使用，数值读仍检查原局部量是否已移动。最终原样 2,000 轮在默认限制下返回 50000，990,049 条指令，执行 0.270s。生产 AutoDown AV 检查耗时 1.114s，002/036 分别 0.630s/0.615s。r5-final-fuel.log SHA256 2df957446c2dbf755d723d987e7b22d327bf7d9ec3bf7f3fdc82cab0d27e365d；记录越界、副本、显式 owned 和可变绑定反例及 Unicode/空分段控制 r5-controls.json SHA256 28a0544104714a331e526143f9f4f15d2f1488f5a8083ea4cf0861145b13826b。
- 生产公共同源套件在已构建的普通 worker 下 245 PASS、108 SKIP、0 FAIL/ERROR，退出 0。r5-work-public/report.json SHA256 8ac6753d607cf399454af46a43b846328736424ddb0e4031d975cee78a5ee4b2。生产清单核 39 个 legacy 包装器均指向唯一作者，逐字节核原 Auto 正文及 VM oracle，旧执行器调用 0。22 个新增 AV 检查最大 1.326s。公共文档首轮 lint 因两个 Probe 写成内部阶段名 bind 拒绝，正确使用作者阶段 resolve 后 full reference admission 通过。
- 使用已构建的 ACE Rust 库显式准备 plan1086_ownership 独立测试进程，0.748s，退出 0。七条逐条重放通过，含普通缺能力拒绝、4,096 字节扫描的 128 指令预算拒绝、原 limits 成功、故障、取消、关闭和多轮退出的释放。测试单进程 0.041–0.054s。冻结并集与复审仍待。
- 冻结准备第一轮到 ace-av 的共享编译时，ace-native 对新 MIR 指令匹配不穷尽（E0004，未执行测试）。修复为 native 明确拒绝 AV 扫描指令。同步到 master 68fd4c59222c77a5de0963ff5827f1353abd7b4f 后，原七个提交 range-diff 全等，冻结候选 4a909d6ff50807546e0de0fa34fa6df53727a012。同步包含 PLAN-125 测试驱动和 PLAN-1051 共享源变更，对冻结并集重新准备。

- r5 首轮冻结并集。以脚本 r5-av-phase.py 运行固定组准备，保留工具 case-parent、av 和 execution-fixture。普通组原排除性能，另按 Windows 裁定排除 Linux/WSL，具名过滤 ace-av 中混合非 AV 执行腿。准备 90.260s。一次 run-prepared 尝试因 AutoUI 正在编译占用，49.997s 后退出 10，未执行用例。重排等待 87.843s 后，`cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering ace-artifacts auto-test-case --run-prepared <prepared.json> --out <scope>` 实际完成八组，整命令约 162s，退出 1。墙钟含子 nextest 启动和提取；清理未单独计时。

| 组 | PASS | FAIL | SKIP 或具名排除 | 子墙钟 / exit |
|---|---:|---:|---|---|
| case-av-language-parity | 245 | 0 | 108 SKIP | 26.730s / 0 |
| ace-source | 932 | 0 | 1 原 ignore | 10.135s / 0 |
| ace-syntax | 157 | 0 | 3 性能腿排除 | 4.517s / 0 |
| ace-semantics | 294 | 1 | 10 性能腿排除 | 16.035s / 100 |
| ace-av | 1468 | 1 | 23 性能、1 Linux、27 混合非 AV 腿排除 | 50.671s / 100 |
| ace-lowering | 9 | 0 | 0 | 1.475s / 0 |
| ace-artifacts | 43 | 0 | 6 性能腿排除、1 原 ignore | 12.513s / 0 |
| auto-test-case | 157 | 0 | 0 | 4.567s / 0 |

  准备收据 SHA256 727735b446255d60cd573b7c40155fa7906551719fbe73a1540c080ac2c9bc52，运行结果 5e4599654172469ff30c831cd49dac18248966320f4d38e5672c2d116d39dd24，选组配置 c6a23e7944077afa56e6ac419b4e0a62a4136b47a7370240a086741406de3265，source state 558ee0fa4e994ebd5c9d1692f446cf8648fd8615bf350c5e46df5962bd2b9724。PLAN-125 修改的两个 console 契约继任用例与 deep_not_chain 此轮通过。新红 `probe972::probe972_t02_str_surface` 是原规范 §5.3 已标明由本包闭合的局部计算视图，聚合 profile 已准入，旧用例仍断言 verify.local-type。纠正为保留原源码并独立断言 `"A中😀".slice(0, 1).len_bytes() == 1`，其余子例和原执行条件无变化。测试修正提交 e47f86744ab90025e85047e611c2193cb41fd46a，生产源码、公共语料、配置和依赖与 4a909d6ff 完全相同；仅重跑受影响 ace-av 组。

  继承红 `sema_boundary_case_files_executed_status_honest` 重现 D-1037-01，原断言和缺 executed 状态的文档均与 master 构件相同（Git blob 0dbe00f1c81b30153fd06a869d262ad1efa1d8a8 与 c4874997b7f528c6e6a1faf3efefb3d463a934b8）；保持 PLAN-1062 的原责任与失败状态。此轮没有其它语义组失败。七个 Rust 组发现阶段分别为 1.720、2.270、13.140、5.527、1.120、2.200、3.560s，各组按上表顺序。排除计数由发现清单逐条分类；唯一 Windows 排除是 `stdlib_assembly_1043_linux_target_ignores_missing_windows_sdk`，归专用 Linux 环境。22 个新增 AutoDown 检查实际为 0.515–0.965s，016 原样检查为 1.245s。七个新增 Rust 检查在首轮冻结 AV 组为 0.096–0.117s。

- r5 修正后，`python r5-av-phase.py ace-av --prepare-only` 和 `cargo t ace-av --run-prepared <prepared.json>` 在 e47f86744 实跑。等资源 355.873s，准备命令 11.465s（驱动 10.967s），发现 4.837s，子运行 56.306s（nextest 51.389s），整运行命令 62.107s。清理无单独计时。1469 PASS、0 FAIL、51 原条件排除，子与父均退出 0。原聚合文本探针通过，6.410s。七个新 Rust 检查 0.07–0.75s。准备 SHA256 1822c864b20d3def5e1e8b985e3102bb651884fe4f01ce4f825233ca038dbf7a，运行 SHA256 c36c417eb745bfde3837b4fdb69362a295d09fe2e8bee9054b6ed88c1610c113，source state c96bd9b33f22687d68e08a87e3ffa711c7039a1ceb8689ae70087385bfe9a849。复用其余七组，唯一变更为 AV conformance 的 probe972 预期，所以其余测试二进制输入和生产依赖无变化。
- stage: work | plan_id: PLAN-1086 | plan_revision: r5 | outcome: pass（本包验收齐，已有语义红单记） | worktree_path: D:/autostack/.wt/lang-1086/auto-lang | base_commit: 68fd4c59222c77a5de0963ff5827f1353abd7b4f | code_commit: e47f86744ab90025e85047e611c2193cb41fd46a | dependency: auto-down@72b097006f34db3a871449f10e213820d51abef6 | task_ids: T-01..06, T-09 | next: review。

### 2026-11-20 r5 复审

复审在同一会话的独立验证阶段进入原专属树，没有另设 reviewer。以八个提交的实际 diff、生产消费者、同源语料/原 oracle、两份完整组收据及另选反例核验。运行过的 AV 组证据绑定 e47f86744，其他七组绑定生产输入相同的 4a909d6ff。拟落地四个规范组件与两条模块导航、两条路线出处经核对。规范草案 SHA256 6fc53f6ef13e55c47a30b8db6e90bfccc242222c16474542ca4b6d2aba267606，导航草案 2a5a559830dc18bad379402b661d30b91967ae6623f5bb3b12a515a74cad7161。

| 验收 | 复审所核事实 |
|---|---|
| AC-01、AC-09 | 从普通 AV 父入口取得 canonical 正例实际值和双流，选测作用域结束后 append 返回 alpha beta、Unicode 去空白长度为 2、两个 record 持有者移出与修改打印 b/c、空分隔符 split 返回 13。collections 最小状态函数和拥有文本列表返回 42。聚合探针使用原源码和独立长度 1 的 oracle。 |
| AC-02、AC-03 | move 后重复读取缓存变量在执行前仍拒绝。对照活跃借用冲突、源 span、旧式副本/显式 owned、逃逸、空 pop、燃料、能力、取消、host 失败与闭合传播。七个生命周期和扫描见证均运行，资源账含实际对象 ID、live 与 frame。 |
| AC-04、AC-FC | 39 个生产 wrapper 只指向同一作者，原 Auto、VM oracle 和换行条件逐字节相同。28 个主承接与 11 个接续均有 AV 实际；22 个新增检查另计。早期同源修改失效和实际重放有源身份对照。 |
| AC-05、AC-AV-ONLY | 共享前门、MIR、ABC 重准入、普通 AV 执行依次可见，旧执行派发为零。复选控制分别是基线、错误的已忽略 VM oracle（均退出 0），无 AV、另一 Case 有 AV 但缺 required、重复 ID（均退出 2），以及原 016 的 AV oracle 49999 vs 实际 50000（退出 1）。准备只含 AV 与公共工具，混合非 AV 和 Linux 执行腿按发现清单排除。 |
| AC-06、AC-07 | 八组实际执行；修正聚合探针后 AV 范围无新红。语义文档失败按原 owner 与基面构件保留。119 个具位置编译警告无一落在本 Plan 改动行。probe972 的已有 rustfmt 差异与基面相同，git diff --check 通过，未引入测试调试输出。 |

另选 worker 对照 SHA256 9292f652025397f601b2386d185b4b2defad53bb2a6a572a5af16c33348171cd，父侧选择反控 SHA256 2b320c78cb22e3776f23b7b207412a448d6fe966cb847ebdc453472db3623da9，资源 exact 记录 SHA256 7cf3459faa75716268c6125c719df8b7cbb38a38aac276207e78f62e0c64fb3f。复审确认兼容范围依 §10，脚本发布、旧式只读记录复制与显式 owned 的边界均有反例。AutoUI 105 完整运行仍属其产品 owner，已测同结构最小输入。旧 Box/Arc parser-only 状态与非 AV 后端义务保持原 owner。无本包新增的开放债务。

- stage: review | plan_id: PLAN-1086 | plan_revision: r5 | outcome: pass | reviewed_commit: e47f86744ab90025e85047e611c2193cb41fd46a | base_commit: 68fd4c59222c77a5de0963ff5827f1353abd7b4f | dependency: auto-down@72b097006f34db3a871449f10e213820d51abef6 | delta: SD-01 / SD-02 / SD-03 / SD-FC 与导航 | inherited_red: D-1037-01（PLAN-1062） | next: merge。

- stage: goal | plan_id: PLAN-1086 | plan_revision: r5 | round: 5 | from: work/review | to: merge | status: reviewed | outcome: pass | commit: e47f86744ab90025e85047e611c2193cb41fd46a。

### 2026-11-20 r5 合入与归档

- checkpoint: prepared | key: PLAN-1086:r5 | reviewed_commit: e47f86744ab90025e85047e611c2193cb41fd46a | delivery_commit: 426eef929855622ef1cc48c990b981d6fb20f015。两提交间只有已复审的四份规范和导航，无运行输入变化。复用冻结八组，保留已识别的 D-1037-01。
- checkpoint: landed | key: PLAN-1086:r5 | base: 68fd4c59222c77a5de0963ff5827f1353abd7b4f | delivery_commit: 426eef929855622ef1cc48c990b981d6fb20f015。合入前专属树 clean，`wt-guard.ps1` 退出 0。主检出 `git merge --ff-only plan-1086-dev` 退出 0，刚落主 HEAD 等于 delivery，无 merge commit。七个早期提交的旧到新映射为 c05684f74→f42334407、ec1131fa6→701ba54e5、c0a9cadb8→716157ce9、19e5ef772→af50aefee、90ceb0be6→f44d83578、7e7a9bf63→2c63243c3、f7815509b→4a909d6ff；range-diff 七条均等。
- checkpoint: canonical_verified | key: PLAN-1086:r5 | delivery_commit: 426eef929855622ef1cc48c990b981d6fb20f015。主检出各文件的 Git blob 与交付提交相同，SHA256 见下。主检出原样语料中单选 016，以已准备的同源普通 AV 工具执行，1 PASS、108 SKIP、244 NOT_RUN，退出 0，墙钟 1.910s，实际 50000。报告 SHA256 298e6b8821d6c11c57ab333b7a234117e9a972c6e4197637ef72dd6a19b76f9c。

| 落地规范与导航 | 主检出 SHA256 |
|---|---|
| `docs/specs/ace-migration/m2-aggregates.md` | 203f496d1020457e599334fc5ad64cf451ad2875bd7306f91846eda4483bb8d7 |
| `docs/specs/auto-av/language-parity.md` | a7b8f3c32e15e05ab0c55ccc2e7e0b2991b74fb8d8282c696a00c0e7b45a8ce6 |
| `docs/specs/auto-test-case/feature-organized-corpus.md` | 25f16201967020f6812615f2a71869d6ab7dd92b62d45887d09b6eca264696d6 |
| `docs/specs/auto-test-case/resource-lifecycle-observations.md` | c7db4e492314faccc88e97a346763671f4a37ad82aadb4294b514155cb5212a3 |
| `docs/specs/auto-av/overview.md` | e68d87802575a8f64aaf0a874a12eb866842bb18a8d160391f31b917de3b51a7 |
| `docs/specs/auto-test-case/architecture.md` | 12d57e20f951faa3e68d8a5c12c49ae08dc75fb164d9e961df0233a298743e16 |
| `docs/design/strategy/av-autovm-language-parity.md` | 2d83c3def0363d6fd8fbada8d5ab558f6cf33664e202243be96054ed3ca2a723 |
| `docs/design/strategy/av-autovm-legacy-case-map.md` | c085ebdd57756449855f3ab588c80ef4d35c86731380bd2f5ce1406fb0926dc2 |

- checkpoint: archived | key: PLAN-1086:r5 | completion_kind: delivered | path: docs/plans/archive/1086-av-parity-borrows-moves-and-storage.md | cleanup: pending。归档阶段保 T-08 与 AC-08 待清理核销。
- checkpoint: cleaned | key: PLAN-1086:r5 | archive_commit: 9436c3ca543cca8dabc5bcf5e07c0ccbd29082b7。归档前实际检查 8/9 任务、10/11 验收项、8 份入口文件和 17 条出链。归档后两树状态 clean，分别核对已落主 auto-lang@426eef929 和 detached auto-down@72b097006。对 `D:/autostack/.wt/lang-1086/auto-lang` 及 `D:/autostack/.wt/lang-1086/auto-down` 各执行 `wt-guard.ps1`，均 clean，退出 0。分别删除 126674 文件、22181953269 字节与 1294 文件、26841767 字节，含原树 target 和 972 探针构件；删除已合入分支 `plan-1086-dev` 与空组目录。未回收中间产物。发现的 nextest 系统临时提取目录已由驱动清理。
- 唯一计划材料根 `D:/autostack/plan-artifacts/plan-1086` 的内容按 §9 折叠，核对登记 owner 与绝对路径，经 `wt-guard.ps1` clean 后用 PowerShell 删除 4393 文件、620086471 字节。已验证两个 worktree、分支、组目录和材料根均不存在。未发现外部安装、消费者副本或另盘计划专用输出。现有基线红按照原 owner 跟踪，本计划无具名材料持有。
- 发布现状：主检出的 `experimental/ace/target/release/av-worker.exe` 构建时间为 2026-11-19 11:10:13 +08:00，早于交付提交。此次交付的执行证据来自已准备的 debug 工具，未进行发布构建或部署。
- 周期批量预检归独立主检出任务。上次收据 2026-11-20T00:00:00Z，覆盖 949fdc5e1，最高已落主 ID 1098；在该覆盖提交之后归档的 1085 触发 `%5`。主检出仍有其它会话在编辑的设计文件，批量门的干净输入条件未满足。既有批量准备缺陷归 PLAN-1108。未启动 `cargo t all`，收据没有前移；本 Plan 采用上述 AV 范围冻结证据。
- stage: merge | plan_id: PLAN-1086 | plan_revision: r5 | outcome: pass | delivery_commit: 426eef929855622ef1cc48c990b981d6fb20f015 | cleanup: verified | task_ids: T-01..09 complete。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。

### 2026-11-20 r4 兼容裁定与待验收

1. 用户批准旧 SSE 用例 99_plan231/001_nested_mutfn、002_direct_drain 保留源码和原 oracle。AV 将这些拥有型存储位置的小写 str 归一为 owned Str。两条原样 stdout 已通过，见 §9。
2. 用户裁定 14_borrow/003_mut_basic 的 VM 预期保留 hello，AV 单独验证 m.append(" world") 实际写回 hello world。
3. 用户批准 14_borrow/015_str_slice_in_array 仅在脚本结果发布时将借用文本复制为 owned Str，保留源码及 ["hello", "world"] 预期。普通存储仍沿现行所有权规则。
4. 用户批准在本计划内优化 08_strings/016_heaprc_string_pool，保留原源码、2,000 轮、50000 独立预期和原预算。首次完整 AV 执行在 1,000,000 指令处返回 av.resource.instr-steps，辅助 100 轮返回 2500，500 轮亦耗尽预算。重新验证原样输入与新指令的能力拒绝、文本范围、工作量计费。
5. 用户批准 29_list_shims/002_struct_list 采用 AV 记录列表 .pop() 的有界移出语义，空列表报告具名越界。原样 oracle 仍为 2 / alpha / beta / beta / 1（五行，含末尾换行），旧 VM 检查按原条件保留。
6. 用户批准 10_types/036_struct_in_array 仅在含旧 str 字段的旧式只读绑定中获取独立记录副本，保留源码和原预期。显式 owned Str 记录声明和可变绑定的借用诊断须对照核验。
7. 六项兼容裁定已落实，冻结验证与复审记录见 §9。
