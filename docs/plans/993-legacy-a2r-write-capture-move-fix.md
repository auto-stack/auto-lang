---
plan_id: PLAN-993
status: executing
feature_name: 旧A2R K06写捕获闭包move发射修复与认证
author: [Codex]
created_at: 2026-11-07
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-lang/trans/architecture.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991, PLAN-992]
current_step: 0
total_steps: 8
---

# [PLAN-993] 旧A2R K06写捕获闭包move发射修复与认证

## 0. 变更摘要

认领991 H-05：修复写捕获Rc/RefCell闭包与显式move叠加发射两个move的缺陷，保共享可变捕获语义，重审K06金样并完成原域认证。

本轮仅正式规划，drafting r1/0-of-8；全部实现/认证/复审均未执行。按991分诊证据提出的候选不是本包当前PASS。本包不修改执行中的939/941，不启动其他会话，不改变M1范围/原source/oracle或预算。

## 1. 目标

关闭本包§5逐格清单的原旧A2R值例认证义务；完整actual、独立reference、已审golden、strict receipt、零spawn日常及来源登记全部可核，交941原owner独立接纳。

非目标：H-04集合、H-06宽度/trap、H-07场景裁定；补任意新语言/新HIR后端、重写所有旧A2R、放宽receipt/预算/NA/required、修939/941代码或计划、隐式认证/自动改golden。

## 2. 架构方案

继承991分诊且正式依赖为PLAN-991, PLAN-992；全链统一走旧source→production A2R→已审golden/完整actual→原显式认证→原daily/strict核查→逐格交接。未来实现确认实际基面/工具/SHA，WIP/reviewed不是D；无直接依赖939/941整包交付，不借其未提交代码。

## 3. 技术栈

- `crates/auto-lang-core/src/trans/rust/expr_emit.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang-core/src/trans/rust/stmt_emit.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang/src/tests/a2r_tests.rs`（现存；T-01重核实际符号）。
- 共同入口：`crates/auto-lang/src/bin/ace-certify.rs`、`crates/auto-lang-core/src/test_cases/ace_certify.rs`、`crates/auto-lang-core/src/test_cases/a2r.rs`。
- 真实语料根：`crates/auto-lang/test/cases/ace-m1/`（binding/applicability/source-manifest/src/oracle/certs/reference与原Case）。

## 4. 需求分析与背景调查

用户2026-11-07授权依据991正式规划17格认证、H-05闭包与H-03文本后继，明确本轮只做规划，不施工、不改正在执行的939/941、不启动其他会话。只允许本组三份Plan及中央取号簿记；未授权golden实际变更/认证执行/合入。无用户指定模型、工期或token预算。

**实施授权（2026-11-08）**：用户以 `/goal 完成计划993` 授权全链交付（work→review→bounded repair→merge，auto-plan-goal 范式）。基面核验：D991 已归档、D992 已归档交付（delivery `64f93cb99`，cert10→27/gap38→21/snapshot26→31，见 `c43e90116`）。唯一树 `D:/autostack/.wt/lang-993/auto-lang` 分支 `plan-993-dev`，base=master `eef2f3f21`（含 D992）；scope `D:/autostack/plan-artifacts/plan-993/` 已登记。主检出仅有他会在途 docs/* 改动，无代码路径 WIP。

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
| K06 | `snapshot-only-cert-gap` | `value / 42` | 修复发射、重审K06原golden、认证 |
### 已定位机制与修复边界

现`expr_emit.rs`的`Expr::Closure`先在captured非空时发`move `，随后closure.is_move再次发`move `；K06真实生成`move move`而非源语法问题。改动应基于捕获计划/标志只在需要时恰发一次，保每个写捕获的Rc::clone与borrow/borrow_mut逻辑，不能在生成字节上全局替换字符串、删除源move或拿新HIR closure代跑。

T-01原K06 source先重现rustc语法失败；合法K01/K03各42。T-02补四类可区分组合：写捕获+显式move、写捕获无显式move、只读/按值显式move、无捕获非move；再覆盖多个写捕获、嵌套与跨两次调用的共享cell/别名观察。不能将每次调用独立clone值/改变capture owner来过42；各组合由原Auto合法语义手算独立结果，unsupported不扩域。

K06源两次f调用依次21/21，返回42；语义审阅同时看实际cell更新、真实捕获清理/近邻，不能只grep没有move move。T-03仅原`oracle/m1_k06.rs`按G04重冻结及corpus:K06:a2r→ok-emit，原legacy-composite Case已有snapshot，不新增重复Case。reference K06不变；golden差异仅修复产生的合法字节，不顺手fmt其余golden。

正式基面需D992：证书27→28，gap21→20，snapshot31仍31（K06原已在26中）。10原证书与992的17证书按当前轴strict验证/复用，不为emitter身份审计变化一律重编；实际文件/工具轴变化才STALE。T-01核真实集合，不用写死总数代逐格证明。若其他相同capture原golden也确受修复影响，先给来源/反控与G04清单；超出本包K06关闭范围的语义变化须修订，不静默改旧批准。

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
| SD-03 | modify | `docs/specs/auto-lang/trans/architecture.md` | 写捕获与显式move组合恰一次发射、共享cell语义与真实证据 | 只沉淀reviewed实际行为；本轮不写Spec | AC-01..04 |

## 6. 测试设计

所有V为拟执行；T-01使能真实反例/合法近邻，T-02/03接永久用例，T-04/05完整关闭，最终T-06独立复审。

| V | AC / invariant | 输入与实际入口/owner | 区分性oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02/03 双move根因与K06实际行为 | T-01/02，原K06、K01/K03及四类capture组合，经原生产发射+真实rustc/运行 | baseline原K06编译红、修复后原42；近邻保持正确，重复两次调用共享状态可区分 | 不重做ownership/闭包平台，不修H-04集合 |
| V-02 | AC-01/02/03 金样/认证守卫 | T-03/04，K06原snapshot+新cert、破坏move/Rc行为/金样变异 | 修复后G04审阅字节与actual相同；改回双move编译红、改成值副本被独立状态oracle抓红 | 原wrapper/schema/expected不改，K01/K03已认证仍strict绿 |
| V-03 | AC-01/02/03 父登记和余账 | T-04/05，D992基面已有27cert/31snapshot与当前acceptance | K06确实认证且gap仅21→20；原31检查都在、旧H03/H04/H06/H07不被消去 | 向941交本格证据，不改其reader/Plan |
| V-04 | AC-04 余账/范围/身份 | T-05原binding/required/manifest与本包每行；所有旧域及941交接 | 只本包格位转已认证；313/批准集合不改、旧首次红保留；缺证据不填0/PASS，剩gap如实且claim false | 不操作939/941，不由局部判据替全M1 |
| V-05 | AC-05 独立复审 | T-06同树独立上下文，原src→producer→golden/actual→reference/cert→reader逐项 | 重建G04与全部反控、SD/遗漏/警告/格式；假关闭/未知必需面needs_fix | 实现内部自查不顶最终review，不另开会话 |

### 共同反控与验证入口

T-01冻结可用CLI/固定组/工具及完整原Case集合，未来显式例：预构建`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only <本包逐例列表> --scratch <scope>/cert-scratch`，使用原证书路径；不同shell的最终命令在work记录实际绝对路径。原`powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope snapshot|receipts|acceptance -RepoRoot <tree> ...`中三scope分开运行，不把管道字符当参数；snapshot给scope输出，receipts提供当前BuildIdentity。具体CLI参数T-01以真实帮助/源码冻结，本轮均未执行。

独立reference是已有手写`reference/<ID>.rs`；两侧各对原stdout `i32:<expected>\n`及成功终态，而非相互相等。变异在scope副本，不触正在执行计划：golden变1B/expected错/receipt错source或跨新HIR域/截断/旧schema/工具身份漂移/缺Case或缺cert；strict必须拒或STALE。单例重新认证/编译失败不得留下可授信半证书或覆坏证书；恢复合法副本后可重试。缺工具/超时/输出溢出保持infra/resource，不充语言负例。

实现→内部自查→最终独立上下文review同一唯一树顺序进行，不自动创建agent/chat；内部PASS不替最终review。冻结职责并集在最终审查后按真实触面运行一次，未变身份的有效证据逐项说明复用，不为文档/提交重复全experimental/all。周期`cargo t all`只按due在主检出单实例，由原owner排队，不成为本包全局门。

职责基础为现有`cargo t infra-ace-m1 auto-test-case ace-adapters case-ace-m1-base a2r`；按实际修改冻结必要并集，原ignore/平台/重内存条件保留。生产emitter变更必须完整`a2r`职责及上述必要消费者，不能只跑新例或grep发射文本。若只改计划文档仅做静态检查，不运行这些产品门。拟新case组先注册后实际选中，不能写了Case文件即算已执行。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 本包全部原source与独立reference结果正确，所需production修复/语义重审、合法与非法近邻符合§5/§6，不以typed/新HIR替旧域 |
| AC-02 | work | 全部目标golden有逐例G04审阅与SHA，actual现场完整生成且字节一致，原schema真实双侧编译运行与strict receipt全部通过 |
| AC-03 | work | 原base与本包已审扩展日常检查完整、零认证面spawn，坏golden/证书/身份/漏case与合法复用反控正确；相邻职责与原条件通过 |
| AC-04 | work | 每格关闭证据与原分母/批准/余账守恒，全部reader/manifest/SD候选与941只读交接清楚，未裁定格和全M1仍不可冒绿 |
| AC-05 | review | 同树独立上下文逐AC/V/SD/source/golden审阅/实际命令/reader/遗漏健康核验，结论绑定r1/冻结源码与配置 |
| AC-06 | merge | review pass后冻结适用并集/漂移核验、SD沉淀读回、来源交付/归档，全输出路径核对、scope整删与guard清树；不改939/941计划 |

## 8. 执行步骤

预定唯一树`D:/autostack/.wt/lang-993/auto-lang`、分支`plan-993-dev`；本轮未创建。后续实施授权后先核唯一树/仓/分支/base/HEAD及实际D；材料只在`D:/autostack/plan-artifacts/plan-993/`，开工登记owner/用途/在途；无junction、跨树可写target或第二输出根。常规target在专树，外置target只能在本scope。原始证据merge折计划摘要后整删，独立review同树停写。

| T | owner_stage | 前置/写面 | 具体结果/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；实际D991/D992 | 冻结当前Expr::Closure/capture路径、K06首次红/K01K03合法值、四类capture全链反例与资源边界；V-01..V-04按对应入口，AC-01..04 | [ ] |
| T-02 | work | T-01 | 最小按机制修重复move，保Rc/RefCell/alias语义，新增永久K06/捕获组合实际编译运行反例；不改parser或新HIR；V-01..V-04按对应入口，AC-01..04 | [ ] |
| T-03 | work | T-02 | 审阅K06 source/actual/既有reference及新golden语义，记录G04；仅目标token/manifest/字节更新并核所有登记读者；V-01..V-04按对应入口，AC-01..04 | [ ] |
| T-04 | work | T-03 | 原域K06显式认证、旧27strict复用、原日常31检查零spawn与receipt/变异反控，确认21→20；V-01..V-04按对应入口，AC-01..04 | [ ] |
| T-05 | work | T-04 | 完整a2r职责与相邻旧Case/治理通过，SD候选/具名余账/逐格941只读交接，冻结实现交review；V-01..V-04按对应入口，AC-01..04 | [ ] |
| T-06 | review | T-01..05完成；冻结后停实现写入 | V-05独立逐AC/SD/全部原例反控/reader/健康审查，绑定r1/冻结commit；AC-05 | [ ] |
| T-07 | merge | T-06 pass与后续合入指令 | 冻结明确职责并集/有效证据复用与触码漂移核验，SD实际沉淀读回/来源交付/归档，逐格证据交接但不写941；AC-06 | [ ] |
| T-08 | merge | T-07来源与归档核实 | 全部命令输出根对账、证据摘要折本Plan后scope整删（具名持有另记），wt-guard clean后移除唯一树/分支/空组，登记刷新；AC-06 | [ ] |

## 9. 复审记录

2026-11-07 stage:new / PLAN-993 r1 / outcome:pass（仅规划静态合同，非实施/产品/认证通过） / next:后续实施指令及实际依赖就绪后T-01。T-01..08全部未执行，0/8，未建树/scope，未修改production/current Specs/语料/证书；规划不等golden批准或M1签章。静态复核收据在此续录。

2026-11-07 规划静态复核：0..10章节、T8/AC6/V5/SD3唯一连续且互引完整，work/review/merge任务均未执行；本包1格与当前binding的corpus域/原token/outcome/expected完全对账，src与独立reference存在，三包22格不重叠，余16不在本次范围。当前Spec路径、计划交叉链接、表格列数/空白/编码、现有固定组名与拟新增组标记、唯一编号/中央next-id=995通过。第一轮文档检查器将固定all入口误当未知组而退出，纠正检查器识别后完整静态复核通过；未修改产品入口或为通过放宽产品门。939/941规划前后内容hash一致；本轮仅写992/993/994与取号.next-id，未施工、未运行产品/认证/性能/独立实施review、未建树/scope/会话。

## 10. 待澄清事项

1. 991的实测用于选定反例与范围，不等当前本包正式认证；T-01重核实际源/工具与现入口，缺证据不拿旧PASS补。
2. G04逐例审阅在任务内完成，非自动接受actual；语义无法证明/出现超出本包的producer问题，保具名阻塞并修订，不能换expected。原cert schema/std-only与资源帽不放宽。
3. H-04/H-06/H-07仍需原用户裁定/正式后继；本轮不预填NA/accepted、不改零gap守卫，三包全闭仍余16格。
4. 本轮只有规划授权，未来work/review/merge按后续指令；不为等价内部实现重复审批，不将其扩为新语言/新认证平台或修改939/941的授权。
