---
plan_id: PLAN-1087
status: archived
completion_kind: delivered
feature_name: AV跨调用语言错误与控制流展开补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 4
current_step: 7
total_steps: 8
depends_on: [PLAN-1078, PLAN-1082, PLAN-1083, PLAN-1084, PLAN-1085, PLAN-1086]
supersedes_spec_components: [docs/specs/ace-migration/m2-control-effects.md, docs/specs/auto-lang/vm/design/null-family.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1087] AV跨调用语言错误与控制流展开补齐

## 0. 变更摘要

### 2026-11-20 r4 · 两项旧 VM 行为的 AV 差异

用户裁定保留现行 AV 语义。旧 04.006 的值位非穷尽 `is` 在 AV 绑定阶段拒绝。旧 16.007/008 的 `.?` 在非 Option 返回边界统一拒绝，静态准入与输入是 Some 或 None 无关。三份旧源与四条 VM oracle 原样迁移，AV 为同一源码固定诊断。05.003 的独立范围值沿用 PLAN-962 的迭代位限制，并以同一方式保留其 VM 预期。其余正例与本包验收条件沿用 r3。

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`errors/catch-finally.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-09**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补旧VM合法可捕获错误跨函数/模块传播，并保持finally、资源释放与宿主故障分类。

本包交付`IF-AV-PARITY-ERROR`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- 同一受检语言错误在调用、模块、闭包和动态调用间传播；catch payload/message/span与错误类别稳定，不用固定trap码代替。
- try/catch/finally、rethrow、.?/默认抑制、??、break/continue/return的控制优先级与finally恰一次；stack unwind释放每个scope owner。
- 旧越界/null/Python等可捕获普通错误按已定模式接入；取消/预算/权限/基础设施失败保宿主边界，保首次红/环境与未执行分类。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1087/auto-lang`，分支`plan-1087-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1087/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-20 用户以 /goal 完成计划1087 授权 r3 施工、独立复审与合入收尾。专属树 D:/autostack/.wt/lang-1087/auto-lang，分支 plan-1087-dev，基面 a3655eb2ff2638cdbecc002b7bfd74122d6ab43a。只读兄弟树 auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5。PLAN-1078、1082–1086 均已归档，普通 AV 源码入口、共享作者视图、检查级忽略与所有权基础已定位。材料 D:/autostack/plan-artifacts/plan-1087/ 登记为在途。

2026-11-20 用户回复「保留现行 AV 语义并记录具名差异」，适用于值位非穷尽 `is` 和非 Option 返回边界的 Option `.?`。r4 已纳入本轮执行合同，继承原施工、复审与合入授权。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

965仅catch本帧语言错误，跨调用fail终止；旧VM的RuntimeError可被调用方catch。补错误对象/来源/传播时不能把Trap/Resource/Cancel/未授权host失败全部变成可捕获语言错误。

既有来源与边界：

- [control_effects.document.case.ad](../../../crates/auto-lang/test/cases/ace-m2/control_effects/control_effects.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：04_control_flow；05_loops；16_option_result传播/错误；99_script_err；旧engine null/index/异常Rust测试。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [mir.rs](../../../experimental/ace/crates/av-core/src/mir.rs) | `2a2c32d4746c4a17e6edee0350a9bb2ae027a0faa10305b6524d55f19adefce0` |
| [av.rs](../../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |
| [mod.rs](../../../experimental/ace/crates/av-core/src/mir_verify/mod.rs) | `7c281df8c81c7242d58e1b61ced74b52a06ace8951d4f8820ea847c46d508ecf` |

硬依赖：PLAN-1078, PLAN-1082, PLAN-1083, PLAN-1084, PLAN-1085, PLAN-1086；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 具名旧语义差异

| 原例 | 保留的 VM oracle | 同一源的 AV oracle | 裁定 |
|---|---|---|---|
| 04.006 | stdout `Here is 10!`，结果 `10` | resolve / `source.pattern-nonexhaustive`，`is_stmt.at:11..64` | 非穷尽值位 `is` 静态拒绝 |
| 16.007 | 结果 `42` | resolve / `source.propagate-incompatible`，`propagate_some.at:25..28` | `.?` 的 Option 传播需要 Option 返回类型 |
| 16.008 | 结果 `-1` | resolve / `source.propagate-incompatible`，`propagate_none.at:21..24` | 同一静态规则适用于 None |
| 05.003 | 结果 `1..5` | parse / `source.syntax`，`range_literal.at:1..3` | PLAN-962 仅允许迭代位范围，保留当前前门拒绝 |

另一个已交差异为 16.010。脚本顶层 Result `.?` 依 PLAN-965 抛出原错误 `error`，原 VM 哨兵 `4000000` 作为独立参考保留。

### 输入、真实代码路径与所有权

- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [mir.rs](../../../experimental/ace/crates/av-core/src/mir.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [mod.rs](../../../experimental/ace/crates/av-core/src/mir_verify/mod.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [hir_adapter.rs](../../../experimental/ace/crates/lowering/src/hir_adapter.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/errors/catch-finally.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-control-effects.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1087-原文档清单)静态主承接 **25份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1087.catch-callee-fail` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1087.finally-return-order` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1087.uncaught-keeps-message` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | run: parity-error-xyz | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1087.catch-index-error` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `ERROR-CLASS` | AC-03/AC-04 | work / T-04..05 | catch必须验证错误kind与原source span（已有观察通道扩到父侧），另用授权撤销/resource/cancel反例证它们不可catch，不只函数return42。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `UNWIND-RESOURCE` | AC-03/AC-04 | work / T-04..05 | 中间三层函数各持owned值，内层fail→外层catch继续后二次调用，finally轨迹与live=0均符合；不得双free或遗留旧控制态。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: catch-callee-fail
id: "av.parity.1087.catch-callee-fail"
tags: ["av-language-parity", "plan-1087"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn inner() int { fail("boom") }
fn main() int {
 try { return inner() } catch e { if e == "boom" { return 42 }; return 1 }
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
## Case: finally-return-order
id: "av.parity.1087.finally-return-order"
tags: ["av-language-parity", "plan-1087"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type Trace { n int }
fn inner(mut t Trace) int {
 try { return 19 } finally { t.n = t.n * 10 + 1 }
}
fn main() int {
 var t = Trace(0)
 let a = inner(t.mut)
 try { if a != 19 { return 0 } } finally { t.n = t.n * 10 + 2 }
 if t.n == 12 { return 42 }; return 0
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
## Case: uncaught-keeps-message
id: "av.parity.1087.uncaught-keeps-message"
tags: ["av-language-parity", "plan-1087"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn inner() int { fail("parity-error-xyz") }
fn main() int { return inner() }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "run"
message-contains: "parity-error-xyz"

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
## Case: catch-index-error
id: "av.parity.1087.catch-index-error"
tags: ["av-language-parity", "plan-1087"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let xs = [1]
 try { return xs[9] } catch e { return 42 }
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
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | 同一受检语言错误在调用、模块、闭包和动态调用间传播；catch payload/message/span与错误类别稳定，不用固定trap码代替。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | try/catch/finally、rethrow、.?/默认抑制、??、break/continue/return的控制优先级与finally恰一次；stack unwind释放每个scope owner。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 旧越界/null/Python等可捕获普通错误按已定模式接入；取消/预算/权限/基础设施失败保宿主边界，保首次红/环境与未执行分类。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-ERROR。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态：

- [x] **T-01** work：见上表唯一任务合同。
- [x] **T-02** work：见上表唯一任务合同。
- [x] **T-03** work：见上表唯一任务合同。
- [x] **T-04** work：见上表唯一任务合同。
- [x] **T-05** work：见上表唯一任务合同。
- [x] **T-06** work：见上表唯一任务合同。
- [x] **T-07** review：见上表唯一任务合同。
- [ ] **T-08** merge：见上表唯一任务合同。

当前入口：在 §9 指定的冻结提交上收齐最后一轮适用组、复审和落主证据。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- 2026-11-20 r3 基线取证。普通 AV debug worker 按专属树基面构建，exit 0、22.597s。逐条回放 25 份路线主承接、5 份 1094 主承接的旧脚本角色、4 份 §6 草案，共 34 次子执行，约 1.660s（准备、发现和清理未另测）。25 份旧作者字节的 SHA16 全部与路线表相符。catch-callee-fail 终止于 lang.error/boom，catch-index-error 终止于 av.trap.list-index，finally-return-order 返回 42，uncaught-keeps-message 返回 lang.error/parity-error-xyz、main.at:22..40。旧脚本五例普通入口分别报 source.use-rs.syntax / source.syntax，保其 1094 脚本与协议 owner。t01-baseline.json SHA256 a5eeac8c0bb6c1bbe4c16daa06bb7bc536794dae95490f6b86c0266878aab840。
- 2026-11-20 r3 实施检查点 75e8d86dcaad0f66f52a661525541611430bf62b。普通 Auto 匹配的 Unit 臂按效果求值，保留选臂调用，不为 Unit 分配本地槽；旧 04.008/009、16.020 在基面分别报 mir.unit-placement，修正后输出精确 small、high、propagated error。增唯一作者 errors/catch-finally.case.ad 与 suite 引用；13 个旧义务沿原路径、源和 VM oracle 映射，非 AV #ignore；16.010 VM 4000000 哨兵及原 VM 预期保留，本轮 AV 实测 lang.error/error 与 29..32，沿既有 965 及路线差异。worker 重建 8.988s，34 例二次回放约 1.6s。unit-arm-probe.json SHA256 c0bc847624f201811ae00b92522196cfd996f41e83e0120584fe53975e9ce144。
- 固定组 cargo t case-av-language-parity 的独立准备 107.878s（只准备 AV 与 case-parent；首次两次误加 --debug-tools 在参数校验退出 2，无用例执行）。实际 --run-prepared 父、子均退出 0，260 PASS、121 SKIP、0 FAIL。执行子墙钟 21.174s，整命令 24.715s，发现和清理无独立计时。结果 SHA256 b32e38f504049818a20c412a5293482acd63c45d0192e43092f0e74b8bd8fea2。新增 15 条 AV 检查逐条耗时见下行；其余 12 份主承接与内联角色仍待承接完成。完整冻结组、T-01..08、ERROR-CLASS 与 UNWIND-RESOURCE 仍待。
  av.parity.1087.finally-return-order=656 ms、av.parity.1087.uncaught-keeps-message=602 ms、legacy.04_control_flow.007_asn_upper=632 ms、legacy.04_control_flow.008_is_or_pattern=588 ms、legacy.04_control_flow.009_is_or_groups=526 ms、legacy.04_control_flow.011_as_float_to_int=553 ms、legacy.04_control_flow.012_to_str=596 ms、legacy.05_loops.001_for_range=583 ms、legacy.05_loops.002_range_inclusive=559 ms、legacy.05_loops.005_loop_keyword=546 ms、legacy.16_option_result.009_propagate_ok=595 ms、legacy.16_option_result.010_propagate_err=578 ms、legacy.16_option_result.011_coalesce_some=539 ms、legacy.16_option_result.012_coalesce_none=603 ms、legacy.16_option_result.020_result_propagate=627 ms

- 2026-11-20 r3 检查点：受检错误边。普通 Auto 的 try/catch/finally 带成对的受检控制局部，经共享 HIR 降为 MIR/ABC 的正常与异常后继。异常后继写消息并禁止读取未产生的调用结果，弹出被展开的帧，保留 handler 帧。已回放直接调用、闭包、跨模块及原始 lib.at 重抛位置；finally 轨迹为 middle / leaf / outer，后一 finally 的新错误覆盖先前错误。下标负值和越界可交 handler，原窄入口保持 trap 路径。普通语言 lowering 身份独立更新。原生对照实跑包括 3 层释放后二次调用、帧额度绕过 catch、重抛保索引位置、损坏异常边读取结果在 MIR 拒绝；最初两条位置断言红，分别修复失败终止符的独立位置锚和下标右括号，随后四例通过。
- Python 实际 provider 对照。独立 debug 构建同源 CPython provider，执行器通过已准入会话的 error_stage/str_word 取证。捕获 list.pop 真实 IndexError，值 42、无输出、class=4、原调用 span=80..91；未捕获保持 ForeignStatus；同一会话再次调用成功 42。恢复对照使用真实 list append/pop。原 1094 的五条 .as 脚本协议角色仍由 1094 承接。
- 原样旧源迁移初步对账。20/25 份、21 条原 VM check 的旧整文档 SHA16、Auto 正文及原 oracle 均匹配，VM check 在唯一活跃作者侧阶段 #ignore。普通 AV 逐条从共享 worker 取值；剩余 04.005/006、05.003、16.007/008 在实际原源上仍拒绝，见 §10。该检查点对应语料/代码的最终固定组准备及正式复审待完成。
  证据 SHA256 checkpoint2-control.json=ab4c3cce19562f5563ada9eeab5dcc9944fe0b030dbc0a8989705af2da22bd15，checkpoint4-scope.json=5054791ad7227e13b5392afe468d1f1275d4f0fec25a16f61b6f819993ffca30，checkpoint5-baseline.json=914ccfb96f444ceefea222f29101177404cc3b385bd54badf84f4c09a52e0caf，spans-test-run.json=44b69c6fa38e1ec9fc5a2a40ea29fc9d0a18330c4b627f668ac928dfe6b7f0ba，py-probe-run5.log=8d0beabe61c358225eeb190a7fab7a962471bf1aa119a8f0538348a83657f072，checkpoint5-migration-audit.json=44a1202ff8645e2f2f84a97a8b80cb6f2b83192b9761bf86632ed5dda6e59704。

### 2026-11-20 r4 施工与复审问题收据

- 用户已选择保留现行 AV 语义，四份旧源与独立诊断见 §5。PLAN-1087 主承接 25 份原文档、27 条旧 VM 检查。逐份比对整文档 SHA16、Auto 源字节、原 check ID、oracle 和 `final-newline`。同源作者有 43 个 Case，旧 VM 检查阶段 `#ignore`。99_script_err 五份原文档的主 owner 为 PLAN-1094，原源码原条件保留。其旧 `use.rs`、with 和动态 Python 组合在普通 portable AV 前门的观测为未准入。PLAN-1087 单独验证当前 provider 能力。旧 VM 内联 `plan781_e01_cross_frame_raise_catch_restores_caller_frame` 的调用者恢复义务，由三层 owned 栈及二次调用见证承接。
- 起始代码检查点 `75e8d86dcaad0f66f52a661525541611430bf62b`。期间 PLAN-1046 合入主分支，实施树两提交重放到 `defff852a0d5d6c232f23f391220e1d4b18bbdd3`，未发生冲突。AAC 使用现有 `generate-grammar` 从共享语法生成冻结数据，未修改生成器和窄 profile 的原预期。
- f57e60749dbd06b36b261059bab03227a17dd89c 首轮固定组准备 116.783s，运行 105.171s，退出 1。公共用例 286 PASS、135 SKIP，源组 931 PASS、1 FAIL，语义组 294 PASS、1 继承 FAIL，AV 1483 PASS、1 FAIL。其余共享组实际退出见最终验证对照。本轮源码组首红为 Aggregates 的 grammar ID 由 f31c1e33… 漂移到 89e3e64c…，AV 首红为指令契约仍断言原 61 项。定向冻结语法身份测试编译 21.68s 后用时 0.35s，所有冻结面逐字相等。指令契约保留原计数断言并加入三个术语的完整枚举。语义组 `sema_boundary_case_files_executed_status_honest` 复现既有 D-1037-01，缺 `status: executed` 的文档与基面一致。
- 复审负例 `try { fail("original error") } catch { rethrow() }` 在嵌套跨函数重抛时，旧候选返回原消息，位置误报整个 try 的 20..70。实作新 checked `RAISE` 边之后，初次原样重放暴露 HIR 错判受保护循环后继不可达。修补 producer 与 checker 的同一边分类，重放成功定位最初消息字节 31..47，类别 explicit。参数求值失败、finally 失败覆盖返回和列表迭代中展开到外层 catch 均返回 42，最后一项实际输入用具名 list 绑定。定向 AV conformance 六个测试通过，测试合计 0.05s，准备 22.91s。索引 payload 随后按旧 null-family 契约从泛化消息修为原有的 `IndexError: index 9 out of range`。冻结提交 `05c72e33867f91d14a706bc319c63d386f679b1f`。
- 普通 AV 父端控制。激活正确 AV 检查并故意写错阶段忽略的 VM 预期，得到 AV PASS 和 VM SKIP，退出 0。AV 预期 42 改 41 时实际返回 42，FAIL，退出 1。删除 AV required 检查和重复 case 各退出 2，并给出对应入门诊断。运行使用 `auto-case run --target av`，原目录文件未修改。
- ERROR-CLASS 补充证据。列表越界用精确类别 ListIndex 与 `xs[9]` 原字节切片验证，宿主拒授权、I/O 故障、在内层执行期间取消和帧限制四条实际反例保持原分类且 catch 次数为 0。Python 用独立显式预备的真实 CPython provider，在同一 `PySession` 验证空列表 pop 捕获 Python 类别 4、原 pop 字节 80..91、返回 42，未捕获对照为 ForeignStatus、下次 append/pop 返回 42。实际捕获、未捕获与恢复三次分别 0.085s、0.012s、0.013s，预备 0.738s，工具 rustc 1.98.0。UNWIND-RESOURCE 断言实际进入六次子调用帧、第一次出错后第二次调用得到 42、arena 的分配和释放 ID 集完全相等，最终 live=0。补充观测使用真实 AV 运行器与同一受检源码。
- 修复命令 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core --test conformance plan981_grammar_scope::p981_grammar_ids_match_the_recorded_frozen_values -- --exact` 以及 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av --test conformance plan1087_errors -- --test-threads=1`（两者均把 target 指向本计划材料目录）。后续同一冻结验证和重放的结果见下表。

### 2026-11-20 r4 冻结验证与施工交接

生产代码冻结在 `05c72e33867f91d14a706bc319c63d386f679b1f`。随后 `ee8c25b1d7f569c157d51cb572f0953f406a22d9` 只将公共 AV 负下标用例中的消息检查从 9 改为实际下标 −1。同一 Auto 源的其余行、所有旧 VM 源与预期、所有 Rust 测试输入、编译器与 worker 字节未变。公共 AV 组在后一个提交重跑。其余七组保留原生产代码提交的收据。

| 组 | PASS | FAIL / 排除 | 运行子墙钟 |
|---|---:|---|---:|
| case-av-language-parity | 288 | 0 FAIL，135 SKIP | 35.322s 含固定组驱动 |
| ace-source | 932 | 1 原 ignore | 8.094s |
| ace-syntax | 159 | 0 FAIL，按原性能条件选组 | 3.499s |
| ace-semantics | 294 | 1 既有 FAIL，10 按原条件过滤 | 5.923s |
| ace-av | 1483，另有 2 串行重放 PASS | 首次 2 次平台墙钟拒绝，41 条按原性能及 AV-only/Windows 条件不执行 | 94.557s，定向重放 12.439s |
| ace-lowering | 9 | 0 FAIL | 0.968s |
| ace-artifacts | 43 | 1 原 ignore，按原性能条件选组 | 6.294s |
| auto-test-case | 157 | 0 FAIL | 2.243s |

八组固定门准备 86.418s，等待本机空闲 55.404s，整执行 171.456s，首次退出 1。额外等待 45.232s 后公共 AV 单组重新准备 3.228s，执行退出 0。发现阶段 ace-source 0.563、ace-syntax 0.995、ace-semantics 3.055、ace-av 1.800、ace-lowering 0.435、ace-artifacts 0.544、auto-test-case 1.599s，公共 AV 结构查询与清理未独立计时。Windows 平台未执行适用的 Linux/WSL 执行腿及混合外部 backend 目标。沿用原样本和用例条件。独立 AAC `aac1046_grammar_data_identity_binding` 子例 0.21s，含预备编译 26.293s，退出 0，逐 profile byte-exact。

初次公共组 287 PASS、1 FAIL、135 SKIP，唯一失败为负下标 case 的签名消息断言写成 9、输入为 −1。只纠正该独立断言。AV 组的两个已有用例 `kv_zip_1064::v03_kv_wire_oracle_and_reopen_av`、`kv_zip_1064::v03_zip_negative_corrupt_bomb_and_preserve_av` 在并发冻结组分别 23.714s、24.585s 触发 platform call ExecutionWall。在相同准备清单的 AV 组 archive 上用 nextest `--test-threads 1` 精确重放，两例分别 7.717s、2.849s，子出 0。既有红 D-1037-01 仍由 PLAN-1062 持有，基面和冻结提交均未改对应语义测试或旧文档。

43 个本包 AV 检查父侧单项 0.520–1.296s。补充 Rust 六例依次为授权/取消/I/O 0.141s、帧限 0.127s、同帧原点重抛 0.150s、三层释放与二次调用 0.131s、列表越界原点 0.164s、拒绝异常边读结果 0.172s。同帧起点、跨模块与索引的父收据分别定位至 main.at:31..47、lib.at:31..41 和 main.at:37..43。原调用失败案定位 main.at:22..28，迭代展开定位 main.at:21..38。父侧还断言类型/处理器进入计数及所有 check 确实被选择。

组输入清单 `final-prepared/prepared.json` SHA256 `702e1c3dc06ece9c8537e1bfd361a3b91cc732f443c0809d131509636399e506`；原固定组 `final-run/results.json` SHA256 `c08c7c6c35167f57c9abb9f92819a12e4fcf20209b7e044eee0a4058cc88ac12`；最后公共组 SHA256 `9130cc5e8cbb52453ea48da62c17ad0c882511f48dac748cfea2fc48522a9b57`；串行 KV/ZIP 重放 SHA256 `17dc76776627ae955531175fc7125792b071604271e179efcbd073a7276b6265`；父侧断言摘要 SHA256 `c1d7fe57a77ce633e6eb6a62900be00b1830d10dccec7481d47c6a8f14b3ae50`。

逐案父侧请求时间（毫秒，`legacy.` 或 `av.parity.1087.` 前缀在各 ID 中省略）：

- `uncaught-keeps-message` 1269.1，`rethrow` 757.8，`override` 793.5，`null-through-call` 746.4，`nested-finally` 712.4，`module-rethrow` 932.4，`module` 843.9，`loop-exits` 723.6，`local-origin-rethrow` 1007.5，`local-borrow-unwind` 731.1，`list-index-callee` 780.2，`lazy-defaults` 928.3，`iterator-finally-and-catch` 915.4，`finally-return-order` 1102.4，`conditional-list-effects` 1025.4，`closure` 679.9，`catch-index-error` 726.1，`catch-callee-fail` 748.5。
- `16_option_result.020_result_propagate` 704.6，`16_option_result.012_coalesce_none` 735.6，`16_option_result.011_coalesce_some` 705.3，`16_option_result.010_propagate_err` 708.5，`16_option_result.009_propagate_ok` 734.3，`16_option_result.008_propagate_none` 1115.6，`16_option_result.007_propagate_some` 1223.6，`05_loops.005_loop_keyword` 519.8，`05_loops.004_for_each_object` 709.8，`05_loops.003_range_literal` 1091.5，`05_loops.002_range_inclusive` 535.2，`05_loops.001_for_range` 849.2，`04_control_flow.013_to_int_from_str` 849.2，`04_control_flow.012_to_str` 738.3，`04_control_flow.011_as_float_to_int` 1017.4，`04_control_flow.010_as_int_to_float` 798.8，`04_control_flow.009_is_or_groups` 1096.3，`04_control_flow.008_is_or_pattern` 1193.7，`04_control_flow.007_asn_upper` 1296.0，`04_control_flow.006_is_stmt` 992.9，`04_control_flow.005_if_in_array` 1143.4，`04_control_flow.004_if_with_bool` 671.8，`04_control_flow.003_if_else_if` 685.1，`04_control_flow.002_if_false` 703.9，`04_control_flow.001_if_true` 716.0。

施工交接 `stage: work | plan_id: PLAN-1087 | plan_revision: r4 | outcome: pass | worktree: D:/autostack/.wt/lang-1087/auto-lang | base_commit: defff852a0d5d6c232f23f391220e1d4b18bbdd3 | code_commit: ee8c25b1d7f569c157d51cb572f0953f406a22d9 | dependency: auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | task_ids: T-01..06 | next: review`。

### 2026-11-20 r4 独立复审

同一会话分出独立核验阶段，使用原专属树和提交冻结输入。以构建产物、源码与实际父端收据重新判定。未另设 reviewer。

| 验收 | 裁定 | 验证 |
|---|---|---|
| AC-AV-ONLY | PASS | 固定组实际选择普通 AV recipe。错误的非 AV oracle 在 #ignore 仍 SKIP，错误 AV oracle 实际 FAIL，缺 required 和重复 ID 各退出 2。对照登记 135 SKIP。 |
| AC-FC、AC-04 | PASS | 从原路径和 byte hash 逐条读 25 份原源、27 条旧检查，再核 canonical 文件和生产映射的原 ID、oracle 字节、条件和 owner。两个现有消费者经 CanonicalVmCases 查相同映射。99_script_err 五份的生产持有见 PLAN-1094。 |
| AC-01 | PASS | 共享输入经源前门与受检 IR/ABC 到 AV。核正例调用、闭包、模块、lazy-defaults、嵌套 finally、循环退出和条件列表。额外选择嵌套参数求值和 finally 主动错误，均完成祖先 catch，实际返回 42。 |
| AC-02 | PASS | 实验重抛反例用原消息和来源的位置精确断言。同帧与不同模块各检查真实原点。核 -1 与 9 的独立越界消息、Option/值位 is 拒绝。资源、授权、取消、host I/O 和非语言 provider 失败保持原分类。 |
| AC-03 | PASS | ERROR-CLASS 从父端收据取类别及源位置。UNWIND-RESOURCE 核实际进入六次调用、三层 owned 退出后第二次成功、alloc/free ID 等集和 live=0。破坏 MIR 异常边后的结果读由验证器拒绝。 |
| AC-05 | PASS | 逐段检查 AutoDown → 普通报文 → 共享 CST/绑定 → HIR LoopUnwind → MIR/ABC → AV 真实 dispatch → 父端判定。新错误状态不进 legacy dispatcher。Python observer 按签章 provider、manifest、授权及 generation 复核。plain worker 沿现有 portable 能力。 |
| AC-06 | PASS | 以冻结代码、完整共享组和适用目标收据复核；两个 I/O 墙钟首红在相同 AV 档串行通过。D-1037-01 原文档 blob 为 `c4874997b7f528c6e6a1faf3efb3d463a934b8`，原断言较基面未改，保留 1 个已登记失败。新例单项在 10 秒内。 |
| AC-07 | PASS | 对 f57 的复审问题见上。修复后的最终提交保持窄语法字节身份、有效 MIR 边和原错误位置，检查未封存的生产语义、可复现输入与规范差异。 |

遗漏、延后和 workaround 核查。01..06 的旧语义差异和 shared gates 按已裁定规则具名。普通生成器的挂起和关闭生命周期由 PLAN-1089 消费本接口。旧 Python 动态/with/宿主组合五原例由 PLAN-1094 承接。非 AV 后端继续按 r3 所定 owner 和独立 oracle 处理。语义组既有失败保持原债务，未新增未批准延后。检查了冗余 parser、case 名特判、源位置伪造、测试预期误配、临时输出和本包新增编译告警；未见未闭修复项。

已核 SD-01、SD-02、SD-03、SD-FC 与现有目录。拟沉淀普通入口的检查边、原错误定位、null 和 provider 分类、独立旧 VM 义务及后继边界。reviewed spec delta SHA256 `da322d2de5fe38e5754bf5106ea515268cc206840bb92c263701750884664549`。复审代码与合规后续 Docs-only 交付分开绑定。

`stage: review | review_scope: final | reviewer: 同一会话独立核验阶段 | plan_id: PLAN-1087 | plan_revision: r4 | outcome: pass | worktree: D:/autostack/.wt/lang-1087/auto-lang | branch: plan-1087-dev | reviewed_commit: ee8c25b1d7f569c157d51cb572f0953f406a22d9 | base_commit: defff852a0d5d6c232f23f391220e1d4b18bbdd3 | dependency: auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | next: merge`。有效冻结证据复用，剩余合入、落地烟测、归档和清理。

### 2026-11-20 r4 合入归档收据

- prepared、landed、canonical_verified。代码复审 `ee8c25b1d7f569c157d51cb572f0953f406a22d9`。规范与原导航独立提交 `ad9a3a2f2f6348b7a553a3dc397f9a0cd2c143c4`，从主分支 `defff852a0d5d6c232f23f391220e1d4b18bbdd3` 经 `git merge --ff-only plan-1087-dev` 快进。代码复审提交到交付提交只有六份文档差异。
- 主检出用同组已准备的 AV worker 与父端执行 `auto-case run --suite …/language.suite.ad --case av.parity.1087.local-origin-rethrow --target av`，1 PASS、135 条既有 SKIP、287 条本次未选，退出 0，1.789s。
- 最终代码上的真实 Python provider 再次准备，probe 编译 24.634s、provider 热准备 0.877s。捕获 IndexError 类别 4 指向 80..91 并返回 42（0.094s），原未捕获 ForeignStatus（0.017s），同会话恢复 42（0.013s），退出 0。provider digest `fa14f9a3ac486ebe95ae7c3af46e9cbff7d94ac834421ae001690aac90f4a308`。
- 规范文件 SHA256 已从落地主检出复核。language-parity.md `d46f3cb9db6a494a8d2f035785f44868fde19c02063be0fd80a0cb219f820b77`，m2-control-effects.md `39b6382d5846b70e1011387cfdc14adacc7522673d2541fc005cbf9ef9261cc5`，null-family.md `e5fb0cc99f6b95feab3773f2e339033313fbfa631a7ce0770419663e6192d9eb`，feature-organized-corpus.md `03f5d0729d67c49bf7478e6c0231b40731572c45de81c84a6a7630206f60efb8`。
- 周期回归收据截止编号 1098，时间为 2026-11-20T00:00:00Z。1087 不满足每五包触发，收据时间未到 48h。独立批量门本次未到期。
- 交付形态为源码、语料和规范。主检出已有 debug AV worker 于 2026-11-20 12:44:11 构建，release 于 2026-11-19 11:10:13 构建；本次源码合入尚未刷新这些产物。其它会话的主检出文档及计划编辑仍由原会话拥有。
- archived（delivered）。档案路径为 `docs/plans/archive/1087-av-parity-errors-and-unwinding.md`，仅 T-08 清理为本收据的后续操作。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
2026-11-20 实施中的语义裁定。现规范只允许非穷尽的语句 is 继续执行，值位必须穷尽。原 04.006 的尾部 is 命中时同时要求 stdout 与脚本结果 10。原 16.007/008 在非 Option 返回类型的脚本使用 Option .?，旧值 42/-1，现绑定统一报 source.propagate-incompatible。保留旧 oracle 与原条件，现已有用户裁定，三例按 §5 具名差异以及原 VM oracle 双轨验收。05.003 由已交 PLAN-962 范围规则处理。04.005 的原条件列表源码已迁入，父侧断言原值。条件全假的新例验证右侧惰性求值及源序。
