---
plan_id: PLAN-994
status: reviewed
feature_name: 旧A2R四个文本值例lowering修复与独立认证
author: [Codex]
created_at: 2026-11-07
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-lang/trans/legacy-rust-mut-and-text-evidence.md, docs/specs/auto-lang/trans/design/a2r-string-ownership.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991, PLAN-992]
current_step: 5
total_steps: 8
---

# [PLAN-994] 旧A2R四个文本值例lowering修复与独立认证

## 0. 变更摘要

认领991 H-03：修复原T01..T04限定source路径的字符数/字节数、字符位两参slice、codepoint发射，使用std-only完整产物完成四格认证；不全局重定义旧len或更换expected。

2026-11-08 `/goal 完成计划994`为实施指令（一次授权覆盖work/review/bounded repair/merge）；work阶段（T-01..T-05）已完成：声明型字符串局部len=字符数/len_bytes=UTF8字节/两参slice=917字符位守卫块恢复/char to_codepoint=标量值；四格certified（cert 28→32）、snapshot日常26+5+4=35、cert-gap 20→16、can_claim_m1=false不变；917三红（D-993-2）随守卫块恢复转绿；详见§7.3/§9。本包未修改939/941，M1范围/原source/oracle/预算不变。

## 1. 目标

关闭本包§5逐格清单的原旧A2R值例认证义务；完整actual、独立reference、已审golden、strict receipt、零spawn日常及来源登记全部可核，交941原owner独立接纳。

非目标：H-04集合、H-06宽度/trap、H-07场景裁定；补任意新语言/新HIR后端、重写所有旧A2R、放宽receipt/预算/NA/required、修939/941代码或计划、隐式认证/自动改golden。

## 2. 架构方案

继承991分诊且正式依赖为PLAN-991, PLAN-992；全链统一走旧source→production A2R→已审golden/完整actual→原显式认证→原daily/strict核查→逐格交接。未来实现确认实际基面/工具/SHA，WIP/reviewed不是D；无直接依赖939/941整包交付，不借其未提交代码。

## 3. 技术栈

- `crates/auto-lang-core/src/trans/rust/call_emit/call_dot_methods.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang-core/src/trans/rust/expr_emit.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang-core/src/trans/rust/call_emit.rs`（现存；T-01重核实际符号）。
- `crates/auto-lang/src/tests/a2r_tests.rs`（现存；T-01重核实际符号）。
- `crates/a2r-std/src/str.rs`（现存；T-01重核实际符号）。
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
| T01 | `emit-ok-run-unverified` | `value / 38` | 新golden审阅/冻结、认证 |
| T02 | `emit-ok-run-unverified` | `value / 27` | 新golden审阅/冻结、认证 |
| T03 | `emit-ok-run-unverified` | `value / 23` | 新golden审阅/冻结、认证 |
| T04 | `emit-ok-run-unverified` | `value / 128512` | 新golden审阅/冻结、认证 |
### 当前源码、规范差额与先决决定

991首次反证：T01生成不存在的String::len_bytes；T02引入a2r_std依赖（不满足cert std-only）并缺len_bytes；T03实际25而原oracle23；T04生成不存在的char::to_codepoint。当前发射位置已拆分：`call_dot_methods.rs`处理slice/len/generic fallback，`expr_emit.rs::emit_str_slice_bytes`仍设置a2r_std_used并调用`a2r_std::str::str_slice`。后者是runtime字节域；而917 current Spec对source两参slice承诺字符位、界外panic、接收者/两界各一次。本包T-01必须保留该Spec-vs-code差额与实际917入口回归，不把991旧路径描述当唯一根因。

目标只覆盖T01..T04已冻结调用域：T01("A中😀")=3 scalar/8 bytes→38；T02按字符位slice(1,3)取"中😀"=2 scalar/7 bytes→27；T03原a/clone/move/追加中文保持独立owner，len组合→23；T04补充平面char U+1F600→128512。公开Str.len/动态len/length/显式byte_len的其他已批准差异不在本包统一；外部builder `.len(n)`、List/Map长度、foreign/raw Rust同名方法和用户record方法都不能被名称匹配改写。

T-01先证明receiver分类真实来自现类型/语境，冻结typed-known字符串/char与非字符串近邻的选择规则；不得按测试ID、字面量内容或source正则特判。`len_bytes`按UTF8字节，适用的len按四源与对应现规范字符数，char codepoint按标量值；所有复杂receiver/索引恰求值一次，保源序与owned/view不悬空。合法空串、嵌NUL、组合字符、emoji、字符串参数/字段与非字符串同名方法各有近邻。

source两参slice的std-only合法化必须遵守917字符位/界检查/原panic观察且双界/receiver一次；不要改变`a2r_std::str_slice`公共字节API或全量helper、不要将通用foreign调用len默认变Unicode。计划默认仅对原source字符串受检/已知路径发自包含std表达式或私有helper；T-01比较等价策略后冻结，若必须改deps/新schema或宽语言规则则needs_replan，不默扩`ace.m1.cert/1`。原单参slice与所有相关已支持行为回归保留，无法兼容时先具名修订，不凭截断/全拒掩盖。

T-03经G04审阅后新增`oracle/m1_t01.rs`..`m1_t04.rs`，新四份cert和显式旧a2r-only扩展Case（拟`legacy-a2r-text-994.case.ad`/`.suite.ad`，新固定组`case-legacy-a2r-text-994`，T-03注册并接原snapshot日常的已审扩展列表）。原T01..04源/参考/expected不改，T05=42与917 R07/T08域（尤其边界panic不是Auto trap cert）必须保留。不得把修好值例用于核销H-06 trap或H-07场景格。

依赖D992的认证/日常扩展接缝；推荐D993后正式写入，以串行核登记/同emitter消费者；**993不是语义前置**。若994先于993交付，按实际合法闭集计算：D992后四格可使gap21→17/cert27→31；按推荐顺序则20→16/cert28→32。snapshot31→35（加原四文本）；不能为符合20→16数字先将K06算绿。三包齐后的总目标38→16、cert10→32，剩余H04六+H06四+H07六，不改三类未裁定义务。

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
| SD-03 | modify | `docs/specs/auto-lang/trans/legacy-rust-mut-and-text-evidence.md` | 限定source文本字符/字节/切片/单求值与runtime API边界和原917观察 | 只沉淀reviewed实际行为；本轮不写Spec | AC-01..04 |
| SD-04 | modify | `docs/specs/auto-lang/trans/design/a2r-string-ownership.md` | 限定source文本字符/字节/切片/单求值与runtime API边界和原917观察 | 只沉淀reviewed实际行为；本轮不写Spec | AC-01..04 |

## 6. 测试设计

所有V为拟执行；T-01使能真实反例/合法近邻，T-02/03接永久用例，T-04/05完整关闭，最终T-06独立复审。

| V | AC / invariant | 输入与实际入口/owner | 区分性oracle | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02/03 四原源值与首次红 | T-01/02，原T01/02/03/04 production actual及独立reference各编译运行 | 旧E0599/依赖错/25≠23保留；修后逐例38/27/23/128512，两侧分别对oracle | 不以四端相等/只编过替语义 |
| V-02 | AC-01/02/03 Unicode/选择/所有权兄弟 | T-02，空串/NUL/组合字符/emoji/参数字段、clone-move和非字符串同名方法 | 字符数/UTF8字节区分，源序单求值，原owner有效；List/Map和builder len(n)保持语义 | 仅原支持输入；其他len差异不归一 |
| V-03 | AC-01/02/03 slice与917边界 | T-01/02/05，source字符slice(1,3)、越界原T08、合法近邻及原单参slice | T02完整std-only实际27；917原panic/字节域显式runtime API不变，两界receiver各一次 | panic仅旧917观察，不扩trap cert；typed helper不代source |
| V-04 | AC-01/02/03 四证书与日常全闭 | T-03/04，新四golden经G04、四cert、扩展日常，与旧已认证集 | actual/ref各oracle、strict收据/漂移/零spawn和漏新check红；真实gap降低4，未裁H04/06/07不变 | 依实际D993与否计算计数，不写死成功 |
| V-05 | AC-04 余账/范围/身份 | T-05原binding/required/manifest与本包每行；所有旧域及941交接 | 只本包格位转已认证；313/批准集合不改、旧首次红保留；缺证据不填0/PASS，剩gap如实且claim false | 不操作939/941，不由局部判据替全M1 |
| V-06 | AC-05 独立复审 | T-06同树独立上下文，原src→producer→golden/actual→reference/cert→reader逐项 | 重建G04与全部反控、SD/遗漏/警告/格式；假关闭/未知必需面needs_fix | 实现内部自查不顶最终review，不另开会话 |

### 共同反控与验证入口

T-01冻结可用CLI/固定组/工具及完整原Case集合，未来显式例：预构建`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only <本包逐例列表> --scratch <scope>/cert-scratch`，使用原证书路径；不同shell的最终命令在work记录实际绝对路径。原`powershell -NoProfile -File scripts/verify-ace-m1-852.ps1 -Scope snapshot|receipts|acceptance -RepoRoot <tree> ...`中三scope分开运行，不把管道字符当参数；snapshot给scope输出，receipts提供当前BuildIdentity。具体CLI参数T-01以真实帮助/源码冻结，本轮均未执行。

独立reference是已有手写`reference/<ID>.rs`；两侧各对原stdout `i32:<expected>\n`及成功终态，而非相互相等。变异在scope副本，不触正在执行计划：golden变1B/expected错/receipt错source或跨新HIR域/截断/旧schema/工具身份漂移/缺Case或缺cert；strict必须拒或STALE。单例重新认证/编译失败不得留下可授信半证书或覆坏证书；恢复合法副本后可重试。缺工具/超时/输出溢出保持infra/resource，不充语言负例。

实现→内部自查→最终独立上下文review同一唯一树顺序进行，不自动创建agent/chat；内部PASS不替最终review。冻结职责并集在最终审查后按真实触面运行一次，未变身份的有效证据逐项说明复用，不为文档/提交重复全experimental/all。周期`cargo t all`只按due在主检出单实例，由原owner排队，不成为本包全局门。

职责基础为现有`cargo t infra-ace-m1 auto-test-case ace-adapters case-ace-m1-base a2r case-ace-m1-a2r-emit-917`；按实际修改冻结必要并集，原ignore/平台/重内存条件保留。生产emitter变更必须完整`a2r`职责及上述必要消费者，不能只跑新例或grep发射文本。若只改计划文档仅做静态检查，不运行这些产品门。拟新case组先注册后实际选中，不能写了Case文件即算已执行。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 本包全部原source与独立reference结果正确，所需production修复/语义重审、合法与非法近邻符合§5/§6，不以typed/新HIR替旧域 |
| AC-02 | work | 全部目标golden有逐例G04审阅与SHA，actual现场完整生成且字节一致，原schema真实双侧编译运行与strict receipt全部通过 |
| AC-03 | work | 原base与本包已审扩展日常检查完整、零认证面spawn，坏golden/证书/身份/漏case与合法复用反控正确；相邻职责与原条件通过 |
| AC-04 | work | 每格关闭证据与原分母/批准/余账守恒，全部reader/manifest/SD候选与941只读交接清楚，未裁定格和全M1仍不可冒绿 |
| AC-05 | review | 同树独立上下文逐AC/V/SD/source/golden审阅/实际命令/reader/遗漏健康核验，结论绑定r1/冻结源码与配置 |
| AC-06 | merge | review pass后冻结适用并集/漂移核验、SD沉淀读回、来源交付/归档，全输出路径核对、scope整删与guard清树；不改939/941计划 |

### 7.3 work阶段验证记录（2026-11-08，树`D:/autostack/.wt/lang-994/auto-lang`，base`8514d2f87`，code`adddf8d06`）

- **入口冻结**：`cargo build -p auto-lang --bin auto-case --bin ace-certify` exit 0；rustc 1.98.0 (88d9e12ae 2026-08-18) PATH身份=32证书`default_identity_at_certify`一致。
- **首次红重放**（scope `t01-freeze/first-red/`，wrap.rs现场编译运行）：T01 E0599`len_bytes`、T02 E0432`use a2r_std`、T03 run `i32:25`≠23、T04 E0599`to_codepoint`——与991 §5.4.1逐字一致。
- **修复**（emit路径全冻结于`plan-artifacts/plan-994/t01-freeze/FREEZE.md`）：`str_typed_stores`/`char_typed_stores`声明型集合（store()注册、fn_decl清空、重声明改型即除名）；`len`/`len_bytes`/`to_codepoint`三零参臂；`emit_str_slice_chars`恢复917守卫块（两参）+chars-skip（单参），sub/slice/Bina镜像共4调用点同步，不再设a2r_std_used。
- **四值全链**：actual编译运行`i32:38/27/23/128512`全对oracle；reference独立同值；T08现场运行panic"auto str slice out of range"（917冻结trap回归）。
- **近邻字节稳定**：字段`(b.name.len() as i64)`/模式绑定`(e.len() as i64)`/参数`(s.len() as i64)`/List计数/T06调用结果——修后发射与base逐字节一致；K04在解析期即拒（auto_syntax_E0001）。值近邻：空串0、NUL串33、emoji 14、ASCII codepoint 65、单参slice 27、sub(1,3) 27、越界panic。
- **认证**：`ace-certify --only t01,t02,t03,t04` → 4 certified exit 0；全量复跑 32 reused/0 failed exit 0（旧28 strict复用）。
- **三scope+contract**：snapshot=**35** a2r全绿（26 base+5 ext-992+4 ext-994）+32证书strict+认证面spawn=0；receipts（-BuildIdentity）32有效；acceptance=cert-gap **16**/can_claim_m1 **false**/313/45/32不变；contract=131行+manifest全验PASS。
- **固定组**：`cargo t a2r`（405例含plan994单元8例）、`case-legacy-a2r-text-994`、`case-legacy-a2r-cert-992`、`case-ace-m1-base`、`case-ace-m1-a2r-emit-917`（**D-993-2三红转绿**）、`infra-ace-m1`、`auto-test-case`、`ace-adapters`、`case-ace-m1-a2r-semantics-916`全PASS；`case-ace-m1-shared-mutable-909`首跑红（`av-s02-closures-source-fresh`，`%TEMP%/av798-session-av_closures_909_s02`暖store，同993环境归因），parent清store后单独复跑PASS——非代码回归（本包零触909面）。
- **反控**（scope副本，树未触）：C1金样1B漂移→daily FAIL 1/4且树证书hash不变；C2证书未知字段→REJECTED（--force同拒）；C3登记expected 38→39→证书REJECTED desync；C3b删证书重认证→run mismatch FAIL且无半证书；C4扩展套件删t04检查→snapshot FAIL `34 < 35`；C5伪BuildIdentity→证书STALE。
- **具名修订**：3个cookbook冻结期望随声明型len投影更新（rand_passwd/hex/graphemes，ASCII值恒等、仅发射形变）——"所有相关已支持行为回归保留"的具名修订面，其余a2r全绿。
- **941只读交接**（原owner按真实身份接纳，本包不写941）：

| 格位 | source SHA16 | golden SHA16 | cert SHA16 | 终态 |
|---|---|---|---|---|
| corpus:T01:a2r | `d60380e49fee0af3` | `cd580bd003bea128` | `bf805815c5f52aa0` | certified |
| corpus:T02:a2r | `58deb24678579081` | `a1494c1b3e83af19` | `e6c345b50c45c1c4` | certified |
| corpus:T03:a2r | `f895f9d1577bbea6` | `2edff30cd95a9b65` | `d98f313820fc6ff0` | certified |
| corpus:T04:a2r | `314fb56ca9ccbdae` | `f5f5c31c01ecf134` | `f2416fcfa8f7f354` | certified |

- **剩余16格**（acceptance冻结，本包不预填/不改守卫）：H-04 L01/L02/L03/L04/L05/L07（集合缺失）、H-06 B13/B14/B15/B20（trap/宽度裁定）、H-07 scenario R01-R06（场景面）。全M1仍不可冒绿：`can_claim_m1=false`（cert-gap 16>0 且 future 45 在账）。
- **健康**：新增代码零警告（既有警告不增）、触面文件rustfmt净（a2r_tests.rs既有8-18行漂移为master存量，不属本包）、零调试打印；worktree提交`adddf8d06`，树clean。
- **材料根**：唯一scope `D:/autostack/plan-artifacts/plan-994/`（t01-freeze/t01-out/t02-verify/t03-goldens/t04-certify/t04-scopes/t04-controls等，merge折叠后整删）；树内常规target正常，无第二输出根。

## 8. 执行步骤

预定唯一树`D:/autostack/.wt/lang-994/auto-lang`、分支`plan-994-dev`；本轮未创建。后续实施授权后先核唯一树/仓/分支/base/HEAD及实际D；材料只在`D:/autostack/plan-artifacts/plan-994/`，开工登记owner/用途/在途；无junction、跨树可写target或第二输出根。常规target在专树，外置target只能在本scope。原始证据merge折计划摘要后整删，独立review同树停写。

| T | owner_stage | 前置/写面 | 具体结果/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；D991/D992；993仅推荐写窗序 | 冻结原四source/参考/首次红与拆分发射路径、917 Spec差额、receiver识别/自包含策略及合法/非法近邻；不能凭日志推当前机制；V-01..V-05按对应入口，AC-01..04 | [x] 2026-11-08：D991/992/993均归档核验；首次红四例现场重放逐字同991（E0599/E0432/i32:25/E0599）；发射路径与956→917投影差额冻结（scope `t01-freeze/FREEZE.md`：956(764f454d1)将emit_str_slice_chars换a2r_std字节域未重冻917金样=三红根因）；分类规则冻结=声明型let/var文本局部（显式str/Str/str(N)/char），字段/模式绑定/参数/推断/调用结果保持字节remap（近邻基线全部字节稳定）；策略比较冻结=内联自包含std表达式（与917金样字节一致，私有helper会改T08金样否决） |
| T-02 | work | T-01 | 实现有限文本方法/字符slice/codepoint最小修复，std-only、源序单求值、clone/move与非文本方法保护；永久原例及兄弟真实编译运行；V-01..V-05按对应入口，AC-01..04 | [x] 2026-11-08：`str_typed_stores`/`char_typed_stores`+len/len_bytes/to_codepoint零参臂+emit_str_slice_chars恢复（4调用点）；四值38/27/23/128512双侧编译运行对oracle；T08 panic回归；永久单元plan994_*8例进a2r_tests.rs；值近邻（空/NUL/组合/emoji/单参slice/sub/bounds调用）全绿；近邻字节稳定（字段/模式/参数/List/调用结果）；K04解析拒绝确认 |
| T-03 | work | T-02 | 逐源G04审阅，冻结四新golden、四格token/manifest与a2r-only Case/日常扩展接线及拟新固定组；保原base不扩；V-01..V-05按对应入口，AC-01..04 | [x] 2026-11-08：逐源G04（求值序/绑定/资源/单求值/语义忠实）结论入§7.3；`oracle/m1_t01..t04.rs`冻结（SHA256全64位见manifest）；binding.tsv仅4行f[8]→ok-emit+PLAN-994注（131/313/批准集不变）；`legacy-a2r-text-994.case.ad`+`.suite.ad`（ID ace.m1.a2rt.t01..t04避让t-r-native同名）；固定组`case-legacy-a2r-text-994`注册并实跑PASS；verify-ace-m1-852.ps1 snapshot三套件插桩+`-lt 35`并集守卫；manifest：binding sha更新/golden 29→33/certs 28→32（T-04）/suites 83→85；certifier谓词未动 |
| T-04 | work | T-03 | 四例原域显式认证，已有证书strict复用，原+扩展35检查零spawn、缺件/坏轴/非法选择反控；按实际基面降低4gap；V-01..V-05按对应入口，AC-01..04 | [x] 2026-11-08：`ace-certify --only t01,t02,t03,t04`→4 certified exit 0；全量32 reused/0 failed；snapshot=35全绿+32 strict+spawn 0（含base+ext-992+ext-994三套件插桩）；receipts 32有效；acceptance cert-gap 20→**16**、can_claim_m1=false、313/45/32不变；contract PASS；反控C1-C5全红（金样漂移/未知字段--force拒/登记desync/删证run mismatch无半证书/删检查34<35/伪身份STALE） |
| T-05 | work | T-04 | 完整a2r/旧base/917及治理职责通过，记录917/runtime字节与source字符边界SD、source-gap/余16或17清单与941只读交接；V-01..V-05按对应入口，AC-01..04 | [x] 2026-11-08：冻结并集10组全绿（a2r 405例/994ext/992ext/base/917【D-993-2三红转绿】/infra-ace-m1/auto-test-case/ace-adapters/916/909【暖store环境归因，parent清后PASS】）；SD-01..04候选维持§5表（源字符/917守卫块/runtime a2r_std字节API边界——review验证、merge沉淀）；具名修订3个cookbook期望（ASCII值恒等）；余16清单冻结于§7.3；941只读交接表（4格SHA16）；健康零新警告/触面fmt净/无调试输出；commit `adddf8d06` |
| T-06 | review | T-01..05完成；冻结后停实现写入 | V-06独立逐AC/SD/全部原例反控/reader/健康审查，绑定r1/冻结commit；AC-05 | [ ] |
| T-07 | merge | T-06 pass与后续合入指令 | 冻结明确职责并集/有效证据复用与触码漂移核验，SD实际沉淀读回/来源交付/归档，逐格证据交接但不写941；AC-06 | [ ] |
| T-08 | merge | T-07来源与归档核实 | 全部命令输出根对账、证据摘要折本Plan后scope整删（具名持有另记），wt-guard clean后移除唯一树/分支/空组，登记刷新；AC-06 | [ ] |

## 9. 复审记录

2026-11-07 stage:new / PLAN-994 r1 / outcome:pass（仅规划静态合同，非实施/产品/认证通过） / next:后续实施指令及实际依赖就绪后T-01。T-01..08全部未执行，0/8，未建树/scope，未修改production/current Specs/语料/证书；规划不等golden批准或M1签章。静态复核收据在此续录。

2026-11-07 规划静态复核：0..10章节、T8/AC6/V6/SD4唯一连续且互引完整，work/review/merge任务均未执行；本包4格与当前binding的corpus域/原token/outcome/expected完全对账，src与独立reference存在，三包22格不重叠，余16不在本次范围。当前Spec路径、计划交叉链接、表格列数/空白/编码、现有固定组名与拟新增组标记、唯一编号/中央next-id=995通过。第一轮文档检查器将固定all入口误当未知组而退出，纠正检查器识别后完整静态复核通过；未修改产品入口或为通过放宽产品门。939/941规划前后内容hash一致；本轮仅写992/993/994与取号.next-id，未施工、未运行产品/认证/性能/独立实施review、未建树/scope/会话。

2026-11-08 stage: work | plan_id: PLAN-994 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-994/auto-lang | branch: plan-994-dev | base_commit: 8514d2f87 | code_commit: adddf8d06 | dependency_revisions: D991/D992/D993均归档（991计划hash与§4一致；992=64f93cb99、993=79e990f82为base祖先）；rustc 1.98.0 (88d9e12ae 2026-08-18) PATH身份=32证书 | task_ids: T-01..T-05（5/5） | evidence: §7.3（首次红重放/修复/四值双侧38-27-23-128512/4 certified+32 reused/snapshot 35+0 spawn/receipts 32/acceptance gap16 claim false 313/contract PASS/10固定组绿含917三红转绿/反控C1-C5全红/近邻字节稳定/941交接4格SHA16） | blockers: 无阻塞性；909首跑红=暖store环境归因（parent清后PASS，同993）；D-993-2（917三红）由本包守卫块恢复关闭，KNOWN-DEBT落账归review/merge | 未执行: 全仓统一回归（owner=merge按冻结并集；周期cargo t all按due由原owner） | next_responsible_role: review（T-06，同树独立上下文，冻结commit adddf8d06 + 本Plan文件）

stage: goal | plan_id: PLAN-994 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-994/auto-lang | commit: adddf8d06 | next: review（T-06）

2026-11-08 stage: review | review_scope: final | reviewer_identity/limitations: 独立 subagent 上下文（general-purpose，未参与work），同一冻结树只读，材料仅plan-994/t06-review/；未重跑10组冻结并集（merge持有）、周期cargo t all、work的C1-C5原样（以自带独立金样漂移反控替代）、全量警告基线差分（新符号抽查0命中） | plan_id: PLAN-994 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-994/auto-lang | branch: plan-994-dev | reviewed_commit: adddf8d06a33e6b7789a338b365d25f7ca5e319a（树clean，前后核验） | base_commit: 8514d2f87 | dependency_revisions: D991/D992/D993归档核验（祖先链）；rustc 1.98.0 PATH=32证书身份 | spec_inputs: SD-01..04目标文件均在主检出且当前文本待取代；树diff零docs/specs改动（按计划merge沉淀） | acceptance_results: AC-01 pass（独立G04推导：T01 3字符/8字节→38、T02字符位守卫块零a2r_std→27、T03 clone/move/拼接→23、T04标量→128512；声明型集合无测试ID/字面量特判；非声明接收器字节remap由单元断言钉住）；AC-02 pass（goldens wrap编译0错+4引用独立编译同输出；T08 panic 917金样字节未动→D-993-2为真发射修复）；AC-03 pass（snapshot原样"35 a2r checks green…32 STRICTLY valid…0 spawns"；receipts 32；acceptance gap16/claim False/313/45/32；contract 131行；自带1B漂移反控恰1 FAIL且树未触）；AC-04 pass（manifest全hash重算对盘33/32/85/81/43；余16恰=H-04六+H-06四+H-07六无H-03；交接SHA16对盘）；AC-05=本审 | pending_closeout_gates: merge冻结并集复用+漂移核验、SD-01..04实际沉淀读回、D-993-2 KNOWN-DEBT翻账（F-994-R1-03）、scope整删+wt-guard清树 | findings: F-994-R1-01（info）receipts -BuildIdentity经原始argv含换行即假STALE（in-PowerShell正确用法通过，操作注记）；F-994-R1-02（info）acceptance对相对-RepoRoot路径敏感（先存，994 diff仅触snapshot域）；F-994-R1-03（info）D-993-2已事实关闭但KNOWN-DEBT 4134行未翻——merge落账；F-994-R1-04（info）917 t08金样wrap编译paren警告（先存冻结内容） | evidence: 本节+plan-994/t06-review/notes.md（merge整删前已折本节） | next: merge（T-07）

## 10. 待澄清事项

1. 991的实测用于选定反例与范围，不等当前本包正式认证；T-01重核实际源/工具与现入口，缺证据不拿旧PASS补。
2. G04逐例审阅在任务内完成，非自动接受actual；语义无法证明/出现超出本包的producer问题，保具名阻塞并修订，不能换expected。原cert schema/std-only与资源帽不放宽。
3. H-04/H-06/H-07仍需原用户裁定/正式后继；本轮不预填NA/accepted、不改零gap守卫，三包全闭仍余16格。
4. 本轮只有规划授权，未来work/review/merge按后续指令；不为等价内部实现重复审批，不将其扩为新语言/新认证平台或修改939/941的授权。
