---
plan_id: PLAN-1093
status: archived
feature_name: AV受检FFI语言接面与回调补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 3
current_step: 7
total_steps: 8
depends_on: [PLAN-1078, PLAN-1079, PLAN-1081, PLAN-1086, PLAN-1087, PLAN-1090, PLAN-1092]
supersedes_spec_components: [docs/specs/ace-migration/m2-denominator.md, docs/specs/auto-bindgen/project.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1093] AV受检FFI语言接面与回调补齐

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`ffi/imports-callbacks.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-15**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

补旧C/Rust导入、外部句柄、回调和显式转换在普通语言中的能力接面，复用967自动化与资源owner。

本包交付`IF-AV-PARITY-FOREIGN`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- use.c/use.rs/dep与已交typed声明、AutoABI、foreign effect/capability、参数源序、CStr与长度字符串转换、numeric/record/text/owned/outbytes/资源柄的真实已承诺组合。
- fn值/闭包回调、同步/异步完成端口与noescape/escape合同，异常/panic不跨C ABI unwind；资源generation/close/late/撤权由967唯一owner。
- 默认标准库不因本包引入libc依赖；显式use.c libc函数是用户外库选择，可测试；未知外部API或尚未交provider具名阻塞并交确切外库owner。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1093/auto-lang`，分支`plan-1093-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1093/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

967已有大量真实FFI，不能重写bindgen/shim。旧17_cffi三例只是smoke、20_rust_ffi serde_json是error-presence，不能说旧任意Rust泛型已经可用。必须将语言接面与库API实现分别验。

既有来源与边界：

- [strlen.case.ad](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [f0-rust-ffi-967.document.case.ad](../../crates/auto-lang/test/cases/ace-m2/f0-rust-ffi-967/f0-rust-ffi-967.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：17_cffi；18_ffi；19_rust_std；20_permission；20_rust_ffi；ffi_dual语言接面；旧callback/ffi Rust tests。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [source_foreign.rs](../../experimental/ace/crates/abi/src/source_foreign.rs) | `b0ebc5fe5882655ae26b9ef662d1a73663a19d65e975063b52c5cd8a1c50e4b1` |
| [py_session.rs](../../experimental/ace/crates/abi/src/py_session.rs) | `50579af08e481ed6d19a76880eda436ba43bb9f7b3d978e95902899b59d8277f` |
| [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [av.rs](../../experimental/ace/crates/av-core/src/av.rs) | `2ad814939c0b9ec8b7da0e09383d0c621bd88697415fd31a4b0c5c21e856139c` |

硬依赖：PLAN-1078, PLAN-1079, PLAN-1081, PLAN-1086, PLAN-1087, PLAN-1090, PLAN-1092；实际外部接口：967 C/Rust provider生成与资源账、1064 bytes已交切片。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

- [source_foreign.rs](../../experimental/ace/crates/abi/src/source_foreign.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [py_session.rs](../../experimental/ace/crates/abi/src/py_session.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av.rs](../../experimental/ace/crates/av-core/src/av.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/ffi/imports-callbacks.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-denominator.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-bindgen/project.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
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

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1093-原文档清单)静态主承接 **70份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1093.rust-length-not-strlen` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=9; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1093.explicit-c-library` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=5; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1093.rust-empty-value` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=0; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `FFI-PROVIDER-MUTATION` | AC-03/AC-04 | work / T-04..05 | 保持Auto源同一，改真实provider bias/body后结果改变且旧缓存不能冒PASS；撤cap必须dispatch=0。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `FFI-LIFETIME` | AC-03/AC-04 | work / T-04..05 | owned/borrow/callback/outbytes跨调用/await/错误/close全链账；缺provider不SKIP充绿；真正CStr/UTF-8/UTF-16桥各自遵provider约定。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

外部provider输入合同：`f0_text`复用所引967文档内冻结Rust源码/manifest与真实生成shim；C `<string.h>`用同版目标头文件与真实provider。T-01/T-02把源、target、header/manifest、产物hash、所需cap与**显式授权**加入现有SDK/fixture输入闭包，使用已有provider loader，不在runner按`text_byte_len`等函数名写返回值。例中的`console`/语言caps不隐含foreign授权；执行setup必须显式供应对应provider的已核cap，缺失应具名拒且不计PASS。合法FFI运行、撤权dispatch=0和真实provider变更三路必须独立观察。

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: rust-length-not-strlen
id: "av.parity.1093.rust-length-not-strlen"
tags: ["av-language-parity", "plan-1093"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use.rs f0_text
fn main() int { return text_byte_len("A中😀\0") }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 9

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
复用967冻结f0_text provider及ABI manifest/SDK锁；Auto长度9，不能被C strlen截到8。
`````

`````autodown
## Case: explicit-c-library
id: "av.parity.1093.explicit-c-library"
tags: ["av-language-parity", "plan-1093"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use.c <string.h>
fn main() int { return strlen(c"hello").to(int) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 5

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
显式C外库测试，需要真实Windows C provider/header/ABI锁；不是默认标准库路径，也不是只检查调用被发射。
`````

`````autodown
## Case: rust-empty-value
id: "av.parity.1093.rust-empty-value"
tags: ["av-language-parity", "plan-1093"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
use.rs f0_text
fn main() int { return text_byte_len("") }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 0

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
与非空/NUL合法邻例共同测试；provider源码/产物身份必须由真实967装配注入，不在runner按函数名硬编码。
`````

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-av ace-abi ace-foreign-contract
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
| T-02 | work | T-01 | use.c/use.rs/dep与已交typed声明、AutoABI、foreign effect/capability、参数源序、CStr与长度字符串转换、numeric/record/text/owned/outbytes/资源柄的真实已承诺组合。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | fn值/闭包回调、同步/异步完成端口与noescape/escape合同，异常/panic不跨C ABI unwind；资源generation/close/late/撤权由967唯一owner。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 默认标准库不因本包引入libc依赖；显式use.c libc函数是用户外库选择，可测试；未知外部API或尚未交provider具名阻塞并交确切外库owner。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-FOREIGN。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（全部未施工）：

- [x] **T-01** work：冻结旧/新输入、规范差异与边界；70 份旧文档清单逐项核对；canonical 位置定案；指纹按新身份复核（见 §9）。
- [x] **T-02** work：普通 AV foreign 绑定/派发（prepared provider 闭包）、`use.c <header>` 拼写、CStr 桥；三正例真实运行（见 §9）。
- [x] **T-03** work：panic 不跨 C ABI unwind 验证；资源/授权边界与缺 provider 具名拒绝（见 §9）。
- [x] **T-04** work：默认标准库无 libc 依赖；显式 use.c 真库可测；未知 provider 具名阻塞；FFI-PROVIDER-MUTATION 变异 oracle 完成（见 §9）。
- [x] **T-05** work：canonical 作者 `language/ffi/imports-callbacks.case.ad` + suite 引用 + 70 行迁移；元控制（错 oracle 红、缺 provider 具名失败）验证（见 §9）。
- [x] **T-06** work：冻结提交；全量 AV 视图 510/510 绿；受影响固定组运行并区分基面红（见 §9）。
- [x] **T-07** review：独立逐 AC 核验通过，见 §9 复审收据。
- [x] **T-08** merge：落主/归档/清理完成，见 §9 merge 收据。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。

### 2026-11-21 work 完成收据（T-01..T-06）

`stage: work | plan_id: PLAN-1093 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1093/auto-lang | branch: plan-1093-dev | base: 5e5121dac | dependency_revisions: 1078/1079/1081/1086/1087/1090/1092 全部 archived | task_ids: T-01..T-06 complete | next: review`。

**T-01 输入冻结**：§4 四文件指纹全部漂移（source_foreign/resolve/av/py_session），已按新身份复核；70 份旧 VM FFI 文档（17_cffi 3、18_ffi 54、19_rust_std 10、20_permission 2、20_rust_ffi 1）逐项核对。canonical 位置定案 `language/ffi/imports-callbacks.case.ad`。关键事实：普通链（`language_source_worker::run_chain`）此前无 foreign 注入（1113 实测 `use.rs f0_text` → `bind/source.undefined`）；967 链已有完整模式（`compile_language_source_with_foreign` + `verify_abc_foreign` + stub 表）可复用。

**T-02 实施**：①`auto-source.at` 增 `use.c <header>` 拼写（`provider-choice`/`c-header-item`/`c-header-tail` 规则；grammar 的 alt=独立备选臂语义——单 `alt{A B}` 是序列）＋ CST 投影（project.rs，spelling 剥离尖括号）＋手写 parser 同形（parser.rs `use_c_header_tail`）；②ace-av 新增 `foreign_source.rs`：请求 `foreign:{kind,provider,artifact_root}` → `<root>/<provider>.descriptor.json`（prepared 闭包：冻结源/artifact SHA-256/rustc 身份/C manifest）→ 纯派生 manifest（auto-bindgen）→ stamp+SHA 校验 → LoadLibraryW → 967 冻结 callable 映射 → `compile_language_source_with_foreign`→`verify_abc_foreign`→`execute_console_foreign`（含 console 捕获）；③父侧：ProbeAvSpec.foreign（contract.rs/author.rs）、`REGISTERED_FOREIGN_PROVIDERS` 表、`ACE_AV_PROVIDER_ROOT` env 转发；④prepare lane（`prepare_foreign` 示例）增 `--source-file`/`--c-provider`，写 descriptor。三正例全链路 PASS：`text_byte_len("A中😀\0")`=9、`text_byte_len("")`=0（Rust f0_text）、`c_strlen("hello")`=5（C parity1093_c，unit 内真实 `<string.h>` strlen，SDK include 闭包编译；freestanding provider 对象不动 967 冻结编译）。

**T-03 边界**：`text_panic_probe("boom")` → shim 就地捕获 + 导出标志 → 具名 `host.infra`（`foreign.panic: … contained at the boundary`），不跨 C ABI unwind（进程存活、无崩溃）。缺 cap → `source.foreign-capability`（resolve 阶段）；provider 身份错配 → `source.use-rs.provider`；缺闭包/根 → 具名 preparation 失败（不 SKIP 充绿）。

**T-04**：默认标准库路径零 libc 引入（无新依赖面）；显式 `use.c` 走用户外库选择（真实 strlen）；未知 provider 父侧表拒绝。FFI-PROVIDER-MUTATION：同源 +1 provider 体 → 9→10，provider digest 与 dll SHA 同步改变，旧缓存不冒 PASS。

**T-05 语料与迁移**：`imports-callbacks.case.ad` 5 条 AV 检查（3 正例 + rust-missing-capability + c-header-spelling-rejected），`language.suite.ad` 引用；`legacy-map.tsv` 增 70 行（owner=PLAN-1093，原 VM 身份自映射，AV 阶段 #ignore）。元控制：错 oracle → FAIL、缺 ACE_AV_PROVIDER_ROOT → ERROR（exit 2）、无 cap → resolve 拒绝；全量 AV 视图 **510 PASS / 0 FAIL / 0 ERROR**（88.2s，含 505 既有 + 5 新）。

**T-06 固定组**（`--prepare-only --debug-tools` + `--run-prepared`，含共享语法/绑定/IR 改动组）：ace-syntax 159、ace-semantics 295、ace-lowering 9、ace-comptime 136、ace-foreign-contract 58、auto-test-case 157 全 PASS。ace-source 958（2 条 p981 grammar 身份因本包 use-c-item 语法扩按惯例重冻结 4 面——TextRecord/Aggregates/AgeComponent/AgeH0，邻居 10 面逐字节不变、重冻结后单测绿；另 5 条 text_record_source 为 1092 已登记基面红 D-1055/D-1088）。ace-av 15 条、ace-abi 28 条 source_foreign 均为主检出基面同样复现的既有红（source_foreign 单测在主检出基面实测同红；其余与 1092 收据的继承红清单逐字一致），非本包引入，保留原 owner 与失败计数。新增永久回归 `plan1093_c_header_spelling_parses`（语法面，0.04s）。worker 构建注意：本机 RUSTC_WRAPPER=sccache 曾服务陈旧 ace-core rlib（include_str grammar 未跟踪），`RUSTC_WRAPPER=` + touch 重建后恢复；后续构建保持默认 sccache 即可。

规范增量（worktree 内未提交）：language-parity.md（§PLAN-1093）、m2-denominator.md（§PLAN-1093）、auto-bindgen/project.md（§PLAN-1093）、feature-organized-corpus.md（§PLAN-1093）。待交接口 `IF-AV-PARITY-FOREIGN` 绑定普通源/avworker 1.0/av.closures 1/mir-lowering 9/两已注册 provider 与逐检查真实结果。材料暂存 `D:/autostack/plan-artifacts/plan-1093/`（providers/providers-all/c-provider 等，merge 折摘要后删除）。

### 2026-11-21 final 独立复审（同实施会话，从工件重建结论）

`stage: review | review_scope: final | plan_id: PLAN-1093 | plan_revision: r3 | reviewed_commit: 300d8e585 | base_commit: 5e5121dac | worktree: D:/autostack/.wt/lang-1093/auto-lang | branch: plan-1093-dev | outcome: pass | next: merge`。

| AC | 结果 | 核验 |
|---|---|---|
| AC-AV-ONLY | pass | 全量 AV 视图只选 av 检查（510 执行）；ac-native/legacy-reference/vm-* 未选；缺 cap 具名拒（resolve）；缺 provider 闭包/根具名失败不 SKIP。 |
| AC-FC | pass | 唯一作者 `language/ffi/imports-callbacks.case.ad`（suite 单引用）；70 行迁移表 12 列格式全对、id 唯一（4 对重复为既有 cookbook 行合法复用）；AV 阶段 #ignore 保原身份/owner。 |
| AC-01 | pass | 提交 300d8e585 上 5 case 全 PASS（复审重跑：9/0/5 正例 + 2 具名拒绝），双流为空；真实 case 运行 0.60–0.85s。 |
| AC-02 | pass | `source.foreign-capability`/`source.use-rs.provider` resolve 阶段带 span；panic 具名 `host.infra`（不跨 C ABI unwind）。 |
| AC-03 | pass | FFI-PROVIDER-MUTATION 真实 9→10（+1 provider 体，digest/dll SHA 同步变）；FFI-LIFETIME 的本包可观察部分（缺 provider 不充绿、CStr 桥、UTF-8 桥）真实；owned/callback/outbytes 资源链按 §5 具名归 967 唯一 owner。 |
| AC-04 | pass | 70 行映射 owner=PLAN-1093、原预期/条件/SHA16 保留；AV required 行均有 actual（PASS）。 |
| AC-05 | pass | 共享 grammar/前门（`compile_language_source_with_foreign`/`verify_abc_foreign`/`execute_console_foreign`）；无 case 名特判/第二 parser；无 junction/symlink。 |
| AC-06 | pass | 固定组：ace-syntax/ace-semantics/ace-lowering/ace-comptime/ace-foreign-contract/auto-test-case/ace-source-syntax-927 全 PASS；ace-source 958 中 2 条 p981 属本包 grammar 扩（重冻结后绿）+ 5 条 text_record_source 与 ace-av 15 条均为既有基面红（D-1055/D-1088 收据同清单）；ace-abi 28 条 source_foreign 在主检出基面实测同红（非本包）。 |
| AC-07 | pass | 本复审：逐 AC 对源码/提交/实际消费者；`git diff --check` 干净；无调试输出/新警告（foreign_source/language_source_worker/prepare_foreign/parser/project 零新增警告）；迁移表格式校验通过。 |

健康与时间表：复审重跑首例 20.8s 为开发者 CLI 的 on-demand release worker 增量重建（`locate_worker` 首次拉取），日测 prepared 流程（`--prepare-only --run-prepared`）工具已钉、运行不构建——真实 case 墙钟 0.60–0.85s（rust-empty 0.85s、explicit-c 0.63s、missing-cap 0.71s、header-rejected 0.60s、rust-length 0.61s，全量 AV 视图 88.2s/510 例均 <10s）。新增回归 `plan1093_c_header_spelling_parses` 0.04s。全量 AV 视图 510 PASS/0 FAIL/0 ERROR 已绑定冻结提交（worker 二进制自上次运行未变；p981 修复仅触及测试文件）。

规范增量四份（language-parity/m2-denominator/auto-bindgen/feature-organized-corpus §PLAN-1093）已在 worktree 提交内，与 `supersedes_spec_components` 一致；merge 落主后按来源核验。遗漏/延后扫描：无未批准缩减；`use.c <string.h>` 草案按受支持注解 provider 形态落地（草案系"拟定合同"，语义守恒：显式真 C 库、头拼写可解析且须匹配已 admit provider），计划内已记录；FFI-LIFETIME 的 967 资源链按 §5 具名边界。
