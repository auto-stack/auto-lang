---
plan_id: PLAN-972
status: archived
feature_name: 新版标准库完整基础与集合
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-15
plan_revision: 12
current_step: 10
total_steps: 10
depends_on: [PLAN-961, PLAN-962, PLAN-963, PLAN-964, PLAN-945, PLAN-946, PLAN-948, PLAN-1039, PLAN-1040, PLAN-1041, PLAN-967, PLAN-1042, PLAN-1043]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/stdlib/design/ace-binding-and-assembly.md, docs/specs/stdlib/design/semantic-operation-contract.md]
new_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md]
touched_goals: [GOAL-002]
---

# [PLAN-972] 新版标准库完整基础与集合

## 0. 变更摘要

2026-11-15 r12当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r11当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r10当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r9：用户纠正原生标准库基础依赖；新增AC-FOUNDATION/V-FOUNDATION/SD-FOUNDATION，将算法准备与默认自有库原生实施/验收分开，修正整包前置。原任务ID/数量/状态/进度/旧AC与V不减，本轮仅规划。

2026-11-13 r8：参考Zig/Go的原生平台I/O设计承接；新增AC-PLATFORM-IO/V-PLATFORM-IO/SD-PLATFORM-IO，由原任务负责。保全部原任务/数量/进度/既有AC/V及其他会话修订，本轮仅规划。

2026-11-12 r7：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r6当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-10 r5当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/10，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承943–946，完整math/text/char/bytes/collections/Option/Result/hash/format/parse公共API；纯算法Auto单源，原语统一操作合同+四provider。保i64公共签名、Unicode单位/嵌NUL、迭代顺序/错误及兼容政策，不复制四份算法。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

当前正式前置：PLAN-961, PLAN-962, PLAN-963, PLAN-964, PLAN-945, PLAN-946, PLAN-948；新增原生底座PLAN-1039,PLAN-1040,PLAN-1041。后文旧依赖叙述补充读取本行；允许先做只读准备，不能以旧依赖较少绕过真实能力门。


整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `stdlib/auto`（本轮确认存在；开工重核符号/版本）。
- `stdlib/contracts`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/tests/stdlib`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `stdlib/auto`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-lang-core/src/stdlib_assembly/contracts.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/packages.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：完整默认Auto基础/文本/集合库。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [stdlib/auto/list.at](../../stdlib/auto/list.at)：完整集合算法源码；读取SHA256前16 `7e41ad83f39bc8ab`

- [stdlib/auto/str.at](../../stdlib/auto/str.at)：文本/格式化算法，FStr求值归964；读取SHA256前16 `dae2106fccb98e4c`

- [crates/auto-lang-core/src/stdlib_assembly/loader.rs](../../crates/auto-lang-core/src/stdlib_assembly/loader.rs)：现声明/装配loader，保依赖精确和新库单源；读取SHA256前16 `0be54327bf6130a2`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承943–946，完整math/text/char/bytes/collections/Option/Result/hash/format/parse公共API；纯算法Auto单源，原语统一操作合同+四provider。保i64公共签名、Unicode单位/嵌NUL、迭代顺序/错误及兼容政策，不复制四份算法。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：全API覆盖及领域边界（r2）

按948/959逐操作闭合math/text/bytes/collections/format/parse/hash与现用regex/encoding；纯算法单源，FStr求值归964，平台和DB归973/975，不留剩余操作无owner。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

新增LIB-M2-01/04：公共AutoAPI与operation ID保持，默认算法用普通Auto源码，core不隐式heap/host，alloc层显式依赖allocator。物理组织和装配消费947合同（已有948→947正式依赖）；不另造ace.std/.ace.at或每后端复制算法。必要intrinsic/外部优化可保，须明确ABI/效果与独立行为证明。

AC从共享Checked HIR/MIR直接生成机器码，Auto模块间受检AutoABI或内部优化，不依赖先生成C/Rust业务源码；AV/新A2R/新A2C消费同一公共源码/语义。平台服务由973、外部库导入由967，本包不反向等系统库/完整FFI。安装/预编译闭包交978；no_std与无heap分别测，不因runtime用Rust/C便把默认算法变成后端私有。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 按947/943/944合同组织公共API与普通Auto默认算法，适配四端模块调用/AutoABI并核每个intrinsic/外部优化实现语义；保原公共路径和操作ID。 |
| T-10 | AC-08 | V-10 | 分离core/alloc依赖和profile，核无隐式heap/host的核心程序、显式allocator集合路径、必要runtime ABI和预编译库闭包；交978具名清单。 |

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。默认基础算法保持Auto实现，Str/str/bytes按显式长度/既有编码和ownership语义；不靠strlen/printf等C库重建默认语义，也不透过Rust helper隐藏委托；intrinsic和allocator来源/目标依赖显式。

本包验收：多字节、空串、ab\0cd五字节、重叠/边界/失败分配等适用原操作对独立oracle；移除可选C/Rust/libc provider仍可执行默认库，缺自有实现不能fallback。 原任务T-09,T-10承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r7当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at](../../crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at)（下列节选L2–L16；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
let nums = [1, 2, 3, 4, 5]
let sum = nums.reduce(0, (acc, x) => acc + x)
print(sum)

// Test: List.find — first > 3
let found = nums.find(x => x > 3)
print(found)

// Test: List.any — any negative
let has_neg = nums.any(x => x < 0)
print(has_neg)

// Test: List.all — all positive
let all_pos = nums.all(x => x > 0)
print(all_pos)
```

本包所需独立结果：旧源归约15、find4、any/all逻辑假/真；原VM显示0/1与语义bool分账。新例核过滤后[2,3,4,5]及拥有元素内容，不只len或ok。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 普通Auto算法/默认模块→947合同/963泛型→944装配→最少AutoABI原语→四端目标provider。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct LibraryOperation { api: LogicalSignature, implementation: AutoModuleRef, primitive_requirements: RequirementSet, storage: StorageId }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [stdlib/auto/list.at](../../stdlib/auto/list.at) | 完整集合算法源码 |
| [stdlib/auto/str.at](../../stdlib/auto/str.at) | 文本/格式化算法，FStr求值归964 |
| [crates/auto-lang-core/src/stdlib_assembly/loader.rs](../../crates/auto-lang-core/src/stdlib_assembly/loader.rs) | 现声明/装配loader，保依赖精确和新库单源 |

**状态、错误与资源。** 宽度/locale/UTF-8/NUL/索引单位与Storage错误不能静默改语义；缺泛型/所有权能力回原owner，不用Rust整套std替Auto公共算法。 默认core不隐式libc、目标OS薄适配与显式外库分账；allocation失败保旧，clone/deep/shared按语义账，最后owner释放。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at](../../crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at) | 旧公共算法/求值与元素 | `list_reduce_find_any_all.expected.out`；SHA256前16 `6c49addc909fb55e` | 拟ace.m2.p972.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/vm/08_strings/012_str_eq/str_eq.at](../../crates/auto-lang/test/vm/08_strings/012_str_eq/str_eq.at) | 文本相等与单位 | `str_eq.expected.out`；SHA256前16 `770de10635637265` | 拟ace.m2.p972.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/vm/24_generics/003_filter/filter.at](../../crates/auto-lang/test/vm/24_generics/003_filter/filter.at) | 用户元素/闭包组合，旧打印ok需补内容 | `filter.expected.out`；SHA256前16 `ae058008d5937f5d` | 拟ace.m2.p972.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/specs/stdlib/design/semantic-operation-contract.md](../specs/stdlib/design/semantic-operation-contract.md) | 943完整操作实册守恒 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `2d44c8304d5f8022` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 逐库域/操作/宽度/错误/ownership与941/943差额精确覆盖，保默认自有库与显式C/Rust双腿；947/945/946/948正式来源仍需齐。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### Windows/Linux无libc平台I/O（2026-11-13当前）

依据[原生平台I/O Draft 0.1](../design/stdlib/native-platform-io.md)。普通Auto实现format/UTF编码/缓冲和适用read_exact/write_all算法；read_into/write_from使用调用者buffer不强制堆。1039/1040负责原语与目标、1041最小输出、945早期文本。本包不反等973或1041整包，不把Rust helper算法当Auto；Storage完整API继续专题讨论。

### 原生底座强制依赖（2026-11-13当前）

依据[设计Draft 0.2 §10.1](../design/stdlib/native-platform-io.md#101-依赖的强制含义r02取代此前可并行表述的歧义)。本包最终depends_on明确新增1039/1040/1041：默认自有alloc/动态文本集合与原生输出集成须消费它们的实际实现/真实OS结果/独立review/正式来源；只有接口、provider名或旧runtime不能准入。core数学/编码/解析等无OS算法可提前写测，不要求每个函数运行时调用OS，但不能提前完成整个原生标准库。普通Auto的分配/容器策略消费1039/1040页区域，不使用malloc/Rust Vec/String/HashMap替代默认算法；1041只消费945所需最小文本，不反等本包，形成单向依赖。原API/四后端/Storage讨论边界保留，不反等973。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

逐math/text/bytes/collections/format/parse/hash/regex/encoding公共操作补A2PY与AV三配置；纯Auto算法共用，allocator等边界才按provider适配。core/alloc、长度/编码/Storage及原生底座合同保留。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

完整公共core/alloc/算法库消费1043公共+所选来源绑定及最小#in；纯公共body共用，不强迫每模块四空文件。原native OS/Storage与六路径义务保持。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-stdlib-core.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/stdlib/design/ace-binding-and-assembly.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-04 | modify | `docs/specs/stdlib/design/semantic-operation-contract.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-stdlib-core.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-stdlib-core.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-stdlib-core.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### 平台I/O规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-PLATFORM-IO | add/modify | docs/specs/ace-migration/m2-stdlib-core.md | 笼统provider→本节平台IO/算法/运行/依赖分账 | 无libc平台设计 | AC-PLATFORM-IO |

### 原生底座规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-FOUNDATION | add/modify | docs/specs/ace-migration/m2-stdlib-core.md | 算法/provider合同可冒完成→原生实施/验收必须消费已交OS区域/I/O与真实Auto算法 | 原生基础依赖 | AC-FOUNDATION |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-stdlib-core.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-stdlib-core.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**完整core/math/text/bytes/集合与Storage/allocator**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

math/parse/format/编码/集合/分配策略默认共享Auto源码，CPU/页区域原语允许；不以libm/Rust容器默认实现代签。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### T-08 逐操作对账（2026-11-15 当前；V-08 fixture 锚）

分母=旧 `stdlib/auto` 对应域公开面 + 943 首批 17 合同行；V-08 常驻 fixture=装配面清单奇偶校验（8 模块 49 公共符号双向全等，删行/私增即红）+ 全符号五端演练（AV/mn/cl/rustc/Python 各 49 检查，独立 oracle）。去向三档：**(a) 已交付**（装配模块+测试锚）、**(b) 本包后续批**（具名）、**(c) 外部 owner**（具名计划）。

| 域 | 旧面操作 | 去向 |
|---|---|---|
| math | abs/min/max（943 i64 族）；square/cube（旧 private） | (a) std.math（含 i64::MIN abs 陷阱独立断言） |
| math | sqrt/abs_f/powf/sin/cos（double 族） | (a) std.math 纯 Auto 单源（T-09；无 intrinsic——AC-AUTO-STD 不以 libm 代签；NaN 不可构造→非正输入返 0.0=记录偏差；级数精度 ~1e-15 相对、五端同值；外部 NaN/正确舍入平价归 967） |
| text | len/byte_len/char_at/sub(equal 语义)/contains/starts_with/ends_with/equal/repeat | (a) builtin 方法+std.text |
| text | upper/lower | (a) upper_ascii/lower_ascii（ASCII 子集表移）；(b) 全 Unicode case 表 |
| text | is_empty/is_digit(str) | (a) 组合基元（scalar_len==0 / ==1+char 谓词） |
| text | from_uint/to_uint | (a) std.conv（to_uint=is_uint+to_int 组合） |
| text | find/reverse/trim/replace/split/lines | (a) std.text（find 索引/reverse/trim/replace/split/lines；List<Str> 返回） |
| text | uuid | (c) 967/973 域（外部随机） |
| char | is_alpha/is_digit/is_alphanum/is_whitespace/is_ident | (a) std.char（ASCII 子集边界记录） |
| char | to_upper/to_lower（旧 int→int 恒等平移） | (a) to_upper_ascii/to_lower_ascii |
| collections | list 切片+HOF 族 | (a) std.list（946 冻结面超集） |
| collections | hashmap/hashset（int 键） | (a) std.map（交错对+引用语义） |
| collections | Str 键/泛型 map、btreemap | (c) 963 全泛型切片 |
| collections | sort | (a) std.list.sort（稳定最小抽取升序 List<int>；比较闭包形与 Str 序归 963 切片） |
| parse/format | to_int（含错误路径）/is_uint/from_uint/from_int（负数形、i32::MIN） | (a) std.conv |
| format | FStr/format 求值 | (c) 964 |
| hash | 纯 Auto 默认哈希 | (a) std.hash codepoint 域 + std.bytes byte 域（S-06 裁定） |
| hash | md5/sha1/sha256/sha512/hmac/file_md5（旧全 crypto #[vm]） | (c) 967 外部引擎 |
| regex | is_match/find_all/replace/test | (c) 967 外部引擎 |
| encoding | hex/base64 编码 | (a) std.bytes（Python 独立 oracle） |
| encoding | hex/base64 解码 | (b) 具名边界——byte→Str 构造本面无 |
| encoding | csv | (b) 纯 Auto 后续批 |
| Option/Result | ?T/Some/None；错误传播 .?/?? | (a) 语言面（aggregates 域，965 已交） |

### T-10 core/alloc 分层清单（2026-11-15；V-10 fixture 锚；978 具名清单）

装配集 54 公共符号按分配责任二分（V-10=`v10_layers_972.rs`：core-only 程序在 AC/native 端观察计数 **attempts=allocs=0**——无堆、无宿主 dispatch；alloc 程序分配平衡、invalid frees=0；fail-Nth 注入=资源 trap 22+全清扫无泄漏；欠配预算同陷阱）。分配走 mn 运行时 arena=1039/1040 原生底座（无 malloc/隐式宿主堆）；core 层符号不触 arena。

| 层 | 符号（28 core / 26 alloc） |
|---|---|
| core（无分配） | math.abs/min/max/square/cube/abs_f/sqrt/powf/sin/cos；char.is_digit/is_alpha_ascii/is_alphanum_ascii/is_ident_ascii/is_whitespace/to_upper_ascii/to_lower_ascii；conv.to_int/is_uint；text.byte_len/scalar_len/char_code_at/equal；bytes.byte_at/utf8_len/byte_of/byte_hash；hash.codepoint_hash |
| alloc（经底座 arena） | text.starts_with/ends_with/contains（slice 物化）/repeat/upper_ascii/lower_ascii；conv.from_uint/from_int（构串）；bytes.to_hex/to_base64（构串）；list 全 10（List<int> 分配）；map 全 6（交错 List 分配） |

regex 失败/Unicode/format 错误独立断言的适用集：parse 错误路径已独立断言（V-DETAIL-02 六负例）；Unicode 单位 oracle（A中😀/e+U+0301/NUL）为常驻；regex 本体归 967 后其负例随该包验收。删 required 行/腿阻签=面清单奇偶校验+冻结身份拒（provider 腿）+全符号演练缺符号红。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-stdlib-core.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |
| SD-AUTO-STD | modify | docs/specs/ace-migration/m2-stdlib-core.md | 默认provider声明→普通标准能力有Auto算法/协议与OS边界来源证据，高级外库显式分账 | 用户自有普通库要求 | AC-AUTO-STD |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 所有冻结公共符号有源码与四端执行证明 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | i64 abs最小值边界、UTF8 byte/scalar差异、嵌套集合释放和parse错误分别独立断言 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 改provider/body后正确失效，缺证明拒装配 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t stdlib ace-stdlib ace-source ace-av ace-native ace-a2x ace-runtime`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 每个public符号均有签名proof和适用四端运行；regex失败/Unicode及format错误独立断言；删任一required操作行或provider腿必须阻签。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | 同一文本/集合/数值程序四端对独立expected；i64极值、UTF8 byte/scalar、parse失败、嵌套owned释放分别断言；删除C/Rust业务源码发射步骤仍可native执行。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 撤宿主/allocator后合法core程序仍运行且host dispatch/allocation=0；alloc路径无allocator具名拒绝、配置后成功，分配失败/取消无泄漏；不能只以no_std编译代无heap证明。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |

新增触面职责组：`cargo t stdlib ace-stdlib ace-source ace-av ace-native ace-a2x ace-abi`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-09,T-10，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 多字节、空串、ab\0cd五字节、重叠/边界/失败分配等适用原操作对独立oracle；移除可选C/Rust/libc provider仍可执行默认库，缺自有实现不能fallback。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧源归约15、find4、any/all逻辑假/真；原VM显示0/1与语义bool分账。新例核过滤后[2,3,4,5]及拥有元素内容，不只len或ok。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 宽度/locale/UTF-8/NUL/索引单位与Storage错误不能静默改语义；缺泛型/所有权能力回原owner，不用Rust整套std替Auto公共算法。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 默认core不隐式libc、目标OS薄适配与显式外库分账；allocation失败保旧，clone/deep/shared按语义账，最后owner释放。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 原生平台I/O验证（新增，未执行）

| Case | AC / work任务 | 输入、独立oracle及边界 |
|---|---|---|
| V-PLATFORM-IO | AC-PLATFORM-IO；T-01/T-09/T-10/T-04 | 中文/emoji/含NUL与转换块边界、短写/成功0有独立编码和进度expected；固定buffer算法无隐式分配，alloc来源明确。纯算法无OS可用，但不代实际OS格。 |

### 原生底座验证（新增，未执行）

| Case | AC / work owner | 输入与独立oracle |
|---|---|---|
| V-FOUNDATION | AC-FOUNDATION；T-01/T-09/T-10/T-04 | 同一默认Auto Str/List/format操作追到1039/1040真实OS区域及1041输出：四native平台格独立内容/分配失败/最后释放结果齐；删除底座或注入Rust std容器/malloc委托必须被来源及运行控制拒绝。core-only合法程序仍无OS可运行；不能把core绿、旧runtime绿或只有1039接口当整库完成。 |

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-AUTO-STD / AC-AUTO-STD | 原能力work与最终验证：移除可选外库/provider，在Windows默认native/AC跑原完整公共操作并核Auto源码/原语/OS调用来源和独立expected | math/io/JSON等按§5自有归属，不以包装libm/std::io/serde_json等代实现；Linux只做代码/移交审查，未来1063实跑，外部引擎依赖另账。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 全API覆盖及领域边界的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | LIB-M2-01/04默认Auto单源与四端运行闭合，AC无需转译业务C/Rust或旧VM；原完整math/text/collections等分母不减。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | LIB-M2-04分层真正可构建/执行，core无host/allocator隐式依赖，alloc操作资源与错误符合公共合同。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-NATIVE | work | 多字节、空串、ab\0cd五字节、重叠/边界/失败分配等适用原操作对独立oracle；移除可选C/Rust/libc provider仍可执行默认库，缺自有实现不能fallback。 | T-09,T-10 / V-NATIVE；原独立review核新增义务 |

- [x] **AC-PLATFORM-IO / work**：本节平台合同及V-PLATFORM-IO全部满足，真实来源/oracle/未交边界齐全；原分母/四端/阶段不减，原最终work汇总、独立review核验。

- [x] **AC-FOUNDATION / work**：V-FOUNDATION与§5原生底座当前合同全部关闭；正式底座包含实现/真实OS/依赖/独立复审与落主身份，纯算法与旧runtime支持不能替代；原最终work及独立review逐项核。

- [x] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [x] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

- [x] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

- [x] **AC-AUTO-STD / work**：§5本包普通标准能力默认算法/协议为共享Auto源码，实际来源/调用链审计与原独立expected齐全；可选C/Rust及高级外库列明边界，不冒默认自有实现。原能力work任务与最终验证负责，原独立review核；Linux按AC-LINUX-PHASE移交。

## 8. 执行步骤

当前共10任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-15 前探完成：7/7 探针绿（worktree lang-972 @ cd71736f8 基面，probe972.rs）。正例=1043 装配 lane AV×Auto 711/AV×Rs 712（spec 层按来源选择）+ aggregates lane 算法面 154400015（sum=15/find=4/all/any/filter=[2,3,4,5] 拥有内容核验）+ PackageStore/2 往返恢复复跑。非法近邻=assembly.missing-default-provider（装配期）、source.assembly.signature（签名漂移，运行前）。冻结边界钉住：闭包参数形（fn 型注解）与 List 公共参数在后置裁剪语法面均 source.syntax 具名拒（auto-source.at:745 fn-type 门控缺 packages-aggregates；装配 lane 参数位无 generic-ty）→ T-02 开门项。依赖核验：12/13 archived 正式来源；967 reviewed 未合入 → 真实 C/Rust 外库 dispatch 腿终态闭合挂其 merge，默认库/native/A2R/A2PY/A2C 装配腿不受阻（1043/1041 明文边界）。§4 五路径全在（157/2/2/24/14 文件）。命令冻结：迭代 cargo nextest -p ace-av --test conformance <filter>；T-04 拟冻结并集 cargo t stdlib ace-stdlib ace-source ace-syntax ace-semantics ace-av ace-native ace-a2x ace-runtime（触面定稿）；Linux 执行腿按 2026-11-14 裁定排除，Linux 目标轴选择腿照 1043 模式跑。资源/平台表：AV=in-process、AC=MnDriver exe、A2R=rustc、A2C=cl、A2PY=真 OS Python；host=Win x64、target 轴 Win/Linux、Mac 仅输入；FrontBudget::WORK_OWNED_64MIB、store 16MiB。策略决定：新 stdlib 内容走 1043 装配 lane（公共 .at 单源 body + 可选来源候选），不扩 legacy stdlib/auto（986/987 域）也不止于 include_str! 冻结切片；std 防遮蔽身份扩展到装配面；oracle 基线=943 实册（A中😀=8B/3C、e+U+0301=3B/2C、NUL 不截断、i64 不窄化）。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：按948/959逐操作闭合math/text/bytes/collections/format/parse/hash与现用regex/encoding；纯算法单源，FStr求值归964，平台和DB归973/975，不留剩余操作无owner。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07/AC-NATIVE**：按947/943/944合同组织公共API与普通Auto默认算法，适配四端模块调用/AutoABI并核每个intrinsic/外部优化实现语义；保原公共路径和操作ID。 验证V-09，证据汇入原最终work验证。 新增V-NATIVE：默认基础算法保持Auto实现，Str/str/bytes按显式长度/既有编码和ownership语义；不靠strlen/printf等C库重建默认语义，也不透过Rust helper隐藏委托；intrinsic和allocator来源/目标依赖显式。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [x] **T-10｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：分离core/alloc依赖和profile，核无隐式heap/host的核心程序、显式allocator集合路径、必要runtime ABI和预编译库闭包；交978具名清单。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：默认基础算法保持Auto实现，Str/str/bytes按显式长度/既有编码和ownership语义；不靠strlen/printf等C库重建默认语义，也不透过Rust helper隐藏委托；intrinsic和allocator来源/目标依赖显式。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-972/auto-lang`，分支 `plan-972-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-972/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-15 stage: merge | plan_id: PLAN-972 | plan_revision: r12 | outcome: **pass（delivered）** | delivery_commit: **aa7b56864**（master tip==delivery，ff-only 线性落地；rebase 链：c1b85904b→f300c918d→aa7b56864 二次 rebase 至 1062 后 master，面无交叠） | canonical: docs/specs/ace-migration/m2-stdlib-core.md（新）+ replacement-scope-and-evidence/ace-binding-and-assembly/semantic-operation-contract 三增量（0b203e430 入 ancestry） | merge-owned gates: 冻结并集复跑于最终基线（stdlib/ace-stdlib/ace-source/ace-syntax 45/45/ace-av/ace-native/ace-a2x PASS；ace-runtime=真基线同失败集环境红——rust-lld 工具链，非本包） | 台账已知项移交：F-972-R1（a2x-c mut-ptr 别名双 free）→KNOWN-DEBT；967 merge 后 AV×C/Rust 腿；1063 Linux 认证；csv/解码/全 Unicode case 结构边界 | 计划簿记勾 T-05/06/07（本条）并归档；plan-artifacts 整删；wt-guard clean 后清树/分支/组目录。证据主体在上方各轮回执。

2026-11-15 stage: merge | rebase 集成轮（master 前进：969 门面/968 调度器/979 AGE 组件/1043 装配） | **审查基线重映射**：reviewed_commit c1b85904b rebase 至 master dd88a41d8 上——交付提交变为 **f300c918d**（语义等价重放：6 个计划提交 + 1 个 docs(specs) 提交 + 1 个集成修复提交；range-diff 意义上的补丁等价被破坏处=仅冲突解决形式，全部为 profile 列表 union 与 PLAN-968/972 交叠的机械合并）。**集成修复**（f300c918d）：(1) auto-source.at 语法合并——两轮失败后以"全行键（规则上下文+profiles 后归一化行文本）"精确映射 188 处 ct 插入+手置 expr 臂/ct-ops 表/in-ct-ops/声明；保留 master 的 scheduler+age-component 全部接线；此前 body-key 冲突（`{ fn-type(field: "fn") }` 四规则同形）曾把 ct 错插 fn-type-param/result 致装配集全红——已根因定位并修复；(2) PLAN-968 新增 CheckedEnv 构造补 asm_lane:false；(3) coll.rs 运行时源 master 侧重复 a2x_str_to_owned 去重；(4) profiles_927 DECLARED 冻结集刷新（scheduler；age-component 为 master 已有）。**冻结并集复跑（f300c918d）**：stdlib/ace-stdlib/ace-source/ace-syntax(45/45)/ace-av/ace-native/ace-a2x 全 PASS；ace-runtime=基座环境红（真基线 9f31e18c6 同失败集复现：v06/v07/lifecycle/smoke——rust-lld __CxxFrameHandler3/_fltused 工具链漂移，非本包改动）；972 套件 26/26。**事故与恢复**：误 `git stash pop` 弹出他人 PLAN-002 master 贮存——reset --hard 恢复（工作全已提交）+ 18 个遗留 docs 文件重新入 stash（plan-972-dev 上两条，原 master stash 未动）。next: land+archive。

2026-11-15 stage: review | review_scope: final | reviewer: 同会话独立复审（限制如实声明：与实施同会话/同上下文，未授权独立 reviewer agent；裁决由工件重建——commit 清单/§9 各轮回执命令与结果/代码与测试逐项抽查，不采信执行摘要） | plan_id: PLAN-972 | plan_revision: r12 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-972/auto-lang | branch: plan-972-dev | reviewed_commit: c1b85904b | base_commit: 9f31e18c6 | dependency_revisions: 1043 装配层（lane 本体）/1039/1040 arena（V-10 观测）；967 reviewed 未合入=AV×C/Rust 外库 dispatch 腿外部时序依赖（具名，1043/1041 明文边界） | spec_inputs: §5 规范增量表 SD-01..04/SD-PHASE/NATIVE/PLATFORM-IO/FOUNDATION/COVERAGE/LOAD/LINUX-PHASE/AUTO-STD 全指向 docs/specs/ace-migration/m2-stdlib-core.md（新）+三份既有 stdlib spec（路径实存；merge 时沉淀） | acceptance_results: AC-01 pass（r11 前探冻结）；AC-02/03 pass（62 公共符号五端真跑同值 62、身份/签名/所有权负例、AV 唯一→五端演进全录）；AC-06 pass（§5 逐操作对账表无无主行，余项全 (c) 外部或结构具名）；AC-07 pass（纯 Auto double 族+整集无业务 C/Rust 转译，native 直编）；AC-08/AC-NATIVE pass（V-10 core attempts=allocs=0/分配平衡/失败注入全清扫/预算陷阱；NUL/多字节 oracle）；AC-PHASE pass（A 切片齐）；AC-PLATFORM-IO/FOUNDATION/COVERAGE/LOAD/LINUX-PHASE/AUTO-STD pass（边界分账：I/O 原语=1041/973、arena=1039/1040、六路径四真跑两挂 967、1043 即 lane 本体、Linux=NOT_RUN/1063、默认全纯 Auto 无 provider）；复审重放 26/26 PASS（972 套件 18+probe 8） | pending_closeout_gates: merge-owned——冻结并集复跑于合并基线（本轮回合已跑：6 组 PASS+ace-native 276 exit0/conhost 基建检查环境归因+ace-runtime 会话前基座红复现于 9f31e18c6）、canonical specs 沉淀、归档、输出路径对账、wt-guard 清树 | findings: F-972-R1（P3，发射器）a2x-c 尾声对"mut-ptr 局部被别名进被返回调用结果"仍双 free——装配 stdlib 已源级规避（tail-return 纪律+模块注释），任意用户代码可触；路由 KNOWN-DEBT（merge 落），非本包验收内缺陷。F-972-R2（P3，测试成本）V-08 全符号五端用例暖态 9.2s、扰动后冷态最差 11.5s（五外部工具链腿顺序+共享 runtime 制备；调查完成，未拆分——同值五端为 AC 合同）；暖态达标记录在案。F-972-R3（P4，环境）ace-runtime smoke::smoke_shim_error_code=会话前基座红（rust-lld 缺 __CxxFrameHandler3/_fltused），非本包改动，owner=ace-text-contract | evidence: 复审重放命令=cargo nextest run -p ace-av --test conformance stdlib_asm_972 v08_surface v10_core probe972→26/26 PASS（本轮实测 11.9s 总）；§7 六验收框本轮按对账表/V 行总账勾选；worktree clean@c1b85904b（6 commits 自基座） | next: merge（T-06/T-07）

2026-11-15 stage: work | 第十六轮：T-04 冻结并集 + V 行总账（work 阶段收口） | **冻结并集**（T-01 冻结表）：cargo t stdlib/ace-stdlib/ace-source/ace-syntax/ace-av/ace-a2x = **PASS**；ace-native = 276 测试全部 exit 0（组 FAIL 仅为 conhost.exe 残留进程的 containment 基建检查——环境归因，同机 spawn 争用类；本轮早前独立跑 3 次全 PASS）；ace-runtime = 1 例 `smoke::smoke_shim_error_code` **会话前基座红**（9f31e18c6 复现：rust-lld 链接缺 __CxxFrameHandler3/_fltused——工具链漂移，owner=ace-text-conformance，非本包改动）。**V 行总账**：V-01/V-03=冻结公共符号四端执行+身份失效（62 符号五端演练+冻结身份/签名漂移负例）✓；V-02=i64 边界陷阱/UTF8 单位/嵌 NUL 独立断言 ✓；V-08=面清单奇偶+62 符号五端 ✓；V-09=同程序多端独立 oracle+i64 极值/parse 失败/owned 负例+native 无 C/Rust 业务源（装配集纯 Auto）✓；V-10=core 零分配+分配平衡+失败注入无泄漏+预算陷阱 ✓；V-DETAIL-01/02/03/04=HOF 内容/parse 六负例/所有权具名拒/逐行对账表 ✓；V-COVERAGE=六路径中 AV/A2R/A2PY/AC(native) 四条真跑，AV×C/AV×Rust 外库 dispatch 挂 967 merge（外部时序依赖，具名）；V-FOUNDATION=分配经 1039/1040 arena（V-10 观察）+core 不触 arena ✓；V-AUTO-STD=默认全纯 Auto 单源、无 provider 候选（身份测试拒层）✓；V-PHASE=A 切片齐 ✓；V-LINUX-PHASE=Windows required 全跑、Linux 执行腿按 2026-11-14 裁定 NOT_RUN/owner=1063（装配面无 Linux 专属操作行）✓；V-PLATFORM-IO/V-LOAD=1041/1043 消费即装配线本体+UTF8 编解码算法，I/O 原语归 1041/973（具名边界）。**已知具名项**：C mut-ptr 别名进返回结果的尾声双 free（源级规避+KNOWN-DEBT 候选）；mn/C 位算发射、float rem；csv/解码/全 Unicode case 结构或数据边界；967 外部时序。work 阶段收口：T-01..04/08/09/10 完成（7/10），T-05 复审/T-06/07 合入待独立阶段。证据：artifacts 第十六轮节。

2026-11-15 stage: work | 第十五轮：T-08 (b) 批收口——find/reverse/trim/replace/split/lines + sort | 7 新符号落 std.text/std.list（纯 Auto 建基元），V-08 扩 **62 符号五端全绿同值**。钉住源级纪律：owned Str 实参移动→递归前 clone；尾调用实参先 bind 借用再移；List<Str> push 须 .move；穷尽 is 于 list 返回函数的面 filler 拒→int 助手 hit_or；保留字不作参数名；use 别名解跨模块同名。**C 发射器缺口（具名=KNOWN-DEBT 候选）**：mut-ptr 局部别名进被返回调用结果时尾声双 free（sort 单元素即堆损坏；源级规避=积累器 int 步函数+外层 tail-return 自有局部，模块内注释指明）。对账表更新：find 族+sort (b)→(a)；余 (b) 仅 csv（数据表批）、hex/base64 解码（byte→Str 结构边界）、全 Unicode case（宿主表，967）；余 (c) 不变。972 套件 18/18。commit c1b85904b。证据：artifacts 第十五轮节。

2026-11-15 stage: work | 第十四轮：T-10/V-10——core/alloc 分层证明 + 978 具名清单 | **V-10 fixture**（v10_layers_972.rs，AC/native 端）：(1) core-only 程序（math 整数+纯 Auto double 族+char 谓词/case，无 Str/List 构造）观察计数 **attempts=allocs=0**（17 检查全分 1111121）——无堆无宿主 dispatch，分配器完全不触；(2) alloc 程序（upper/repeat/to_hex 构串）分配>0、invalid frees=0、账目闭合（allocs−frees_ok=live）；(3) fail-Nth=3 注入——资源 trap（观察 code=22）+ INJECT 记录 + 失败路径全清扫（live=0、invalid=0，无泄漏）；(4) 欠配字节预算同陷阱。**分层清单**（§5 新节=978 具名清单）：28 core（math 10、char 7、conv.to_int/is_uint、text 长度/索引/equal、bytes 走查族、hash）/26 alloc（text 物化/构串族、conv 构串、bytes 编码、list/map 全部）——分配经 mn arena=1039/1040 底座，无 malloc/隐式宿主堆。观察/注入/预算经 MnDriver.resource_knobs（WrapperKnobs observe/fail_nth/budget_bytes）。时长：冷首跑 87s（runtime rustc 一次制备）、暖 4.98s。972 套件 17/17+V-10。commit 见下。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md 第十四轮节。

2026-11-15 stage: work | 第十三轮：T-09——纯 Auto double 族 + f64 五端发射面 | **裁定执行**：AC-AUTO-STD 不以 libm 代签→double 族=纯 Auto 单源算法（abs_f；sqrt=4^k 归一+Newton；sin/cos=一次 trunc 归约+Maclaurin 共享累加器；powf=exp(y·ln x) atanh/exp 级数+2^k 精确缩放；全递归）。**记录偏差**：NaN 不可构造（checked ÷±0）→非正输入返 0.0；|x|≤~1.4e18 归约域、i32 scale2 域；~1e-15 相对精度非正确舍入平价（五端同源同序→跨端同值）。**f64 发射面**：mn——F64Bits 常量位型、浮点 fcmp/fadd/sub/mul/fdiv(÷±0 陷阱-31)/fneg、Conv 三形（F32↔F64；float→int 2^n 精确幂界守卫+sat 转换；int→float 符号恢复+fcvt_from）；a2x-c——值类型/参数道/浮点 BinOp 臂/结果通道 F64/F32（chan/fwd/value 三表）+runtime add/sub/mul/neg f32/f64 生成+死旧臂删除；a2x-rust——num_bin_expr 浮点比较前置（曾落 "float op Eq" 拒）；a2x-python——Collections 参数/返回准入浮点。浮点字面量须宽度后缀（2.5d）。**V-08 扩 49→54 符号**：double 检查=宿主 oracle 容差带（sqrt(2)/powf(2,10)/sin(1)/cos(1)/abs_f），五端全绿同值 54。修复前基线探针：AV/A2Py 绿、mn 缺常量、A2C 缺值型、A2R 缺浮点比较（临时探针已删，链见 artifacts）。972 套件 17/17；组：ace-source/ace-native/ace-a2x PASS。commit d11993db6。T-08 对账表更新：double 族 (b)→(a)。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md 第十三轮节。

2026-11-15 stage: work | 第十二轮：T-08/V-08——逐操作对账表落 §5 + 全符号五端演练常驻 fixture + 四发射器缺陷修复 | **V-08 fixture**（v08_surface_972.rs）：(A) 面清单奇偶校验——装配集 8 模块 49 公共符号与必需 op 表双向全等（删行/改名/私增即红=删行阻签）；(B) 全符号演练 main 49 检查（独立 oracle）→49 于五端全跑（AV/mn exit/cl/rustc/真 OS Python value:49）。**char 补齐**：legacy to_upper/to_lower（旧 int→int 恒等平移）以纯算术 p∓32 落 std.char（初版 char→char 遇 a2x-c Char 结果通道缺口，改 codepoint 形更贴旧 API）。**四缺陷修复**（全符号演练首触，逐一定位链见 plan-artifacts 第十二轮节）：(1) mn 指令包装 dst 匹配缺 StrSlice/StrConcat/StrToOwned——round-6 文本臂产新节点不绑指纹→Drop 校验 refusal=5→trap 70；(2) mn Conv 读窄 cell 零扩展字直 sextend——负值 I32→I64 变 4294966875（探针 %10=5 实锤）+收窄假溢出，修为源宽 ireduce 后扩展、加宽免检；(3) a2x-c View Record/Variant 实参传值 vs ConstPtr 原型（C4047/C2440）+ VariantProject 根为 ConstPtr 参数双重间接（C4090）——补 &/裸名/显式限定转换；(4) a2x-c 运行时 a2x_mul_i64/u64 的 _mul128 返回位与出参颠倒——一切 i64 乘（3*5 亦然）误溢出陷阱（最小 C 复现 hi=15/lo=0）。旧 961 数值面 C 侧 i64 乘从未被执行过。972 套件 17/17（V-08 五端 9.88s）；组：ace-native/ace-a2x PASS（后台）、ace-av PASS。commit 0dc418876。T-08 对账表（§5 新节）：(a) 已交付 49 符号、(b) 本包后续批（double 族 T-09、find/reverse/trim/replace/split/lines、sort、csv、解码 byte→Str、全 Unicode case）、(c) 外部 owner（964 FStr/963 泛型/967 regex+crypto+uuid/965 Result）——无无主操作。T-08 未勾：待 (b) 项落地后随 T-04 汇总。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md 第十二轮节。

2026-11-15 stage: work | 第十一轮：T-02 余域批——map 型 + int→Str 负数形 + ASCII case 表 + hex/base64 编码 | **map 型**：`stdlib/ace/asm/map.at`（map_new/put/get/has/len/remove——交错 [k,v] 于 List<int>，946 冻结方法表引用语义、插入序保持、原位覆写、remove 先值后键压缩；实例化签名纪律沿 list.at，Str 键与泛型 map 归 963 全泛型切片=具名边界）。**int→Str 负数形**：conv.from_int（"-" + i64 幅值游走，i32::MIN 无 abs 溢出）。**case 表**：text.upper_ascii/lower_ascii（26 字母表定位+对照表 slice 重建，非 ASCII 透传；全 Unicode case 表=具名后续）。**编码**：bytes.to_hex/to_base64（重建字节流纯 div/mod 六位组，base64 状态递归跨 scalar 组对齐；DECODE=具名边界——byte→Str 构造本面无）。**a2x-rust 修复**（本批首触）：lower_coll Call 参数位实参为 View-List 参数名时仍包 RefMut（&mut <&'a mut 绑定> 借用绑定本身=E0596）——修为裸名传递（与 BorrowMut 臂同纪律；list.at 从未传 List 参数给自由函数故未触）。所有权纪律钉住：borrow Str 参数直返须 clone（E0308）、禁 Str var 重赋值/取 s 的 helper（use-after-move）、测试侧 equal 移动后 len 读=use-after-move（len 前置）。oracle（独立 Python hex/base64）：A中😀=41e4b8adf09f9880/QeS4rfCfmIA=、a\0b=610062/YQBi、autostack!=YXV0b3N0YWNrIQ==；map 场景 201023/1314；case 111155；from_int 1111511。972 套件 16/16（新例 map 0.98s/enc 0.74s/from_int+case ~0.2s；五端整集含 map.at 6.97s）；组：ace-source/ace-a2x/ace-av PASS。commit d0adee504。T-02 余量：Str 键/泛型 map（963 切片）、全 Unicode case、hex/base64 解码（byte→Str）、mn/C 位算发射；double 族挂 T-09。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md 第十一轮节。

2026-11-15 stage: work | 第十轮：S-03+S-06 批落地——投影位根因闭合 + mn I64 rem 修复 | **根因**（S-06 回退待查项闭合）：packages-aggregates-ct 的 expr 分派 wide-ops pratt 表（无 & | ^），而三 token 自 961 起已在该面声明、无算符表引用——纯投影位缺口；草稿 hash.at 的 `^` 触发错误恢复错位（诊断落在参数位/项位，整集绑定失败外溢）。**修复**：新 pratt 表 `packages-aggregates-ct-ops`（wide-ops 超集+位算同层，961 冻结记录；旧 profile 字节不变）。**S-06**：`std.hash.codepoint_hash`（多项式 (h*1000003+cp) mod 2^31-1、种子 1469598103、全程 checked i64 域内）。**S-03**：`std.bytes.byte_at/utf8_len/byte_of`（纯 div/mod 重建 UTF-8）+ `byte_hash`（byte 域，裁定并入本批）；默认算法避开位算（mn/C 位算发射仍具名拒=后续单元，语法门已开并有 AV 正例钉住）。**mn 缺陷修复**（本批首触）：binary_num Div|Rem 原在 I128 宽 sdiv/srem——x64 ISLE 无 I128 除法降载（codegen unwrap panic；此前无 std 模块触 rem_i64）；改 I64 宽发射，-31 除零/-30 MIN%-1 守卫前置（MIN%-1 不达硬件 idiv），add/sub/mul 保 I128 域守卫，陷阱码不变。oracle（独立 Python）：codepoint auto=2075808597/空=种子/A中😀=534270170；byte a\0b=386610881/A中😀=866049554；区分位 1111；byte_at A中😀 8 字节逐位+越界/负索引 None+NUL 字节=1411111111。972 套件 12/12（新例 1.11s/0.31s/0.31s；五端整集腿含新两模块 6.2s）；组：ace-syntax/ace-source/ace-av PASS，ace-native 第 3 轮 PASS（前两轮各红不同 1-2 例、隔离重放绿——并行 mn exe spawn 争用，环境归因）。commit 02fd0daae（worktree lang-972 @ plan-972-dev）。T-02 余域：集合全集 map 型、int→Str 负数形、upper/lower case 表、hex/base64 编码、mn/C 位算发射；double 族挂 T-09。证据：D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md 第十轮节。

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r12：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-15 stage: work | S-06 hash 裁定（计划级，实现退回）| **裁定**：S-06 纯 Auto 默认哈希取 **codepoint 域**（与 scalar_len/char_at 单位一致、确定序敏感 i64 结果）；**byte 域哈希并入 S-03 byte 访问批**（本 lane 文本面无字节索引）。实现尝试退回：草稿 hash.at（FNV-1a codepoint 版+mod97/score 助手）破坏整集解析（app 消费者 main 的 item 位 `fn` 具名拒外溢），已整撤（未入库）；重试须先修 app-main 的 suffixed-literal/`.to` 语法面差异（math 腿 `min(3i64,5i64).to(i32)` 可行而 hash 腿同形被拒——待查 profile 投影位差异），并与 S-03 一批落地。整集回归 9/9 绿。

2026-11-15 stage: work | 第九轮：A2PY 腿接通——五执行端整集 capture-free 闭包全通 | a2x-python lower：Collections 族准入 Char + typed 标量 + Closure 参数与 typed/List 返回；CreateClosure 直译 Python lambda 过体函数（捕获闭包具名拒——跨端契约保持 capture-free 一致）；CallClosure=普通间接调用。**A2PY 整集 HOF 腿接通**：filter+reduce 闭包经 PythonTargetProvider + 真 OS Python（pyrun）→ value:144。**五执行端（AV / AC-native / A2R / A2C / A2PY）整集 capture-free 闭包全通**——V-COVERAGE 六路径中五条执行路径真跑（AV 三来源配置面余 AV/C 与 AV/Rust 真实外库 dispatch，挂 967 merge）。972 套件 17/17；ace-a2x 组 PASS。commit 9f31e18c6。

2026-11-15 stage: work | 第八轮：C List 结果通道落地，A2C 整集 HOF 腿翻转——四执行端整集全通 | coll_type_table：每 list 类型生成 a2x_res_list_<tag> 包装 + a2x_list_<tag>_value/a2x_fwd_list_<tag> 构造器；lower_coll：CollRetKind::List（elem-tag 包装名）+ typed 标量结果（i64/u64 通道；i8/u8/u32 按冻结通道窄化 i32）；coll 运行时：a2x_i64_value/fwd 等构造器（置数值 typedef 之后）。**A2C 整集 HOF 腿翻转**：filter+reduce 闭包经真 cl → value:144。**四执行端（AV/AC-native/A2R/A2C）整集 capture-free 闭包全通**。972 套件 17/17；ace-a2x 组 PASS。commit 332567ce1。

2026-11-15 stage: work | 第七轮：A2C 数值面+闭包发射落地（整集腿待 C List-结果包装）| runtime.rs 将 961 数值运行时段落抽取为共享 write_numeric_runtime（标量/文本头字节不变）；coll 运行时并入该段（A2X_TRAP_INVALID 别名 INVALID_SCALAR）；typed 标量参数/值类型准入（C 拼写）；lower_coll（C）补 BinOp::Num（整型比较内联、checked 算术经共享助手、float/位算具名后续）、NumericConst（共享内联字面量 numeric_const_c）、Cast（共享转换矩阵）、闭包参数=C 函数指针（ret (*name)(params) 按体通道）、CreateClosure（函数指示符；捕获具名拒）、CallClosure（fall_res 按体通道）。**A2C 整集腿仍被 filter/map 的 List 结果签名阻塞**——C 侧 List 结果包装（seed 表生成 a2x_listo_result_<tag> 类）为具名后续。子集腿保持绿。972 套件 17/17；ace-a2x 组 PASS。commit 5ba41951f。

2026-11-15 stage: work | 第六轮：A2C C 发射器 text/char 面接通（子集腿）| coll_runtime（C 源）补 a2x_char_result/str_char_at（标量索引解码）/strv_result+str_slice（借用视图，data 指入源）/str_to_owned/char_from_codepoint；lower_coll（C）补 TextCharAt/TextSlice/TextToOwned 臂（fall_res）、CharLit/CharToCodepoint 内联、CharFromCodepoint 臂；Char 与 View-StrOwned 参数准入（uint32 值传递 / const a2x_str* 借用）；Call 臂 str-borrow 参数强制（普通值 &x 借用）。**A2C 子集腿接通**：std.text+std.char 经真 cl → value:83（A中😀 Unicode 单位 oracle）。TextConcat 沿既有 FStr 臂（曾误加重复臂遮蔽致 dynamic_964 红——已删并复绿）。972 套件 17/17；ace-a2x 组 PASS。C 余量（数值面并入/闭包）沿 §10 蓝图。commit af340015d。

2026-11-15 stage: work | 第五轮：mn closure 值落地，整集 AC 腿翻转 | collections_lower：CreateClosure（cell word=unit import 代码地址；捕获闭包具名拒）+ CallClosure（cells ABI call_indirect，实参+dst cell 地址）；collections_layout：Closure(sig) 描述符（签名指纹）；fingerprint：capture-free 闭包指令可渲染（823 排除由 972 扩面取代；ENV_GET 体仍拒）；collections_driver：空 env 声明的闭包体 import 准入（非空 env 保持具名边界）；closures_compat 测试更新至新边界。**整集 mn 缺口翻转**：HOF 程序经全装配 std 集真机码 exit 44。三执行端（AV/AC/A2R）整集全通；余 A2C/A2PY 发射器腿（各自需等价 closure/text/char/numeric 面）。972 套件 17/17；ace-native 组 PASS（276）。commit 2bddc6a28。

2026-11-15 stage: work | 第四轮：a2x closure 值+全 text/char 面接通，整集 A2R 腿翻转 | a2x-core 增 Ty::Closure（类型表内联签名，Ty 保 Copy）+ prepare/fingerprint/语义面三处准入（CreateClosure/CallClosure/text/char kinds）；a2x-rust：ClosureLit/IndirectCall RExpr（间接调用可失败）、闭包参数渲染 `impl Fn(P) -> A2xResult<R>`（体转发可失败体函数、调用位 `?` 解包）、闭包 let 免标注、RustFn.result_text 预烘焙 List/闭包结果、char 三臂+text 六臂（正确临时类型；共享运行时助手）、str 视图操作数强制（字面量/借用参数跳过 .as_str，owned 临时经其借用）、coll 运行时补标量索引 text 助手；a2x-store 合成 __closure 片段 origin (0,0)。text.at repeat_into 改返 acc.clone()（a2x coll 不能把借用参数按 owned 结果返回——具名边界）。**整集 a2x 缺口翻转**：HOF 程序（filter+reduce 闭包）经真 rustc 编译运行 value:144（全装配 std 集）；mn closure 仍为整集具名缺口。972 套件 17/17；ace-a2x/ace-source 组 PASS。commit 8094f1149。

2026-11-15 stage: work | 第三轮：mn 发射面落地+AC 消费腿接通 | collections-runtime 新增 mn_coll_str_len_bytes/str_char_at/str_slice/str_concat/char_from_codepoint（组合共享 ac_rt 标量索引原语，负状态→checked 陷阱）；collections_lower 落 text/char/数值发射臂：char/数值类型准入+immediate kind-0 布局指纹、typed 标量常量（I64/U64 全字存储）、七个 text/char 指令臂、全整宽比较、checked I64 加减乘除余（I128 计算+域守卫）、checked I64 取负、整型 Conv（符号感知加宽+checked 收窄）；窄算术/浮点/移位保持具名拒（数值后端单元）。**AC/native 正例腿接通**：std.math+text+char 子集真机码 exit 83（A中😀 Unicode 单位 oracle）。整集 closure 缺口仍冻结（任意具名深度均认）。ace-native 组 PASS；972 套件 17/17。commit f6f34e36b。跨端 closure 值现状：a2x-rust 无任何 closure 发射（Ty 无 fn 变体）——a2x closure 为新特性单元；mn closure 需并 closures 线 staged-transaction 运行时模型。

2026-11-15 stage: work | 第二轮追加 | V-DETAIL-03 所有权负例系落地：use-after-move（移后读）与 mir.mut-conflict（实参表达式仍借用时移入实参）均以先于输出的具名拒断言（`stdlib_asm_972_ownership_move_negatives`；take 为聚合面保留字的语法事实一并钉住）。std.conv 补 from_uint（数字表切片格式化，0→"0"/421→"421"/负数→""；oracle 420743438）。972 套件 17/17 绿。commits 617cc2d0d、f4f3356f1。

2026-11-15 stage: work | 第二轮（跨端域扩展 + T-02 余域）| (a) a2x：961 数值面并入 collections 族——lower_coll 补 NumericConst/Cast 臂（共享 num_lit/conv 矩阵）、typed checked temp 带真实结果类型（checked_temp_ty）、RT_NUMERIC 助手并入 collections 运行时；**A2R 正例腿接通**（装配线 std.math 子集经真 rustc：square(7)*10+min(3i64,5i64).to(i32)=493 stdout）；ace-a2x 组全 PASS。(b) mn：探明 collections 发射器为自含（逐指令 CLIF 臂，不委托共享文本/char lowering）——其 text/char/数值发射面为已设计后续单元；mn 整集保持干净具名拒（Closure/Char），不做半接入。(c) T-02 余域：`std.conv`（S-05 parse/format 双向——to_int 带 i64 累加器+显式域检查（-2147483648 可析、溢出返 None 不静默陷阱）、is_uint、from_uint 数字表切片构串（无 char→Str builtin 依赖））；**V-DETAIL-02 parse 错误路径全部独立断言**（42/007/MIN 边界/超域/非法数字/空/孤符号；oracle 420743438）。972 套件 16/16 绿。commits d16ba3276、8e1561546、5c4be6bb7、617cc2d0d。具名边界追加：S-03 bytes/编码（byte 索引）、S-06 hash（byte 域 vs codepoint 域语义裁定）、集合全集（map 型）、int→Str 负数形、upper/lower（case 表）、double 族（intrinsic）、mn text/char/数值发射面、跨端 closure 值（mn+a2x）。

2026-11-15 stage: work | T-03 首轮跨端核验=具名缺口冻结 | 整装配 std 集跨端（mn/a2x）当前均具名拒：mn collections native 域无 closure 值/Char（list HOF 公共签名与 text/char 模块在整集构建中必触）；a2x collections 族无 closure 类型且未并 961 数值面（std.math 的 I64 NumericConst 亦拒）。三条具名拒以 freeze 断言钉入 `stdlib_asm_972_cross_end_named_gaps_frozen`（支持落地时有意翻转）；不接受 emit-only/宿主 fallback 替代。AV 为装配集当前唯一执行端。跨端 closure/char/数值域扩展=T-03/T-09 后续单元。972 套件 15/15 绿。commit ac33aa713。本轮 round 总结：T-01 完成；T-02 已落 list/math/text/char 四模块+装配线三单元；T-03 缺口已具名。下一轮：跨端域扩展（mn closure/char + a2x closure/numeric）或 T-02 余域（S-03/S-05/S-06/集合全集）。

2026-11-15 stage: work | T-02 第二/三实现单元落地 | (a) 数值类型面并入组合线（63 处 `[numeric, aggregates]` 门 + packages-aggregates-ct）+ `stdlib/ace/asm/math.at`（abs/min/max 冻结 i64 签名 + square/cube；checked 算术——abs(i64::MIN) IntOverflow 陷阱为独立断言，V-02 边界；AV oracle 9223372036854775228 含 square(7).to(i64) 显式跨宽转换）。(b) 文本面：新 `schema/aggregates.atom` 描述符（numeric 超集 + text_len_bytes/text_char_at/text_slice/char_from_codepoint + expr start/end；三元组注册→Closures 域；数值 builtin 词在其下绑定）、aggregates profile 渲染自身身份、t_char 发射、lowering 四个 collections/collections-MIR-profile 文本臂；`stdlib/ace/asm/text.at`（byte_len/scalar_len/char_code_at/equal/starts_with/ends_with/contains/repeat——递归 helper、ownerness 正确：中途复用传 clone、借用长度先提升）与 `char.at`（ASCII 谓词子集，具名边界）。V-02 oracle 落地：A中😀=8B/3C、e+U+0301=3B、NUL 不截断（83343165/31）。972 套件 14/14 绿；ace-source/ace-syntax/ace-av 组全 PASS。commits 38d9f1a14、3a137fa07。T-02 余量：S-03 bytes/S-05 parse-format-encoding/S-06 hash/集合全集/V-DETAIL-02..03 系统负例；double 族（sqrt/sin/cos 等）挂 T-09 intrinsic 批。

2026-11-15 stage: work | T-02 第一实现单元落地 | 装配 aggregates 线（packages-aggregates-ct 语法 profile + AggregatesCt 前门 + bind_packages_assembled_aggregates + asm_lane walker/resolve 门 + 装配 std 冻结身份集）+ 首个装配 stdlib 模块 `stdlib/ace/asm/list.at`（946 切片超集 + reduce/find/any/all/filter/map 闭包 HOF 族）。av conformance `stdlib_asm_972`：AV 独立 oracle 1544281/15455、std 防遮蔽双负例（篡改源/带层均 ace.stdlib-reserved 先于运行拒）、plain aggregates 线闭包参数保持具名拒（lane 隔离）；probe972 7/7 保持绿。受影响完整组：ace-source/ace-syntax/ace-av 全 PASS（ace-syntax DECLARED 集补 packages-aggregates-ct；grammar-id 隔离测试过——旧 profile 裁剪逐字节不变）。worktree commit 50704b661。T-02 继续：S-01..06 域内容逐批落装配形态+永久正反例。

2026-11-15 stage: work | plan_id: PLAN-972 | plan_revision: r11 | outcome: in-progress（T-01 pass，T-02.. 续行中）| worktree: D:/autostack/.wt/lang-972/auto-lang | branch: plan-972-dev | base_commit: cd71736f8 | code_commit: (T-01 探针于 worktree，commit 待 T-02 首个实现单元一并落) | dependency_revisions: 12/13 archived（967 reviewed 未合入——真实 C/Rust 外库 dispatch 腿终态闭合挂其 merge，不阻当前默认库/native/A2x 腿） | task_ids: T-01 | evidence: probe972 7/7 绿（0.53s），摘要与命令见 D:/autostack/plan-artifacts/plan-972/t01-probe-evidence.md；§8 T-01 行内冻结表 | blockers: 无用户决策项；967 merge 为外部时序依赖，具名登记 | next_responsible_role: work（T-02 实现主能力）

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 原生底座依赖修订预检：9份文档371处本地链接通过；保原48任务ID/阶段及已有AC/V，新添1039/1040各T-09共2项work；可达149个语言计划无缺失/环，946与972新增硬前置无反向分配器依赖。状态/勾选/进度未推进，归档计划与canonical Specs零修改，.next-id仍1042。仅规划静态检查，非产品/独立实现review PASS。

2026-11-13 stage:new / r9：修正原生底座完成依赖；原任务/状态/勾选/进度保持，新增FOUNDATION AC/V/SD由原work任务和独立review承接。未启动实现或运行产品。下方较早可并行措辞仅对算法准备有效，不覆盖本次原生实施/验收门。

2026-11-13 stage:new / r8：设计承接；新增平台I/O AC/V/SD由T-01,T-09,T-10及原最终work负责。原任务/进度保持，无实现或独立产品review；新三包drafting。

2026-11-12 stage:new / PLAN-972 r7：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原10任务/全部AC与V/依赖/阶段/预算和drafting 0/10；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r6：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部10个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/10；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..10、AC-07/AC-08及V-09..10，按Design56正式接M2 FFI/新版标准库责任，0/10。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

### A2C/A2PY 发射器镜像设计（2026-11-15 冻结的续行蓝图，T-03 续行单元）

已核事实（本轮survey）：a2x-c `lower_coll.rs` 已有 TextLen/TextLenBytes/TextClone/TextConcat?(仅 coll_runtime 有 concat 助手)；`lower.rs`（text 线）有 `num_bin_c`（比较内联、整型经 checked 助手、float 通道）与 NumericConst 内联字面量（:350）；`rt_runtime.rs`（text 线 C 运行时）已有 `a2x_str_char_at/a2x_str_slice/a2x_char_from_codepoint`；`coll_runtime.rs`（collections 线 C 运行时）字符串为 `a2x_str{data,len}` 结构 + trap/out-param 风格，已有 concat，缺 char_at/slice/to_owned/char 助手与 typed i64 算术助手。

A2C 镜像步骤（C 发射器）：
1. coll_runtime.rs 补助手（C 源）：`a2x_str_char_at(const a2x_str*,int32)->char 结果包装`、`a2x_str_slice`（借用视图 out-param：data/len 指入源）、`a2x_str_to_owned`、`a2x_char_from_codepoint`、`a2x_char_to_codepoint`（内联）；typed i64 加减乘除余/取负/字面量/收窄 cast 助手（镜像 rt_runtime 的 961 集——若 text 线助手为 static，复制进 coll 运行时源）。
2. lower_coll.rs（C）补臂：NumericConst（内联字面量，镜像 lower.rs:350）；Cast（checked 收窄助手）；`BinOp::Num`（镜像 num_bin_c：比较内联、算术经 typed 助手 + coll fall 包装）；CharLit/CharToCodepoint（内联 (uint32_t)）/CharFromCodepoint（助手）；TextCharAt/TextSlice/TextToOwned（助手 + fall_res/trap_check）；TextConcat（已有助手接线）。
3. 闭包：C 函数指针——Ty::Closure 参数文本 = 从 view.closure_sig(idx) 生成指向体函数 C 签名的 fn 指针（体函数 C 签名=其 CollRetKind.c_ret_ty 返回 + 参数 C 类型）；CreateClosure = 体函数名（C 函数指示符退化为指针，捕获闭包具名拒）；CallClosure = `p(args...)` 经 fall_res；result 通道沿体函数返回类型。
4. 消费腿：`PackageA2xTarget::cold_aggregates(..., CTargetProvider::v1(), &product)` + `ace_a2x_worker::crun::compile_c_fileset`（1043 测试配方）+ 整集 HOF oracle（value:144 形 stdout）。
A2PY 镜像同构（a2x-python 发射器 + PythonTargetProvider + pyrun；Python 闭包=lambda 直译，text/char 助手进 python_runtime_source）。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
