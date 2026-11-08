---
plan_id: PLAN-992
status: reviewed
feature_name: 旧A2R 17格值例金样审阅与独立认证
author: [Codex]
created_at: 2026-11-07
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991]
current_step: 6
total_steps: 8
---

# [PLAN-992] 旧A2R 17格值例金样审阅与独立认证

## 0. 变更摘要

认领991 H-01的12个既有复合金样与H-02的5个缺金样值例；不修改emitter，通过逐源G04语义审阅、精确登记与原域真实认证，形成17格实际关闭证据。

2026-11-08 `/goal 完成计划992`为实施指令（一次授权覆盖work/review/bounded repair/merge）；work阶段（T-01..T-05）已完成：17格G04重审+原域显式认证全certified、扩展suite并入日常snapshot（26→31）、证书10→27 strict有效、cert-gap 38→21、can_claim_m1=false不变；详见§7.3/§9。本包未修改emitter与939/941，M1范围/原source/oracle/预算不变。

## 1. 目标

关闭本包§5逐格清单的原旧A2R值例认证义务；完整actual、独立reference、已审golden、strict receipt、零spawn日常及来源登记全部可核，交941原owner独立接纳。

非目标：H-04集合、H-06宽度/trap、H-07场景裁定；补任意新语言/新HIR后端、重写所有旧A2R、放宽receipt/预算/NA/required、修939/941代码或计划、隐式认证/自动改golden。 本包不修改emitter；发现真实producer缺陷先阻塞/修订，不能混入17格认证接入。

## 2. 架构方案

继承991分诊且正式依赖为PLAN-991；全链统一走旧source→production A2R→已审golden/完整actual→原显式认证→原daily/strict核查→逐格交接。未来实现确认实际基面/工具/SHA，WIP/reviewed不是D；无直接依赖939/941整包交付，不借其未提交代码。

## 3. 技术栈

- `crates/auto-lang-core/src/test_cases/ace_certify.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang-core/src/test_cases/a2r.rs`（现存；T-01重核实际符号）。
- `scripts/verify-ace-m1-852.ps1`（现存；T-01重核实际符号）。
- `scripts/tests/test_ace_m1_acceptance.py`（现存；T-01重核实际符号）。
- 共同入口：`crates/auto-lang/src/bin/ace-certify.rs`、`crates/auto-lang-core/src/test_cases/ace_certify.rs`、`crates/auto-lang-core/src/test_cases/a2r.rs`。
- 真实语料根：`crates/auto-lang/test/cases/ace-m1/`（binding/applicability/source-manifest/src/oracle/certs/reference与原Case）。

## 4. 需求分析与背景调查

用户2026-11-07授权依据991正式规划17格认证、H-05闭包与H-03文本后继，明确本轮只做规划，不施工、不改正在执行的939/941、不启动其他会话。只允许本组三份Plan及中央取号簿记；未授权golden实际变更/认证执行/合入。无用户指定模型、工期或token预算。

只读取材main=`fcc474acd`。读991 §5.4.1/§7/§10与current Specs/实际拆分代码后，区分历史证据和当前观察；991旧scope已清，不能恢复ignored附件依赖。源码路径从原`trans/rust.rs`重锚到现子模块；当前证书仅10份，raw binding仍有38个cert-gap token。这是规划静态事实，不是本轮运行结果。Spec有历史过概括（“所有复合伪Rust”）及917 source-slice与当前runtime-helper投影差额，均按本包SD/T-01处理，不据代码改原要求。

| 读取输入 | SHA256 |
|---|---|
| `docs/plans/archive/991-ace-m1-legacy-a2r-cert-gap-triage.md` | `37fbf69aff2616d8ce03bb537f5592f1f1bab2c6b84735c4f1a313e230a64632` |
| `docs/specs/auto-test-case/a2r-golden-certification.md` | `32c2c4dc2dd9f7eee13745f63a217b01454e5162d73ce235774da766dfc46e72` |
| `docs/specs/auto-test-case/ace-m1-gap-audit.md` | `e858780c9d48108f833cb639fb7a27753ad7322ed1ba16a871f058235670d53d` |
| `crates/auto-lang-core/src/test_cases/ace_certify.rs` | `6f1919edbf314961be244b400d170860c375f6463b3c9daab98c1874b158d9e3` |
| `scripts/verify-ace-m1-852.ps1` | `1a9c3bb809609a1b1e310b1723c205cfe17ff802b94f165259dc31dcea86f829` |
| `scripts/tests/test_ace_m1_acceptance.py` | `d72fee5402d72805dda79d7f027ec454b6c82874d6f3c711fdf7d09cfaacc478` |
| `crates\auto-lang\test\cases\ace-m1\binding.tsv` | `303cdde9e8b5d56a8debeb5ff261519d2ee5c263684a4eefc84fafbe3917a259` |
| `crates\auto-lang\test\cases\ace-m1\source-manifest.json` | `5209f30010be94da007343b502e85f9ecdb85a57940315dfbad8314e4589db5a` |

## 5. 详细设计

### 逐格输入与原独立期望（全部待本包验证）

根`crates/auto-lang/test/cases/ace-m1/`；原source=`src/<ID>.at`、reference=`reference/<ID>.rs`，原lower-id golden/receipt名由T-01按当前实际目录对账。下列每行均是**corpus:<ID>:a2r**，不得借scenario同名格；原source/reference/expected hash在T-01冻结。

| ID | 原token | 原outcome / expected | 本包预期动作 |
|---|---|---|---|
| T05 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| R02 | `snapshot-only-cert-gap` | `value / 9` | 既有golden语义重审、字节不改、认证 |
| R05 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| R06 | `snapshot-only-cert-gap` | `value / 21` | 既有golden语义重审、字节不改、认证 |
| V01 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| V02 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| V04 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| V05 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| V06 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| V07 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| K01 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| K03 | `snapshot-only-cert-gap` | `value / 42` | 既有golden语义重审、字节不改、认证 |
| B08 | `emit-ok-run-unverified` | `value / 7` | 新golden审阅/冻结、认证 |
| B11 | `emit-ok-run-unverified` | `value / 7` | 新golden审阅/冻结、认证 |
| R01 | `emit-ok-run-unverified` | `value / 19` | 新golden审阅/冻结、认证 |
| R03 | `emit-ok-run-unverified` | `value / 23` | 新golden审阅/冻结、认证 |
| R04 | `emit-ok-run-unverified` | `value / 51` | 新golden审阅/冻结、认证 |
### 首包范围与实施决定

H-01的12个golden字节必须保持不变：T05、R02、R05、R06、V01、V02、V04、V05、V06、V07、K01、K03；991曾实测双侧符合oracle但不是正式证书。H-02的B08/B11/R01/R03/R04从**当前**旧producer现场生成proposal，经逐源审阅后才冻结新增`oracle/m1_{b08,b11,r01,r03,r04}.rs`，不得直接把已删除scope里的旧actual当唯一材料。

T-02先做一例H-01 record和一例H-02命名参数B11的有界全链可行性核查，再审全17个；B11冻结原返回7、源码求值序与命名绑定，914闭包形式允许但语义必须审。若某个现有golden与source语义不符，本包停止该例并具名复现，不能顺手改H-01字节或改expected；需修订范围。

T-03明确新增文件：`crates/auto-lang/test/cases/ace-m1/legacy-a2r-cert-992.case.ad`和同名`.suite.ad`（拟新增，仅H-02五个旧a2r-snapshot检查，ID带corpus域）；原12个snapshot检查仍在legacy-composite，不复制成第二义务。拟固定组`case-legacy-a2r-cert-992`由T-03沿`scripts/test-groups.toml`现机制注册，尚不可执行。原script snapshot显式运行原base与该扩展并核原/新完整集合与相同工具身份；不是新runner或新证书schema。若现script不能无歧义并入，T-01冻结最小原入口适配，不另做总验收平台。

目标：certifiable10→27、缺口38→21、a2r日常检查26→31（原26+新5）；原base语义检查集合不变。只有17份新cert逐项strict PASS才允许此计数；原10合法reused/STALE重验如实记，剩余21格精确为H-03/04/05/06/07。更新规范中过度概括的“所有复合伪Rust”说法，限定为991已证集合等具体域，不宣称任意泛型/复合支持。

### 原义务、证书与更新顺序

只消费旧production A2R，入口为`auto-lang-core/src/test_cases/a2r.rs`→`transpile_rust_with_source_dir`/实际RustTrans（`trans/rust.rs`组合根及`trans/rust/`子模块）；不引入旧VM执行fallback，不用新HIR Rust/C证书抵旧域。现场source→完整actual字节→经显式G04语义审阅的golden→冻结wrapper→actual与独立reference各实际编译运行→原strict receipt→日常emit/bytes/身份核查。原source、reference、outcome/expected逐字保留；不由actual生成reference/expected，不剪main/分支/依赖以过rustc。

G04是具体审阅任务：在任务表指定点，记录每例Auto语义→Rust求值序/绑定/资源/返回值的审阅结论和proposal/golden SHA，才允许登记`ok-emit`与生成正式证书。新增/重冻结golden不自动批准：实现上下文的审阅为工作材料，最终T-06仍以独立上下文核全部差异；若无法证明语义忠实则本例阻塞，不靠认证恰得同一个整数覆盖。用户本轮批准规划，不是提前批准未来golden字节。

`ace.m1.cert/1`、原wrapper与flags `[--edition=2021,-O]`、std-only/deps空外部依赖、strict REJECTED/STALE、实际启动rustc身份、所有原预算不变。certifier仍只选corpus/value/ok-emit；不能删选择谓词或使任意token自动可认证。只有本包经G04审阅的目标格允许改a2r token+该格说明，任何token先绿但无有效证书的中间态不得交付。新证书原路径`oracle/certs/m1_<lower-id>.cert.json`按原格式跟踪；原10份证书仅严格验证后可reused，不为计数强制重认证、不假设必然有效。坏结构不以--force覆盖，过期如实显式重验，生产二进制/工具变化按实际身份核准入。

### 当前登记与日常认证边界

binding仍131行/12列、313 required、批准NA/32 accepted/45 future不变，required集合hash保持；仅目标corpus:a2r token及说明可改。`source-manifest.json`只更新本包实际影响的binding/新增golden/cert/suite文件身份，不重写未变历史批准或宽泛替换hash。cert-gap按真实required逐格重算，**零gap守卫保留**；本包通过不签M1。原38分诊历史不重写，Spec以新状态段记录实际关闭格与余账。

保原base套件36 cases/97 checks的冻结义务，不能为补5/4例扩入原不支持的AV/旧VM路径。992创建显式旧A2R-only扩展suite（新路径见其§5），原base 26个a2r日常检查保留，扩展仅新5例；994再加4个文本检查。`snapshot`完整日常入口必须显式包含这些已审扩展及原base，不能只让专用新组绿而原日常漏测。原`source` scope的97下限/集合不机械变更，合法增加的独立a2r检查数量由真实Case/check集合审计；漏任何原/新必需检查、空选择、缺证书、跨域receipt均红，不能只提高一个计数常量就算覆盖。

原script acceptance exit0仅会计有效；余gap仍>0时`can_claim_m1=false`。`receipts`/`snapshot`须strict通过，接受来源绑定原Case及build-rustc-identity；JSON父PASS/目录存在不代证据。daily compiler/cargo/被认证目标spawn=0，预构建工具准备单列，显式certify才真实compile/run。

### 资源、冻结与交接

使用原受控工具/Job/并发排水、120s compile/30s run/每流1MiB及适用更严帽；不另造runner/meter，不自动下载工具，不改变原压力/ignored/平台条件。所有新probe、proposal、stdout/stderr、cert scratch、变异副本都在本包唯一scope；源码和常规target仅本包树。所需正式认证/性能与939/941错峰，避免同机重载污染时间证据。

最终对941交付格位→旧producer/source/oracle/证书/check/raw终态/内容hash/余账的只读材料。941是否接纳由原owner后续按真实身份执行；本包不修改其计划/代码/在途文件，不自动发消息/启动会话。实际本域认证已闭与941组合已接纳分别记；不能因接纳尚未发生绕过本包正确性审查，也不能将本包局部绿称M1已过。

### 三包顺序与共享写面

推荐正式交付顺序：[992](992-legacy-a2r-seventeen-value-certification.md)→[993](993-legacy-a2r-write-capture-move-fix.md)→[994](994-legacy-a2r-text-certification-repair.md)。992提供本域认证/日常扩展接缝，993/994都正式依赖D992；993/994私有调查/补例设计可并行，emitter注册/方法分类/Case/manifest/脚本/证书/Spec全文件按稳定SHA单写，不以append-only并写。994不硬等D993才能准备；若改变交付顺序，T-01重算真实合法cert/check/gap集合，不改变零gap判据或丢任何别包已交成果。

当三包全部真实关闭且无其他变化：38→21（992）→20（993）→16（994）；cert10→27→28→32；snapshot26→31→31→35。剩16=H-04集合6+H-06 trap/宽度4+H-07场景6，仍待原裁定/后继，不因此签M1。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/a2r-golden-certification.md` | 仅本包实际格位认证集/缺口与旧/新producer域说明；保历史原值与零gap守卫 | 只沉淀reviewed实际行为；本轮不写Spec | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-gap-audit.md` | 仅本包实际格位认证集/缺口与旧/新producer域说明；保历史原值与零gap守卫 | 只沉淀reviewed实际行为；本轮不写Spec | AC-01..04 |

## 6. 测试设计

所有V为拟执行；T-01使能真实反例/合法近邻，T-02/03接永久用例，T-04/05完整关闭，最终T-06独立复审。

| V | AC / invariant | 输入与实际入口/owner | 区分性oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02/03 17值例语义与身份 | T-01/02，全部原src/reference/golden及当前production actual | 17原值逐例正确，12旧golden不变、5新golden有G04表；B11=7不被旧AVM差异排除 | 只此17，不能替新HIR或其他legacy格 |
| V-02 | AC-01/02/03 登记/扩展日常完整性 | T-03/04，原base与拟992 a2r-only扩展，删B08检查或证书反控 | 原base保持、新5可运行；缺任一新/旧check红，26+5=31不由自报数 | 不增加旧VM/AC/AV支持，97原义务不改 |
| V-03 | AC-01/02/03 原域真实认证 | T-04，原ace-certify逐本包ID，错golden/坏cert/旧工具控制 | 17新cert strict PASS，actual/ref各对oracle；原10验证可复用，不放宽谓词/schema | 余21 gap保持，empty/--only未选中非PASS |
| V-04 | AC-04 余账/范围/身份 | T-05原binding/required/manifest与本包每行；所有旧域及941交接 | 只本包格位转已认证；313/批准集合不改、旧首次红保留；缺证据不填0/PASS，剩gap如实且claim false | 不操作939/941，不由局部判据替全M1 |
| V-05 | AC-05 独立复审 | T-06同树独立上下文，原src→producer→golden/actual→reference/cert→reader逐项 | 重建G04与全部反控、SD/遗漏/警告/格式；假关闭/未知必需面needs_fix | 实现内部自查不顶最终review，不另开会话 |

### 共同反控与验证入口

T-01冻结可用CLI/固定组/工具及完整原Case集合，未来显式例：预构建`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only <本包逐例列表> --scratch <scope>/cert-scratch`，使用原证书路径；不同shell的最终命令在work记录实际绝对路径。原`powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope snapshot|receipts|acceptance -RepoRoot <tree> ...`中三scope分开运行，不把管道字符当参数；snapshot给scope输出，receipts提供当前BuildIdentity。具体CLI参数T-01以真实帮助/源码冻结，本轮均未执行。

独立reference是已有手写`reference/<ID>.rs`；两侧各对原stdout `i32:<expected>\n`及成功终态，而非相互相等。变异在scope副本，不触正在执行计划：golden变1B/expected错/receipt错source或跨新HIR域/截断/旧schema/工具身份漂移/缺Case或缺cert；strict必须拒或STALE。单例重新认证/编译失败不得留下可授信半证书或覆坏证书；恢复合法副本后可重试。缺工具/超时/输出溢出保持infra/resource，不充语言负例。

实现→内部自查→最终独立上下文review同一唯一树顺序进行，不自动创建agent/chat；内部PASS不替最终review。冻结职责并集在最终审查后按真实触面运行一次，未变身份的有效证据逐项说明复用，不为文档/提交重复全experimental/all。周期`cargo t all`只按due在主检出单实例，由原owner排队，不成为本包全局门。

职责基础为现有`cargo t infra-ace-m1 auto-test-case ace-adapters case-ace-m1-base`；按实际修改冻结必要并集，原ignore/平台/重内存条件保留。992不修改emitter，默认不跑完整a2r；若实查触及producer先修订scope。若只改计划文档仅做静态检查，不运行这些产品门。拟新case组先注册后实际选中，不能写了Case文件即算已执行。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 本包全部原source与独立reference结果正确，所需production修复/语义重审、合法与非法近邻符合§5/§6，不以typed/新HIR替旧域 |
| AC-02 | work | 全部目标golden有逐例G04审阅与SHA，actual现场完整生成且字节一致，原schema真实双侧编译运行与strict receipt全部通过 |
| AC-03 | work | 原base与本包已审扩展日常检查完整、零认证面spawn，坏golden/证书/身份/漏case与合法复用反控正确；相邻职责与原条件通过 |
| AC-04 | work | 每格关闭证据与原分母/批准/余账守恒，全部reader/manifest/SD候选与941只读交接清楚，未裁定格和全M1仍不可冒绿 |
| AC-05 | review | 同树独立上下文逐AC/V/SD/source/golden审阅/实际命令/reader/遗漏健康核验，结论绑定r1/冻结源码与配置 |
| AC-06 | merge | review pass后冻结适用并集/漂移核验、SD沉淀读回、来源交付/归档，全输出路径核对、scope整删与guard清树；不改939/941计划 |

### 7.3 work阶段验证记录（2026-11-08，树`D:/autostack/.wt/lang-992/auto-lang`，base`36b5fbd4a`，code`ec92e5ff4`）

- **入口冻结**：`cargo build -p auto-lang --bin auto-case --bin ace-certify` exit 0（1m09s，5+131条既有警告，无新增）；rustc 1.98.0 (88d9e12ae 2026-08-18)——与旧10证书`default_identity_at_certify`一致（identity_source=PATH）。
- **认证**：`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only t05,r02,r05,r06,v01,v02,v04,v05,v06,v07,k01,k03,b08,b11,r01,r03,r04 --scratch <scope>/t04-certify/cert-scratch` → 17 certified exit 0；全量（无--only）→ 27 reused/0 failed exit 0。
- **三scope**（`powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope … -RepoRoot <绝对路径>`）：snapshot=31 a2r全绿+27证书strict+base 97/扩展5全绿+认证面spawn 0；receipts（-BuildIdentity取自m1_b01证书）=27有效；acceptance=cert-gap **21**/can_claim_m1 **false**/required 313/future 45/ruled-accepted 32；contract=131行+manifest全验PASS。
- **固定组**：`cargo t case-legacy-a2r-cert-992` PASS、`cargo t case-ace-m1-base` PASS、`cargo t infra-ace-m1` PASS（脚本终版复跑）、`cargo t auto-test-case` PASS、`cargo t ace-adapters` PASS。
- **反控**（scope副本，树未触）：C1金样1B漂移→`fail B11 actual != golden` exit 1+证书SHA前后不变；C2证书加未知字段→REJECTED（--force同样拒绝）；C3登记expected 7→8→既有证书REJECTED（registry desync fail-closed）、C3b删证书重认证→`actual-wrapper run mismatched the fixed oracle: stdout="i32:7\n" (expected "i32:8\n")` exit 1+无半证书；C4扩展套件删B08检查→snapshot FAIL `29 < 31 (checks were dropped)`。G02观测器三误报源（sccache包装、rustc再派生、ac身份探针`rustc --print sysroot`）以工具链祖先链+纯查询无编译轴分类修复，认证面判据（直连rustc/m1_*无工具链祖先）保留，regex单元判定三例+冷churn实跑+终跑0违规。
- **941只读交接**（原owner按真实身份接纳，本包不写941）：

| 格位 | source SHA16 | golden SHA16 | cert SHA16 | 终态 |
|---|---|---|---|---|
| corpus:T05:a2r | `84541676753c15d0` | `95f83961f608f29a` | `8330025c4b1cab95` | certified |
| corpus:R02:a2r | `6c74d77b3ba20bfc` | `09d3f186faf52c65` | `66fcd5f3bdf92ef3` | certified |
| corpus:R05:a2r | `f46125c461bdf557` | `11375a3948d66147` | `cdd3b76dc647b47b` | certified |
| corpus:R06:a2r | `c3f55da8433a4cc6` | `14fbe8ebff218723` | `dd3a825cc0bc4398` | certified |
| corpus:V01:a2r | `900acc88f43f9505` | `496f4c067efa8a8d` | `d47ac3d58a17708a` | certified |
| corpus:V02:a2r | `281396b1a3d5ea12` | `dea6dda290b25c1a` | `f5ce313be9c225ef` | certified |
| corpus:V04:a2r | `c9303cad597133e0` | `76ed2a76f78a0f02` | `c09fc07b58883576` | certified |
| corpus:V05:a2r | `734c211d1ea0e62d` | `63b72a7074c502b6` | `cabc5ac63740d230` | certified |
| corpus:V06:a2r | `7145928b2c570c84` | `4db41ba368309501` | `f7880bac1c9c964c` | certified |
| corpus:V07:a2r | `04844f208d458a04` | `6237a3350fa24546` | `e8887e36a3dbff47` | certified |
| corpus:K01:a2r | `d287a88b8009e586` | `a41dd1f0074cfcef` | `67c4798f41d75d82` | certified |
| corpus:K03:a2r | `7118326786b8a6ed` | `350b633e11fce94c` | `1238ad561df9b7f8` | certified |
| corpus:B08:a2r | `4e068c9c8ccaad2b` | `5e63982b3dd1cfbe` | `dba4660fe60f60ad` | certified |
| corpus:B11:a2r | `730c801b1aca7e53` | `9503344ae9737a70` | `55e0f7e6c858623d` | certified |
| corpus:R01:a2r | `6ee9917c5c08e48d` | `b535213ebf143c4a` | `8c6650fd9eddf82b` | certified |
| corpus:R03:a2r | `5f8253fa744f39f4` | `1586b56106fe37d3` | `a155853a06ff8fc3` | certified |
| corpus:R04:a2r | `f3c69ae52141466b` | `5ace7a57920a2853` | `767757a1b0d1fa91` | certified |

- **剩余21格**（acceptance ledger冻结，本包不预填/不改守卫）：corpus H-03 T01/T02/T03/T04（文本lowering缺陷）、H-04 L01/L02/L03/L04/L05/L07（集合缺失）、H-05 K06（move重复）、H-06 B13/B14/B15/B20（trap/宽度裁定）；scenario H-07 R01/R02/R03/R04/R05/R06（场景面）。全M1仍不可冒绿：`can_claim_m1=false`（cert-gap 21>0 且 future 45 在账）。
- **材料根**：唯一scope `D:/autostack/plan-artifacts/plan-992/`（t01-freeze/t04-certify/t04-controls/probe等，merge折叠后整删）；树内常规target正常，无第二输出根。

## 8. 执行步骤

预定唯一树`D:/autostack/.wt/lang-992/auto-lang`、分支`plan-992-dev`；已于2026-11-08按`/goal 完成计划992`实施授权创建（base=master`36b5fbd4a`），work代码提交`ec92e5ff4`。材料只在`D:/autostack/plan-artifacts/plan-992/`（已登记owner/用途/在途）；无junction、跨树可写target或第二输出根。常规target在专树，外置target只能在本scope。原始证据merge折计划摘要后整删，独立review同树停写。

| T | owner_stage | 前置/写面 | 具体结果/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 无实施依赖；后续实施指令 | 核D991、当前17source/义务/入口/旧10证书与全部读者，冻结一例record+B11全链及原预算/集合；无可用路径先具名阻塞；V-01..V-04按对应入口，AC-01..04 | [x] 2026-11-08：991归档hash逐位一致（§4八输入SHA全对）；17 corpus行token/expected对账（R01/B08/B11与scenario同名行分开）；ace-certify CLI（--corpus/--certs/--only/--force/--rustc/--scratch）与ps1三scope实参冻结；树`D:/autostack/.wt/lang-992/auto-lang`@`36b5fbd4a`创建、scope登记；probe suite经生产a2r adapter发射17例实际（12 H-01实际==树金样逐字节MATCH、5 H-02提案SHA与991 §7.1 P2逐位相同：b08`5e63982b…`/b11`9503344a…`/r01`b535213e…`/r03`1586b561…`/r04`5ace7a57…`）；scope副本corpus（binding只改R02/B11→ok-emit+B11金样=提案）上`ace-certify --only r02,b11`双certified exit 0（R02 record例+B11命名实参例有界全链可行） |
| T-02 | work | T-01 | 对17原source/参考与golden逐例G04语义审阅，当前actual生成proposal；12旧golden保持，5新proposal审核后冻结；V-01..V-04按对应入口，AC-01..04 | [x] 2026-11-08：17例逐源G04语义审阅全部忠实（T05 NUL字节比较/R02嵌套record变异/R05 move传参+`as i64`长度/R06 RHS先物化/V01-V07 Option/Result/enum分支/K01 move闭包/K03借用闭包/B08词法遮蔽/B11 914命名实参闭包形`(\|a0,a1\| sub(a1,a0))(3,10)`=7/R01 `.clone()`值复制/R03字段名绑定序/R04深clone String）；12旧金样字节未动（probe MATCH）；5新金样`oracle/m1_{b08,b11,r01,r03,r04}.rs`冻结入库（SHA同上） |
| T-03 | work | T-02；H注册单写窗 | 仅目标17a2r token/说明及相应manifest身份更新，新增5例a2r-only扩展suite/固定组，适配snapshot完整读取；不扩大certifier谓词；V-01..V-04按对应入口，AC-01..04 | [x] 2026-11-08：binding.tsv仅17行f[8]→`ok-emit`+PLAN-992注（131行/12列/313 required/批准集不变，17格无applicability claim对账NONE）；新增`legacy-a2r-cert-992.case.ad`+`.suite.ad`（仅5例a2r-snapshot，corpus域id，原12检查留legacy-composite不复制）；固定组`case-legacy-a2r-cert-992`入test-groups.toml（--select-only解析+实跑PASS）；verify-ace-m1-852.ps1 snapshot改双套件插桩运行并集守卫`-lt 31`；manifest：binding sha`a86365c8…`/oracle_golden 24→29/suites_cases 81→83（certs轴10→27在T-04）；certifier谓词`ace_certify.rs:111`未动 |
| T-04 | work | T-03 | 17值例原域显式认证+旧10strict复用，坏件/漂移/漏check/零spawn与半证书反控闭合，验38→21/10→27/26→31；V-01..V-04按对应入口，AC-01..04 | [x] 2026-11-08：`ace-certify --only <17>`17 certified exit 0；全量复跑27 reused/0 failed exit 0（旧10 strict复用不重认证）；manifest certs 10→27；snapshot scope（绝对RepoRoot）：31 a2r检查全绿（26 base+5扩展）、27证书strict有效、base 97+扩展5全绿、认证面spawn=0；receipts scope（-BuildIdentity直缝）27有效；acceptance：cert-gap 38→**21**、can_claim_m1=false、313/45/32不变；contract PASS。反控全红：C1金样漂移1B→FAIL且证书hash不变；C2坏cert未知字段→REJECTED（--force也不覆盖）；C3登记expected改→证书REJECTED desync、C3b删cert后fresh认证→run mismatch FAIL且无半证书；C4扩展套件删检查→snapshot红`29<31`。G02观测器修复：冷构建churn下rustc经`sccache.exe`/rustc-under-rustc/`--print`身份探针被误判认证面——分类改为工具链祖先链识别（cargo/sccache/rustc祖先=维护、纯`--print`无编译轴=身份探针、直连rustc/m1_*仍违规），regex单元判定+churn实跑+终跑0违规验证 |
| T-05 | work | T-04 | 完整职责/原旧域相邻回归、Spec候选与逐格941只读交接，剩余21清单与不签M1明确；V-01..V-04按对应入口，AC-01..04 | [x] 2026-11-08：职责组并集全绿——`cargo t case-legacy-a2r-cert-992`/`case-ace-m1-base`/`infra-ace-m1`（终版脚本复跑）/`auto-test-case`/`ace-adapters`全PASS；SD-01/02候选维持§5表（review验证、merge沉淀）；941只读交接表（§7.3：格位/源/金样/证书SHA16×17）；剩余21清单冻结（§7.3：H-03 T01-T04/H-04 L01-L05,L07/H-05 K06/H-06 B13,B14,B15,B20/H-07 scn R01-R06），M1不签（can_claim_m1=false）；worktree提交`ec92e5ff4` |
| T-06 | review | T-01..05完成；冻结后停实现写入 | V-05独立逐AC/SD/全部原例反控/reader/健康审查，绑定r1/冻结commit；AC-05 | [x] 2026-11-08：独立subagent上下文终审**pass**（AC-01..05全pass；7例独立G04重审含B11闭包形/R01克隆/T05 NUL；27证书strict+轴hash重算对盘；gap 21格清单逐格核对；17行binding/manifest 83+29+27全hash对盘；自带金样漂移反控red；G02豁免判据regex五例评估不可掩盖认证面；发现仅3条info级，见§9） |
| T-07 | merge | T-06 pass与后续合入指令 | 冻结明确职责并集/有效证据复用与触码漂移核验，SD实际沉淀读回/来源交付/归档，逐格证据交接但不写941；AC-06 | [ ] |
| T-08 | merge | T-07来源与归档核实 | 全部命令输出根对账、证据摘要折本Plan后scope整删（具名持有另记），wt-guard clean后移除唯一树/分支/空组，登记刷新；AC-06 | [ ] |

## 9. 复审记录

2026-11-07 stage:new / PLAN-992 r1 / outcome:pass（仅规划静态合同，非实施/产品/认证通过） / next:后续实施指令及实际依赖就绪后T-01。T-01..08全部未执行，0/8，未建树/scope，未修改production/current Specs/语料/证书；规划不等golden批准或M1签章。静态复核收据在此续录。

2026-11-07 规划静态复核：0..10章节、T8/AC6/V5/SD2唯一连续且互引完整，work/review/merge任务均未执行；本包17格与当前binding的corpus域/原token/outcome/expected完全对账，src与独立reference存在，三包22格不重叠，余16不在本次范围。当前Spec路径、计划交叉链接、表格列数/空白/编码、现有固定组名与拟新增组标记、唯一编号/中央next-id=995通过。第一轮文档检查器将固定all入口误当未知组而退出，纠正检查器识别后完整静态复核通过；未修改产品入口或为通过放宽产品门。939/941规划前后内容hash一致；本轮仅写992/993/994与取号.next-id，未施工、未运行产品/认证/性能/独立实施review、未建树/scope/会话。

2026-11-08 stage: work | plan_id: PLAN-992 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-992/auto-lang | branch: plan-992-dev | base_commit: 36b5fbd4a | code_commit: ec92e5ff4 | dependency_revisions: D991已核（归档hash一致）；主检出仅他组docs WIP，无代码WIP | task_ids: T-01..T-05 | evidence: §7.3（17 certified+10 reused=27 strict、snapshot 31/97+5/0 spawn、receipts 27、acceptance gap21/can_claim_m1 false/313、contract PASS、5固定组PASS、C1-C4反控全红、G02观测器修复+判据保留） | blockers: 无 | 未执行: 全仓统一回归（owner=merge按冻结并集；周期cargo t all按due由原owner） | next_responsible_role: review（T-06，同树独立上下文，冻结commit ec92e5ff4 + 本Plan文件）

stage: goal | plan_id: PLAN-992 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-992/auto-lang | commit: ec92e5ff4 | next: review（T-06）

2026-11-08 stage: review | review_scope: final | reviewer_identity/limitations: 独立 subagent 上下文（general-purpose，未参与work），同一冻结树只读，材料仅plan-992/review/；未重放work全部C1-C4（以自带独立金样漂移反控替代）、未跑未变更全量组 | plan_id: PLAN-992 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-992/auto-lang | branch: plan-992-dev | reviewed_commit: ec92e5ff4（树clean，diff inventory恰为4改+24增：binding/manifest/groups/ps1改，5金样+17证书+case/suite增；oracle既有m1_*.rs零修改；crates/源码零改动，emitter/a2r.rs/ace_certify.rs谓词未动）| base_commit: 36b5fbd4a | dependency_revisions: D991归档核验；簿记d8ba5e0ad仅本Plan文件 | spec_inputs: SD-01/02目标规范存在于master且当前文本（"认证集=标量10例/伪Rust不可编译"、"38格+certs仅10"）恰为本次reviewed交付（27证书/gap21/旧producer域/零gap守卫保留）所取代；merge按此沉淀 | acceptance_results: AC-01 pass（7例独立G04：B11闭包形=7/R01 clone=19/T05 NUL=42/V04=42/B08遮蔽=7/R03字段名=23/R04深clone=51；12旧金样字节==master）；AC-02 pass（--only b11,t05→2 reused exit0；27证书schema/oracle文本/双侧run/轴hash抽样+全量stdout核对）；AC-03 pass（receipts 27 strict；acceptance gap恰21=H-03×4+H-04×6+H-05×1+H-06×4+scn×6、can_claim_m1 False、313/32/45；contract PASS 131行；新组PASS；reviewer自带1B金样漂移反控red且同跑30+97检查绿/0违规）；AC-04 pass（17行token恰为目标集、manifest 81+29+27+83全hash对盘、§7.3交接表17行SHA16全对、binding sha=a86365c8…）；AC-05=本审 | pending_closeout_gates: merge冻结并集（已跑组有效复用+漂移核验）、SD-01/02实际沉淀读回、scope整删+wt-guard清树 | findings: F-992-R1-01（info）Get-AncestorNames祖先链未在runner PID截断——若snapshot自cargo树内启动（现无该入口）认证面rustc可被误分类maintenance的潜伏面，建议后续硬化：走到runner即停；F-992-R1-02（info）manifest末行无换行符（cosmetic，contract解析正常）；F-992-R1-03（info）m1_b08.rs金样`}    }`排版怪（字节冻结/可编译/已认证，无需动作） | evidence: 本节+plan-992/review/（ctl反控等，merge整删） | next: merge（T-07）

## 10. 待澄清事项

1. 991的实测用于选定反例与范围，不等当前本包正式认证；T-01重核实际源/工具与现入口，缺证据不拿旧PASS补。
2. G04逐例审阅在任务内完成，非自动接受actual；语义无法证明/出现超出本包的producer问题，保具名阻塞并修订，不能换expected。原cert schema/std-only与资源帽不放宽。
3. H-04/H-06/H-07仍需原用户裁定/正式后继；本轮不预填NA/accepted、不改零gap守卫，三包全闭仍余16格。
4. 本轮只有规划授权，未来work/review/merge按后续指令；不为等价内部实现重复审批，不将其扩为新语言/新认证平台或修改939/941的授权。
