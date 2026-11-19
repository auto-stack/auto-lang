---
plan_id: PLAN-1079
status: archived
feature_name: AV标量、转换与文本语言语义补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: [PLAN-1078]
supersedes_spec_components: [docs/specs/ace-migration/m2-numbers.md, docs/specs/ace-migration/m2-dynamic.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1079] AV标量、转换与文本语言语义补齐

## 0. 变更摘要

### 2026-11-19 r4 · work 施工（`/goal 完成计划1079`）

用户 `/goal` 目标授权整链（work → review → bounded repair → merge）。T-01..T-05 已在唯一专属树
`D:/autostack/.wt/lang-1079/auto-lang`（分支 `plan-1079-dev`，基面 `b2d830a2c`）实施并真机验证：
commit `fea1a45ae`（Auto 面基数/分隔符字面量解码、i32 类型化常量绑定、两份 canonical 语料、
68 行迁移映射、位型观测测试）与 `fe039c686`（四份 spec 增量预置）。`cargo t case-av-language-parity`
PASS 8/8；`cargo test -p ace-av --test conformance plan1079` 6/6。四项具名边界（复合赋值语句、
char→Str 装箱、计算借用视图局部量、owned→str 实参）已在 §9/§5 写 owner 与关闭条件，不缩分母。
T-06 的固定组并集因其它会话满载（100% CPU）等待空闲后执行。

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`numbers/literals-conversions.case.ad`、`text/interpolation-unicode.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-01**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

将已交数值/文本能力接入普通Auto入口，并补旧合法字面量、运算/转换与FStr组合，保留已批准数值差异。

本包交付`IF-AV-PARITY-SCALAR`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- L-01..05/L-49完整已定宽度、后缀、运算符优先级、短路、复合赋值、浮点±0/NaN/舍入/越界，保已裁定语义。
- char/str/Str/CStr与嵌入NUL、UTF-8视图/长度区分、转义、FStr全部已支持插值形及副作用恰一次；通用Unicode算法留标准库owner，语言装箱/显示与编码正确性归本包。
- 继承972/1064已有text/bytes原语，禁止创建第二套字串或用C strlen实现Auto长度。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1079/auto-lang`，分支`plan-1079-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1079/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

961已交宽数值/checked转换，不是缺整个数值系统；但普通组合/profile、完整FStr显示和旧拼写仍需证明。旧int=i64、隐式混宽晋升、截断cast与ACE既定int=i32/checked规则冲突，属于显式差异，不能照抄旧expected。

既有来源与边界：

- [scalar.case.ad](../../crates/auto-lang/test/cases/ace-m2/documents/scalar.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：01_basics；02_bit_ops；08_strings；11_compound_ops；12_type_coercion；21_conv；25_method_u64；26_str_method_on_heap；99_short_circuit；99_str_natives。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [auto-source.at](../../experimental/ace/grammars/auto-source.at) | `9e5248b2a6aeacd74d2ef61c77c41aae4181a47dd7e3f4d5fdd8c78e970ab25a` |
| [project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs) | `2223bfbbf79279bd4fc88596febc6dd455a477bd793832084ab8ffcba2ff38cc` |
| [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [av.rs](../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

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
- [project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/numbers/literals-conversions.case.ad`、`crates/auto-lang/test/cases/language/text/interpolation-unicode.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-numbers.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-dynamic.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1079-原文档清单)静态主承接 **68份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1079.wide-exact` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1079.text-nul-and-format` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1079.overflow-is-not-wrap` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | run: int-overflow | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1079.mixed-width-explicit` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `NAN-ZERO` | AC-03/AC-04 | work / T-04..05 | 以既有typed值/bit observer补NaN、-0、f32舍入、窄化反例，文本显示相同不能代替位型观察。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `FSTR-ORDER` | AC-03/AC-04 | work / T-04..05 | 插值表达式修改共享trace，验证源序与一次求值；bool/char/浮点/owned Str显示逐类型确定格式，未定格式T-01先冻结。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: wide-exact
id: "av.parity.1079.wide-exact"
tags: ["av-language-parity", "plan-1079"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let n = 9007199254740993i64
 if n - 9007199254740992i64 == 1i64 { return 42 }
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
## Case: text-nul-and-format
id: "av.parity.1079.text-nul-and-format"
tags: ["av-language-parity", "plan-1079"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let s = "A中😀\0".clone()
 let f = f"v=${20 + 22}"
 if s.len() != 4 || s.len_bytes() != 9 { return 0 }
 if f != "v=42" { return 0 }
 return 42
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
## Case: overflow-is-not-wrap
id: "av.parity.1079.overflow-is-not-wrap"
tags: ["av-language-parity", "plan-1079"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 var n = 2147483647
 return n + 1
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "run"
message-contains: "int-overflow"

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
## Case: mixed-width-explicit
id: "av.parity.1079.mixed-width-explicit"
tags: ["av-language-parity", "plan-1079"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int {
 let n = 40i64 + 2i64
 return n.to(int)
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
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [x] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [x] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [x] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [x] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span（**span 轴具名未观测**：父侧 probe receipt 的 `spans: []` 不序列化源 span，与 1078 复审同一边界；本包以正确 stage + 精确 code 断言：`source.type-mismatch`/`source.literal-range` 在 parse/verify 阶段、`av.trap.int-overflow` 在 run 阶段，并以 oracle 变异证明可判红）；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
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
| T-02 | work | T-01 | L-01..05/L-49完整已定宽度、后缀、运算符优先级、短路、复合赋值、浮点±0/NaN/舍入/越界，保已裁定语义。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | char/str/Str/CStr与嵌入NUL、UTF-8视图/长度区分、转义、FStr全部已支持插值形及副作用恰一次；通用Unicode算法留标准库owner，语言装箱/显示与编码正确性归本包。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 继承972/1064已有text/bytes原语，禁止创建第二套字串或用C strlen实现Auto长度。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-SCALAR。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（2026-11-19 work 施工，见 §9 施工收据）：

- [x] **T-01** work：冻结 68 份旧文档（原 id/check/oracle/源 SHA16 逐行入 legacy-map.tsv）+ 有界全链探测（8 批探针，见 §9）：Auto 面缺基数/分隔符字面量解码、i32 类型化常量渲染；具名边界 4 项（复合赋值语句、char→Str 装箱、计算借用视图局部量、owned→str 实参）。
- [x] **T-02** work：数值 canonical `language/numbers/literals-conversions.case.ad`（3 case）真机 PASS：i64 超 2^53 精确、`0xFFu8`/`0b1010u8`/`1_000_000`、`to(int)`/`as(i64)`/`as(u64)`、`& | ^` 同层、`.shl/.shr`、整除取余、i32 MAX+1 → `av.trap.int-overflow`。复合赋值语句按具名边界交 PLAN-1080（同一 checked 加法经 `x = x + 1` 实测）。
- [x] **T-03** work：文本 canonical `language/text/interpolation-unicode.case.ad`（2 case）真机 PASS：`len`=4/`len_bytes`=9 带 NUL 与 astral、`char_at`+`to_codepoint`、`\n`/`\t\\\"`/`\u{4e2d}` 转义、FStr 源序恰一次（exact stdout `A\nB\n`）、`Str` 相等、负数显示。`char.to(Str)`/借用视图局部量/`str` 形参按具名边界交 975/1086。
- [x] **T-04** work：§6 两个补充 ID 以父侧类型化/位观察落实（`av/tests/cases/plan1079_language_semantics.rs` 6 例全 PASS，0.03s）：NAN-ZERO（运行时 `x % ±0.0` 得 NaN 自不等/无序/不 trap、`-0.0` 经取负/相加/f64→f32 收窄保符号位、f64→f32 ties-to-even、checked 转换与 `source.literal-range`/`source.type-mismatch`）；FSTR-ORDER 由 exact stdout 字节见证。
- [x] **T-05** work：canonical 文件为唯一活跃作者（suite 只追加引用）；`legacy-map.tsv` 追加 68 行（mapped 22 / research-hold 34 / no-such-shape 12，原义务不缩）；元控制：oracle 变异 → FAIL `expected "41", actual "Some(42)"`；全 AV 检查 `#ignore` 视图 → 组级 NOT_APPLICABLE 且单独运行 exit=1。
- [x] **T-06** work：冻结提交 `83d829d5f`（含复审修复；前序 `fea1a45ae`/`fe039c686`/`7992ce56e`）；固定组并集 `ace-source/ace-syntax/ace-semantics/ace-av/ace-lowering` 已在冻结提交上执行并逐组记录（§9 T-06 段），修复后受影响范围 `case-av-language-parity`/`ace-source`/`ace-syntax` 复跑全 PASS（gate_passed=True）。
- [x] **T-07** review：见上表唯一任务合同。
- [x] **T-08** merge：见上表唯一任务合同。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- 2026-11-19 `stage: merge | plan_id: PLAN-1079 | plan_revision: 3 | outcome: pass | delivery_commit: bedad3cff（ff-only，master 尖端） | reviewed_commit: 83d829d5f | archived: docs/plans/archive/1079-av-parity-scalars-and-text.md | checkpoints: prepared / landed / canonical_verified / archived / cleaned | next: done`

  **prepared**：复审基线 `83d829d5f`（review pass，含 §5.4）；冻结增量 = SD-01/02/03/FC（4 份 canonical Spec）+ 2 份 canonical 语料 + `legacy-map.tsv`（78 行）+ 2 处实现修复 + 7 例 AV oracle；交付提交 `bedad3cff`。

  **landed**：worktree 内 rebase 到当时 master 并 ff-only 落主，master 尖端 = `bedad3cff` = dev 尖端，**无 merge commit**。主检出在本次合并期间被其它会话推进两次（`c29961996`→`9fb68d61e`），因此 rebase 做了两轮；每轮均以 `git range-diff` 证明补丁等价（全部 `=`）并确认新 master 对我影响面为空（`git diff --stat c29961996..9fb68d61e -- <本包路径>` 无输出）。哈希映射（reviewed → landed）：
  `fea1a45ae→b3075c9a6`、`fe039c686→9d2acf91a`、`7992ce56e→8917a85a7`、`83d829d5f→bedad3cff`。

  **canonical_verified**（按 **git blob** 身份核对，权威口径）：7 个交付文件的 blob 在复审提交与落地提交中**逐一相同**——
  `language-parity.md 45f0c0693d`、`m2-numbers.md e027be5098`、`m2-dynamic.md d431e88d86`、
  `feature-organized-corpus.md 9b8d44d00a`、`legacy-map.tsv 190cc21988`、
  `literals-conversions.case.ad 4ae030bf5f`、`interpolation-unicode.case.ad 67d917b3b0`；
  §5.4 L 行验收表已在落主内容中。**收据修正**：复审记录里的 `spec_inputs` SHA16 是对 worktree 的
  CRLF 副本计算的，而交付 blob 为 git 归一后的 LF，因此那组数值与落主哈希不相等（内容一致，见上面的
  blob 同一性）——以 blob ID 为准，本条为口径更正，不是内容漂移。

  **落地门**：rebase 后在 `bedad3cff` 上复跑 `cargo t case-av-language-parity` **PASS**（8/8，run-acco8lim）；
  修复轮的并集证据（`ace-source` 923 / `ace-syntax` 159 / `ace-semantics` 294-1既有红 / `ace-av` 1433-2既有红 /
  `ace-lowering` 9）在无路径重叠的前提下复用（未见任何本包路径的新改动）。`tests/test-groups.toml` 未改动，
  临时控制组已移除。

  **archived**：计划文件移入 `docs/plans/archive/` 并 `status: archived`，与本次合并簿记一并入库
  （该文件此前在主检出为未跟踪 WIP，随本提交首次纳入版本控制）。

  **cleaned**：`wt-guard.ps1` 输出 clean → 移除工作树与 `plan-1079-dev` 分支 → 删除空的组目录 →
  删除 `plan-artifacts/plan-1079/` 全量 scope（记录已折入本文件）→ 刷新 `.retention/policy.json`。
  构建中间产物随工作树删除，未回收。

  **deployment observation**：本包只改 ACE（`experimental/ace`）与测试语料，不改 `auto` CLI 主程序；
  `target/debug/auto.exe` 与 `auto` 包装脚本不涉及本包代码，无需重建/重启，无发布制品受影响。

  **批量回归**：`docs/plans/.last-batch-regression.json` 收据为 1064 / `2026-11-16T10:04Z`，
  距今 >48h 且其后有多次合并 → **DUE**。PLAN-1022 的收口已把该 DUE 移交 `auto-plan-regress`，
  本合并不另起竞争实例，记 `batch_gate: pending(owner=auto-plan-regress, single-instance on main)`。

  **cleaned 验证（执行后实测，2026-11-19 20:3x）**：`wt-guard.ps1` 对 `auto-lang` 与 `auto-down` 两棵工作组树均输出
  `clean`；`git worktree remove` exit 0（两棵）；`git branch -d plan-1079-dev` → "Deleted branch … (was bedad3cff)"；
  组目录 `D:/autostack/.wt/lang-1079` 删除后 `Test-Path=False`；`plan-artifacts/plan-1079` 删除 **1.13 MB / 144 文件**后
  `Test-Path=False`；`.retention/policy.json` 刷新为 129 条（新增本计划 `deleted-at-merge (no artifact holds)`）；
  `git worktree list` 与 `git branch --list 'plan-1079*'` 均无残留；主检出本包路径 `git status` 为空（归档文件已跟踪）。
  最终 master 尖端 `a925dda9d`（簿记）→ 交付提交 `bedad3cff`。
  **具名 hold**：本计划 §4/§5 引用的三份 AV 系列设计文档（`docs/design/strategy/av-autovm-language-parity.md`、
  `docs/design/strategy/av-autovm-legacy-case-map.md`、`docs/design/testing/ace-feature-organized-autodown-corpus.md`）
  在主检出仍为**未跟踪 WIP**（与 1078 归档时同状）。属 AV 系列规划会话的在途文档（1079–1096 共用），
  merge 只提交本计划文件，不代其它会话入库；owner=AV 系列规划会话/下一 AVP 批次，关闭条件=该系列下一次交付时一并入库。
- 2026-11-19 `stage: goal | plan_id: PLAN-1079 | plan_revision: r3 | round: 6 | from: reviewed | to: archived | status: archived | outcome: pass | delivery_commit: bedad3cff | archive: docs/plans/archive/1079-av-parity-scalars-and-text.md | cleanup: worktree/branch/group removed (guard clean), plan-artifacts 1.13MB/144 files deleted, registry refreshed | batch_gate: pending(owner=auto-plan-regress, DUE) | next: done`——目标「完成计划1079」达成：work→review(needs_fix)→work(repair)→review(pass)→merge(ff-only)/archive/clean 全链完成。
- 2026-11-19 `stage: goal | plan_id: PLAN-1079 | plan_revision: r3 | round: 5 | from: execution_done | to: reviewed | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1079/auto-lang | commit: 83d829d5f | next: merge`——本轮完成 T-07 复审复核（F-01/F-02 均 closed，全 AC 复核 pass），计划达 reviewed；下一阶段 merge（T-08）。
- 2026-11-19 `stage: review | review_scope: final | re_review_of: round-2 needs_fix (F-01/F-02) | plan_id: PLAN-1079 | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1079/auto-lang | branch: plan-1079-dev | reviewed_commit: 83d829d5f | base_commit: b2d830a2c | dependency_revisions: PLAN-1078 archived (d4125815a/47618eb2f); auto-down detached@72b0970 | spec_inputs: language-parity.md (sha16 3830BA4E81FD993E, incl. §5.4), m2-numbers.md (6EA8EAE0164706BE), m2-dynamic.md (35DEA99F3997B47A), feature-organized-corpus.md (99071BA8F75956AB), legacy-map.tsv (22001BFDB5753FC0), literals-conversions.case.ad (79F647AE4F9DE805), interpolation-unicode.case.ad (A2085CB8E25FCADC) | pending_closeout_gates: merge-owned frozen affected-union re-run + `cargo t all` due-check | next: merge`

  **审阅者身份与独立性限制**：仍是实施会话（`reviewer_identity: lead (implementation session)`），未被授权新建独立 agent；判定从工件重建（diff/真实运行/receipt/脚本化一致性核对），并显式保留该限制。

  **修复差异范围**（`7992ce56e..83d829d5f`，5 文件）：`cst/project.rs`（F-02：body 切分对所有基数保留 `.`）、`plan1079_language_semantics.rs`（F-02 断言 + `compile_diagnostic`）、`literals-conversions.case.ad`（L-02/L-03 的 float/double/byte/uint 字面量断言 13–16）、`legacy-map.tsv`（+5 L 行）、`language-parity.md`（+§5.4）。

  **发现关闭复核（逐条重放，非采信收据）**：

  | 发现 | 复核方法（本轮真实执行） | 结果 |
  |---|---|---|
  | F-01（AC-04，L 行映射） | 脚本核对：5 行存在、12 列齐、状态枚举合法、canonical 引用全部命中 corpus（8 个 Canonical ID）；spec §5.4 引用的 8 个 case ID 全部存在，文中提及的测试函数名全部在 `plan1079_language_semantics.rs` 中定义 | **closed**（`PROBLEMS: none`） |
  | F-02（AC-02，分数诊断/死分支） | 独立探针：`0x1F.5` → `radix literal \`0x1F.5\` cannot carry a fraction`；`0b1010.5` → 同款文案（均为 `source.literal-range`）；接受控制 `0xFFu8`/`1.5`/`1.5d`/`5b`/`7u` 全部仍成功 | **closed** |

  **修复回归（受影响范围，冻结提交 `83d829d5f` 上）**：`case-av-language-parity` PASS(8)、
  `ace-source` PASS(923)、`ace-syntax` PASS(159)，`gate_passed=True`，wall 815.9s；
  `cargo test -p ace-av --test conformance plan1079` 7/7 PASS（0.02s，含 F-02 两条断言）。
  `ace-source`/`ace-syntax` 执行数与修复前完全相同（923/159），未见新增红。

  **验收映射（本轮全量复核，AC-07）**：

  | AC | 结论 | 证据（冻结提交） |
  |---|---|---|
  | AC-AV-ONLY | pass | AV 视图 `--check av` 选中 8 行、非 AV 行 NOT_RUN/SKIP；非 AV 未 prepare |
  | AC-FC | pass | 唯一作者（2 新 canonical 文件 + suite 引用）；68 行文档映射 + 5 行 L 映射，原义务不缩；`#ignore` 只在非 AV 检查 |
  | AC-01 | pass | 5 例 AV 真机 PASS（528–568ms），oracle 为退出值/双流/冻结 trap code |
  | AC-02 | pass | 反例给具名 stage+code（本轮 7 条探针复核，含 2 条 radix-fraction）；span 轴仍具名未观测（父侧 `spans: []`） |
  | AC-03 | pass | 7 例类型化/位型 oracle（0.02s）+ FStr exact stdout `A\nB\n` |
  | AC-04 | pass | L-01/02/03/05/49 映射行 + §5.4 表，状态可回溯到实际 case/测试 |
  | AC-05 | pass | 共用 `compile_language_source`；无 case 名特判；legacy dispatch=0 |
  | AC-06 | pass | 冻结提交上固定组并集逐组记录；既有红 1+2 由基面重放/既有债务（D-1037-01/D-1098-01）归因 |
  | AC-07 | pass | 本记录即独立上下文核验输出（限制已声明） |
  | AC-08 | 待 merge | — |

  **用例耗时（≤10s 预算）**：本包 5 例 AV 546/528/549/568/539ms（含 worker 会话）；
  1078 三例 532/565/580ms；`plan1079` Rust 7 例合计 0.02s。修复未新增 case，只加断言。

  **遗留（不阻 merge，均具名 owner）**：`D-1079-01`（1078 两行 source-SHA 列口径）、
  `D-1079-02`（词法器吞 `.成员`）、`D-1037-01`（ace-semantics 既有红）、
  `D-1098-01`（host_console 两例陈旧 oracle，明确禁止 ignore/改 expected 充绿）。
  merge 需带走的门：冻结并集的最终复跑（本轮已绿，若 merge 前无代码变更可复用）与
  `cargo t all` 周期检查。

  **下一步**：merge（T-08）——落主已审 Spec、归档、清理工作树与材料；计划文件与三份
  AV 系列设计文档目前在主检出为未跟踪 WIP，merge 时必须一并入库或由用户裁定归属。
- 2026-11-19 `stage: goal | plan_id: PLAN-1079 | plan_revision: r3 | round: 3 | from: executing | to: execution_done | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1079/auto-lang | commit: 83d829d5f | next: review`——本轮执行 work 修复轮（F-01 L 行映射、F-02 分数诊断），受影响组复跑 `case-av-language-parity`(8)/`ace-source`(923)/`ace-syntax`(159) 全 PASS，7 例 Rust oracle 0.07s 全 PASS；T-01/T-02/T-05/T-06 与对应 AC 重新闭合。
- 2026-11-19 `stage: work | repair_of: review round-2 (F-01/F-02) | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1079/auto-lang | branch: plan-1079-dev | base: b2d830a2c | code: 83d829d5f | tasks: T-01/T-02/T-05/T-06 (reopened → closed) | gates: case-av-language-parity PASS(8) / ace-source PASS(923) / ace-syntax PASS(159) / gate_passed=True wall=815.9s；plan1079 tests 7/7 0.07s | next: review (T-07 re-review on 83d829d5f)`

  **F-01 关闭（AC-04/AC-FC，L 行映射）**：
  - finding：AVP-01 的 L-01/02/03/05/49 无映射行。
  - 不变量：路线 §6 指派的每条 L 行都必须有「原行 → canonical case/check → 预期 → owner → 状态」，且状态可由实际运行证据回溯。
  - 修复：`language/migration/legacy-map.tsv` 追加 5 行（同 1078 L 行表格式，12 列；含原行名/原形/条件/canonical/状态/owner）；`docs/specs/auto-av/language-parity.md` 新增 §5.4 表，逐行给实际证据名（case ID 或 Rust 测试名）。L-03 的 CStr 子项显式 `no-such-shape`（ACE `TypeWord` 无 CStr），L-01 显式记 legacy `int=i64` 具名差异，不在状态里含糊。
  - 当前结果：映射表 78 行、列数全 12；其中 L-01/02/05/49 = mapped、L-03 = mapped（CStr 子项 no-such-shape）。
  - 剩余覆盖边界：L-03 的 CStr 属 1093 foreign surface（不充 PASS）；L-02 的 NaN payload 保留政策仍只按 IEEE 记录，无语言级 payload 构造/观察（NaN 无字面量语法）。

  **F-02 关闭（AC-02，分数诊断/死分支）**：
  - finding：1079 的切分改动使 `has_dot && radix != 10` 不可达，`0x1F.5` 退化为 "suffix `.5`" 文案。
  - 不变量：非十进制基数携带分点时，必须命中冻结的 "radix literal cannot carry a fraction" 具名拒绝，而不是通用后缀错误。
  - 修复：`decode_typed_literal` 的 body 切分对**所有**基数保留 `.`（`split` 不再按 radix 排除点），使该分支重新可达；`plan1079_language_semantics.rs::conversions_are_checked_and_mixed_widths_are_source_rejections` 增加两条断言（`0x1F.5`、`0b1010.5` → `source.literal-range` 且 message 含 "fraction"）。
  - 当前结果：7/7 测试 PASS（0.07s），含上述两条；`case-av-language-parity` 8/8 PASS。
  - 剩余覆盖边界：radix + 合法后缀形（`0xFFu8`）仍按既有用例覆盖；`0x1Fb` 的「基数位优先」裁定不变（§5.1）。

  **探针归类记录（非产品缺陷）**：复审与修复中曾用探针判定 `1.5f32.as(f64) != 1.5d` 为真，进一步定位发现是**探针逻辑写反**（相等时返回 1），实际加宽与比较均正确；同批还观测到 `1.5f32.as(...)`（不加括号）被词法器并入数字 token 而报 `source.unsupported`——该词法"数字体吞 `.成员`"现象早于本包，属语法制面已知项，已记入 KNOWN-DEBT（D-1079-02），不影响 `(1.5f32).as(...)` 的规范写法。

  **同时在修复轮补充的覆盖**（不扩大 scope，只补齐 L-02/L-03 的 canonical 证据）：`numbers/literals-conversions.case.ad` 的 `mixed-width-explicit` 增加 `1.5f32`/`1.5d` 解码、`(1.5f32).as(f64) == 1.5d` 精确加宽、`-0.0d == 0.0d`、`5b` byte 字面量、`7u` uint 后缀共 4 条断言（返回码 13–16 可定位）。

- 2026-11-19 `stage: goal | plan_id: PLAN-1079 | plan_revision: r3 | round: 2 | from: execution_done | to: executing | status: archived | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1079/auto-lang | commit: 7992ce56e | next: work`——本轮执行 review 阶段（T-07），判定 needs_fix（F-01 L 行映射缺失 / F-02 分数诊断退化+死分支；F-03 记债），受影响任务 T-01/T-02/T-05/T-06 与 AC-01/02/04/06/FC 已重开，`current_step` 重算为 2（T-03/T-04 保持完成）。

- 2026-11-19 `stage: review | review_scope: final | plan_id: PLAN-1079 | plan_revision: 3 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1079/auto-lang | branch: plan-1079-dev | reviewed_commit: 7992ce56e | base_commit: b2d830a2c | dependency_revisions: PLAN-1078 archived (d4125815a/47618eb2f); group auto-down detached@72b0970 | spec_inputs: docs/specs/{auto-av/language-parity.md, ace-migration/m2-numbers.md, ace-migration/m2-dynamic.md, auto-test-case/feature-organized-corpus.md} @7992ce56e | pending_closeout_gates: merge-owned frozen affected-union re-run + `cargo t all` due-check | next: work`

  **审阅者身份与独立性限制**：本会话即实施会话，`reviewer_identity: lead (implementation session)`；没有独立角色/模型可用，也未被授权新建 agent。因此本判定完全从工件重建（diff、真实运行、receipt、基面重放），不采信实施叙述；**显式记录该限制**：这是一次同会话自审，若用户要求独立上下文复审，应另行安排。

  **基线**：HEAD `7992ce56e`，工作树 clean（`git status --porcelain` 空）。复核中发现并已排除一次"陈旧工件"陷阱：为核对既有红而把两处实现文件回退到基面后**恢复**时，文件 mtime 早于构建产物，`cargo` 复用旧 rlib 导致本包 2 例假红；touch 后重建即 7/7 PASS，并重跑 `case-av-language-parity` PASS。该现象只发生在审阅者的临时回退/恢复操作中，交付树本身从未处于该状态。

  **验收映射（AC → 证据）**：

  | AC | 结论 | 证据 |
  |---|---|---|
  | AC-AV-ONLY | pass | AV 组 `--check av` 只选 av 行；非 AV 行 `#ignore` → SKIP 且不 prepare（1078 FC-08 机制复用）；AV 视图不含 legacy-reference |
  | AC-FC | partial | 唯一作者/suite 引用/68 行迁移 ✓；但本包自身的 L 行映射缺失（见 F-01） |
  | AC-01 | pass | 5 例 AV 真机 PASS（退出值/双流独立 oracle），单例 0.495–0.535s |
  | AC-02 | partial | 反例均给可辨识 code+stage（`source.literal-range`/`source.type-mismatch`/`av.trap.int-overflow`），审阅者 9 条独立边界探针全部按预期拒绝或接受；span 轴未序列化（已具名）。分数诊断退化见 F-02 |
  | AC-03 | pass | NAN-ZERO/FSTR-ORDER 由 7 例类型化测试（0.02s）+ exact stdout `A\nB\n` 落实 |
  | AC-04 | **fail** | L-01/02/03/05/49 在本包无 owner/预期/状态映射行（F-01） |
  | AC-05 | pass | 共用 `compile_language_source` 前门；无 case 名特判；legacy dispatch=0 |
  | AC-06 | pass（待重跑） | 冻结提交上固定组并集已执行并逐组记录；F-01/F-02 修复后需在该（新）冻结提交复跑受影响组 |
  | AC-07 | —（本记录即 review 输出） | |
  | AC-08 | 待 merge | |

  **审阅者独立边界探针（9 例，`auto-case` 直跑，未写入交付树）**：`0x`（空体）、`0x1F.5`、
  `0xFFFFFFFFFF`（超 i32）、`0b2`（非法二进制位）、`0x1Fi128`（未知后缀）、
  `0xFFFFFFFFFFFFFFFFu64`（MAX，接受）、`0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF`（超 u128/i32）、
  `0x_FF`（前导分隔符，接受）、`0xFFu8`（接受）——全部按预期：6 例给 `source.literal-range`
  具名诊断，3 例真实运行成功。

  **工件完整性核对**：（a）canonical 语料无重复 Case ID；（b）`legacy-map.tsv` 中 5 个
  canonical 引用全部指向存在的 Case；（c）逐行重算 68 行的 Auto 源 SHA16，**68/68 与表中一致**；
  （d）同时发现 1078 遗留的 2 行 aavm2（`legacy.aavm2.001/002_*`）在 `source_sha16` 列填的是
  **文档整篇**哈希而非 Auto 源哈希（`4a9bf96f…`/`b5b50899…` 为文档 SHA；按同口径重算为
  `ce7731ae…`/`defffe73…`）——非本包行，记入 KNOWN-DEBT，不在本包修。

  **新增用例行为与耗时**（均 ≤10s）：本包 5 例 AutoDown AV 0.528/0.533/0.535/0.519/0.531s
  （含 worker 会话）；`plan1079_language_semantics` 7 例合计 0.02s（`--release` 测试二进制）。
  本轮审阅执行：7 例 Rust 复跑 0.02s、`case-av-language-parity` 复跑 PASS、9 例边界探针
  约 9s；未执行 product 全量回归（归 merge 的冻结并集）。

  **发现（F-01 / F-02 必修；F-03 记债）**：

  **F-01（major，违反 AC-04/AC-FC）——本包 L 行映射缺失。**
  违反规则：AC-04「本包原case/check与L行映射有唯一owner/预期/状态」；路线设计
  `docs/design/strategy/av-autovm-language-parity.md` §6 明确把 **L-01、L-02、L-03（与 AVP-15）、
  L-05、L-49** 指派给 AVP-01（=本计划）。实际：`language/migration/legacy-map.tsv` 只有 1078
  留下的 L-04/45/46 三行，本包五条 L 行**没有任何映射行**，spec 里也没有逐行状态。
  预期 vs 观察：预期每个 L 行有「原行 → canonical case/check → 预期 → owner → 状态」；
  观察为零行。影响面：AC-04、AC-FC；同类消费者是后续 AVP 批次与 1068/985 的完整门。
  更正（工作范围，不改语义）：按 1078 的 L 行表格式追加 5 行（旧行名/原形/条件列写实际、
  canonical 指向本包 canonical Case 或既有 1078 Case、状态取 mapped/partial/no-such-shape、
  缺失子项如 `byte` 字面量 parser-only 与 `CStr` 面显式具名 owner），并在
  `docs/specs/auto-av/language-parity.md` §5 增加一张 L 行验收状态表（每行给 actual 证据：
  case 名或 Rust 测试名）。关闭 oracle：映射行存在且每个 L 行的状态能对应到实际运行证据；
  `cargo t case-av-language-parity` 与 `cargo test -p ace-av --test conformance plan1079` 复跑绿。

  **F-02（minor，代码卫生/诊断精度）——分数诊断退化 + 死分支。**
  违反规则：AC-02 要求可辨识错误；`decode_typed_literal` 里 `has_dot && radix != 10`
  的「radix literal cannot carry a fraction」在 1079 切分改动后**不可达**（16/2 进制的 `.`
  已被 `split` 划进 suffix），实测 `0x1F.5` 报 `suffix \`.5\` is not an integer literal suffix`。
  拒绝行为仍成立（不是 silent accept），但精确理由丢失且留下死分支。
  更正：让 `split` 在非十进制基数下也把 `.` 归入 body（使该分支重新可达并保留原文案），
  或删除该死分支并把 suffix 诊断改写为 "radix literal cannot carry a fraction" 的等价具名文案；
  二者择一，附一条探针断言（`0x1F.5` → `source.literal-range` 且 message 含 fraction）。
  关闭 oracle：探针断言绿 + 该文件无不可达分支（审查者以 `0x1F.5` 复跑确认）。

  **F-03（info，记债，不阻本包）——1078 遗留 2 行 source-SHA 列存的是文档哈希**（见上文
  (d)）。owner：PLAN-1078 归档后的维护者/下一 AVP 批次；关闭条件：按 Auto 源口径重算或
  在行内注明该列刻意记文档哈希。已记入 `docs/plans/KNOWN-DEBT-AND-RISKS.md`。

  **保留有效的既有证据**（不因本轮重跑而重复）：1078 的 3 例 AV 历史绿、`ace-source`/
  `ace-syntax`/`ace-lowering` 组 PASS（923/159/9）、`ace-av` 1433/1435 与 `ace-semantics`
  294/295 的既有红归因（基面重放已证：回退本包两处实现文件后 `host_console::*` 两例
  同样失败）。这些组的代码面在 F-01/F-02 修复后需按受影响范围复跑：F-02 触及
  `cst/project.rs`（字面量解码）→ 复跑 `case-av-language-parity` + `ace-source` +
  `ace-syntax` + `cargo test -p ace-av --test conformance plan1079`；F-01 纯数据/文档 →
  `case-av-language-parity`。

  **运行环境事实**：本轮机器有其它会话 `cargo nextest archive`（~90–100% CPU）与一次
  `STATUS_ACCESS_VIOLATION` 链接崩溃（已记于 T-06 段），复跑均绿。

  **下一步**：返回 work 修 F-01/F-02（同一专属树），重新提交后按上述受影响范围复跑，
  再 request 本阶段复审。

- 2026-11-19 `stage: goal | plan_id: PLAN-1079 | plan_revision: r3 | round: 1 | from: drafting | to: execution_done | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1079/auto-lang | commit: 7992ce56e | next: review`——目标「完成计划1079」作为整链授权记录于 §4（work→review→bounded repair→merge）；本轮只执行 work 阶段，未跳阶段。

- 2026-11-19 **T-06 固定组并集（冻结提交上真机执行）**：`stage: work | outcome: pass(条件红已归因) | plan_revision: 3 | commits: fea1a45ae / fe039c686 / 7992ce56e | 命令: cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering | wall: 720.1s`

  | 组 | 结果 | 执行数 | 说明 |
  |---|---|---|---|
  | `case-av-language-parity` | **PASS**（复跑） | 8 | 首轮为工具准备期 rustc 链接崩溃（见环境事件），复跑 PASS；单例 495–535ms |
  | `ace-source` | PASS | 923 | |
  | `ace-syntax` | PASS | 159 | |
  | `ace-semantics` | FAIL（既有红 1） | 295（294 PASS / 10 skipped / 3.279s） | `sema_boundary_audit::sema_boundary_case_files_executed_status_honest` → `ac-semantics/cache-roundtrip.document.case.ad` 未带 `status: executed`。**归因**：本 Plan 对 `crates/auto-lang/test/cases/ac-semantics/**`、`lowering`、`av-core/src/av.rs`、`host_console.rs` 的 diff 全为空（`git diff --stat b2d830a2c HEAD -- <paths>` 无输出），且基面 `b2d830a2c` 该文件同样无 `status:` 行；1078 复审已把「ac-semantics cache-roundtrip 在途 T-06 status」记为既有红。 |
  | `ace-av` | FAIL（既有红 2） | 1435（1433 PASS / 22 skipped / 106.004s） | `host_console::av_console_entry_selection_cannot_bypass_main_policy`、`host_console::av_console_owned_front_door_enforces_console_entry_only`。**归因（基面重放）**：把本包仅有的两处实现改动（`core/src/source/adapter.rs`、`core/src/source/cst/project.rs`）回退到 `b2d830a2c` 后重建，同两条用例以同样断言失败（`Option::unwrap() on None` / `Result::unwrap_err() on Ok`），其余 19 条 `host_console::*` 仍 PASS → 非本包触面。恢复后复跑本包 7 例全 PASS。 |
  | `ace-lowering` | PASS | 9 | |

  **环境事件（不判代码失败）**：首轮 `case-av-language-parity` 在 av 工具准备期
  rustc 链接 `av-worker` 报 `STATUS_ACCESS_VIOLATION (0xc0000005)`（`run-m15i5knk/0/tools/av-build.log`），
  同目录重试即 `Finished release in 24.09s`；机器当时有其它会话 ~90–100% CPU 的
  `cargo nextest archive`。复跑该组 PASS，两处记录并存、不互相替代。

  **新增用例耗时**（均含 worker 会话，≤10s 预算）：本包 5 例 AV 0.528/0.533/0.535/0.519/0.531s；
  `plan1079_language_semantics` 7 例合计 0.02s。准备/执行/清理：单组 gate 720.1s 中含一次工具重建。

  **IF-AV-PARITY-SCALAR（本包交付摘要）**：
  - 普通 AV 入口新增准入：完整宽度/后缀/`0x`/`0b`/`_` 字面量面（与 Numeric 面同一解码器）、
    i32 类型化常量可绑定；i64 超 2^53 精确、checked 溢出 trap、显式转换、`& | ^` 同层、
    `.shl/.shr`、整除/取余；文本 `len`（标量）与 `len_bytes`（UTF-8 字节）分离、嵌入 NUL
    为普通标量、`char_at`+`to_codepoint`、转义表、FStr 源序恰一次、`Str` 相等、i32 十进制显示（含负号）。
  - 消费者契约：`cargo t case-av-language-parity` = `language.suite.ad` 的 AV 视图（`--check av`）；
    零选中必红（NOT_APPLICABLE 且单独运行 exit=1）；非 AV 检查 `#ignore`/NOT_RUN 不充 PASS。
  - canonical 作者：`language/numbers/literals-conversions.case.ad`、`language/text/interpolation-unicode.case.ad`；
    迁移映射 68 行（mapped 22 / research-hold 34 / no-such-shape 12），原义务不缩。
  - 具名边界（owner/关闭条件见 `docs/specs/auto-av/language-parity.md` §5.3 与本轮 §9 探测表）：复合赋值语句（1080）、
    `char.to(Str)`（975 后续）、计算借用视图局部量/`str` 形参收 owned 实参（1086）。

- 2026-11-19 work 施工收据（`stage: work | outcome: pass(待 T-06 固定组并集) | plan_revision: 3`）：
  `worktree: D:/autostack/.wt/lang-1079/auto-lang | branch: plan-1079-dev | base: b2d830a2c | code: fea1a45ae + fe039c686（spec 预置） | dependency: PLAN-1078 已归档（d4125815a/47618eb2f）；组内 auto-down detached@72b0970 只读 | tasks: T-01..T-05 | next: T-06`

  **T-01 有界全链探测（真实运行，8 批探针，receipts 在 `plan-artifacts/plan-1079/probe*/`）**：环境
  `auto-case.exe run --suite <probe> --check av` + `AC_AV_CASE_WORKER=<av-worker.exe>`。结论：

  | 形态 | 探测结果 | 处置 |
  |---|---|---|
  | `40i64` / `42i32` / `1.5` / `c.to_codepoint()` / `'\n'` / `"\u{4e2d}"` / `Str == str` / `x = 2` | 可用 | 直接进 canonical case |
  | `0xFF` / `0xF` / `0xFFu8` | `source.literal-range`「carries no digits」 | **修复**（基数体切分 + 按基数判数位） |
  | `0b1010` / `1_000_000`（无后缀 affix-num） | Atom bind 内部错误「constant value must be an integer value」 | **修复**（i32 `TypedConst` 渲染 `{value}i32`） |
  | `acc += 22` | `source.syntax`「expected expression, found `=`」 | 具名边界 → PLAN-1080 |
  | `c.to(Str)` / `c.to(str)` | 前门拒绝（legacy 回退报「`fn` is not part…」） | 具名边界 → PLAN-975 |
  | `let t = s.slice(1, 3)` | `verify.local-type`（aggregates 局部量只收 owned `Str`） | 具名边界 → PLAN-1086 |
  | `fn f(s str)` 收 owned `Str` 实参 | `source.type-mismatch`（`expected str, found Str`） | 具名边界 → PLAN-1086 |
  | `r"a\b"` | `text.invalid-escape`（无 raw string 字面量） | 旧义务记 `no-such-shape` |

  **T-02..T-05 真实结果**：`cargo t case-av-language-parity` **PASS**（8/8：1078 三例 + 本包五例，
  单例 451–535ms）。`cargo test -p ace-av --test conformance plan1079` **6/6 PASS**（0.03s：
  NaN/-0 位型、ties-to-even 收窄、checked 转换/字面量越界/混宽拒绝、基数与分隔符、NUL 长度）。
  `legacy-map.tsv` 68 行（mapped 22 / research-hold 34 / no-such-shape 12；原 id/check/oracle/源 SHA16 保留）。
  元控制：oracle 变异（成功例 `exit: 41`）→ FAIL `exit[exit] mismatch: expected "41", actual "Some(42)"`；
  全 AV 检查 `#ignore` 的视图单独运行 → `NOT_APPLICABLE` + exit=1（零选中不绿）。
  期望形态修订：console 族错误/trap 预期必须带显式 code（`stage: "run"` + `code: "av.trap.int-overflow"`），
  §6 草案的 `message-contains` 单独不满足 `validate_console_oracle`；实施按 `code` 落地。

  **与 §6 草案的具名差异（不是放松验收）**：草案 4 例按 T-03 的 char/转义面拆为 5 例（新增
  `av.parity.1079.text-literal-escapes`，同 canonical 文件、同 owner）；trap 例的预期形态由
  `message-contains` 改为显式 `code`；`mixed-width-explicit` 的累加用 `acc = acc + 22`（复合赋值语句
  见具名边界）；文本例的 stdout 预期为 `A\nB\n`（FStr 源序恰一次的真实字节见证）。

  **环境/状态事实**：（a）`scripts/new-wt-group.sh` 本次在 WSL bash 下运行，写入 `/mnt/d/...`
  gitdir 链接，Windows git 无法解析；已按 4 个文件（两个 worktree 的 `.git` 与两个 admin `gitdir`）
  改写为 Windows 路径并验证 `git rev-parse --show-toplevel`/`branch --show-current` 正确——后续会话
  建议用 Windows Git Bash 或直接 `git worktree add`。（b）本计划文件与路线/case-map/共享语料三份
  设计文档在主检出为**未跟踪**WIP（`git ls-files` 空），worktree 内没有它们；T-01 只读引用主检出副本，
  merge 前需与 1078 一样把它们一并入库，或由用户裁定归属。（c）主检出有其它会话的 134 个未跟踪文件
  与大量 `M docs/**`，本包不改动它们。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
