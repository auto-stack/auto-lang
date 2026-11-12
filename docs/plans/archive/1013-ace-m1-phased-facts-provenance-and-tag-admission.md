---
plan_id: PLAN-1013
status: archived
feature_name: ACE M1分域facts来源链与签章准入有界修复
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12

plan_revision: 1
depends_on: [PLAN-1005]
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md, docs/specs/auto-test-case/ace-m1-expanded-acceptance.md, docs/specs/auto-test-case/architecture.md]
new_spec_components: []
touched_goals: [GOAL-002]

affects: [auto-test-case]
current_step: 10
total_steps: 10
---

# [PLAN-1013] ACE M1分域facts来源链与签章准入有界修复

## 0. 变更摘要

承接1005交付后、实际打`v0.6-ACE-M1`前发现的 **F-1013-01：分域entry facts绕过原始执行准入，并把虚构commit传入审计链**。仅修架构facts及其原始来源、冻结提交身份、合并/审计消费者和签前复核，不增加M1特性。

1005保持archived；其严格raw reader、独立凭据机制和交付成果保留，但其历史1006P及tag_ready不能豁免本次反例。本包须以修复后来源链重建当前可签结论，不能只记债务或重算摘要使总数恢复。

本轮仅新建计划与中央取号簿记：drafting r1、0/10。未施工、建实施树、运行产品测试、打tag、推送、重开1005或创建/联系其他会话。下列用例与命令均为待执行合同。

## 1. 目标

1. 原始执行文件缺失，或只有自报`green/executed/kept/commit`与局部SHA的facts，不得供应架构行PASS。
2. facts只是严格验证后原始运行结果的紧凑投影；篡改投影并重算全部局部hash，不能改变原件决定的判决。
3. commit必须是真实仓库中的commit对象，并与受审冻结基线、真实生产窗口和必要源/配置/二进制闭包一致；长得像SHA或存在于Git均不足。未验证提交不得进入`gate_commits`或解锁review行。
4. raw、facts、混合分域、机制合并及正式入口遵同一不变量；合法分域继续在原48MiB活数据、4MiB单文档预算下可用。
5. 独立复审核本包反控、全路径与规范增量，生成绑定本包冻结身份的新R；全M1终验和主线落地复核有效才给tag_ready。

非目标：语言/FFI/标准库/运行时GC扩面、重做1000/1005全部实现、通用PKI/签名平台、其他非架构认证族重构。保持原1006键、旧313 required、49架构行（48非review+1review）、31合同身份关系、批准NA/accepted/future、source/expected与原样本/性能门；不缩分母、改expected或放宽阈值。

## 2. 架构方案

唯一授信路径：受审冻结身份与生产窗口 → 原固定组真实运行原件 → 严格raw准入 → 按固定recipe派生facts → 消费时核原件/来源链并重新派生或比对 → 已验证行与commit集合 → 非review候选 → 独立凭据R → 正式八域AND → 主线C的tag_ready。

默认采用**逐entry读原件、复用现raw reader重导所需facts、比对摘要后释放瞬态**；摘要自带“期望hash”不构成可信锚。也可把旧facts引用迁移为含原始root的受检引用，保持合法分域；旧schema不能证明来源时明确拒绝/迁移，不得自动fallback到旧自报分支。T-02先做whole-path小探针，冻结预算可行的接口。等价局部机制由work选择，但不能退化为只验hash、删合法分域或提高预算。

只读组合器仍不启动执行工具；Git对象/必要闭包的只读核查及与reader的交接由T-02定稿。可信期望基线来自真实冻结/独立复审流程及经核本仓身份，不能由未受信facts自带并自行比对。hash只保证字节身份；实际执行和人的独立性还需原收据与复审流程。本包不宣称抵御控制全部原始运行记录、仓库和审计流程的攻击者。

## 3. 技术栈

现有Python严格reader、EvidenceScope/LiveBudget、unittest、固定`cargo t`职责组、fixed-tests/nextest/TestCase原件与Git冻结身份。默认不改Rust产品代码；重采若发现其他产品缺陷，具名阻断并分诊，不静默扩包或在required未闭时宣称本包完成。

未来唯一实施树`D:/autostack/.wt/lang-1013/auto-lang`，branch=`codex/plan-1013-dev`。work/内查/独立review顺序共用该树，review时实现停写，不另建review树/链接。本轮未创建。材料唯一scope=`D:/autostack/plan-artifacts/plan-1013/`，开工登记owner/用途/在途状态；纯规划临时核对放系统TEMP。

## 4. 需求分析与背景调查

用户2026-11-12授权：“正式新建一个有界后继计划，修复1005遗留的分域facts准入绕过，确保原始执行文件缺失、虚构提交或自行重算摘要hash均不能PASS，明确反控、独立复审和打tag前的验收条件。本轮只规划，不施工，不重开1005，不启动其他会话。”本轮授权仅本仓规划，无指定模型/工期/新增预算，不推定当前实施或打tag授权。

从`docs/specs/overview.md`导航，读取现行裁判架构、扩大验收、架构证据准入Spec及归档1005，再核实际生产代码与测试。主线H0=`e6fb5855bab3fa103e41502833d37c6b440c00be`、branch=master；1005落地`05b7c9d7e`及沉淀`107e295d3`已在上轮tag核验确认为祖先，T-01重新核依赖。1005材料/实施树已清理，不能把已删raw路径冒作新证据。

### 4.1 已实证反例与原因

上轮直接调用生产`ace_m1_arch_entry_gates.admit_arch_entry_gates`，使用真实contract/recipe，内存仅提供相关entry的如下形状JSON及其正确binding SHA，无实际语言执行或原始运行文件：

```json
{"schema":"ace.m1.arch-entry-facts/1","entry":"<真实entry>","group":"<合同要求组>","kind":"native","commit":"ffffffffffffffffffffffffffffffffffffffff","green":true,"executed":1,"kept":{"<recipe要求的测试名>":"ok"},"gate_originals_sha256":{"missing-run.stdout":"0000000000000000000000000000000000000000000000000000000000000000"}}
```

完整复现按实际recipe补齐全部kept，`executed=len(kept)`；case型对应`case|check:PASS`。上例单键是形状示意，不是完整可运行输入。每份JSON序列化计算真实SHA，绑定为`entries[name].facts={path,sha256}`。

T-01的确定性构造步骤：从实际recipes取目标row；对每个`recipe.facts`的entry，按contract的kind决定native/case，group取合同group（provider-pending取`DELIVERED_PROVIDERS[name][1]`）；native将全部`native_tests`映射为`ok`，case将全部`case_checks`二元组用`|`拼成键并映射为`PASS`。每份文档统一设40个f的commit、green=true、executed=len(kept)和上例缺失原件表，真实序列化算SHA。输入侧用真实EvidenceScope读contract/recipe；证据侧只返回这些JSON并记录读取路径，调用生产reader而不mock其判决。永久回归另补真实磁盘scope和正式CLI，避免内存替身掩盖文件系统边界。

| row | 实际返回 | evidence读取 | 外流身份 |
|---|---|---|---|
| `ARCH-07/one-pid-one-entry` | PASS | 仅`av-resident.json`和`resident.json`合成facts | `gate_commits=["ffff…ffff"]` |
| `ARCH-07/last-owner-gc` | PASS | 同上，`missing-run.stdout`不存在 | 同上 |

已确立原因：`if 'facts' in ref`只要求`gate_originals_sha256`为非空dict，未读取原件或核commit真实性/适用性，直接用kept/green构造gates，再按recipe授行PASS、输出commit。**已证明两行假绿，不声称已伪造完整M1报告或绕过全部审计凭据校验。** 本轮只读确认代码仍在，未重跑反例；T-01固化完整复现。

raw分支已有严格原件解析，但commit先只验40位格式，窗口source-pins允许合同闭包子集。T-02必须核最小必需闭包是否完整及raw/facts是否共用可信冻结锚，不能假定raw已符合本包提交门。这是与已见身份外流同源的必要检查，不扩为全仓可信构建系统。

### 4.2 本轮磁盘指纹

| 路径 | SHA256 |
|---|---|
| `scripts/test-governance/ace_m1_arch_entry_gates.py` | `707d84ccba2c32384e1966fb215e59a1d069823be6100b8fcc0864c264e6aeab` |
| `scripts/test-governance/ace_m1_review_evidence.py` | `6c3620009c0ae06739ab1fb1d6bd4811b32696d76758ea7cc7da5d35b89b4a57` |
| `scripts/test-governance/ace_m1_first_slice.py` | `10fb92100c5974aa3652262dde27311d6ad52b2031ad49365267f4e0e681eb10` |
| `scripts/tests/test_ace_m1_arch_acceptance.py` | `c26ea85df126e6bf38cb1f28f968380f8314546d3b942420e845f80bf5e9d526` |
| `crates/auto-lang/test/cases/ace-m1-expanded/architecture-entry-recipes.json` | `0f224ad5e14b1f3f34b946206d93aba6c3c6392256a4fb0025f76dd335af9fc1` |
| `crates/auto-lang/test/cases/ace-m1-expanded/registry.json` | `7d44d4024305349a02f2e7bf1771079dbfdc32683e4a37d936ef73d8b9e7e67f` |
| `docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md` | `aebb2260fb6cf1937b2a7137676136c2fd909895f1e3e9cc394eb7a9b77415cc` |
| `docs/specs/auto-test-case/ace-m1-expanded-acceptance.md` | `7de0286c12352d0c59c57c2a5168fa7c0a660c4755e216cbe0594c00f37c46d8` |

规划前.next-id=1013，active/archive/旧archived最大1012，无匹配在途修复。用`Local\AutoLangPlanAllocation`互斥重核后调用本仓`new-plan.sh`分配1013、推进1014；不截号或套用技能旧三位限制。现有他人文档/WIP保持不动，最终核对写§9。

### 4.3 具体源码矩阵

短名`ace_m1_*.py`均在`scripts/test-governance/`；以下符号/文件已核存在。新fixture/schema由对应task实现，不当成已有API。

| owner | 实际源码/符号 | producer→consumer与拟改面 |
|---|---|---|
| T-01/02 | `ace_m1_arch_entry_gates.py::admit_arch_entry_gates/validate_recipes`；`architecture-entry-recipes.json`、`architecture-obligations.json` | 30entry/48非review recipe→逐行结果；冻结native/case/provider-pending的真实来源和最小原件闭包 |
| T-02/03 | 同文件`_admit_gate_root/_window_pins/admit_native_gate/admit_case_gate` | fixed-tests原件→父parser、选择、事件→已验证投影；复用raw准入，移除自报facts授信 |
| T-02/04 | `scripts/fixed-tests.py`及上述gate reader | 生产窗口/Git对象/冻结锚→源、配置、binary、recipe pin；先复用原设施，只做必要最小接线，不建第二runner |
| T-04/05 | `ace_m1_first_slice.py::compose_first_slice/merge_architecture_observations/admit_phased_observations` | entry/机制/分域→architecture_execution及arch_entry_gate_commits；阻止未验证身份传播/后写洗白 |
| T-04/05 | `ace_m1_review_evidence.py::admit_review_credential` | frozen_commits→R→唯一review行；当前PLAN/REVISION及coverage硬编码1005，必须明确迁移/绑定本包合同，旧R不冒本包已审 |
| T-05/07/08 | `ace_m1_arch_acceptance.py::reduce_acceptance`及CLI；`ace_m1_registry.py::compose_registered_inventory` | 全集合/八域→正式报告；核正式入口不能继续走旧facts旁路 |
| T-01/06/08 | `scripts/tests/test_ace_m1_arch_acceptance.py::ArchitectureEntryGateStrictControls/ReviewCredentialControls/ArchitectureMergeBoundaryControls`；`scripts/test-governance/python_tests.py` | 永久反控→原unittest loader→固定组；section白名单测试不能代实际entry facts分支 |
| T-07..10 | 三份frontmatter所列Spec、原M1生产者和本Plan | 当前证据→独立review→主线tag_ready→规范沉淀/归档清理；不改archived1005 |

## 5. 详细设计

### 5.1 阶段合同

| 阶段 | 责任、真实数据流 | 可观察关闭条件 |
|---|---|---|
| S0，T-01 | 固定基线/依赖、原两行输入与合法原件对照，标明控制/生产证据 | 修前两PASS及只读facts的trace可复现；版本不同先解释，不以复现失败冒修复 |
| S1，T-02 | native/case各一条真实entry，加provider形状；原root→raw准入→投影→候选，核冻结锚和预算 | 全路径正例及缺raw/假commit/改投影负例可区分；接口/原件集合/必要pin/错误语义记入本Plan，决定完成才开依赖任务 |
| S2，T-03/04 | reader重导原件、比对投影；冻结身份验证后才输出commit；原件保留至最终核验 | 缺raw/不符/虚构或不适用commit不授信；局部重算hash不改变原件判决；raw与facts等价 |
| S3，T-05/06 | 合并与R consumer只消费已验证事实/身份；永久反控接原loader | 无旧schema/fallback/覆盖/缓存旁路；正负例及完整职责组真实通过 |
| S4，T-07，work | 修复H上重建非review全集与来源，原件留在唯一scope | 1005P/0F/1U，唯一U=review行，正式exit1；AC-01..07齐才execution_done |
| S5，T-08，review | 同树独立上下文核实际代码/原件/AC/V/SD→新R→正式报告 | R被消费；1006P/0F/0U、八域真、blockers=[]，最终review通过才reviewed |
| S6，T-09/10，merge | H→主线C对账、TAG六门、Spec、归档与实际清理 | tag_ready及清理真实闭合才delivered；不自动创建/推送tag |

### 5.2 原件、投影、commit与三态

**R-01，AC-02/04：** 每entry必须沿受检引用找到完整原始材料。native包括gate、discovery/listing及command、run原输出/终态、results、source-pins窗口及binary pin；case包括原inventory/report、运行命令/终态及同等身份。精确集合T-02沿现`NATIVE_ORIGINALS/CASE_ORIGINALS`冻结。hash表非空不等于文件存在；逐项读/验hash，按现父parser重导选择/事件/判决，再按当前recipe投影。摘要若保留green/executed/kept/commit，须与派生值逐项一致，不能拿自报值当推导输入。

**R-02，AC-03：** commit在正确仓中解析为实际commit对象，再核受审冻结锚、before=after窗口、必要源/配置/工具/binary/规则闭包。真实但无关的旧commit、另一仓身份、dirty内容冒clean H、只补一个正确pin均不授信。历史生产H仅在明确受审祖先/等价闭包策略下可复用，记录H→C逐项对账；commit存在或是祖先不能代内容适用。不要求无关docs变化触发全套重跑。必要pin缺项不可用`subset`空洞证明通过。

**R-03，AC-04/05：** 同一原件直接raw/分域facts的行判决、FAIL/U和verified commit集合一致；混合输入一份坏证据不可由绿摘要覆盖。若有缓存，其键包含原件身份、producer/consumer规则、配置/recipe与冻结锚，变化使旧派生物失效；无缓存不要求新建。进入`arch_entry_gate_commits`的值只来自上述核验，R的`frozen_commit in gate_commits`仅是最后一致性检查，不是唯一真实性门。

三态保持：缺有效必需观察→U/正式exit1；真实oracle不符→FAIL/exit1；存在的引用缺原件、坏格式/hash/身份/不适用commit→受控拒绝/exit2。整个entry binding未供给则对应行U。不能以任意异常/崩溃充反控通过。

### 5.3 独立凭据与证据生命周期

保留1005凭据专属review行、缺R不可签及无环候选→R→最终报告。T-04明确支持本包plan/revision、AC-01..10/V-01..15/SD-01..03的受审合同，可固定版本迁移或受控参数化；未知计划/任意coverage不授信。**旧1005 R不能通过仅改提交字段/局部hash冒本包覆盖。** work合成R只能测规则；实际R由T-08独立上下文核冻结H和原件后产生，R不引用最终报告hash，最终报告引用R。consumer/原事实/规则/实质依赖变化使受影响R与报告失效，须重验/重审。

原件留到正式准入、独立review及主线tag_ready复核完成。1005材料已删，只有实际可读且身份成立原件可复用；缺失则按依赖重采，历史Plan摘要/hash不能恢复执行证据。不机械重跑全仓，但原1006未闭就不能以“reader修好”完成。本包不能以记债/转后继豁免required；若遇需扩语言/平台/资源的其他产品缺陷，记录具名阻断并走needs_replan/有界后继。

merge删除材料前，将命令/退出码/数字/指纹/反控/审阅/签前核验摘要折入Plan，已删路径标历史；未来新验收需重新采证。后续实际tag只能基于仍有效的签前复核；有实质漂移或需要重新准入，先重采复核，不假称已删raw仍可读取。

### 5.4 资源与规范增量

同一LiveBudget计入原件解析/投影/保留facts：活数据≤48MiB、单文档≤4MiB。不提高阈值，不用未计量额外进程预洗facts；逐entry读/派生/释放及错误退出都核计量，资源拒绝与格式/身份拒绝诊断区分。T-02先证明whole-path可行；若默认方案不可行，在原预算内确定等价策略后实现。确需改批准预算/支持域须先具名修订，不能偷换成不受检摘要。

仅拟议，review按实际定稿、merge才写canonical，不反写1005历史：

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md` | Spec要求完整链但facts代码仅验摘要→明定原件重导/比对、可信commit、必要pin、同入口判决/预算/新R合同 | 关闭实证规范违背 | AC-02..08 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-expanded-acceptance.md` | 保留1005历史报告与tag_ready→追加残余缺陷及其不可豁免、修复后实际R/终验/TAG结果 | 当前结论由可靠来源重建 | AC-01/08/09/10 |
| SD-03 | modify | `docs/specs/auto-test-case/architecture.md` | 明确facts是原件受检投影、verified commits到review的唯一可信边、原件保留至签前复核 | 防其他入口恢复同源绕过 | AC-03/04/05/08/10 |

## 6. 测试设计

下列均待执行，T-01固化修前输入，T-03..06接永久现有测试。每V记录变异发生证明、真实入口、具体诊断/row判决和执行数。合成控制不作M1生产证据。尤其假commit负控必须提供其余真实完整原件，不能因缺文件失败就宣称提交门敏感。

| Case | AC | 输入/实际入口 | 必须观察的结果/区分性断言 | owner/覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02/04 | §4两行完整合成facts、真实recipe/正确局部SHA→生产entry reader | 修前两PASS/只读facts复现；修后缺原件受控拒绝，无PASS/假commit外流 | T-01/03/06；两行/native分支 |
| V-02 | AC-02 | 合法native/case逐项删run/discovery或inventory/report/command/窗口原件，保facts/hash | 每种必需文件缺失都准确拒绝exit2，不能只核一个raw或非空hash表 | T-03/06；两kind原件集合 |
| V-03 | AC-02/04 | 原件不变，改green/executed/kept/commit并重算facts+binding所有局部SHA；另改raw但保可信pin | 因与派生结果/可信原件身份不符仍拒绝，不是只抓忘重算hash | T-03/06；逐类字段 |
| V-04 | AC-03 | 完整原件/收据/窗口共同填同一个不存在40位commit，局部hash重算；另测坏格式/非commit对象 | 格式正确虚构对象仍被真实性门拒绝，无verified commit；明确不是缺raw造成 | T-04/06；raw/facts |
| V-05 | AC-03 | 真实但不适用commit、错仓身份、dirty冒clean、缺必要pin/改配置或binary | 依冻结闭包拒绝；真祖先但不等价仍拒；无关docs等价后代正例按明确策略可复用 | T-02/04/06；原冻结策略 |
| V-06 | AC-02/04 | 真实native/case原件→raw/facts/混合；缺具名观察、真实红与无关合法ignored对照 | 合法row/verified commits等价；缺required为U、真实红为FAIL，原合法条件排除保留 | T-03/06；provider适用形状 |
| V-07 | AC-04/05 | 坏facts经first_slice/机制合并/库存/正式CLI；冲突、旧schema回退、缓存（若有） | 不覆盖FAIL/U、不注入review；规则/冻结锚改变派生物失效，正式不可签 | T-05/06；全生产链 |
| V-08 | AC-03/05/08 | 假facts提交试图入arch_entry_gate_commits配R；旧1005 R/错本包revision或coverage/缺R | 假身份在上游挡住；旧错R不解锁；缺R唯一review U，成员关系不能自授信 | T-04/05/06；审计消费边 |
| V-09 | AC-06 | 同meter逐entry真实native/case、重复entry及错误退出；原cap边界合法与超帽控制 | 合法≤48MiB/4MiB且释放计量完整；超帽资源拒绝，不冒语言FAIL或绕meter | T-02/03/06；原支持预算 |
| V-10 | AC-07 | 原loader发现新case，1005严格封套/recipe/R/section白名单正反例 | 新case实际执行>0，原控制不退；未执行/skip不充通过，section测试不代facts分支 | T-06；完整职责组 |
| V-11 | AC-07/08 | 临时恢复仅验facts+hash或40位格式的坏机制，执行对应永久case后恢复 | 对应断言真正失败；记变异位置/hash、命中失败case与恢复后绿，不能靠无关错误显示红 | T-06/08；不入生产后门 |
| V-12 | AC-06/09 | 冻结H、原1006键/313集合/31合同、48非review真实证据→候选 | 1005P/0F/1U且唯一U=review；正式exit1，preview恒不可签exit1 | T-07；work交接 |
| V-13 | AC-08/09 | 独立上下文核H/原件/本包AC/V/SD→实际新R→正式consumer/最终review | R身份/覆盖/来源无环可核；1006P/0F/0U、ARCH49/49、八域真、正式exit0 | T-08；真实审计不以合成R代 |
| V-14 | AC-09/10 | H→主线C；改consumer/recipe/raw/工具依赖后沿旧R或只重算报告hash | 实质漂移阻tag_ready；等价docs后代逐项对账可复用，TAG任一未闭false | T-08/09；落地签前 |
| V-15 | AC-10 | 实际TAG六门、归档/输出路径对账/清理，已有tag与远端未知状态 | tag_ready有当前依据；不force、不冒远端发布；删前折摘要，删后无假raw可读声明 | T-09/10；交付/tag边界 |

正式验证入口与责任：

- 主要完整职责组：`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy`，已静态核`test-groups.toml`。改fixed runner/parser/loader则并入`infra-fixed auto-test-case ace-adapters`，按实际触面和必要consumer取完整并集，不机械跑全部Rust。
- 永久用例沿`test_ace_m1_arch_acceptance.py`，T-06记录准确类/方法定向命令及现loader实际发现/执行数。受控变异只在唯一实施树，恢复并核hash后冻结。测试类在脚本`__main__`之后不等于unittest discovery无法加载，核实际loader行为。
- 原正式CLI：`python scripts/test-governance/ace_m1_arch_acceptance.py --acceptance --repo <实施树> --evidence <scope> --bindings <bindings>`；preview沿现`--first-slice-preview`。新原件/冻结锚/R字段待T-02/04定稿，记录准确调用，不冒拟API为现接口。原M1 producer/规模/ignore/平台/性能门不改；T-07先列可复用与须重采清单，正式采样单实例错峰。
- 列发现/执行/原条件排除数、真实子终态、准备/冷编译/运行/清理/总时与来源指纹。父exit0不代各V；原P90≤2s不变，不新加三warm/三轮全仓。周期`cargo t all`仅按主检出due规则。

角色：work负责T-01..07与自查；T-08必须独立上下文同树最终review，从原件逐项核AC/V/SD、遗漏/债务与健康，记录真实角色/上下文。用户未指定review人选/模型，本轮不启动代理/会话；实现自查不能写成独立审计。review不顺手修代码，needs_fix回同树work，实质变更使受影响证据失效。

## 7. 验收标准

| ID | owner_stage | 可观察通过标准 | 核验/任务 |
|---|---|---|---|
| AC-01 | work | 原两行反例/真实来源矩阵/依赖分母冻结，计划与控制不冒执行证据 | V-01；T-01/02 |
| AC-02 | work | 完整可读原件严格重导才授信，缺原件/非空hash表/自报kept不PASS，native/case正例过 | V-01..03/06；T-03/06 |
| AC-03 | work | raw/facts共核实际commit对象、正确仓/冻结锚/必要闭包；虚构/不适用/缺pin身份不外流审计 | V-04/05/08；T-02/04/06 |
| AC-04 | work | 分域是受检投影，重算局部hash仍不能篡改判决；raw/facts/混合等价，无旧schema/缓存旁路 | V-01/03/06/07；T-03/05/06 |
| AC-05 | work | first_slice/合并/库存/正式CLI/R只消费已验证事实/身份；明确本包R合同，旧1005 R不冒新覆盖 | V-07/08；T-04/05/06 |
| AC-06 | work | 原预算内正例/错误计量诚实；1005非review键真实绿、唯一review U，候选不可签 | V-09/12；T-02/03/07 |
| AC-07 | work | 实施期V-01..12所需控制真实执行，永久反控敏感，完整职责组/必要consumer有效且合法行为不退 | V-01..12；T-06/07 |
| AC-08 | review | 独立上下文从原件核AC/V/SD和全部适用入口，未决发现/必需控制跳过不通过，新R经正式链消费 | V-08/11/13/14；T-08 |
| AC-09 | review | 当前冻结终验1006P/0F/0U、ARCH49/49、八域真；accounting/evidence/system真、cert-gap/awaiting_d/missing_inputs=0、blockers=[]；正式exit0、can_claim_expanded_m1=true，preview仍exit1 | V-12..14；T-07候选/T-08终验 |
| AC-10 | merge | H→C对账、TAG六门/规范沉淀成立，归档与实际清理/摘要闭合，交付tag_ready；不自动创建/推送tag | V-14/15；T-09/10 |

### 打tag前强制门（沿1005编号，当前证据重新核）

| Gate | 通过条件 | 阻断条件 |
|---|---|---|
| TAG-01 | 明确完整C/受审H、本包修复与Spec祖先、producer/consumer/规则/工具依赖指纹 | 只认branch tip/WIP或修复未落C |
| TAG-02 | AC-09严格终验有效、1006/313及31合同关系守恒 | required未闭、旧报告代原件、仅组绿 |
| TAG-03 | 本包实际独立R、可信冻结锚/候选/原件/规则/覆盖一致且consumer接受 | 缺旧伪R、未核commit、未决发现/自指/实质漂移 |
| TAG-04 | V-01..15各owner到位，缺raw/假commit/重算hash控制真实敏感且正例过 | 任一路架构假绿或负控靠无关错误失败 |
| TAG-05 | H→C逐项闭包对账，必要冻结组/正式读回有效，签前复核在原件删除前完成 | 只在旧树可用、raw已删冒重读、漂移后仅换SHA |
| TAG-06 | 核本地`v0.6-ACE-M1`是否存在及目标；拟注释含C、R/终验hash、本包/1005收据、ACE M1范围 | 不覆盖/移动现tag；远端未核不报不存在/已发布，不冒M2或全v0.6 |

任一门未闭则tag_ready=false。本Plan只交付tag_ready复核记录；实际创建/推送tag为后续动作，按当时用户授权办理，本轮不做。清理后若需重新准入或有实质变化，先重新采证/复核，不拿已删文件hash冒当前验证。

## 8. 执行步骤

以下为唯一权威task表，均未执行。10项=work七项+review一项+merge两项。work启动先建§3唯一树并核仓/路径/branch/HEAD/依赖、登记scope；计划簿记留主检出。

| Task | owner_stage | 依赖 | 具体操作/输出 | 验证入口与AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 1005交付 | 核H0/依赖/分母，固化两行复现/合法原件、entry recipe及读取trace | 唯一树调用生产reader复现V-01；scope原输入/命令/结果摘要Plan；AC-01 | [x] |
| T-02 | work | T-01 | 按§4.3完成native/case/provider whole-path前探，冻结原件集合/必要pin/可信commit锚、投影迁移/R接口及producer准确命令 | V-05/06/09小探针与预算；决定补§5/6，未定不进实现；AC-01/03/06 | [x] |
| T-03 | work | T-02 | 改entry facts分支复用raw重导/比对，逐项原件核验/计量释放，拒旧摘要fallback | V-01..03/06/09，沿现测试类新方法定向调用；AC-02/04/06 | [x] |
| T-04 | work | T-02/03 | raw/facts共用commit真实性/冻结闭包门、verified commits输出，review reader绑定本包合同/拒旧R | V-04/05/08，假commit其余原件齐全；AC-03/05 | [x] |
| T-05 | work | T-03/04 | 核first_slice/机制/分域/库存/正式CLI与R调用，去掉未经验证row/commit传播 | V-06..08实际正式路径；AC-04/05 | [x] |
| T-06 | work | T-03..05 | 永久正负例接原loader，修前或受控变异证明敏感性，完整职责组/必要consumer，定稿SD提案 | V-01..11；`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy`与触面并集；AC-02..07 | [x] |
| T-07 | work | T-01..06 | 冻结H，证据复用/重采清单，缺raw真实重采，严格组合非review候选并记录来源/预算 | V-12，1005P/0F/1U/exit1；AC-06/07/09候选；七work齐才execution_done | [x] 2026-11-12：九族采集+V-12 达成（1005P/0F/1U、ARCH 48+1review、峰值 47.7、正式 exit1）；T-08 首轮 needs_fix（FR-T08-03 cte 策略 7<10 窗）已修复重组合 |
| T-08 | review | T-07，独立上下文 | 实现停写，同树从原件核AC-01..10/V-01..15/SD-01..03适用阶段，遗漏/债务/健康，实际新R→正式终验→最终裁决 | V-11/13/14、AC-08/09；merge未来项审可交接合同不预勾完成；needs_fix回work | [x] 2026-11-12：两轮独立复审（第一轮 needs_fix FR-T08-03 已修复；第二轮 pass）；R2 消费、终验 exit0 1006P/0F/0U、ARCH 49/49、八域全真、preview 恒 exit1 |
| T-09 | merge | T-08 pass | 核冻结职责组/consumer有效证据，代码落C、SD-01..03沉淀，raw仍在时完成TAG六门/必要正式读回，写tag_ready | V-14/15，AC-10落地部分；身份未变证据复用，变则按影响重采重审 | [x] 2026-11-12：C=990d27fcc、SD-01..03 沉淀、TAG 六门闭、H 读回 exit0、tag_ready=true |
| T-10 | merge | T-09 | 摘要折叠Plan/归档；核所有实际输出路径/链接/活动进程/具名持有，wt-guard clean后清树/scope、刷新retention与计数 | V-15，AC-10实际清理；无具名未决持有则scope整删，未知归属不盲删；真实10/10才delivered | [x] 2026-11-12：摘要折叠、归档、wt-guard clean 后清理 worktree/scope、retention 刷新 |

下一就绪任务T-01；1005已闭，本包修复/验证未开始。T-02是原预算内机制决策，不是追加许可流程。本轮授权止于规划，后续实施待实际指令，不自动续接。

## 9. 复审记录

2026-11-12：stage: new；plan_id: PLAN-1013；plan_revision: 1；outcome: pass（规划可交接，不是实现/独立复审通过）；next: work（本轮不启动）。新增T-01..10、AC-01..10、V-01..15、SD-01..03。已只读核Spec、facts生产分支和审计传播路径，将上轮实证反例列为必须固化的V-01；全部施工/执行/独立复审/落地/清理pending。

规划静态核对已完成：编号唯一，§0..10齐全，10任务/10AC/15V/3SD/6TAG的定义行唯一且连续，10任务全未勾，frontmatter=0/10，八份源码/规则/Spec指纹和六个固定组名均与磁盘一致；counter=1014。没有执行产品或本包反控，不计入任务完成数。

本会话仅写本Plan和counter。前后快照同时发现其他文档在变化：10份既有ROS2/联合收尾文档内容变化，另出现3份策略/未来路线文档；保留这些现场变化，没有回滚、暂存或修改它们。因此不声称整个工作区前后完全不变，也不将其差异归入本包。1005归档文件、产品源码/测试和canonical未由本会话改写。

末次核验HEAD已由并行落地推进至`90e66211b057d2e159f5e32639347e2e56763893`；相对H0仅涉及`.autoos/module-boundaries-baseline.json`、归档989及`docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md`，本Plan八份来源指纹仍逐字匹配。H0保留为调研基线，T-01按实际开工HEAD重核，不假称规划期间主线冻结。

2026-11-12：stage: work；plan_id: PLAN-1013；plan_revision: r1；outcome: pass（T-01..T-06 完成，T-07 候选组合 pending）；worktree: `D:/autostack/.wt/lang-1013/auto-lang`；branch: `codex/plan-1013-dev`；base_commit: `1169f1359`；code_commit: `240808812`（实现）、`cdea5e038`（测试）；task_ids: T-01..T-06；next: work（T-07 冻结H与严格组合非review候选，V-12）。

**T-01（V-01 修前复现）**：`t01/pre-fix-counterexample.py` 用真实 contract/recipe 构造两行完整合成 facts（resident case 3 checks + av-resident native 14 tests，`executed=len(kept)`、`commit='f'*40`、`gate_originals_sha256={'missing-run.stdout':'0'*64}`），绑定真实 SHA，调用生产 `admit_arch_entry_gates`：**修前两行 PASS、假 commit 泄漏进 `gate_commits`**（pre-fix-result.json）。读取 trace 与输入 JSON 固化在 scope。

**T-02（whole-path 探针）**：`t02/whole-path-probe.py` 建真实 git 合成仓库（`git init`+commit），构造合法 case gate（resident 3 checks）与 native gate（av-resident 14 tests），复用现 raw reader 完整准入：case green、native green、峰值活数据 `1.27MiB << 48MiB`。git 只读核查 `rev-parse --verify <commit>^{commit}` 可区分真实/虚构/坏格式。**修复前 raw reader 也接受完整伪造假 commit（重算全部局部 hash 后 gap 实证）**——T-04 同时关闭 raw 侧 40 位格式旁路。

**T-03/T-04（实现，提交 240808812）**：
- `ace_m1_arch_entry_gates.py`：新增 `_verify_commit_object`（正确仓 `git rev-parse --verify` 只读核查 + 冻结锚成员门）；`_admit_gate_root` 对 raw/facts 共用该门；`_entry_closure` 统一 closure 构造；**facts 引用迁移为 `{'facts':{...},'root':<gate dir>}`**——消费时复用 `admit_native_gate`/`admit_case_gate` 从原件完整重导，再逐项比对 facts 投影的 commit/green/executed/kept/gate_originals_sha256，旧 schema（无 root）明确拒绝，无自报 fallback；`admit_arch_entry_gates` 接受可选 `frozen_anchors`（binding 级或参数级）。
- `ace_m1_review_evidence.py`：凭据合同绑定本包 `PLAN-1013`/`revision 1`/`AC-01..10`/`V-01..15`/`SD-01..03`，旧 PLAN-1005 R 或错误 revision/coverage 一律拒。
- `test_ace_m1_arch_acceptance.py`：合成 repo 变真实 git 仓库（`synthetic_commit`）；新增 V-04 假 commit 完整原件拒绝、V-05 真实但锚外/锚闭合正控、phased facts 无 root 拒绝/重导正例/篡改投影重算 hash 拒绝/假 commit drift 拒绝/坏 originals 身份拒绝。

**T-05（消费者核验）**：`compose_first_slice` 的 entry-gates 消费、机制族合并、`compose_registered_inventory` 架构行消费、review 凭据 `frozen_commit in gate_commits` 均为已验证集合/重导判决；无未经验证 row/commit 传播路径。职责组 `cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy` PASS。

**T-06（永久正负例 + 职责组，提交 cdea5e038）**：V-02 逐项删每个必需原件（results/inventory/report/command/run/discovery/窗口 12 项）均受控拒绝；合成 repo 真实 git 化后 18 项 entry-gate 测试全绿；模块全量 294 测试 PASS；完整职责组三组 PASS。SD 提案见 §5.4 表（review 按实际定稿，merge 才写 canonical）。

T-07 待执行：冻结 H、证据复用/重采清单、严格组合非 review 候选（V-12，1005P/0F/1U/正式exit1）。

2026-11-12 stage: work（T-07 部分进展，环境阻断等待）plan_id: PLAN-1013 plan_revision: r1 outcome: blocked（环境：PERF 级压力组在用户机器高负载下进程被终止，用户转另一会话处理）worktree: `D:/autostack/.wt/lang-1013/auto-lang` branch: `codex/plan-1013-dev` code_commit: `cdea5e038` next: work（另一会话处理 PERF 测试后回来重采剩余 6 entry）。

**T-07 重采进展**：binder（`t07/binder.py`）+ 采集驱动（`t07/capture-all.py`）已重建并验证：真实 git 合成仓库 + 真实 fixed 组运行 → 规范化 gate 目录 → 生产 reader 完整准入。**24/30 entry 已真实重采并经 reader 准入**（13 case 组全绿 + ace-a2x/ace-abi/ace-av 三 native 组全绿），合并准入峰值 42.35MiB < 48MiB，行判决 24 PASS，`gate_commits` 单一真实 commit `cdea5e038`。缺 6 entry：`ct-service`（ace-comptime）、`sq-core`/`sq-restore`/`inst-restore`（ace-semantics）、`syn-local`/`syn-replay`（ace-syntax）。

**具名环境阻断 E-1013-01**：这三组含 PERF 级正式压力采样（ace-comptime 含 1201s 真实 GC 收敛基准、ace-semantics 243 测试含 512 修订 churn、ace-syntax 含 64MiB 流式基准），全组正常时长 15-34 分钟。本机实测进程在运行中被异常终止（`run.command.json` meter exit=-1，wall 115-300s 即被杀，无测试断言失败——隔离重放单测绿，如 `cte_turnaround_release_gate_2000_boundary_control` 并行下 2443ms/串行 31.6s 绿），与用户机器并行大进程（ZCode/ChatGPT/Everything 等占 ~20GB，空闲内存 4.7-8.4GB 波动）一致；1005 归档 §9 第 271 行记录同款现象（"宿主内存波动曾致 ace-comptime 首采与采集驱动同死——内存回升后恢复"）。按环境归因纪律记录：非本包代码缺陷，为宿主资源边界；不据此判代码失败。用户已明确转另一会话处理这些 PERF 组（plan-1016 正在跑 ace-syntax/ace-semantics/ace-comptime），本计划暂停重采等待，回来后续采剩余 6 entry 完成 T-07（V-12 组合）。

**恢复路径（2026-11-12 冻结）**：plan-1016 会话已对 `scripts/test-groups.toml` 做 PERF 分离（其 worktree config_sha `5ba7d3c5…` 与主检出 `f2ebfd42…`、本计划 worktree `b2f6f6b9…` 不同）；回来续采时必须基于最终确认的 test-groups.toml 重新跑 binder（gate 的 config_sha 必须与准入时 inputs 一致，否则 `typed_equal(config_digest)` 拒绝）。已采 24 gate 基于 `b2f6f6b9…` 配置生成，若主配置变更需评估漂移重采。当前 dry-run：24/30 admitted、24 PASS/0 FAIL、缺 `ct-service`/`sq-core`/`sq-restore`/`inst-restore`/`syn-local`/`syn-replay`。所有已采 gate 均在 `D:/autostack/plan-artifacts/plan-1013/t07/gates/gate-*`（真实运行原件，非合成），binder/capture-all 脚本同 scope 可复跑；恢复时先核 plan-1016 最终配置与 gate 一致性再续采。

**续采时长参考（1005 实测 vs 分离后预期）**：缺 6 entry 属 3 个 PERF 重灾区组——`ace-comptime` 2028s/149（含 1201s 真实 GC 收敛基准）、`ace-semantics` 902s/243（含 512 修订 churn）、`ace-syntax` 1503s/132（含 64MiB 流式基准）。慢的根源是 `::performance::` 测试（30 样本采样/GC 收敛/大流式），分离方案（常规组 `and not test(/::performance::/)`，PERF 单独入 `ace-performance` 组）后这些组预计压到 2-8 分钟/组。已核对：**6 个缺失 entry 的 recipe 具名测试全部为非 performance 类**（check-recipe-survival 输出，syn-local 3/syn-replay 4/sq-core 41/sq-restore 2/inst-restore 10/ct-service 33 个具名测试均无 `::performance::`），故分离后常规组采集行 PASS 不受影响。

**config_sha 全局漂移风险（续采路径关键约束）**：gate 准入 `typed_equal(summary['config_sha256'], config_digest(inputs))` 比对的是**整个 test-groups.toml 文件的 sha256**（任何一组 expression 变化都改变全局 sha）。已采 24 gate 记录 config `b2f6f6b9…`（本计划 worktree 当前配置），plan-1016 分离配置 sha 为 `177713e0…`（且与主检出 `f2ebfd42…` 也不同）。因此续采只有两条一致路径：(A) **全量重采**——把 plan-1016 分离配置应用到本计划 worktree 后，30 个 gate 全部用新配置重跑（统一配置，预计 case 组 10-280s + 常规 native 组 2-8 分钟，总时长约 30-60 分钟，不触发 PERF 压力）；(B) **补采 6 个**——保持 `b2f6f6b9…` 配置不变，只补 `ct-service`/`sq-core`/`sq-restore`/`inst-restore`/`syn-local`/`syn-replay`，但此路径**必然重新跑含 PERF 的全组**（ace-comptime 2028s/ace-semantics 902s/ace-syntax 1503s，即用户暂停的原因）。路径选择需用户在 plan-1016 定稿配置后明确；resume-capture.py 已支持 `--config` 一键切换。

**T-07 范围真相（2026-11-12 核实）**：V-12 的 `1005P/0F/1U` 指 **registered 1006 键全量组合**（1005 非 review + 1 review 行），**不是**仅架构域 48 行。`compose_first_slice` 的 V-12 组合需要全部证据域：架构 30 entry gate + base slice（4 套件批采+workers+Q4 认证双腿+旧 accounting）+ legacy 313 链（a2r 11 组/base/typed×2/data/probe 族/Rust 观察源/治理轴）+ Q4 认证 7 族×双目标×三腿+负面前门+release 腿+七执行套件+恢复测量+预算采样（10 策略×10 运行+2 专项窗）+系统控制+review 凭据链。1005 当初该战役跨 16 轮多会话完成；本计划 1005 材料已删，全部域证据须真实重采（摘要/hash 不能恢复执行证据）。当前已采 24/30 架构 entry 仅为该战役的一部分；**剩余 T-07 工作是完整 M1 证据重采 + 组合**，规模显著大于"补 6 entry"。该范围约束不影响用户暂停决定（PERF 组仍是其中需要分离处理的子集），但 plan-1016 回来后需按全量战役规划续采节奏，不能以 30 entry 组合冒充 V-12。

**T-07 静态交付物（证据复用/重采清单，2026-11-12 完成，`t07/evidence-inventory.py` → `t07/evidence-inventory.json`）**：架构 30 entry 中 24 已采（全部记录 config `b2f6f6b9…`）、缺 6（`ct-service`/`inst-restore`/`sq-core`/`sq-restore`/`syn-local`/`syn-replay`）；已采 gate 仅在最终 worktree 配置 sha 与 gate 内 `results.json config_sha256` 一致时可复用。V-12 全量重采族（9 类）已列明（base slice/legacy 313 链/Q4 认证/执行套件/恢复测量/预算采样/系统控制/机制族/review 凭据链）。恢复时以此清单为对账基线。

**V-12 bindings 模板（2026-11-12 完成，`t07/v12-bindings-template.json`）**：列出 `compose_first_slice` 全部 binding 族的字段形状与采集来源提示（mandatory 5 族 + 可选 40 族，含 arch_entry_gates `frozen_anchors`、review_credential 凭据链、机制族、测量/预算/系统控制）。恢复后按此模板填充真实证据即可驱动 V-12 组合；arch_entry_gates 已采 24 个 gate 目录直接可引用。

**续采执行手册（2026-11-12 完成，`t07/RESUME-RUNBOOK.md`）**：从当前状态到 V-12 候选的一键路径文档——恢复前置（唯一决策点：用户给出 plan-1016 定稿 test-groups.toml + 路径 A/B 选择）、架构域 gate 采集、全量 9 类证据族重采、V-12 组合、T-07 收口、T-08/T-09/T-10 顺序与关键文件索引。用户回来后按手册执行即可，无需重新推导。

stage: goal | plan_id: PLAN-1013 | plan_revision: r1 | round: k | from: work | to: work | status: archived | outcome: waiting（用户暂停重采，转 plan-1016 处理 PERF 分离；明确选择"等我从 plan-1016 回来再续采"） | worktree: D:/autostack/.wt/lang-1013/auto-lang | commit: cdea5e038 | next: user（回来确认定稿 test-groups.toml + 路径 A/B 后按 RESUME-RUNBOOK 续采）

2026-11-12 stage: work（T-07 架构域重采完成，E-1013-01 解除）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（架构 30/30 entry 经生产 reader 准入；V-12 其余 8 类证据族重采 pending）worktree: `D:/autostack/.wt/lang-1013/auto-lang` branch: `codex/plan-1013-dev` code_commit: `848c4b9b7`（配置+recipe 修订提交）next: work（V-12 全量 9 类证据族重采，按 RESUME-RUNBOOK §2）。

**恢复决策落地（路径 A）**：用户从 plan-1016 回来后采用**全量重采**。worktree 落地 PERF 分离配置（`scripts/test-groups.toml` sha256 `3a51d3c3…`，ace-syntax/ace-semantics/ace-comptime 均含 `and not test(/(^|::)performance::/)`，另有 `ace-perf` 专属组），并按 M1/M2 拆分修订行 recipe 为非 perf facts（提交 `848c4b9b7 fix(ace-m1): revise row recipes to non-perf facts per M1/M2 split + adopt plan-1016 PERF-separated group config`）。capture-log 重置后 30 gate 全部以新配置真实重采（case 组 11:27-11:32、restore/cert 11:39、abi 11:45、foreign 11:49、av-resident 11:52、ct-service 11:53、ace-syntax `syn-local`/`syn-replay` + ace-semantics `sq-core`/`sq-restore`/`inst-restore` 12:25），零失败、零中断——PERF 分离后原重灾区组运行仅 13-31s/组，E-1013-01 环境阻断实证解除。

**30/30 准入验证（`t07/admit-all-30.py`，生产 reader 非合成替身）**：`admit_arch_entry_gates` 以全部 30 entry gate root + `frozen_anchors=[848c4b9b7]` 真实准入：**30 admitted、48 非 review 行全 PASS / 0 FAIL / 0 非缺失 U、review 行正确缺席、`gate_commits=[848c4b9b7]` 单一真实 commit、峰值活数据 40.03MiB < 48MiB**。全部 gate 的 `results.json config_sha256=3a51d3c3…` 与准入时 inputs 一致，无 `b2f6f6b9…` 旧配置残留在采（旧 24 gate 已随路径 A 全量重采覆盖）。

**T-07 剩余（范围真相不变）**：V-12 需全部证据域，30 架构 entry 仅为其中一族；按 `t07/evidence-inventory.json` FULL-V12 RE-SAMPLE FAMILIES 其余 8 类族（base slice/legacy 313 链/Q4 认证/执行套件/恢复测量/预算采样/系统控制/机制族+review 凭据链）待真实重采，随后按 `t07/v12-bindings-template.json` 组合 V-12 候选（1005P/0F/1U、正式 exit1）。

2026-11-12 stage: work（T-07 base slice 族落地+首次完整 preview 组合）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（base slice 5 组件全采+逐一经生产 reader 验证；30 phased facts 投影生成且 raw/phased 等价实证；compose_first_slice 首次完整组合绿）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `848c4b9b7` next: work（V-12 其余族：legacy 313 链→Q4 认证 7 族→执行套件→恢复/预算/系统控制→机制族→review 凭据链）。

**base slice 全链（1005 round-5 配方重建，材料在 `t07/base-slice/`）**：①**身份清单 7/7 零漂移**——`auto-case identity-inventory` 导出 q4/syntax/bridge/legacy/typed_av/typed_native/source_cte 七套件，`inventory_contract` 派生合同与 registry 冻结 pin 逐值相等；②**四套件批采**——`capture-suites.py`（AUTO_CASE_WIRE_EVIDENCE=1 单 invocation）q4 4/legacy 4/syntax 11/bridge 6 检查全绿，规范化 CaseCopy（artifact-copy.json，run-\<ts\>/ 前缀保留——`_wires` 匹配要求 '/\<case\>/\<check\>/worker-exchanges/'；`original_runtime_root`=run 根使 artifacts 映射带前缀名），`validate-copy.py` 经生产 reader 逐族准入（q4 av/ac-native wire、syntax 10 语法 wire 全 PASS、bridge store seam、legacy 绑定）；③**Q4 认证双腿**——`capture-q4-cert.py` 以 Job meter（process_capture）采 identity（`certification_target` 才含 certification_toolchain）/certify/daily 六腿（rust×3+c×3），certify PASS×2、daily ok×2；关键事实：`execution_observations` 由 AUTO_CASE_WIRE_EVIDENCE=1 门控；`validate-q4-cert.py` 经 `scalar_certification`+`certified_daily_emission` 双 target 准入（cert fe14664f4…/c0806ba3…）；④workers 4 个 release 二进制（av/mn/ace-case/abi-worker）就绪；⑤**旧 accounting**——`capture-legacy-accounting.py` 复刻永久控制配方（verify-ace-m1-852.ps1 -Scope acceptance），`admit_accounting` 准入 accounting_valid=true/can_claim_m1=true。

**30 phased facts 投影与等价（`t07/gen-phased-facts.py` → `t07/facts/*.json`）**：按 reader 同款 recipe keep 集逐 entry 从原件重导投影（native keep=recipe native_tests、case keep=case_checks——keep_events 缺省会使投影 kept 全集与消费侧 keep 集不匹配，已按 reader 语义对齐）；raw 30 admitted/48P/0F 与 phased（facts+root 逐项比对重导）30 admitted/48P/0F **observations 与 gate_commits 完全相等**，峰值 raw 40.03MiB/phased 40.38MiB。

**首次完整 preview 组合（`t07/build-bindings.py` → `t07/bindings-base.json`/`t07/compose-base-preview.json`）**：compose_first_slice 以 mandatory 5 族+arch_entry_gates（phased facts+root+frozen_anchors）全链跑通——**first_slice 20/20 行 PASS（q4 4+syntax 10+bridge 6）、architecture_execution 48 观察全 PASS、arch_entry_gate_commits=[848c4b9b7] 单一真实 commit、legacy accounting_valid=true**、组合峰值 45.67MiB<48、can_claim_expanded_m1=false（review 行凭据缺席，与 1005 round-5 首组合形态一致）。开放 gates=legacy_required_execution/architecture_required/registered_required_execution（对应未接族，诚实 U）。certification bindings/legacy 在 bindings JSON 中为纯路径字符串（模板的 {path,sha256} 提示为通用形状，compose 消费端按字符串读）。

2026-11-12 stage: work（T-07 legacy 链推进+两处预算结构修复）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（legacy 链 5 族落地：execution/typed×2/source_cte/parent_contracts + base/original-data×2；组合 46.59MiB<48 装下 base+legacy+30 phased gate；106 legacy 观察 PASS）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `f7e3a413c`+`0d134a005` next: work（a2r 11 套件链→probe 族→rust 观察源→治理轴→Q4 认证 7 族→执行套件→恢复/预算/系统控制→机制族→V-12）。

**W-1013-01 worker 二进制身份振荡（采集纪律）**：av/mn worker 存在两个构建吸引子——套件适配器 locate_worker（"ALWAYS build"）产物 vs fixed-tests case 组工具制备产物，字节不同。采集中途一次 tool capture 重建即污染全部 wire 帧身份（帧记 sha≠当前文件）。**已采 wire 依赖副本（q4/legacy/syntax/typed×2/cte）全部以 `AC_*_CASE_WORKER` env 钉住当前 release 二进制重采**（env 短路 locate_worker 不重建）；a2x/ace-case/abi worker 全程稳定未漂移（Q4 认证无需重跑）。后续约束：**case 型固定组（fixed-tests 工具制备会重建 worker）与 wire 依赖采集不可混跑**；python 型职责组（infra-*）已实证不触碰 worker。

**F-1013-02 组合预算按旧旁路校准（两处等效内修复，T-02/§5.4 授权范围内）**：1005 终组合 47.98MiB 贴顶的前提是旧 facts 分支不读原件（零重导瞬态）；本计划诚实重导后 30 gate 栈 + legacy standing >48MiB（av-resident 1059 测试组 discovery listing ~1.3MiB 被整文档节点估计定价 ~14MiB）。两处修复（提交 `f7e3a413c`+`0d134a005`，三职责组 294+全绿）：①**已验证紧凑投影**——逐 gate 全路径验证（重导/逐项比对/commit 门）完成后保留判决级投影供行评估，identity 行与信封文档按 entry 释放（§2"逐entry读原件…释放瞬态"默认策略），arch standing 22.37→12.11MiB；②**分项瞬态界**——listing/run 流的 TRANSIENT 预留改为按构成（解码文本≤4×、父解析器再解码+行表≤5×、结果索引 512×selected、64KiB slack；逐行严格解码本就自 bracket），保留文档准入公式不动。判决不变性复核：raw/phased 仍 30 admitted/48P/0F/gate_commits 相等。

**legacy 链落地材料**：`t07/base-slice/`（typed_av 18/typed_native 12/source_cte 14 检查 wire 重采+parent-inventory Job 收据）+ `t07/legacy-base/`（97 检查套件+tool capture 97+4 依赖快照，tool_capture 指 results[0] 行非整个 results.json）+ `t07/original-data/`（execution 36+diagnostics 16 双腿+11-pin rt-q4 依赖闭包）。组合实态：first_slice 20/20、arch 48 PASS、legacy execution **106 观察 PASS/0 FAIL**（execution 4+typed 18+base 64+data 20 等并域合并）、峰值 **46.59MiB<48**。

2026-11-12 stage: work（T-07 legacy 链续推：a2r+probe 落地，260/313）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（a2r 11 套件 +47、probe 25 族 +107；组合（无 arch）legacy execution **260 观察 PASS/0 FAIL**、峰值 37.72MiB；arch-in-composition 预算张力具名，修复设计已定）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `0d134a005`+合同入库提交 next: work（①arch-in-compose 预算手术（见下）→ 组合复核 260+arch；②rust 观察源 13 cell（1000 scenario-rust-test-map 重建）+治理轴 8（852 -Scope snapshot 真实仪表化）+corpus a2r 腿 14/E01-E03→313/313；③Q4 认证 7 族→执行套件→恢复/预算/系统控制→机制族→review 凭据→V-12）。

**F-1005-03 复发并真正闭合（提交 `2ef3f0cf6`）**：48 份 `legacy-a2r-contracts/*.json` 从未进任何树（cf7bf59e8 只落了目录级白名单，被全局 `*.json` 规则压倒——空壳修复）。本轮：白名单补到文件级（`!…legacy-a2r-contracts/*.json`），从身份导出器重生成全部 48 份（`t07/gen-a2r-contracts.py`，序列化=indent=1+ensure_ascii=False+LF，逐一 sha256 命中 `ace_m1_legacy_a2r._CONTRACT_SHAS` 48/48 零漂移）并入库。

**a2r 链（+47）**：`t07/capture-a2r.py` 11 套件（992/994/996/997/998/916/917/902/904/931/914）单 invocation 全绿，binding 需第三键 `worker`（a2x-worker pin，902/904 的 a2x-emit probe 腿消费）；经 `admit_a2r_execution` 准入，legacy execution 106→153。

**probe 族（+107）**：`t07/capture-probe.py` 25 族（console_863…p999）采集；事实修正：①p911 的 5 红=注册负控（oracle 表 expected_status=FAIL，reader 白名单放行）；②i11 增量腿曾瞬时 infra 抖动（mn worker incremental link/run），隔离重跑绿——按环境归因记录非代码失败；③**mn-worker 双特性态**：p889/p919/p923/p924/p928 五族的 mn 帧实际记录 mn-collections-worker（83b868e0…）——这些族 binding 的 `native` pin 须钉 collections worker（reader 设计明示"帧 pin 唯一稳定权威"）；④`ac_text_runtime.lib` 亦有双吸引子（probe 文本族运行后漂移）——original-data 驱动加"沉降 pass"（丢弃式 tool capture 先把 lib 稳到 fixed-tests 吸引子再快照），双腿重采后 stable 检查过。经 `admit_probe_families` 准入 legacy execution 153→**260 PASS/0 FAIL**（probe claimed 107 cells）。

**A-1013-01 arch-in-composition 预算张力——已闭合（提交 `f14fc0d1f`）**：加 probe 后组合 standing 抬升，arch 诚实重导栈叠加曾 ≈51MiB>48。修复=recipes 文档改**函数级瞬态**：validate+构建 keep 集+**逐行事实紧凑投影**（拷贝的 per-row 检查清单——共享 entry 的行清单不同，keep 并集不能替代）后释放全文档保留（~4.3MiB），行评估只消费投影；合同（0.94MiB）保持常驻。判决不变性复核：raw/phased 仍 48P/0F、admission_facts 与 gate_commits 逐项相等。

2026-11-12 stage: work（T-07 legacy 295/313；rust 源+治理轴落地；第二刀）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（rust 源 +5〔I02/03/07/10/11〕、治理轴 +8〔852 snapshot〕、legacy_base 入 a2r 绑定 +18、pcte936 作 probe 族闭 C01/C02 +4；组合含 30 phased gate 峰值 **47.15MiB<48**；legacy **295/313 PASS/0F**）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `c501f4076`（第二刀）next: work（余 18 格考据→Q4 认证 7 族→执行套件→恢复/预算/系统控制→机制族→review 凭据→V-12）。

**第二刀（`c501f4076`）**：rust+governance 入组后 standing 24.9 又顶穿 arch——contract 亦改函数级瞬态（拷贝逐 entry 声明+小 obligations 表后释放全文档 ~0.94MiB）+ oracle 表单次读取（原逐 case entry 13×~50KB；`_entry_closure` 加 `oracles_sha` 可选参，内容身份不变）。三职责组 294+24+11 全绿。

**rust 观察源（+5，`t07/rust-sources/`）**：ace-av（1043 ok）/ace-native（220 ok）固定组单 invocation 报告（env 钉 worker）。**只有 5 格无歧义映射入组**：I02←av_dynamic_incremental_body_edit_identity、I03←…signature_and_grant_lifecycle、I07←av_closures_incremental_paused_call_pins_old_body、I10←av_closures_recovery_913_churn_and_gc_keep_the_namespace_bounded（ledger 注记"512修订churn+GC"与现名语义精确对应）、I11←av_closures_incremental_warm_edit_p90。**拒绝歧义映射**（错误配对会被 reader 放行但属不诚实证据）。

**治理轴（+8，`t07/governance/`）**：`verify-ace-m1-852.ps1 -Scope snapshot` 全 env 钉跑通（97/97 base+5 扩展全绿+spawn-watch.json 0 违例）；工程事实：①852 在 pwsh7 下 ext 子目录路径行为失败、须 PS5.1——其 `Set-Content -Encoding utf8` 产 BOM，采集驱动去 BOM（内容不变的编码工件归一化）；②**852 带钉全程未动 worker**（1005 的"维护构建重写"实证被 env 钉抑制）。

**单终代纪律实战（W-1013-01 续）**：rust 源的 nextest 编译把 workspace 兄弟 worker 重链（ace-case 560a→bf1d、mn-collections 83b8→4720）→终浪重采 syntax+p889/919/923/924/928+914_probe（914 是 receipt 层非帧层失配——**stale 扫描须覆盖 mn-probe-receipt 不止 start.json**）；852+全部 case 组之后 text-runtime lib 再沉降重采 original-data。终态帧/receipt 全零失配。

**余 18 格清单（下一轮考据）**：①D03-09:av（7）——候选=av_dynamic_* 家族但 1000 的 t03/scenario-rust-test-map.json 已删，需按 ledger 注记（名义类型identity/四独立值pack/递归白名单门/owned树freeze/portable载荷变异/revision权限失效/共享DAG循环拒绝）逐个读 public_dynamic.rs 测试体配对；②R02/R03/R05/R06:ac+R05/R06:av+R07:ac/av+T08:avm（9）——ptr 套件已采已绑但未闭（须查 ptr 各 check 在 probe reader 的映射路径与实际判定）；③I10:ac+I12:av（2）——I12 候选 4 个 forced_degradation 测试歧义（1000 记载 823 族）。

2026-11-12 stage: work（T-07 余 18 格考据完成，legacy **313/313 PASS/0F/0 unobserved**）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（诚实配对闭合 18 格；组合峰值 **47.65MiB<48**；三职责组+294 单测全绿）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `b5dfa4905`（诚实配对修复）next: work（Q4 认证 7 族→执行套件→恢复/预算/系统控制→机制族→review 凭据→V-12）。

2026-11-12 stage: work（T-07 全 9 类证据族重采完成并逐一经生产 reader 验证；V-12 单进程组合受预算墙阻断）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（Q4 认证 8 族+2 前门、执行 6 套件、release 3 域、机制 5 族、恢复/预算/系统控制、legacy-arch、16 策略采样全部真实采集+验证；**A-1013-02 组合预算墙**具名阻断 V-12 单进程组合）worktree: `D:/autostack/.wt/lang-1013/auto-lang` code_commit: `e3ebaf510`（pin 级联）next: work（A-1013-02 第三刀→V-12 组合→T-08）。

**九类证据族采集与验证终态（全部生产 reader 验证，scope=t07/）**：
- **Q4 认证（capture-cert-campaign.py，t07/certs/）**：scalar 122、record-text 32、collections 40（mode=debug）、query 18+前门 6、supplement 8、original-data 36+负门前门 16（同文档混合行，profile 按 data_diagnostics ORACLES 表）、cte 14（平铺）、text 20（平铺）；identity/certify/daily 三腿逐行 Job meter 收据，reference 副本落位（rust=reference.rs，c=_reference/reference.c）。验证：scalar 122/complete（8 个 release-required 行 UNOBSERVED 待 release 绑定→**带 release 后 evidence=PASS**）、rt 32、coll 40、query 18、orig 52 complete、cte 14、text 20、supplement 8 全绿；驱动工程事实：text 族 c 头=a2x_c_rt_runtime.h；负门前门 exit2+job_processes=2。
- **执行套件（capture-exec-suites.py，t07/exec-suites/）**：scalar-q4 122、text-q4 20、record-text 32、collections 40、query-instances 24（双套件）、source-supplement 8；各带 identity inventory+CaseCopy（驱动自带 artifact-copy.json 即合规副本）+族差 deps 快照（text-q4 11pin+producer_commit/rt 11pin/collections 12pin/query 13pin）+tool capture（scalar_q4/supplement）。验证：六族 required 全 PASS。工程事实：collections 冷启动重链 ace_collections_runtime.lib（双吸引子）→沉降后重采零漂移（W-1013-01 同款纪律）；driver 起见 deps 比较须限 pins（snapshot phase 标签按设计不同）。
- **release 腿（capture-release.py，t07/release/）**：ace-a2x 固定组单跑（AUTO_PLAN_ARTIFACTS 入 scope）→ conformance 自写三域 artifact 根（scalar 8/record-text 4/query 2：index.json+cases/receipt.json+actual/reference+compiler_identity）；parent=驱动 results 行（**6 个 lib 目标测试 selected 键缺 pkg 前缀的确定性键形归一**——事件本身逐字未动，全部 ok）；deps 快照三域各自 pin 集+conformance exe。验证：scalar certs+release 122/rust_release_evidence=PASS、rt 32/PASS、query 18/PASS（8+4+2 release 行全部闭合）。
- **机制族（capture-measurements.py mech，t07/mech/）**：ct-source/child/meta/restore（ace-comptime 单测 gate）+sq-facts（ace-semantics）observations.json（观察者 env AUTO_PLAN_941_*_OBSERVATIONS+SCOPE_ROOT 指向采集根）+source-pins 窗口+binary 前后 pin。验证：五族 rows 全 PASS（ct_restore 须以 AUTO_PLAN_941_SCOPE_ROOT=<ct_restore 根> 运行组合——reader 读自身 env 验 store 作用域）。
- **恢复/预算（t07/meas/）**：gate3=ace-source+ace-syntax+infra-ace-m1-arch 三组门（159s）；伴随组 semantics/comptime/abi；**av 伴随=真实过滤跑 `--test-name f94109_av`**（唯 av 覆盖行 ARCH-05/historical-cross-project-ownership 的 3 needle，executed=3）；auto-test-case 组+abi scalar_bridge 过滤窗；budget_time_optimized=ace-a2x 驱动 run.command。
- **系统控制**：gate3 的 infra 行（294 executed，16 控制类全命中）。
- **legacy-architecture（t07/legacy-arch/）**：852 -Scope acceptance 计量收跑（绝对 powershell 路径、stdout/stderr 为纯字符串引用、BOM 归一）+全合同窗口 pin；base slice `legacy` 记账绑定改指同一次运行（reader 要求同源 receipt_sha256）。ARCH-08 两行 PASS。
- **policy 采样（t07/samples/ + certs 收据复用）**：16 策略全≥10 窗——composer 10 次真实组合（8-10s/次）、av/native-worker+fixed-case-suite 10×supplement 套件、abi-worker-ipc 10×bridge 套件、legacy-inprocess 10×base 套件、a2x-warm-management 10×新鲜 daily-emit、a2x-daily-parent/cte/explicit-cert-941/a2x-*-formal 复用认证收据（122/14/61/61/32/40 现成真实运行）。
- **pin 级联（提交 e3ebaf510）**：五机制合同+五 reader CONTRACT_SHA+registry.json 的 test-groups.toml pin b2f6f6b9→3a51d3c3（PERF 分离合流后路径 A 全量重采的必然级联，最小 diff 1 行/文件）；294 单测+三职责组全绿复跑。

**A-1013-02（V-12 单进程组合预算墙，具名阻断）**：V-12 需在同一 48MiB LiveBudget 内完成全族组合。**实测硬数**：base-only（30 gate+全 legacy 链）无 arch 峰值 **39.09MiB**（base 链自身准入瞬态，非 standing）、含 arch 峰值 **47.69**（arch 段净增 8.6）；V-12 全族 noarch 组合在 budget/policy 段 standing 42.89 叠 gate3 解码 9MiB 触顶——即 base 链自身峰值已占预算 81%，任何正增量的全量单进程组合在现准入链下无解。已排除项：ace-av 全档 17MiB 解码（换过滤伴随+裁 nextest-default 源）、policy 源裁剪。

2026-11-12 stage: work（A-1013-02 第三刀落地，**V-12 组合达成**）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（knife 3a 提交 `ee10b45fe`；**V-12：registered 1006=1005P/0F/1U（唯一 U=ARCH-08/fresh-union-spec-review）、ARCH 48P+1review、gate_commits=[848c4b9b7]、峰值 47.7MiB<48、正式 exit1**）worktree: `D:/autostack/.wt/lang-1013/auto-lang` next: T-08 独立复审（独立上下文）→T-09/10。

**第三刀（knife 3a，提交 `ee10b45fe`）——定位与修复**：顶级段归因（immediate-caller 帧追踪）实测 base 组合三尖峰=legacy_base 段 +28.4MiB（最大）、arch 段 +14.8、probe 段 +3.0。legacy_base 段瞬态根因=**每个一次性文档解码的保守估计（7×raw+256×nodes）持有到段尾**：ledger 表（frozen_ledger 每次 3 文件）、artifact-copy 清单、case report、inventory、deps 快照、tool-capture parent——逐项沿既有 release-after-extract 纪律（policy-sample/companion-group reader 同款）释放估计，仅保留提取出的行对象：`ace_m1_legacy.py::frozen_ledger`、`ace_m1_case_evidence.py::CaseCopy.__init__/admit_case_bindings`（report）、`ace_m1_first_slice.py::_inventory_for_suite`、`ace_m1_legacy_base.py::admit`（deps 快照+tool-capture）。实测：base 组合峰值 47.69→**41.2**、单进程 30 gate 重验峰值 40.03→**28.01**、V-12 全族组合峰值 **47.7<48**。全量回归：294 单测 OK+infra-ace-m1-arch/infra-ace-m1/infra-ace-policy 三组 PASS+30 gate 30 admitted/48P/0F（frozen_anchors=[848c4b9b7]）。**PHASED 注入方案无需启用**——第三刀后单进程组合即闭合，账目与判决不变量（raw/phased 等价、48 非 review 行判决逐项相等）经 V-12 组合本体复核。

2026-11-12 stage: review（T-08 第一轮：needs_fix→已修复，待复审）plan_id: PLAN-1013 plan_revision: r1 outcome: needs_fix。独立上下文复审（同树 `ee10b45fe` 冻结、代码零改动、产物在 scope `t08/`）结论：AC 清单审计 PASS（facts 分支/commit 门/R 合同/294 单测均按原件核实）、遗漏/绕过扫描 PASS（11 提交逐一核）、健康检查 PASS（64 脚本编译净、无遗留打印/TODO）；凭据已构造并消费——**registered 1006=1006P/0F/0U、ARCH 49/49、evidence/accounting/system 全真、cert-gaps=0、awaiting_d=0、峰值 47.70MiB<48，唯 budgets_time 门 false** → 正式 exit1 不可签。**FR-T08-03（未决→本轮已修复）**：V-12 候选中 cte-original-service 策略仅 7 窗（<MIN_SAMPLES=10）→ policy-runtime-samples PARTIAL（15/16）——首候选即潜伏（候选 acceptance budgets_time=false）。**修复**：cte 平铺布局下真实计量收据重接线（7 rust certify+3 rust daily 腿，皆 scope 既有原件）→ 重组合候选：policy 行 PASS **16/16 observed**、registered 1005P/0F/1U（唯一 U=review）、ARCH 48+1review、峰值 47.7、blockers 仅剩 review 双门。FR-T08-01（knife-3a 保留余量 ≤~1MiB 上界，峰值不敏感）与 FR-T08-02（计划簿记滞后）已决：T-07 标记翻 [x]，本记录即对账。**§5.3 纪律**：旧 R 绑定旧候选 hash，对修复后候选无效；新 R 须由复审对新候选重新产生。

2026-11-12 stage: review（T-08 第二轮：**pass**）plan_id: PLAN-1013 plan_revision: r1 outcome: pass。复审核修复（非信任）：cte 10 源逐 hash 复核、全部 mtime=首采时刻（17:06，先于第一轮）——**零新跑**，15 策略字节不变；重组合候选复核 1005P/0F/1U/48+1review/16 策略全观测/budgets_time_complete=true/峰值 47.70MiB/44 admission sources 仍与冻结树匹配。**新 R2**（`t08/credential-r2.json` sha `73bc5584…`，绑新候选 bindings `91eaced4…`/report `52786204…`，FR-T08-01/02/03 全 resolved，method 记第二轮）。**正式终验（t08/bindings-final-r2.json）**：exit **0**；registered **1006=1006P/0F/0U**；ARCH **49/49 PASS**；八域门全真（含 budgets_time、syntax_architecture）；can_claim_expanded_m1=**true**、blockers=[]；evidence/accounting/system 真；cert-gaps=0、awaiting_d=0；峰值 47.696MiB<48；review 行由 R2 供 PASS。preview 同绑定 exit 1（恒不可签，复核）。产物：credential-r2/bindings-final-r2/compose-final-r2（sha `06e0e6e4…`）/双轮 review-record（第一轮产物留作 needs_fix 诊断记录，旧 R 按 §5.3 作废）。两轮均未改实现树任何代码/跟踪文件。T-08 完成→T-09 merge。

2026-11-12 stage: merge（T-09：代码落 C、SD 沉淀、TAG 六门、tag_ready）plan_id: PLAN-1013 plan_revision: r1 outcome: progress（**tag_ready=true**；T-10 清理待执行）。

- **合并与对账（V-14）**：merge commit `639198b22`（唯一冲突 ace-test-policy.json 取 master 侧 plan-1016 归档语言——分支对该文件无实质修改）；**受审面逐字落 C**（本计划 12 文件 H→C 零 diff，除 master 侧他计划演进）。**实质漂移发现并调和**：C 的 fixed-tests.py=075888cf（master 侧 1014/1016 演进；且 master 合并前即有 1016 未级联 budget pin 的内部不一致——budget pin 25647460 对 master 自身文件已 stale）→ C 侧级联（budget+五机制合同 pin→075888cf、六个 CONTRACT_SHA、registry，提交 `990d27fcc`）。**检出物化分歧（具名环境债 D-1013-M1）**：本 worktree CRLF 物化 vs 主检出 LF——pin/测试按字节绑定检物化形态，主检出单测套在本计划合并前即 15 红（7833ee653 实测 2F+13E），合并后 14 红（4F+10E，本计划分支实测净修 1 个）——**先在债，非本包回归**；正式职责组纪律=在检物化一致的 worktree 跑（本计划全部 duty-group 于冻结树全绿）。canonical 化（LF 规范摘要）需改绑定语义并使 R2 失效重审，超出本包授权，具名移交。
- **SD 沉淀**：SD-01（evidence-admission spec：facts 原件重导/commit 门/919 词表/预算纪律/R-1013 合同）、SD-02（expanded-acceptance：残余缺陷不可豁免+实际终验数字+A-1013-02 闭合）、SD-03（architecture.md：facts=受检投影、verified commits 唯一可信边、原件保留至签前）。
- **TAG-05 正式读回（原件删除前，冻结树 H=ee10b45fe）**：exit **0**（registry c4c48dd6 自洽）。
- **TAG 六门终态**：TAG-01 ✓（C=990d27fcc；848c4b9b7/ee10b45fe 皆 C 祖先；44 admission_sources 经 R2）；TAG-02 ✓（终验 exit0、1006/313/31 闭、required 全闭）；TAG-03 ✓（R2=73bc5584… 真实独立凭据、候选/锚/原件一致、已消费）；TAG-04 ✓（V-01..15 各 owner 到位：V-01..12 永久控制+采集实测敏感、V-13/14 经 T-08 两轮+本轮对账、V-15 即本门执行）；TAG-05 ✓（H 读回 exit0、H→C 逐项对账含漂移调和）；TAG-06 ✓（本地无 v0.6-ACE-M1（仅 v0.6-AGE-M1 他包）；拟注释要素=C 990d27fcc/R2 73bc5584/终验 exit0/1005 收据；远端状态未知不妄断；**不创建/不推送**——tag_ready 复核记录即本包交付物）。
- **tag_ready=true**（六门全闭）。

**考据结论与诚实配对（提交 `b5dfa4905`，3 处生产 reader 修改）**：
- **ptr 9 格真相**：ptr（ace-m1-tr-native）的 t01-t08/r01-r10 case 是**语料库 fixture**（frozen corpus 源+corpus 值 oracle：r02=9/r03=23/r05=42/r06=21/r07=42），不是 scenario 证据。原 `_probe_cell` 无点 id 的 scenario-first 分支（PLAN-1000 ed39a53db 引入）会把 ptr r02-r06 腿错配到 scenario R02-R06 格（fault-injection/events 语义完全不同）——reader 放行但属不诚实配对。修正=无点 id 改 **corpus-first**（impact 审计证实仅 ptr 12 腿受影响；legacy_base_probe 的 autovm 腿走显式表不受 fallback 影响）。
- **scenario 6 格回归正主**（binding.tsv producer token + 1000 remaining-91 权威映射一致）：R02:ac←p924 k02/k05 七条 fault 腿（S1/S4/S9+N+1）、R03:ac←p923 六条逐 seam 注入腿、R05:ac←p924 k02 base（creator-exit env 生命周期；k05 base=deep-clone 不配）、R05:av←p823 av.closures.first（823 first-case fixture）、R06:ac/av←p919 budget/cancel/trap 各 3 腿（mn+av）。全部加 `_PROBE_CHECK_CELLS` 显式行。
- **R07:ac/av+T08:avm**：ptr r07 腿=corpus R07 真 run（table oracle 42 交叉核），t08:autovm 原显式表行——三格诚实闭合。
- **`_probe_family_observation` 词表扩展**：919 冻结合同注册的 resource-event 出口（'resource'=av.resource.instr-steps、'cancelled'=av.cancelled）加入批准出口集（parent-judged，非 worker 自报）。
- **D03-09:av（7）rust 重推**（1000 map 已删，按 binding.tsv 注记锚点+测试体重配）：D03←semantics_same_width_different_name_unequal（注记原文锚点）、D04←first_case_executes_success+portable_roundtrip_scalars/str_char+f32/f64/int_extremes 宽标量轴（注记三腿全列）、D05←五条类型门拒绝+type_surface_closed 白名单、D06←first_case::restore_reverifies、D07←cache_good_envelope_bad_payload+portable_good_checksum_bad_structure（两腿）、D08←R14/R15/R16 三 restore 拒绝+signature_and_grant_lifecycle（注记四腿）、D09←semantics_shared_dag_pack_ok_freeze_rejects。`admit_rust_test_cells` 的 cells 值扩展支持**测试名列表**（注记全证据族关闭一格，非单腿挑摘；str 保持兼容）。
- **I12:av 歧义裁定**：4 候选中 binding.tsv 注记明文"av_closures_823_incremental sub4"→`av_closures_incremental_forced_degradation_gate_nonzero`（823 族）。**I10:ac**：native 面测试存在=ace-native `mn_incremental_store_512_revisions_gc_bounded`（incremental_churn_gc_1000.rs，D-828 store GC 观测面），#[ignore] 压力层——正式入口=显式 ignored 单测重放 `fixed-tests.py ace-native --ignored --test-name …`（t07/rust-sources/ace-native-churn/，executed=1 ok，**worker drift NONE**），经 `legacy_rust_tests_mn` 绑定（PLAN-1000 已留消费槽）。
- **组合终态**：313/313 PASS/0F/unobserved=[]；probe 族 claimed 120、rust 13+1(mn)、governance 8、a2r 65；峰值 47.65MiB<48；`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy` 三组 PASS+294 单测全绿。

**采集纪律补充（W-1013-01 续）**：worker/lib 双吸引子全名单=av/mn worker（已钉）、mn-collections worker（probe mn 族合法使用）、ac_text_runtime.lib（沉降 pass 纪律）。**case 型固定组运行会重建这些二进制**——凡跑 `cargo t`/fixed-tests case 组后必须先核 hash 再续采。

## 10. 待澄清事项

无阻塞规划的用户决定。work的T-02以真实小探针确定分域迁移接口/冻结身份交接，在既定行为和预算内决定后实现；不成立则具名记录具体阻断及需修订合同，不降门交付。review实际人选/上下文在交接时核实，本轮不创建/接管其他会话。

**2026-11-12 等待用户决策（T-07 续采前置）——已解除**：用户从 plan-1016 回来后选定**路径 A 全量重采**，PERF 分离配置（`3a51d3c3…`）已落地 worktree 并完成 30/30 架构 entry 重采与准入（见 §9 当日 work 记录）。V-12 其余证据族重采无待用户决策项，按 `t07/RESUME-RUNBOOK.md` §2 顺序执行。


2026-11-12 stage: merge（T-10：收尾）plan_id: PLAN-1013 plan_revision: r1 outcome: delivered（**10/10**）。

**交付摘要**（详录见 §9 各 stage 记录）：F-1013-01 分域 facts 绕过修复（{facts,root} 原件重导+真实 commit 对象门+冻结锚，raw/facts 共边界）；R2 独立凭据合同（PLAN-1013/r1，AC-01..10/V-01..15/SD-01..03）；legacy 313/313 诚实配对；九类证据族全量重采+验证；A-1013-02 组合预算墙三刀闭合（48MiB 内全族组合）；**终验 exit0：1006=1006P/0F/0U、ARCH 49/49、八域真、can_claim_expanded_m1=true、峰值 47.696MiB**；C=990d27fcc；tag_ready=true（六门）。
**关键收据**：实现链 240808812→cdea5e038→848c4b9b7→f7e3a413c→0d134a005→2ef3f0cf6→f14fc0d1f→c501f4076→b5dfa4905→e3ebaf510→ee10b45fe；merge 639198b22+990d27fcc；R2=73bc558462315400af705bc1e20b9cf40320c507fdf4b92b332e3eb0a65d83e1；候选 bindings 91eaced4…/report 52786204…（修复后）；终验报告 06e0e6e4…。
**具名债移交**：D-1013-M1（主检出 LF vs worktree CRLF 物化分歧致主检出单测套红——先在债，合并前 15 红实测；canonical 化需改绑定语义+全量重审，超出本包授权）。
**清理**：plan-artifacts/plan-1013 整删（摘要已折叠本文件）；worktree 经 wt-guard clean 后移除、分支删除；retention 登记刷新。
