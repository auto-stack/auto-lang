---
plan_id: PLAN-966
status: drafting
feature_name: 完整编译期执行宏与领域生成
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 6
current_step: 0
total_steps: 10
depends_on: [PLAN-960, PLAN-963, PLAN-964, PLAN-965, PLAN-936, PLAN-958]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-lang/comptime/design/hash-syntax.md, docs/specs/auto-cte/scalar-query-profile.md]
new_spec_components: [docs/specs/ace-migration/m2-metaprogramming.md]
touched_goals: [GOAL-002]
---

# [PLAN-966] 完整编译期执行宏与领域生成

## 0. 变更摘要

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r5当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/10，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承936真实AV CT，扩#消歧、#if/#for/#is、宏展开与实际metadata查询；生成声明重过统一绑定/verify，保hygiene和来源链；CT依赖/权限/预算/冻结格式独立于运行时。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

当前正式前置：PLAN-960, PLAN-963, PLAN-964, PLAN-965, PLAN-936, PLAN-958。后文旧依赖叙述补充读取本行；允许先做只读准备，不能以旧依赖较少绕过真实能力门。


整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/source-cte`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/comptime`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/source-query`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/comptime`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `docs/specs/auto-lang/comptime/design/hash-syntax.md`（现存入口/规范；开工核真实符号与SHA）。
- `docs/specs/auto-cte/scalar-query-profile.md`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/core/src/source/ast.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/session.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：#编译期求值/条件/展开/元信息。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/source-cte](../../experimental/ace/crates/source-cte)：已交936源CT/宿主；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/comptime](../../experimental/ace/crates/comptime)：编译期服务/预算；目录职责已核，确切符号/闭包开工再核

- [crates/auto-lang-core/src/comptime](../../crates/auto-lang-core/src/comptime)：旧Hash规则研究，不移旧VM执行到ACE；目录职责已核，确切符号/闭包开工再核

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承936真实AV CT，扩#消歧、#if/#for/#is、宏展开与实际metadata查询；生成声明重过统一绑定/verify，保hygiene和来源链；CT依赖/权限/预算/冻结格式独立于运行时。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：实际worker CT前门（r2）

消费已归档958的A2X worker CT接线，逐端从公共worker而非私有service调用验证宏/CT回灌；冻结CT可用stdlib子集、显式环境输入与不可冻结值策略。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

新增LIB-M2-03的完整target条件接入；947只冻结装配合同，本包拥有源码CTE实现，不反向等973或978。按现行hash-syntax使用`#if`/`#is`，不引入Rust式cfg替代Auto语法。`#`上下文sys.os等显式输入来自编译目标，宿主信息另命名；函数/类型/常量/导入均可选，必要语法检查仍保留，未选分支不符号绑定/类型检查/链接。

选中use按所在模块参与名字冲突/循环/重导出/初始化规则，不能落入临时块作用域。目标、实际读取条件、选中provider及其ABI进入已有query依赖图；相关变更失效、无关变更复用，失败不破坏上一发布。Windows/Linux至少各有选择正反例；真实平台运行按支持矩阵取证，跨目标选择测试不冒本地执行。原完整宏/#for/metadata/workerCT义务均不减少。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 实现目标条件#语法/求值到统一模块导入和声明选择；明确目标OS/arch/env/ABI事实来源，未选分支不绑定平台依赖、选中use进入所在模块。 |
| T-10 | AC-08 | V-10 | 把条件输入/选中模块/provider/ABI接入现有query/store和workerCT，验证跨目标切换、相关失效、无关复用及失败保旧/恢复；覆盖函数/类型/常量和导入。 |

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/source-cte-936/src/choose42.at](../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/choose42.at)（下列节选L1–L7；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    return #{ choose(40, 42, false) }
}
```

本包所需独立结果：旧936源的编译期选择应得到其独立42；另以#if target切换Windows/Linux选不同use，未选分支不绑定/链接，选中声明重过verify。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** #if/#for/#is/#{ }普通源→既有AV CT与显式目标输入→冻结结果/生成声明→唯一绑定verify→四端worker。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct CompileTimeQuery { expression: ExprId, target: TargetFacts, read_set: DepSet, grants: CtGrants, budget: CtBudget }
struct GeneratedDecl { candidate: UntrustedDecl, origin: SourceMap, dependencies: DepSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/source-cte](../../experimental/ace/crates/source-cte) | 已交936源CT/宿主 |
| [experimental/ace/crates/comptime](../../experimental/ace/crates/comptime) | 编译期服务/预算 |
| [crates/auto-lang-core/src/comptime](../../crates/auto-lang-core/src/comptime) | 旧Hash规则研究，不移旧VM执行到ACE |

**状态、错误与资源。** #/#[歧义、不可冻结句柄、越权读取、生成坏decl或宏捕获同名均拒；工具失败不当语言拒绝。 CT和runtime授权/预算分账；target/文件读取变化使正确查询失效，无关变化复用，失败不发布半展开；公共worker接线保958交付。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [docs/specs/auto-lang/comptime/design/hash-syntax.md](../specs/auto-lang/comptime/design/hash-syntax.md) | 四种#语法及旧未兑现边界 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `c1dd22e46fc0373c` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [experimental/ace/crates/core/tests/cases/source_ct_936.rs](../../experimental/ace/crates/core/tests/cases/source_ct_936.rs) | capability/普通源既有正反控 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `4e9e9a9a534183d8` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/a2r/16_interop/017_comptime_read_text/comptime_read_text.at](../../crates/auto-lang/test/a2r/16_interop/017_comptime_read_text/comptime_read_text.at) | 环境/文件读取显式依赖，路径不冒新权限 | `comptime_read_text.expected.rs`；`comptime_read_text.expected.wrong.rs`；SHA256前16 `aa094e0629cbd6a8` | 拟ace.m2.p966.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 真实四构造/#if use作用域、hygiene/source-map、meta支持域及冻结值/预算边界明确；不反等973/978整包、不用Rust cfg代Auto条件。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-metaprogramming.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-lang/comptime/design/hash-syntax.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-04 | modify | `docs/specs/auto-cte/scalar-query-profile.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-metaprogramming.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-metaprogramming.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 宏生成同名局部不捕获调用者 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | CT读取类型字段后字段变更重算，未读body不重算 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 禁止持久化地址/handle | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 循环展开预算失败明确，下一编辑修复可恢复 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 不得把CT静默运行时执行 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-semantics ace-comptime ace-artifacts ace-av ace-a2x`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | CT生成声明改动使实际四端worker结果改变；无服务时不冒运行期求值；环境输入变更失效，未声明IO拒绝；MIX-02由本包产永久输入。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | Windows宿主目标Linux与反向目标分别选择正确分支；未选分支平台库缺失不阻绑定，语法坏仍报；选中缺库/重复符号/无匹配平台分别具名失败，外层函数可引用选中导入。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 先Windows再Linux切换不得复用错误库；修改被读条件/选中provider应失效，未选平台实现body变化不重编本目标；失败后旧版本可用、修复后新发布正确；四端按独立expected运行。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |

新增触面职责组：`cargo t ace-syntax ace-source ace-semantics ace-comptime ace-artifacts ace-av ace-native ace-a2x`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧936源的编译期选择应得到其独立42；另以#if target切换Windows/Linux选不同use，未选分支不绑定/链接，选中声明重过verify。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | #/#[歧义、不可冻结句柄、越权读取、生成坏decl或宏捕获同名均拒；工具失败不当语言拒绝。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | CT和runtime授权/预算分账；target/文件读取变化使正确查询失效，无关变化复用，失败不发布半展开；公共worker接线保958交付。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 实际worker CT前门的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | LIB-M2-03源码选择语义从公共worker/source入口闭合，无旧VM执行#、无独立平台parser/装配旁路。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | LIB-M2-03缓存与四端实际消费均有证据；不能用仅AST或仅生成片段代执行，平台环境排除单列。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

## 8. 执行步骤

当前共10任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [ ] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。

- [ ] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。

- [ ] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06**：消费已归档958的A2X worker CT接线，逐端从公共worker而非私有service调用验证宏/CT回灌；冻结CT可用stdlib子集、显式环境输入与不可冻结值策略。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [ ] **T-09｜M2-A｜owner_stage: work｜AC-07**：实现目标条件#语法/求值到统一模块导入和声明选择；明确目标OS/arch/env/ABI事实来源，未选分支不绑定平台依赖、选中use进入所在模块。 验证V-09，证据汇入原最终work验证。

- [ ] **T-10｜M2-A｜owner_stage: work｜AC-08**：把条件输入/选中模块/provider/ABI接入现有query/store和workerCT，验证跨目标切换、相关失效、无关复用及失败保旧/恢复；覆盖函数/类型/常量和导入。 验证V-10，证据汇入原最终work验证。

- [ ] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-966/auto-lang`，分支 `plan-966-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-966/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-966 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原10任务/全部AC与V/依赖/阶段/预算和drafting 0/10；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部10个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/10；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..10、AC-07/AC-08及V-09..10，按Design56正式接M2 FFI/新版标准库责任，0/10。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
