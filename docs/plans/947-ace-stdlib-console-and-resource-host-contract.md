---
plan_id: PLAN-947
status: executing
feature_name: ACE 控制台接入、标准库分层与装配合同
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-12
plan_revision: 9
supersedes_spec_components: [docs/specs/auto-ac/source-console-output.md, docs/specs/stdlib/project.md, docs/specs/stdlib/design/ace-binding-and-assembly.md, docs/specs/stdlib/design/assembly-manifest.md]
new_spec_components: [docs/specs/stdlib/design/ace-host-services.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
depends_on: [PLAN-943, PLAN-944, PLAN-960]
current_step: 3
total_steps: 10
---

# [PLAN-947] ACE 控制台接入、标准库分层与装配合同

## 0. 变更摘要

2026-11-12 r9：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r8当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-10 r7当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r6当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r5：按用户最新时序决定，将本包定位为**M2公共标准库合同、组织接入及原最小控制台实现**。[Design55 §1.1 M1-TAG](../design/55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)是全部研究/盘点/前探/实现的先行门；不再采用“可先于M1收口调查”的安排。本轮只修订合同，status=drafting、0/10，没有执行授权或任务完成声明。原任务/验收保留，新增门核验并入T-01及独立review。

历史修订：2026-11-09 r4：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/10，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

将有限公共控制台输出接入已有AV/native host-console机制，验证每轮授权/取消/恢复；为File/Task/Stream形成有证据的有限资源合同和后继接口。

设计依据：[Design54](../design/54-stdlib-semantics-and-ace-integration.md)，衔接 Design33/46/51。**本轮仅编写设计和计划，先不施工**。全部任务均未执行（当前任务数以frontmatter为准）；未建实施树/材料scope、未编译/跑产品测试、未切默认。`cargo t --list` 仅核入口，非测试证据。

## 1. 目标

本包目标：将有限公共控制台输出接入已有AV/native host-console机制，验证每轮授权/取消/恢复；为File/Task/Stream形成有证据的有限资源合同和后继接口。

非目标：不承诺四端全部host，不实现完整fs/net/task/HTTP，不切CLI/SDK默认，不新增线程/执行器，不把trusted provider当隔离，不提前使用未交D939。

不改变原M1/M2 required/accepted-diff/NA、语料/oracle、既有资源预算及CAP进入门。缺口保留，不为通过删除子项。

## 2. 架构方案

当前正式前置：M1-TAG + PLAN-943, PLAN-944, PLAN-960正式交付；另需本包实施授权。tag前不执行本包只读调查、probe或实现；本轮文档规划不算执行。


复用当前统一source/Checked HIR、共享MIR/资源事实及query/store。A2X消费HIR，AV/native消费共享MIR；公共语义与物理实现分开，旧runtime只按实际合同复用。标准操作身份、兼容策略和manifest双指纹遵守Design54 §4–7。

依赖：M1-TAG（见Design55 §1.1） + D943、D944、D960；阶段依赖见§5/§8，历史快照不覆盖此行。

## 3. 技术栈

Auto公共源码、现有Rust装配/ACE workspace、既有TestCase与固定Cargo组。不得添加平行编译器、资源分析、store或自报PASS系统；新文件/接口为拟议，T-01先核真实consumer再定位置。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

从 `docs/specs/overview.md` 的stdlib/语言/ACE边界取材，读取current stdlib项目卡、assembly-manifest/backend-assembly、相关模块源码及Design33/46/51。读取基线HEAD `a0750086c`；部分设计在磁盘有其他会话WIP，确切输入hash见Design54 §12，不把WIP当正式D。

用户授权原文：“按这个方向编写……设计文档……拆分后续实施计划，先不施工。”本轮仅AutoLang设计/计划/索引簿记，未授权任何实现、外仓写入或自动续跑；没有预算/模型/agent指派。未来执行需要后续授权，不能以本draft推断批准施工。

已核路径（后继开工重核实际符号与版本）：

- `stdlib/auto/io.at`（现存；文件/目录）
- `experimental/ace/crates/av-core/src/host.rs`（现存；文件/目录）
- `experimental/ace/crates/av/src/console_host.rs`（现存；文件/目录）
- `experimental/ace/crates/core/schema/host-console.atom`（现存；文件/目录）
- `experimental/ace/crates/native/tests/cases/host_console_compat.rs`（现存；文件/目录）
- `experimental/ace/crates/foreign-contract`（现存；文件/目录）
- `docs/specs/auto-ac/source-console-output.md`（现存；文件/目录）
- `docs/specs/language-sdk/http-host-port.md`（现存；文件/目录）

边界：现有文档的历史数量/状态不自动当当前运行事实；`primitive-str-byte-len`与872绑定矩阵差异先由943按具体入口处理。旧math公开i64与ACE i32域分开；普通Rust/C能编译不证明Auto语义。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `crates/auto-lang-core/src/stdlib_assembly/model.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-lang-core/src/stdlib_assembly/manifest.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-lang-core/src/stdlib_assembly/contracts.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-lang-core/src/stdlib_assembly/loader.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/packages.rs`（现存入口/规范；开工核真实符号与SHA）。
- `docs/specs/auto-lang/comptime/design/hash-syntax.md`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：公共标准库分层/装配与console宿主合同。已读当前规范入口：[docs/specs/auto-ac/source-console-output.md](../specs/auto-ac/source-console-output.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-lang-core/src/stdlib_assembly/model.rs](../../crates/auto-lang-core/src/stdlib_assembly/model.rs)：复用AssemblyContext/LayerSelection/LogicalSignature，扩必要层/目标事实；读取SHA256前16 `95fbc7911a149d36`

- [crates/auto-lang-core/src/stdlib_assembly/manifest.rs](../../crates/auto-lang-core/src/stdlib_assembly/manifest.rs)：公共声明与provider选择；读取SHA256前16 `ef380114d6a21eb0`

- [experimental/ace/crates/av/src/console_host.rs](../../experimental/ace/crates/av/src/console_host.rs)：每run授权和原字节预算；读取SHA256前16 `ce2788ea93e54a1e`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


### M1 tag后的M2定位与进入门（r5，当前）

T-09在tag后首先整理统一Auto API/默认Auto算法、core/Storage/host/platform/domain分层，以及#if/#is的目标条件合同；T-10沿944单源装配落实必要模型/接口。Storage依据内存专题保留默认政策、具体实例、失败和释放职责，实际函数签名/ABI布局须在届时讨论冻结，不预设Zig式处处allocator。原console授权/预算/恢复与File/Task/Stream有限前探义务全部保持；C/Rust自动binding归967、目标条件实现归966、完整基础库归972、真实系统服务归973，不为本包加完整966/967/972后置依赖。

[Design55 §1.1 M1-TAG](../design/55-ace-m2-complete-language-and-product-replacement.md#11-m1-tag标准库准备计划的统一进入门)定义唯一时序判据。T-01须记正式tag ref/commit、M1最终验收/合并收据与当前base祖先关系；不能把943/944归档、941任务数、某个旧tag或拆分重构989的tag-ready当作本门。tag缺失/身份不符时不调用probe/compiler/provider、不建实施树；取得真实收据后重新核门，不由时长自动放行。tag和实现授权是两个独立条件。

已归档943/944不重开；只重核实际消费的源码/Spec/接口与适用证据，不把旧差额表或runtime原语直接当新版公共标准库已接。研究输入和结论对正式tag及当前source分别绑定，不能从WIP冻结下一阶段合同。

### 2026-11-06 M2总体路线重排（r2当前增量）

原AC-01..06与T-01..07、有限支持/非目标保持；本轮授权重新制定M2设计/计划，只改后继职责，不增加实现完成声明。依据[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)与[M2路线](../design/strategy/ace-m2-delivery-roadmap.md)，本包未覆盖的完整能力由 **PLAN-967、PLAN-968、PLAN-973** 接管。T-05交接必须把每个缺口映射具体后继AC/输入/oracle；不能只写“后续另立项”。后继对本包单向依赖，本包不等待后继完成才交基础切片。

D943/D944/D950当前均已归档；旧§9的“前置未齐”记录保留为当时历史，开工须核当前正式SHA/适用能力。945/946仍不得把i64窄化，947仍不冒已实现File/Task/Stream，948仍不冒全域已迁。新后继交付时在对应Plan记收据，不倒填本包旧证据。


首期输出限943冻结公共print/say或明确新别名映射，以已有console/1支持的i32/bool/str/char为实际基础；say是否带换行必须按旧入口逐字冻结，不能默认等同print。AV/native分别真实调用，Rust/C host缺能力单列，不拿纯函数四端绿冒宿主四端绿。

ConsoleHost的MAX_CALL_BYTES=65536和MAX_RUN_BYTES=1048576及原poll/原子提交保持。scope授权每轮注入，不序列化；未授权provider dispatch=0。File/Task/Stream仅有界前探：创建/失败/borrow/move/close/cancel/退出/恢复逐状态定义，明确哪个路径依赖939或领域owner，必要真实探针在其已支持域内；未实现状态诚实记capability-missing，计划通过不代表资源服务交付。

### 执行合同与下一入口

输入→实际producer→存储/中间表示→consumer见上列路径及§6，未新增接线由T-01/T-02创建。开工先核主检出计划revision、正式依赖SHA与当前树HEAD，记录实际已闭/未闭项；T-01只允许有界前探与合同冻结，不能用整包重构解决未知。需改变公开语义/支持阈值/跨仓权限则修订Plan并向用户说明具体决定。

所有临时probe、日志、receipts、外部验证副本统一 `D:/autostack/plan-artifacts/plan-947/`，开工登记owner/用途/在途状态；本轮未创建。常规构建在唯一实施树target，外置target也只在此scope。留存结论最终折入本Plan；测试fixtures若为产品长期用例留在真实owner测试目录并明确T-02..05登记，不把临时探针散落仓根。


### 本次复核补充：console目标与资源前探分账（r3）

先消费960的公共宿主声明/装配；分别记录AV、AC、Rust、C的console支持，不把Rust/C缺口移交后无人承接：973负责四端补齐；File/Task/Stream仅前探，967/968/973接真实服务。

本节由T-08实施并以AC-07/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r4，当前合同）

原console/资源前探义务保持，新增LIB-M2-01..03的早期合同与装配基础。公共API/operation ID与默认Auto算法、core/alloc、host/domain分层；执行后端、宿主、目标平台与provider实现语言分别建模。禁止新增ace.std或.ace.at分叉，也不能把AV/native console已接当四端系统服务已交。

T-09先核当前sys/OS公共名字与hash-syntax，冻结`#if`/`#is`的目标而非宿主语义、未选分支仅语法检查、导入提升到所在模块、无/歧义provider具名失败。T-10把组织/版本/loader/授权恢复合同落入现有stdlib_assembly与source-query包事实，交可验证schema及合法/非法装配用例；本包不实施完整CTE或完整IO，966负责条件编译源码实现，972/973/978各接算法/系统/分发。因此本包不依赖966/967的整包交付。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-08 | V-09 | 冻结四轴装配、单一公共API和core/alloc/host/domain分层；按现行#语法核sys/OS名字，形成明确目标条件/模块导入语义与迁移表，交966实施而不在此另造CTE。 |
| T-10 | AC-09 | V-10 | 在现有装配model/manifest/contracts/loader及包查询中落实公共声明、默认Auto实现、host/platform/foreign和生成产物分工、schema版本/兼容规则、源码与运行加载边界；保944proof、943操作ID及恢复重授权。 |

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。明确默认core/alloc/基础系统库由Auto自有实现和薄OS适配承担，不以libc或Rust std间接委托作为默认；公共AutoABI与OS/外部ABI分开，Rust/C/provider依赖显式选择。Storage/allocator接口仍待原有M2讨论，不能提前冻结。

本包验收：操作→自有实现/OS适配/显式外库→默认选择→运行依赖逐行齐；缺默认provider或自动回落libc必须拒绝，合法自有provider可装配。 原任务T-09,T-10承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r9当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at](../../crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at)（下列节选L1–L5；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int {
 print("你好\n")
 eprint("é")
 return 0
}
```

本包所需独立结果：旧O03单次运行stdout准确为你好+两个LF，stderr为é+LF，exit0；空串/空流与无尾换行单独区分，不默认say=print。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 公共Auto声明→AssemblyContext/LogicalSignature→944装配/依赖→console授权→一次原子输出。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct LibraryBinding { declaration: SigRef, layer: LayerKind, target: TargetAbi, provider: ProviderId, required_grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-lang-core/src/stdlib_assembly/model.rs](../../crates/auto-lang-core/src/stdlib_assembly/model.rs) | 复用AssemblyContext/LayerSelection/LogicalSignature，扩必要层/目标事实 |
| [crates/auto-lang-core/src/stdlib_assembly/manifest.rs](../../crates/auto-lang-core/src/stdlib_assembly/manifest.rs) | 公共声明与provider选择 |
| [experimental/ace/crates/av/src/console_host.rs](../../experimental/ace/crates/av/src/console_host.rs) | 每run授权和原字节预算 |

**状态、错误与资源。** 错target/ABI/provider、无权限、65537字节调用或超过原run上限拒且输出不半提交；合法边界仍接受。 grant每run重授、非缓存信任；console原65536/1048576预算；File/Task/Stream只定义端口/有限前探，不假称已实现服务。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at](../../crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at) | UTF-8双流/exit原check保持一次run | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `cfdd28a727189e42` | 拟ace.m2.p947.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/vm/20_permission/002_ext_policy/ext_policy.at](../../crates/auto-lang/test/vm/20_permission/002_ext_policy/ext_policy.at) | 原权限拒绝入口 | `ext_policy.expected.out`；SHA256前16 `374a6f4a7ec012ca` | 拟ace.m2.p947.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/specs/stdlib/design/ace-binding-and-assembly.md](../specs/stdlib/design/ace-binding-and-assembly.md) | 943/944已交装配，不重复研究 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `6c8c8b068a3c235c` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 默认Auto/core/Storage/host/platform/domain分层和#if目标合同先交；实际系统实现归973、FFI归967、条件实现归966；原M1-TAG前不执行本Plan。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### T-01 冻结记录（2026-11-13，M1-TAG 后开工基线）

**基线重建**：tag `v0.6-ACE-M1`（commit b923362f4=C）→ 当前 master HEAD 84c575a54 全部 diff 仅为 docs/**（0 个非 docs 文件）——M1 验收身份与当前基面零代码漂移。依赖 D943（operations.toml `console.say` v1）/D944（AssemblyManifest schema4 + semantics-contract `ace-stdlib-assembly/1` 四 producer 白名单）/D960（packages profile + NATIVE/C producer 加入）按归档合同消费，6 个 r9 取材 SHA 逐一重算一致（§4.1）。

**公共输出冻结**（逐字依据，非推断）：
- 公共 `say(msg str)`（io.at:21 声明；io.vm.at:51 `say→print`；VM codegen expr_calls.rs:1398 `("io","say")→auto.print_str`；natives_io.rs:5-9 `vm_print` 在字符串后**恰追加一个 `\n`**）→ **say ≡ console/1 `auto.console.print.str.v1`（stdout，UTF-8 原字节+恰一个 0a，参数仅 str/Str）**——不是 eprint、无第二换行、非任意类型。
- host-console 源域 `print`/`eprint` 四类型 lane（i32/bool/str/char）与格式化（i32 十进制无正号/前导零、MIN 负号、bool true/false、char UTF-8 标量、每调用恰一个 0a）按 863 spec + av.rs:7006 `with_console_input` 保持不变。
- **预算**：MAX_CALL_BYTES=65536（含追加 LF 的长度）、MAX_RUN_BYTES=1048576、每模块 ≤8 console overload、两次 cancel poll 间原子提交、ProviderScope 无 guest 重入（av-core/host.rs:5-9, av.rs:5338-5420）。
- **AV 入口**：front_door_admit → compile_host_console_source → lower_checked_console → verify_mir → encode_abc → execute_console（每轮 authorize(grants) 先于任何 dispatch）。恢复产物不含授权/地址（console_host.rs envelope/key 均身份事实）。
- **native 现状与目标**：mn 四点 fail-closed 拒绝 host-console（emit.rs:145-186 profile 白名单、lower.rs:2666 HostCall、fingerprint.rs:123、typed_entry.rs:299）；本包 T-02 经专用 console 发射入口给 mn 真实 console 通道（机器码直调 OS write，admission 侧共享 av-core authorize 每轮授权，guest 侧 per-call 预算，shim 侧累计预算），不改普通 mn 单一函数准入面。
- **Storage/内存待定边界**（M2 讨论确认项，本包不冻结 allocator API）：默认政策、具体实例、失败与释放职责保留；实际函数签名/ABI 布局留待内存专题（Design56 §0.2），不预设 Zig 式处处 allocator，也不以 libc/Rust std 间接委托为默认（AC-NATIVE 分账在 T-09/T-10 落实）。

**T-09 先于依赖**：四轴装配/公共API分层合同（T-09）先于依赖它的 T-10 实现；不重复造线程/执行器。

### 规范增量

以下是**拟议**delta，review按实际范围定稿，merge才写canonical Specs；本轮不写current Spec。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-ac/source-console-output.md` | 只把AV/native公共console映射写implemented；File/Task/Stream有界前探结论标proposed/缺口，保宿主owner。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-02 | modify | `docs/specs/stdlib/project.md` | 只把AV/native公共console映射写implemented；File/Task/Stream有界前探结论标proposed/缺口，保宿主owner。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-03 | add | `docs/specs/stdlib/design/ace-host-services.md` | 只把AV/native公共console映射写implemented；File/Task/Stream有界前探结论标proposed/缺口，保宿主owner。 | Design54/本包实际边界 | AC-01..04/AC-07 |
| SD-04 | modify | `docs/specs/stdlib/design/ace-binding-and-assembly.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-08/AC-09 |
| SD-05 | modify | `docs/specs/stdlib/design/assembly-manifest.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-08/AC-09 |
| SD-06 | modify | `docs/specs/stdlib/project.md` | 后续实际交付记录消费的M1 tag/源码/公共合同来源；规划门未闭不提高任何实现状态；943/944历史不重写 | M2时序与证据适用性 | AC-10 |
| SD-PHASE | add | `docs/specs/stdlib/design/ace-host-services.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-NATIVE | add | `docs/specs/stdlib/design/ace-host-services.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |


| SD-DETAIL | add | `docs/specs/stdlib/design/ace-host-services.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-05及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部是待执行设计，非当前证据。首批实际运行限既有已认证Windows x64；有界T-01只做§6列出的最小输入与对应合法/非法近邻，预算沿各原recipe，不临时添加新性能阈值或做全库压力。新增fixture由T-02..05接到真实entrypoint，T-01把下表的概念输入落实成当前合法语法/入口，原不支持域不改成伪正例。

| Case ID | AC/invariant | 输入/设置 | 真实入口或拟议接线 | 独立可观察结果 | owner/coverage |
|---|---|---|---|---|---|
| V-01 | AC-01 | 冻结公共输出A中😀及含NUL消息 | 统一source→AV/native真实ConsoleHost | 精确字节/换行；stderr与stdout分开观察 | work/T-05；仅本包冻结域 |
| V-02 | AC-02 | 同一合法调用去授权或改精确签名，恢复旧快照 | 既有host admission + T-02/03公共绑定 | dispatch=0且具名拒；重新授权合法通过 | work/T-05；仅本包冻结域 |
| V-03 | AC-03 | 65536单次字节合法/65537超界及累计1MiB边界、实际poll取消 | ConsoleHost/对应native provider | 原Resource/Cancelled分类与原子输出；不是只检查cancel flag | work/T-05；仅本包冻结域 |
| V-04 | AC-04 | File/Task/Stream状态及939缺失/可用两种依赖事实 | T-04前探和真实原端口/支持域 | 有证据区分可接、缺能力、未交付；不以手工schema冒产品 | work/T-05；仅本包冻结域 |

验证入口：`cargo t stdlib ace-console ace-source ace-av ace-native ace-host`。组名已从本轮 `cargo t --list` 核验；work按实际改动确认完整职责组与必要下游并集，最终review后冻结提交沿同并集核门，身份/输入未变证据可复用。实际修改ABI事实或消费939时追加 ace-foreign-contract ace-abi；若只有只读核查不跑ABI全套。Rust/C host不在本包支持承诺中。

新用例必须进入所选固定完整组；若现有组无法覆盖新合同包，T-05明确登记新固定职责组并在review前冻结，不靠过滤的零选择假绿。原ignore/平台/性能/认证条件不减；不按计划机械运行全ACE或全仓。周期`cargo t all`仅由主检出due门负责。列发现/执行/条件排除数，准备/运行/清理/总时间与子终态；编译错误、超时、未执行不算语义反例PASS。

实现者自查不能替代final independent review。本轮未指派独立agent；未来review以独立上下文在同一唯一worktree核每条AC、遗漏/延后/workaround、warning/fmt/debug及Spec delta，绑定revision/SHA。复审期间停止实施写入，发现问题交回同树修复。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-07 | say与print分别按冻结原bytes断言；撤权限后同一恢复产物dispatch=0；资源前探未实现应输出capability-missing而非PASS。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-08 | 将同一公共操作映射不同目标/宿主；构建宿主Windows而目标Linux时合同选Linux。拒绝无/歧义provider；把平台ABI标为AutoABI的反例必须被审计拒绝。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-09 | 合法默认Auto与显式平台provider装配成功；删operation proof、混目标ABI、重复provider、恢复旧授权分别在dispatch前拒绝；旧公共路径有明确兼容映射。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |

新增触面职责组：`cargo t stdlib ace-stdlib ace-artifacts ace-source ace-semantics`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### M1-TAG进入核验（待执行）

| Case | AC | 输入/反控 | 入口/证据 | 可观察结果 | owner/边界 |
|---|---|---|---|---|---|
| V-11 | AC-10 | T-01核M1-TAG收据：无tag、tag指向未验收/WIP、只有989 tag-ready、缺祖先/正式收据分别拒绝；合法M1交付tag+本包授权+适用依赖为正对照。 | 本包开工入口（T-01核验及T-05/独立review核收据）；未来以实际交付ref/原收据验证，负控只用隔离输入副本，不创建/移动正式tag | 未闭门时研究/probe/compiler/provider调用及建树均为0；合法对照仅允许本包依赖已齐阶段，不自动提升支持状态 | work/T-01/T-05；review/T-06；非产品性能或M1通过证明 |

### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-09,T-10，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 操作→自有实现/OS适配/显式外库→默认选择→运行依赖逐行齐；缺默认provider或自动回落libc必须拒绝，合法自有provider可装配。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧O03单次运行stdout准确为你好+两个LF，stderr为é+LF，exit0；空串/空流与无尾换行单独区分，不默认say=print。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 错target/ABI/provider、无权限、65537字节调用或超过原run上限拒且输出不半提交；合法边界仍接受。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | grant每run重授、非缓存信任；console原65536/1048576预算；File/Task/Stream只定义端口/有限前探，不假称已实现服务。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 公共输出在AV/native实际精确输出，换行与UTF-8按冻结合同；不包装旧VM执行。 | T-01及T-05；§6对应输入与独立观察 |
| AC-02 | work | 无授权/错误签名在dispatch前拒，恢复不带旧授权/地址；新运行重新授权可执行。 | T-02及T-05；§6对应输入与独立观察 |
| AC-03 | work | 沿原预算及poll语义，失败不部分提交不泄漏；合法边界与超界分别观察。 | T-03及T-05；§6对应输入与独立观察 |
| AC-04 | work | File/Task/Stream每项有真实支持/缺口、资源状态、owner与关闭条件；939未交不算已接，不冒完整host交付。 | T-04及T-05；§6对应输入与独立观察 |
| AC-05 | review | 独立核§6反例/合法对照、所有AC、遗漏/延后、warning/fmt/debug及Spec delta；结论绑定冻结revision/SHA | T-06独立复审记录；不信勾选/父exit0 |
| AC-06 | merge | 实际Spec落地、代码来源一致、归档与scope/树清理对账完成；不得把正确性留到merge | T-07实际收据/guard及路径核验 |
| AC-07 | work | console目标与资源前探分账的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-08 | work | LIB-M2-01/03合同落实到真实公共模块/源码与后继AC，现有console范围无扩大冒领。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-09 | work | LIB-M2-02装配基础有实际schema/接线/迁移与反例，972/973/978有唯一接口来源；无第二loader或旧引擎绑定。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-10 | work | 本包全部研究/准备/实施在M1-TAG及实施授权后；T-01绑定正式ref/commit/验收合并收据和base，非法/缺证入口零执行；原AC/T/资源门保持。 | T-01、V-11；T-05及原独立review任务核本revision全项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-05；原独立review复核 |
| AC-NATIVE | work | 操作→自有实现/OS适配/显式外库→默认选择→运行依赖逐行齐；缺默认provider或自动回落libc必须拒绝，合法自有provider可装配。 | T-09,T-10 / V-NATIVE；原独立review核新增义务 |

## 8. 执行步骤

当前共10任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

本组包括调研/前探在内均须先核M1-TAG正式收据与本包实施授权；该门未闭不建树或运行探针。

- [x] **T-01｜M2-A｜owner_stage: work｜AC-01/AC-10**：先核M1-TAG及本包实施授权，消费D943/D944/D960并对tag/当前主线重建适用基线；在M2讨论确认Storage/内存资源待定边界，冻结公共输出拼写、换行、AV/native入口及原预算；T-09合同先于依赖它的实现，不重复造线程/执行器。 [✅ 2026-11-13：M1-TAG 四要件实测闭合（§4.1：tag v0.6-ACE-M1→b923362f4=HEAD祖先、1025 H/C 双 exit0、/goal 即实施授权、943/944/960/941/999 归档）；基线零代码漂移（tag→HEAD 全 docs）；say≡print.str.v1 逐字冻结（io.at:21/io.vm.at:51/expr_calls.rs:1398/natives_io.rs:5-9 四链证据）；预算/入口/Storage待定边界冻结于 §5 T-01 记录。]

- [x] **T-02｜M2-A｜owner_stage: work｜AC-02**：实现公共输出绑定和每轮宿主注入，复用console/1导入/签名/capability及既有执行路径，保不支持host的明确诊断。 [✅ 2026-11-13：say(str) 接入 console 源域（core/source/console.rs 拼写映射+三处保留名统一真源 is_reserved_name）；native(mn) 通道落地：split_units 搬运 host 声明（av/src/cache.rs）、专用 admit_console_unit 门（863 兼容不放松）、LowerMode::Console 发射（guest 侧精确格式化+65536 per-call 预算+单次 shim 原子分发）、shim 对象（累计 1MiB+WriteFile 原子）、每轮 av-core authorize 共享（未授权→不发射不执行 dispatch=0）；commit f81f9f4*（"feat(ace-console): public say binding..."）+ console_lane 9 测试绿。不支持 host 的明确诊断：Unit-entry 在 mn=命名能力边界（mn ABI 冻结零参 i32），Rust/C host 无能力单列（T-08 分账）。]

- [x] **T-03｜M2-A｜owner_stage: work｜AC-03**：接装配/持久恢复/重新授权、预算和真实取消poll边界；不缓存provider地址，不扩权，不承诺尚未到达poll时即时终止。 [✅ 2026-11-13：AV 侧 persist/restore/重新授权既有（console_host.rs，快照身份测试在案）；mn 通道 worker recipe ac/mn/console-run-v1 内建 persist→restore 重接纳→本轮 caps 授权（不持久化任何授权/地址）；预算：AV 侧既有 65536/65537/1MiB 测试 + mn 侧 per-call（guest guard）与 cumulative（shim 计数器）双层、65537 拒绝原子性与 1MiB 边界前输出保留均有永久测试（console_lane 9 例 + 语料 7 行双 lane + 3 个期望面反例控制）；取消：AV 进程内双 poll 点既有语义保持，native=deadline 杀灭的受控运行（每次调用原子提交可观察），不承诺未到 poll 点即时终止。]

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-07**：先消费960的公共宿主声明/装配；分别记录AV、AC、Rust、C的console支持，不把Rust/C缺口移交后无人承接：973负责四端补齐；File/Task/Stream仅前探，967/968/973接真实服务。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [ ] **T-09｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：冻结四轴装配、单一公共API和core/alloc/host/domain分层；按现行#语法核sys/OS名字，形成明确目标条件/模块导入语义与迁移表，交966实施而不在此另造CTE。 验证V-09，证据汇入原最终work验证。 新增V-NATIVE：明确默认core/alloc/基础系统库由Auto自有实现和薄OS适配承担，不以libc或Rust std间接委托作为默认；公共AutoABI与OS/外部ABI分开，Rust/C/provider依赖显式选择。Storage/allocator接口仍待原有M2讨论，不能提前冻结。

- [ ] **T-10｜M2-A｜owner_stage: work｜AC-09/AC-NATIVE**：在现有装配model/manifest/contracts/loader及包查询中落实公共声明、默认Auto实现、host/platform/foreign和生成产物分工、schema版本/兼容规则、源码与运行加载边界；保944proof、943操作ID及恢复重授权。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：明确默认core/alloc/基础系统库由Auto自有实现和薄OS适配承担，不以libc或Rust std间接委托作为默认；公共AutoABI与OS/外部ABI分开，Rust/C/provider依赖显式选择。Storage/allocator接口仍待原有M2讨论，不能提前冻结。

- [ ] **T-04｜M2-A｜owner_stage: work｜AC-04**：在scope做File/Task/Stream有限资源合同前探，列实际producer/consumer与成功、失败、close、取消、最后owner状态；核939正式可用性，缺口逐项归owner/后继，绝不宣称资源已交付。

- [ ] **T-05｜M2-A｜owner_stage: work｜AC-04**：补授权/错签名/跨会话资源/预算/NUL/取消和恢复近邻，AV/native分别真调用；折叠前探结论并跑完整职责组。

- [ ] **T-06｜M2-A｜owner_stage: review｜AC-05**：在同一实施树以独立上下文执行 `/auto-plan:review`，逐AC/反例与真实diff核验，具名记录债务候选；实际通过才进入reviewed。

- [ ] **T-07｜M2-A｜owner_stage: merge｜AC-06**：按 `/auto-plan:merge` 核来源/落Spec/归档；证据摘要折入Plan，逐命令实际输出路径对账，删统一scope（具名持有除外），刷新retention；`wt-guard` clean后移除树/分支，实际完成后记账。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-13 stage: work 进度一（M1-TAG 开工 + T-01/T-02/T-03 交付，executing 3/10，非交接完成）。
- **门**：M1-TAG 四要件实测闭合（§4.1：tag v0.6-ACE-M1→b923362f4=HEAD 祖先、1025 H/C 双 exit0、941/999/943/944/960 归档、/goal 即整链授权）；tag C→HEAD 零代码漂移；6 个 r9 SHA 一致。实施树 `D:/autostack/.wt/lang-947/auto-lang` @ plan-947-dev（base 84c575a54）；scope `D:/autostack/plan-artifacts/plan-947/`（OWNER.md 登记）。
- **T-01 ✅**：say≡auto.console.print.str.v1 逐字冻结（io.at:21/io.vm.at:51/expr_calls.rs:1398/natives_io.rs:5-9 四链证据：旧 say 输出后恰一个 \n）；预算/入口/Storage 待定边界记入 §5 T-01 冻结记录。
- **T-02 ✅**：say 拼写接入 console 源域（映射+保留名+专用 str-only 诊断）；**native(mn) console/1 通道**——split_units 搬运 host 声明、admit_console_unit 专用门（普通 lane 863 fail-closed 不变，compat 测试在案）、LowerMode::Console guest 侧格式化（i32 十进制含 MIN/bool/char UTF-8 至 4 字节/str 原字节+恰一 LF）+per-call 65536 预算+单 shim 分发、shim（累计 1MiB+kernel32 GetStdHandle/WriteFile 每调用原子）、每轮共享 av-core authorize（未授权→不发射/不链接/不执行）；发现并修复 4 个实现期缺陷（guard 条件反转、正数取补码、bool LF 偏移、64KiB 栈槽跳守卫页→BSS 暂存）。commit "feat(ace-console): public say binding and the native console/1 lane"。
- **T-03 ✅**：AV persist/restore/重授权既有+保持；mn recipe（ac/mn/console-run-v1，mn-worker 内 persist→restore 重接纳→本轮 caps）落地；TestCase 全链接入（合同解码注册 recipe/mn 绑定、ac_mn/console.rs 父侧判定含 stage 归一/双流字节精确/工具链轴+专用 stale 族）；永久语料 `crates/auto-lang/test/cases/ace-m2/p947-console/`（7 行双 lane：O03 双流、say、错型拒、未授权 dispatch=0、65537 预算拒）+一致性测试（含 3 个期望面变异反例控制：错字节 stdout/stderr、授权对照）。commit "feat(ace-console): ac/mn/console-run-v1 recipe, p947 corpus and TestCase wiring"。
- **测试证据**：ace-native conformance console_lane 9/9 绿；ace-conformance console_p947 2/2 绿（语料 7 行全 Pass+Freshness Fresh）；ace-av say 冻结 lane 测试绿。组并集 `cargo t stdlib ace-console ace-source ace-av ace-native` exit0：stdlib 160/ace-console 5/ace-source 433/ace-av 1067/ace-native 226，合计 executed 1891/failed 0（run-676lsnlw，收据 D:/autostack/plan-artifacts/plan-947/t02-t03-groups.md）。ace-host 全 conformance：163 例中 2 例红——`source::av_source_inventory_denominators_replayable`（examples/ui/015-notes app.at scan-error）与 `rust_lowering::a2r_hir_worker_credential_pin`（pin 不匹配）——**两例均在主检出 master（无本包改动）复现一致，系既有红、与本包 diff 无关**（本包未触 examples/ui 与 a2r worker 面），交周期回归分诊；其余 161 例绿。
- **剩余**：T-08（四端分账表：AV/native=implemented、Rust/C=capability-missing 单列、V-08 已有主体证据）、T-09/T-10（四轴装配合同+stdlib_assembly/packages 落实+V-09/V-10/V-NATIVE）、T-04（File/Task/Stream 前探，939 Handle/Callback 契约为基）、T-05（冻结提交完整组复验）、T-06 review、T-07 merge。
- 注意：master 主检出存在其他 Plan 的 3 个文档 WIP（roadmap/1002/1017），非本包文件未动。

2026-11-13 stage: goal | plan_id: PLAN-947 | plan_revision: r9 | round: 1 | from: work | to: work | status: executing | outcome: pass-so-far（T-01..T-03 完成、组并集运行中） | worktree: D:/autostack/.wt/lang-947/auto-lang | commit: 2 实现提交（say+mn console lane；recipe+corpus+wiring） | next: work（T-08→T-09→T-10→T-04→T-05；组并集结果在 D:/autostack/plan-artifacts/plan-947/ 登记后补记 §9）

2026-11-12 stage:new / PLAN-947 r9：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原10任务/全部AC与V/依赖/阶段/预算和drafting 0/10；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r8：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-10 stage=new / r7：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部10个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r6：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/10；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

本轮实际静态核验：9份文档本地链接/门锚点、四Plan的0–10章节/任务唯一性与total_steps、AC/V引用、drafting/零进度通过；保35个原任务，新增4条时序AC/V，相关可达97个语言Plan节点无环且无缺失前置；空白检查通过。未运行研究/产品测试，未核M1签章或tag通过。

2026-11-09 r5时序与M2定位修订：用户将未执行标准库准备统一置于M1完成并打正式tag之后。本轮改§0/2/5/6/7/8及依赖，保全部原任务/验收，新增AC-10/V-11并入T-01/05和原独立review；无新增任务，进度0/10。stage=new，outcome=blocked-for-work（M1-TAG未核通过，且未获施工授权）；next=M1正式交付tag后核对应授权、来源和阶段依赖。历史“可以提前调查”的安排被本revision撤销，不是当前进入许可。文档静态检查不构成产品或独立实现复审PASS。


2026-11-09 r4范围修订：新增T-09..10、AC-08/AC-09及V-09..10，按Design56正式接M2 FFI/新版标准库责任，0/10。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r3，新增T-08/AC-07/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06 r2 new交接：新增§5明确后继映射；原验收不变，T-05的差额交接绑定正式新计划。status仍drafting，未实施；历史r1判定不是当前状态。


2026-11-05 draft handoff：`stage: new | plan_id: PLAN-947 | plan_revision: 1 | outcome: blocked（正式依赖/首探结论未齐；这是new交接判定，不是Plan状态） | next: 等待用户实施授权及PLAN-943 正式交付（D943）、PLAN-944 正式交付（D944）；先做§8允许的有界准备，不执行依赖未齐的任务。`

本轮只做文档静态预检与固定组名称核验；实施、产品验证、独立final review、Spec沉淀、merge/archive/cleanup全部未执行。current_step=0/7。非完成声明。

实际文档预检：Design54及六份Plan的本地链接、现存源码路径、0–10章节、七任务/六AC、Spec delta映射、drafting/0-of-7及943–948编号唯一性通过；`.next-id=949`。已核所列固定组名；相关跟踪文档`git diff --check`通过。该检查不证明任何拟议功能、产品测试或独立final review已通过。

## 4.1 实施授权与 M1-TAG 门核验（2026-11-13，T-01 开工记录）

用户 `/goal 完成计划947`（auto-plan-goal 链）为本包 work/review/有界修复/merge 整链授权；据 auto-plan-goal 技能，该目标即实施授权（M1-TAG 门第4条）。门核验（Design55 §1.1 四要件，全部实测非推断）：

1. **M1最终组合验收核齐**：PLAN-941、PLAN-999、PLAN-1025 均已归档（docs/plans/archive/ 下实测存在）；1025 为 M1 最终收口——H=40b51f809 正式 `--acceptance` exit0（1006P/0F/0U，ARCH 49/49），C=b923362f47bc8861407e9a470d63ddfdb505982b 实际正式读回 exit0（报告 sha efd1822aa12a 与 H 相同），TAG 六门闭合 tag_ready=true（1025 §8 T-07 收据）。
2. **已合入主线**：C=b923362f4 于 2026-11-13 经 `--no-ff` 合入 master（master 历史实测含 b923362f4、3654f89e8、84c575a54）。
3. **正式 tag 存在**：`v0.6-ACE-M1` 为 annotated tag（tag object acc3eb5b7f50cc043194fb21d993b1a206b3c091，tagger 2026-11-13 08:36:22 +0800，subject "ACE M1 milestone v0.6-ACE-M1"），解析 commit=b923362f47bc8861407e9a470d63ddfdb505982b；`git merge-base --is-ancestor v0.6-ACE-M1 HEAD` 实测通过（当前工作基面 84c575a54 的祖先）。
4. **实施授权 + 本包依赖**：本 /goal 即授权；PLAN-943/944/960 均已归档（实测）。

r9 记录的 6 个取材 SHA256 前16 在当前 master HEAD=84c575a54 逐一重算比对全部一致（model.rs `95fbc7911a149d36`、manifest.rs `ef380114d6a21eb0`、console_host.rs `ce2788ea93e54a1e`、O03.at `cfdd28a727189e42`、ext_policy.at `374a6f4a7ec012ca`、ace-binding-and-assembly.md `6c8c8b068a3c235c`）——r9 规划基线零漂移。实施树：`git worktree add D:/autostack/.wt/lang-947/auto-lang -b plan-947-dev`（base=84c575a54）；材料 scope：`D:/autostack/plan-artifacts/plan-947/`（OWNER.md 已登记）。主检出另有 3 个其他 Plan 的文档 WIP（ace-m2-delivery-roadmap.md/1002/1017），非代码路径、非本包文件，按仓规保留不动。

## 10. 待澄清事项


正式M1 tag名/commit与最终验收合并收据由实际交付填写；本轮未核门、未打tag。具体Storage/分配服务接口在M2实施阶段讨论，不因prelude愿景预定allocator API。

- 首批符号/合法source/profile/接口位置由943及本包T-01有界前探冻结；§5/§6给出了最小关闭条件，缺能力保持具名阻塞，不静默降scope。
- 完整CAP仍守Design46 A/B/C1–C3门；本包无权提前完整求解或以候选字段宣称已实现。
- 实施授权尚未给出；其余未指定预算/模型/角色不虚构约束。跨仓若后续需写代码，由对应owner另建合同与授权。
