---
plan_id: PLAN-946
status: drafting
feature_name: ACE 集合与拥有资源公共算法
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-12
plan_revision: 7
supersedes_spec_components: [docs/specs/auto-ac/source-collections-variants.md, docs/specs/auto-a2c/collections-hir-backend.md, docs/specs/auto-a2x/checked-resource-facts.md]
new_spec_components: [docs/specs/stdlib/design/ace-collections.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
depends_on: [PLAN-943, PLAN-944, PLAN-945, PLAN-962]
current_step: 0
total_steps: 8
---

# [PLAN-946] ACE 集合与拥有资源公共算法

## 0. 变更摘要

2026-11-12 r7：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r6当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r5当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r4：按用户最新时序决定，将本包定位为**集合与拥有资源的M2有限实施切片**。[Design55 §1.1 M1-TAG](../design/55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)是全部研究/盘点/前探/实现的先行门；不再采用“可先于M1收口调查”的安排。本轮只修订合同，status=drafting、0/8，没有执行授权或任务完成声明。原任务/验收保留，新增门核验并入T-01及独立review。

在既有合法 collections profile 内交付公共集合算法与 Option/Result 消费路径，复用共享资源事实，证明失败提交、借用和最后owner清理。

设计依据：[Design54](../design/54-stdlib-semantics-and-ace-integration.md)，衔接 Design33/46/51。**本轮仅编写设计和计划，先不施工**。8个任务全部未执行；未建实施树/材料scope、未编译/跑产品测试、未切默认。`cargo t --list` 仅核入口，非测试证据。

## 1. 目标

本包目标：在既有合法 collections profile 内交付公共集合算法与 Option/Result 消费路径，复用共享资源事实，证明失败提交、借用和最后owner清理。

非目标：不扩嵌List/任意用户泛型/闭包HOF/公开容器ABI，不复制资源分析，不把JSON泛型反射当集合附带能力，不降低原深链/预算门。

不改变原M1/M2 required/accepted-diff/NA、语料/oracle、既有资源预算及CAP进入门。缺口保留，不为通过删除子项。

## 2. 架构方案

当前正式前置：M1-TAG + PLAN-943, PLAN-944, PLAN-945, PLAN-962正式交付；另需本包实施授权。tag前不执行本包只读调查、probe或实现；本轮文档规划不算执行。


复用当前统一source/Checked HIR、共享MIR/资源事实及query/store。A2X消费HIR，AV/native消费共享MIR；公共语义与物理实现分开，旧runtime只按实际合同复用。标准操作身份、兼容策略和manifest双指纹遵守Design54 §4–7。

依赖：M1-TAG（见Design55 §1.1） + D943、D944、D945、D962；阶段依赖见§5/§8，历史快照不覆盖此行。

## 3. 技术栈

Auto公共源码、现有Rust装配/ACE workspace、既有TestCase与固定Cargo组。不得添加平行编译器、资源分析、store或自报PASS系统；新文件/接口为拟议，T-01先核真实consumer再定位置。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

从 `docs/specs/overview.md` 的stdlib/语言/ACE边界取材，读取current stdlib项目卡、assembly-manifest/backend-assembly、相关模块源码及Design33/46/51。读取基线HEAD `a0750086c`；部分设计在磁盘有其他会话WIP，确切输入hash见Design54 §12，不把WIP当正式D。

用户授权原文：“按这个方向编写……设计文档……拆分后续实施计划，先不施工。”本轮仅AutoLang设计/计划/索引簿记，未授权任何实现、外仓写入或自动续跑；没有预算/模型/agent指派。未来执行需要后续授权，不能以本draft推断批准施工。

已核路径（后继开工重核实际符号与版本）：

- `stdlib/auto/list.at`（现存；文件/目录）
- `experimental/ace/crates/core/src/collections_facts.rs`（现存；文件/目录）
- `experimental/ace/crates/lowering`（现存；文件/目录）
- `experimental/ace/crates/av-core`（现存；文件/目录）
- `experimental/ace/crates/native`（现存；文件/目录）
- `experimental/ace/crates/collections-runtime`（现存；文件/目录）
- `experimental/ace/crates/a2x-rust`（现存；文件/目录）
- `experimental/ace/crates/a2x-c`（现存；文件/目录）

边界：现有文档的历史数量/状态不自动当当前运行事实；`primitive-str-byte-len`与872绑定矩阵差异先由943按具体入口处理。旧math公开i64与ACE i32域分开；普通Rust/C能编译不证明Auto语义。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：有限集合/拥有元素公共算法。已读当前规范入口：[docs/specs/auto-ac/source-collections-variants.md](../specs/auto-ac/source-collections-variants.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [stdlib/auto/list.at](../../stdlib/auto/list.at)：公共算法，不全部native化；读取SHA256前16 `7e41ad83f39bc8ab`

- [experimental/ace/crates/core/src/collections_facts.rs](../../experimental/ace/crates/core/src/collections_facts.rs)：元素/视图/转移事实；读取SHA256前16 `4ce74d59e6a5305a`

- [experimental/ace/crates/collections-runtime/src/lib.rs](../../experimental/ace/crates/collections-runtime/src/lib.rs)：容量/分配/销毁原语；读取SHA256前16 `8bbbb4aa477f1110`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


### M1 tag后的M2定位与进入门（r4，当前）

本包保留现行有限集合/Option/Result、真实公共源码四端、失败提交与最后owner验证，不替代972完整集合库。T-01消费945及其947合同来源，依据Storage设计方向核真实布局/容量/资源来源；不把DefaultStorage、Fixed或arena语法当已定接口，不由类型名推断无堆。具体Storage与底层分配接口在M2阶段结合实际支持域再冻结，缺语言能力先正式解决前置，禁止typed替代。

[Design55 §1.1 M1-TAG](../design/55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)定义唯一时序判据。T-01须记正式tag ref/commit、M1最终验收/合并收据与当前base祖先关系；不能把943/944归档、941任务数、某个旧tag或拆分重构989的tag-ready当作本门。tag缺失/身份不符时不调用probe/compiler/provider、不建实施树；取得真实收据后重新核门，不由时长自动放行。tag和实现授权是两个独立条件。

已归档943/944不重开；只重核实际消费的源码/Spec/接口与适用证据，不把旧差额表或runtime原语直接当新版公共标准库已接。研究输入和结论对正式tag及当前source分别绑定，不能从WIP冻结下一阶段合同。

### 2026-11-06 M2总体路线重排（r2当前增量）

原AC-01..06与T-01..07、有限支持/非目标保持；本轮授权重新制定M2设计/计划，只改后继职责，不增加实现完成声明。依据[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)与[M2路线](../design/strategy/ace-m2-delivery-roadmap.md)，本包未覆盖的完整能力由 **PLAN-972** 接管。T-05交接必须把每个缺口映射具体后继AC/输入/oracle；不能只写“后续另立项”。后继对本包单向依赖，本包不等待后继完成才交基础切片。

D943/D944/D950当前均已归档；旧§9的“前置未齐”记录保留为当时历史，开工须核当前正式SHA/适用能力。945/946仍不得把i64窄化，947仍不冒已实现File/Task/Stream，948仍不冒全域已迁。新后继交付时在对应Plan记收据，不倒填本包旧证据。


消费864/889/904/926正式合法域，T-01至少冻结一个标量List归约/查询公共算法、一个拥有Str元素的公共操作及Option/Result分支，给出真实公共符号、参数/返回/借用形态。当前List参数/返回、嵌List和源码List.clone有拒绝边界，不能假定一般list.at已可装载。

优先以已有合法的局部List/for-in/inline受检实现承载；若公共算法必需的参数/可见性超域，T-01必须提交有限扩域方案与反例并修订本Plan或建立前置，不能手写typed MIR替代真实source。公共算法不全部native化，底层容器操作沿现有MIR/runtime。Option/Result只承诺profile已有有限形态，不承诺通用泛型库。

### 执行合同与下一入口

输入→实际producer→存储/中间表示→consumer见上列路径及§6，未新增接线由T-01/T-02创建。开工先核主检出计划revision、正式依赖SHA与当前树HEAD，记录实际已闭/未闭项；T-01只允许有界前探与合同冻结，不能用整包重构解决未知。需改变公开语义/支持阈值/跨仓权限则修订Plan并向用户说明具体决定。

所有临时probe、日志、receipts、外部验证副本统一 `D:/autostack/plan-artifacts/plan-946/`，开工登记owner/用途/在途状态；本轮未创建。常规构建在唯一实施树target，外置target也只在此scope。留存结论最终折入本Plan；测试fixtures若为产品长期用例留在真实owner测试目录并明确T-02..05登记，不把临时探针散落仓根。


### 本次复核补充：跨模块公共集合调用（r3）

消费962的集合参数/返回与资源签名，冻结至少一个真正跨模块的List公共调用；局部inline只作内部实现，不能替代公共API。

本节由T-08实施并以AC-07/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r7当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.at](../../crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.at)（下列节选L1–L7；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
var list = List.new()
list.push(10)
list.push(20)
list.push(30)
var popped = list.pop()
list.drop()
popped
```

本包所需独立结果：旧源push10/20/30后pop应30；独立归约[1..5]为15，find首个>3为4。新ACE另核内容，不只打印ok或容量。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 普通List/Option/Result源→962布局/资源事实→公共Auto算法→容器MIR/runtime→四端值/释放。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct CollectionOp { element: TypeRef, storage: StorageId, access: BorrowMode, result_owner: OwnerRule }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [stdlib/auto/list.at](../../stdlib/auto/list.at) | 公共算法，不全部native化 |
| [experimental/ace/crates/core/src/collections_facts.rs](../../experimental/ace/crates/core/src/collections_facts.rs) | 元素/视图/转移事实 |
| [experimental/ace/crates/collections-runtime/src/lib.rs](../../experimental/ace/crates/collections-runtime/src/lib.rs) | 容量/分配/销毁原语 |

**状态、错误与资源。** 空pop/越界按原Option/默认值/错误合同分别返回或拒绝，不一律改为异常；move后使用、视图物化/释放后句柄须正确拒，D-962-C4不能作为新M2合法所有权。 失败grow/clone不丢旧元素；借用不能越scope，拥有Str元素释放一次；Storage计量与原预算不变。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.at](../../crates/auto-lang/test/vm/13_collections/021_list_push_pop/list_push_pop.at) | push/pop/drop及值 | `list_push_pop.expected.result`；SHA256前16 `8031d9b0630cf28d` | 拟ace.m2.p946.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at](../../crates/auto-lang/test/vm/09_functions/012_list_reduce_find_any_all/list_reduce_find_any_all.at) | 归约/查找/any/all旧公开断言；闭包完整组合交964/972 | `list_reduce_find_any_all.expected.out`；SHA256前16 `6c49addc909fb55e` | 拟ace.m2.p946.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at](../../crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at) | 拥有结构元素，List内建不代用户泛型 | `list_struct.expected.out`；SHA256前16 `6adacccc13da0620` | 拟ace.m2.p946.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 有限公开算法精确源形/参数返回与Storage职责先冻结；不把DefaultStorage/Fixed/arena名当已定API，不要求先完成972整库。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

以下是**拟议**delta，review按实际范围定稿，merge才写canonical Specs；本轮不写current Spec。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-ac/source-collections-variants.md` | 登记有限公共算法与资源义务、真实source拒绝集；不推广到任意容器/泛型。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-02 | modify | `docs/specs/auto-a2c/collections-hir-backend.md` | 登记有限公共算法与资源义务、真实source拒绝集；不推广到任意容器/泛型。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-03 | modify | `docs/specs/auto-a2x/checked-resource-facts.md` | 登记有限公共算法与资源义务、真实source拒绝集；不推广到任意容器/泛型。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-04 | add | `docs/specs/stdlib/design/ace-collections.md` | 登记有限公共算法与资源义务、真实source拒绝集；不推广到任意容器/泛型。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-05 | modify | `docs/specs/stdlib/project.md` | 后续实际交付记录消费的M1 tag/源码/公共合同来源；规划门未闭不提高任何实现状态；943/944历史不重写 | M2时序与证据适用性 | AC-08 |
| SD-PHASE | add | `docs/specs/stdlib/design/ace-collections.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/stdlib/design/ace-collections.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-05及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部是待执行设计，非当前证据。首批实际运行限既有已认证Windows x64；有界T-01只做§6列出的最小输入与对应合法/非法近邻，预算沿各原recipe，不临时添加新性能阈值或做全库压力。新增fixture由T-02..05接到真实entrypoint，T-01把下表的概念输入落实成当前合法语法/入口，原不支持域不改成伪正例。

| Case ID | AC/invariant | 输入/设置 | 真实入口或拟议接线 | 独立可观察结果 | owner/coverage |
|---|---|---|---|---|---|
| V-01 | AC-01 | 冻结局部List[1,2,3]归约、拥有字符串分支 | 公共source→四端真实执行（T-01/02/03） | 归约6与独立字符串/分支expected；不能只比较四端 | work/T-05；仅本包冻结域 |
| V-02 | AC-02 | 同一公共调用错元素型、借用逃逸及合法邻例 | 统一type/resource verifier | 各拒绝在后端前，合法例进入producer；typed成功不顶source | work/T-05；仅本包冻结域 |
| V-03 | AC-03 | push/set/switch的分配/预算故障，旧owner仍被读取 | AV/native/Rust/C原fault与资源计数 | 失败后旧值不变，输入仍由合同方负责，无泄漏/双析构 | work/T-05；仅本包冻结域 |
| V-04 | AC-04 | 单公共算法body与元素descriptor变化，重启首编辑 | 现有query/store/四端消费 | 精确失效和新值；错descriptor使用前拒，无整包冷重编冒恢复 | work/T-05；仅本包冻结域 |

验证入口：`cargo t stdlib ace-source ace-lowering ace-av ace-native ace-a2x ace-runtime case-ace-m1-collections-864 case-ace-m1-native-collections-889`。组名已从本轮 `cargo t --list` 核验；work按实际改动确认完整职责组与必要下游并集，最终review后冻结提交沿同并集核门，身份/输入未变证据可复用。904/926新HIR实际编译/认证腿沿其原recipe；现有emit-daily固定组只作对应轴，不冒真实运行。

新用例必须进入所选固定完整组；若现有组无法覆盖新合同包，T-05明确登记新固定职责组并在review前冻结，不靠过滤的零选择假绿。原ignore/平台/性能/认证条件不减；不按计划机械运行全ACE或全仓。周期`cargo t all`仅由主检出due门负责。列发现/执行/条件排除数，准备/运行/清理/总时间与子终态；编译错误、超时、未执行不算语义反例PASS。

实现者自查不能替代final independent review。本轮未指派独立agent；未来review以独立上下文在同一唯一worktree核每条AC、遗漏/延后/workaround、warning/fmt/debug及Spec delta，绑定revision/SHA。复审期间停止实施写入，发现问题交回同树修复。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-07 | 入口将List[1,2,3]传给独立公共模块得到6；返回owned文本集合后源owner释放仍可读，借用逃逸反例调用前拒绝。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### M1-TAG进入核验（待执行）

| Case | AC | 输入/反控 | 入口/证据 | 可观察结果 | owner/边界 |
|---|---|---|---|---|---|
| V-09 | AC-08 | T-01核M1-TAG收据：无tag、tag指向未验收/WIP、只有989 tag-ready、缺祖先/正式收据分别拒绝；合法M1交付tag+本包授权+适用依赖为正对照。 | 本包开工入口（T-01核验及T-05/独立review核收据）；未来以实际交付ref/原收据验证，负控只用隔离输入副本，不创建/移动正式tag | 未闭门时研究/probe/compiler/provider调用及建树均为0；合法对照仅允许本包依赖已齐阶段，不自动提升支持状态 | work/T-01/T-05；review/T-06；非产品性能或M1通过证明 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 旧源push10/20/30后pop应30；独立归约[1..5]为15，find首个>3为4。新ACE另核内容，不只打印ok或容量。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 空pop/越界按原Option/默认值/错误合同分别返回或拒绝，不一律改为异常；move后使用、视图物化/释放后句柄须正确拒，D-962-C4不能作为新M2合法所有权。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 失败grow/clone不丢旧元素；借用不能越scope，拥有Str元素释放一次；Storage计量与原预算不变。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 冻结的标量List算法、拥有Str操作、有限Option/Result公共source路径在四端对独立oracle通过。 | T-01及T-05；§6对应输入与独立观察 |
| AC-02 | work | 无新的未批准容器域；错误元素/借用逃逸/不支持嵌List具名拒，合法近邻仍过。 | T-02及T-05；§6对应输入与独立观察 |
| AC-03 | work | 失败保旧owner/out，move/clone与最后owner清理正确；原深链和资源预算保持。 | T-03及T-05；§6对应输入与独立观察 |
| AC-04 | work | 集合合同/接口变更与恢复首编辑正确，资源分析保持单源且未改函数不重产。 | T-04及T-05；§6对应输入与独立观察 |
| AC-05 | review | 独立核§6反例/合法对照、所有AC、遗漏/延后、warning/fmt/debug及Spec delta；结论绑定冻结revision/SHA | T-06独立复审记录；不信勾选/父exit0 |
| AC-06 | merge | 实际Spec落地、代码来源一致、归档与scope/树清理对账完成；不得把正确性留到merge | T-07实际收据/guard及路径核验 |
| AC-07 | work | 跨模块公共集合调用的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-08 | work | 本包全部研究/准备/实施在M1-TAG及实施授权后；T-01绑定正式ref/commit/验收合并收据和base，非法/缺证入口零执行；原AC/T/资源门保持。 | T-01、V-09；T-05及原独立review任务核本revision全项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-05；原独立review复核 |

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

本组包括调研/前探在内均须先核M1-TAG正式收据与本包实施授权；该门未闭不建树或运行探针。

- [ ] **T-01｜M2-A｜owner_stage: work｜AC-01/AC-08**：核M1-TAG实际收据和本包实施授权，消费D943/D944/D945/D962及947合同来源；从tag与当前主线复核原collections拒绝集、公共调用/资源形态及Storage设计待定项，完成whole-path前探后冻结支持表；门未闭不执行。

- [ ] **T-02｜M2-A｜owner_stage: work｜AC-02**：实现已冻结公共算法及绑定，复用现有for-in/分支展开与合同，不引入重复指令族/资源分析；必要有限扩域须先修订验收边界。

- [ ] **T-03｜M2-A｜owner_stage: work｜AC-03**：接拥有Str/Variant及Option/Result分支的四端producer与runtime；明确view不分配、move/clone/旧臂销毁及失败前检查→提交。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-07**：消费962的集合参数/返回与资源签名，冻结至少一个真正跨模块的List公共调用；局部inline只作内部实现，不能替代公共API。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [ ] **T-04｜M2-A｜owner_stage: work｜AC-04**：验证公共算法的接口/元素类型/操作合同变化引发真实依赖失效与恢复首编辑，复用既有增量/预算机制。

- [ ] **T-05｜M2-A｜owner_stage: work｜AC-04**：补错元素/越界/失败回滚/借用逃逸与合法邻例、拥有容器深链和资源fault，跑完整职责组与原collections命名oracle。

- [ ] **T-06｜M2-A｜owner_stage: review｜AC-05**：在同一实施树以独立上下文执行 `/auto-plan:review`，逐AC/反例与真实diff核验，具名记录债务候选；实际通过才进入reviewed。

- [ ] **T-07｜M2-A｜owner_stage: merge｜AC-06**：按 `/auto-plan:merge` 核来源/落Spec/归档；证据摘要折入Plan，逐命令实际输出路径对账，删统一scope（具名持有除外），刷新retention；`wt-guard` clean后移除树/分支，实际完成后记账。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-946 r7：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r6：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r5：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

本轮实际静态核验：9份文档本地链接/门锚点、四Plan的0–10章节/任务唯一性与total_steps、AC/V引用、drafting/零进度通过；保35个原任务，新增4条时序AC/V，相关可达97个语言Plan节点无环且无缺失前置；空白检查通过。未运行研究/产品测试，未核M1签章或tag通过。

2026-11-09 r4时序与M2定位修订：用户将未执行标准库准备统一置于M1完成并打正式tag之后。本轮改§0/2/5/6/7/8及依赖，保全部原任务/验收，新增AC-08/V-09并入T-01/05和原独立review；无新增任务，进度0/8。stage=new，outcome=blocked-for-work（M1-TAG未核通过，且未获施工授权）；next=M1正式交付tag后核对应授权、来源和阶段依赖。历史“可以提前调查”的安排被本revision撤销，不是当前进入许可。文档静态检查不构成产品或独立实现复审PASS。


2026-11-06第二轮计划复核：r3，新增T-08/AC-07/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06 r2 new交接：新增§5明确后继映射；原验收不变，T-05的差额交接绑定正式新计划。status仍drafting，未实施；历史r1判定不是当前状态。


2026-11-05 draft handoff：`stage: new | plan_id: PLAN-946 | plan_revision: 1 | outcome: blocked（正式依赖/首探结论未齐；这是new交接判定，不是Plan状态） | next: 等待用户实施授权及PLAN-943 正式交付（D943）、PLAN-944 正式交付（D944）、PLAN-945 正式交付（D945）；先做§8允许的有界准备，不执行依赖未齐的任务。`

本轮只做文档静态预检与固定组名称核验；实施、产品验证、独立final review、Spec沉淀、merge/archive/cleanup全部未执行。current_step=0/7。非完成声明。

实际文档预检：Design54及六份Plan的本地链接、现存源码路径、0–10章节、七任务/六AC、Spec delta映射、drafting/0-of-7及943–948编号唯一性通过；`.next-id=949`。已核所列固定组名；相关跟踪文档`git diff --check`通过。该检查不证明任何拟议功能、产品测试或独立final review已通过。

## 10. 待澄清事项

正式M1 tag名/commit与最终验收合并收据由实际交付填写；本轮未核门、未打tag。具体Storage/分配服务接口在M2实施阶段讨论，不因prelude愿景预定allocator API。

- 首批符号/合法source/profile/接口位置由943及本包T-01有界前探冻结；§5/§6给出了最小关闭条件，缺能力保持具名阻塞，不静默降scope。
- 完整CAP仍守Design46 A/B/C1–C3门；本包无权提前完整求解或以候选字段宣称已实现。
- 实施授权尚未给出；其余未指定预算/模型/角色不虚构约束。跨仓若后续需写代码，由对应owner另建合同与授权。
