---
plan_id: PLAN-1092
status: archived
completion_kind: delivered
feature_name: AV语言标注、元数据与反射接面补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 3
current_step: 7
total_steps: 8
depends_on: [PLAN-1078, PLAN-1088, PLAN-1091]
supersedes_spec_components: [docs/specs/ace-migration/m2-generics.md, docs/specs/ace-migration/m2-metaprogramming.md, docs/specs/ace-migration/m2-tools.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1092] AV语言标注、元数据与反射接面补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`metadata/attributes.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-14**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

统一旧合法语言属性的解析/绑定/消费者路由，并补普通工程所需元数据查询。

本包交付`IF-AV-PARITY-METADATA`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- #[with(T as Spec)]、#[test]、#[vm]/#[c]/#[rs]、pub/export/api、文件#[script]/#[rust]、task single/async按真实语言与target语义逐项分类，未知/位置错/重复属性具名诊断。
- 属性只经统一Schema/CST与现有constraint/effect/ABI数据路由，#[with]复用generic批见证，#[test]复用971runner，外库标注复用967，不另写注解解释器。
- FieldCount/HasField与原可达metadata消费者、泛型/跨模块缓存失效；字段遍历/自动serde若不是旧AVM已交功能，列后续库/CT增强，不阻本路线纯legacy齐平。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1092/auto-lang`，分支`plan-1092-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1092/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-21 用户以 `/goal 完成计划1092` 授权按 r3 合同执行、独立复审、合同内修复、合入归档与清理。AV-ONLY 阶段边界继续适用。

2026-11-21 用户补充授权由 PLAN-1092 在同组 `auto-web` 工作树完成 `#[api]` 到既有 AV 服务的接线和实际路由验证。仅扩展真实消费端所需实现与验证，沿用既有服务运输、授权及业务契约。语言与 Web 两仓分别提交、复审、合入并对账。

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

D-963-C3 #[with(...)]仍缺共享属性语法；FieldCount/HasField已交不等任意运行期反射。旧#[derive]/#[serde]/#[api]中有目标透传元数据，AV不得伪执行成Rust宏。

既有来源与边界：

- [source_specs_generic_963.rs](../../../experimental/ace/crates/core/tests/cases/source_specs_generic_963.rs)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：旧parser annotation与971 test discovery Rust tests；声明/文件/field属性实际外部consumer。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [decls.rs](../../../crates/auto-lang-core/src/parser/decls.rs) | `cbc25bdb24d97c9af39698e0387d8419b1213747749477f380df5c72f3d28165` |
| [auto-source.at](../../../experimental/ace/grammars/auto-source.at) | `9e5248b2a6aeacd74d2ef61c77c41aae4181a47dd7e3f4d5fdd8c78e970ab25a` |
| [project.rs](../../../experimental/ace/crates/core/src/source/cst/project.rs) | `2223bfbbf79279bd4fc88596febc6dd455a477bd793832084ab8ffcba2ff38cc` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |

硬依赖：PLAN-1078, PLAN-1088, PLAN-1091；实际外部接口：971测试工具/967 ABI对应正式切片。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [decls.rs](../../../crates/auto-lang-core/src/parser/decls.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [auto-source.at](../../../experimental/ace/grammars/auto-source.at)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [project.rs](../../../experimental/ace/crates/core/src/source/cst/project.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [source_foreign.rs](../../../experimental/ace/crates/abi/src/source_foreign.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/metadata/attributes.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-generics.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/ace-migration/m2-metaprogramming.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-DISCOVERY | modify | `docs/specs/ace-migration/m2-tools.md` | 显式 ordinary AV 项目读取 `#[test]`，使用 971 实际发现、执行和报告。 | 同一工具保留窄面契约 | AC-01、AC-03、AC-08 |
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

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1092-原文档清单)静态主承接 **0份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1092.with-constraint-spelling` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1092.test-attribute-source` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1092.unknown-attribute` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.attribute-unknown | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `ATTR-ROUTING` | AC-03/AC-04 | work / T-04..05 | 实际#{}/文件pragma、test/public/export/foreign每个consumer读取正确metadata与源span；Rust专属透传不能静默丢，也不虚构AV执行语义。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `META-IDENTITY` | AC-03/AC-04 | work / T-04..05 | record字段数/名称与约束/属性变更后，依赖查询和普通运行均失效；无关函数不重算，parent实际观察。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: with-constraint-spelling
id: "av.parity.1092.with-constraint-spelling"
tags: ["av-language-parity", "plan-1092"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
spec S { fn value() int }
type N as S { n int; fn value() int { return .n } }
#[with(T as S)]
fn get(x T) int { return x.value() }
fn main() int { return get(N(42)) }
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
旧parser确有此属性形式；T-01确认参数引入/约束合同，规范等价于共享泛型约束，不以删除注解实现兼容。
`````

`````autodown
## Case: test-attribute-source
id: "av.parity.1092.test-attribute-source"
tags: ["av-language-parity", "plan-1092"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
#[test]
fn answer_test() { let n = 40 + 2; assert(n == 42) }
fn main() int { answer_test(); return 42 }
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
还须通过971的实际发现/执行入口只执行一次；普通main调用不代test discovery证明。
`````

`````autodown
## Case: unknown-attribute
id: "av.parity.1092.unknown-attribute"
tags: ["av-language-parity", "plan-1092"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
#[parity_unknown_attribute]
fn main() int { return 42 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "resolve"
code: "bind/source.attribute-unknown"

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
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-abi
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
| T-02 | work | T-01 | #[with(T as Spec)]、#[test]、#[vm]/#[c]/#[rs]、pub/export/api、文件#[script]/#[rust]、task single/async按真实语言与target语义逐项分类，未知/位置错/重复属性具名诊断。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 属性只经统一Schema/CST与现有constraint/effect/ABI数据路由，#[with]复用generic批见证，#[test]复用971runner，外库标注复用967，不另写注解解释器。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | FieldCount/HasField与原可达metadata消费者、泛型/跨模块缓存失效；字段遍历/自动serde若不是旧AVM已交功能，列后续库/CT增强，不阻本路线纯legacy齐平。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-METADATA。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

任务状态：

- [x] **T-01** work：核定正式依赖身份、原用例和消费者。普通 AV 原样草案、同义显式泛型、反例和调度探针的基线已记录于 §9。现有消费者、边界和源位置列于下表。
- [x] **T-02** work：普通 AV 属性和文件模式进入共享语法，保留完整声明事实与具名诊断。
- [x] **T-03** work：复用 971 测试发现、包可见性、调度器、受检 Web 服务入口和 967 ABI 所有权。
- [x] **T-04** work：字段查询、泛型实例隔离、跨模块重开失效与 AV-only 反控已定向通过。
- [x] **T-05** work：唯一公共作者与迁移数据已提交，逐例定向 AV 判决和元控制见 §9。
- [x] **T-06** work：见上表唯一任务合同。
- [x] **T-07** review：见上表唯一任务合同。
- [ ] **T-08** merge：见上表唯一任务合同。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- `stage: goal | plan_id: PLAN-1092 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | worktree: D:/autostack/.wt/lang-1092/auto-lang | commit: 282e6de9b3674937f3b48d19a868b9b3ef36f15e | next: work`。继承 §4 端到端授权。组内只读 auto-down `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。1078 的普通 AV recipe/固定组提交 `3842c826c`、`3e14cadda`，1088 泛型 `50fa8e018`、1091 CT `2ee399ed0` 均已在基面。专属材料目录已登记。T-01 开始。

### 2026-11-21 实施检查点

- 基线 `cargo build --manifest-path experimental/ace/Cargo.toml -p ace-av --bin av-worker` debug 37.992s / exit 0。普通 worker 实跑六个探针（单个 0.041–0.184s）。无属性与 `<T has S>` 两份控制返回 42。r3 草案的 `with`、`test`、未知属性与额外 `single` 均在 `text/source.syntax` 拒绝 `#`。原始结果 `baseline-observations.json` SHA256 `034f3fc6a99cf9aa6de2a0671a9308255f1917ef32b69fc1f98e7cbcbeeceb7d`。
- 候选 `18fb0ac1687ae71db59eaeb62264a8f178579ef0` 接入既有 Schema/CST，`with` 归一到泛型约束，`test` 源事实送到 971 发现和独立执行入口。仅显式普通 AV 项目能力选标注发现，971 冻结窄面仍用其签名规则。未知属性的 worker 实际字段为 `stage: bind, code: source.attribute-unknown`，公共 Case 用现有绑定→`resolve` 阶段映射及独立 code 字段。
- canonical 文档的五条 Auto 源已通过独立进程 worker 定向探针。按文档顺序为 0.048522 / 0.045743 / 0.036909 / 0.038487 / 0.036712s，两正例各返回 42，三反例验具名绑定诊断和字节范围，双流均为空。候选 worker debug 准备 13.05s，`initial-cases.json` SHA256 `bd61b4c9b8f9abb2b1ebaf70b24625c847f8a15a40a8f68ff779d00266a95a05`。公共父侧执行留待正式验证。
- `cargo test -p auto-lang-ace-host --test p971_tool_face -- --test-threads=1` debug 重放 3/3，构建 13.25s，运行 0.52s。新用例单独同版二进制实测 0.290039s（含启动与清理）：一条标注声明只跑一次，helper 不被选中，故意改坏断言得到一个失败结果。保留首轮模块未放行 `assert`、第二轮尾表达式推断失败，现均已接现有 ordinary resolver。目标语法冻结组 `profiles_927` 8/8，39 未选，构建 4.45s，运行 0.78s，整命令 5.455s。发现、清理无单独读数。
- T-01 消费者核对：旧 VM `08_strings/011_vm_stub_panic` 已由 PLAN-1079 具名迁移，新映射保原 VM error-presence。961/963 等窄面保既有拒绝。982 的 AV 服务入口实际在 auto-web 的 `AvServiceEngine`，当前使用旧 `try_full_parse` 取得路由并剥除属性行。该入口切换尚未完成。1090 普通 AV 调度由独立在途工作树提供；本计划在该正式交付前继续可独立的属性与元数据工作。
- 当前 T-01 至 T-06 的完整验收仍在执行。其它属性、缓存改动与元控制、冻结固定组、复审与合入均待完成。

### 2026-11-21 输入与消费者清单

| 原输入或事实 | 实际消费者与核定边界 |
|---|---|
| `source_spec_super_assoc_963.rs` 的 O-963-SA11、D-963-C3、A2R/A2C 原 `with_constraint.at` | 旧窄面具名拒绝照旧。普通 AV 用已交 spec 与泛型查询，原计划 `#[with(T as S)]` 与同义声明同归名义见证。目标语言专属 trait fixture 的判决仍归其原目标。 |
| `p971_tool_face.rs`，D-971-B-01 的 helper 假阳性 | 971 explicit AV project API 的 check 与 test lane，共用真实声明与源 span。普通 AV 选中明确标记的声明，窄面冻结签名判定继续保留。 |
| `decls.rs`、`statements.rs`、`mode.rs`，旧 `08_strings/011_vm_stub_panic`、bootstrap 转译 fixture | 普通 AV 选择 VM-compatible 函数/类型，包可见性走已交 binder。字符串 stub 已归 1079，无本包迁移副本。Bootstrap 转译用例保原 owner 与冻结原源。文件模式优先级 `rust > script > 扩展名`，Python 动态语义归 PLAN-1094。 |
| 967 `source_foreign`、610 ABI 声明、312/982 API 输入、A2R derive/serde fixture | 声明从同一 CST 给出所有者、span 与 ABI/目标事实。外库仍取 967 provider 权限和身份，Rust 子检查保待启用。HTTP 由 auto-web 的既有 AV host 实际消费，已获同组接线授权。 |
| 旧 `Task.singleton_send`、Design12、968/1090 AV 调度 | 旧静态请求复用 64 槽懒启动邮箱。任务生命周期和未来值走已交调度器；1090 代码基面 `c4ecdf12a` 已同步，完成收据 `b99fc0ff3` 已在主分支。 |
| 936/941 的 FieldCount、HasField、字段选择和缓存边界 | 真实元数据查询由 CT 侧服务共有记录信息。受管存储重开变更验证按字段数、缺变有/有变缺、拒绝与恢复观察。普通 AV 跨模块选择子仍须经过类型可见性绑定。 |

- `stage: goal | plan_id: PLAN-1092 | plan_revision: r3 | round: 1 | from: work | to: work | status: executing | worktree: D:/autostack/.wt/lang-1092/auto-lang | commit: 013c5e62f0cca655fa0ca4afbaac1f61cc990a95 | next: work`。输入审计 T-01 完成，其余职责执行中。同步已合入的 1090，保留两批 canonical suite 引用。新增 Web 实际消费端，组内 `D:/autostack/.wt/lang-1092/auto-web` 分支 `plan-1092-dev` 基面 `26d085f7`，同组语言工作树作为路径依赖。用户授权见 §4。

### 2026-11-21 T-02 至 T-05 候选

- 语言 `7b836e7eabad88e1c968733dd0091761d6026a82`，Web `0cfc7c78446650631d244a9ea442d28635072958`，auto-down `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。同步 1090 落主基面后，普通 AV lowering 缓存身份从 8 进到 9。单例使用 1090 调度器的每次运行任务表和既有取消/关闭逻辑，首次静态访问按 task driver 懒建 64 槽邮箱；普通 spawn 的编码不变。async 标记沿旧约定用已有 `~T` 判定实际效果。
- CST 路由保 `#[with]` 普通名义约束、`#[test]` 标记、目标遮蔽和包 `pub`，以及导出的 ABI 名和 Rust 属性原始参数 span。960/963 等窄面保持其原入口。971 显式 ordinary AV 路径只执行选定声明一次。独立的 610 Rust 导出及 937 C 导出拥有产物合约，967 外库调用仍走原 ABI provider/授权；普通 AV 交付 `Export` 声明事实供其后端使用，当前阶段不派发 A2R/C emitter。
- `auto-web` 实际 `AvServiceEngine` 从共享已检 CST 读路由和 target 可见性，从受检 HIR 读借用 `str`，统一经 MIR/ABC 准入。依原服务协议分别发送文件和借用文本，连续文本请求各派发一次且与文件通道隔离。更新后的 7 条 AV HTTP 真实 TCP 检查串行通过（3 条非 AV 检查未选），构建 29.397s，运行 2.282s（子运行 2.26s）；含实际 Jade Garden 源、精确 JSON、关闭端口、隐藏非 AV 路由和 legacy=0。两次早期错误分别为文件路由 500（专用旧加载器约束）和 `mir.entry-sig` 拒借用文本，改为同一受检驻留入口后通过。串行组日志 SHA256 `b90d2011314a97ea23f443721dfd11ea91f88a715aeae2655ab522b99d877421`。
- 字段查询同一实例从实际记录布局取值。在同一模板的两个泛型实例中，第一次对照为 43（按相同源码 span 复用了另一实例的结果），改为函数实例身份与 span 联合定位后返回 42。受管存储重开依次观察 42、43、1、42；后续三次独立闭合标量调用均命中已有 CT 值，AV CT 执行为零。涉及声明、间接调用或全局的调用继续使用完整源闭包身份。CT 服务 38/38（debug 构建 22.15s，运行 3.19s）。
- 父侧定向语料 18 PASS、1 显式 AC SKIP，1.751s。18 项各 449–599ms。补充复用原 runner 的七项元控制：错 AC oracle 与缺 AC worker（exit 0，AC SKIP）、移除 AC ignore（exit 0，AC NOT_RUN）、错 AV oracle（exit 1）、忽略或删 AV（各 exit 2）、重复 Case（exit 2）、源等值编辑后 source hash 改变且 oracle hash 保持（exit 0）。均为同一 Auto 源，完全未执行所忽略或未选非 AV 检查。源变更有效性和报告分列由父端独立判断。最终语料报告 SHA256 `c2c1c137e454539706b7b62dcdf45bb78fe82e16cdbd63a2dfa24f01eeea1769`，控制清单 SHA256 `41c92dfaf4fd395e1139c43ef2732193f0263506066224ba6a469d334ceddde9`。早期输出根与输入根重叠被拒绝，已迁到 disjoint scope；新重复参数首个草案 code 多写绑定前缀，由父报告纠正真实 code 后通过。
- 本包主承接 0 份旧 VM 文档。唯一活跃公共作者为 `language/metadata/attributes.case.ad`，已被 `language.suite.ad` 引用。旧义务 TSV 新增 10 行，连到 963、971、555、936、967、Task 旧 shim 及四个保留原预期的 A2R/A2C 输入。两目标间原源或判决边界不同的输入各保原 owner。Web 消费源与 982/983 契约留在原仓。
- T-06 冻结公共 AV 语料已通过。debug 父工具构建 56.56s、AV worker 26.13s，正式准备外层 84.290s、exit 0。AV-only 派生配置的第一次执行因外层校验只接受仓库配置 SHA 停在用例前，保留本次 exit 1。重放分别验证原配置和派生选择，完整执行 77.209s、exit 0，其中 Case 父端 70.666s。667 检查中 505 PASS、162 SKIP、0 FAIL，本包 18 项耗时 1.076–1.586s。准备、门报告和逐例报告 SHA256 分别为 `e674f5922b9ae9453e6bac76671608e2e8612ec7e4978afbe649e8e1b4c8dd3a`、`aafc109d43197437e874406fb1e5454d8eebca63f6bdc43a79e0371423f7a988`、`5db7004167fd3d33d1662a11828f81c50e95555a0266c241c1ff8b4d29ba93f4`。正式准备只用 debug `case-parent` 与 `av`，运行未准备非 AV 工具。Case 进程启动前等待空闲 33.715s，随后其他在途会话启动编译。
- 以准确的 `AC_MN_CASE_WORKER` 指向不存在的文件重放隔离控制。七项保持原判决，报告汇总 SHA256 `8d8636e85fcdff4c2853a64ef5ab15b9ce50367637d9b331e6d5a9b0268f72e2`。固定组选集仍需完整运行，并分别核旧红和资源首红。
### T-06 冻结固定组与容量对账

本次使用 `av-phase.py` 调仓内 `scripts/fixed-tests.py`，分开 `--prepare-only --debug-tools` 与 `--run-prepared`。九个固定职责组按下表选择。必需 AV 和共享正确性进入门，原有性能、排除和平台边界保留。组内 ABI 只运行适用的公共约束校验，支持 AV 的既有 C/Rust 真 provider 另用同归档执行构建集成。AC/A2R/A2C/A2PY、VM 和 Linux/WSL 执行均未授权。

| 固定组 | 实际执行 | 首轮 PASS / FAIL | Nextest 排除 | 发现 s | 执行命令 s |
|---|---:|---:|---:|---:|---:|
| ace-source | 932 | 932 / 0 | 1 | 0.554 | 8.715 |
| ace-syntax | 159 | 159 / 0 | 3 | 1.016 | 3.292 |
| ace-semantics | 295 | 295 / 0 | 10 | 3.093 | 5.178 |
| ace-av | 1495 | 1492 / 3 | 51 | 1.559 | 62.809 |
| ace-abi | 78 | 78 / 0 | 258 | 1.333 | 4.149 |
| ace-comptime | 136 | 136 / 0 | 19 | 3.298 | 22.456 |
| ace-lowering | 9 | 9 / 0 | 0 | 1.029 | 1.291 |
| ace-artifacts | 43 | 42 / 1 | 7 | 1.537 | 9.261 |
| ace-facade | 85 | 84 / 1 | 另 5 非 AV 二进制未选 | 15.769 | 13.422 |

首轮原始报告计 3232 条执行，3227 PASS、5 FAIL，外层 162.227s / exit 1。组间主要清理和汇总计入总命令，未单独取样。AV 域具名过滤沿 1089 收据的 27 项非 AV、1 项 Linux，其余 23 条正式性能依原命名空间过滤。ABI 和 facade 根据实际消费者选执行，编译依赖可包含共同库。准备等待 116.104s，第一次准备 312.768s / exit 1，最后 root facade 归档卡于 `.config/nextest-full.toml` index 14/15 的 AAC package matcher 与 Cargo 图不相容。依最新批量回归收据的已知选择问题，只在本计划派生 facade 配置去掉两条外域 matcher。其余八份归档以同时满足相同源状态、命令、退出码和归档哈希为复用条件。重放准备等待 29.305s，构建和绑定 9.038s / exit 0，固定运行等待 0.140s。准备清单 SHA256 `9d29c71de7a514f16b7ec7fe38ce96b3dacebb6389e4dba62e40b1f04b7cb229`，首轮执行 SHA256 `2c815924c28c3d342046a1da5e62622782b6dbe49f4fe2bfd5e699309b67ee94`，派生的 facade 配置 SHA256 `529b0827b5f256916b3cd0c09ba1c6901e1fd7eee551e550ac1c6cea5dbd2055`。

首轮失败中，`ace-av-core layout::tests::layout_rejects_non_record_and_empty` 的断言源在基面与候选均为 blob `2c9adfefb425a7bf56f1fc6d3355d41b6962d991`（D-1088-LAYOUT-01）。`p969_t02_console_lane_grants_and_capture` 的旧入口检查实际收到 helper 值 1，D-1098-01 已记录同输入和终态。另两项 `kv_zip_1064` 的 `v03_kv_wire_oracle_and_reopen_av` 与 `v03_zip_negative_corrupt_bomb_and_preserve_av` 分别 14.360s、15.152s 达既有墙钟上限，另 `review_r3::r3_f01_concurrent_staging_budget_is_atomic` 在 1.138s 得现有并发症状。本轮保留各自失败计数，待同归档串行重放核终态。

新增永久检查均小于 10s。完整公共 AV 门的逐项墙钟如下，均保各自独立值、双流或诊断 oracle。

| 公共 AV 用例尾名 | ms | 公共 AV 用例尾名 | ms | 公共 AV 用例尾名 | ms |
|---|---:|---|---:|---|---:|
| with-constraint-spelling | 1173 | test-attribute-source | 1159 | unknown-attribute | 1245 |
| duplicate-test-attribute | 1103 | test-attribute-on-record | 1218 | compile-time-field-metadata | 1076 |
| imported-record-metadata | 1333 | metadata-without-capability | 1094 | target-selected-implementation | 1586 |
| target-excluded-public-function | 1547 | singleton-task-reuses-mailbox | 1505 | ordinary-task-rejects-static-access | 1485 |
| async-attribute-keeps-future-effect | 1556 | field-query-uses-generic-instance | 1450 | with-requires-a-nominal-witness | 1395 |
| target-selection-on-method | 1273 | with-unconstrained-type-parameter | 1427 | with-rejects-a-duplicate-parameter | 1499 |

固定组中新增普通文件模式准入、原 route/ABI/Rust 参数位置、跨模块失效与无关 CT 复用分别 145 / 214 / 368ms，971 的显式测试发现和执行 551ms。Web 新增多行路由测试带原真实 TCP、目标隐藏、严格派发计数与文件不存在断言，单项实际外层 735.613ms（子测试 0.71s）。已有 971 正反 oracle 和 Web 7 项组证据继续绑定各自冻结提交。

为留足验证磁盘空间，在无本计划 Web 构建时只清除 `D:/autostack/.wt/lang-1092/auto-web/target/debug/incremental`。核 Git 忽略、准确绝对路径、reparse guard clean 和冻结测试可执行件前后一致。删除 7764 个可重建缓存文件，4803450844 字节，2.032s，保留的 Web 测试执行件 SHA256 `b39b44559230007dbff037a8d2d01d0f414d2f1dc4e4f2142462dc46953fa47a`。其余相关路径合入后对账。
### T-06 后续基面同步

- 初轮同归档串行重放关闭三个资源相关首红。两条 ZIP/KV 均通过，暂存 race 单项通过；原空记录和旧控制台两条仍按原失败计数。已知项维护归 D-1088-LAYOUT-01、D-1098-01。本轮 ABI provider 独立集成复用固定 ABI 归档，发现 1.358s、实际执行命令 36.626s，26 PASS、310 排除，外层 38.815s / exit 0。执行使用真实 C/Rust provider 与 AV 消费端，构建和运行隔离于日常共同 ABI 组。结果 SHA256 `0dc692567f70ae94c93e6b11f5b00998a110e18c6ebaa984337fd4ddf42814e0`。
- 冻结检验期间主分支 advance 至 PLAN-105 的 `0b754af59`，包含共享 grammar、语义、MIR 和 AV 的实际改动。本计划先在专属树同步成 `5fec3fe70`。构建发现两项新的 AGE 合成函数构造需要显式 `test_attribute: None`。随后普通 AV 因已有语法条目回退，在 18 项定向测试中 17 项 `source.infra`。追溯到 `63e98fb58` 重放时混入旧普通 AV 语法，缺 `task-item`、`pub`、CT 和实例化等 profile 成员。依据 PLAN-105 实际原始提交 `354534418`、原分支后继 `f7d067252` 与已交 1090/1091 的原源码恢复共同语法。保留 AGE 扩展、独立作者面和 1092 属性规则。另移除两条 W35 的原始合并尾标。最终 `4b98dc1daf3a4d0e223e31eb4f64914714839c7c` 定向运行 18 PASS 和 1 非 AV SKIP，约 2.210s；合并基面后的实际 debug worker 构建 10.304s。全量固定组和 Web 依赖绑定至最终两仓候选。
- 为避免工件和临时解包耗尽磁盘，仅清理已停止编译的本计划语言根检出 `target/debug/incremental`。核绝对路径、Git 忽略、reparse guard clean、`auto-case.exe` 内容不变。19791 个生成文件、7271871883 字节，3.751s。此前的程序和原始验证收据保留至结束。
### 冻结 profile 复核及最终候选

- 收敛版本 `4b98dc1d` 的 marker、AV 语料和 Web 已通过。Web 最终依赖构建 53.287s。磁盘空间不足使首次 HTTP 启动未进入用例，随后用相同二进制重放 7 PASS，外层 2.229s、子运行 2.20s。新多行路由用例精确重放 0.098s，执行件 SHA256 `bafc9334ac65b1942752102b6f0a9941aefb9474dce36c7517b483fbe1e07b3a`。后续修正仅触及 AGE 规则的投影，Web 使用的普通 Auto 语法与受检实现相同，复用该同内容证据。
- 从 4b98 的首份归档隔离运行 981 四条词法/文法身份校验，2 PASS、2 FAIL、exit 100，1.533s。对照当前主分支和候选的可执行文法逐条比较，定位五个 AGE 规则与一个条件分支的差异。还原 Plan 105 的已冻规则序和诊断标签。原有两条 oracle 始终保留。通过 `ace_syntax_grammar::compile_profile` 重算，`age-component` 恢复 `ac48cc92afeec1bcf893ade984dd65ae8a69d58861bf6dd741bfa9b5dacc71f7`，`age-package` 恢复 `0d016cd528fc40adfb88969aa51be45e095abb6cdb730af3562c2859f5fd73c3`，均逐字等于 PLAN-105 的已冻身份。普通 `auto` 为 `45c136ce19ec3b1d7e9869687f578a650ae47bfb1d740884d2712fa62005c95e`，`auto-packages` 为 `b868219f22280df7ca2286de5995bede607b1a760fb200d2892cdf24399e0920`，并与 4b98 等同。冻结 `text-record`、`host-console`、`scheduler` 的身份亦等同。共享 AGE 路径的 AAC 衍生语法随冻结 AGE 形同步，仅检查静态源码语法身份，按 r3 未执行非 AV 目标。
- 为保证主分支线性，用具同一 Git tree `cd82a24dcc84822b22a8870cf95cee7d5d4f9859` 的单一内容提交固化原合解修复 `6158b5bc0b66149fc56cd7fc81f09175d7f9cd3b`。最终候选 `b02e596567c2060f6e7b2cc2b310c2ec2251a3f0` 的唯一父提交是 `0b754af59dd3da68e90d3739093bd66c9ca0b8a8`。工作树、索引与有序合并树内容逐字匹配。此前 4b98 的九份归档准备 294.525s，发现 AGE 身份差后仅停止本计划尚在等待的执行，未计为全量执行结果。
- 磁盘空间应对只处理本计划的生成目标。分别在 guard clean、绝对路径和测试件一致的条件下清 `experimental/ace/target/debug/incremental`（30399 文件、8836713391 字节，25.584s）和续编后的 Web incremental（2178 文件、1506941299 字节，0.256s）。Web 7 项通过后，`cargo clean --locked --offline --target-dir` 对专属 Web `target` 实际删除 4180 文件、4888257491 字节，0.703s，保留上述测试件指纹和日志。此前窄包清理移除 41 个生成文件、约 1.1 GiB；一次自动审批拒绝的批量缓存操作没有执行。无项目源或其它工作树清理。

### PLAN-1055 再基线与旧用例兼容性复验

- PLAN-1055 在主分支落地后，本计划同步 `58dc6d8d2d2ed74c0825e02cb0785cb4dad1ec04`，形成 `2c0467fd1b259aba0a5d97d63245248f2676d39d`。保留 1055 的 AAC 条件循环与显式所有权转移。普通 `auto`、`auto-packages`、两只 AGE profile 与三只原辅助 profile 的编译身份逐项不变。主分支随后至 `67a4862d9e73ed01cffbf537902dbefdaf7f922f` 只增加 1055 回归收据与归档更新。1092 的周期回归未到期，主检出 1055 独立的批量红单独跟踪。
- `2c0467fd1` 九组已准备，外层 189.807s、exit 0，尚未执行固定组。公共 AV 语料在该候选上 45.727s / exit 1，实际为 486 PASS、18 FAIL、1 ERROR、162 SKIP，新 18 项均通过。失败报告 SHA256 `9f01175dd1a3d0742175d44016d1cd4bd399a1f7f32ad786648d913fec8a0a82`。17 个动态对象用例命中 `LIST_FIELD_VIEW` 与 `OBJECT_SET` 同为 `0x37`；actor 的位置参数因合并丢失 mailbox 构造上下文；旧 2000 轮字符串池由普通 AV 误用 AGE 视图降级，原 990049 条指令变为 1086049，超过 1000000 的原燃料边界。
- 修复独立分配 `LIST_FIELD_VIEW=0x57`，只在独立 collections v3 保留旧字节读入，混合族按原 `OBJECT_SET=0x37` 解码。还原位置参数的消息臂见证并保留显式移动标记。整元素文本视图由 AGE 能力启用，所有列表访问只求值一次，普通 AV 继续使用已有 place 路径。原源码、oracle 和限额保持。针对对象、具名 actor、原字符串池、1092 单例的直接重放均 exit 0，命令外层分别 2.237s，8.335s，2.489s，2.430s。第一轮重放保留三项绿及字符串池 exit 2。普通 AV lowering 标识仍为 v9，本计划仅在落主时一次发布。
- 修复后源码提交 `eaab283d0742a6d5d5e3af7db1147d9f2825e43d` 的唯一父提交为 `67a4862d9`，两个专属分支干净。守护脚本核准两处缓存根 clean 后，根 `target/debug` 经 Cargo 删除 11298 个可重建文件、6.9 GiB、1.230s，ACE 仅四个受影响包删除 478 个可重建文件、3.4 GiB、0.323s，准备好的旧父工具 SHA256 `cce807436ed15a63196345791a99b21e4858a5824737e4d25b7f2af069022983` 不变。


### 冻结公开门与独立反例

- 最终源提交 `eaab283d0742a6d5d5e3af7db1147d9f2825e43d` 绑定九个固定职责组。首次准备已生成九份 exit 0 的测试归档，工具准备遇 2 GiB 磁盘保护，组合准备 exit 1。只清可重建 `experimental/ace/target/debug`（3867 文件、8.4 GiB）和 facade 包缓存（254 文件、2.3 GiB）。清理前后九归档指纹核对一致。工具重绑复验各归档命令、源码和配置身份，9 份 manifest exit 0，外层 107.968s、内部 107.354s，SHA256 `6826495517637d4545737a151d301f770f6724c8a981152d80a898c16f4e98ba`。Case 工具独立准备 1.534s、exit 0。
- 最终公共 `auto-case` 的 inventory 0.878s，子进程执行 42.603s、exit 0。667 项为 505 PASS、162 SKIP，0 FAIL、0 ERROR。新增 18 条 AV 检查分别 589–1371ms，另 1 条本包 AC 检查显式 SKIP。先前 19 项逐一恢复 PASS。AV worker SHA256 `95525c45c8c2fc147fdf339c9442099598c0b87fc4bf873bc80c0d02bb025313`，父报告 SHA256 `a6b95c61a817af1eaea3b3770e56135d86a29c1720a42be71345ce65d33adffc`。实际 `avworker/1.0`、`av.closures/1`、普通 lowering 9。
- 复审从最终 manifest 独立重放五个未入语料的近邻：双参中单独名义约束、多目标合并中的 VM 选择、f-string 改写之后的原 `test` byte span、同一运行内两种单例互不串邮箱和导入私有记录拒绝。结果分别为 42、42、绑定诊断 `source.attribute-duplicate`（55..59）、42、`source.module-private`（0..23），双流均为空，逐项 0.042–0.052s。复审结果 SHA256 `82dc45c4de19a8202c010f96bc794dea041620930bd94beb1e2a552aedef686e`。受检源、18 个作者 ID、10 行旧义务与 5 个 Spec 草案身份另逐项核对。


### 最终固定职责组与两个下游

串行执行固定入口的源、配置与工具身份已绑定至 `eaab283d0742a6d5d5e3af7db1147d9f2825e43d`。下表报告 first run；环境重放与已登记基面失败各自列出。门采用固定组原包和原选择，按 AV 阶段声明使用 1089 的具名 Windows/非 AV 排除，另外保留性能、平台与各组原条件。完整 Web 初始 3 条非 AV 检查未选。

| 组 | 执行 | 原 PASS / FAIL | 排除 | 发现 / 执行 s | 同归档重放 PASS | 已登记 FAIL |
|---|---:|---:|---:|---:|---:|---:|
| ace-source | 958 | 953 / 5 | 1 | 1.247 / 8.934 | 0 | 5 |
| ace-syntax | 159 | 155 / 4 | 3 | 0.945 / 2.939 | 4 | 0 |
| ace-semantics | 295 | 292 / 3 | 10 | 2.596 / 5.707 | 3 | 0 |
| ace-av | 1507 | 1476 / 31 | 51 | 1.752 / 41.689 | 17 | 14 |
| ace-abi | 78 | 78 / 0 | 258 | 1.276 / 3.405 | 0 | 0 |
| ace-comptime | 136 | 130 / 6 | 19 | 1.963 / 5.835 | 6 | 0 |
| ace-lowering | 9 | 9 / 0 | 0 | 0.353 / 0.483 | 0 | 0 |
| ace-artifacts | 43 | 43 / 0 | 7 | 0.519 / 4.667 | 0 | 0 |
| ace-facade | 85 | 84 / 1 | 0 | 8.886 / 11.639 | 0 | 1 |

实际固定组 3270 条，首轮 3220 PASS、50 FAIL，运行前等待 185.835s，外层 106.716s / exit 1。清理磁盘时删去了若干测试按原绝对路径启动的辅助程序，原始错误为 Win32 code 2。核对归档 SHA 后把 `ac787-query`、`ac787-gen`、`sema-worker`、两个 AV restore probe 与 `ace-comptime-worker` 按归档原字节恢复且现场复核 SHA。对 30 条实测失败从相同 nextest 归档、原规则配置与原判据加 `--test-threads 1` 单独重放。`ace-syntax` 4 PASS / 2.281s，`ace-semantics` 3 PASS / 5.870s，`ace-av` 17 PASS / 6.481s，`ace-comptime` 6 PASS / 17.182s。合计获得 3250 项有效 PASS 和 20 项既有 FAIL。重放绿单列，首红保持在原结果。

20 条继承红按 PLAN-1055 已逐 ID 比较的 D-1055-BASE-01、D-1088-LAYOUT-01 和 D-1098-01 继续归原 owner。首次表和有效结果都保留其失败计数。基面空记录 oracle 的代码 blob 仍是 `2c9adfefb425a7bf56f1fc6d3355d41b6962d991`，旧控制台失败保持 helper 的原值。以下是基面红的实际选中范围。

- `ace-core::conformance text_record_source::flagship_combination_compiles_and_verifies`
- `ace-core::conformance text_record_source::generated_atom_carries_the_combination_surface`
- `ace-core::conformance text_record_source::move_arguments_transfer_whole_roots_not_views`
- `ace-core::conformance text_record_source::named_ctor_source_order_is_the_written_order`
- `ace-core::conformance text_record_source::positional_ctor_and_owning_contexts_promote_literals`
- `ace-av::conformance av_data_semantics::av_data_instruction_contract_round_trip_all_opcodes`
- `ace-av::conformance av_method_modes::use_after_move_of_method_arg_rejected`
- `ace-av::conformance av_method_owned::v03_flagship_value_thirty`
- `ace-av::conformance av_method_owned::v03_token_microprogram_fifty_seven`
- `ace-av::conformance av_method_owned::v10_method_field_borrow_return_zero_alloc`
- `ace-av::conformance av_method_owned::v10_second_alloc_failure_rolls_back_partially_built_output`
- `ace-av::conformance av_owned_first_case::av_owned_first_case_whole_path_and_live_zero`
- `ace-av::conformance av_owned_recipe::av_owned_recipe_fixture_face_matches_hand_audit`
- `ace-av::conformance av_test_batch::c_data_profile_keeps_its_scalar_entry_boundary`
- `ace-av::conformance component_event_sessions_1106::v02b_text_leaves_are_pure_reads`
- `ace-av::conformance resource_915::resource_915_r02_l04_arena_sweep_is_complete`
- `ace-av::conformance resource_915::resource_915_r02_l05_v05_sweeps`
- `ace-av::conformance resource_915::resource_915_r02_l04_value_sweep_is_complete`
- `ace-av-core layout::tests::layout_rejects_non_record_and_empty`
- `auto-lang-ace-host::t02_facade p969_t02_console_lane_grants_and_capture`

26 条现有 C/Rust provider 和 AV 独立集成均 PASS，外层 31.907s。子命令 29.906s、exit 0，实际使用同份 ABI 归档和单进程选集。真实 Web 程序以 `0cfc7c78446650631d244a9ea442d28635072958` 和同组最终语言提交构建。构建 167.054s、exit 0。真实 TCP 7 PASS，外层 2.400s、子测试 2.26s。新增多行路由的精确复验外层 0.111s、子测试 0.08s。运行含原 Jade Garden 消费源码、两种实际路由、目标排除、文件/借用文本通道和 listener 释放。新用例实测低于 10s，执行件 SHA256 `8b5ca14cb041f91a4d9d6f383d432527af29b53460faf257c4964585cfd72494`。清 `auto-lang/target/debug` 4905 个生成文件、3.3 GiB、1.680s 且最终父工具 SHA 复验一致，为 Web 保留存储。各源收据 固定组原始汇总 `a65f5f765724f7e2f0141038acea0ff3bdfbf95f98f15194a10f1ba4be72a70c`，原归档串行重放 `78bdeb461539a85e13add42d64ea5fdfa01d333b6aab4be6054ec0b29526ddeb`，归档提取的同行程序 `20bd1ad31711fe919b511b7cd7209c1cfbea148dc0de1c43a9fbedf55eb58c46`，C/Rust provider 联验 `aed18dec3f411c72511f61c2e797058db11708aac3bbbe1c5bad4b674586e416`，HTTP 串行日志 `b90d2011314a97ea23f443721dfd11ea91f88a715aeae2655ab522b99d877421`。

- `stage: work | plan_id: PLAN-1092 | plan_revision: r3 | outcome: pass | commit: eaab283d0742a6d5d5e3af7db1147d9f2825e43d | web_commit: 0cfc7c78446650631d244a9ea442d28635072958 | next: review`。T-06 已核 AV 初始与重放证据，全部 505 条 AV 语料以及 T-01..06 本包要求通过。已登记的基面红及其它目标未选仍有效。


### T-07 r3 最终独立复审

从用户批准的 r3 源文档与实际入口反向核验。语言源 `eaab283d0742a6d5d5e3af7db1147d9f2825e43d`、Web `0cfc7c78446650631d244a9ea442d28635072958`、AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。

| 验收项 | 结果 | 核验 |
|---|---|---|
| AC-AV-ONLY | pass | 公共 suite 真 AV 505，显式 SKIP 162。移除非 AV ignore 后观察 NOT_RUN，缺非 AV worker 时原 AV 正例仍绿。错 AV oracle、零有效 AV 和缺 required AV 均红。Windows 不执行具名 Linux/WSL 腿。 |
| AC-FC | pass | 单一作者、18 个 AV 目标判决、原语料路径与重复 ID 负控对齐。改源改变源码身份，原预期摘要保持。四个任务外目标的留存输入、原预期和 owner 可对应。 |
| AC-01 | pass | 原 `#[with(T as S)]` 与 `#[test]` 源各返回 42。普通目标选择、文件模式优先级、单例与 async 走同一普通 AV 链。 |
| AC-02 | pass | 未知、错误位置、重复标注、未授权元查询、错误泛型见证与非单例静态访问都有原码诊断。追加复验 f-string 改写后原 `test` byte span 与私有导入拒绝。 |
| AC-03 | pass | 971 校验 helper 未被发现、一条独立标注仅执行一次、错误报告可见。真实双通道 HTTP 以同一源路由事实，legacy 派发计数 0。跨模块受管查询依次得到 42、43、1、42，无关纯标量 CT 继续命中。 |
| AC-04 | pass | 10 行原义务按原来源逐项核对，目标观察和研究留存各自归属。旧 VM 文档本包主迁移量为 0。 |
| AC-05 | pass | 声明元数据由共有 Schema/CST 产生。普通 AV 经 Checked HIR、MIR、ABC 与真实执行。971 使用原报告，HTTP 用共享 route 构造和受检签名，外部 host/provider 仍经原授权。 |
| AC-06 | pass | 对照九组实际计数、原 failure、同归档重放、资源配置、26 条 AV provider 与 7 条 Web。新永久项小于 10s。17 条 worker 告警逐一追到基面，`git diff --check` 通过。20 条已登记基面失败保原失败计数。 |
| AC-07 | pass | 重新核对原义务、重要近邻与反例、真实消费入口和五份规范增量，合入责任独立记录。 |

遗漏/延后/临时兼容复扫完成。字段遍历与自动 serde 归后续库，旧 A2R/Rust 发射与非 AV 运行测试仍归原目标，外库注册和授权保 967/1093 持有。普通显式 971 与共用属性语法分别关闭 D-971-B-01、D-963-C3 的本轮 AV 差额。基面的三条已登记债沿原 owner，无本包未清的实现项或需保留的原始材料。五文档基面和拟议内容 SHA 清单 `f8e9e01f0a548fb4d2e4612abfc849f4143c7cb7f281a2827fdc48eab37a73b2`，增加 SD-DISCOVERY 以覆盖真实工具消费者；GOAL-002 保持现行部分交付。

- `stage: review | review_scope: final | plan_id: PLAN-1092 | plan_revision: r3 | reviewed_commit: eaab283d0742a6d5d5e3af7db1147d9f2825e43d | outcome: pass | status: reviewed | next: merge`。合入阶段待验证 canonical 文档、两仓默认分支来源、归档与清理。


### 合入检查点

- `prepared` · 最终复审绑定 r3 语言源码 `eaab283d0742a6d5d5e3af7db1147d9f2825e43d`。规范增量及登记文档提交 `41adbdca21f80612794086a37d2116c5020ebdb5` 只改六份文档。Web `0cfc7c78446650631d244a9ea442d28635072958` 实测绑定该语言候选，两仓以各自 `master` 作线性合入。规范清单源 SHA 与复审记录一致。周期回归沿主分支收据 `67a4862d9`，本计划编号及时间间隔均未触发新批次。


- `landed` · 本地 `auto-lang/master` 快进至 `41adbdca21f80612794086a37d2116c5020ebdb5`，`auto-web/master` 快进至 `0cfc7c78446650631d244a9ea442d28635072958`。合入前后 12 份其它工作中已跟踪文件的内容身份已逐项核对。主检出 canonical suite 上的约束正例与字段元查询均 exit 0，仍用已冻结的 AV worker。从专属树执行验证，主检出只读用例输入。
- `canonical_verified` · `IF-AV-PARITY-METADATA` 从下列现行 Spec 与两仓代码提交派生。实际读到的 canonical Case SHA256 `234483bbc5fc4fa57ce40495be317852a38ad86851cc92ed3df8d4647d7467a7`、suite SHA256 `69fc7c808c58c8ac4104b4eff6d108fde15d035c819ccc4ae21f422fbe033c9a`，Web 服务源 `0c159b9e9fbbfdd84c42e8da8cdcae883c87c1a0a3334f216dad2abc2fee5d13`。两份落地主源抽查分别 `e9ae6d950496e77f7b33c7d16ca55820ac087102a931b508c97e274c72f9b03e` 和 `2f9bdaaed617efb8003fc5add3499efed16d73322bdb5aa0e58cf67e06f81a8b`。落地核查总清单 SHA256 `1f62dd5ec044fa7fdeaba261f4ff4638a4c5bab98fb1af46ab20270a90d5cdd5`。

- `docs/specs/auto-av/language-parity.md` · SHA256 `8280b5aac7492f524cd80764c7925f617f8402f4e6d5b2ead75a5aa11b08d2a7`
- `docs/specs/ace-migration/m2-generics.md` · SHA256 `de4bb23e087d6e85e1943820958f66eb23e47ee8cba1a4420b6bb3f9b6dbb4c8`
- `docs/specs/ace-migration/m2-metaprogramming.md` · SHA256 `5add4013b1c982baa3210fa3ed6bff8f545f50343b2a3ce44a8cdf2f6d03cbe2`
- `docs/specs/auto-test-case/feature-organized-corpus.md` · SHA256 `311816dc5a74b64e18855da4b9931d4d07a06c41807eae17054d431119f97a1a`
- `docs/specs/ace-migration/m2-tools.md` · SHA256 `ee3d2b965455d72edd9e79f3da13844dcd06896777d35777eaa14f37d5f8c290`

- `archived` · 完成交付记录移至 `docs/plans/archive/1092-av-parity-attributes-and-metadata.md`。终态 `archived`，交付类型 `delivered`。归档时实施/审查步骤 7/8。唯一余项是已交付三工作树、两分支和统一材料区的守护核对与清理。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
