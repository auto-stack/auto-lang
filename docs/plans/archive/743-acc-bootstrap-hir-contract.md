---
plan_id: PLAN-743
status: archived
feature_name: ACC 自举能力盘点与 HIR 阶段契约
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-04
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-acc/project.md
  - docs/specs/auto-hir/stage-contract.md
touched_goals: []
affects: [auto-acc, auto-hir]
---

# [PLAN-743] ACC 自举能力盘点与 HIR 阶段契约

## 0. 变更摘要

AC/ACC 主线第二个计划，承接已交付的 PLAN-741。
742 已由 api-handler-body-parity 独立开发线领取，本计划采用仓库中央下一编号743。

把战略 NEXT-A 转为可执行合同：盘点现有 Auto 自举源码真正使用的语言/运行时能力，
区分可复用组件与须新增/重写的 ACC 主体；明确公共 HIR 的阶段和 pass 边界，
给源码 adapter、运行时、ABI 与 ACC 迁移提供有证据的前置条件。

成果是可重跑的盘点工具、审定清单、架构决策和验收矩阵。不是 ACC 实现或自举完成声明。
本计划不修改 canonical Specs；执行期提交 proposed delta，独立复审通过后由 merge 沉淀。

## 1. 目标

1. 建立有来源/hash/证据的 ACC 主体及语言能力清单，回答“编译器自己需要 AC 支持什么”。
2. 逐模块决定复用、适配、重写、参考或非主体角色；不得把 AAVM/AA2R 的既有自举等同 AC/ACC 自举。
3. 确定前端工作态、Checked 公共 HIR、native 降级和后端工具桥的职责；
   将阶段不变量、pass 前后置条件、分析失效、来源映射和能力检查落实为可评审契约。
4. 建立源码编译、代表模块及原生自编译代际的验收矩阵，收敛最近两张候选实施合同，
   按真实缺口重新核查26–36个计划的估算，不提前占用后续编号。

范围：仅 auto-lang 仓，新增独立调研工具/报告/设计文档及本计划簿记。
前置：741已归档交付，其 auto-hir/auto-ac Specs 和实验 API 为事实基线。
非目标：修改旧 parser/typeck/VM/A2X、auto/lib/*.at、生产 Cargo workspace/lock，
扩大741 profile，实现完整 Atom/Batom/Schema、优化器/自主SSA、生产ABI/DLL热重载、
ACC桥接实现、完成源码→native 或 ACC 自举。外仓只列依赖，不写入、不新建外仓worktree。

## 2. 架构方案

~~~text
固定输入快照 / source manifest
  → 保守源码扫描 + 人工语义复核
  → ACC主体/语言能力/运行时服务清单
  → 迁移与依赖决策、HIR阶段/pass契约
  → 代表程序和代际自举验收矩阵
  → 最近实施工作包合同候选 + proposed Spec delta
~~~

两个维度分开：工具的词法观察 ≠ 已完成语义解析；项目过去支持某能力 ≠ AC原生已支持。
所有可疑调用、接收者和间接依赖保留unknown，交人工核对，不以“扫描没找到”证明不使用。

公共 HIR 不强制SSA。Cranelift桥是工具内部边界，生产模块ABI是另一条合同。
v0.6继续用Cranelift；自研低层IR/机器码研究不进入本计划。

## 3. 技术栈

- 盘点工具：Python标准库，支持本机Python3.14及3.11+；不下载包、不调用网络。
- 输出：稳定JSON manifest及Markdown报告；UTF-8、仓库相对路径、SHA-256输入指纹。
- 参考：独立 experimental/ac-core Rust模型/descriptor/verify，Cranelift0.126.2。
- 实施工作树（新建，执行阶段才创建）：
  D:/autostack/.wt/lang-743/auto-lang，branch plan-743-dev，基于届时完整v0.6-dev。
  不以master为默认落点；用户已将本主线安排在主检出的v0.6-dev。
  禁止junction/symlink，不需要外仓依赖或完整编译。

## 4. 需求分析与背景调查

### 4.1 授权与版本

用户2026-10-04要求“继续规划第二个计划”，授权起草与提交计划；
未授权在本轮实施，未指定预算或自动连续执行范围。
此前已确认v0.6 AC/ACC通过Cranelift完成约定主体原生自举，最近优先独立新工作。
本次具体合同保持drafting，执行仍需用户指令；不把战略确认当作本计划执行批准。

调查主检出：D:/autostack/auto-lang，branch v0.6-dev。
调查HEAD：a6c0f9691dbb8e176b028291de758bbffa499fe3；
741归档合入收据指向304519113（含Spec/ledger沉淀），742已存在，.next-id=743。
执行前须重取HEAD/hash并列差异；主机器未push源码不能从该快照推断。
下一段是来源指纹而非要求执行永远停留在此提交。

| 来源 | 本次SHA-256 |
|---|---|
| docs/specs/auto-hir/project.md | 04e95f7ce4805b7e409bf89af49fe9c43be0632157043470099e76e1fe83bee6 |
| docs/specs/auto-ac/project.md | 31f20c811950600a1afcdb5fb05a8199713188e7e3b54e1872dbd336f9870b17 |
| docs/specs/aavm/project.md | 79f913d0ea673fb85e6db2d50bb6e7aefd354855183db12b0089a57187aca5be |
| docs/design/strategy/auto-native-backend-evolution.md | becb2f45d66bffc81b98bea33c3c6c0bb40418be2f958cd1b9a58de283a5be27 |
| docs/design/strategy/auto-hir-design.md | c51caa7dd454010904c1f2055f57539ce01343ee5e7fee1fe4613e38e8c3ba0d |
| experimental/ac-core/src/hir.rs | 251d973f75a991675c7331bbc9dd683e61b7e830a9615120cd5cd6d5c6f29e63 |
| experimental/ac-core/src/verify.rs | 1ddc9cf6669ead02948222347e009ec8922a6183271dda178b43e21be5dc0ae2 |

其他已读：docs/specs/overview.md、auto/lib/{token,lexer,parser,typeinfo,codegen,engine,a2r}.at、
auto/lib/README.md、auto/aavm.at、crates/auto-lang/src/lib.rs的AUTO_LIB_FILES_V2、
scripts/{aavm_lib_xref,aavm_shim_inventory}.py。执行时manifest须记录全量输入hash。

### 4.2 现状与缺口

- auto-hir已有core-i32-draft：绑定后的类别/owner引用、精确i32/bool、有限运算、
  CheckedModule私有构造、verify入口与来源诊断；尚无完整源码adapter/公共pass框架。
- auto-ac已有Checked HIR→Windows COFF→PE；没有源码→native、生产模块ABI或ACC自举。
- auto/lib/parser.at主要直接生成S-expression；typeinfo.at以Display字符串做轻量型推断，
  支持unknown且不是完整独立typeck，不能替代公共Checked HIR的有效性凭证。
- auto/lib/codegen.at发ABC，engine.at执行ABC，a2r.at是token游标驱动的转译路径；
  各自职责不能被“Auto写的编译器”这一统称掩盖。
- token/lexer/parser实际使用enum、record、方法、List<Token>与嵌套List、str等；
  741的i32/bool不足以编译它们。迁移所需精确能力/所有权/运行时还需系统盘点。
- scripts/aavm_lib_xref.py已有启发式交叉引用；不能当完整名字解析器。
  aavm_shim_inventory.py扫描Rust源，不能直接代替Auto主体清单。
- canonical Specs尚无auto-acc主体合同与独立stage-contract，必须在本计划提出增量。
  GOAL-017是旧AAVM/AA2R已达成目标，本计划不改写或重开它；
  touched_goals为空，native自举目标若要入账另经目标治理，不自行冒用旧达成结论。
- AAVM语料实际根为crates/auto-lang/test/vm/aavm2/；
  旧Spec部分命令里的test/vm/aavm2仅是crate内相对路径。

## 5. 详细设计

### 5.1 可复现清单与工具边界

新增 scripts/acc_inventory.py 及 scripts/tests/test_acc_inventory.py、
scripts/tests/fixtures/acc-inventory/。不改老盘点工具。
输入根显式 --root；输出根 --output，拒绝将报告输出写入输入源码目录。
初始输入覆盖七个lib文件、auto/aavm.at、lib.rs的文件注册段及人工列出的关联运行时证据。
发现新的use/import/运行时依赖须纳入manifest或以external/unknown逐条解释，
禁止忽略缺失文件后退出成功；检测到注册表变更或输入hash变更，--check必须返回非零。

拟定命令（本计划新建接口，非声称已存在）：
~~~powershell
python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --write
python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check
python -m unittest discover -s scripts/tests -p test_acc_inventory.py
~~~

manifest至少记录：format版本、来源身份、相对路径/hash、声明与方法候选、
显式use边、语言能力及证据位置、运行时调用候选/unknown、人工结论及其证据。
相同输入与结论重跑得到字节一致的扫描制品；时间/机器路径不混入比较数据。
记录当前HEAD供审计，但验证以受管输入hash为准，避免仅提交报告导致HEAD变化误报。
手工审定与生成观察分开存储；重跑不覆盖人工判定，过期判定需失效提示。
多行注释、字符串内伪代码、同名方法、动态/未解析接收者必须有保护或明确未知标记。
工具不能以扫描统计做语义编译或穷举正确性承诺。

### 5.2 主体、能力与迁移决策

新增 docs/design/strategy/auto-acc-bootstrap-contract.md，并在docs/design/00-intro.md登记。
七个lib模块及CLI逐一记录：现有职责、真实入口/证据、复用/适配/重写/参考/非主体、
迁移原因、前置能力、代表语料。此处不预先决定所有模块必须搬迁。

拟定ACC主体清单还须含目前不存在的解析/类型/HIR构造与校验、
语义lowering、驱动/诊断职责，以“新建”标注，不能从旧lib名册自动推导它们已存在。
engine/ABC发射/AA2R可作为参考或测试依赖，但不能据此声称native主体迁移已完成。
保留Rust Cranelift桥、运行时、系统linker及构建工具清单；解释哪些服务不算主体。

每个能力用独立capability ID，区分compiler-source-demand与compiled-language-support，
即“编译器自身用到的语言”与“编译器能编译的语言”。
覆盖数值宽度/int映射、字符串/字符索引、容器、record/enum/模式、
方法/泛型、全局/多模块、别名/移动/释放、I/O/进程/错误。
状态仅可为implemented/required/unknown/not-required（附证据），
不能把旧VM/A2R绿当AC implemented；有界i32溢出70与完整异常ABI分开。
无法裁定的语义不强行冻结：记录owner、最小探针、所在后续工作包和受阻验收。

### 5.3 HIR阶段与pass契约

契约明确分层，不要求为每层新建一套HIR：
前端工作态（未解析/推导中）→ bound/typed候选 → semantic verify →
Checked公共HIR → 显式profile-specific lowering/目标能力门 → Cranelift adapter。
既有741 bind+verify路径须准确映射，不能宣称已有完整源码类型推导或优化pass。

每阶段列：合法输入/输出、允许未决项、schema/profile/revision、引用/owner、
来源映射、语义/效果/资源义务、验证入口、失败诊断和消费者。
Checked层禁止Error/Unknown/悬空引用以成功状态通过；
显式Dynamic若未来支持必须有独立语义，不与推导失败混同。
公共HIR不泄漏CLIF、寄存器或桥接私有布局，泛型/ownership等完整规则本计划不发明实现。

pass合同模板必须包含：ID/版本、输入输出阶段/profile、前置/后置条件、执行顺序、
语义保持义务、effect/trap/调用顺序/释放、来源保持、分析缓存失效、再校验和错误报告。
改写后不能沿用失效的Checked凭证；校验关口不能靠输入中的checked标志绕过。
未知effect按保守策略处理；未执行分支trap不可提升，命名参数不可按字段排序重排。
只交付模板及规范化/常量求值的合同示例，不实施优化或承诺完整effect lattice。

### 5.4 后端桥与自举判据

以薄Rust C ABI和独立后端进程为候选，给出输入/输出职责、版本/target/profile、
能力拒绝、所有权/错误与批量开销的决策矩阵。至少明确哪些条件会改变选择；
不冻结字段编号、不实施桥、不复用Rust Vec/String私有布局。
后端不能补做Auto名字解析/typecheck；明确工具协议与生产模块ABI分别演进。

后续验收矩阵分层：
源码计算子集、字符串/容器/record等微程序、lexer/parser代表入口、
最终约定ACC主体编译及Gen1→Gen2→Gen3代际。
逐项写输入、oracle、预期结果/诊断、运行形态、工具依赖、前置capability、证据状态。
至少8个代表锚点：算术/循环、命名参数顺序、错误诊断、token/lexer、
parser、类型检查、文件/模块解析、代际自编译；可复用已有语料，不要求本计划新跑native。
计算锚点对齐741既有fixtures；lib锚点关联真实入口和crate内语料，缺口标为待建。
bitwise固定点仅在可复现条件约定后作为门禁；不将A2R转译中转冒充直接AC native自举。

### 5.5 范围与后续工作包

仅细化两张无编号合同候选：
1. 源码计算adapter及精确类型边界：输入/输出、正反语料、既有API依赖和基线恢复门。
2. 编译器所需类型/运行时首批，或生产ABI探针：由盘点结果决定优先切口，
   写明选择证据和与独立ABI线的依赖，必要时更换候选并说明原因。

分别列可现在做/须完整v0.5基线/须前置能力，重新核查总估算与关键路径。
不分配744/745，不伞形承诺完成整个26–36计划群。

### 规范增量

执行期提交 docs/reports/743-acc-hir-contract/proposed-spec-delta.md；
canonical落地仅在review/merge阶段进行。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-acc/project.md | 无独立ACC合同→以明确主体/依赖/代际判据记录约定目标，implemented/required分开 | 防止旧自举和新native自举混称 | AC-01, AC-03, AC-06 |
| SD-02 | add | docs/specs/auto-hir/stage-contract.md | 741单profile事实与战略阶段意图→版本化阶段/pass模板及其实现状态/不变量 | 为后续adapter/pass提供可验证边界 | AC-04, AC-05 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 无stage-contract入口→新增关联入口，保持741现状、非目标与profile声明 | 索引可发现，避免把设计描述写成实现 | AC-04, AC-07 |

SD-03仅新增交叉链接，不替换现有组件，所以supersedes_spec_components为空。
待复审最终确认metadata；本计划不写canonical新文件或生成ledger。

## 6. 测试设计

属于新增调研工具/文档，遵守Category A：不跑cargo t、tv、taa、docs_gen，
不为了“读过AAVM”触发aavm运行档，不更改语法参考或正式Schema定义。
Python工具需有限但有意义的反例测试：缺文件、注册表/源码漂移、注释/字符串伪代码、
同名/未解析接收者、人工结论过期、不合法报告、不覆盖输入及重跑确定性。
测试验证清单可信度和失败路径，不只镜像扫描实现。

执行者在plan worktree运行§5.1命令及git diff --check，预期成功档exit0。
故障/陈旧报告测试期望非零和具体原因，无静默“自动修好”报告。
另人工逐项审查主体表、stage/pass表、8+语料锚点、两张候选合同及delta映射。
架构案例须说明如何保留741调用顺序/overflow/来源诊断，以及如何拒绝Unknown/effect不明的非法变换。
本计划不会产生native正确性新证据，现有741归档收据只作基线依赖。

## 7. 验收标准

- [ ] AC-01：8个源模块（七lib+CLI）及注册清单100%入manifest；真实use/外部候选有去向，
  每项带hash/位置，source-demand与target-language-support分开；对照输入和人工检查表。
- [ ] AC-02：工具三条命令成功，固定输入生成字节稳定；缺文件/漂移/未知/人工结论过期
  有明确处理，保护测试通过。--check对陈旧清单非零，不覆盖输入或人工结论。
- [ ] AC-03：逐模块迁移表+新增主体职责+保留依赖清单齐全；
  parser/typeinfo/codegen/engine/a2r至少各有真实入口证据和决定，不借旧路径宣称native完成。
- [ ] AC-04：stage-contract逐阶段完整并映射741真实API；
  Checked门、Unknown拒绝、profile/版本、来源/owner、target门及公共/低层边界有可审查规则。
- [ ] AC-05：pass模板及两个合同示例齐全，涵盖effect/trap/顺序/释放/再校验/缓存失效；
  保守unknown策略和至少两项非法变换反例明确，不声称优化实现。
- [ ] AC-06：8+验收锚点及native代际矩阵完备，oracle/expected/前置/证据状态可追踪；
  两种桥接候选有边界、选择依据和待验证问题，不混淆生产ABI。
- [ ] AC-07：两张后续无编号候选与离线/基线门明确；26–36估算核查有依据；
  SD-01..03 proposed delta逐条映射，无改canonical Spec/旧源码/741profile/外仓。
- [ ] AC-08：独立review验证AC-01..07、文档links/格式和工具结果，
  留revision-bound收据与遗漏/延后/风险登记；无未批准缩减或实施范围扩张。

## 8. 执行步骤

- [x] T-01 固定输入与差异清单（无任务前置；AC-01, AC-07）。
  在专用worktree读§4来源和741归档；创建新目录
  docs/reports/743-acc-hir-contract/，记录执行HEAD/input hash/741 API身份/离线限制。
  核对auto/lib、auto/aavm.at和AUTO_LIB_FILES_V2，新增source-manifest.json初稿。
  验证：路径/hash逐项匹配；变化有分类，新旧快照不拼成未经说明的“最新”状态。
  [✅ 已完成] worktree commit 802e62aa6。七份证据输入执行期 hash 与起草表逐项一致
  （差异分类=none）；run-context.md 记录执行基点 c1ac219e7、741 API 身份、离线限制。
- [x] T-02 可重跑盘点工具（依赖T-01；AC-01, AC-02）。
  新建scripts/acc_inventory.py、scripts/tests/test_acc_inventory.py及fixtures；
  输出生成观察与独立人工结论，诊断缺失/漂移/未知，落实§5.1接口与失败测试。
  验证：§5.1三命令exit0；故障案例按§6非零；同输入两次输出字节一致。
  [✅ 已完成] commit fd7fb366d（工具 18 测试含 T-06 加固后全绿）；真实仓 --write/--check
  exit0、字节确定性测试锁定；校准实证：File 命名空间/str 方法族补充（engine.at:808/819、
  a2r 头方法映射证据）、enum 载荷变体识别（engine.at:55）、跨行字符串能力（engine.at:314）。
- [x] T-03 审定主体/语言/运行时迁移表（依赖T-02；AC-01, AC-03）。
  人工复核auto/lib真实定义、use/方法/容器/运行时使用，
  形成报告inventory.md及manual-decisions.json（均新建），unknown有owner/后续探针。
  对照旧aavm Spec但不改它；--check绿且每个必需能力有源码位置或缺口证据。
  [✅ 已完成] commit aeae23bb8。46 条 hash 绑定决定（MD-101..506：8 模块角色+6 新建主体
  +21 能力+113 unknown 全消解）；--check 人工结论层 ok。
  [↩ 复审R1退回] F-03：use 边计数误写"15 条"，实际 18 条（manifest 为准）；
  修正 inventory.md §4.1 与 MD-413 注记。
  [✅ 修复完成] commit d7013b290：两处改"18 条"并附七文件分布
  （engine2/lexer1/typeinfo3/parser2/a2r5/codegen4/aavm1）；"15 条"全仓清零；
  decisions 绑定新鲜、--check 绿。
- [x] T-04 HIR与桥接架构合同（依赖T-03；AC-04, AC-05, AC-06）。
  新建docs/design/strategy/auto-acc-bootstrap-contract.md，在00-intro.md注册；
  形成阶段表/pass模板/反例/桥接候选矩阵，精确引用
  experimental/ac-core/src/{hir,verify,native}.rs和schema/core-i32.atom，均只读。
  验证：契约逐字段审查、741路径映射不矛盾、链接可达，未把新阶段标成已实现。
  [✅ 已完成] commit da4b8bce5。S0–S6 阶段表（实现状态逐列标注）、pass 模板+
  norm.canonical-form/eval.const-fold 两合同示例、X1–X8 反例、A/B 桥接矩阵与选择条件、
  Gen1–3 代际判据；00-intro.md 策略表已登记。
- [ ] T-05 验收与后续合同/Spec提案（依赖T-04；AC-06, AC-07）。
  新建报告acceptance-matrix.md、next-work-packages.md、proposed-spec-delta.md；
  给语料/代际/依赖门及两张无编号候选，核查估算，提出SD-01..03正文增量。
  验证：8+锚点输入/oracle/结果齐全；每个未决项有探针和owner；AC/delta映射完整。
  [✅ 已完成] commit c0a0981f7。10 锚点（existing 3/gated 7，门槛逐项落 owner）、
  候选①（源码计算 adapter）/候选②（str+容器+int 运行时首批，选择证据=MD-405/406/409）、
  26–36 估算逐组核查零变化、SD-01..03 提案正文。
  [↩ 复审R1退回] F-01：acceptance-matrix.md 对契约文档的相对链接多一级目录
  （`../../../design/...` 应为 `../../design/...`），AC-08 文档 links 项不过。
  [✅ 修复完成] commit d7013b290：链接改两级上级，程序化解析确认实达
  docs/design/strategy/auto-acc-bootstrap-contract.md。
- [ ] T-06 复核与独立交接（依赖T-05；AC-08）。
  在worktree再跑§5.1与git diff --check，检查工具/报告/合同diff；
  主检出仅维护本Plan进度，按/auto-plan:review独立检查并绑定revision1及实现HEAD。
  复审前将status置execution_done；通过后reviewed，不自行归档/合入。
  验证：收据覆盖全部AC，遗漏/债务登记；handoff交merge（以用户授权为准）。
  [✅ 已完成] commit b96f7beab（+check 门对 HEAD 审计字段免疫的加固与回归测试）。
  终验：三命令 exit0、18/18 测试绿、`git diff --check c1ac219e7..HEAD` clean、
  worktree clean；实现 HEAD=b96f7beab（plan-743-dev，基点 c1ac219e7）。
  [↩ 复审R1退回] 修复 F-01/F-03 后须重跑终验门并刷新交接记录，方可再入 review。
  [✅ 修复完成] commit d7013b290 后终验重跑：三命令 exit0、18/18 测试绿、
  `git diff --check` clean、worktree clean（0 dirty）；实现 HEAD=d7013b290。

## 9. 复审记录

### 合并沉淀收据（2026-10-04，stage: merge，key: PLAN-743:r1）

- stage: merge
- plan_id: PLAN-743 · plan_revision: 1 · outcome: pass（delivery/ledger/archived/cleanup 见下方 checkpoint 实证）
- reviewed 基线：R2 pass @ d7013b290（worktree plan-743-dev，delta 冻结 b6b807110e5beac6）
- 落点分支：v0.6-dev（本主线约定落点；741 先例一致）

**checkpoint 实证**：

| checkpoint | 证据 |
|---|---|
| prepared | worktree 内按冻结 delta 逐字落 SD-01/02/03：新建 docs/specs/auto-acc/{project,plans}.md、docs/specs/auto-hir/stage-contract.md；auto-hir/project.md 插入阶段契约节；overview.md/INDEX.md/spec-index.py 组表登记；spec 链接程序化全通。commit 4b54ba805（reviewed_commit 的纯文档后代，实施/依赖零变化） |
| landed | rebase v0.6-dev（他 Session 并行推进两轮：3a7967262、c8d869878，均与本案零文件重叠）；range-diff 全等号（8/8 `=`，安全重写证明）。旧→新映射：802e62aa6→32ba22c5c、fd7fb366d→b773e4c5f、aeae23bb8→3a9b870d1、da4b8bce5→24c9a3aab、c0a0981f7→4b337c8fc、b96f7beab→0a416c84c、d7013b290→9abc87acc、4b54ba805→05d0127a4。`git merge --ff-only` 两次（05d0127a4、bbfca37b8），零 merge commit；delivery 终点=bbfca37b8 |
| landed（SD-03 落地效应处置） | 主检出冒烟 --check 报 4 条 decision-stale（MD-203/406/409/421 绑定 docs/specs/auto-hir/project.md，SD-03 按设计改变其 hash）——staleness 门按预期工作；结论与被引证据行未变（交叉链接为纯追加），worktree 内机械重绑至落地 hash b050db47 并注明理由，commit c49bd4e2a→rebase→bbfca37b8，重绑后 --check 全绿。经验注记：人工决定绑定"本计划 delta 将修改的文件"时，落地必然触发重绑；后续计划可在起草时对 delta 目标文件预注 exempt |
| ledger_refreshed | .autoos/specs.json upsert P743-1/P743-2（designs→两份 canonical spec）+ P743-3（reviews→R1/R2 裁定链），worktree 提交 274a9ded4 后 ff 落主检出，read-back 验证三 ID 在案；INDEX.md 由 spec-index.py 再生（+1 行 auto-acc） |
| archived | 本文件 git mv 至 docs/plans/archive/，status: archived |
| cleaned | 见文末补记（wt-guard clean 后移除 worktree/分支/组目录） |

- 规范增量落地：SD-01→docs/specs/auto-acc/project.md（add）、SD-02→docs/specs/auto-hir/stage-contract.md（add）、SD-03→docs/specs/auto-hir/project.md（交叉链接节）。canonical 落地文本与冻结 delta 一致（R2 复核 b6b80711 未触）。
- 主检出冒烟：--check exit0（46 决定绑定新鲜）、18/18 测试、v0.6-dev tip 干净。

### 独立复审 R2（2026-10-04，stage: review，R1 修复复核）

- stage: review
- plan_id: PLAN-743
- plan_revision: 1
- outcome: **pass**（R1 两项发现确认修复；全部 AC 转满足）
- reviewed_commit: d7013b290f3e94ff91a5f0ee3b868d221c408ac6（worktree plan-743-dev，clean 0 dirty）
- base_commit: c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75（v0.6-dev）；R1 基线 b96f7beab
- dependency_revisions: 无外仓依赖
- spec_inputs: 9 受管输入 hash --check 全绿；七份证据输入与 run-context.md §3 一致
- 独立性声明：复审在实现会话内进行（同 R1）；结论由命令重放与制品重建。

**修复复核（R1 findings → R2 verdict）**：

- F-01 → **fixed**：acceptance-matrix.md:6 链接改为两级上级，程序化解析实达
  `docs/design/strategy/auto-acc-bootstrap-contract.md`。
- F-03 → **fixed**："15 条"于全部 8 个报告制品 0 命中；inventory.md 与 MD-413 各恰一处
  "18 条"；manifest 独立重数 use 边 = 18，逐文件分布与修正注记完全一致
  （aavm1/a2r5/codegen4/engine2/lexer1/parser2/typeinfo3）。
- 修复 diff 范围审计：b96f7beab..d7013b290 仅 4 文件 4 行（3 处修正 + manifest 审计
  HEAD 字段刷新），无其它变更混入；proposed-spec-delta.md 冻结哈希不变
  （b6b807110e5beac6，R1 冻结仍有效）。

**门禁重放（reviewed_commit 上）**：--check exit0（46 决定绑定新鲜）；18/18 测试；
`git diff --check b96f7beab..d7013b290` clean。

**收据冻结哈希（reviewed@d7013b290）**：manual-decisions.json fdbc1a86ad8025e6、
inventory.md d45c7316e0673247、acceptance-matrix.md 896d268d8ea1a75a、
proposed-spec-delta.md b6b807110e5beac6。

**AC 终态**：AC-01..07 pass（R1 已证，本轮未受修复影响——修复不触工具/清单/合同正文）；
AC-08 pass（链接、格式、工具结果、revision-bound 收据 R1+R2、遗漏/延后经 F-01/F-03
登记并修复闭环）。

- findings: 无新增
- evidence: 本记录命令/结果摘录；制品经 merge 落 v0.6-dev 后同 hash 可溯
- next: merge（沉淀 SD-01..03、归档计划、清理 worktree——以用户授权为准；
  归档/合入不在 review 职责内）

### 修复交接（2026-10-04，stage: work，R1 needs_fix 修复）

- stage: work（needs_fix 修复循环 1/3）
- plan_id: PLAN-743
- plan_revision: 1
- outcome: pass（F-01/F-03 已修复并重验；非独立复审结论，待 review 复核）
- code_commit: d7013b290（worktree plan-743-dev；R1 reviewed 基线 b96f7beab 之上仅追加本修复）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留）
- task_ids: T-03/T-05/T-06 复位 [x]（修复证据见各任务行）；current_step 6/6
- evidence: "15 条"全仓清零（grep 两文件 0 命中）；F-01 链接程序化解析实达契约文档；
  三命令 exit0；18/18 测试；`git diff --check` clean；worktree clean。
  修复 diff 仅 3 文件：acceptance-matrix.md（链接）、inventory.md（计数）、
  manual-decisions.json MD-413 注记（绑定 hash 未动，--check 人工结论层绿）。
- blockers: 无
- next: /auto-plan:review 复核 F-01/F-03 修正（R1 记录预设：复核两项即可转 pass）。

### 独立复审 R1（2026-10-04，stage: review）

- stage: review
- plan_id: PLAN-743
- plan_revision: 1
- outcome: **needs_fix**（2 项 P2 发现；其余全部核验通过）
- reviewed_commit: b96f7beabb760bc22f386c414513c31de45018a4（worktree plan-743-dev，clean）
- base_commit: c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75（v0.6-dev）
- dependency_revisions: 无外仓依赖；741 归档基线=归档时收据（本计划只读引用）
- spec_inputs: 七份证据输入 + 9 受管输入 hash 经 --check 全绿复核（与 run-context.md §3 一致）
- 独立性声明：复审在实现会话内进行（无独立会话可用）；结论全部由命令重放与制品
  重建，不采信执行摘要。
- diff 范围审计：21 文件全部位于 docs/reports/743-*/、scripts/acc_inventory.py、
  scripts/tests/**、docs/design/{00-intro,strategy/auto-acc-bootstrap-contract}；
  crates/、auto/lib、docs/specs/、experimental/、Cargo 零触碰（grep 探针无命中）。

**逐项核验结果**：

| AC | 结果 | 方法与证据 |
|---|---|---|
| AC-01 | pass | manifest 9 输入（8 模块+注册表）100%；18 条 use 边全部解析到 lib 模块且每个导入符号在目的地存在（程序化核验 NONE-missing）；113 unknown 全带去向；source-demand/support 维度字段在 manifest scan_semantics 显式分离 |
| AC-02 | pass | 三命令重放 exit0；双临时目录 --write 字节一致且与仓内制品 cmp 一致；漂移负例（临时副本仓改 token.at）exit1 且诊断具体；--write 对 manual-decisions.json 字节不变（sha256 220d59df 前后一致）；篡改/缺文件/过期/注册表漂移由 18 用例覆盖 |
| AC-03 | pass | 8 模块迁移表+6 新建主体+保留服务清单齐全；parser/typeinfo/codegen/engine/a2r 真实入口证据逐行命中（ar_emit_program@4479、type P@71、AUTO_LIB_FILES_V2@1893 等抽检 8/8 命中） |
| AC-04 | pass | S0–S6 阶段表实现状态逐列标注，未把未实现阶段标成已实现；741 API 引用逐行核实（verify.rs:59、native.rs:43/111/189、link.rs:166/229、schema/core-i32.atom 身份）；Checked 门/Unknown 拒绝/profile/来源/公共-低层边界均有可审规则 |
| AC-05 | pass | pass 模板全字段（effect/trap/顺序/释放/再校验/缓存失效）+ norm.canonical-form、eval.const-fold 两合同示例；X1–X8 反例（≥2 要求）；"合同非实现"声明显式 |
| AC-06 | pass | 10 锚点（≥8）各带输入/oracle/运行形态/工具依赖/前置 capability/证据状态；Gen1–3 矩阵+依赖清单归档；双桥候选边界/选择条件/4 项待验证问题齐备，未混淆生产 ABI |
| AC-07 | pass | 两候选含可现在做/须基线/须前置分类；26–36 估算逐组核查有据零变化；SD-01..03 逐条映射且提案正文与目标现状一致（目标文件确认尚不存在；SD-03 为纯插入）；canonical Spec/旧源码/741 profile/外仓零改动 |
| AC-08 | partial | 本复审即该项执行；发现 F-01/F-03 退回（见下）；修复后重验转 pass |

**发现（needs_fix 依据）**：

- **F-01（P2，AC-08/AC-06 载体文档）**：`docs/reports/743-acc-hir-contract/acceptance-matrix.md`
  顶部对契约文档的相对链接 `../../../design/strategy/auto-acc-bootstrap-contract.md` 多一级
  目录（743-acc-hir-contract → docs 需两级上级），链接不可达。
  修正：改为 `../../design/strategy/auto-acc-bootstrap-contract.md`。
- **F-03（P2，AC-01/AC-03 证据注记准确性）**：inventory.md §4.1 与 manual-decisions.json
  MD-413 注记写"显式 use 边 15 条"，manifest 实测 **18 条**（2+1+3+2+5+4+1）。
  manifest 本身正确；人工叙事层须以 18 修正，防止审计数字失真。
- （澄清，非发现）proposed-spec-delta.md 内 plans.md/stage-contract.md "断链"位于提案正文
  代码围栏内，是对 delta 目标未来文件的引用，非本报告活动链接，不改。
  独立副本篡改探针因本机 /tmp 路径翻译 inconclusive，该失败路径由套内
  test_tampered_manifest_fails_check 覆盖（18 用例含）。

**delta 冻结哈希（reviewed@b96f7beab）**：proposed-spec-delta.md b6b807110e5beac6…、
manual-decisions.json 220d59df6e8d3e5e…、inventory.md b7c9bc44ef180e3d…、
acceptance-matrix.md d8ea8e6253b508c4…。

**处置**：状态回 executing（current_step 4）；T-03/T-05/T-06 重开（保留历史证据），
修复 F-01/F-03 → 重跑 T-06 终验门 → 再入 review 复核两项修正即可转 pass。
未发现未批准缩减、范围扩张或 workaround；main 检出上他 Session 的 741-quality WIP
（vue.rs 等）不属本计划 diff，已在执行交接报告并向用户呈报，须由其所有者路由。

- evidence: 本记录内命令/结果摘录 + worktree 提交链 b96f7beab（worktree 移除后经
  merge 落 v0.6-dev 的同 hash 提交可溯）
- next: work（修复 F-01/F-03，循环上限内 1 次）→ review 复核

### 执行交接（2026-10-04，stage: work）

- stage: work
- plan_id: PLAN-743
- plan_revision: 1
- outcome: pass（任务/验收映射完成；非独立复审结论）
- code_commit: b96f7beab（worktree plan-743-dev；基点=执行基 c1ac219e7，v0.6-dev）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留待 review/merge）
- task_ids: T-01..T-06 全部 [x]；current_step 6/6
- evidence: §5.1 三命令 exit0（--write/--check/unittest 18 用例）；字节确定性测试锁定；
  `git diff --check c1ac219e7..HEAD` clean；worktree clean；46 条人工决定 hash 绑定
  全部新鲜；manifest 覆盖 9 受管输入、注册表一致。实现期校准三项有据：
  File 命名空间、str 方法族、enum 载荷变体（engine.at:55/808/819）。
- blockers: 无
- next: /auto-plan:review（独立复审，绑定 revision 1 + 实现 HEAD b96f7beab；
  通过后由 merge 沉淀 SD-01..03）。本记录不替代复审，不自行归档/合入。

### 起草交接（2026-10-04）

- stage: new
- plan_id: PLAN-743
- plan_revision: 1
- outcome: pass（合同完整性检查；不是实施复审通过）
- next: work（收到用户执行指令后；本轮仅规划）
- 新增任务：T-01..T-06；验收：AC-01..AC-08；Spec提案：SD-01..SD-03。
- 741已归档为依赖；不等待主机器即可做固定快照盘点，旧代码接线留后续基线门。
- 起草检查：任务覆盖全部AC和delta；真实已有路径已核对，新路径显式标注；
  工具命令为本计划待实现接口，未声称运行通过；无实施改动、无cargo门禁执行。
- 授权仅为起草/提交；此记录不批准后续实施或替代独立/auto-plan:review。

## 10. 待澄清事项

1. ACC最终主体/目标语言coverage：执行者在T-03..05提出有证据的具体边界；
   改变已约定v0.6自举目标的决定须交用户，不以调研结果自动缩减验收。
2. int/i32/Unicode/容器别名与释放等未定细则：记录最小探针和负责工作包；
   本计划允许研究待定，不能把unknown当implemented，后续相关实施前必须解决。
3. 进程内/进程外桥：本计划完成候选矩阵，实际探针与最终编码另立实施合同。
4. 主机器恢复后新输入：保留本次manifest，先报告差异/结论失效；
   重跑盘点属本计划内校准，扩大仓库/目标/验收则修订合同。
5. 无必须先由用户裁定的问题阻止起草完成；未获本计划实施指令，不自动开工。
