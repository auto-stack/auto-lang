---
plan_id: PLAN-991
status: archived
feature_name: M1旧A2R 38格认证缺口分诊与关闭交接
plan_revision: 1
author: [Codex]
created_at: 2026-11-07
updated_at: 2026-11-07
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-test-case, legacy-a2r]
depends_on: []
current_step: 7
total_steps: 7
---

# [PLAN-991] M1旧A2R 38格认证缺口分诊与关闭交接

## 0. 变更摘要

认领38个required旧A2R cert-gap的逐格分诊与可执行关闭交接：现有本域证据漏接线、陈旧/缺证据、实际producer/认证通道缺能力分别处理；保留每格原source/oracle/producer/批准与当前零cert-gap守卫。本包交付可信的分诊和关闭材料，**不承诺交付即38格全部绿或M1可签**；缺实现/新认证平台由T-04形成具名待批准修复合同，不能伪造cert/PASS。

与[939](../939-foreign-resource-handle-and-callback-bridge.md)并行且无相互前置。正在执行的[941](../941-ace-m1-expanded-and-architecture-acceptance.md)持有最终证据接线/组合权威，本包只交接逐格材料，不修改其计划/代码/在途文件；938亦不接管。M2认证985消费M1结论，不能反过来让M1等985解决本组缺口。

## 1. 目标

1. 每个原格位有稳定ID、原token/source/oracle/义务与真实本域证据查找结果，不按新HIR证据猜旧支持。
2. 将登记状态与根因分开。逐格得到已定位的关闭动作或具名unknown及有界下一步，不能以“后续处理”完成分诊。
3. 在已有入口/能力范围内验证可复用证据或显式重验的适用性，交给941 owner按原规则接线；缺功能/通道时先形成最小修复合同。
4. 分诊交付与实际认证关闭分列，原313/38守卫与source/expected/NA/accepted/future不改。

非目标：修改legacy emitter/语义/协议/认证schema、修正金样/expected、换新HIR producer抵旧域、批量认证全部ACE、扩大M1/M2、给执行中计划加phase或自动创建会话。若需上述生产修复或认证平台变更，T-04只起草范围/路径/反例/完整职责/验收/关闭接线提案，依后续授权另走new/work；不能在本包顺手实施。

## 2. 架构方案

原binding/applicability/source-manifest + Specs → 逐格查找本域入口/证书/原执行终态 → 当前来源/producer/工具/完整actual/reference适用性核验 → 分诊分类与关闭动作 → 独立复审 → 只读交接941 owner。沿现有strict reader/受控工具执行器，不新建证书数据库、全局账本、runner、meter或第二套M1判定。

本包无生产/规范行为变更，规范增量为空有明确理由（§5）；如果T-02发现现有入口无法表达某格原义务，记录事实并交T-04合同，不假设`ace.m1.cert/1`可覆盖复合/trap/场景。

## 3. 技术栈

- 原登记：`crates/auto-lang/test/cases/ace-m1/{binding.tsv,applicability.json,source-manifest.json}`；原src/oracle/reference/suite及scenario登记。
- 原脚本：`scripts/verify-ace-m1-852.ps1`（Load-Binding/Load-Applicability/Get-CellState与acceptance，纯核算exit0不等M1通过）。
- 原认证：`crates/auto-lang/src/bin/ace-certify.rs`→`crates/auto-lang-core/src/test_cases/ace_certify.rs`（certify/validate_receipt，当前只选corpus value+ok-emit标量集）；独立917域`a2r_emit_917_cert.rs`不得跨域借证。
- 现规范：`docs/specs/auto-test-case/a2r-golden-certification.md`、`ace-m1-applicability.md`、`ace-m1-gap-audit.md`、`ace-m1-scenario-oracles.md`。

## 4. 需求分析与背景调查

### 授权与边界

用户2026-11-07明确授权正式修订939并把38格分诊/关闭路径落实到计划；**本轮只改计划，不施工、不修改938/941、不启动其他会话**。因此本包drafting r1/0-of-7，未创建实施树/材料scope，未执行编译/认证/产品测试；未来实施须后续指令。939与本包文档可在主检出；不为本次规划建worktree/agent/goal。

读时主线`7565e98c4`，38格为静态登记事实，非38个新实测FAIL：18个emit-ok-run-unverified、16个snapshot-only-cert-gap、4个cert-diff；未被applicability的NA/accepted/future规则排除。原131行、313 required不变。原完整名单来自当前文件而非复制历史计数。

现Spec指出原certifier是10例标量值域；复合伪Rust与trap/场景能力不能直接套标量cert schema。Spec原G-DEC双轴口径与924脚本更严守卫有历史差异；本包服从当前`certGapCells.Count -eq 0`，不请求/假设已获撤守卫授权。941当前修复与身份变化按其最新收据，冻结本包时重核真实版本；历史PASS不迁用。

### 输入内容身份（规划时只读）

| 路径 | SHA256 |
|---|---|
| `crates/auto-lang/test/cases/ace-m1/binding.tsv` | `303cdde9e8b5d56a8debeb5ff261519d2ee5c263684a4eefc84fafbe3917a259` |
| `crates/auto-lang/test/cases/ace-m1/applicability.json` | `74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0` |
| `crates/auto-lang/test/cases/ace-m1/source-manifest.json` | `5209f30010be94da007343b502e85f9ecdb85a57940315dfbad8314e4589db5a` |
| `scripts/verify-ace-m1-852.ps1` | `1a9c3bb809609a1b1e310b1723c205cfe17ff802b94f165259dc31dcea86f829` |
| `docs/specs/auto-test-case/a2r-golden-certification.md` | `32c2c4dc2dd9f7eee13745f63a217b01454e5162d73ce235774da766dfc46e72` |

## 5. 详细设计

### 5.1 固定逐格清单与下一步owner

以下38行均为**待分诊**，当前token不是根因判定。G01..G38仅本Plan行ID，外部身份恒为`ns:id:a2r`。corpus输入以原`src/<id>.at`及source-manifest定位；scenario以原binding/scenario-oracles与对应suite/fixture定位，禁止假设`src/R01.at`可替`scenario:R01`。T-01须冻结实际source/oracle路径与SHA和每格适用状态；当前表不伪造不存在的场景源路径。

| 行ID | 原格位 | 原a2r token | 原outcome / expected | 原输入定位 | 初始关闭候选／责任task | 当前状态 |
|---|---|---|---|---|---|---|
| G01 | `corpus:B08:a2r` | `emit-ok-run-unverified` | `value / 7` | `src/B08.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING（无冻结金样/无日常a2r检查）／C-CHANNEL（certifier仅选`ok-emit`）→**H-02**；P2 emit→compile 0→run `i32:7`==oracle，reference同；**已验证关闭候选（未认证）** |
| G02 | `corpus:B11:a2r` | `emit-ok-run-unverified` | `value / 7` | `src/B11.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-CHANNEL→**H-02**；P2（914闭包形）compile 0→`i32:7`==oracle，reference同；**已验证关闭候选（未认证）** |
| G03 | `corpus:B13:a2r` | `cert-diff` | `trap / overflow` | `src/B13.at` | T-02对账差异与原oracle；真实缺陷/规则问题→T-04 | 已分诊：E-MISSING（无金样/reference/cert）／D-CONTRACT（a2r `int`=i64 vs 冻结i32 trap profile，owner=语言语义裁定）+C-CHANNEL（`ace.m1.cert/1`无trap oracle）→**H-06**；P2 `2147483647 + 1`(i64) compile 0→wrapper `as i32`观察`i32:-2147483648`无trap；未关闭 |
| G04 | `corpus:B14:a2r` | `cert-diff` | `trap / div-by-zero` | `src/B14.at` | T-02对账差异与原oracle；真实缺陷/规则问题→T-04 | 已分诊：E-MISSING／C-PRODUCER（字面`10 / 0`，rustc deny-lint `unconditional_panic` 编译失败）+C-CHANNEL（无trap oracle；Rust panic≠Auto trap）→**H-06**；未关闭 |
| G05 | `corpus:B15:a2r` | `cert-diff` | `trap / rem-overflow` | `src/B15.at` | T-02对账差异与原oracle；真实缺陷/规则问题→T-04 | 已分诊：E-MISSING／D-CONTRACT（i64宽度）+C-CHANNEL→**H-06**；P2 `-2147483648 % -1`(i64) run `i32:0`无trap；未关闭 |
| G06 | `corpus:B20:a2r` | `cert-diff` | `trap / div-by-zero` | `src/B20.at` | T-02对账差异与原oracle；真实缺陷/规则问题→T-04 | 已分诊：E-MISSING／C-PRODUCER（`true && 10 / 0 > 0`，`unconditional_panic`编译失败）+C-CHANNEL→**H-06**；未关闭 |
| G07 | `corpus:T01:a2r` | `emit-ok-run-unverified` | `value / 38` | `src/T01.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（文本：发射`String::len_bytes`不存在，E0599）→**H-03**；reference `i32:38` ok；未关闭 |
| G08 | `corpus:T02:a2r` | `emit-ok-run-unverified` | `value / 27` | `src/T02.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（`use a2r_std`未解析E0432 + `len_bytes`）+C-CHANNEL（证书deps轴冻结`std-only`）→**H-03**；未关闭 |
| G09 | `corpus:T03:a2r` | `emit-ok-run-unverified` | `value / 23` | `src/T03.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（语义缺陷：Auto `len`=字符数，发射字节`len()`，run `i32:25`≠23；reference 23 ok）→**H-03**；未关闭 |
| G10 | `corpus:T04:a2r` | `emit-ok-run-unverified` | `value / 128512` | `src/T04.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（`char::to_codepoint`不存在，E0599）→**H-03**；未关闭 |
| G11 | `corpus:T05:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/T05.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE（binding注“伪Rust不可编译”对本金样不成立）／C-CHANNEL（仅选择）→**H-01**；P1冻结金样 compile 0→`i32:42`==oracle，reference同，日常a2r PASS；**已验证关闭候选（未认证）** |
| G12 | `corpus:R01:a2r` | `emit-ok-run-unverified` | `value / 19` | `src/R01.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-CHANNEL→**H-02**；P2 compile 0→`i32:19`==oracle，reference同；**已验证关闭候选（未认证）** |
| G13 | `corpus:R02:a2r` | `snapshot-only-cert-gap` | `value / 9` | `src/R02.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；P1 `i32:9`==oracle，reference同，日常PASS；`ace-certify --only r02`→“matched no certifiable case” exit 2；**已验证关闭候选（未认证）** |
| G14 | `corpus:R03:a2r` | `emit-ok-run-unverified` | `value / 23` | `src/R03.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-CHANNEL→**H-02**；P2 `i32:23`==oracle，reference同；**已验证关闭候选（未认证）** |
| G15 | `corpus:R04:a2r` | `emit-ok-run-unverified` | `value / 51` | `src/R04.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-CHANNEL→**H-02**；P2 `i32:51`==oracle，reference同；**已验证关闭候选（未认证）** |
| G16 | `corpus:R05:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/R05.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；P1 `i32:42`==oracle，reference同，日常PASS；**已验证关闭候选（未认证）** |
| G17 | `corpus:R06:a2r` | `snapshot-only-cert-gap` | `value / 21` | `src/R06.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；P1 `i32:21`==oracle，reference同，日常PASS；**已验证关闭候选（未认证）** |
| G18 | `corpus:L01:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L01.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（旧a2r无集合lowering：`List<int>.new()`伪Rust，rustc“comparison operators cannot be chained”）→**H-04**；reference ok；未关闭 |
| G19 | `corpus:L02:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L02.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（同L01伪Rust）→**H-04**；未关闭 |
| G20 | `corpus:L03:a2r` | `emit-ok-run-unverified` | `value / 0` | `src/L03.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（同L01伪Rust）→**H-04**；未关闭 |
| G21 | `corpus:L04:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L04.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（同L01伪Rust）→**H-04**；未关闭 |
| G22 | `corpus:L05:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L05.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（同L01伪Rust）→**H-04**；未关闭 |
| G23 | `corpus:L07:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L07.at` | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（同L01伪Rust）→**H-04**；未关闭 |
| G24 | `corpus:V01:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V01.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；P1 `i32:42`==oracle，reference同，日常PASS；**已验证关闭候选（未认证）** |
| G25 | `corpus:V02:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V02.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同V01；**已验证关闭候选（未认证）** |
| G26 | `corpus:V04:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V04.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同V01；**已验证关闭候选（未认证）** |
| G27 | `corpus:V05:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V05.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同V01；**已验证关闭候选（未认证）** |
| G28 | `corpus:V06:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V06.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同V01；**已验证关闭候选（未认证）** |
| G29 | `corpus:V07:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/V07.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同V01；**已验证关闭候选（未认证）** |
| G30 | `corpus:K01:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/K01.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；P1 move闭包 `i32:42`==oracle，reference同，日常PASS；**已验证关闭候选（未认证）** |
| G31 | `corpus:K03:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/K03.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE／C-CHANNEL→**H-01**；同K01；**已验证关闭候选（未认证）** |
| G32 | `corpus:K06:a2r` | `snapshot-only-cert-gap` | `value / 42` | `src/K06.at` | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：注准确（金样确不可编译）／C-PRODUCER（闭包发射器Rc/RefCell可写env路径发出重复关键字`move move |n: i64|`）→**H-05**；日常bytes PASS，reference ok；未关闭 |
| G33 | `scenario:R01:a2r` | `emit-ok-run-unverified` | `events / lifo` | `ace-m1-scenario-oracles.md`的R01原suite/fixture | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING（无场景源；a2r面=底层T03/R04/R05的emit）／C-CHANNEL（cert/1仅i32值，a2r无生命周期事件观测）→**H-07**；底层T03属H-03；未关闭 |
| G34 | `scenario:R02:a2r` | `snapshot-only-cert-gap` | `events / -` | `ace-m1-scenario-oracles.md`的R02原suite/fixture | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE（token“snapshot green”仅V05金样支撑；L04/L05伪Rust，K02 a2r=`diff-legacy-grammar(parse)`无金样）／C-CHANNEL（无故障注入通道）+C-PRODUCER（L04/L05）→**H-07**；未关闭 |
| G35 | `scenario:R03:a2r` | `emit-ok-run-unverified` | `events / -` | `ace-m1-scenario-oracles.md`的R03原suite/fixture | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING／C-PRODUCER（集合，L族）+C-CHANNEL（无stale-place trap通道）→**H-07**（依赖H-04）；未关闭 |
| G36 | `scenario:R04:a2r` | `emit-ok-run-unverified` | `events / once` | `ace-m1-scenario-oracles.md`的R04原suite/fixture | T-02查本域证据；T-03仅在已有支持入口显式重验，否则T-04 | 已分诊：E-MISSING（事件面）／C-CHANNEL（release-once/inactive payload不可在a2r观测）→**H-07**；底层V06值面属H-01；未关闭 |
| G37 | `scenario:R05:a2r` | `snapshot-only-cert-gap` | `events / 42` | `ace-m1-scenario-oracles.md`的R05原suite/fixture | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：E-STALE/不一致（token声称冻结快照，但`oracle/m1_k02.rs`不存在且corpus K02 a2r=`diff-legacy-grammar(parse)`）／C-CHANNEL+登记不一致须裁定→**H-07**；未关闭 |
| G38 | `scenario:R06:a2r` | `snapshot-only-cert-gap` | `events / -` | `ace-m1-scenario-oracles.md`的R06原suite/fixture | T-02核实际lowering/认证通道；能力缺口→T-04 | 已分诊：U-UNKNOWN（token所依赖金样在binding/SCENARIOS均未指明）／C-CHANNEL（无budget/cancel/trap通道）→**H-07**；未关闭 |

### 5.2 根因分类与实际关闭谓词

T-02给每格**证据状态**和**根因状态**分别记账；一个格可有多个问题，不强行只贴一个标签。每个问题唯一owner/task，实际全部关闭才可交“本格关闭候选”。

| 分诊类 | 充分调查与下一动作 | 关闭条件 / owner |
|---|---|---|
| E-WIRE已有本域证据漏接线 | 找到真实case/check/raw/完整证书，当前source/producer/target/工具/配置/oracle全匹配；无伪摘要/缺执行 | T-03以原strict reader核验证据，形成逐格映射；941 owner后续实际接纳才称接线关闭，未接纳保pending |
| E-STALE身份陈旧 | 按既有schema具体列source/tool/config等失配轴；坏结构REJECTED另记，不能降STALE绕过 | T-03在原可用入口显式重验actual+独立reference，再daily零spawn核查；缺可用入口则转C-CHANNEL，不能强认证 |
| E-MISSING无适用证据 | 完成原src/suite/证书/历史交付的有界查找，区分未跑与根本无支持 | 既有入口且能力可用走T-03；否则列能力/通道复现与T-04最小修复合同 |
| C-PRODUCER实现/语义缺陷 | 原source在原producer失败或与独立oracle冲突，列expected/observed、实际源码位置和近邻 | T-04冻结具名修复提案；未来正式修复须永久反例/完整职责/实际本域认证及独立review，交付后本格重验才可关闭 |
| C-CHANNEL认证通道不足 | 原certifier不选此格/不能表达其值/trap/资源/scenario，目录里有文件不算可用入口 | T-04冻结最小入口/schema/readers/预算与反控提案；不得在本Plan实现新平台或复用标量wrapper剪裁原程序 |
| D-CONTRACT规则/语义争议 | cert-diff有可复现的差异但规范互相矛盾或批准不明 | T-04给出冲突来源、保持原规则与可选裁定；未裁定前阻塞，不改expected/NA/守卫 |
| U-UNKNOWN调查不足/不可读 | 明列缺哪一份来源、owner及一次有界核查输入/入口；不是“无问题” | 留未关闭、具名关闭任务；T-04仅当调查边界及缺资料的取得条件具体可审核时可交分诊结果，不授认证绿 |

逐格关闭单字段：格位/source+oracle/producer域/配置工具身份、原token、case/check、raw命令与终态、完整actual与独立reference及适用schema、各strict结果、分类/复现/accepted邻例、关闭任务、负责模块owner、下一接收者941、是否已接纳、剩余项。缺字段记具名缺证据，不填0/PASS。

### 5.3 现有执行入口与受限重验

T-01先从真实固定组/CLI帮助和源码冻结**每格可选入口**。现有`ace-certify`仅选10个ok-emit标量值例，不能用改token、忽略选择条件或将整38传给它冒重认证；917证书也不能跨域。可只读检查现有strict receipt，不等全部941/D938/D939交付。

T-03未来仅在已有入口确能消费原source与oracle时显式认证；actual现场由旧A2R完整生成，独立reference是预先语义审核的真实参考，不由actual复制。值例两侧各对固定值；trap/资源/diagnostic/scenario按原各自合同验证，rustc报错/任意crash非语言trap。现有golden/expected身份不变；实际漂移记录后转T-04，不能直接更新金样。若无对应手写reference，先给出独立推导/审核任务与关闭条件，不能把未审参考当已认证。

继承已有120s compile/30s run/每流1MiB及原更严域门、原Job包含树终止/流回收，显式scratch位于唯一scope，不自动下载安装工具。daily emit+bytes+strict身份核查rustc/cargo/guest spawn0；显式认证与工具准备单列。坏证书不得--force覆盖，旧首次红及失效材料保留身份；采样/认证错峰，不为分诊跑全experimental/all。

### 5.4 缺陷修复交接与其他在途问题

T-04按**共同根因**合并最小修复提案，而非38个Plan或一个无边界全库包；每提案在本计划内有稳定H-<序号>、涉及G行、实际源/入口/expected-vs-observed、确定根因或假说、允许代码/所有reader、反例/合法近邻、保原预算、完整职责组/认证、Spec delta及实际关闭谓词。需后继正式Plan时仅在后续授权/中央取号后填真实编号；未分配前明确“待正式立项”，不预占号/假称有人实施。本包可交付分诊与有界关闭合同，不能把H提案完成冒G认证已关。

已发现F-941-05/06/08工作阶段已闭（最终review归941），F09已授权并在执行，F07 test-only提案仍待具名授权，原8scalar红须由原owner逐义务归因；本Plan只登记这些关联，**不增加/修订941任务、不接手、不套用其他phase授权**。38格C-PRODUCER若与其同形，也须证明同producer身份与同根因，不用新HIR修复代关旧域。

接收941的条件：只交冻结逐格映射与真实证据/关闭提案；941仍独立读原required与守卫。若本Plan发现941需适配，写具体字段/入口/理由给原owner后续安排；本轮不发消息、不改941。939按其D938→资源桥继续，本包不成为其前置。985只消费已形成M1结论，不作为本38格的关闭任务。

#### 5.4.1 H 关闭合同（T-04 冻结，2026-11-07，按共同根因分组；全部**待正式立项／裁定**，本包不实施）

共同事实（T-02/T-03）：`ace-certify` 的选择谓词 `ns==corpus ∧ outcome==value ∧ a2r=="ok-emit"`（`ace_certify.rs:111`）在构造上排除全部 38 格；`--only` 任何 38 格 id → `--only matched no certifiable case` exit 2。认证 schema `ace.m1.cert/1` 仅表达 `i32:<n>\n` 值 oracle、`deps=std-only`。本包未改 token/金样/cert/binding/applicability/NA/守卫。

| H | 涉及 G | 根因组 | 实际 expected-vs-observed（冻结树 b4b16dd9a，rustc 1.98.0） | 允许改动面 / 所有 reader | 反例与合法近邻 | 完整职责组 / 认证 | Spec delta | 实际关闭谓词 | 状态 |
|---|---|---|---|---|---|---|---|---|---|
| **H-01** 复合金样重绑定与显式认证 | G11 T05、G13 R02、G16 R05、G17 R06、G24 V01、G25 V02、G26 V04、G27 V05、G28 V06、G29 V07、G30 K01、G31 K03（12） | E-STALE + C-CHANNEL：binding 注“伪 Rust 不可编译”对这 12 个冻结金样不成立；唯一阻塞是 certifier 选择谓词 | 12 个冻结金样在冻结 wrapper 下 rustc 编译 0 并运行 == `i32:<expected>`；手写 reference 同；日常 a2r emit 26/26 PASS（bytes==golden）；`--only r02` 被拒 exit 2 | 仅 `binding.tsv` 12 行 a2r token `snapshot-only-cert-gap→ok-emit`（+注）、`source-manifest.json` binding hash、新增 `oracle/certs/m1_*.cert.json`×12；reader：`ace_certify.rs` certifiable_rows / `verify-ace-m1-852.ps1` Invoke-ReceiptValidation（certRows 10→22）/ acceptance（cert-gap 38→26）；**不改金样字节、不改 wrapper/schema** | 反例：篡改任一金样 → certify FAIL 且 cert 不动（G04）；K06 保持排除（H-05）；合法近邻：B01..B12 现有 10 证书 reused | `infra-ace-m1`（脚本守卫 26/97/313 常量需随改）、`case-ace-m1-base`（snapshot）、显式 `ace-certify --only <12>` 一次认证 | `a2r-golden-certification.md` §范围边界“当前认证集=标量 10 例”→22 例并记 E-STALE 勘误；`ace-m1-gap-audit.md` F-A2R-3 账 38→26 | 12 份新 cert 经 strict 验证 PASS；snapshot/receipts/acceptance 三 scope 绿且 cert-gap=26；941 owner 按真实身份接纳 | 待正式立项（需 G04 复审：金样 Rust 语义忠实于 Auto 源，852 审阅时按“伪 Rust”理解，须重审） |
| **H-02** 可编译值例金样冻结 | G01 B08、G02 B11、G12 R01、G14 R03、G15 R04（5） | E-MISSING + C-CHANNEL：从未冻结金样/无日常 a2r 检查 | P2 生产 emit → 编译 0 → 运行 == oracle（7/7/19/23/51），reference 同；B08/B11 因 avm 差异不在 `b-scalar.case.ad`，其 a2r 面与 avm 无关 | 新增 `oracle/m1_{b08,b11,r01,r03,r04}.rs`（经 G04 显式审阅后冻结 P2 实际字节，SHA 见 §7.1）、a2r-snapshot 检查入 case 文件（b-scalar 头注需改或另建 case）、binding token→`ok-emit`、manifest hash、5 份 cert | 反例：金样漂移红；B11 的 914 闭包形 `(\|__p914_a0,__p914_a1\| sub(__p914_a1,__p914_a0))(3,10)` 为生产命名实参 lowering，审阅须确认其为预期形态；近邻：E01/E02 914 域同形已认证 | 同 H-01（snapshot 检查数 26→31 守卫常量） | 同 H-01（认证集 22→27） | 5 cert strict PASS；cert-gap 26→21 | 待正式立项 |
| **H-03** 旧 a2r 文本 lowering 缺陷 | G07 T01、G08 T02、G09 T03、G10 T04（4） | C-PRODUCER：`crates/auto-lang-core/src/trans/rust.rs` 文本方法映射 | T01：E0599 `String::len_bytes` 不存在；T02：E0432 `use a2r_std` 未解析（且 cert deps 轴冻结 `std-only`，即便可解析亦不进现 schema）+`len_bytes`；T03：编译 0 但 `i32:25`≠23（Auto `len`=字符数，发射字节 `len()`）；T04：E0599 `char::to_codepoint` | `trans/rust.rs` 文本 `len/len_bytes/slice/codepoint` 发射；T02 须自包含（不依赖 `a2r_std`）或另议 deps 轴；reader：a2r adapter（bytes）、certifier | 永久反例 T01..T04 四源（现 P2 实际为首次红身份，SHA §7.1）；合法近邻 T05（文本 NUL 比较，编译运行 42 绿）；894 已在**共享** lowering 修 T05 eq/ne，旧 a2r 独立 | 修复后走 H-02 路径冻结金样+认证；职责组 `case-ace-m1-base` + 旧 a2r 单元（`a2r_tests.rs` 文本面） | gap-audit F-A2R-3 细分“文本 4 格” | 四格 emit 可编译、运行 == oracle、reference 同、cert PASS | 待正式立项（242 旧 a2r 线） |
| **H-04** 旧 a2r 集合 lowering 缺失 | G18 L01、G19 L02、G20 L03、G21 L04、G22 L05、G23 L07（6）＋G35 scn:R03 依赖 | C-PRODUCER：旧 a2r 对 `List<T>` 发射 `List<int>.new()` 伪 Rust（rustc “comparison operators cannot be chained”）——这是 852“伪 Rust”注的**真实所指** | 六格 emit 成功但全部编译失败（同一错误）；reference 全部 == oracle | `trans/rust.rs` 集合类型/方法（`Vec<i64>`、push/get/len/clear…）；或**裁定**：旧 a2r 不补集合、M1 a2r 消费者迁新 HIR（904 线）——该裁定改变旧域身份，须用户决定 | 永久反例 L01..L07 六源；合法近邻 R02/R05（record 发射绿）；新 HIR 904 证据**不得**代关旧格 | 修复后 H-02 路径；`case-ace-m1-base` + 旧 a2r 单元集合面 | gap-audit F-A2R-3 细分“集合 6 格” | 六格可编译运行 == oracle + cert | 待裁定/立项（最大合同） |
| **H-05** 闭包发射器重复 `move` | G32 K06（1） | C-PRODUCER：Rc/RefCell 可写 env 闭包路径发出 `move move \|n: i64\|` | 冻结金样 rustc 语法错误 “expected one of `async`, `\|`, or `\|\|`, found keyword `move`”；reference 42 ok | `trans/rust.rs` 闭包发射（一处关键字重复）；金样需 G04 重冻结（bytes 变） | 反例 K06；合法近邻 K01/K03（move 闭包编译运行 42 绿） | `case-ace-m1-base`（金样重冻结后 snapshot）；随后 H-01 路径认证 | 同 H-01 | K06 金样可编译运行 42 + cert | 待正式立项（小） |
| **H-06** trap 认证通道 + int 宽度裁定 | G03 B13、G04 B14、G05 B15、G06 B20（4） | C-CHANNEL（`ace.m1.cert/1` 无 trap oracle；917 对 T08 的“Rust panic 观察≠Auto trap 认证”仍适用）+ D-CONTRACT（B13/B15：a2r `int`=i64，`2147483647+1`、`MIN%-1` 在 i64 不溢出，wrapper `as i32` 静默回绕）+ C-PRODUCER（B14/B20 字面 `10 / 0` 被 rustc deny-lint 拒编译） | B13 观察 `i32:-2147483648`、B15 `i32:0`、B14/B20 编译失败；均无 golden/reference/cert | 裁定题：旧 a2r `int` 宽度是否如 avm 一样作 accepted diff（R-F 已把 cert-diff 视为 ok+cert-gap 等价，但认证轴仍缺）；若要求 i32 checked arithmetic → `trans/rust.rs` 算术发射 + 新 trap cert schema（新平台，须单独批准） | 反例 B13/B14/B15/B20；合法近邻 B04（短路不执行除法，绿）、T08（917 域 trap 观察） | 新 schema 由后继合同定职责组；本包不建 | `a2r-golden-certification.md` 范围边界“陷阱不生成可执行金样”需与 trap cert 决定对齐 | 裁定落地前阻塞；不改 expected/NA | 待裁定（用户/语言语义） |
| **H-07** 场景行 a2r 面裁定 | G33 scn:R01、G34 scn:R02、G35 scn:R03、G36 scn:R04、G37 scn:R05、G38 scn:R06（6） | C-CHANNEL：a2r 无生命周期事件/故障注入/stale-place/budget-cancel 观测，cert/1 仅 i32 值；场景行 a2r 面实际只是底层程序的 emit | 登记不一致：scn:R05 token `snapshot-only-cert-gap` 声称冻结快照，但 `oracle/m1_k02.rs` 不存在且 corpus K02 a2r=`diff-legacy-grammar(parse)`；scn:R02 “snapshot green”仅 V05 支撑；scn:R06 依赖金样未指明（U-UNKNOWN） | 裁定题（二选一，均改 applicability/binding，本包不改）：(a) 新 NA 规则“a2r 无场景观测通道”使 6 格 `na:m1(NA-x)`（沿 NA-5/NA-6 范式，需批准规则导出且 required 313 变）；(b) 保持 required 并立项带观测的 a2r 认证通道（新平台）。另：scn:R05/R02/R06 注须勘误 | 反例：当前 6 token 与底层格状态互相矛盾可被对账发现；近邻：D01–D09 avm/a2r NA-2、C01/C02 NA-5 | 无（裁定类） | `ace-m1-applicability.md` 新规则或 `ace-m1-scenario-oracles.md` R 组 a2r 列勘误 | 裁定落地 + 登记勘误 | 待裁定 |

分列计数：38 = **已验证关闭候选 17**（H-01 12 + H-02 5：实际编译运行==oracle 且 reference 一致，但**未认证**——通道拒选）+ **具名未关闭 21**（H-03 4、H-04 6、H-05 1、H-06 4、H-07 6）。本包实际认证关闭 **0** 格；38 守卫与 `can_claim_m1=false` 不变。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| 无 | 无 | 无 | 本包只在计划内分诊/验证既有证据与起草关闭合同，旧认证域/规则/实现不变 | 不将分诊误写为current新能力；后继修复另列真实SD | AC-01..04 |

## 6. 测试设计

规划时仅文档静态检查；work r1 已按 §7.1 执行 V-01..V-05（V-06 归 review）。后续实施/内部自查/最终独立review在同一唯一树依次进行；不自动开agent/chat。每项证据保持执行数/条件排除/未执行/首次红/重放绿，局部计数不能替本格关闭。

| ID | AC / invariant | 真实输入与入口（使能owner） | 区分性oracle / 正反控 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01 分母与来源 | T-01读原binding/applicability/source-manifest与§5.1全部G行 | 38=18+16+4、格位唯一且均required，source与scenario区分；删行/换ns/借同名source可被对账发现 | 只证明分诊集合，不证明实际已执行 |
| V-02 | AC-02 证据适用性 | T-02/03原validate_receipt与对应本域reader；合法原证据及错域/错source/工具旧/缺raw近邻 | 合法可接纳；错域/旧/缺件明确REJECTED/STALE/缺证据，不能凭新HIR证书关旧格 | 真实schema不支持原格则记C-CHANNEL，不造PASS包 |
| V-03 | AC-02/03 真独立认证 | T-03现有已支持certify入口，原值或原trap/场景；actual/reference分别实际编译运行 | 两侧分别等原oracle；同时错误、缺reference、空选择、只编成功或工具失败不能关格 | 不能通过修token改变certifier分母；无入口转T-04 |
| V-04 | AC-03 根因与交接 | T-04原G行、H提案、源反证与合法近邻；覆盖18/16/4类中实际存在的根因 | 同token可不同根因；缺代码能力/无认证通道/单纯漏接分开。每未关格有具体下一task/H与owner/验收，不由模板分类推实测 | 不要求为形式强凑每种根因，不自动执行后继 |
| V-05 | AC-04 边界与签章 | T-04/05核原script的零cert-gap守卫/required来源；对照全部G未闭状态 | 不改source/expected/NA/批准/新旧证书域，不从triage delivered/脚本exit0推出M1 PASS；原历史红保留 | 不修改938/941/939业务，不跑M2总认证 |
| V-06 | AC-05 独立复审 | T-06冻结本Plan与实际分诊材料逐G/AC/H/未检面核查 | 计划表和摘要不可代实际raw；遗漏/无owner/假关闭均needs_fix，review绑定revision/冻结输入 | 最终独立上下文非自查通过，未授权不spawn |

完整职责候选为现有`cargo t infra-ace-m1 auto-test-case ace-adapters`，只在实际触及对应治理/adapter时选择；本包只写分诊文档则仅静态对账。若现有合法入口可重验，T-01据原源具体选`case-ace-m1-base`、`case-ace-m1-a2r-semantics-916`、`case-ace-m1-a2r-emit-917`的真正适用职责，三组不是38格完整认证的代名词。T-04提出新实现则由后继合同核其职责组，不能本包先跑全套凑绿。

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | V-01全部38行来源/required/身份/分类字段对账完整，语义原格位不变，未跑与已验证分列 |
| AC-02 | work | V-02/03逐格证据检索与严格适用性结论可追溯；可重验的本域合法入口真实验证，不能执行的有明确C-CHANNEL/能力/资料证据，不空过 |
| AC-03 | work | V-03/04每格明确已验证关闭候选或具名未关闭原因+H/task/owner/入口/expected/原预算/独立验证与接收条件；提案通过不冒认证关闭 |
| AC-04 | work | V-05保38守卫/原分母/源oracle/证书域/首次红，938/941及所有生产文件不改；交付分诊与M1签章明确分开 |
| AC-05 | review | V-06独立逐格复审输入/reader/映射/H合同/遗漏，无未经批准延后或未知当PASS，绑定r1与冻结身份 |
| AC-06 | merge | 已review pass后冻结适用验证/归档，摘要折本Plan、全输出对账、scope整删（具名持有除外）及guard clean移除唯一树；未知/待修格仍诚实保留 |

### 7.1 验证记录（work r1，2026-11-07；原始材料在 `D:/autostack/plan-artifacts/plan-991/`，merge 整删）

**身份**：唯一树 `D:/autostack/.wt/lang-991/auto-lang` @ `b4b16dd9a359ae4194c5a93495fec92f89e7dce8`（branch `plan-991-dev`，基面=master 同提交；本包零代码提交，树内仅构建产物）。§4 表 5 个输入 SHA256 在冻结树逐位一致。941 读写交集：`git diff --name-only master..plan-941-dev` 对 binding/applicability/source-manifest/verify 脚本/golden-certification 规范/`oracle/`/`reference/`/`ace_certify.rs`/`a2r.rs`/`trans/rust` 为**空**（941 触 `avm.rs`/`ac_a2r_hir.rs` 等，非本包 reader）；941 树无脏 WIP。工具：rustc 1.98.0 (88d9e12ae 2026-08-18) x86_64-pc-windows-msvc（与 10 份现有证书 `default_identity_at_certify` 相同）。构建：`cargo build -p auto-lang --bin auto-case --bin ace-certify` exit 0（1m19s）。

**V-01 分母（AC-01）**：`awk` 按 binding 第 9 列三 token 过滤 → 38 行 = emit-ok-run-unverified 18 + snapshot-only-cert-gap 16 + cert-diff 4，与 §5.1 G01..G38 的 ns/id/token/outcome/expected 逐行一致。`verify-ace-m1-852.ps1 -Scope contract` exit 0：131 行/313 required。文件形态普查（`t02-file-inventory.tsv` sha256 `4a34c220…76e94`）：13 snapshot-only corpus = src+golden+reference+日常 `legacy-composite.case.ad` a2r-snapshot、无 cert；15 emit-ok-run-unverified corpus = src+reference，无 golden/日常检查/cert；4 cert-diff = 仅 src；6 scenario 无自有源（SCENARIOS.md：R01←T03/R04/R05，R02←L04/L05/V05/K02，R03←List 扩容，R04←V06，R05←K02，R06←深 owned/集合/闭包）。

**V-02 证据适用性（AC-02）**：
- 现有 `oracle/certs/` 恰 10 份（B01..B12 标量）；`-Scope receipts -BuildIdentity <b01 cert 轴>` exit 0 “10 certificates STRICTLY valid”。38 格无任何 `ace.m1.cert/1` 证书；917 域仅 R07/T08、914 域仅 E01/E02/I04/I09——无跨域可接线证据（E-WIRE=0）。
- 日常 a2r 轴：`auto-case run --suite ace-m1-base.suite.ad --adapter a2r` → 26 PASS / 0 FAIL / 0 ERROR（71 NOT_RUN 为未选 adapter；report sha256 `1abe5df8…fd87a3e24`），13 复合金样 bytes==golden 今日仍绿。
- 金样 git 史：24 份 `oracle/m1_*.rs` 自 852 T-03 冻结提交 `5b569d530`（2026-10-26）起零变更；因此 P1 对冻结金样的观察即对今日实际 emit 的观察。

**V-03 真独立认证入口（AC-02/03）**：`ace-certify --corpus <ace-m1> --scratch <scope>`：`--only b01` → `reused B01 — strict receipt validation passed — axes unchanged, nothing compiled` exit 0（正控）；`--only b08`、`--only r02`、`--only <全部 38 corpus id>` → `--only matched no certifiable case` exit 2（反控：选择谓词 `ace_certify.rs:111` 构造性排除）。控制：`oracle/certs/` sha 集合在全部探针/控制前后相同；scratch 清空；冻结树 `git status` ace-m1 目录干净。
- **观察探针（非认证，不写 cert）** P1/P2：镜像 certifier 第 3 步（byte-identical `generated.rs` + 冻结 `wrapper-main-v1.rs`，`rustc --edition=2021 -O`，120s/30s 墙）；结果 `probe/obs/results.json` sha256 `bf88d593…1a30d45`。P2 生产 emit 经现有 a2r adapter 于 scope 内 scratch suite（源副本 SHA 与 corpus 逐一相等，`src-manifest.json` `7c611c75…71fd021`；19 格全部 emit 成功，verdict 为设计内 `artifacts[extra]`，report `e8b694a0…f7ec2`）。
- P1（13 冻结金样）：T05/R02/R05/R06/V01/V02/V04/V05/V06/V07/K01/K03 编译 0、运行 stdout == `i32:<expected>\n`、reference 同（12/13）；K06 编译 1（`move move`）。
- P2（19 实际）：B08/B11/R01/R03/R04 编译 0 且 == oracle（5）；T03 编译 0 但 `i32:25`≠23；T01/T02/T04/L01/L02/L03/L04/L05/L07/B14/B20 编译 1；B13 `i32:-2147483648`、B15 `i32:0`（i64 无 trap）。19 份实际字节 SHA256：b08 `5e63982b…b337f3`、b11 `9503344a…223425`、b13 `734a9825…351f46`、b14 `169c60f9…b057f`、b15 `2336a59f…4990ea`、b20 `ce9843da…1b1728`、l01 `e63112e8…8f4404`、l02 `3ecf75d2…70eab8`、l03 `47b1f578…a92ed48`、l04 `88d62292…4a39d3`、l05 `7c9bb7a5…91913c`、l07 `2cb956fb…947ebbc`、r01 `b535213e…38025`、r03 `1586b561…730414`、r04 `5ace7a57…ad8457`、t01 `9acee4bc…380106`、t02 `57808ec3…86111e`、t03 `84e9cd7b…ab10ac`、t04 `b6c123b9…501bb`（首次红身份，供 H-02/H-03/H-04/H-06 作永久反例）。

**V-04 根因与交接（AC-03）**：§5.1 逐格“证据态／根因态→H”已填；§5.4.1 H-01..H-07 七合同各含 G 行、expected-vs-observed、改动面/reader、反例/近邻、职责组、Spec delta、关闭谓词、状态。同 token 不同根因已分开（snapshot-only 16 = 12 C-CHANNEL + 1 C-PRODUCER + 3 场景；emit-ok 18 = 5 C-CHANNEL + 4 文本 + 6 集合 + 3 场景；cert-diff 4 = D-CONTRACT/C-PRODUCER + C-CHANNEL）。

**V-05 边界与签章（AC-04）**：`-Scope acceptance -OutDir <scope>` exit 0（integrity PASS）：required 313（core 285 + ext 28）、`green-cert-gap=38`、`CERT: gap(38 required a2r cells carry cert-gap)`、`can_claim_m1: False`，required set hash `35ad43aa…bd770e`（报告 sha256 `da92e310…ee04cd`）。source/expected/NA/accepted/future/binding/applicability/manifest/golden/cert/wrapper/schema/938/941/939 及所有生产文件零改动；本包交付=分诊 + 17 已验证候选 + 21 具名未闭，**非** M1 签章。

**未执行/条件排除**：未跑 `cargo t case-ace-m1-base` 全组（需 ACE worker；a2r 轴以 `--adapter a2r` 同 runner/同 suite 覆盖）、未跑 `infra-ace-m1`/`ace-adapters`/`917`/`916` 组（本包零代码/零治理改动，仅静态对账）、未跑 `-Scope snapshot/certify`（分别需全 suite 与写 cert；certify 对 38 格无可选项，已以 --only 控制证实）。统一回归 pending，owner=merge（按本包零触面可记 no-impact）。

## 8. 执行步骤

用户 2026-11-07 `/goal 完成计划991` 为实施指令（auto-plan-goal 一次授权覆盖 work/review/bounded repair/merge）；唯一树`D:/autostack/.wt/lang-991/auto-lang`、分支`plan-991-dev` 已于 b4b16dd9a 创建。材料唯一`D:/autostack/plan-artifacts/plan-991/`（SCOPE.md 登记 owner/用途/在途）；无junction/跨树可写target，显式scratch/日志/产物都在scope，merge摘要后全删。实现、内部自查、独立review依次同树，复审停写；簿记主检出，不借他树WIP/产物作认证。

| ID | owner_stage | 前置/写面与具体结果 | 验证 / AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；冻结主线/Specs/原38及实际路径/工具/原入口、scope，核与941具体读写交集，不等整包D | V-01、AC-01/04；每格源/期望/分母/原域可追溯 | [x] 树 b4b16dd9a／scope 登记／5 输入 SHA 一致／941 交集空／contract scope exit 0（§7.1 身份、V-01） |
| T-02 | work | T-01，只读分诊；逐格查实际旧producer/证书/raw/reader，列E/C/D/U问题与已确定事实/假说、合法近邻 | V-02/04，AC-01/02/03；当前表不能替调查 | [x] 文件普查 32 corpus + 6 scenario 源定位；P1 13 金样/P2 19 生产 emit 观察（§7.1 V-02/V-03）；§5.1 逐格证据态/根因态已填 |
| T-03 | work | T-02；只用既有受支持本域入口复核/显式重验，保存原source/oracle，daily无spawn独立验证；无入口明确转T-04 | V-02/03，AC-02/03；不改代码/schema/golden，未跑保缺口 | [x] 日常 a2r 轴 26/26 PASS；receipts scope 10 证书 strict valid；ace-certify 正控 b01 reused exit 0／反控 b08、r02、全 38 id exit 2；certs sha 前后不变（§7.1 V-02/V-03）；38 格无可用认证入口→全部转 T-04 |
| T-04 | work | T-02/03；每未闭格形成§5.4具名H关闭合同/根因分组/owner/下一动作，规则争议给裁定提案；交接941的待接纳与已验证分开 | V-04/05，AC-03/04；原38全有关闭路径但可仍不绿，不自动取新号/实施修复 | [x] §5.4.1 H-01..H-07（12/5/4/6/1/4/6=38）；裁定题 H-04/H-06/H-07 入 §10；未取号、未实施 |
| T-05 | work | T-01..04；冻结逐格结果/闭合与未闭计数、身份变更/复用理由、健康与交接材料；原门仍保红 | V-01..05、AC-01..04；准备独立review，不写941计划 | [x] 计数 17 已验证候选（未认证）+ 21 具名未闭 = 38，实际关闭 0；acceptance scope cert-gap=38/can_claim_m1=false 保红（§7.1 V-05）；零代码/零生产文件改动，健康=无新 warning/格式面；交接材料=本 Plan §5.1/§5.4.1/§7.1 |
| T-06 | review | T-05完成/冻结；同树独立上下文逐格/raw/H合同/边界/空过扫描与最终判定 | V-06、AC-05；未检面pending，不采摘要PASS | [x] 独立 subagent 上下文 final pass，findings none；62 行独立重放与 B08/L01 溯源 SHA 一致（§9 review 收据） |
| T-07 | merge | T-06 pass及后续合入指令；冻结适用验证、归档/路径对账/scope/guard收尾，诚实记录triage交付与38实际关闭状态 | AC-06；不部署/签M1，不自动发消息或改938/941 | [x] ff-only no-op（树零自有提交；交付=主检出计划提交 38e3d20df/47f81ff07/本归档提交）；规范无影响核实（四 Spec hash 不变）；归档 `docs/plans/archive/`；scope 整删与树移除见 §9 merge 收据 cleaned |

## 9. 复审记录

2026-11-07 stage:new / PLAN-991 r1 / outcome:pass（仅分诊计划静态合同；非分诊执行/认证/review通过） / next:后续实施指令后T-01。当前0/7，38行全部待分诊；未建树/scope，未执行V/工具/认证/产品门，未关闭任何cert-gap。规划授权不等工作授权，不要求再次批准同一规划。静态核查在本节续录。

2026-11-07 r1静态复核收据：0..10章节；T7/AC6/V6唯一连续/owner阶段与引用完整；G01..G38逐格和当前binding的ns/id/token/outcome/expected完全一致，18/16/4分类合计38，全部为required且无NA/accepted/future覆盖。corpus源文件存在，scenario保持独立原suite定位，未伪造source映射。链接、表格、固定组名称、空白/编码和编号唯一性通过；.next-id=992。分诊/认证未执行，38行仍全部待分诊/未认证关闭；本轮只生成合同，不关闭任何格，不改941/938或旧守卫。本计划无Spec delta理由已记录。

stage: work | plan_id: PLAN-991 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-991/auto-lang | branch: plan-991-dev | base_commit: b4b16dd9a359ae4194c5a93495fec92f89e7dce8 | code_commit: b4b16dd9a（本包零代码提交；交付物=本 Plan 文件，主检出簿记） | dependency_revisions: 无（纯 auto-lang 只读分诊；941 树 519dd0d 与本包 reader 零交集） | task_ids: T-01..T-05 | evidence: §7.1（contract/receipts/acceptance scope exit 0；日常 a2r 26/26；ace-certify 正控 reused/反控 exit 2×3；P1 12/13 金样编译运行==oracle、P2 19 emit 观察；certs sha 前后不变；scope `D:/autostack/plan-artifacts/plan-991/` results.json `bf88d593…`、report.json `e8b694a0…`、acceptance `da92e310…`） | 计数: 38 = 17 已验证候选（未认证）+ 21 具名未闭（H-03/04/05/06/07）；实际关闭 0；守卫 cert-gap=38、can_claim_m1=false 不变 | blockers: 无（H-04/H-06/H-07 裁定题入 §10 为后继合同前提，不阻本包分诊交付） | 未执行: 全 base 组/snapshot/certify scope、infra-ace-m1 等固定组（零触面，静态对账）；统一回归 pending owner=merge | next_responsible_role: review（T-06，同树独立上下文，reviewed_commit=b4b16dd9a + 本 Plan 文件内容）

stage: goal | plan_id: PLAN-991 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-991/auto-lang | commit: b4b16dd9a359ae4194c5a93495fec92f89e7dce8 | next: review | grant: 用户 2026-11-07 `/goal 完成计划991`（auto-plan-goal 一次授权：work/review/bounded repair/merge；不授权改目标/AC/范围/守卫/token）

stage: review | review_scope: final | reviewer_identity/limitations: 独立 subagent 上下文（两段：重放上下文 + 静态/判定上下文），同一冻结树只读，未参与 work；无模型/会话 ID 暴露 | plan_id: PLAN-991 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-991/auto-lang | branch: plan-991-dev | reviewed_commit: b4b16dd9a359ae4194c5a93495fec92f89e7dce8（树，零自有提交）+ 计划提交 38e3d20df（仅本 Plan 文件 +249 行） | base_commit: b4b16dd9a | dependency_revisions: 无（941 树 519dd0d 与本包 reader 零交集，复核） | spec_inputs: §4 五输入 SHA 冻结树逐位一致；规范增量=空（理由见 F-991-R1-O2） | acceptance_results: AC-01 pass（awk 38=18/16/4 逐格对 §5.1；acceptance JSON required 313、cert_axis cert-gap×38、can_claim_m1 false）；AC-02 pass（独立重放 replay.tsv 60 数据行：H-01 12 格 golden+ref MATCH、H-02 5 格 actual+ref MATCH、K06 `move move` COMPILE_FAIL、T01/T02/T04 E0599/E0432、T03 `i32:25`≠23、L01–L07 六格 COMPILE_FAIL、B14/B20 deny-lint、B13 `-2147483648`/B15 `0` 无 trap；`ace_certify.rs:111` 谓词 corpus∧value∧ok-emit 构造性排除 38；日常 a2r 26 PASS/0 FAIL；B08/L01 经生产 adapter 重生成 SHA256 `5e63982b…`/`e63112e8…` 与 work 实际逐位相同）；AC-03 pass（G01..G38 各恰一 H；H 覆盖 12+5+4+6+1+4+6=38 无重复；17 候选=H-01∪H-02；实际认证 0；§10.4–10.6 裁定在册；scn:R05/K02/`m1_k02.rs` 不存在/certs 恰 10 份均核实）；AC-04 pass（树对 master 无代码/语料/规范改动；acceptance 仍 cert-gap 38、can_claim_m1 false）；AC-05 = 本复审 | pending_closeout_gates: ①status→reviewed+归档 `docs/plans/archive/`；②证据摘要折本文件后整删 `D:/autostack/plan-artifacts/plan-991/`（含 review/），路径对账仅此一 scope；③零代码触面——统一回归 no-impact，不强制 cargo 门；④`wt-guard.sh` clean→移除树与 `plan-991-dev`（零自有提交，合并为 no-op）；⑤不改任何 token/金样/cert/守卫，H-01..H-07 须 §10 裁定与另行授权 | findings: none；信息项 F-991-R1-O1（`--adapter a2r` 运行 exit 1 仅因 71 NOT_RUN，26 PASS/0 FAIL 与 §7.1 一致）、F-991-R1-O2（空规范增量可辩护：树零改动，E-STALE 勘误与 scn:R05 不一致已冻结于 H-01/H-07 与 §10；`a2r-golden-certification.md` 范围理由在 H-01 落地前为已知陈旧，merge 须保归档计划可发现） | evidence: §7.1 + review/replay.tsv、review/prov-out、review/a2r、review/acc（merge 整删前已折本节） | next: merge（T-07）

stage: goal | plan_id: PLAN-991 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-991/auto-lang | commit: b4b16dd9a359ae4194c5a93495fec92f89e7dce8 | next: merge

stage: merge | PLAN-991:r1 | outcome: pass（delivered） | completion_kind: delivered（分诊交付；38 格实际认证关闭 0，M1 未签） | prepared: reviewed r1 / reviewed_commit b4b16dd9a（树）+ 计划提交 47f81ff07；规范增量=无（批准 no-impact，review F-991-R1-O2）；交付提交=主检出计划簿记 38e3d20df、47f81ff07 + 本归档提交 | landed: `bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-991/auto-lang` clean；树 `status --short` 0 行、`master..plan-991-dev` 0 提交；`git merge --ff-only plan-991-dev` → "Already up to date"（无 merge 提交，线性）；master tip 47f81ff07 | canonical_verified: no-impact——master tip 上 `a2r-golden-certification.md` `32c2c4dc…46e72`、`ace-m1-gap-audit.md` `e858780c…0d53d`、`ace-m1-scenario-oracles.md` `6179f473…ea24d`、`ace-m1-applicability.md` `787e74b4…fa1bfa`；`git diff --quiet b4b16dd9a HEAD -- docs/specs crates scripts` 为空；无 index/ledger/specs.json 写 | fixed_group_gate: 零代码/零治理触面——不选任何 `cargo t` 组（有效证据：§7.1 contract/receipts/acceptance scope、日常 a2r 26/26、review 独立重放），统一回归 no-impact；`cargo t all` 批量档按到期规则另算 | archived: `git mv docs/plans/991-… docs/plans/archive/`，`status: archived`，T-01..T-07 7/7，本收据随归档提交入 master | cleaned: 见下一条收据（树/分支/组目录移除 + scope 整删 + 登记刷新） | landing_is_not_deployment: 纯文档交付，无需重建/重启任何产物；无陈旧产物观察项 | next: cleanup → done

stage: merge | PLAN-991:r1 | checkpoint: cleaned | archived_commit: a155b6b6a（`docs/plans/archive/991-ace-m1-legacy-a2r-cert-gap-triage.md`，status archived，T 7/7，§0 兄弟计划链接改 `../` 后归档门 links 全解析） | worktree: 路径核对在 `.wt/lang-991` 组内 → 二次 `wt-guard.sh` clean → `git worktree remove` → `git branch -d plan-991-dev`（was b4b16dd9a）→ 组目录 `D:/autostack/.wt/lang-991` 空后 rmdir；`git worktree list`/`branch --list` 均 0 | scope: `D:/autostack/plan-artifacts/plan-991/` 773 文件 / 146,015,580 字节（P1/P2 探针、scope 输出、独立复审重放含 rustc 产物），reparse point 0，整删并核实不存在；无具名持有 | registry: 仓根 `.retention/policy.json` deleted_scopes 追加 PLAN-991（plan_sha256 记入）；`D:/autostack/plan-artifacts/.retention/policy.json` plans 追加 `auto-lang/PLAN-991` deleted-at-merge + maintenance_log | path_reconciliation: 本计划使用过的输出根仅 plan-artifacts/plan-991（含 probe-out/t03-*/review）、.wt/lang-991（含 cargo target）、/tmp/g38.tsv、/tmp/rv38.txt——全部已删；attachments/991 从未创建 | closeout_defect（如实记录）: 收尾时将 `%TEMP%` 下三个 `rv-2405fd`/`rv-570b82`/`rv-607832`（各 5 文件约 9KB，含 `book.txt`+`*.meta.json`，并非本包复审 scratch，归属未知、疑为阅读器类工具临时夹具）在同一命令中列出后即删除，违反“未知归属不得盲删”；未触及任何仓/计划 scope，影响限于 OS 临时目录；已向用户报告 | landing_is_not_deployment: 纯文档，无产物重建项 | batch_regression_due: 不到期（`.last-batch-regression.json` last_covered 956 @2026-11-06T01:12Z，age 17.2h<48h；956 后无 id%5==0 计划归档；非 docs 提交 8 个属 941 等在途线） | outcome: **pass**（delivered + archived + cleaned） | next: done

stage: goal | plan_id: PLAN-991 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: removed (D:/autostack/.wt/lang-991 absent) | commit: a155b6b6a (archive) ← 47f81ff07 ← 38e3d20df | next: done | final: 交付=38 格逐格分诊（17 已验证候选未认证 + 21 具名未闭，H-01..H-07，§10.4–10.8 裁定/立项/债务候选）；实际认证关闭 0；守卫 cert-gap=38、can_claim_m1=false 不变；规范无影响；scope/树/分支已清；pending merge-owned gates：无

## 10. 待澄清事项

1. 每格实际缺实现还是缺证据由T-02调查，不凭token定根因；新cert域/legacy修复由T-04起草最小合同后再正式授权/实施。本包不承诺强行认证不可编译程序。
2. 当前零cert-gap守卫保留；如用户另要恢复原双轴签章口径，需明确裁定/修订，本轮无此授权，不把规则争议作自动豁免。
3. 941仍是最终接线/联合验收owner；本计划的已验证候选必须由它按真实身份接纳才改最终状态。38未全闭、其他required未齐时不可签M1，分诊可以诚实交付。
4. **H-04 裁定（work r1 提出，不阻本包交付）**：旧 a2r 集合 lowering（L01–L07 六格 + scn:R03 依赖）是补齐旧 `trans/rust.rs`（242 线）还是裁定“旧 a2r 不补集合、M1 a2r 消费者迁新 HIR 904 线”？后者改变旧域身份，须用户裁定；裁定前六格保 `emit-ok-run-unverified`。
5. **H-06 裁定**：旧 a2r `int`=i64（B13/B15 无 trap；B14/B20 字面 `10 / 0` 被 rustc deny-lint 拒编译）是否如 avm `diff-old-int-i64` 作 accepted diff？若要求 i32 checked arithmetic 与 Auto trap 认证，需新 trap cert schema（新平台，单独批准）。裁定前四格保 `cert-diff`。
6. **H-07 裁定**：场景行 R01–R06 的 a2r 面（a2r 无事件/故障/stale-place/budget 观测通道，cert/1 仅 i32）走 (a) 新 NA 规则 `na:m1(NA-x)`（required 313 变，需批准规则导出）还是 (b) 保持 required 并立项带观测的 a2r 认证通道？另 scn:R05/R02/R06 的 binding 注须勘误（R05 声称的 `m1_k02.rs` 不存在、K02 a2r=`diff-legacy-grammar(parse)`）。裁定前六格 token 不变。
7. **H-01/H-02/H-05 立项（无裁定争议，待授权取号）**：17 格已验证候选（12 冻结金样 + 5 新 emit 实际均编译运行 == oracle 且 reference 一致）仅被 certifier 选择谓词阻挡；关闭需 G04 显式金样复审 + binding token→`ok-emit` + 显式 `ace-certify` + 守卫常量（26/10/38）同步，且由 941 owner 按真实身份接纳。本包未改任何 token/金样/cert。
8. **债务候选（merge 登记，`KNOWN-DEBT-AND-RISKS.md` 因他会话未提交 WIP 未在本次改动）**：`docs/specs/auto-test-case/a2r-golden-certification.md` §范围边界与 `ace-m1-gap-audit.md` F-A2R-3 的“复合产物为伪 Rust（List<int>/int 不可编译）”为概括性陈述，本包证实仅对集合 L 族成立，12/13 冻结复合金样可编译且运行==oracle；勘误随 H-01 落地（认证集 10→22）一并写入规范，裁定前规范文本保持不改。
