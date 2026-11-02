---
plan_id: PLAN-911
status: archived
feature_name: Native源码诊断事实通道与M1 B17–B19收口
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-02
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-ac/mir-native-scalar.md, docs/specs/auto-ac/mir-native-cache.md, docs/specs/auto-ac/project.md, docs/specs/auto-av/resident-function-install.md, docs/specs/auto-test-case/profile-v2-native.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/project.md]
new_spec_components: [docs/specs/auto-ac/native-source-diagnostics.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-av, auto-test-case, native-diagnostics]
depends_on: [PLAN-908, PLAN-818, PLAN-871]
current_step: 10
total_steps: 10
---

# [PLAN-911] Native源码诊断事实通道与M1 B17–B19收口

## 0. 变更摘要

承908已批准并交付的语义族判面，为现有native前端拒绝增加显式、严格、有界的诊断事实通道，使原B17/B18/B19的native检查真正验证type/assign-immutable/missing-return。复用共享前端实际诊断，不新增语言检查器，不在parent按源码/expected猜错误。一个Plan完成native typed cause→worker→父侧准入/映射→原源码检查/负控→M1三格证据回填与失败暖编辑验收。

**drafting r2、0/10，本轮只规划。** r2补齐操作级诊断快照/真实终止profile、父侧先准入后比较、首失败即停/类型化拒绝原因、所有返回路径真实计量、正式父证据绑定及909符号释放条件；新增V-15..20，不改变三格目标/原源/行为oracle或已批准G-DEC。不开实施树/材料，不运行构建/语言测试/探针/认证，不改生产/current Spec/binding/既有expected。架构见[Design43 §15](../../design/43-ac-av-typed-mir-and-abc.md#15-native源码诊断事实通道与m1-b17b192026-11-01target-state)。909用户确认正在实施，本包N/parent主体可并行；必要AV只读事实出口及公共H逐符号短窗，不改909捕获/资源生命周期，不等D909整包。

非目标：修新语言语义、旧a2r静态检查/未定型/认证、B18旧AVM infra、K06/C-19、AV资源/代际、scenario工具、native console/loader/ABI、切默认或M1签章。不重做908规则/runner，不改NA/accepted/分母，不把generic unsupported或任意非零当正确语言拒绝。

## 1. 目标

- G1：B17/B18/B19由真实共享编译器产生typed原因，保raw stage/code/message/span/顺序与来源；§5.3列明source-v1实际cold/warm/fallback链，默认/独立owned消费者保兼容，不把未接的新路由称为已支持。
- G2：显式严格模式在TestCase decoder、worker、parent一致启用；旧worker/缺因/错源/错版本/超额/伪载荷拒绝，默认通道保兼容。primary由compiler次序决定，不选择“恰好符合expected”的原因。
- G3：失败body候选准确绑定base/candidate/step/span来源，不覆盖现行code/source/env/依赖；修复、历史hit与新进程恢复真可用，未改函数生产/IO=0与对象/代际保持。
- G4：原3负例与NB17/18/19=2/2/0正控、错族/infra/缺因反控真实执行；只据正式父报告回填三个native格，保原源/行为oracle和全表/双轴。
- G5：完整受影响职责/消费者、资源预算、SDK/worker版本与909写面交接可核，治理及其他M1缺口诚实保留。

## 2. 架构方案

`共享source compiler/verify的真实Diagnostic → native只读typed快照/原因对象 → mn-worker显式source诊断观察模式 → 有界版本化source-diagnostic事实 → parent核请求/候选/工具/范围 → raw stage映射TestCase stage并按实际code判 → 独立源/近邻/反控 → M1格位证据`。

默认/未选模式沿现行外层MnError代码和分类。严格模式拟`mn.diagnostics: "source-v1"`（新增v2可选字段，由T-01冻结精确decoder/请求/echo形状，T-03启用），只准`ac/mir-native-scalar-v1`既有source域；v1文档/其他recipe塞字段、未知值具名拒绝。该recipe包含现有标量/record/text等正式前门，不扩新source支持。collections/typed默认保持旧合同，本包不声称它们新增严格模式。

## 3. 技术栈

既有ace-native Rust库/worker、ace-core Diagnostic、AV通用函数session已存在诊断事实、auto-lang-core TestCase adapter、auto-test-case v2严格decoder/report、908 tracked元数据及PowerShell验收入口。无新增crate/三方/runner/全仓目录。拟新增native私有诊断模块与AutoDown case/suite属于本职责长期资产；文件在T-02/04使可运行。

## 4. 需求分析与背景调查

### 4.1 授权、交付与选择依据

用户：“计划909正在执行；请规划下一步计划？”授权本轮选择并正式规划下一包，延续只规划不施工。选择本包是因为908已证实三个native pending主要为观察面丢失，N/parent独立于909共享捕获主体；AV资源/代际已有部分profile机制且与909 av-core/clone交叉，旧a2r检查是另一较大能力包，不先混入本包。所有已批准G-DEC/NA/R和双轴直接继承，不再请求重复批准。

**执行授权（2026-11-02）**：用户以 `/goal 完成计划911` 授权全链交付——在
`D:/autostack/.wt/lang-911/auto-lang`（branch `plan-911-dev`）内实施 T-01..T-06、独立复审、
批复内修复与 merge（canonical Specs、归档、worktree 清理、landing checks）。
开工时基面 `5a4eb4e25`（master HEAD = 909 归档提交）：**909 已交付**，§5.6/§5.7 的
W-S1/W-S2/H/M1 协调窗按“909先合入”分支全部视为已释放，本树单写；
V-20 改为对 909 交付基面的 consumer 验证（编译+909 相关组回归），不设 WIP 接口等待。
材料 scope `D:/autostack/plan-artifacts/plan-911/` 已登记（registry in_flight）。
T-01 冻结文档：`D:/autostack/plan-artifacts/plan-911/T-01-freeze.md`（全部出口清单、
`mn.source-diagnostic/1` 精确形状、互斥结果、raw stage 映射、caps、attempt 策略、报告落点）。

主线读取`367258e7ac3b90aa51fd90cc2d4b4e148522e011`。908 archived r2、12/12，delivery`dc8eca3ae`为HEAD祖先(exit0)，归档2a78a3247、清理da4fcf14f；906已交付清理8/8，905归档7/7。908当前规范记录core247/285、38阻塞；extension28/28、future45、cert38 gap、can_claim_m1=false。这是其冻结收据，不是本轮重新跑验收或当前未来计数预测。

909用户确认实施中，读取时Plan头仍drafting0/12；后续已观察到`lang-909`树@367258e7a，无已列WIP，不据header/瞬时clean断言未开工或改其状态。910为他方刚分配的generated-cargo-build-governance草稿，非本需求，不覆盖其文件/号源。本包不联系他方或启动agent/chat。

### 4.2 已观察真实链与内容身份

r2授权：用户要求复核诊断保真、父侧严格准入、失败暖编辑/函数增量及909交接，有遗漏直接修订，仍仅规划；另询问910职责。本轮只修911合同及相关目标设计/导航，保909/910计划与既有批准不变，不取新号。

| 来源 | SHA256 / 观察 |
|---|---|
| docs/specs/auto-test-case/ace-m1-gap-audit.md §11 | `63343b86c3424ca99d504a75a0125c42804f25ab9f515ea41f43aef48b569e51`；B17–19 ac泛化码原因不足、NB三引擎9/9实跑绿；旧a2r仍真实gap |
| ace-m1-applicability.md | `0cba0d2039674803a17700c3ea5c3d345f002efcf1390a0f53ddc20474546c25`；G-DEC-2已批准语义族，不要求各后端错误码同字面 |
| docs/specs/auto-ac/mir-native-scalar.md | `e5b3e8411491980c51be27acb3fb99e03e7da41fdaa56c30dcffed5c38f56e1c`；native准入/资源/数值与执行合同继承 |
| crates/auto-lang/test/cases/ace-m1/binding.tsv | `caf5f79fdc79f7853dd7fd66ee7d5b933cafc6b1a207bd05fd0e12a81185afd2`；131行12列/524格，B17–19 ac为diff:pending；原expected不改 |
| src/B17.at / B18.at / B19.at | 分别`90c03f2756b676a0cab70158b65f130e19b04634725325eeca325418e8247184` / `cf9358d904e6fa3b11a731a86f8cfb536fb25042c1c40348cdcafee8bf3b89df` / `1de47798f2993250be6f5e3fd88f2c1989483b3b8bb56f284fe5c5bafb5c5217` |
| native/src/lib.rs / driver.rs | 分别`34fd5615363533a69494d1222eab97c4aae13ea82483fe857c6d553141041477` / `0e415c26a85a955c50d844c084e20beae2c0e4bc5f85dbbafb684838c7ea3687`；MnError仅Unsupported等，compile_escalating真实Vec<Diagnostic>被压成首条formatted string |
| core/src/source/typecheck.rs / core/src/verify.rs | B17实际source.type-mismatch；B18 verify.immutable-write；B19 verify.no-return-path；前者类型检查、后两者HIR语义验证已拥有判定权，不新增前端检查 |
| native/src/{incremental,collections_incremental}.rs | whole_module_units丢stage/code只留首message；EditObservation.refused_reason是String；既有非快路径理由不能当语言错误 |
| av/src/session.rs::EditOutcome / render_diags | diagnostics为Vec<String>，compile_*的真实Diagnostic已render；若暖路径需typed出口，仅作诊断事实只读传递，不改变prepare/commit及捕获语义 |
| native/src/bin/mn-worker.rs | 当前外层mnworker/1、worker_version=mnworker/1.0；cold、incremental、warm_edit_fallback等多处手写code/message，需要具名穷举，不能只改一处 |
| crates/auto-lang-core/src/test_cases/ac_mn.rs | `f9a8e6d3141fd7b94dc852b6e60ab011c943c344472cb2033214a84401da0950`；whole与步骤判面按mn.unsupported.frontend归parse；目前不携带完整raw原因 |
| auto-test-case/src/contract.rs::ProbeMnSpec / report.rs::NativeRecord | v2 mn有entry/caps/session/typed，无diagnostic mode；report已有原生观察/ArtifactRef，扩面须保持v1/未选路径及既有精确键合同 |
| b-diags.case.ad、nb-diags-908.case/suite.ad | 原native B17–19仅expect parse/generic；NB真实值2/2/0，原852 base36case/97check及908近邻3case/9check不靠增删凑数量 |

### 4.3 r2静态复核新增事实

- driver::compile_escalating返回Vec<Diagnostic>而未保留终止profile，scalar/record的surface拒绝会尝试更宽域；需在真实终止域生成来源，不把前面被消耗的surface拒绝选为primary。
- av::session::EditOutcome只存rendered diagnostics/non_fast_path_reason；native::incremental失败早退常以Default建观察，结构delta只在尾部成功路径汇总；AV部分阶段counter也在成功之后才bump。默认0不证明失败操作未做生产，必须从真实dispatch/已有core::body_work（实现位于core/src/lib.rs内）和所有退出路径取证。
- mn-worker::serve_incremental当前用`r.contains("syntax fallback")`选择回退，拒绝后仍可能继续后续step；whole_fallback会物化剩余编辑。严格新模式必须有类型化refusal分类、首失败终止和accepted-prefix事实，不能把非法中间态被后续修复覆盖后的结果当本步拒绝证据。
- 910已成形为drafting r1、0/9的跨仓生成构建治理合同，依赖905/906交付；不属于M1诊断/语言语义包。909树仍在，当前读取无列出的WIP不是其未执行证明，仍按用户确认“实施中”安排符号窗。

## 5. 详细设计

### 5.1 typed原因与三族映射

| 原义务 | 原source事实 | raw原因（当前版本） | 严格TestCase判面 / M1族 |
|---|---|---|---|
| corpus:B17 × ac | true+1非法 | verify / source.type-mismatch | typecheck / 原始code；family=type |
| corpus:B18 × ac | let局部赋值非法 | verify / verify.immutable-write | typecheck / 原始code；family=assign-immutable |
| corpus:B19 × ac | 非Unit函数非全路径返回 | verify / verify.no-return-path | typecheck / 原始code；family=missing-return |

映射是批准语义族的本producer显式表，不改binding expected为特定backend码，也不要求AV/旧引擎同步同码。raw text→parse、bind→resolve、verify→typecheck；未知stage拒绝授信，不能default成typecheck。完整raw stage/code/message/span另存诊断事实，message不作为唯一证据。只从同一次真实编译得到的typed诊断构造事实，native可用自有Eq快照避免为此改core Diagnostic基础类型。

保持compiler顺序，当前源compiler返回Vec而无另一个primary标识，故本schema的primary固定为0；不按oracle/expected挑一条命中的原因。capability、source.unsupported、syntax、未知名、backend/MIR准入/资源/工具/信号等不能凭generic外壳算以上三族。未知raw code保真但不给该族PASS；如实际已有code与表不同，先对真实code/来源修正映射并记录，不改语言规则或解析消息伪造。

原因必须是**本次操作拥有的不可变失败快照**：同一次真实编译/verify失败在退出点保留完整有序诊断、原始message Unicode内容、raw stage/code/span及产生phase/profile。默认字符串展示由同一事实投影，不能分别计算或反解析Debug。禁止以global/thread-local或session的“last diagnostics”迟读替代此返回对象，也不能在下一次调用清空/覆盖当前已返回事实；重名case/相同source/同base revision的多次失败仍各自绑定操作，成功/关闭/下一次拒绝不得借用上次原因。额外快照只持有受预算约束的元数据，不pin AST/HIR/code/env或live closure graph。

域升级按真实执行保真：记录尝试的profile及仅用来升级的surface拒绝类别，primary只来自最终终止域的真实原因；混合semantic错误与surface拒绝时不越过语义错误。该证明不允许在worker重跑scalar/record/text判断“刚才大概在哪失败”。captured profile/producer/语义版本必须来自实际调用点，不能把所有source错误写成scalar。

### 5.2 显式观察模式、协议与父侧准入

拟新增`mn.diagnostics: "source-v1"`，只在v2/native source recipe合法，默认省略时保旧行为。T-01冻结精确字段、worker capability echo、payload与report落点：优先保持外层mnworker/1和现有recipe，新增内层schema `mn.source-diagnostic/1`；若必须变worker version，worker/parent/身份探测及各实际镜像同步，不能一边旧常量、一边新解释。严格模式**即使合法程序无诊断**也须回显已启用的mode/version，旧worker静默忽略新字段不能假装兼容；未选路径不被迫获得新字段/判面。

payload至少携带schema/mode、source profile/producer、raw诊断有序数组及primary、来源path/span基准与source SHA256、实际候选/step身份；worker实际SHA/工具/请求echo沿原信任链核。source input/expected分离，expected/case ID不进cause生成函数。cause列表≤64、单message UTF8≤4KiB、payload总UTF8≤64KiB；code/stage/path字段亦受总cap与精确schema约束。空列表、无primary/非0、重复对象键、矛盾来源、越界/半UTF8 span、未知字段/版本/错源/错step均拒；不为去重而删除compiler真实返回的相同诊断条目。超额为Resource/协议ERROR，不截断后取一条当PASS，不影响语言支持集。

T-01固定以下互斥结果形状：①成功=严格mode/version确认+原正常执行结果，不带source-error原因；②源码拒绝=source-error快照+必要worker元数据，不带exit/value/guest成功结论，cause非空且primary是JSON整数0；③非快路径/错误请求/陈旧或跨session候选/关闭/资源/infra/native准入等=各自typed类别，不伪装source-error。原协议允许`ok`容器承载worker_version/counters/spans等元数据，因此不能仅凭存在`ok`就认为成功；但source-error同时夹带执行结果必须拒。`0.0`、`"0"`、false均不充整数0，任何隐式类型转换不可授信。

完整关联包括parent请求/操作ID、实际session/attempt、base revision与accepted-prefix、失败candidate源码SHA、step序号、recipe/entry/caps的规范配置、实际profile/producer/模式/工具身份；同source hash也不得跨请求/session/caps/step重放。parent独立核它拥有的请求/编辑链/配置/实际worker SHA；producer的FnEnv context_revision与实际读取事实/既有只读context见证相连，不让parent新做语义检查，也不能把payload自报的family/verified/PASS当证明。多字节变长编辑保raw span_basis及映射来历；不可逆或未提供映射时如实标明不可定位，不能用0补造精确位置；实际producer合法返回的零宽/非零span均保真，给出显示位置需有可核坐标变换。

producer实际接收的路径（现有mn.at/probe.at）与case逻辑路径分别记录，在源码hash/输入绑定和已知映射下关联；不能拿case basename重写raw path，也不把这两类路径天然要求同字面。raw span保原producer坐标和精度，只有宣称source/fragment UTF8字节坐标的字段才按对应basis核边界，不改成字符序号；必要显示坐标另记可核映射，未知精确位置不伪造。T-01须以真实三负例及多字节正控冻结可核basis/精度形状，不用格式化message推断坐标。

严格解码从受cap约束的**原始响应字节**检查新payload的重复键/字段类型/有限结构；先反序列化进会覆盖重复键的Map，再查去重后的keys不足。拒绝控制必须真的在线上JSON字节制造重复stage/code/primary键及矛盾结果，不用已去重对象伪装。先检查总字节/条数/字段长度再额外clone/format/写artifact；新观察cap仅适用于source-v1，不改变未选路径/源compiler的既有支持或错误显示。

父侧先核协议/身份/来源/限额，再产生observed stage/code。缺typed cause的generic错误在严格模式不能降级成成功或合法语言拒绝。既有MnError/CLI外层分类默认兼容；对工具不存在/worker超时/崩溃/取消/流超额/读写失败沿ERROR或Resource，不因附了错误字符串变语言诊断。

准入失败必须在oracle比较前终止，既不把协议错误改成“错误码不匹配”语言FAIL，也不允许 expected声明了一个error就满足；合法但错族是独立的oracle mismatch。版本字符串/worker自报hash不等实际工具身份，启动/观察沿父侧原身份设施核实际二进制及capability；工具或上下文漂移为STALE/ERROR，不能悄悄选择旧worker降级。expected/stage/code/family不进入事实生产请求；换expected只能改变parent比较结果，不能改变worker事实或primary。

优先用现有报告ArtifactRef挂父侧已校验的诊断事实JSON，绑定hash/来源/工具/step，不改test-report/1键集、不新造report版本/runner。精确选用NativeRecord还是证据artifact由T-01结合序列化/消费者冻结，必须能从正式父报告独立读回raw事实、准入结论和对应原source；仅stdout一句“原因正确”不足。该JSON是当次诊断证据，不是guest COFF/PE产物或可缓存PASS。

### 5.3 生产链覆盖与默认兼容

兼容比较须区分成功代码/语义与工具观察元数据：有效源码的COFF/ABI/位型/缓存语义不为诊断改写；worker真实SHA、版本/能力或report的诊断模式元数据因新构建变化时如实登记，原receipt/工具身份需要STALE就按原门重核。不得既承诺旧worker字节/所有report字节不变，又升级其实际协议；也不能为了复用旧receipt伪装同一工具。未选择的v1/default字段接受/拒绝集及原显示保持，严格模式mode/事实不泄漏到其他recipe。

入口支持矩阵须与实际代码一致，不能用“同一recipe”推断所有warm域已存在：

| 当前实际入口 | 新严格观察 / 增量承诺 | 具体关闭条件 |
|---|---|---|
| MnDriver whole source / mn-worker普通run | 现有scalar→record→text-record升级前门可观察typed拒绝 | cold首次语义编译保终止profile/原序；合法程序沿原link/run，源码拒绝不进入后段 |
| mn-worker::serve_incremental/serve_rounds → MnIncrementalSession | 原正式标量会话的source-v1拒绝、首失败/accepted-prefix与函数增量 | 支持的scalar body-only语义拒绝实际单body工作、未改全0；严格mode/operation/context必须穿过serve→内部request/session/rounds，不被现行只复制recipe/steps的中间对象漏掉 |
| serve_incremental_whole_fallback / rounds物化 | 原允许非快路径的严格逐步拒绝/显式whole工作 | 继承相同观察policy/上下文，第一次真实semantic失败即停；不在已知语义错后重编译取因，不先应用tail |
| owned_source_incremental.rs::OwnedSourceMnSession | 现有record/text暖API作为实际受影响consumer保兼容；不是已存在的上述worker edit路由 | T-01登记其MnError转换/默认计量与单元身份。需传播共同错误类型时同源透传，不借本包新造多域warm worker；若正式新mode确需支持这条API则先有界修订入口与source合同 |
| typed/collections专用recipe、default/v1 | 不接新mode、原功能保持 | mode错recipe拒绝；新typed cause不能被当作该profile已支持或改其ABI/源域 |

r1“含type暖source”没有区分标量worker与独立owned API，属于未核实入口的歧义；r2修正为**实际支持路径**保原强门。worker不支持的type/enum/声明变更必须给原明确不支持或非快路径，不能静默假装单body增量，也不能因新诊断capability echo把它算为已支持。已有owned API的含type暖能力保原完整兼容/函数增量证据，不借修正误删原支持。标量worker做多字节变长/多函数/多步错误，owned consumer做其原合法type路径；新增多域诊断入口另需实际producer验收，不从904暖片段合同直接类推。

N主写native/lib.rs及拟新增私有source_diagnostics.rs，driver::compile_escalating与真正前端失败转换，incremental/src及worker所有相关cold/steps/warm_edit_fallback出口；用一个typed转换/序列化入口防止分支丢原因。T-01列出实际编译调用点→错误对象→worker分支→parent消费者，逐项标changed/legacy-only，不靠字符串全文替换。

只对已经由编译器判定的事实做透传，不新增类型/控制流/所有权分析、不再执行一次compiler重建原因。typed入口/collections recipe不接受新模式，原旧字段/版本/ABI与结果位型/资源/五域默认有效程序保持；未选source新模式仍可按旧generic检查运行。新mode对应B17–19检查明确迁移stage/code，其他既有expected/golden/cert不批量改。

共享S为只读诊断出口时保持现有字符串展示与API兼容：在原真实Diagnostic仍在的位置保留typed事实，交给native观察，不重跑source、不重新推断、不改变prepare/commit/对象寿命。909若正修改同符号，先交接最小出口再让窗；不能拿其分支实现当主线或把909完成设全局前置。若必须变更前端语义/通用查询生命周期才可获原因，needs_replan，不将 workaround当本包已闭。

现行字符串化分叉必须显式覆盖两类原生产点：prepare/body frontend的Err(ds)、verify_function的Err(ds)；还要穷举cold init、非快路径与commit拒绝。source-diagnostic snapshot随同本次Err/结果返回，保留raw phase，既有字符串为同份事实的展示投影。新增只读出口不是“从可变session.last_errors读一次”；只读查询不得重算诊断、改变FnEnv读取依赖或刷新旧凭证。允许必要的有界复制/记录数据，不允许改变909捕获表示/效果、commit逻辑或活对象生命周期。

### 5.4 函数增量、失败原子与恢复

暖编辑诊断绑定**失败候选**：base全文SHA、candidate全文SHA、step序号/编辑范围和必要fragment source基准；父侧由实际请求编辑链重建/核对。fragment span先标基准，再以已验证范围换算完整source，保raw坐标；禁止给新错误贴最后有效snapshot身份、重放旧body诊断或把span越界强行clamp。

失败验收按实际phase分开：①本包三类**源码frontend/HIR语义拒绝**，不commit source/env/FnEnv已读事实/安装roots/active manifest/receipt，不进入MIR lower/codegen/link/run guest，不产生新guest COFF/PE；②未开始语义检查的non-fast-path/请求/关闭/跨session/陈旧candidate拒绝，给typed refusal，不能编造source-error；③后段MIR准入/资源/link/发布/传输失败保其原错误分类与既有事务合同，不能伪装上述三族。后段失败前可能已有真实prepare/codegen或不可变cache写入，必须如实计量/说明处置，不承诺“所有失败均零生产或从未有暂存文件”；活动source/code/roots不被观察功能改变。本包不借此扩修原通用发布平台。

不能为取诊断调用run_current或whole重编译第二遍。支持的单body源码拒绝只有改函到达的必要前端阶段可做工作，未改函数parse/resolve/typecheck/HIR/verify/lower/emit/工件IO=0，同HIR/MIR/code对象和代际保留；扫描/计量窗口包含诊断准备，不能把全量工作藏在counter前。到达阶段的**attempt**在真实调用之前/producer入口记录，成功与失败分账；未到达为0，未观测为gap，不能把提前返回的Default零字段或仅成功后bump的counter当证明。沿现有body_work/实际调度入口和native阶段计数补所有return/Err出口快照，记录changed_fn已知/未知，不能按结果推算次数。强制旧全量工作再filter、重复编译取因及清零早退统计的真实负控均须使门红。

AC-03要求的实际支持路径若仍缺生产阶段观测，gap表示尚未关闭，不能把它排除后宣布增量验收通过；只有明确未到达的阶段可有证据记0。接口目录/声明脚手架与真实body生产分列，不能把允许的有限目录遍历偷换成其他body重建，也不能只凭总counter相同推断每个未改单元为0。

严格模式从原分支保留typed refusal kind，区分language diagnostics、syntax-not-converged、signature/context drift、invalid edit、session/candidate、resource/infra；禁`contains("syntax fallback")`或解析rendered reason做路由。语义诊断是终态，不能进入fallback。只有原合同允许、发生在语义检查之前的非快路径可走显式fallback；该step第一次真正的whole semantic compile允许，但不把它冒单body快路径或“未改工作0”。旧默认模式/拒绝显示保持兼容，T-01列清两个模式实际分支与消费者。

source-v1多step请求执行到**首个失败即停**：保存已接受prefix，每个原因绑定当步实际candidate；后续step不得物化/编译/run或把中间非法源修复后只报最终成功。专门测成功edit A→非法edit B→后续edit C修复B的序列，结果必须仍是B的真实拒绝，C未执行；如因合法syntax非快路径进入物化模式，也在每步构建的候选失败处停，不先把全部tail应用完。accepted-prefix/base来自A之后当前有效源，不能错用初始源，也不能把B失败的源变成current。恢复/修复用下一次独立API操作或受支持的后续会话请求证明，不能偷跑同请求尾部修复。

按合法warm源→非法body候选→读回原source/代码并单独执行原版本→合法修复→改回历史→close/新进程restore验证。错误响应本身不得执行原PE伪装成功；原版本可用验证是独立后续动作。fresh/hit/restart/body-edit四状态都核当前来源/原因，命中/恢复不缓存诊断PASS；输入/工具/配置漂移按原身份重核。只读store写0，取消/超额/写盘失败不留半诊断证据，下一次有效read/restore必须真实可用。

以上API继续操作与worker请求终止分别验：library session拒绝后仍可基于最后有效版本修复；one-shot worker若按原合同退出/close，不承诺自动保持活session，只证明下个进程从已接受持久版本恢复。ROUNDS/持久请求若存在按其正式边界验新操作，operation/session/context身份必须重核；不能把进程重启当在线修复或HMR。含type/enum和多字节变长编辑须保body/context凭证绑定，尤其同owner/context异body及同source异caps/session拒绝，元数据取证不能刷新查询依赖。

### 5.5 文件化、M1接线与保护

T-04使拟长期资产`crates/auto-lang/test/cases/ace-m1/native-diags-911.case.ad`、`.suite.ad`可运行，原负例直接引用src/B17..19.at；合法正控优先引用NB17/18/19原源，复用独立oracle2/2/0，不复制改写原程序。原b-diags.case.ad的三个native check切严格mode并验实际原因，case/check ID和base97义务不丢。新增错族/同族变形源与协议反控只在本包长期夹具或材料副本，不能提交一套故意错误的原expected。

只有真实compiler原因+父侧strict判面+合法近邻/错族/infra反控+原职责组通过，才能在实现树迁移corpus:B17/B18/B19的ac token及note。保原12列/id/ns/outcome/expected/source、其他路径/规则/NA/accepted/分母。908 source-manifest同步本包实际改变的case/长期oracle/消费者hash，治理contract/acceptance按正式同身份证据重验，不因sidecar能读就算执行证据。

M1回填必须引用正式父报告的case/check/source、worker/protocol/mode、raw原因事实hash、判决及日期；旧generic check PASS、raw ok或手改token不足。若908当前入口不能核这种证据，T-04仅为这三格补既有严格入口准入，不重建全局账本/runner。原908 core247/285与903计数保历史，当前数据另记，不预言本包后必然250/285；909或他方可同时改变其他格，按真实身份合并。其余阻塞/future45/cert轴保留，本包不签M1、不改a2r认证或expected凑绿。

T-01把父报告落点收敛为一种可执行合同：优先现有ArtifactRef引用parent创建的有界诊断JSON；若现有序列化确需NativeRecord扩展，须先写出精确合法/非法样例与旧report/2读者行为，再实施，不留双套随意准入。artifact由parent在本check受管目录内校验后原子发布，worker不提供任意文件路径或PASS指令；schema/hash/来源/工具/mode/operation/primary与父报告execution_key（suite hash+case/check绑定）相符，不能跨case/step复制。半文件、缺引用、错hash/未持有文件或保存失败均ERROR，不以正确code字符串或binding token补绿；随后重新读取该正式报告/文件须通过。同一组有效证据可按完整身份复用，不能把新运行旧facts文件混为同一版本。

### 5.6 预算、正式门及写面

诊断新内存分区沿64KiB/64条/4KiB上限、转换/JSON/父侧artifact均计入预算；native源、managed cache/working-set/历史、worker stdout/stderr/Job/deadline沿原更严cap，不加后台进程/大日志。T-01抄录真实现行数值与归属，不能直接拿core较大的输入cap覆盖native原较小cap。溢出不panic/截断授信；清理结束无遗留worker/guest/未登记临时输出。

正式四状态沿native现有周转起点、原规模/样本与结构门，再以本包source-v1模式验fresh/hit/restart/body-edit各30样本，P90≤2000ms、长期亚秒目标保留；冷准备/构建、发现/执行/清理/整命令与诊断传递成本分列，记录父峰值/管理集、未改生产和对象身份。既有更严格条件/原采样不弱化，不新增全仓三warm/影响图。正式计量由T-05启用原performance入口中的命名case，日常源case不隐式启动rustc认证/全压力。

| 面/owner | 911范围 | 接受/回交与禁止 |
|---|---|---|
| N911 | native/lib/driver/incremental/worker诊断转换/序列化、私有反控 | 不改CLIF/MIR语义、数值/ABI/typed位型或native runtime；真实源码诊断前codegen0，不以补检查代透传 |
| S只读出口 | av/src/session.rs的typed诊断事实/必要查询错误出口 | 与909按函数/字段短窗，保原字符串/prepare-commit/依赖/捕获/资源合同；不改core/typecheck/verify的规则，未知额外写面先具名核查 |
| H911 | auto-test-case::ProbeMnSpec decoder、ac_mn parent/source判面、必要report artifact | 新mode严格可选，v1/默认/typed/collections兼容；909若接同contract/mod.rs只按各自符号单写，不以append-only准并写 |
| M1 H | B17–19 ac check/token/note、source-manifest与严格准入；其余只读 | 909拥有K06与其source suite，911拥有B17–19 ac证据；同binding/manifest文件分段仍须串行窗口，记录before/after hash与case集合 |
| D908 | applicability/SCN规范/852治理正式版本 | 已归档可消费，不改G-DEC/required，capability新报告只重核三格相关来源，不能回开908/903 |
| 910治理 | Cargo/生成工程/工具与公共注册若实际相交 | 读其正式阶段收据，不借草稿假设功能存在；工具身份/固定组微窗，非整包前置 |
| 运行/收尾 | 本包相关组、原条件正式命名计量 | 与909资源/函数增量采样错峰，不停对方进程、不持写窗等机器；正式门通过后独立review/冻结门/沉淀清理 |

交接收据至少owner/revision、路径/符号、主线SHA、worker/mode/schema、源/manifest/规则hash、已跑/pending、窗口开始/释放。当前owner依实际写面登记释放，不预设909必须先让窗；N/parent可先推进，依赖出口的新warm链只等该短窗。接受主线或owner明确交接、已提交且有验证收据的稳定阶段，不能消费WIP/把草稿接口当已发布；最终独立review绑定实际合入的接口/工具身份。共享出口未释放不能跳过warm门签交付。909先合入则911接受其基面/核出口与K06来源影响；911先合入则交付只读API、default兼容和完整消费者收据供909重核，不代改909合同/进度。

### 5.7 与909的具名窗口与失败关闭条件

| 窗口 | 实际路径/符号 | 911允许的写入与释放条件 |
|---|---|---|
| W-S1 通用失败出口 | av/src/session.rs::EditOutcome/rejected、prepare的frontend Err(ds)/verify_function Err(ds)、必要只读结果accessor | 操作级不可变诊断快照、拒绝kind与候选来源；不改prepare/commit决策/安装/捕获/资源生命周期。新增Rust字段不是天然append-only兼容：列全部构造/匹配/consumer，和909冻结类型/所有权/默认字符串/版本后单写，旧caller与909相关consumer编译验证后释放 |
| W-S2 真实生产计量 | core/src/lib.rs::body_work、现有resolve/typecheck入口与HIR verify调度；av/session失败返回快照 | 优先读取既有producer观测；缺attempt钩子时仅具名加实际dispatch计量，不改core语言规则/捕获分析。909拥有相关S，未登记不得以“只是观察”并写；旧全量/早退零值反控必须生效 |
| W-N 911私有面 | native::{lib,driver,incremental}、mn-worker source-v1分支/转换器 | typed cause/refusal与所有返回路径观测，默认行为兼容；不得改909 lowering/AV graph。可独立做cold/协议最小链，warm部分接受W-S1/2接口后实证 |
| W-H 公共合同/注册 | auto-test-case::ProbeMnSpec/decoder、ac_mn.rs、test_cases/mod.rs、test-groups.toml及实际report序列化符号 | 精确字段/分支/组名单单写，统一新mode/版本/default省略规则，不能提交909别处WIP。消费者与旧recipe/SDK门通过后放窗 |
| W-M1 同表不同格 | binding.tsv、source-manifest.json及相关source suite/current Spec导航 | 911只动B17–19 ac检查/状态与自己的新增输入，909只动K06 av及其新suite。表和manifest仍整文件原子发布，先读对方正式SHA再形成候选、复核原字段/各自case集合，不能两个窗口并写同文件或用旧总表覆盖新K06 |

若909的共享表示/代码对象/查询上下文接口在稳定阶段后再次变更，标记911受影响证据STALE，重绑接口/身份后只重核受影响V/完整职责组；不拿旧pass免除真实consumer变动，也不要求13慢例或整波全部重跑。S/P/H各自未释放只阻其实际依赖任务；worker工具/正式采样另排机器窗，不持源码写窗等运行。窗口未取到要在T-01/T-05记录具体owner/符号/下一动作，不能称911全面无条件就绪或悄悄把必需项留给909。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/native-source-diagnostics.md | 无→操作级typed快照/终止profile/上下文与primary、互斥结果/版本/上限/拒绝合同 | 原因不能迟读last-errors、靠generic/expected猜 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-ac/mir-native-scalar.md ; docs/specs/auto-ac/mir-native-cache.md | 泛化错误→真实原因/失败candidate与accepted-prefix、typed refusal/首失败停、全出口attempt统计；成功cache语义不变 | 分phase失败原子与真函数增量 | AC-01/03/05 |
| SD-03 | modify | docs/specs/auto-test-case/profile-v2-native.md | mn旧字段→显式source-v1/能力echo、原始JSON严格准入及正式事实artifact，v1/未选不变 | 父侧先准入再比较、原子证据/下一次read | AC-02/04/05 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md ; docs/specs/auto-test-case/ace-m1-gap-audit.md | 三格pending→实际实证/仍缺状态，保908冻结数、原源/预期/双轴 | 不自动凭token称M1绿 | AC-04 |
| SD-05 | modify | docs/specs/auto-ac/project.md ; docs/specs/auto-test-case/project.md | 原导航→新正式诊断/消费者/资源及残余owner入口 | 合并后current可定位 | AC-05/07 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | rendered原因→随本次失败返回的只读typed快照/拒绝kind/候选context与attempt出口，原字符串/prepare-commit/对象生命周期不变 | 暖路径不能反解析/二次编译/伪零计数；W-S1/2与909交接 | AC-01/03/05 |

## 6. 测试设计

全部拟执行。Rust用已有native/tests/conformance.rs和cases模块，adapter/contract沿现有目标；新suite和必要固定组拟`case-ace-m1-native-diags-911`由T-04启用（all=false、显式ACE）。故障控制使用原worker隔离测试缝/材料副本，必须证明变异生效及门真实红后恢复合法控制；不把改oracle的测试副本提交成原期望。

| V | 入口/输入 | 必须观察的oracle、反例与边界 | AC / owner_stage |
|---|---|---|---|
| V-01 | B17原源→真实driver/worker/strict parent | raw verify/source.type-mismatch、family type、无guest产物/codegen/link/run0；只generic/错阶段不能过 | AC-01/04 work T-01..04 |
| V-02 | B18原源，同链 | verify/verify.immutable-write、assign-immutable；不能用syntax/非零代正确拒绝 | AC-01/04 work T-02..04 |
| V-03 | B19原源，同链 | verify/verify.no-return-path、missing-return；与“补return0”正控区分，不新增return检查 | AC-01/04 work T-02..04 |
| V-04 | NB17/NB18/NB19原源，default及source-v1 | 真实PE运行2/2/0；严格模式合法也echo能力，不因统一拒绝/旧worker静默忽略过门；908原9check保留 | AC-02/04/05 work T-03..05 |
| V-05 | 同源错族、syntax/resolve/unsupported、backend/infra/resource伪装 | 三族只按真实code+stage；原因不足不绿；capability/崩溃/超时保持各自非语言分类 | AC-01/02 work T-02/03/05 |
| V-06 | schema/worker/mode/echo/source/step错误、空因/未知字段、UTF8边界/超64条/64KiB | parent在比较前拒；超额ERROR/Resource；合法非零span/多字节正控过，不truncate授信 | AC-02 work T-03/05 |
| V-07 | 多诊断顺序/primary；换case id/path、空白和同族新形状，错误expected试探 | primary不按expected挑；真实cause顺序保留；不依caseID/原B文件名/消息关键词或二次重编译；错expected控制真红 | AC-01/02 work T-02..05 |
| V-08 | 合法标量worker warm→非法单body，多函数/UTF8变长；owned API原type路径独立兼容 | candidate/base/step/span绑定真实失败源，source语义拒绝不commit/后段生产，未改生产IO0/对象代际恒等；未知域明确拒或非快路径，不能把owned API冒worker已有路由 | AC-03 work T-02/05 |
| V-09 | 非快路径/fallback错误与已提交step后新错误 | fallback明确计数/原因分态，不猜rendered string；每步身份准确，不能拿初始源或旧诊断冒新失败；合法fallback正控保留 | AC-01/03 work T-02/05 |
| V-10 | 修复/改回历史/hit/restart/read-only/损坏及旧tool身份 | 真实有效read/restore再运行；readonly写0、合法hit不重生产；错误不缓存PASS或复活旧env，漂移重核受影响 | AC-03/05 work T-05 |
| V-11 | 3原case+父报告/事实artifact+binding/manifest+908严格入口 | hash/worker/源/check一致才回填；旧generic绿/缺/伪报告/零选择/SKIP不可授信；131/524/12列及未涉格不变 | AC-04 work T-04/05 |
| V-12 | v1未知新字段、默认五域/typed/collections/OwnedSourceMnSession及909相关消费者 | 未选代码/ABI/位型/cache语义不变，工具观察元数据/STALE如实；按§5.3矩阵保原type暖API，不补造worker多域；mode错recipe拒；旧版default正控/strict旧版拒分开 | AC-05 work T-03/05 |
| V-13 | 四状态30样本/原规模与正式native结构/资源门 | P90≤2000ms、未改真实生产0/身份/原cap同过；父峰值/诊断字节/prepare/整命令分列，原native样本不减 | AC-03/05 work T-05 |
| V-14 | 取消/流超额/写盘失败/半响应/故障注入后的下一次请求 | 保infra/resource、无半事实或进程遗留，原source/code仍可用；恢复正常请求真通过 | AC-02/03/05 work T-03/05 |
| V-15 | 同次producer→Err→cold/step/rounds/fallback→parent；同source/base连续失败/成功、另一session/caps/profile | 原字段/顺序/重复项保真，mode/context/operation不在内部转发丢失、profile为真实终止域；无last-errors/活对象pin，合法多cold profile正控过、不伪造多域warm | AC-01/02 work T-01..03/05 |
| V-16 | 成功A→非法B→后续C修复、syntax合法非快路径及semantic message含fallback字样 | B绑定A后candidate，C未物化/编译/run；typed kind不读文本、semantic不重编译取因；真非快路径首次whole只做当步且继承strict policy/身份，计数诚实 | AC-01/03 work T-02/03/05 |
| V-17 | resolve/typecheck/HIR-verify失败早退、invalid edit/关闭/陈旧候选与合法正控；强制旧全量或早退Default零统计 | 实际dispatch attempt含失败、未到达0/未观测gap，changed/unaffected与真实身份/IO可核；隐藏whole/二次编译/清零观测反控门红，不由成功计数或结果重建 | AC-03/05 work T-01/02/05 |
| V-18 | 原始线上JSON重复key两种顺序、primary字符串/浮点/bool、矛盾source-error+exit、未知嵌套字段、合法metadata-only ok | 在oracle比较前ERROR，错误expected不能承接协议错；合法成功仅mode确认/结果、不带失败cause，source-error带metadata可接但执行结果必拒；合法0/非0/零宽span按basis保真 | AC-02 work T-03/05 |
| V-19 | 正式父报告/diagnostic artifact的source/mode/tool/context/operation/execution_key/hash错配、半文件/发布失败、下一次read | 只有完全同身份证据可回填三格；旧generic/新case旧fact/worker自报PASS不能绿；失败无半引用，恢复重读真实通过，未涉格不覆盖 | AC-02/04 work T-04/05 |
| V-20 | W-S1/2稳定接口与consumer验证、W-H/W-M1串行发布；909前后合入与接口再变 | 具名SHA/revision/默认行为/ownership与全部消费者收据齐；Rust构造/匹配不因加字段漏编；接受阶段后再漂移STALE只重核受影响，S/H未释放不冒warm门通过 | AC-05 work T-01/05 |

完整职责/必要消费者初选：`cargo t ace-native ace-native-worker ace-adapters auto-test-case`；确触AV诊断公共出口补`ace-av ace-source`的完整职责与909相关原闭包source组，不能只跑native自证；公共宿主观察变选`ace-host`；元数据/入口选`infra-ace-m1 infra-fixed`；正式源组`case-ace-m1-base`与拟新增911组，原typed/collections消费者按MnError/worker实际影响选`case-ace-m1-typed-native-891 case-ace-m1-native-collections-889`。decoder可能影响其他producer时选必要`ace-a2x`等实际消费者，写明理由，不自动展开全仓。正式性能沿`ace-performance`新增/受影响命名case及原ignore/平台/Job/资源组，不重跑13慢测整包；原callee/资源门不因诊断字段改动而删。

work内部逐AC/V/SD自查，final独立上下文review同唯一树顺序执行，不能由自查替代；用户未指定人员/模型，不创建其他agent/chat。对相同输入/工具身份的有效证据有据复用，发现真实修复/漂移只重验受影响门；未执行/首红/环境归因/重放绿分别保留。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01..03/05/07/09/15/16：cold/warm/fallback同操作不可变typed快照，真实终止profile/raw字段/顺序/primary保真，不迟读last-errors、不新增检查/重编译取因/按expected选原因 |
| AC-02 | work | V-04..07/14/15/18/19：显式mode/版本/echo/互斥结果、原始JSON类型/重复键/来源/span/context/cap先准入再比较；旧/缺/错/超额/协议错不降为语言断言，合法metadata与多字节/多因正控过 |
| AC-03 | work | V-08..10/13/14/16/17：本包source拒绝phase无后段生产/commit/guest执行；typed非快路径/后段失败各自诚实，首失败停/accepted-prefix及全出口attempt/未改生产IO0/对象恒等，下一独立操作或进程真修复恢复，正式原门/资源同过 |
| AC-04 | work | V-01..04/11/19：原3负例/2/2/0近邻+反控文件化与独立oracle，仅据正式父报告+完整绑定/原子发布fact/重读回填三格；原源/行为expected/其他格/分母/NA/双轴保护，不冒M1 |
| AC-05 | work | V-04/10/12..14/17/20：默认/v1/其他recipe/typed/collections及909实际消费者兼容；工具元数据变更按原STALE，W-S1/2/H/M1释放回交与预算/SD-01..06/残余owner齐，无新runner/未处理warnings/调试输出 |
| AC-06 | review | 独立逐AC-01..05/V-01..20/SD/源/身份/负控/遗漏延期与健康核验，revision+冻结SHA绑定pass，不以合成/计数器代机制 |
| AC-07 | merge | 独立pass后冻结完整受影响组/必要消费者门、真实合入/SD读回/归档、wt-guard/全输出对账清理和残余后继，10/10真实，不重开旧Plan或写旧ledger |

## 8. 执行步骤

6 work+1 review+3 merge=10项，全未勾。后续实施从T-01开始，最小whole+warm错误及parent链冻结后再全面透传；本轮不执行。S出口/公共H未释放只等具体依赖，不设D909/910整包前置。

| T | owner_stage | 前置/面 | 产物/验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D908正式基面；N与W-S/H | 冻结源/工具/全部cause/refusal/阶段出口、原资源cap及W-S1/2/API消费者；B17+NB17/非法warm最小链（warm部分先取窗），冻结mode/互斥schema/raw span与context/operation、唯一父报告落点、所有退出attempt及首失败策略；V-15/17/20，不二次编译取因 | AC-01..05 | [x] `3de08acad`：冻结文档 plan-artifacts/plan-911/T-01-freeze.md（出口清单/形状/caps/映射/attempt/落点）；`source_diagnostics.rs` + `MnError::Source`（legacy投影字节同）；`compile_escalating`保终止profile/attempted/surface；incremental warmup先mn单元编译取typed、`edit()`全出口`close_exit`+typed `EditRefusal`；AV `EditOutcome.rejection`（14站点kind/phase/typed ds/changed_fn）+ attempt计数先于真实调用；consumer：case-worker穷尽match、resources_tier双变体。测试 `mn_sd911_*` 6/6（cold B17/18/19 verify/code快照、NB17=2、record升级终止域、warm非法body typed Frontend拒绝+候选绑定+unaffected0+attempt可见+下一次修复ok、caps、7 stage投影）。组：ace-av PASS；ace-native 166/167（transient_spike_sampler内存采样器并发首红、隔离重放绿、本diff未触，环境归因）。909已合入→W窗全释放，V-20改为consumer编译+909组回归 |
| T-02 | work | T-01；N/W-S1/2具体稳定接口 | 操作级typed失败快照/真实终止profile/refusal kind、cold/warm/fallback所有出口和全返回计量；严格step首失败停，原default分类/display兼容，V-01..03/05/07..09/15..17；不改909语义/生命周期 | AC-01/03/05 | [x] (R1 重闭 `4cf907fb7`：rounds 单次自增、`operation.step`=请求内 0-based 索引、一次请求顶层事实亦记 step；worker 腿 7/7 B=step 1) 原证据 `a1b898fcf`：mn-worker `DiagMode::SourceV1`（未知值具名 `mn.worker.request`）；所有 ok 容器 echo `diagnostics_mode/schema`（合法程序亦 echo）；cold 形状②（顶层 `source_error` + legacy `diagnostic`、无执行结果）；incremental warmup 拒绝=typed 事实（原为 infra）；typed 拒绝**首失败即停**+`accepted_prefix{steps_applied,source_sha256}`，后续 step 不执行；syntax-not-converged 走严格逐候选 fallback（`MnDriver::check_source` 仅前端编译 0 codegen/link/run，首语义失败停，全过才一次整请求）；rounds 各出口同 echo/typed；default 路径字节形状不变（含"覆盖中间态"旧行为保留为兼容对照）。测试 `mn_sd911_worker_*` 7 腿真实二进制（cold strict/legal/default、未知 mode、A→B→C 首失败停：C 未执行/accepted_prefix=A 后源/snapshot 绑 B 候选、default 覆盖对照、warmup 两模式）；mn_sd911 13/13；incremental_tier 5/5（结构+P95）。V-15/16/17 native/worker 腿闭；V-01..03/05/07 parent 判面腿在 T-03/T-04 闭；syntax-fallback 严格门无端到端触发形状（代码路径+单元逻辑，T-05 具名 gap） |
| T-03 | work | T-01/02；contract/parent W-H | mode decoder/能力echo、原始JSON严格准入/互斥结果及事实artifact；先准入后比较，错context/operation/来源/旧worker/infra/超额/重复key/类型真实反控，V-04..07/12/14/15/18 | AC-01/02/05 | [x] (R1 重闭 `4cf907fb7`：`OperationExpectation` 携带父侧自身 step 并相等绑定，Cold 请求级须 null/rounds 内 cold 须等于回合索引；准入单测 15/15 含 step 错配拒；nd911 17/17 warm artifact step==1==parent B) 原证据 `b5c4bed46`：decoder `mn.diagnostics`（仅 scalar recipe、仅 "source-v1"、错值/错型/错 recipe 具名拒，省略=旧合同；`tests/mn_diagnostics_911` 过）；`ac_mn::source_diag` 严格准入先于 oracle 比较：**原始字节**重复键扫描（两种顺序）、精确字段集、schema/mode/basis 字面、profile+升级链真值、producer path、hex sha、causes 1..=64/message≤4KiB/payload≤64KiB、span 边界、primary=JSON 整数 0（"0"/0.0/false/1/null 拒）、仅 text/bind/verify 可承族、operation 按 parent 自身视图绑定（cold/warm-edit/fallback-cold base+candidate）、与执行结果互斥、legacy diagnostic 投影一致；mode echo 对每个严格响应强制（合法程序含）；raw stage 决定判面（text→parse/bind→resolve/verify→typecheck）、code=primary 原样；协议/事实失败=Error 非语言 FAIL；rounds 路径 typed 语言拒绝准入+**父侧首失败停**+源回滚 accepted prefix，非语言 typed 拒绝=Error；fact artifact `mn-source-diagnostic.json`（snapshot 原样+准入块绑 execution_key，写后重读）经 ArtifactRef 发布，不改 NativeRecord/test-report 键集。`source_diag_tests` 13/13 负控真实生效；`auto-test-case`/`ace-adapters` PASS |
| T-04 | work | T-02/03；三格/manifest/注册W-M1/H | 原B17..19 check受控迁移、NB/新suite/独立反控/必要固定组；父报告execution_key+原子fact/context/hash→三格token/note/manifest及正式入口真实重读，V-01..04/11/19，保909 K06 | AC-04/05 | [x] `b5c4bed46`：b-diags.case.ad B17/18/19 ac 腿迁移 strict source-v1 + 真实 raw code（av 腿/id/source 不变）；新长期套件 `native-diags-911`���12 case：3 负例引用 src/B17..19.at、3 近邻 src/NB17..19.at 2/2/0+echo、3 同族变体、多因 primary、Text→parse/Bind→resolve 异族）+ `nd911-src/` 夹具（src/ 冻结 81 不动）；固定组 `case-ace-m1-native-diags-911`；binding.tsv B17/B18/B19 ac 格 `diff:pending`→`ok`（任务列 +/911、备注记事实 artifact 证据，其余 9 列/128 行不动）；source-manifest.json 同步（b-diags hash、8 新文件、74 总、plan_911_update 注）。运行：新套件 12/12 PASS；`case-ace-m1-base` 97 检查迁移后 PASS；`nb-diags-908` 9/9 不变 PASS；fact artifact 读回 `execution_key=ace-m1-native-diags-911:<suite sha>:ace.m1.nd911.b17:ac-native-strict`、raw verify→typecheck、profile scalar、path mn.at、worker mnworker/1.0。909 K06/shared-mutable 行未触 |
| T-05 | work | T-01..04；稳定运行/计量与W收据 | V-01..20及真实生效反控、完整职责/必要消费者、非法warm全出口attempt/首失败/下一API修复与跨进程恢复、四状态30样本/P90/原预算/对象生产门；实际W-S/H/M1回交、接口再变STALE受影响重核 | AC-01..05 | [x] `a8dfd3380`/`2bd15c614`/`2acac4d81`：span utf8 char-boundary 核 + `source_len`==父候选字节 + 多字节注释正控 case（MB17.at）；artifact 写失败负控；**登记红控**（case-control-oracles.json expected FAIL）错族/错 stage/旧泛化壳三个，真实 mismatch 非 Error；**rounds 路径 fact artifact 发布**（自查发现遗漏修复）+ 值级互斥 + warm rounds A→B→C 腿（parent 在 B 停：receipt warmup+A+B，artifact warm-edit 绑 A 后源/B 候选/rev/编辑区间）；V-13 命名 `::performance::mn_sd911_source_v1_four_state_p90_30`（restart-warm P90 1770 / body-edit 1532 / signature-reject 243 ms ≤2000 门过；fresh 8703 ms 报告不门，default 对照 6755 同成本级）按 `ace-performance` 精确 flags PASS。组：nd911 17/17（14绿+3登记红）、base 97、nb-908 9/9、typed-891/collections-889/infra-ace-m1/infra-fixed PASS、shared-mutable-909 首红（%TEMP% 遗留 session store 非幂等，D-911-1）→清理重放 PASS、ace-host 170/172（turnaround p95=909 登记既有红 D-909-1；rounds liveness 并发首红→隔离重放绿）。W：909 已合入基面，全部 consumer 编译+组回归即 V-20 |
| T-06 | work | T-05 | 自查逐AC/V/SD/原源hash/实际H/残余gap/健康，真实命令/退出/数字折计划，冻结SD提案与提交/树clean→execution_done，不冒独立review/M1 | AC-01..05 | [x] `1e853fbf9`（SD-01..06 树内提案：新 `auto-ac/native-source-diagnostics.md` + 8 处 Spec 追加节）/`fa8dbc003`（健康：rounds gate driver 改惰性消除 default 唯一可观察差异；两处 `///`→`//`、死赋值、未读字段入 artifact；触及文件零新增 warning；clippy 三条为既有 lint 行号平移）。原源 SHA：B17/B18/B19/binding 保 §4.2 值（binding 仅 3 格+备注变，§4.2 原 SHA 为迁移前基线，新 SHA 入 manifest）。AC-01..05 逐项见 §9 work 回执；残余具名 gap D-911-1/2/3 登记 KNOWN-DEBT。树 clean（仅 plan827 测试副产物未跟踪，D-911-2）。**不冒独立 review、不签 M1** |
| T-07 | review | T-06冻结提交；同树独立上下文 | 逐机制/数据流/判据/来源/反控/未执行及遗漏延期独立核，修复重绑身份，pass→reviewed | AC-06 | [x] R1 needs_fix（F-911-R1-01 step 轴）→ 修复 `4cf907fb7` → R2 final **pass**（同会话 artifact-based review，自选反例 A/A2/B/D + E 代码核 + 受影响重跑；见 §9 review R1/R2） |
| T-08 | merge | T-07 pass | 冻结完整受影响职责/必要消费者门或有据同身份证据；漂移仅重核受影响；cargo t all按due主检出单实例/既有用户择时，不造每计划全局门 | AC-07 | [x] rebase `plan-911-dev` 4cf907fb7→**`ca4848f5d`**（onto master `5923d4dee`=910 归档，`range-diff` 9/9 `=`，零文件重叠）；先清 `%TEMP%/av798-session-av_closures_909_*`；冻结并集一次调用 14 组：**12 PASS**（auto-test-case / ace-adapters / infra-fixed / infra-ace-m1 / nd911 / base / nb-908 / typed-891 / collections-889 / shared-mutable-909 / ace-native-worker / ace-av），ace-native 174/175、ace-host 170/172——三红全为 `::performance::` 时序硬门：① 本包 V-13 腿（并集下 restart-warm P90 3407ms；机器残留负载时隔离红一次=PE 5s run-budget 超限表现为 exit None；安静单实例在 ca4848f5d **PASS 435s**，累计 4 次安静绿）→ D-911-4；② `abi_recipe_formal_turnaround_hard_gates` 隔离 **PASS** 49s（负载归因，非本 diff 触面）；③ `turnaround_native_six_scenarios_p95_gates` 隔离仍红 = master 已知 **D-909-1**。`cargo t all`：due（905/910 %5==0 已落）→ 沿 910 收据 user-pending 登记，不在计划树内跑。日志 `frozen-union.log`/`isolated-replays.log`/`perf-fail3.log` 摘要折本行 |
| T-09 | merge | T-08 | 合入、SD-01..06/代码来源身份读回、M1状态与909并行接口/残余后继沉淀、archive delivered，不签M1 | AC-07 | [x] **landed**：主检出 `git merge --ff-only plan-911-dev` → master tip **`ca4848f5d`**（无 merge commit）。**canonical_verified**（master blob hash 前 12）：native-source-diagnostics.md `7e8e9d2e3537`（新）/ mir-native-scalar `badd9f083521` / mir-native-cache `4b1ba334a299` / auto-ac project `5d1b68db4021` / resident-function-install `7a1ebc265b0b` / profile-v2-native `2fa6f2e909c4` / auto-test-case project `9b79232d5fae` / ace-m1-conformance `cfcf0e99a5d7` / ace-m1-gap-audit `bed874142f3b`；源身份 source_diagnostics.rs `b6617a1944c0`、ac_mn.rs `654538e35553`、binding.tsv `a54c4a11594b`。M1 未签章；909 接口按交付基面单写无并行窗。**archived**：本文件 `git mv` → `docs/plans/archive/`，status archived，与 KNOWN-DEBT 同提交落 master |
| T-10 | merge | T-09 | 摘要/hash/所有实际输出/hold对账；fresh wt-guard clean后清唯一树/分支/组/整个材料scope，刷新登记，真实10/10 | AC-07 | [x] **cleaned**：交付 `ca4848f5d` 为 master 祖先核；树 clean（0 行）；fresh `wt-guard.sh` clean → `git worktree remove` + `git branch -d plan-911-dev`（was ca4848f5d）+ `rmdir .wt/lang-911`（组内无兄弟）；`git worktree list` 无 lang-911。材料 scope `plan-artifacts/plan-911`（344K/38 文件：探针脚本、组日志、T-01 冻结文档）整删，registry `auto-lang/PLAN-911` → archived/cleaned、无 holds。输出路径对账：计划命令仅用 worktree 内 `target/`（随树删）、`%TEMP%/mn911-*`/`rv911-*`（测试自清）、`%TEMP%/av798-session-av_closures_909_*`（909 套件产物，已清，D-911-1）、树内 `docs/reports/plan796|818|827/*.json` 测试回写（恢复未入提交，D-911-2）；无散落外置目录。**落地观察**：主检出无 release `auto.exe`；debug `auto-case.exe` 03:27 早于交付 08:53——主检出工具链未重建（ACE 实验通道无生产进程消费；完整冷编译为用户 2026-10-30 裁定问题项，登记不重建）。批量回归 due（905/910 已落）沿 910 收据 **user-pending**。真实 **10/10** |

唯一拟树`D:/autostack/.wt/lang-911/auto-lang`、分支`plan-911-dev`，本轮未建。开工登记`D:/autostack/plan-artifacts/plan-911/` owner/用途/在途；探针/日志/JSON报告/变异副本/发行或外部target只在此scope，长期case/oracle入其职责路径。work/自查/独立review同树顺序。merge摘要折计划，核全部实际输出目录后整个scope/树产物删除；具名未解决问题才hold，不碰他方目录，不写attachments/散落证据/legacy specs.json。

## 9. 复审记录

### r1立项历史（当前合同以r2及上方§5–8为准）

2026-11-01 | stage:new | PLAN-911 r1 | outcome:pass（规划合同预检，非功能/执行/独立review PASS） | 用户要求规划909执行后的下一包；选择现有native诊断观察三格，避开909共享捕获/AV生命周期主体。D908 delivery dc8eca3ae祖先0、已归档12/12；core247/285/38阻塞等按其冻结Spec引用，本轮未重跑。

取号前主检出持有docs/plans/.allocation.lock独占文件句柄，核活动/归档最大号910（他方Cargo治理已分配）/.next-id=911，经scripts/new-plan.sh唯一分配911、下一号912；未覆盖910或另取号。909用户确认已执行，实际树已观察，header滞后不代改；本包只读相关Spec/源，不消费其WIP/联系会话。

本轮drafting r1、0/10，仅计划/Design43 §15/导航簿记；无构建/语言测试/探针/认证/材料目录/实施树/agent/chat，未改生产/current Spec/原源/expected/binding/golden/cert/909合同。next: 后续用户要求实施911时T-01，N/parent可与909并行、S只读出口与H按§5.6短窗；所有实际V/AC证据待执行。

规划静态预检：章节0..10、T10/AC7/V14/SD6唯一且引用均有定义，10项任务全未勾；Plan/Design43 §15及本轮导航新增链接/锚点与空白检查通过，现有Spec目标及所列固定组实存。B17/B18/B19原源与binding的SHA256保持§4.2值，D908祖先核验0；911活动/归档唯一、取号后.next-id=912，911实施树/材料scope不存在。该检查只证明规划结构/范围与来源一致，不是实际语言/协议/性能/最终review或M1 PASS。

### r2规划复核与直接修订（2026-11-01，当前入口）

stage:new | PLAN-911 r1→r2 | outcome:pass（规划合同修订，非功能/实际final review PASS） | authority: 用户要求重点复核四类合同，有遗漏直接修订且仅规划；原三格目标/已批准G-DEC/原source和行为oracle不变，任务仍10项/0完成。V由14补至20，AC7/SD6保稳定ID并加严关闭点。

| 规划遗漏/歧义 | r2修订及承担任务 | 关闭验证（均待实施） |
|---|---|---|
| 因果快照若迟读session last-errors，profile/context/operation可能错绑 | §5.1/5.2/5.3固定同操作不可变Err快照、实际终止域/primary、accepted-prefix/上下文与Raw坐标；T-01/02 | V-15及V-06/07/08 |
| 字段齐不等父侧信任，重复key已被Map抹平/矛盾ok/类型隐式转换可绕过 | §5.2先原始字节准入，互斥结果/整数0/重复对象键/原工具来源/ERROR与oracle mismatch分开；T-03 | V-18及V-05/06 |
| 现行fallback读字符串/物化tail，失败早退Default零统计不证明零生产 | §5.4区分失败phase、typed refusal/首失败停、真实attempt全return取证及下一API/进程恢复；T-01/02/05 | V-16/17及V-08..10/13/14 |
| r1含type warm没有区分scalar worker与独立owned API，内部转发还会漏新mode | §5.3入口矩阵明确已有源域/路由和兼容边界，不偷扩新worker；T-01/02/03冻结/透传policy和context | V-08/12/15/16 |
| raw ok/三格token与artifact若无父execution_key/发布读回，可拿旧证据充新 | §5.5固定一种报告落点、原子证据与完整绑定/正式入口重读；T-04 | V-19及V-11 |
| “默认909先让出”未获交接、Rust加字段不是天然兼容 | §5.7具名W-S1/2/N/H/M1，列实际construct/match/consumer与主线/稳定阶段SHA；接受顺序/再漂移STALE；T-01/05 | V-20及V-12 |

主线仍367258e7a；只读已确认909树，未修改其Plan/代码/进度或请求其让窗；910已读取drafting0/9的实际合同，职责为生成UI/Web工程/Cargo配置与可重建缓存生命周期，非ACE语义前置。无新号/树/材料/构建/语言测试/探针/认证/agent/chat，本轮仅911/Design43 §15和导航文档，current Spec/源/binding/expected/golden/cert不动。

next: 后续实施从r2 T-01开始，N/parent可先行，依赖typed出口/真实计量的warm部分取W-S1/2稳定接口和公共H窗后完成；不得把必需门延后给909或重复请求既有批准。实际V-01..20及AC仍待实施/独立复审。

r2静态预检：章节0..10齐，T10/AC7/V20/SD6唯一且引用均定义，10项任务全未勾；本轮Plan/Design43 §15/索引与路线新增引用/锚点/空白检查通过，既有Spec目标实存。原B17/B18/B19与binding SHA保持§4.2值；生产/current Spec/测试脚本无主检出diff。911号唯一、无实施树/材料scope，本轮未取号，.next-id仍912。该结果只证明规划结构/来源/范围一致，不证明语言、worker协议、故障/性能或最终review通过；未修改909/910计划与进度。

### work 回执（2026-11-02，T-01..T-06）

stage: work | plan_id: PLAN-911 | plan_revision: r2 | outcome: **pass** | worktree_path: `D:/autostack/.wt/lang-911/auto-lang` | branch: `plan-911-dev` | base_commit: `5a4eb4e25`（master = 909 归档） | code_commit: **`fa8dbc003`**（分支 9 提交：3de08acad / a1b898fcf / b5c4bed46 / a8dfd3380 / 2bd15c614 / 2acac4d81 / 1e853fbf9 / fa8dbc003 + 1 amend） | dependency_revisions: ace workspace 同树；auto-down 未触 | task_ids: T-01..T-06 (6/10) | blockers: 无 | next_responsible_role: **review（T-07，同树独立上下文）**

**AC 自查（工作阶段，非独立 review）**
- AC-01（typed 快照/终止 profile/顺序/primary/不迟读/不重编译）：`source_diagnostics.rs` + `compile_escalating` 终止域 + warmup 先 mn 单元编译；`mn_sd911_cold_negative_snapshots`（B17/18/19 raw verify/code、attempted/surface、JSON 往返、legacy 投影字节）、`*_escalation_names_terminating_profile`（record 终止 + scalar surface 记录）、`*_warm_illegal_body_edit_typed_refusal`（候选绑定、base/rev/编辑区间）。
- AC-02（显式 mode/echo/互斥/原始 JSON 准入先于比较）：worker `DiagMode` + 每 ok echo；父 `source_diag::admit`（原始字节重复键两序、精确字段、caps、span 边界+char boundary、primary 整数 0、stage 族门、operation 父侧绑定、值级互斥、legacy 投影）15/15 单元；decoder `mn.diagnostics` 测试；登记红控三个真实 mismatch；nb 近邻严格 echo 2/2/0。
- AC-03（源码拒绝零后段/首失败停/accepted-prefix/全出口 attempt/未改 IO 0/下一操作修复/原门）：cold 拒绝 codegen/link/run=0 断言；`edit()` 全出口 `close_exit`（failed edit unaffected=0、body_work attempt 可见——Default 零早退会令该断言红即为反控）；worker A→B→C C 未执行 + accepted_prefix；parent rounds 在 B 停 + 回滚；下一次独立修复 ok（native 测试 exit 7）；restart/hit 由既有 cache/turnaround/incremental_tier 组（PASS）；V-13 四状态门过。
- AC-04（三格仅据正式父报告 fact 回填）：b-diags 三 ac 腿迁移真实 raw code；nd911 套件 17；fact artifact 读回（execution_key/raw verify/profile/path/worker）；binding.tsv 3 格 ok（其余列/行不动）；manifest 同步；base 97/nb-908 9 不变。
- AC-05（默认/v1/typed/collections/909 消费者兼容；工具元数据；W 回交；SD；无 warning/调试输出）：default worker 字节形状测试（cold/incremental/warmup）；typed-891/collections-889/shared-mutable-909 组 PASS；`OwnedSourceMnSession` 未触（owned_text 测试在 ace-native 组内 PASS）；全部 `EditOutcome`/`MnError` consumer 编译+组回归；SD-01..06 树内；健康清。

**V 账（20）**：V-01..03 ✓（native/worker/case/base 四层）；V-04 ✓；V-05 ✓（Text/Bind 异族 case + 非族 stage 准入拒 + 红控错族）；V-06 ✓（15 单元含 caps/span/utf8/未知字段）；V-07 ✓（多因 primary=0、换 path 拒、登记红控错 expected）；V-08 ✓（候选/base/step/span 绑定、unaffected 0、owned API 保原）；V-09 △（typed kind 路由不读文本 ✓、语言诊断不进 fallback ✓；**无端到端 syntax-not-converged 触发形状**→D-911-3）；V-10 ✓（下一次修复、hit/restart 既有组、拒绝不缓存）；V-11 ✓；V-12 ✓（default/typed/collections/decoder 错 recipe 拒）；V-13 ✓（门状态过；fresh 报告口径见 KNOWN-DEBT 注记）；V-14 ✓（截断/半响应准入拒、既有 fault fixtures 默认路径、后续请求真通过）；V-15 ✓；V-16 ✓（worker 停 + parent 停；fallback 半边同 V-09）；V-17 ✓；V-18 ✓；V-19 ✓（artifact 绑定/写失败/重读；rounds 路径补发布）；V-20 ✓（909 基面 consumer 验证替代窗口）。

**组账（冻结提交 fa8dbc003 之前各单元运行；merge T-08 冻结并集重跑）**：ace-av PASS；ace-native 166/167（transient_spike 内存采样器并发首红→隔离绿，环境）；ace-native-worker PASS；ace-adapters PASS；auto-test-case PASS；case-ace-m1-base PASS(97)；case-ace-m1-nb-diags-908 PASS(9)；case-ace-m1-native-diags-911 PASS(17=14+3红)；case-ace-m1-typed-native-891 / native-collections-889 / infra-ace-m1 / infra-fixed PASS；case-ace-m1-shared-mutable-909 首红（D-911-1 环境）→清理后 PASS；ace-host 170/172（D-909-1 既有 + rounds liveness 并发首红→隔离绿）；ace-performance 命名 case 精确 flags PASS（1/1）。原始日志在 `D:/autostack/plan-artifacts/plan-911/*.log`。

**残余（具名，非延后）**：D-911-1（909 套件 %TEMP% 非幂等）、D-911-2（AV 性能腿写仓内 plan827）、D-911-3（syntax fallback 无触发形状）——见 KNOWN-DEBT-AND-RISKS.md Plan 911 节。M1 未签章。

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 1 | from: drafting | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-911/auto-lang | commit: fa8dbc003 | next: review

### review R1（2026-11-02，final，needs_fix）

stage: review | review_scope: final | reviewer_identity/limitations: **同会话 artifact-based review**（实施会话内，无独立模型/会话；按计划 §6 不创建其他 agent；裁决自树内提交/真实二进制/报告与 artifact 文件重建，自选反例） | plan_id: PLAN-911 | plan_revision: r2 | outcome: **needs_fix** | worktree_path: `D:/autostack/.wt/lang-911/auto-lang` | branch: `plan-911-dev` | reviewed_commit: `fa8dbc003` | base_commit: `5a4eb4e25`（master 已前进至 `5923d4dee`=910 归档，与本分支零文件重叠） | dependency_revisions: ace workspace 同树 | spec_inputs: 9 个 `docs/specs` 文件 diff（3-dot 核对与 frontmatter `supersedes`×8 + `new`×1 完全一致） | acceptance_results: AC-01 pass（复审自选新非法源 `let z bool = 3` 走真实 worker：typed verify/source.type-mismatch、primary int 0、无执行结果、sha 绑定、profile scalar/attempted[scalar]）；AC-02 **partial**（F-911-R1-01）；AC-03 pass（E：一次请求 fallback 每候选一次 `check_source`+最终一次 request；rounds 末候选双编译已在 SD-01 具名）；AC-04 pass（binding 131 行×12 列仅 B17/18/19 三行变；nd911 在 fa8dbc003 重跑 17/17；warm artifact base/candidate SHA 绑定核）；AC-05 pass（默认响应对非法/合法源**零新键泄漏**；D-911-1/2/3 登记） | pending_closeout_gates: 冻结受影响组并集（merge T-08，须在 rebase 至 master 后）；`cargo t all` 按 due | findings: **F-911-R1-01（P2，原合同缺口，AC-02/V-06/V-08；T-02+T-03）** `mn-worker.rs:1081` 残留 `round_no += 1`（成功编辑路径二次自增→B 的 `operation.step` 报 3 而非 2）；父侧 `source_diag::admit` warm/fallback-cold 仅核 `step` 为整数、未按冻结 §3.6#6 绑定 parent 自身回合索引。复现：nd911 `warm-reject-stops` artifact `operation.step=3`、receipt 步骤 warmup+A+B。违反规则：§5.2 "实际候选/step 身份…不得跨 step 重放"。关闭 oracle：删除重复自增；admit 接 `expected_step`（parent 1-based 回合序）并相等核；worker A→B 测试断言 step==2；父侧单元新增 step 错配拒；受影响重跑 mn_sd911_worker/admission/nd911。边界：候选/base SHA 绑定本已正确，step 为补充身份轴 | evidence: `plan-artifacts/plan-911/review_probes.py` 输出（A/A2/B/D）、`review-nd911.log`、树内 `docs/reports/*` 测试副产物 8 文件已恢复（D-911-2 同类，未入提交） | next: **work**（修复 F-911-R1-01，重闭 T-02/T-03，再提交最终复审）

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 2 | from: execution_done | to: executing | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-911/auto-lang | commit: fa8dbc003 | next: work

### work 修复回执 R1（2026-11-02）

stage: work | plan_id: PLAN-911 | plan_revision: r2 | outcome: pass（修复 F-911-R1-01，重闭 T-02/T-03） | worktree_path: `D:/autostack/.wt/lang-911/auto-lang` | branch: `plan-911-dev` | base_commit: `5a4eb4e25` | code_commit: **`4cf907fb7`** | dependency_revisions: 同树 | task_ids: T-02, T-03 | evidence: 闭合行 `F-911-R1-01 → 不变量「operation.step = 父侧可核的请求内 0-based 步骤索引」→ 受影响路径 rounds（双自增）/一次请求顶层事实（未设 step）/父侧 admit（仅核存在） → 修复：worker 单点 `step=round_no; round_no+=1`、一次请求 stop 处 `snap.operation.step=Some(st_idx)`、admit 三变体携带父侧 step 相等核 + 请求级 cold 须 null → 永久用例：准入 `mn911_admit_warm_edit_binding` 新增 step 错配拒、worker `*_strict_first_failure_stops` 断言 B step==1、套件 `warm-reject-stops` artifact step==1 → 结果：15/15、7/7、17/17 → 剩余缺口：无`；SD-01 文字补 0-based 语义 | blockers: 无 | next_responsible_role: review（受影响 AC-02 重核）

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 3 | from: executing | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-911/auto-lang | commit: 4cf907fb7 | next: review

### review R2（2026-11-02，final，pass）

stage: review | review_scope: final（R1 修复重核：受影响 AC-02/V-06/V-08 重验；其余 AC-01/03/04/05 证据在 fa8dbc003→4cf907fb7 间实现/配置/输入未变（diff 仅 step 轴 + 测试 + SD-01 措辞）复用） | reviewer_identity/limitations: 同会话 artifact-based review（同 R1 声明） | plan_id: PLAN-911 | plan_revision: r2 | outcome: **pass** | worktree_path: `D:/autostack/.wt/lang-911/auto-lang` | branch: `plan-911-dev` | reviewed_commit: **`4cf907fb7`** | base_commit: `5a4eb4e25`（master 现 `5923d4dee`，零重叠） | dependency_revisions: 同树 | spec_inputs: 9 Spec 文件（SD-01 本轮增 step 语义一行；frontmatter 影响字段不变） | acceptance_results: AC-01 pass（复用 R1 自选反例 A）；AC-02 **pass**（F-911-R1-01 关闭：复现输入 nd911 warm 腿 artifact step 由 3→1==parent 索引；准入 step 错配控制真实拒；一次请求顶层事��� step=1；Cold 请求级 null 核保持）；AC-03 pass（复用）；AC-04 pass（nd911 在 4cf907fb7 重跑 17/17；binding/manifest 未变）；AC-05 pass（复用；默认路径 diff 未触）；AC-06（本复审）pass | pending_closeout_gates: **merge 持有**——rebase 至 master `5923d4dee` 后冻结受影响并集（ace-av / ace-native / ace-native-worker / ace-adapters / auto-test-case / case-ace-m1-base / nb-diags-908 / native-diags-911 / typed-891 / collections-889 / shared-mutable-909（先清 %TEMP% 残留 D-911-1）/ infra-ace-m1 / infra-fixed / ace-host（已知 D-909-1）/ ace-performance 命名 case）；`cargo t all` 按 due（911%5≠0，由批量回归承接） | findings: 无合同内未闭；具名残余 D-911-1/2/3 登记 KNOWN-DEBT（非阻塞） | evidence: `plan-artifacts/plan-911/nd911-r1-final.log`、`review_probes.py` 输出；SD 冻结 hash 由 merge 读回登记 | next: **merge**

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 4 | from: execution_done | to: reviewed | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-911/auto-lang | commit: 4cf907fb7 | next: merge

### merge 收据 PLAN-911:r2（2026-11-02）

stage: merge | plan_id: PLAN-911 | plan_revision: r2 | completion_kind: **delivered** | reviewed_commit: `4cf907fb7`（rebase 前）→ delivery_commit: **`ca4848f5d`**（range-diff 9/9 等价） | base: `5923d4dee` | checkpoints: **prepared** ✓（reviewed+R2 pass、SD 增量冻结、源目标核）→ **landed** ✓（master==ca4848f5d ff-only）→ **canonical_verified** ✓（9 路径 hash 见 T-09）→ **archived** ✓（本��交）→ cleaned：见下一收据 | gates: 冻结并集见 T-08（12 PASS + 3 具名时序红：D-911-4 本腿负载/安静单实例绿、abi 隔离绿、D-909-1 既有）；`cargo t all` due user-pending（沿 910） | 残余：D-911-1..4（KNOWN-DEBT Plan 911 节）；M1 不签 | next: cleanup（T-10）

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 5 | from: reviewed | to: archived | status: archived | outcome: pass（cleanup pending） | worktree: D:/autostack/.wt/lang-911/auto-lang | commit: ca4848f5d | next: merge-cleanup

### merge 收据 PLAN-911:r2 — cleaned（2026-11-02）

stage: merge | plan_id: PLAN-911 | plan_revision: r2 | outcome: **pass** | completion_kind: delivered | delivery_commit: `ca4848f5d`（master ff-only）| archive_commit: `be54108e9` | canonical: 9 Spec 路径与 hash 见 T-09 | archive_path: `docs/plans/archive/911-native-source-diagnostics-and-m1-b17-b19.md` | cleanup: worktree/分支/组目录已移除（guard clean）、scope plan-911 整删、registry 刷新、无 holds | pending merge-owned: `cargo t all` 批量回归 due → user-pending（沿 905/910 用户裁定语境，不在计划树内跑）；D-911-4 perf 腿并集并行调度治理（候选后继） | landing observation: 主检出工具链未重建（见 T-10）| 10/10

stage: goal | plan_id: PLAN-911 | plan_revision: r2 | round: 5 | from: archived | to: archived | status: archived | outcome: pass | worktree: removed | commit: ca4848f5d | next: done

## 10. 待澄清事项

已批准语义族/规则继承，无新增用户政策选择待批；以下为工作阶段有界接口/机制冻结项，不能作为重新索取G-DEC批准的理由。

| D | owner/入口 | 当前边界与关闭条件 |
|---|---|---|
| D-01 精确数据形状 | T-01；911 | source-v1/能力echo/互斥source-diagnostic/1、operation/context/span基准、唯一报告落点在最小链冻结；不造全局报告/runner，旧缺因/矛盾结果不授信 |
| D-02 AV只读事实出口 | T-01/02；W-S1/2与909 | 同操作typed快照/拒绝kind/真实attempt从原生产点保留，不从last-errors迟读，不改提交/依赖/捕获；列Rust全部构造/consumer并取稳定窗口，需语义平台改动则needs_replan |
| D-03 全部出口/阶段 | T-01/02；911 | 列cold/steps/fallback/record-text/每个early return及默认其他recipe；首失败停、typed refusal与source错误分态，后段失败实际工作不假报0 |
| D-04 工具与资源 | T-01/03；911，910若真实相交 | 原cap更严值、先限额后额外物化/原始JSON与artifact发布预算冻结；实际worker身份/新旧元数据按原STALE，不借910草稿改变工具或索重复批准 |
| D-05 M1残余 | ACE路线；各能力owner | 本包只认B17–19 native观察面，K06由909；旧a2r/B18 avm/scenario/AV资源及future/cert账保留，后继按真实净缺口规划，不预占号或签M1 |
