---
plan_id: PLAN-743
status: reviewed
feature_name: ACC 自举能力盘点与 HIR 阶段契约
author: [Codex]
created_at: 2026-10-04
updated_at: 2026-10-06
plan_revision: 5
current_step: 32
total_steps: 33
supersedes_spec_components:
  - docs/specs/auto-acc/project.md
new_spec_components: []
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


### Phase 2 再激活（2026-10-05，plan_revision 2）

合入后独立复核发现工具、合同与验收资料的新缺口；按用户明确授权激活同一 PLAN-743，
追加 P743-QA-01..07 修复 Phase，不另取号。r1 六步骤及原修复/复审/合入历史保留；
旧 T-05/T-06 勾选按既有交接证据纠正为历史已完成，新任务 T-07..14 尚未执行。
本轮只复审与修订合同，不修工具实现、不宣称当前清单已恢复新鲜或 ACC 已实现。

### Phase 3 再激活（2026-10-05，plan_revision 3）

r2 已落地；合入后独立复审 needs_fix（基线 5983f8aeedeae2dc5769f9a0820eec4b722d4c6d）。
按用户此前明确授权再次激活同 ID，修 P743-R2-QA-01..06；不重复旧 Phase、不取新编号。
T-01..14 的执行/复审/合入作为历史保留；T-14 按真实收据补勾，不将旧 pass 当作 r3 pass。
本轮仅合同/证据/簿记，待实施 T-15..21；current_step=14、total_steps=21。
旧 Phase 中“本轮未实施/新任务尚未执行”等表述为其起草时记录，以新 Phase/收据区分。

### Phase 4 再激活（2026-10-05，plan_revision 4）

r3 已落地；合入后独立复审 needs_fix，基线 8e8f6f19b96516fdbc9d4c893297fa1757a7aad4。
用户此前授权有问题时激活同 ID、追加修复 Phase；本轮修 P743-R3-QA-01..04，不取号、不实施工具。
T-01..21 的交付/复审/merge/clean 均作历史保留；T-21 按真实收据补完成，不把旧 pass 当 r4 pass。
当前 executing/r4，completed21/total27；新待办 T-22..27。旧 Phase 的起草态数字/“尚未实施”是历史，不表示当前状态。

### Phase 5 再激活（2026-10-06补齐落地，plan_revision5）

合入r4后独立复审 **needs_fix**，绑定HEAD c865adf66a75099df07113b053eb9d19e92986a3。
按用户此前明确授权激活同ID，详见[本轮报告](../reports/743-r4-quality-review-20261005/REVIEW.md)。
原复审基线83/83、47新鲜决定、SD-08/canonical及27/27/ledger通过，但P743-R4-QA-01..04有反例；后续741 Spec变更使5条决定stale，当前strict不宣称全绿。
本轮仅合同/证据/簿记，不修Python、不改canonical/ledger、不占新号。r1..r4实施/复审/merge收据保留为历史；不能替代r5验收。
当前任务完成T01..22=22/33；T23..27重新待验（紧随其后的旧完成证据仅描述r4，旧[✅]不覆盖当前[ ]），新增T28..33。
此前Phase3/4的“当前/待实施”数字为起草历史；本段及frontmatter为r5当前状态。


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


### 4.3 Phase 2 授权与当前输入

用户 2026-10-05 明确要求：“计划743也已经实施并合并了；请同样进行检查：如果有问题，
激活计划文件，并把问题和解决方案作为新的phase更新到计划文件里去。”
因此允许同 ID 再激活、更新合同与簿记，优先于不重开归档的通常规则；本轮不自动实施修复。

复核基线 fa3abe47677992d41b1aef948f36ad46c645f7f8（v0.6-dev，clean）；原实施基线 c1ac219e7。
交付 9abc87acc/05d0127a4/bbfca37b8/274a9ded4/bab649682 的祖先关系已核实。
取材 docs/specs/overview.md、auto-acc/project.md、auto-hir/stage-contract.md 与当前 auto-hir/auto-ac Specs；
完整 SHA-256 与新反例见 [独立复核报告](../reports/743-quality-review-20261005/REVIEW.md)
及 results.json/audit.json。实施 T-07 重取实际激活提交 hash，不从旧 plan-743 tip 起步。

本轮实跑 18/18 测试、双输出确定性、15 活动链接及三个决定/两份扫描反例；
9 扫描输入与生成观察未漂移，但 8 条人工决定有 9 处绑定过期。
scope 仍仅本仓调研工具/文档：scripts/acc_inventory.py、scripts/tests/test_acc_inventory.py/
fixtures、743 报告、auto-acc-bootstrap-contract 设计及计划簿记。
不改 auto/lib/*.at、crates/**、旧 parser/VM/A2X、根 Cargo、741 profile 或外仓，
不实现优化器/后端桥/ACC；canonical 行为 Spec 只在独立 review pass 后由 merge 沉淀。
新 Phase 有自己的验收和冻结 delta；r1 pass 为历史结论，受影响旧 AC-01..08 必须重新核验。

### 4.4 Phase 3 授权、基线与输入

本轮用户要求检查 743:r2；沿用此前“有问题激活计划，并把问题和解决方案作为新的 phase”的授权。
范围仍本仓调研工具/文档，未授权在复审中实施修复；不改旧源码/741/native/ABI/外仓。
5983f8aee 的 r2 已落地且 worktree 清理，完整 reviewed SHA/Spec/源码指纹及 AC 对账见
[新独立报告](../reports/743-r2-quality-review-20261005/REVIEW.md)。主线并行 ed2d00b90 仅是 NOTES-001 sqlite 运行时依赖，不归 743。
T-15 须取含本合同的最新 v0.6-dev，重核指纹/变动；不从已删除 r2 分支或旧报告审计 HEAD 开始。
Python 3.14.2；本轮 41/41、两次扫描字节一致、真实严格门 47 条新鲜/9 输入/146 unknown 绿。
绿只覆盖当前正例；独立负例在报告中：未绑定证据变化仍绿、resolved 不引用决定仍绿、类型错误 traceback、跨 owner 分类越权。
原 canonical 文字与冻结 SD-04/05 正文一致，但 R3 旧摘要未同步；r3 要补新 delta，不降低原约束。
.next-id=744，不分配新号；741 r3 / ABI / 742 / NOTES 其它工作线保持独立。

### 4.5 Phase 4 授权、基线与输入

用户本轮要求“743又一次修复并合并，继续检查”，沿用此前明确的发现问题激活同计划追加 Phase 授权。
当前 reviewed_commit=8e8f6f19b96516fdbc9d4c893297fa1757a7aad4；r3 基点=a0f8ddd443418fb32a67d308cbd4a0fd00f74835。
[新独立报告](../reports/743-r3-quality-review-20261005/REVIEW.md) 保存源码/Spec/提案 SHA-256、组合反例及全 AC 对账。
本轮63/63、双输出确定性、真实严格门47决定/9输入/146unknown绿；20活动链接和SD-06/07三段匹配。
新增失败仅被引用非法记录、evidence覆盖 OR 旁路、导入/参数遮蔽、归档生命周期；不是当前真实人工层已陈旧。
T-22 从含本合同的最新v0.6-dev取基线，保持741/742/ABI检出独立；不覆盖其 WIP/分支。.next-id=744不变。
Python3.14.2标准库、PYTHONUTF8=1、Category A；仍仅本仓工具/测试/报告/设计簿记，无 Rust/native/ACC/外仓实施授权。

### Phase 5 needs-analysis / 可执行边界

草稿准备时主线更新至7b9c6948c1465e87a6f60404510c30c712e1c7bc（741 r4已合入）；743代码/合同未改变。当前strict正确拒绝MD-203/205/301/302/304的AutoAC Spec过期绑定；T28记录新基线，T30须重新审定这5条结论/证据，不能机械换hash。共享债务及导航保留最新741状态。

主检出v0.6-dev基线c865adf66；741、742、ABI其它工作树不属于本修复。四问题定位和完整反例见本轮报告及counterexamples.json。
模块Specs为auto-acc/project与auto-hir/stage-contract；SD-08已承诺完整语义校验的消费者边界与已知参数遮蔽。修复仅现有Python盘点/检查和归档验证，不增加ACC/HIR编译能力。
真实47条决定当前全部fresh，不能从合成错误推断当前语料已有错误；执行后须观察实际分类差异、逐条审定，不机械hash刷新。


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


### 5.6 Phase 2 详细修复方案（P743-QA-01..07）

1. **词法屏蔽（QA-01）**：合法跨行字符串保持字面量状态到真正结束，正确跟踪转义与行号；
   不支持形态整体标记词法 unknown，禁止内容作为真实调用/声明/import。补跨行伪代码、
   转义换行、关闭后真实代码、注释边界回归；复核 MD-506 的“恢复正确”判断。
2. **调用保守分类（QA-02）**：本地 type/owner 与当前作用域证据先于内建方法名；
   只有可证明的宿主 namespace/接收者才标 native-runtime，未知接收者/同名裸调用/管道保持 unknown。
   CG.new/Ar.new、Meter.new/x.len、同名跨 owner 必须断言调用类别；不升级为完整名字解析器。
3. **人工层完整性（QA-03）**：provided 决定验证 schema/version、记录类型、唯一 ID、
   合法状态、非空 hash 绑定、证据文件及绑定覆盖。malformed/无证据/无绑定明确非零且定位 ID。
   增加新接口 --require-decisions（严格完成态门）：无/空人工层及缺覆盖闭环不得成功。
   保留早期 scan-only（人工层 absent）兼容，但不能显示为 ok/完成态；本计划终验必须用严格门。
   不硬编码当前 46/113 数量；用实际模块/能力/unknown 清单及显式决定引用核验覆盖，
   unknown 可保持待研究，但必须带 owner/探针/所在工作包，不能要求所有语义立即已实现。
4. **pass 语义（QA-04）**：普通运行期常量表达式的溢出保留运行期 trap 或等价节点，
   “必经路径”不能自动改为编译期拒绝；仅语言已明确要求编译期求值的语境按独立规则诊断。
   区分保守 effect 近似与真实 effect/trap 行为保持，改正“收窄”方向术语；
   新增 observable 前驱→必经 trap、不可达 trap 的合同反例。
   在阶段表明确哪些字段/API/规则已实现、哪些只是目标态；不新增优化实现或 effect 格冻结。
5. **锚点与候选（QA-05）**：A1..A10 全保留；A2 按 HIR existing/源码 gated 分开并给实际路径，
   A8/A9 待建语料不能是缺能力的候选①必过正例。两候选按最小交付拆分无环前置，
   str/List 归提供能力的工作包，enum/is 明确后续能力负责人/前置；允许后续工作线未编号，
   不伪造第三个已批准实施计划，不抢占 744/745。
6. **校准（QA-06）**：逐条审查 MD-203/205/301/302/304/406/409/421 对当前 Specs 的语义差异，
   记录维持/调整/待定及理由后才重绑；扫描算法变更后重新生成观察、审查新增 unknown 与决定覆盖。
   不统一换 hash、不 exempt 本计划/741 将改的 delta 目标文件。
   741 Phase 3 缺陷修复留在 741，743 记录依赖可用性和限制；最终 Spec 沉淀再次改变 hash 时，
   merge 必须语义复核重绑后严格 --check 冒烟，不能用交付前绿证据替代。
7. **簿记（QA-07）**：保持单一 active/executing/r2 文件，历史六任务完成、八新任务待办；
   同步模块索引/ledger 文件指针与当前引用，历史 pass 不当 r2 pass；最终正确归档写 archived。

原 SD-01..03 为已交付历史；Phase 2 提案另存新文件
 docs/reports/743-acc-hir-contract/proposed-spec-delta-phase2.md，不覆盖 r1 冻结提案。

### 规范增量

执行期提交 docs/reports/743-acc-hir-contract/proposed-spec-delta.md；
canonical落地仅在review/merge阶段进行。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-acc/project.md | 无独立ACC合同→以明确主体/依赖/代际判据记录约定目标，implemented/required分开 | 防止旧自举和新native自举混称 | AC-01, AC-03, AC-06 |
| SD-02 | add | docs/specs/auto-hir/stage-contract.md | 741单profile事实与战略阶段意图→版本化阶段/pass模板及其实现状态/不变量 | 为后续adapter/pass提供可验证边界 | AC-04, AC-05 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 无stage-contract入口→新增关联入口，保持741现状、非目标与profile声明 | 索引可发现，避免把设计描述写成实现 | AC-04, AC-07 |
| SD-04 | modify | docs/specs/auto-acc/project.md | 以 manifest 为准的能力现状→标明人工层严格完整/新鲜证据与未知闭环；锚点/候选前置及当前支持声明一致 | QA-01/02/03/05/06 使主体盘点可复核，不将工具观察当语义支持 | AC-01,AC-02,AC-03,AC-06,AC-07,AC-09,AC-10,AC-12,AC-13 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | pass 语义保持/阶段状态摘要→明确运行期 trap 与编译期诊断界限、保守 effect 方向及已实现/目标态边界 | QA-04 消除常量折叠示例和保真规则冲突，不实现优化器 | AC-04,AC-05,AC-11 |

SD-03仅新增交叉链接，不替换现有组件，所以supersedes_spec_components为空。
待复审最终确认metadata；本计划不写canonical新文件或生成ledger。

### 5.7 Phase 3 详细设计（P743-R2-QA-01..06）

1. **分类证明（QA-01）**：本地 type/fn/import 与宿主名字冲突时禁止 native 推断；隐式 self 只凭当前 enclosing owner 的方法证据升级，管道无 receiver 类型证明保持 unknown。可以做有限词法 owner 记录，不要求完整 resolver；不通过全模块方法名字集猜归属。更新原错误正例并补真正当前 owner 正例，保留跨行保护和 CG.new/Ar.new 正确结果。
2. **证据 binding（QA-02）**：同一决定 evidence 引用的每个仓内文件均须同条 bound_input_hashes 覆盖，身份/类型明确；非 9 扫描输入的证据同样校验。无绑定报 ID/path，变动报 stale；真实当前决定无缺绑定，不伪造“当前输入已 stale”。重绑定必须逐条记录语义审定。
3. **unknown 闭环（QA-03）**：resolved 族必须有非空、存在且适用的决定引用，决定结论/证据与族对应；open 族独立定义 owner、probe、work_package 与可选决定引用，禁止把不存在引用或随意 ID 当闭环。至少明确引用状态/相关性可校验规则并纳入实际人工层；不能强迫 unknown 变成 implemented，不硬编码当前数量。新增/变化的 unknown 逐条分配去向。
4. **完整类型检查（QA-04）**：先校验 top-level/decision/family 字段及元素类型，再哈希/迭代/集合查询；ID、kind、conclusion、evidence、bound map、hash、names、owner/ref/probe 等错误类型均受控拒绝，报告定位 ID（若 ID 本身非法则用记录索引）。无 traceback，不以统一吞异常/空数组回退绕过校验；不实现通用 schema 引擎。
5. **合同一致（QA-05）**：r3 proposed delta 补 canonical R3 摘要为效果分析精化与实际效果双向保持两分；设计正文/模板/X9、canonical/提案保持一致，不实现 pass/effect lattice。报告与候选的真实 CLI 更名 auto-ac-prototype；既有 HIR/待建源码状态保留，不能虚报 741 Phase 3 修复已完成。
6. **生命周期（QA-06）**：历史 T-14 按 r2 收据补勾；新任务另列待办，保持 active/executing/r3。复核归档前后两个报告链接、module plans/README 若有指针、ledger P743-3/4 文件指针，最终计数=21/21 且 archived；ledger 只由 merge 沉淀，本轮保持未修改。保留所有 needs_fix/pass/merge 历史，不用修改旧报告的办法消除新反例。

Phase 3 proposed delta 新路径：docs/reports/743-acc-hir-contract/proposed-spec-delta-phase3.md（执行期新建），不覆盖 r1/r2 冻结提案。

### Phase 3 规范增量（SD-06/07）

| delta_id | 操作 | docs/specs/... target | before → after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-06 | modify | docs/specs/auto-acc/project.md | 仅声称结构/新鲜/覆盖完整 → 明确同条证据绑定、类型受控拒绝、unknown 的适用决定闭环、基于当前 owner 的分类政策 | R2-QA-01..04，杜绝工具假绿，不声称语义 resolver 已实现 | AC-01/02/03/09/10/13/15/16/17/18 |
| SD-07 | modify | docs/specs/auto-hir/stage-contract.md | 旧 R3 摘要 + 新正文并存 → 摘要与分析/语义两分、runtime trap/X9 完全一致 | R2-QA-05，补遗漏，不实施优化器 | AC-04/05/11/19 |

### 5.8 Phase 4 详细设计（P743-R3-QA-01..04）

1. **先验后引用（QA-01）**：族消费者的索引只含经字段/语义校验的决定，或有结构错误时明确终止引用阶段；禁止从原始决定重建无类型保证的索引。无效引用报ID/族/原因，错误ID用索引定位。覆盖被 resolved/open 引用记录的错误字段、缺失/重复ID/顺序组合，stderr 无 traceback；不采用吞异常/空数据回退。
2. **严格证据适用（QA-02）**：resolved 的相关性以同条 evidence 所引用的文件集合证明，再独立要求其 binding 新鲜。hash map 或 subject 不能替代 evidence，移除两条 OR 旁路；“只bound/只subject”负例拒绝，真正 evidence+binding 正例绿。canonical SD-06 已是该约束，不改宽规范来迁就实现；真实七族已有证据无需盲换。
3. **宿主准入（QA-03）**：有导入、参数或已知局部绑定同名冲突就不能猜 native；有限词法层能证明哪些 binding/owner才升级，不需要完整 resolver。把导入冲突传到qualified判定；可识别参数/局部绑定优先，不确定作用域保留unknown。保留bare print导入、本地type/fn及owner既有正确结果；真正宿主无冲突正例过。变化的unknown/人工结论逐条核准，不能硬编码旧47/146。
4. **历史与终态（QA-04）**：按r3 review/merge/clean收据补T-21历史完成，current21/27；新Phase独立待办。final merge须对所有P743-* file指针、task/counter、active/archive、module plans、链接逐项断言；归档27/27，不仅更新最新P743-5。旧“尚未实施”等标为历史起草段，重复R4标题消歧，不更改旧verdict；live ledger只由merge更新。

### Phase 4 规范增量（SD-08）

执行期新建 docs/reports/743-acc-hir-contract/proposed-spec-delta-phase4.md，不覆盖 r1..r3 提案。

| delta_id | 操作 | docs/specs/... target | before → after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-08 | modify | docs/specs/auto-acc/project.md | 当前owner/本地遮蔽政策 → 明确导入/已知参数局部绑定冲突亦阻止native猜测，引用只读验证记录，证据覆盖不能由subject/hash替代 | QA-01..03把原保证落实到所有consumer，保持规范强度，不声称完整resolver | AC-01/02/03/09/10/15/17/18/21/22/23 |

r4不改auto-hir阶段/R3/pass规则；SD-07已确认落地，旧映射在历史收据中保留。生命周期纯簿记无新行为Spec组件。

### Phase 5 设计 / SD-09（未实施）

1. QA-03：先完成记录类型/枚举语义/证据存在/同条绑定/新鲜度/全局唯一ID验证，再建立消费者索引。重复ID整组剔除（含第一条）；或全验证失败后停止族消费。非法记录不贡献覆盖；最终CLI非零不是消费者可信证据。不泛化except吞错或空值兜底。
2. QA-01：普通/mut自由函数及已观察方法（含静态）的参数名收集正确；bare和qualified均在宿主/类型升级前检查绑定冲突，保留unknown。回归导入/局部类型/正常宿主正例；不要求完整resolver，不把探索for/单行绑定列入必做。
3. QA-02：manifest顶层与source_identity容器形状在访问/audit-only归一前验证，受控ERROR、非零、无traceback；不写输入/人工层。正常source/summary漂移仍由完整重生成比较检出，audit-only兼容保持。
4. QA-04：归档时修正文档相对路径。新增docs/reports/743-phase5-validation-fixes/final_assertions.py，检查实际active/archive父目录的Markdown链接（忽略代码围栏）、全部33任务/meta/索引/P743-*指针。r4 helper/报告是历史证据，保留。
5. SD-09 modify auto-acc/project.md，new_spec_components为空；冻结[提案](../reports/743-r4-quality-review-20261005/proposed-spec-delta-phase5.md)。auto-hir无新增阶段/pass；canonical与ledger由review pass后的merge沉淀。激活导致旧archive指针暂缺需记录，最终全部修回。


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


### Phase 2 测试与文档门禁

在专用 plan worktree 运行（--require-decisions 为本 Phase 新建接口）：

    python -B -m unittest discover -s scripts/tests -p test_acc_inventory.py -v
    python -B scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --write
    python -B scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check --require-decisions
    git diff --check

--write 不能覆写人工结论；hash 校准要逐条审定；固定受管输入两次生成字节一致（HEAD 仅审计），
--check/严格 --check 都应在真实当前清单成功，而决定空绑定/证据缺失/空人工层严格档非零。
新增测试须证明跨行内容不进入观察、同名调用未被猜为 native、schema/coverage 被真实检查；
不能把断言改成现有错误类别、ignore 新反例或手改生成 summary 来通过。
设计/矩阵以文档检查验证：每个锚点有实际/待建输入、oracle、状态、前置与 owner；
两候选最小交付的前置无环，runtime trap/observable 合同正反例一致，旧 18 测试与所有新增绿。
链接检查排除冻结提案围栏的未来链接；原型/旧源码相对 r2 实施起点零 diff。
仍 Category A，不跑 cargo/tv/taa/tf/docs_gen；不在本计划新跑 native 自举证据。

### Phase 3 追加验证

继续 Category A；在专用 worktree 跑旧 41 + 新反例单测、真实 --write/严格 --check、两次生成字节确定性、manual --write 不变、git diff --check 与围栏感知链接检查。设置 PYTHONUTF8=1 消除 Windows harness 编码歧义，记录版本与退出码。
报告复现脚本使用新 scratch 输出，不覆写旧报告。字段类型负例断言非零、ERROR/code 与 ID/索引，且 stderr 无 traceback；合法 resolved/open/绑定完整输入正例通过，scan-only 兼容保留。
分类测试包含本地同名 IO/print、其它 owner 方法、无 owner 点/管道，原 CG.new/Ar.new/未知 x.len 和合法跨行保护仍过。
证据在扫描输入外的仓内文件：缺同条绑定拒绝，补绑定通过，改变内容 stale；unknown 缺/错/不适用引用拒绝，open 去向完整通过。
元数据模拟最终 archive 路径检查链接，按历史收据核对任务勾选；执行期不提前写 review/merge 成功。Spec 沉淀影响 hash 时逐条语义重绑，再严格冒烟。

### Phase 4 追加验证

仍Category A，在专用743检出：旧63+新组合测试，真实 --write/--check --require-decisions、manual不变、双输出确定性、范围/hash/diff/链接检查；不跑Cargo/native。
类型负例必须作用在真正被族引用的决定，不只decisions[0]；覆盖字段/缺失/重复与resolved/open，非零+定位ERROR+无Traceback。
证据适用性正反成对：只有subject/hash拒绝，真实evidence加同条hash通过，文件变化stale。
导入/参数/局部同名native负例保持unknown或有证明的用户归属，无冲突宿主/当前owner/跨行保护继续过。清单变化有逐项审定，禁止机械hash更新。
生命周期模拟active与archive两种路径，最终按全部P743-*递归检查存在性；final completed=total=27，归档态与新Phase待办不可混用。

### Phase 5 验证计划（Category A）

在同Plan专用工作树执行，先基线复现本轮reproduce.py，然后83旧测试+新增负例/正例、当前真实strict、双write字节一致和提交制品三方比较、manual无变、SD09冻结、范围/hash与diff --check。
非法conclusion/stale/duplicate两种顺序/错类型均消费者读取0，原CLI受控拒绝；合法层消费并通过。四参数负例unknown，无冲突native正例保留。四manifest形状错误no-traceback并验证制品/manual原字节不变。
实际归档链接是最终merge门，不能用active目录链接代替。纯Python/docs禁止cargo/docs_gen；若范围需要改变先报告，不擅自扩大或预写pass。


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


Phase 2 追加（plan_revision 2；原 AC-01..08 不移除）：

- [ ] AC-09：多行字面量里的 pretend()/伪 fn/use 不进入调用/声明/import，行号与关闭后真实代码正确；本地 Meter.len/new、CG.new/Ar.new 不误标宿主 native，未知/同名接收者保留 unknown。真实重扫与新增反例逐项有证据。
- [ ] AC-10：人工决定的空/缺绑定、缺证据、非法结构/版本/重复 ID/未闭环覆盖不能通过；严格完成态 --require-decisions 对空/缺人工层非零，有效完整人工层成功，early scan-only 的 absent 状态保留但不当人工层 ok；重跑不覆盖决定。
- [ ] AC-11：两个 pass 合同及阶段摘要一致保持精确数值、运行期 trap、observable 顺序、来源、凭证/再校验；“必经路径常量溢出”不能一般性改为编译期拒绝。保守 effect 术语方向无歧义；新增语义反例齐全，不宣称 pass 实现。
- [ ] AC-12：10 锚点全保留，HIR existing 与源码 gated/待建明确分开；两候选交付/前置/语料匹配，无相互依赖才能验收的循环，A8 str/List 和 A9 enum/is 有能力负责人及门槛，不抢占新编号、不扩大本计划实施。
- [ ] AC-13：当前 8 条决定/9 处过期绑定逐项重审，扫描变更后新观察及 unknown 的决定覆盖完整；严格 --check 当前真实报告绿，输入/人工证据 hash 新鲜，语义变化/不变理由可溯。Spec 落地后再做重绑与严格冒烟，不 blind hash update/exempt。
- [ ] AC-14：T-01..06 历史记录和完成勾选一致，r2 T-07..14 进度真实；旧 18+新增测试、确定性/链接/格式/范围检查及 r2 独立复审证据完整，SD-04/05 冻结后才 merge；活动/归档位置与 metadata/指针一致，不复用旧 pass。

Phase 3 追加（r3；原 AC-01..14 全保留，受影响项重验）：

- [ ] AC-15：当前 owner/本地及导入同名证据优先；不能证明归属的点/管道/接收者保持 unknown。宿主不冲突/当前 owner 正例绿，shadowed_host/different_owner/unrelated_dot_pipe 负例类别正确；新扫描/决定不靠固定数量断言。
- [ ] AC-16：同条 evidence 每个文件都有准确 binding；缺失定位拒绝、绑定正确成功、证据变动 stale，非扫描输入证据亦受控。当前完整真实人工层严格门仍绿。
- [ ] AC-17：合法 JSON 中所有 required decision/family 字段及元素错误类型均受控诊断，无 traceback；含原报告四个崩溃反例，有效数据不误拒绝。
- [ ] AC-18：resolved 族有适用、非空、存在的决定引用与证据；open 有具名 owner/probe/work_package；缺失/不适用/状态不符引用不能宣称闭环，真正完整层成功，unknown 保持可研究状态。
- [ ] AC-19：canonical 提案/阶段摘要/设计正文的 R3、runtime trap/X9 一致，真实 CLI/锚点/候选对齐；SD-06/07 经独立 review 后才 merge，不改 741/优化实现。
- [ ] AC-20：r2 历史 T-14/counters 有据修正；r3 pending/完成与计数准确，独立报告绑定 revision/HEAD；最终 active/archive/索引/ledger/链接/清理收据一致，旧提案/报告/收据保留。Category A 全门禁通过，无未批准缩减。

Phase 4追加（r4；原AC-01..20不删除，受影响项重验）：

- [ ] AC-21：被resolved/open族引用的无效决定不会被后续消费者再次使用；类型/缺字段/重复ID组合受控拒绝、定位ID/索引/族，无traceback；有效完整层通过。
- [ ] AC-22：resolved适用性必须由同条evidence覆盖族路径，subject/hash不能替代；两条旁路负例失败，evidence+binding正确/变化三段式通过，真实七族及新变化层严格绿。
- [ ] AC-23：导入/参数/已知局部绑定同名时不误判native；未确定身份保持unknown，无冲突宿主、当前owner与既有修复正例不退化；真实观察/人工决定逐项核准，不依赖固定数量。
- [ ] AC-24：历史T-21据实完成、新T-22..27进度真实；旧63+新增/确定性/严格门/链接/范围/独立复审证据齐全；SD-08冻结后review再merge，final27/27及全部P743-*指针/active/archive/module索引一致，保留所有历史报告和收据。

### Phase 5 验收（新增AC-25..28）

- AC-25：语义非法/绑定过期/重复ID（前后顺序）不入可信索引或失败后停止消费，消费者读取/覆盖贡献0；受控非零，完整合法层pass。闭环旧AC21/T23。
- AC-26：mut/free/method/bare四反例均不升级native，普通参数、导入/局部绑定、合法owner和无冲突宿主正例保留；真实差异逐条审定无机械hash重刷。闭环AC23/T24。
- AC-27：manifest为[]/1、source_identity为[]/null均受控ERROR/nonzero/no traceback，无输入/manual修改；保持audit-only HEAD正常行为及source/summary漂移检出。
- AC-28：旧83+新增、严格门、确定性/manual/hash/SD09/范围均验证，独立review绑定r5和当前修复HEAD；pass后实际archive33/33、链接/meta/模块导航/全部P743-*指针、债项/guard收据闭环。旧历史记录保留，contract≠implementation。


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
- [x] T-05 验收与后续合同/Spec提案（依赖T-04；AC-06, AC-07）。
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
- [x] T-06 复核与独立交接（依赖T-05；AC-08）。
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


### Phase 2 执行步骤（T-07..T-14，新任务尚未执行）〔起草历史标题：r2 合同起草时的待办状态，现 T-07..14 均已完成，见各任务行与收据〕

实施基于含本合同的 v0.6-dev 新提交，专用 D:/autostack/.wt/lang-743/auto-lang、plan-743-dev。
本轮不创建实施 worktree，不修代码；r1 current_step 6 的历史进度保留，总步骤改为 14。

- [x] T-07 固定 r2 基线与工作树（无任务前置；AC-14）。
  /auto-plan:work 核对唯一 ID、741/ABI 并行范围和全部指纹，记录完整激活提交，重建专用分支/检出。
  复跑报告反例，新增 docs/reports/743-phase2-quality-fixes/ 验证资料目录（新路径）；无外仓、无链接。
  [✅ 已完成] worktree commit 2ecccea12；激活/实施基点 c6e4e5689（worktree lang-743 重建，
  branch plan-743-dev）；baseline.md 记录并行范围核对（741 P3 独立、.next-id=744 未占）
  与 QA-01/02/03/06 反例 before 状态逐项复现（pretend 泄漏、x.len/Meter.new 误标、
  真实仓 CG.new×5/Ar.new×1、CHECK-FAIL 9 项）。
- [x] T-08 修词法屏蔽与调用分类（依赖 T-07；AC-01/02/03/09）。
  scripts/acc_inventory.py::mask_comments_and_strings/scan_module/_post_classify；
  scripts/tests/test_acc_inventory.py 与 fixtures/acc-inventory/ 新反例；验证 --write 观察及单测，
  多行伪代码零污染、自定义/未知调用不误标 native、旧扫描保护正例保持。
  [✅ 已完成] commit 212183813：mask 跨行保持字面量态（转义续行/EOF 兜底标记词法不确定/
  char 换行恢复三态分立）；分类只认可证明证据（NATIVE_NAMESPACES 白名单，NATIVE_METHODS
  兜底移除），Meter.new/CG.new/Ar.new=type-qualified、x.len=unknown、dot/pipe 隐式 self
  仅经本地方法集升级；10 新测试 + 旧 18 全绿（28），分类政策更新断言按新合同修正。
- [x] T-09 人工层严格完整性门（依赖 T-08；AC-02/03/10）。
  validate_decisions/main 新增 --require-decisions、schema/hash/evidence/coverage 校验与定位诊断；
  用决定空绑定/证据不存在/无人工层/重复 ID/非法记录/unknown 未分配去向等负例，
  验证早期 scan-only 兼容、严格 gate 拒绝不完整且 --write 不触人工文件。
  [✅ 已完成] commit b62510bca：DECISION_KIND_CONCLUSIONS 四类合法结论集、七必填字段、
  唯一 ID、非空绑定、证据文件存在（升级为 ERROR）；--require-decisions 严格门=人工层
  存在非空 + 输入全覆盖 + unknown_families 双向闭环（open 族强制 owner/probe/work_package，
  孤儿族拒绝）；scan-only 明示非完成态；13 新负例定位决定 ID（41 测试全绿）。
- [x] T-10 pass/阶段合同消歧（依赖 T-07；AC-04/05/11，SD-05）。
  docs/design/strategy/auto-acc-bootstrap-contract.md 澄清 trap、effect 与事实/目标态边界；
  明确 mark_a→必经溢出和不可达 trap 的预期，不实现优化器。准备 phase2 proposed delta 对应正文。
  [✅ 已完成] commit ae63a88b8：R3 改为效果分析（保守出发、证明才精化）与效果语义保持
  （双向禁止，含 trap 与前驱 observable 顺序）两分；eval.const-fold v2 运行期语境溢出
  一律保持运行期 trap、comptime 域出外、无编译期诊断；X9 反例入册；实现状态边界
  （模板/示例=目标态，阶段表现状列为唯一权威）。
- [x] T-11 锚点与候选前置对账（依赖 T-10；AC-06/07/12，SD-04）。
  报告 acceptance-matrix.md/next-work-packages.md/inventory.md：路径与状态逐项核实，
  两候选最小范围/能力/语料一致，gated owner 具体，A8/A9 不预先称源码已实现；估算仅核查不伞形加任务。
  [✅ 已完成] commit ae63a88b8：A2 拆 HIR existing/源码 gated 两态；A8 owner=运行时首批
  候选（原误记候选①首任务）、A9 owner=聚合能力线（未编号，前置=enum/record/is）；
  候选①正向集剔除 A8、②剔除 A9；依赖单向 ①→② 无环声明入册。
- [x] T-12 当前证据与决定校准（依赖 T-08..T-11；AC-01/02/03/13，SD-04）。
  逐条复核 8 决定/9 stale 绑定及扫描分类变化，更新 manual-decisions.json/inventory.md，
  --write 新观察、严格 --check 绿；新增未知分类显式链接决定/owner，不机械替换 hash。
  [✅ 已完成] commit ae63a88b8：8 决定/9 绑定逐条审定（对照 741 r2/r3 spec 差异：
  CLI 更名 auto-ac-prototype、构建原子性/截止时间、verify 归属唯一、bindings 双射——
  结论全部维持，各附具体依据后重绑）；MD-407/506 按 QA-01 修正更新（r1"恢复正确"
  判定被取代，保留为政策变更记录）；新增 MD-507 变量接收者族裁定；unknown_families
  7 族闭环 146 组；严格 --check 绿（47 决定全部新鲜）。
- [x] T-13 终验与 Spec 提案冻结（依赖 T-12；AC-01..14，SD-04/05）。
  全 18+新增 Python 用例、生成确定性、真实严格三命令、links/diff/范围检查；
  新 proposed-spec-delta-phase2.md 冻结 SHA-256，对账所有 AC；簿记主检出回写 execution_done。
  [✅ 已完成] commit a63660137：§6 四命令全绿（41 测试/--write/严格 --check/git diff
  --check）+ 确定性三方 cmp + 围栏感知链接检查 + 范围探针零触碰；phase2 提案冻结
  a963adfbd6b24887，制品冻结哈希与 AC-09..14 对账见
  docs/reports/743-phase2-quality-fixes/verification.md；本文件回写 execution_done
  （current_step 13/14，T-14 留待复审/合入阶段闭环）。
- [x] T-14 独立复审与合入交接（依赖 T-13；AC-08/14）。
  /auto-plan:review 重放原/新反例与严格门，绑定 r2/HEAD、逐 AC/SD/遗漏复核；
  pass 后交 merge 沉淀 SD-04/05、逐条销账、Spec 改 hash 后再审定绑定并严格冒烟，
  实际 archived/归档/指针一致，规定 guard clean 后才清理 worktree。

T-07..14 不以合同就绪代替实施完成；旧 r1 pass 不覆盖新实现或新 Spec 增量。

### Phase 3 执行步骤（T-15..T-21，尚未实施）〔起草历史标题：r3 合同起草时的待办状态，现 T-15..21 均已完成〕

| ID | 前置 | 工作与路径 | 验证/结果 | AC |
|---|---|---|---|---|
| T-15 | r3 合同提交 | 核对唯一 active 743/最新基线和新报告 hash；重建 D:/autostack/.wt/lang-743/auto-lang、plan-743-dev；新建 docs/reports/743-phase3-integrity-fixes/ 留 before/evidence | worktree/status 无 WIP/无 links；复现本报告反例，不占号/不动其它工作线 | AC-20 |
  [✅ 已完成] commit e0008648b；基点=含 r3 合同的 a0f8ddd44（worktree 重建、主检出 0 WIP）；baseline.md 复现 R2-QA-01 三 fixture/类型 traceback/绑定与引用缺口 before 状态，并行范围核对（ed2d00b90 sqlite 不属 743、.next-id=744 未占） |
| T-16 | T-15 | scripts/acc_inventory.py::scan_module/_post_classify 当前 owner/冲突规则；tests/fixtures 正反例 | native/owner 不凭全模块同名猜测；旧 41 正例与合法多行保护继续过 | AC-01/02/03/09/15 |
  [✅ 已完成] commit 45fe4fb21：qualified/点/管道归属推迟至 _post_classify（最终本地集+宿主优先级+enclosing owner 证据）；本地 type IO/fn print 遮蔽宿主；隐式 self 仅当前 owner 方法集升级；4 fixture 测试+owner 正例修正（44 绿） |
| T-17 | T-15 | validate_decisions 完整类型与同条 evidence-binding 校验；参数化错误数据/变化探针 | ERROR/code/ID 定位且无 traceback；绑定完整成功/内容改变 stale | AC-02/03/10/13/16/17 |
  [✅ 已完成] commit a25a30722：字段/元素类型前置校验（9 参数化负例受控 ERROR 定位、零 traceback）；decision-evidence-unbound 同条绑定门（补绑定过/内容变化 stale） |
| T-18 | T-16,T-17 | resolved/open 引用关系和适用规则；manual-decisions.json/inventory/新观察逐项核准，不机械 hash 更新 | 缺/空/不存在/不适用引用拒绝；所有真实 unknown 有去向，严格门绿；unknown 可 open | AC-03/10/13/18 |
  [✅ 已完成] commit a25a30722：resolved 族须引用存在且适用决定（unknown-resolution/resolved+证据覆盖族路径，family-ref-missing/-inapplicable）；open 三要素保留；真实输入分类与 r2 零差异（点/管道均在合法 owner 内），人工层无需变更，严格门绿（47 决定/146 组/7 族） |
| T-19 | T-18 | 设计/矩阵/候选命令与 R3 摘要提案；新建 phase3 delta，更新状态/路径检查（行为 canonical/ledger 留 merge） | SD-06/07 冻结、CLI 当前名称、历史 T-14 与新 pending 区分、链接模拟归档可解析 | AC-04/05/06/07/11/12/19/20 |
  [✅ 已完成] commit 9f634cb52 + 主检出簿记：acceptance-matrix/inventory/run-context 全量更名 auto-ac-prototype（历史处注记）；proposed-spec-delta-phase3.md（SD-06/07 正文）；计划文件 4 处链接改归档可解析（../../reports/，模拟归档验证通过）；plans.md 743 行 r3 executing（激活提交已更）；T-14 补勾在案（激活提交） |
| T-20 | T-16..19 | 旧 41+新增、双输出确定性、真实严格门、manual 不变、links/diff/scope，冻结报告与全 AC/SD 对账 | Category A 门禁全绿；旧源码/根 Cargo/741/外仓未改，任何红逐项归因 | AC-01..20 |
  [✅ 已完成] commit 184749168：PYTHONUTF8=1 全门禁绿（63/63 测试、manual --write 字节不变、严格 --check 47 决定/146 组、三方 cmp、diff --check、围栏感知链接、范围零触碰、CLI 残留 0）；phase3 提案冻结 a4af013b759a1d35；verification.md 全 AC 对账 |
| T-21 | T-20 | /auto-plan:review 独立重放并绑定 r3/HEAD；pass 后 /auto-plan:merge 落 SD-06/07/销账/索引ledger/归档guard | 不预写 pass；最终完成=21/21、归档 metadata/链接/指针一致，无自有 worktree 残留 | AC-08/19/20 |
  [✅ 历史已完成，2026-10-05本轮簿记修正] r3 R4复审、deee10031落地及32e1e8648/8e8f6f19b归档清理收据均在案；旧20/21是遗漏，按工作流完成补为21/21，代码缺口另由Phase4修复。

当前 completed T-01..14=14；总 21，T-15..21 待执行。T-14 补勾只确认 r2 工作流确实落地，不否认本轮 needs_fix。

### Phase 4 执行步骤（T-22..T-27，待实施）〔起草历史标题：r4 合同起草时的待办状态，实施进度见任务行〕

- [x] T-22 基线与重放（AC-24）：从含r4合同的最新v0.6-dev重建 D:/autostack/.wt/lang-743/auto-lang / plan-743-dev，核对唯一ID/hash/WIP/并行边界；新建 docs/reports/743-phase4-consumer-fixes/ 留before证据，不新取号/不建links。
  [✅ 已完成] commit 0967bc844；基点=含 r4 合同的 1d8fba15c（worktree 重建、主检出 0 WIP）；baseline.md 复现 R3-QA-01 traceback（MD-REVIEW-900 组合路径）、R3-QA-02 hash 旁路（完整层 exit0 隔离证实）、R3-QA-03 两 fixture 误判、R3-QA-04 生命周期项；并行边界核对（741/742/ABI 独立、.next-id=744 未占）
- [x] T-23 校验记录引用（依赖T-22；AC-02/10/17/18/21）：修改validate_decisions消费者/索引边界；新增被引用的各字段非法、缺失/重复ID与resolved/open组合负例。非零ERROR定位、无traceback，有效层绿。
  [✅ 已完成] commit 62329eadb：validated-only 消费者索引（类型淘汰/重复 ID 不入索引），族引用被淘汰记录报"未通过校验"定位；MD-REVIEW-900 组合路径受控拒绝零 traceback（测试锁定）
- [x] T-24 证据与宿主判据（依赖T-23；AC-01/02/03/09/10/15/18/22/23）：移除evidence相关性subject/hash旁路；qualified判定纳入导入/参数/已知局部绑定。新增正反测试，核准变化unknown与人工决定；不实施完整resolver。
  [✅ 已完成] commit 62329eadb：适用性=evidence 文件集合（bound/subject OR 旁路移除），hash-only/subject-only 负例拒绝；qualified 判定纳入导入/参数/let-var 绑定遮蔽（import_shadow/parameter_shadow 负例 + 无冲突宿主正例）；真实仓观察零漂移（13 native/687 unknown 不变），人工层无需变更
- [x] T-25 合同/生命周期（依赖T-24；AC-19/20/24）：新建phase4提案SD-08；历史标题/起草数字消歧，module导航/r4进度；实现final归档counter和所有ledger历史指针验证方法，行为Spec/ledger留merge。
  [✅ 已完成] commit b9b77ecc8 + 主检出 638190ccf：proposed-spec-delta-phase4.md（SD-08 正文）；重复 R4 标题去重 1 处、两个起草态标题标注历史；final_assertions.py 交付（归档 27/27、P743-* 全指针、plans.md 行、canonical、严格门七类断言，merge 收口执行）
- [x] T-26 终验与冻结（依赖T-23..25；AC-01..24）：旧63+新增、双生成/manual不变、当前真实严格门、links/diff/source范围与hash；冻结SD-08及AC逐项证据，簿记execution_done；不能预写独立pass。
  [✅ 已完成] commit af3c8040e：PYTHONUTF8=1 全门禁绿（83/83 测试、manual --write 字节不变、严格 --check 47 决定/146 组/7 族、三方 cmp、diff --check、围栏感知链接、范围零触碰、CLI 残留 0）；SD-08 冻结 f8924d0a8266976d；verification.md 全 AC 对账；本文件回写 execution_done（current_step 26/27，T-27 留待复审/合入闭环）
- [x] T-27 独立review/merge收口（依赖T-26；AC-08/24）：绑定r4/HEAD重放原新反例；pass后沉淀SD-08、逐项销账/索引/所有P743-*指针/归档guard。按历史+新task核对27/27与链接/路径；不触741/742/ABI工作树。
  [✅ 已完成] R5 pass（f6a945db8，绑定 r4/af3c8040e）；merge 收据 PLAN-743:r4（union-rebase 映射、§5.6.6 逐条重绑 2c6c3787d、SD-08 沉淀 deee..→5e4e349f8、P743-3 指针对账、P743-6）；归档 a68dabc0f；final_assertions.py 七类断言全过（27/27、P743-* 指针、索引、canonical、严格门）。

当前完成T-01..21=21，总27；新T-22..27待实施。本轮未创建执行worktree或修Python。

### Phase 5 执行步骤（T-28..33，未实施）

- [x] T-28 基线重建与复现（AC25..28）：在含r5合同的最新v0.6-dev新建同计划工作树D:/autostack/.wt/lang-743/auto-lang、plan-743-dev；核对HEAD/唯一ID/并行边界/.next-id；新phase5报告留四问题before及有效对照。复审临时detached树不作为实施树，禁止links。
- [x] T-29 可信索引与消费（依赖28，AC25）：修改validate_decisions，完整类型/语义/绑定/新鲜度/唯一ID门；重复整组作废或失败停消费；新增consumer0及合法对照、两个重复顺序。重验并关闭旧T23。
- [x] T-30 参数绑定与宿主分类（依赖28，AC26）：普通/mut自由函数和方法/静态签名；bare/qualified共同绑定优先；四反例unknown+原有正例；审定真实分类变化与对应人工决定，不固定数量。重验关闭旧T24。
- [x] T-31 manifest形状诊断（依赖28，AC27）：成员访问前容器验证；四错形状ERROR/no traceback/文件字节不变；正常漂移/audit-only positive与旧全部测试。
- [x] T-32 合同和最终验证器（依赖29..31，AC28）：新phase5 SD09冻结；新final_assertions检查actual active及模拟/真实archive链接/33任务/meta/导航/all P743-*。保留旧helper；不先写canonical/ledger。重验关闭T25。
  [✅ 已完成 r5] T-28: commit 7425a707e（基点 39a4f9133，baseline.md 复现四反例+两对照+形状崩溃+5 条 stale 前置）；T-29: commit 739c7aa57（validated-only 索引扩充：非法结论/证据缺失未绑定/过期绑定不入消费者索引，重复整组作废含先插入者）；T-30: commit 739c7aa57（签名提取覆盖方法+mut 名修正；BARE_NATIVES 推迟 post-classify、绑定遮蔽优先；四反例不升级 native、对照不退化；真实仓零漂移；5 条 AutoAC stale 对 741 P4 diff 逐条审定重绑）；T-31: commit 739c7aa57（manifest 顶层/source_identity/inputs 形状门，5 形状负例受控拒绝零 traceback）；T-32: commit efeb03a60（SD-09 冻结 ad3e02d7 核对一致；final_assertions r5：33 任务双格式/真实父目录链接检查/P743-* 指针/canonical r3+r4+r5 检查/严格门）。终验 110/110 测试、严格门、三方确定性、范围零触碰，详见 docs/reports/743-phase5-fixes/verification.md。
- [ ] T-33 终验、独立review和merge（依赖29..32，AC25..28）：83+新增、replay、strict、双生成/三方cmp/manual/hash/links/diff/范围；先execution_done不预写pass，独立review绑定r5/HEAD。pass后SD09沉淀/债项逐条销账/ledger all refs/actual archive33/33/guard cleanup收据闭环，关闭旧T26/T27；保护741/742/ABI工作树。

当前T01..22完成22/33，T23..27重新待验，T28..33未实施；新Phase不能以补勾/历史pass代替验证。


## 9. 复审记录

### 独立复审 R6（2026-10-06，stage: review，r5 P743-R4-QA-01..04 修复复核）

- stage: review
- plan_id: PLAN-743 · plan_revision: 5 · outcome: **pass**（R4-QA-01..04 修复全部确认；
  AC-01..24 按 r5 语义重验通过；AC-25..28 全部通过）
- reviewed_commit: ce61e5e6d73f9a6b43ce2cb41b2ff534924e133f（worktree plan-743-dev，
  clean 0 dirty）；base_commit: 39a4f91339ac9744578cf2064c9731090469e960（激活提交）
- dependency_revisions: Python 3.14.2 标准库；无外仓；PYTHONUTF8=1
- spec_inputs: 9 受管输入 hash 经严格 --check 全绿；r5 对 docs/specs/ 零改动
  （SD-09 为提案态，冻结 ad3e02d7a6c237f8 与 delta-hashes.json 一致）
- 独立性声明：复审在实现会话内进行（同 R1..R5 先例）；结论由命令重放、CLI 级形状
  探针、单元级反例复现与源码核查重建。

**R4-QA 修复逐项复核（独立重放）**：

- R4-QA-01 → fixed：六 fixture 独立重放——mut/method/bare/method-bare 四反例全部
  unknown（不升级 native）；usual_parameter 对照 unknown、conflict_free 对照 native；
  真实仓 engine 宿主正例（List/IO/File/process）零漂移；源码核查：签名 harvest 覆盖
  自由 fn+类型体方法、mut 名取 mut 后 token、BARE_NATIVES 判定推迟 post-classify。
- R4-QA-02 → fixed：四形状（顶层 []/1、source_identity null/[]）CLI 级探针全部
  exit 1 + ERROR[manifest-malformed] + 零 Traceback；inputs/manual 探针前后字节不变。
- R4-QA-03 → fixed：源码核查 record_bad 流（非法 conclusion/证据缺失/未绑定/过期
  绑定均排除出消费者索引）+ 重复 ID 整组作废（validated_by_id.pop）；三组合测试
  （非法结论引用/过期绑定引用/重复 ID）全绿。
- R4-QA-04 → fixed（工具态）：final_assertions r5 交付（33 任务双格式、计数 33/33、
  收据 key PLAN-743:r5、Markdown 链接自实际父目录解析围栏排除、P743-* 全指针、
  canonical r3/r4/r5 三代检查、严格门）；归档态 6 断链待 merge 归档前统一改
  ../../reports/ 后由该断言复核（激活态暂时可解析为既定模式）。

**门禁重放（reviewed_commit 上）**：110/110 测试；严格 --check --require-decisions
exit0（47 决定/146 组/7 族，含 5 条 AutoAC stale 经 741-P4 diff 审定重绑后新鲜）；
三方 cmp BYTE-IDENTICAL；`git diff --check` clean；范围探针 crates/auto-lib/
docs.specs/experimental/Cargo 零触碰。

**AC 终态**：AC-01..24（前轮集合，按 r5 语义重验）pass；AC-25/26/27/28 pass
（AC-28 的 33/33 终态断言为 merge 门，final_assertions 兜底，失败即 blocked）。
**delta 复核**：SD-09（modify auto-acc/project.md，仅细化兑现 SD-08、无新阶段/pass）
冻结一致；supersedes_spec_components=[]、new_spec_components=[]、touched_goals=[]
（r5 合同 §Phase 5 规范增量表说明在案）。

**收据冻结哈希（reviewed@ce61e5e6d）**：phase5 提案 ad3e02d7a6c237f8、
manual-decisions.json 45e04e309fbaeeab、acc_inventory.py 9d315e657a2cd1a0、
test_acc_inventory.py 85c6a63d40fea447。

- findings: 无工具/合同缺陷；簿记卫生 1 项随本记录处理——激活提交引入的连续重复
  "### 独立复审 R5" 标题行已去重（deduped=1，透明注记不改历史 verdict）；
  plans.md 行 executing/r5+active 指针为执行期正确状态，merge 时置归档
- evidence: 本记录命令/结果摘录；verification.md/baseline.md（随交付落地）；
  制品经 merge 落 v0.6-dev 后同 hash 可溯
- next: merge（沉淀 SD-09；P743-3..6 指针归档对账；执行 final_assertions r5
  （33/33、链接、全指针）；逐条销账、归档、guard 后清理）

### Phase 5 执行交接（2026-10-06，stage: work，r5 P743-R4-QA-01..04 修复）
### Phase 5 执行交接（2026-10-06，stage: work，r5 P743-R4-QA-01..04 修复）

- stage: work（r5；外部复核 needs_fix 后的消费者完整性修复实施）
- plan_id: PLAN-743 · plan_revision: 5
- outcome: pass（T-28..T-32 完成 + T-23..27 据 r5 修复重验；非独立复审结论）
- code_commit: ce61e5e6d（worktree plan-743-dev；基点=激活提交 39a4f9133；
  r5 提交链 7425a707e→739c7aa57→efeb03a60→ce61e5e6d，worktree clean 0 dirty）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留待 review/merge）
- task_ids: T-28..T-32 完成、T-23..27 重验（r4 证据保留 + r5 修复覆盖新发现）；
  current_step 32/33；T-33（独立复审与 merge 收口）按定义留待 review/merge 闭环
- evidence: PYTHONUTF8=1 全门禁绿——110/110 测试（83 旧+27 新）、--write 不触人工层
  （字节不变）、严格 --check --require-decisions（47 决定新鲜；5 条 AutoAC stale 对
  741 P4 diff 逐条审定重绑非机械替换）、三方确定性 cmp、git diff --check clean、
  围栏感知链接无断链、范围探针 crates/auto-lib/docs.specs/experimental/Cargo 零触碰；
  R4-QA-01..04 反例 before→after 逐项复现并测试锁定（baseline.md/verification.md）；
  SD-09 冻结 ad3e02d7a6c237f8（复审方交付，核对一致）；final_assertions r5 交付
  （33 任务/真实父目录链接/全指针/canonical/严格门）
- blockers: 无
- next: /auto-plan:review（r5 独立复审：绑定 revision 5 + 实现 HEAD ce61e5e6d，
  重放原/新反例与严格门）；pass 后 merge 沉淀 SD-09、P743-3..6 指针归档对账、
  执行 final_assertions（33/33）、逐条销账、归档、guard 后清理。
  本记录不替代复审；不自行归档/合入。

### Phase 5 再激活 / 修订合同交接（见激活提交 39a4f9133）
### 合并沉淀收据（2026-10-05，stage: merge，key: PLAN-743:r4）

- stage: merge · plan_revision: 4 · outcome: pass
- reviewed 基线：R5 pass @ af3c8040e（r4，delta 冻结 f8924d0a8266976d）
- 落点分支：v0.6-dev（主线约定）

**checkpoint 实证**：

| checkpoint | 证据 |
|---|---|
| prepared | worktree 内按冻结提案逐字落 SD-08（auto-acc/project.md 分类政策条目替换+严格门附加保证段，diff +14/−7）；plans.md 两行归档态 + 账本 P743-1 原位刷新/P743-3 指针对账 archive/reviews P743-6；commit b28cc8ba5（reviewed_commit 的纯文档后代） |
| landed | rebase 遇并行线 PLAN-741 P3 同批文件落地（.autoos/specs.json 与 auto-hir/plans.md UU）——union 解决（双方条目/行均保留：P741-1..5+P743-1..6 共存、741 行 delivered + 743 行 archived），range-diff 提交 1–4 补丁等号、提交 5 为合法 union（语义核验：SD-08 正文/P743 指针/741 条目/plans 行全对）。旧→新映射：0967bc844→63a882097、62329eadb→569b4c2f9、b9b77ecc8→7de5b83ed、af3c8040e→f9b86e4f6、b28cc8ba5→5e4e349f8。ff-only 落地；§5.6.6 条款触发：741 P3 改变 8 决定/9 绑定的两份 spec hash（块包含图 DFS 门/发布回滚/Job Object 截止）——逐条审定结论维持后 spec-only 重绑（commit 2c6c3787d；首次脚本误覆源码绑定即发现并 git 还原重做），主检出严格冒烟 83/83 测试+严格门绿；delivery 终点=2c6c3787d |
| ledger_refreshed | .autoos/specs.json P743-1 原位刷新（r4 强化注记）+ P743-3 指针对账 archive + reviews P743-6（外部 needs_fix→Phase 4→R5 pass 链），随 delivery 落主检出，read-back 验证 P743-1..6 在案且 741 条目无损 |
| archived | 本文件 git mv 至 docs/plans/archive/，status: archived，completion_kind: delivered |
| cleaned | worktree 0 dirty 且 HEAD 898530d57 已是 v0.6-dev 祖先；wt-guard.sh 仍缺失（r1..r3 收据已挂工具债）——等价 PowerShell ReparsePoint 全组扫描：`GUARD-EQUIVALENT-CLEAN: no reparse points in lang-743 group`；git worktree remove 目录删除首次遇 Windows 瞬时句柄 Permission denied（注册已注销，与 r3 同款），重试后目录/`git branch -d plan-743-dev`（was 898530d57）/组目录三方复核清零；final_assertions.py 七类断言在归档+T-27 收口后全过 |

- 规范增量落地：SD-08→docs/specs/auto-acc/project.md（modify；canonical 落地文本与
  冻结提案逐字一致，R5 复核 f8924d0a 未触）。
- final_assertions.py（T-25 交付）在本收据提交后执行，七类断言全过为归档有效条件。

### 独立复审 R5（2026-10-05，stage: review，r4 P743-R3-QA-01..04 修复复核）

- stage: review
- plan_id: PLAN-743 · plan_revision: 4 · outcome: **pass**（R3-QA-01..04 修复全部确认；
  AC-01..23 在 r4 语义下重验通过；AC-24 的终态断言（27/27、全部 P743-* 指针）为
  merge 收口条件，由 final_assertions.py 在落地/归档后执行，失败即 blocked）
- reviewed_commit: af3c8040e5a9995938a5004c4739539d9be335a9（worktree plan-743-dev，
  clean 0 dirty）；base_commit: 1d8fba15c7f8685e670f3bc8a398198895331c72（激活提交）
- dependency_revisions: Python 3.14.2 标准库；无外仓；PYTHONUTF8=1
- spec_inputs: 9 受管输入 hash 经严格 --check 全绿；r4 对 docs/specs/ 零改动
  （SD-08 为提案态）
- 独立性声明：复审在实现会话内进行（同 R1..R4 先例）；结论由命令重放、CLI 级
  组合探针与源码核查重建。

**R3-QA 修复逐项复核（独立重放）**：

- R3-QA-01 → fixed：CLI 级组合探针（完整层 + 被引用决定 evidence=17）→ exit 1、
  ERROR[decision-malformed] 定位 MD-S-900、无 Traceback；源码核查消费者索引=
  validated_by_id（仅通过全部类型/语义校验的决定），重复/淘汰记录引用报
  "未通过校验/不存在"。
- R3-QA-02 → fixed：源码无 bound/subject OR 旁路（字符串级核查）；hash-only 与
  subject-only 两个隔离负例（83 测试套内）均 family-ref-inapplicable；真实七族
  evidence 均覆盖族路径，严格门绿。
- R3-QA-03 → fixed：import_shadow（4 qualified→unknown、print=imported-symbol）、
  parameter_shadow（IO 参数.read_line→unknown）独立重放；无冲突宿主正例不变；
  真实仓观察零漂移（13 native/687 unknown）。
- R3-QA-04 → fixed（提案+工具态）：重复 R4 标题去重（独立 grep 计数=1）、3 处
  起草态标题标注历史、final_assertions.py 交付（七类断言：归档 27/27、任务勾选、
  全部 P743-* 指针可解析且 P743-3 对账 archive、plans.md 行、canonical 无旧摘要、
  严格门）；merge 收口执行，失败即 blocked。

**门禁重放（reviewed_commit 上）**：83/83 测试；严格 --check --require-decisions
exit0（47 决定/146 组/7 族）；三方 cmp BYTE-IDENTICAL；`git diff --check` clean；
范围探针 crates/auto-lib/docs.specs/experimental/Cargo 零触碰。

**AC 终态**：AC-01..23 pass；AC-24 的可预验部分（T-22..26 进度、证据、SD-08 冻结、
final_assertions 交付）pass，终态断言为 merge 门（blocked 机制兜底）。
**delta 复核**：SD-08 提案与冻结哈希一致（f8924d0a8266976d）；正文为持久规则
（收紧不放宽、不声称 resolver）；元数据经 r4 合同 §Phase 4 规范增量表确认。

**收据冻结哈希（reviewed@af3c8040e）**：phase4 提案 f8924d0a8266976d、
manual-decisions.json 3674c687647f88dc、acc_inventory.py 074a32853df94b7f、
test_acc_inventory.py 9f9b3681c2b5c029、final_assertions.py a8e629f87fa84b3d。

- findings: 无新增
- evidence: 本记录命令/结果摘录；verification.md/baseline.md（随交付落地）；
  制品经 merge 落 v0.6-dev 后同 hash 可溯
- next: merge（沉淀 SD-08；P743-3 指针对账 archive；执行 final_assertions.py 七类
  断言；逐条销账、归档、guard 后清理）

### Phase 4 执行交接（2026-10-05，stage: work，r4 P743-R3-QA-01..04 修复）
### Phase 4 执行交接（2026-10-05，stage: work，r4 P743-R3-QA-01..04 修复）

- stage: work（r4；外部复核 needs_fix 后的消费者完整性修复实施）
- plan_id: PLAN-743 · plan_revision: 4
- outcome: pass（T-22..T-26 完成、验收映射成立；非独立复审结论）
- code_commit: af3c8040e（worktree plan-743-dev；基点=激活提交 1d8fba15c；
  r4 提交链 0967bc844→62329eadb→b9b77ecc8→af3c8040e，worktree clean 0 dirty）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留待 review/merge）
- task_ids: T-22..T-26 全部完成（证据见 §8 任务行与 verification.md）；
  current_step 26/27；T-27（独立复审与合入收口）按定义留待 review/merge 阶段闭环
- evidence: PYTHONUTF8=1 全门禁绿——83/83 测试（63 旧+20 新）、--write 不触人工层
  （字节不变）、严格 --check --require-decisions（47 决定新鲜、146 组/7 族含
  evidence-only 适用性）、三方确定性 cmp、git diff --check clean、围栏感知链接
  无断链、范围探针 crates/auto-lib/docs.specs/experimental/Cargo 零触碰、CLI 残留 0；
  R3-QA-01..04 反例 before→after 逐项复现并测试锁定（baseline.md/verification.md）；
  SD-08 冻结 f8924d0a8266976d；final_assertions.py 交付（merge 收口执行七类断言）
- blockers: 无
- next: /auto-plan:review（r4 独立复审：重放原/新反例与严格门，绑定 revision 4 +
  实现 HEAD af3c8040e，逐 AC/SD 复核）；pass 后 merge 沉淀 SD-08、执行
  final_assertions.py（27/27 与全部 P743-* 指针断言、P743-3 指针 archive 对账）、
  逐条销账、归档、guard 后清理。本记录不替代复审；不自行归档/合入。

### 合入后独立复审 R5（2026-10-05，review r3）

- stage: review | plan_id: PLAN-743 | plan_revision: 3 | outcome: **needs_fix**
- reviewed_commit: 8e8f6f19b96516fdbc9d4c893297fa1757a7aad4；base_commit: a0f8ddd443418fb32a67d308cbd4a0fd00f74835。
- dependency_revisions: Python3.14.2/标准库/PYTHONUTF8=1；本会话只参与r3合同修订，未参与其实现。Spec/source/delta完整指纹见新报告。
- acceptance_results: 63/63、真实47决定严格门、双输出确定性绿；20链接/SD-06/07三段匹配，AC-04/05/06/07/11/12/13/16/19通过对应合同/清单范围；AC-01/02/03/09/15partial，AC-08/10/14/17/18/20fail。
- findings: P743-R3-QA-01..04：无效引用记录consumer崩溃、hash/subject代证据、导入/参数同名native、final20/21/P743-3旧指针。
- evidence: [新报告](../reports/743-r3-quality-review-20261005/REVIEW.md)，counterexamples/audit/replay-results/tests/frozen-phase3-spec-delta；旧反例有效修复明确保留。
- next: 用户授权Phase4，执行T-22..27再review；本轮不改实现/canonical/liveledger。

### Phase 4 合同交接（2026-10-05，new r4）

- stage: new | plan_id: PLAN-743 | plan_revision: 4 | outcome: **pass（合同就绪，非实现pass）**
- changed_contract: §0/4.5/5.8/6/7/8，AC-21..24、T-22..27、SD-08；历史T-21按merge/clean据实补完成，current21/total27。
- authorization: 当前继续复审请求+此前有问题激活同计划追加Phase的明确授权；只本仓工具/合同，无额外连续实施/预算授权。
- status: executing/r4；全部r1/r2/r3收据保留；frontmatter当前Spec增量只auto-acc/project.md，auto-hir历史映射仍见旧SD表。
- next: /auto-plan:work 从T-22；无阻止执行的产品裁定，不自动开始修复。



### 合并沉淀收据（2026-10-05，stage: merge，key: PLAN-743:r3）

- stage: merge · plan_revision: 3 · outcome: pass
- reviewed 基线：R4 pass @ 184749168（r3，delta 冻结 a4af013b759a1d35）
- 落点分支：v0.6-dev（主线约定）

**checkpoint 实证**：

| checkpoint | 证据 |
|---|---|
| prepared | worktree 内按冻结提案逐字落 SD-06（auto-acc/project.md 严格门信任段+分类政策条目替换）与 SD-07（stage-contract.md 阶段摘要 R3 两分规则），diff +19/−9，无超出冻结范围内容；plans.md 两行归档态 + 账本 P743-1/2 原位刷新 + reviews P743-5；commit b28cc8ba5（reviewed_commit 的纯文档后代） |
| landed | rebase v0.6-dev（6/6 range-diff 全等号）。旧→新映射：e0008648b→173bf3458、45fe4fb21→c0c6d71e3、a25a30722→b093471db、9f634cb52→b537e6273、184749168→a5b68196b、b28cc8ba5→deee10031。`git merge --ff-only`，零 merge commit；delivery=deee10031 |
| landed（落地后条款） | SD-06/07 目标（auto-acc/project.md、stage-contract.md）无决定绑定→重绑条款条件性不触发；主检出严格冒烟：63/63 测试 + --check --require-decisions 全绿（47 决定/146 组/7 族）；canonical 三项核验（旧 R3 摘要消失、两分规则在案、SD-06 段落在案） |
| ledger_refreshed | .autoos/specs.json P743-1/2 原位更新（r3 强化注记）+ reviews P743-5（外部 needs_fix→Phase 3→R4 pass 链），随 delivery 落主检出，read-back 验证 P743-1..5 在案 |
| archived | 本文件 git mv 至 docs/plans/archive/，status: archived，completion_kind: delivered |
| cleaned | worktree 0 dirty 且 HEAD deee10031 已是 v0.6-dev 祖先；wt-guard.sh 仍缺失（r1/r2 收据已挂工具债）——等价 PowerShell ReparsePoint 全组扫描：`GUARD-EQUIVALENT-CLEAN: no reparse points in lang-743 group`；git worktree remove 目录删除首次遇 Windows 瞬时句柄 Permission denied（注册已注销），重试 rm 后完成；`git branch -d plan-743-dev`（was deee10031）+ 组目录移除，登记/磁盘/分支三方复核 0 |

- 规范增量落地：SD-06→docs/specs/auto-acc/project.md、SD-07→docs/specs/auto-hir/stage-contract.md
  （均 modify；canonical 落地文本与冻结提案逐字一致，R4 复核 a4af013b 未触）。
- 主检出终态：v0.6-dev tip=deee10031（归档收据提交前），工作树 0 dirty，严格门绿。

### 独立复审 R4（2026-10-05，stage: review，r3 P743-R2-QA-01..06 修复复核）

- stage: review
- plan_id: PLAN-743 · plan_revision: 3 · outcome: **pass**（R2-QA-01..06 修复全部确认；
  AC-01..14 在 r3 语义下重验通过，AC-15..20 全部通过）
- reviewed_commit: 1847491685c907d1ce97ef37a316a969089c22cb（worktree plan-743-dev，
  clean 0 dirty）；base_commit: a0f8ddd443418fb32a67d308cbd4a0fd00f74835（激活提交）
- dependency_revisions: Python 3.14.2 标准库；无外仓；PYTHONUTF8=1 消除编码歧义
- spec_inputs: 9 受管输入 hash 经严格 --check 全绿；r3 对 docs/specs/ 零改动
  （git diff 基线探针 0 行，SD-06/07 为提案态）
- 独立性声明：复审在实现会话内进行（同 R1..R3 先例）；结论由命令重放、单元级反例
  复现与源码核查重建。

**R2-QA 修复逐项复核（独立重放）**：

- R2-QA-01 → fixed：shadowed_host（print=local-fn、IO.read_line=type-qualified——
  本地遮蔽宿主）、different_owner（Q 体 .next=unknown）、unrelated_dot_pipe（自由
  函数点/管道=unknown）三 fixture 独立重放正确；真实仓 CG.new/Ar.new 仍 type-qualified；
  源码核查：qualified/点/管道归属推迟至 _post_classify（最终本地集+宿主优先级+
  owner 证据），无全模块方法名猜测。
- R2-QA-02 → fixed：decision-evidence-unbound 定位 ID/path（测试三段式：拒绝→补绑定
  通过→内容变化 stale）；真实层 47 决定证据全绑定，严格门绿。
- R2-QA-03 → fixed：resolved 族须存在+适用引用（kind=unknown-resolution/conclusion=
  resolved/证据覆盖族路径；family-ref-missing/-inapplicable）；open 三要素保留；
  真实 7 族全部通过适用性规则。
- R2-QA-04 → fixed：9 个参数化类型负例受控 ERROR[decision-malformed] 定位、无
  Traceback（独立进程重放验证 exit 1 + stderr 无 traceback）。
- R2-QA-05 → fixed（提案态）：SD-07 冻结提案（a4af013b759a1d35）同步阶段摘要 R3；
  canonical 旧行在案属待沉淀状态（r3 按合同未直接改 canonical，merge 落地）；
  acceptance-matrix/inventory/run-context CLI 全量 auto-ac-prototype，残留 0。
- R2-QA-06 → fixed：计划 4 处链接改归档可解析（独立程序化验证 ALL OK）；plans.md
  行 r3 executing（激活提交）；T-14 补勾有 r2 收据依据；P743-3 指针可解析、
  P743-4 留 merge 对账（合同既定）。

**门禁重放（reviewed_commit 上）**：63/63 测试；严格 --check --require-decisions
exit0（47 决定/146 组/7 族闭环）；三方 cmp BYTE-IDENTICAL；`git diff --check` clean；
范围探针 crates/auto-lib/docs.specs/experimental/Cargo 零触碰。

**AC 终态**：AC-01..14（前轮集合，按 r3 语义重验）pass；AC-15/16/17/18/19/20 pass。
**delta 复核**：SD-06/07 提案与冻结哈希一致；正文为持久规则非执行日记；
supersedes/new_spec_components 元数据经 r3 合同 §Phase 3 规范增量表确认
（均为 modify，不替换组件、不降低原约束）。

**收据冻结哈希（reviewed@184749168）**：phase3 提案 a4af013b759a1d35、
manual-decisions.json 3674c687647f88dc、acc_inventory.py 799c2bb643c5fd3c、
test_acc_inventory.py ae5f8ae26c66bca7。

- findings: 无新增
- evidence: 本记录命令/结果摘录；verification.md/baseline.md（随交付落地）；
  制品经 merge 落 v0.6-dev 后同 hash 可溯
- next: merge（沉淀 SD-06/07；落地后按 §5.6.6 语义复核重绑+严格冒烟；plans.md 行
  归档态、ledger P743-3/4 指针对账；逐条销账、归档、guard 后清理）

### Phase 3 执行交接（2026-10-05，stage: work，r3 P743-R2-QA-01..06 修复）
### Phase 3 执行交接（2026-10-05，stage: work，r3 P743-R2-QA-01..06 修复）

- stage: work（r3；外部复核 needs_fix 后的完整性修复实施）
- plan_id: PLAN-743 · plan_revision: 3
- outcome: pass（T-15..T-20 完成、验收映射成立；非独立复审结论）
- code_commit: 184749168（worktree plan-743-dev；基点=激活提交 a0f8ddd44；
  r3 提交链 e0008648b→45fe4fb21→a25a30722→9f634cb52→184749168，worktree clean 0 dirty）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留待 review/merge）
- task_ids: T-15..T-20 全部完成（证据见 §8 任务行与 verification.md）；
  current_step 20/21；T-21（独立复审与合入交接）按定义留待 review/merge 阶段闭环
- evidence: PYTHONUTF8=1 全门禁绿——63/63 测试（41 旧+22 新）、--write 不触人工层
  （字节不变）、严格 --check --require-decisions（47 决定新鲜、146 组/7 族含新适用性
  规则闭环）、三方确定性 cmp、git diff --check clean、围栏感知链接无断链、
  范围探针 crates/auto-lib/docs.specs/experimental/Cargo 零触碰、CLI 残留 0；
  R2-QA-01..06 反例 before→after 逐项复现并测试锁定（baseline.md/verification.md）；
  phase3 提案冻结 a4af013b759a1d35（SD-06/07）；计划链接改归档可解析（4 处）
- blockers: 无
- next: /auto-plan:review（r3 独立复审：重放原/新反例与严格门，绑定 revision 3 +
  实现 HEAD 184749168，逐 AC/SD 复核）；pass 后 merge 沉淀 SD-06/07（Spec 落地改变
  hash 时按 §5.6.6/§5.7.6 语义复核重绑并严格冒烟）、plans.md 行归档态、ledger
  P743-3/4 指针对账、逐条销账、归档、guard 后清理。本记录不替代复审；不自行归档/合入。

### 合入后独立复审 R4（2026-10-05，review r2）

- stage: review | plan_id: PLAN-743 | plan_revision: 2 | outcome: **needs_fix**
- reviewed_commit: 5983f8aeedeae2dc5769f9a0820eec4b722d4c6d；base_commit: c6e4e568994941883b8c1b6b19dff703feb081cd。
- dependency_revisions: Python 3.14.2 标准库，无外仓；本会话只修订过合同，未参与 r2 实现。Spec/source hashes 与冻结 delta 见新报告。
- acceptance_results: 41/41、严格真实门 47 新鲜/9 输入/146 unknown、确定性绿；AC-06/07/12 合同域 pass，AC-01/02/03/04/05/09/11/13 partial，AC-08/10/14 fail。
- findings: P743-R2-QA-01..06（owner/同名分类、evidence-binding、resolved 引用、字段类型、canonical R3 摘要、归档生命周期）。
- evidence: [新报告](../reports/743-r2-quality-review-20261005/REVIEW.md)，counterexamples.json/audit.json/replay-results.json/tests.txt 与 frozen-phase2-spec-delta.md。
- mainline_movement: ed2d00b90 仅 NOTES-001 sqlite 依赖，743 输入/实现/Spec 没有改变；未将其归属本计划。
- next: 用户授权同 ID Phase 3，执行 T-15..21 再独立复审；本轮不修 Python/不写 canonical 行为/不改 live ledger。

### Phase 3 合同交接（2026-10-05，new r3）

- stage: new | plan_id: PLAN-743 | plan_revision: 3 | outcome: **pass（合同可执行，非实现 pass）**
- changed_contract: §0/4.4/5.7/6/7/8，T-15..21、AC-15..20、SD-06/07；T-14 按 r2 真收据补勾。
- status: executing/r3/current_step14/total_steps21；旧 r1/r2 交付/失败/通过/清理记录完整保留。
- authorization: 当前复审请求 + 此前“发现问题激活同计划追加 Phase”明确指令；沿用原本仓工具/文档边界，无预算/连续实施新授权。
- next: /auto-plan:work 从 T-15；无必须等待用户裁定的产品问题，不启动实施分支或擅自修复。



### 合并沉淀收据（2026-10-05，stage: merge，key: PLAN-743:r2）

- stage: merge · plan_revision: 2 · outcome: pass
- reviewed 基线：R3 pass @ a63660137（r2，delta 冻结 a963adfbd6b24887）
- 落点分支：v0.6-dev（主线约定）

**checkpoint 实证**：

| checkpoint | 证据 |
|---|---|
| prepared | worktree 内按冻结提案逐字落 SD-04（auto-acc/project.md 能力现状节替换+锚点/候选 owner 节新增）与 SD-05（stage-contract.md Pass 模板节替换+三小节），diff +44/−5，无超出冻结范围内容；commit 3b23746a6（reviewed_commit 的纯文档后代） |
| landed | rebase v0.6-dev（6/6 range-diff 全等号；R3 复审时 --write 的审计字段刷新还原为已提交态后 rebase）。旧→新映射：2ecccea12→6859e984f、212183813→89c5850c9、b62510bca→5b1719591、ae63a88b8→fde60c2c9、a63660137→8b44ae855、3b23746a6→4ca002099。`git merge --ff-only` 两次（4ca002099、4bec7575c），零 merge commit |
| landed（§5.6.6 重绑条款） | SD-04/05 目标（auto-acc/project.md、stage-contract.md）无任何决定绑定→重绑条件性不触发；主检出严格冒烟：41/41 测试 + --check --require-decisions 全绿（47 决定新鲜、146 组/7 族闭环），交付后再次验证一致 |
| ledger_refreshed | .autoos/specs.json 按投影规则原位更新 P743-1/2（同 canonical 目标复用既有条目）+ 新增 reviews P743-4（外部 needs_fix→Phase 2→R3 pass 链），commit 4bec7575c 后 ff 落主检出，read-back 验证 P743-1..4 在案 |
| archived | 本文件 git mv 至 docs/plans/archive/，status: archived，completion_kind: delivered |
| cleaned | worktree 0 dirty 且 HEAD 4bec7575c 已是 v0.6-dev 祖先；wt-guard.sh 仍缺失（r1 收据已挂工具债）——等价 PowerShell ReparsePoint 全组扫描：`GUARD-EQUIVALENT-CLEAN: no reparse points in lang-743 group`；随后 worktree remove + `git branch -d plan-743-dev`（was 4bec7575c）+ 组目录 rmdir，登记复核 0。补记：初次 rmdir 因组内 T-13 确定性检查 scratch（scratch-p2/a、b）静默残留失败，清除后组目录移除完成 |

- 规范增量落地：SD-04→docs/specs/auto-acc/project.md、SD-05→docs/specs/auto-hir/stage-contract.md
  （均 modify；canonical 落地文本与冻结提案逐字一致）。
- 主检出终态：v0.6-dev tip=4bec7575c（归档收据提交前），工作树 0 dirty，严格门绿。

### 独立复审 R3（2026-10-05，stage: review，r2 P743-QA-01..07 修复复核）

- stage: review
- plan_id: PLAN-743 · plan_revision: 2 · outcome: **pass**（QA-01..07 修复全部确认；
  AC-01..08 在 r2 语义下重验通过，AC-09..14 全部通过）
- reviewed_commit: a63660137198d4cd5f5af1a4277f2e7901311ee3（worktree plan-743-dev，
  clean 0 dirty）；base_commit: c6e4e568994941883b8c1b6b19dff703feb081cd（激活提交）
- dependency_revisions: Python 3.14.2 标准库；无外仓；741 P3 独立线不在本计划范围
- spec_inputs: 9 受管输入 + 7 证据输入 hash 经严格 --check 全绿（manifest@HEAD 重扫一致）
- 独立性声明：复审在实现会话内进行（同 R1/R2 先例）；结论全部由命令重放、单元级
  反例复现与制品核对重建，不采信执行摘要。

**QA 修复逐项复核（独立重放）**：

- QA-01 → fixed：multiline-string.at 单元重放 `pretend` 零泄漏；跨行能力=起始行 [2]；
  合法跨行（engine.at:314/333）不再记异常、非词法不确定；EOF 未闭合才标记。
- QA-02 → fixed：same-name-method.at 重放 Meter.new=type-qualified、x.len=unknown、
  Meter()=type-construction；真实仓 codegen/a2r 的 `Ty.new` 全部 type-qualified；
  源码核查 NATIVE_METHODS 已移除、分类只经 NATIVE_NAMESPACES/本地符号表。
- QA-03 → fixed：validate_decisions 全 schema 校验 + --require-decisions 严格门
  （输入覆盖 + unknown_families 双向闭环 + open 族 owner/probe/work_package）；
  13 新负例逐一定位决定 ID；scan-only 文案明示非完成态。
- QA-04 → fixed：R3 拆"效果分析（保守、证明才精化）/效果语义保持（双向禁止）"；
  eval.const-fold v2 运行期语境溢出一律保持运行期 trap、comptime 域出外、无编译期
  诊断；X9 反例入册（mark_a 反例原文在案）。
- QA-05 → fixed：A2 拆 HIR existing/源码 gated；A8 owner=运行时首批候选、
  A9 owner=聚合能力线（未编号）；候选①正向集剔 A8、②剔 A9；依赖 ①→② 单向无环。
- QA-06 → fixed（实质审定，非盲换）：8 决定/9 绑定各附对 741 r2/r3 具体差异的
  维持依据（CLI 更名 auto-ac-prototype、原子发布/截止时间、verify 归属唯一、
  bindings 双射）；MD-407/506 记录 QA-01 政策取代；新增 MD-507；7 族闭环 146 组。
- QA-07 → fixed：r2 合同激活时校正；本文件 13 [x]/1 [ ]（T-14 留待复审后）与
  frontmatter execution_done 13/14 一致，历史记录全保。

**门禁重放（reviewed_commit 上）**：41/41 测试；--write exit0；严格 --check
--require-decisions exit0（47 决定新鲜、146 组/7 族闭环）；三方 cmp BYTE-IDENTICAL；
`git diff --check` clean；范围探针 crates/auto-lib/docs.specs/experimental/Cargo
零触碰；围栏感知链接扫描无断链。

**AC 终态**：AC-01..08（r1 集合，按 r2 语义重验）pass；AC-09/10/11/12/13/14 pass。
**delta 复核**：SD-04/05 提案与冻结哈希一致（a963adfbd6b24887，重放验证），正文为
持久规则非执行日记；supersedes=[auto-acc/project.md, auto-hir/stage-contract.md]
（r2 所改组件）、new_spec_components=[]、touched_goals=[]（r2 合同 §4.3 说明在案）。

**收据冻结哈希（reviewed@a63660137）**：phase2 提案 a963adfbd6b24887、
manual-decisions.json 3674c687647f88dc、source-manifest.json 87f11a8fa032c22d、
acc_inventory.py 8be8c39952887304、test_acc_inventory.py 1f8bde2f77b0272f。

- findings: 无新增；（非阻塞观察）无制品目录上严格检查同时报 artifacts-absent 与
  decisions-required 两项，语义清晰，无需处置
- evidence: 本记录命令/结果摘录；verification.md/baseline.md（随交付落地）；制品经
  merge 落 v0.6-dev 后同 hash 可溯
- next: merge（沉淀 SD-04/05；Spec 落地改变 hash 后按 §5.6.6 语义复核重绑并严格
  冒烟；逐条销账、归档、guard 后清理）

### Phase 2 执行交接（2026-10-05，stage: work，r2 P743-QA-01..07 修复）

- stage: work（r2；外部复核 needs_fix 后的修复实施）
- plan_id: PLAN-743 · plan_revision: 2
- outcome: pass（T-07..T-13 完成、验收映射成立；非独立复审结论）
- code_commit: a63660137（worktree plan-743-dev；基点=激活提交 c6e4e5689；
  r2 提交链 2ecccea12→212183813→b62510bca→ae63a88b8→a63660137，worktree clean 0 dirty）
- worktree: D:/autostack/.wt/lang-743/auto-lang（保留待 review/merge）
- task_ids: T-07..T-13 全部 [x]（证据见各任务行与 verification.md）；
  current_step 13/14；T-14（独立复审与合入交接）按定义留待 review/merge 阶段闭环
- evidence: §6 Phase 2 四命令全绿（41/41 测试、--write、严格 --check --require-decisions
  47 决定新鲜+146 unknown 组/7 族双向闭环、git diff --check clean）；确定性三方 cmp
  BYTE-IDENTICAL；围栏感知链接无断链；范围探针 crates/auto-lib/docs.specs/experimental/
  Cargo 零触碰；复审反例 before→after 逐项复现并锁定（baseline.md/verification.md）；
  phase2 提案冻结 a963adfbd6b24887（SD-04/05）
- blockers: 无
- next: /auto-plan:review（r2 独立复审：重放原/新反例与严格门，绑定 revision 2 +
  实现 HEAD a63660137，逐 AC/SD 复核）；pass 后 merge 沉淀 SD-04/05（Spec 落地改变
  hash 后按 §5.6.6 语义复核重绑并严格冒烟）、逐条销账、归档。
  本记录不替代复审；不自行归档/合入。

### 合入后独立复核（2026-10-05，按用户要求再次检查）

- stage: review | plan_id: PLAN-743 | plan_revision: 1 | outcome: **needs_fix**
- reviewed_commit: fa3abe47677992d41b1aef948f36ad46c645f7f8；base_commit: c1ac219e73ee2ed1ef6ba8dfec49c131bfbf1a75。
- dependency_revisions: Python 3.14.2，无外仓；spec_inputs/冻结 delta 指纹见
  [质量复核报告](../reports/743-quality-review-20261005/REVIEW.md) 与相邻 results.json/audit.json。
- acceptance_results: AC-01/03/04/05/06/07 partial；AC-02/08 fail。
- evidence: 18/18 测试、9 输入/8 模块/18 use、字节确定性、15 links 通过；
  多行伪调用/同名 native/无绑定决定三类新反例；8 决定9绑定过期；祖先交付核实。
- findings: P743-QA-01..07（工具三项、合同/候选两项、输入新鲜度、任务勾选）。
  F-01 链接/F-03 use 计数已修，旧 pass 保留历史；不把正确 stale 拒绝当作工具新回归。
- next: 用户已授权非 pass 时再激活追加 Phase；以下 r2 合同交 work，本轮不实施修复。

### Phase 2 再激活 / 修订合同交接（2026-10-05）

- stage: new | plan_id: PLAN-743 | plan_revision: 2 | outcome: **pass（合同就绪，非实现 pass）**
- user_authorization: 本次明确“有问题就激活并追加新 Phase”；归档终态例外已有用户裁定。
- reactivation: archive/ → docs/plans/743-acc-bootstrap-hir-contract.md；executing，current_step=6/total_steps=14，.next-id=744 不变。
- changed_contract: §4.3/5.6/6/7/8，T-07..14、AC-09..14、SD-04/05；原任务/AC/交付记录不移除。
- bookkeeping: 依据 r1 修复完成/merge 证据校正 T-05/06 的历史勾选，当前引用指向 active 文件；
  canonical 行为内容与工具实现保持不变，债务不预销账。
- checks: 编号唯一、任务/AC/Spec 覆盖、路径/计数/引用与差异范围；next: /auto-plan:work。


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
| archived | 本文件 git mv 至 docs/plans/archive/（commit bab649682），status: archived |
| cleaned | worktree 0 dirty 且 HEAD 274a9ded4 已是 v0.6-dev 祖先；**wt-guard.sh 缺失**（AGENTS.md 所述 D:/autostack/wt-guard.sh 与全盘 maxdepth-3 搜索均无此脚本）——以等价 PowerShell ReparsePoint 全组扫描替代：`GUARD-EQUIVALENT-CLEAN: no reparse points in lang-743 group`；随后 `git worktree remove` + `git branch -d plan-743-dev`（was 274a9ded4）+ 组目录 rmdir，`git worktree list` 复核 0 登记。⚠️ 工具债：wt-guard.sh 需重建并回归 AGENTS.md 路径 |

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

### 合入后复审：r4 / c865adf66 / needs_fix（2026-10-05）

完整证据与24条AC对账见[独立报告](../reports/743-r4-quality-review-20261005/REVIEW.md)，本轮冻结SD08核对与SD09提案在同目录。
QA01/P2 参数漏判；QA02/P2 manifest形状崩溃；QA03/P2 无效记录仍消费（最终CLI正确拒绝，无假绿）；QA04/P3 六历史归档链接失效。83/83、47fresh、27/27和ledger真的通过；旧解决问题不再重复报错。
T23..27取消当前验收勾选但保留r4历史实现证据；r5 executing，22/33。本轮不修产品。Module导航只改active，canonical/ledger留merge；P743-3..6历史archive暂缺显式待收口，不宣称全局索引已刷新。


### 2026-10-06文档落地交接

stage:new | plan_id:PLAN-743 | plan_revision:5 | outcome:pass（仅修复合同就绪，非实施复审通过） | next:work
用户本轮要求补齐未落地的743激活；写入前主检出HEAD 0501b0f274088c05bb1b7c5a5fd9a98f4791c835。复审证据继续绑定原r4/c865adf66，工具/测试/目标Spec/原计划/制品指纹未变；不重写旧pass。r5 executing/22-of-33，T23..27当前验收重开，T28..33尚未实施。未改741 r5合同、Python、canonical、live ledger或.next-id；全部QA01..04/修复方案/AC25..28/SD09已入本计划。P743-3..6旧archive派生指针暂缺留r5 merge恢复，不宣称已刷新。

## 10. 待澄清事项

1. ACC最终主体/目标语言coverage：执行者在T-03..05提出有证据的具体边界；
   改变已约定v0.6自举目标的决定须交用户，不以调研结果自动缩减验收。
2. int/i32/Unicode/容器别名与释放等未定细则：记录最小探针和负责工作包；
   本计划允许研究待定，不能把unknown当implemented，后续相关实施前必须解决。
3. 进程内/进程外桥：本计划完成候选矩阵，实际探针与最终编码另立实施合同。
4. 主机器恢复后新输入：保留本次manifest，先报告差异/结论失效；
   重跑盘点属本计划内校准，扩大仓库/目标/验收则修订合同。
5. 无必须先由用户裁定的问题阻止起草完成；未获本计划实施指令，不自动开工。

### Phase 2 交接与范围

无需新增用户决策阻止合同交接。T-08/09 决定有证据的保守词法与完成态校验机制，
T-10 仅纠正语义保持合同，不冻结新语言语义；T-11 的未编号前置由工作线 owner 认领。
741 正在 Phase 3，native 缺陷修复不属 743；只记录依赖状态。不能为消除 stale 暂时缩减绑定。
本轮授权是复审/再激活/修订，未自动执行上述修复。
