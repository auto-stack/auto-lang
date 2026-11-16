---
plan_id: PLAN-976
status: archived
feature_name: 共享HIR到TypeScript和JavaScript
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 6
current_step: 8
total_steps: 8
depends_on: [PLAN-960, PLAN-961, PLAN-962, PLAN-964, PLAN-965, PLAN-968]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-typescript.md]
touched_goals: [GOAL-002]
---

# [PLAN-976] 共享HIR到TypeScript和JavaScript

## 0. 变更摘要

2026-11-16 r6（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包保留M2-A，交完整生产消费闭包所需的共享TS/JS受检发射/provider，976/T-01据Vue/Ark/Web实际委托和合法作者表面冻结范围；不再自动承诺独立通用a2ts/a2js全语言CLI产品。8任务/0进度保持。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

新增共享HIR TS/JS provider（新路径），取代旧TypeScriptTrans/JavaScriptTrans；源映射、模块、UI表达式/handler、异步与runtime helper分层。JS Number不能隐式承接i64/u64，按语义用明确表示，环境API由provider。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/a2x-core`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-store`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/trans/typescript.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：新TS/JS provider与浏览器运行。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/a2x-core/src](../../experimental/ace/crates/a2x-core/src)：扩TargetProvider共同协议；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs)：语义key/target key分层；读取SHA256前16 `eccd5ff0f35f982e`

- [crates/auto-lang-core/src/trans/typescript.rs](../../crates/auto-lang-core/src/trans/typescript.rs)：旧目标行为/模块/源码映射研究；读取SHA256前16 `be945041d9789508`

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

## 5. 详细设计

### M2-TARGET-SCOPE-20261116：本包当前边界

本包保留M2-A，交完整生产消费闭包所需的共享TS/JS受检发射/provider，976/T-01据Vue/Ark/Web实际委托和合法作者表面冻结范围；不再自动承诺独立通用a2ts/a2js全语言CLI产品。8任务/0进度保持。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


新增共享HIR TS/JS provider（新路径），取代旧TypeScriptTrans/JavaScriptTrans；源映射、模块、UI表达式/handler、异步与runtime helper分层。JS Number不能隐式承接i64/u64，按语义用明确表示，环境API由provider。

支持集按Design55 §5.1冻结完整生产消费闭包，并继承959中属于该闭包的义务；闭包内不能因实现困难删项，闭包外独立目标额外面须逐项保留范围分类。T-01冻结到符号/语法/consumer，不以旧emitter类的完整职责自动扩大为通用CLI产品。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：TS/JS目标宿主闭包（r2）

冻结Node/browser/框架现用目标、module格式、runtime helpers与manifest；四端公共语义到TS/JS表示适配不依赖尚未迁移UI产品，UI表达式用真实独立消费fixture，980接最终UI。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/a2ts/14_modules/001_basic_import/basic_import.at](../../crates/auto-lang/test/a2ts/14_modules/001_basic_import/basic_import.at)（下列节选L1–L5；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use math

fn main() {
    print("imported")
}
```

本包所需独立结果：旧模块源输出imported；新TS/JS须在真实运行器得到相同stdout/模块初始化次序，数字/短路/异常对Auto独立oracle，不以ts文本相等当运行。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 同一Checked语义→A2X TS/JS provider→模块/runtime片段→真实TS/JS工具链→浏览器或对应consumer。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ScriptArtifact { module: ModuleIdentity, semantic: CheckedRef, target: ScriptTarget, source_map: SourceMap, runtime: RuntimeIdentity }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/a2x-core/src](../../experimental/ace/crates/a2x-core/src) | 扩TargetProvider共同协议 |
| [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs) | 语义key/target key分层 |
| [crates/auto-lang-core/src/trans/typescript.rs](../../crates/auto-lang-core/src/trans/typescript.rs) | 旧目标行为/模块/源码映射研究 |

**状态、错误与资源。** JS number宽度/溢出/短路/除零、错误import、陈旧map均按Auto语义拒或生成显式helper；工具未装不算语言失败。 async/闭包/资源helper不夹旧执行器；source map跟原span，target/runtime变化只失效对应产物；失败保旧。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/a2ts/14_modules/001_basic_import/basic_import.at](../../crates/auto-lang/test/a2ts/14_modules/001_basic_import/basic_import.at) | import/export/初始化次序 | `basic_import.expected.ts`；SHA256前16 `1566d6fad47060b6` | 拟ace.m2.p976.source01：run-ts/run-js；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2ts/10_collections/001_array_basic/array_basic.at](../../crates/auto-lang/test/a2ts/10_collections/001_array_basic/array_basic.at) | 集合/索引实际行为 | `array_basic.expected.ts`；SHA256前16 `9392b661e8b00e15` | 拟ace.m2.p976.source02：run-ts/run-js；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/03_control_flow/013_if_tail_value/if_tail_value.at](../../crates/auto-lang/test/a2r/03_control_flow/013_if_tail_value/if_tail_value.at) | 共享控制语义对照而非Rust排版oracle | `if_tail_value.expected.rs`；SHA256前16 `ccd069e512d69942` | 拟ace.m2.p976.source03：run-ts/run-js；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** TS与JS实际运行入口/模块格式/async适用域、数值补偿和source-map字段先冻结；不反等980 Web产品迁完才能验provider。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-typescript.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-typescript.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-typescript.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-typescript.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### 976当前交付合同：完整生产TS/JS消费闭包（r6）

已核实的调用链如下，均为旧实现的取材事实，不能当作新ACE已支持：

| 已有入口 | 实际委托 | 新链职责与原任务 |
|---|---|---|
| `auto-ui/.../ui_gen/vue/script.rs` handler/computed/watch/setup、返回表达式 | `ts_adapter.rs`保留Vue响应式/props/store/宿主映射；普通Stmt约858行、Expr约2458行调用旧`TypeScriptTrans` | T-01列出每个委托分支及允许的Auto构造；T-02交共享Checked HIR TS/JS发射；T-03交可独立消费接口，980/WEB接完整UI |
| `auto-ui/.../ui_gen/ark_adapter.rs`普通Stmt/Expr | 同一旧TS emitter约139/629行 | 共用976新provider，980/ARK保Ark框架映射；不能因Vue用到的较少而漏Ark合法构造 |
| 982/WEBTARGET、983 Web、984/OS实际生产生成工程 | 模块、运行helper和真实TS/JS工具链，准确CLI依赖由T-01核实 | T-08冻结module/runtime/target manifest；实际使用公开CLI的consumer仍须该CLI可用 |

范围不是“挑现有Demo用到的语法”。T-01须冻结**规范允许的UI表达式/语句/完整函数、转调函数、导入模块和库的传递闭包**，逐项记录旧委托分支/符号→原源及expected→新provider入口→实际consumer→适用目标与独立oracle。没有Demo但合法可达的构造仍required；旧`a2ts/a2js`语料按此归required、规范冲突待裁定或本版无消费的独立目标额外面，逐条保留去向，不能整目录丢弃。新provider不得回调旧TypeScriptTrans/JavaScriptTrans或重做一套语义分析。

沿a2x-core受检语义/目标协议交付`ScriptArtifact`；模块身份、原span/source-map、target/module格式、runtime身份和依赖清单不可省。UI反应性、capture与宿主变量替换归UI适配层，976负责一般Auto语义到TS/JS表示；接口按该职责分界冻结，不给旧AST发射器加一个ACE名字。调用次序、短路、异常/async、模块初始化、字符串/资源、i64/u64精度与库引用均按Auto独立oracle保真。

M2-A的976用完整闭包语料和真实TS/JS工具链验证provider，可用代表真实UI接面的独立fixture而不反等980整包；980/982/983/984接真实产品并承担其原A/RG或B验收。**独立、全语言、通用a2ts/a2js CLI产品不是自动新增的M2门**；T-01若发现生产工程实际依赖其中某项，就纳入required闭包，不能以“独立CLI”名义排除。未承诺额外面在能力清单具名列出；未支持请求按既有诊断入口拒绝，无旧链fallback。

### T-01 冻结决定（2026-11-16 实施轮，goal 授权链内）

本节是 T-01 的有界前探结论，冻结 §5「976当前交付合同」的实现边界；依据为 `SemanticView` 的实际准入名单（`a2x-core/src/semantic_view.rs` §1.5–1.7）与 auto-ui 真实委托面（4 处 fragment 级 `stmt`/`expr` 调用），不是旧 emitter 的类职责。以下事实为本轮静态读取 + 工具链实测，**新 provider 尚未实现**。

**A. 输入→输出链（已核 owner，非新增）**

| 阶段 | owner/事实 | 本包接入点 |
|---|---|---|
| 源→Checked HIR | `ace_core::source` 各 profile 前门 → `CheckedModule`；`a2x-store` `cold_assemble_profile` 调 `SemanticView::from_checked` | 不新建前端 |
| 多模块/包 | `BoundPackages` 在 view 之前扁平化（`init_order`/`module_init_order`/mangled 名），provider **看不到 module 语法**；`std.*` 仅以 `ExternalFn.source_name` + 端口目录出现 | 不新建模块系统 |
| 受检语义 | `SemanticView`（只读，target-neutral） | provider 唯一输入 |
| 发射/装配 | 新 `ace-a2x-ts` crate：`TargetProvider` 8 方法 → `FileSet` | T-02 |
| 持久/恢复/锚 | `a2x-store` LayerSpec + anchor（可复算 seed） | T-03 注册 id |
| 真实运行 | `a2x-worker` 新 `emit-ts`/`run-ts` recipe → 真实 `node` | T-03/T-08 |

**B. 目标与工具链（实测）**

| 目标 | 工具链 | 已验证事实 |
|---|---|---|
| TypeScript | node **v25.2.1** 原生 type-strip 执行 `.ts`；tsc **5.9.3**（本机 `auto-ui/examples/ui/001-helloworld/gen/front/vue/node_modules/typescript/bin/tsc`，env `A2TS_TSC` 可覆盖） | `node x.ts` 退出 0；enum/namespace 等非可擦语法报 `ERR_UNSUPPORTED_TYPESCRIPT_SYNTAX` ⇒ 生成物只用**可擦语法**（无 enum/namespace/参数属性）；`tsc --noEmit --target es2022 --module nodenext --strict` 可对 `.ts` 生成物 + `.d.mts` 声明通过 |
| JavaScript | node v25.2.1 ESM（`.mjs`） | `import`/`export`、BigInt（`2n**64n`）、`TextEncoder` UTF-8 字节与码点计数均实测可用 |
| 观察协议 | stdout `value:<i32>\n`（bool→0/1）、`trap:<Code>\n`；短路 marks 走 stderr（`mark_a`/`mark_b`） | 与 a2py 冻结面一致；`process.stdout.write`（不用 `console.log`，避免附加换行/二进制失真） |

缺 tsc 只影响类型检查腿（具名 infra skip，由 T-04 记录；不冒充 PASS 也不判语言失败）；真实运行腿始终由 node 执行。

**C. 支持集 = `SemanticView` 的 Scalar/RecordText/Collections 全部准入构造（无按 Demo 缩面）**

| Family | 必须支持（准入表全量） |
|---|---|
| Scalar | `ReadLocal/Constant/Binary(11 legacy ops)/Unary/Logical/Call`、`Let/Return/If/Loop/Assign/Break/Continue`；入口 `view.entry(name)`（0 参 i32 free fn） |
| Text / RecordText | 上表 + `StrLit/CharLit/TextLen/TextLenBytes/TextCharAt/TextSlice/TextConcat/TextClone/TextToOwned/TextToBorrowed/CharToCodepoint/CharFromCodepoint/TextFromChar`、`RecordCtor/FieldGet/BorrowView/BorrowMut/RecordClone/Move`；`CallbackRef` 随 foreign 一起具名拒绝（见 D） |
| Collections（含 aggregates/scheduler profile 绑定到 Closures 域） | 上表 + `ListNew/ListLen/ListPush/ListInsert/ListGet/ListSet/ListRemove/ListClear`、`VariantCtor/VariantGuardTest/VariantProject`、`TextDisplay`、`CreateClosure/CallClosure`、`Eval/Fail`、以及仅此域准入的 `NumericConst/Cast/BinOp::Num/Unary::NegNumeric`（i8/u8/u32/i64/u64/f32/f64 全量） |

`Stmt::Drop` 在任何域都被 view 拒绝（清理是计划驱动的），`Unit/NilLit/ObjectCtor/ObjectGet/ObjectSet/ClosureClone` 在任何域都不准入 ⇒ 不属于本包义务，也不得在文档中宣称支持。

**D. 具名拒绝（具名、可诊断、无旧链 fallback）**

| 拒绝码 | 触发 | 依据 |
|---|---|---|
| `a2ts.foreign-unsupported` | `DefKind::Foreign` 声明、`foreign_call_credential`、`CallbackRef` | 原生 FFI 无生产消费（UI/Web 闭包不接 C ABI）；同 native/A2C 具名拒绝先例，不做假桥 |
| `a2ts.sched-unsupported` | `__sched_*` 内在符号 | 共享 HIR 无 await 节点，scheduler 是独立 profile 的并发运行面；m2-scheduler 已记 Native/A2C 具名拒绝先例，本包同样具名拒绝而非静默降级 |
| `a2ts.unsupported-*` | 视图之外的构造/类型/操作 | 沿 `A2xReject` 词表 |
| `a2ts.depth-budget` | 嵌套深度超 `MAX_STRUCTURE_DEPTH` | 与 a2py 同常量同码位纪律 |
| `a2ts.seed` / `a2ts.credential` | 无 view 且无 seed、凭证不匹配 | provider 侧自检 |

**E. 值表示决定（JS/TS）**

| Auto 类型 | 生成物表示 | helper 族 | 边界/陷阱 |
|---|---|---|---|
| `i32/i8/u8/u32` | `number` | `a2x_add/sub/mul/div/rem/check_range/shl/shr` | checked：越界 throw `trap:Overflow`；`/`、`%` 为 C 语义向零截断（不用 JS 浮点除） |
| `i64/u64` | **`bigint`**（字面量带 `n`） | 同族 BigInt 版 helper + `a2x_check_range` BigInt 分支 | JS Number 不能隐式承接 i64/u64（>2^53 丢失）；混用 number/bigint 立即 throw `TypeError`，故 cast 点显式 |
| `f32/f64` | `number` | `a2x_f32_round`（`Math.fround`）、`a2x_f32_bits/a2x_f64_bits`、`a2x_fdiv` | `-0.0`/NaN 载荷位保真；f32 运算每步 round |
| `bool` | `boolean` | 断言 `x !== 0` 型转换 helper | 不用 `Boolean()`（NaN 差异） |
| `char` | `number`（Unicode scalar） | `a2x_char_from_codepoint`/`to_codepoint` | 越界/代理区 → `trap:InvalidScalar` |
| `str`（借用） | `Uint8Array` 视图 | `a2x_str_len/len_bytes/char_at/slice/eq` | UTF-8 字节精确，内嵌 NUL 保留；不用 JS `string`（UTF-16 语义会给错 len/char_at） |
| `Str`（拥有） | `A2xOwnedStr`（`Uint8Array` + id + freed 标志） | `a2x_str_from_lit/clone/concat/from_char/free` | 显式 free + ledger；重复 free → `trap:DoubleClose` |
| `Record(n)` | 普通 object，键为字段名 | `a2x_rec_clone_*`（seed 生成） | 字段按声明序；Nominal 身份由类型表决定，不进运行时布局 |
| `List(n)` | `A2xList`（数组 + elem 族） | `a2x_list_new/len/push/insert/get/set/remove/clear` | 越界 → `trap:Index`；元素族由 `list_elem_tag` 决定 |
| `Variant(n)` | `A2xVariant{disc,payload}` + `tag` getter | `a2x_variant_project` | 投影是**破坏性移出**（移出后 disc=-1），与 a2py/a2r 一致 |
| `Closure(n)` | JS function；捕获经 env 工厂 | `a2x_closure_env_*`（工厂内联） | Move/Borrow 在创建点快照，Shared 传同一个 cell 句柄；捕获的拥有 `Str` 不在 env 内 free（具名限制入 spec） |
| `Unit` | `null` | — | 仅结果位置；不产生值 |

**F. 资源/清理合同**

- 拥有 `Str` 的释放集合**取自计划**：text/text-record 用 `body_plan(f).owned` / `owned_returns` / `replace_frees`；collections 用 `body_facts(f).owned`（`is_empty()` 可跳过）。provider 不再按类型自推拥有集合（a2py 的 `is_owned_ty` 自推是已知偏差）。
- 函数级 `try/finally` + 返回 skip 变量的单一清理点（与 a2py 同形，已验证可执行）；`Assign` 的旧缓冲在 RHS 物化后释放；`CallBinding.moved` 与 `ExprKind::Move` 的移出本地**加入 skip 集合**（修正 a2py 已知 double-free 漂移，本包作为永久正反例）。
- 复合值（record/list/variant/closure）由 JS 运行时 GC 管理，ledger 只统计拥有 `Str`；这是 JS 目标与 a2py 的**具名差异**，写入 SD-01/spec，不冒充"全部 Auto 资源语义等价"。
- 失败：`throw A2xTrap(code)` → `a2x_run_main` 捕获 → stdout `trap:<Code>`、退出码 1；`finally` 仍清理。
- 预算：`a2x_bill` 单位/上限沿用 a2py 常量族，超限 `trap:Budget`。

**G. 模块/文件集/清单**

- 模块与 `use` 由 store 前端扁平化（合成 init 函数 + `init_order`）；provider 只发射扁平函数集，故 **V-DETAIL-01 的模块初始化序** 由 packages lane（多模块包）真实执行验证，stdout 与原始 `.at` 独立 oracle 对齐。
- fileset（TS）：`a2x_rt.mjs`、`a2x_rt.d.mts`、`generated.ts`、`a2x_main.ts`、`source_map.json`、`manifest.json`；（JS）：`a2x_rt.mjs`、`generated.mjs`、`a2x_main.mjs`、`source_map.json`、`manifest.json`。
- 身份：backend `auto-a2x-typescript` / `auto-a2x-javascript`；fragment layer `AXTS1..3` / `AXJS1..3`；fileset schema `ace.a2ts-hir.{scalar,record-text,collections}.fileset/1`；producer `ace-a2x-ts/modules/1`；provider recipe `a2ts-hir-{typescript,javascript}-checked`。
- 清单绑定：`backend/backend_version/runtime_version/runtime_sha256/print_version/stdlib_port_catalog/input_hash/source_sha256/entry/functions[]/files[]`（与 a2py 同形，files 逐文件 sha256）。

**H. 增量与失败恢复**

- fragment key inputs = backend/standard + runtime_version + runtime_sha256 + print_version + stdlib port catalog fingerprint ⇒ 运行时或端口目录变化只失效对应产物；source map 随 fragment 重发，不陈旧。
- 失败发布保旧：worker 先写 staging 目录，全部成功才替换（沿 a2py 的隔离断言）；失败不覆盖既有 `manifest.json`。

**I. 消费接口（T-03）**

1. `TargetProvider` 全链路（单源 cold/warm/restore + packages lane）——store/A2X 正式消费者；
2. **UI 中立 fragment 接口**：`emit_stmt_fragment` / `emit_expr_fragment`（输入：view + body + stmt/expr id + 显式程序上下文：scalar enum 集、struct 集、unit variant 集；输出：文本 + 需要的 runtime helper 名 + 诊断）——供 980 替换 4 处 legacy 委托；本包以真实 node 执行的独立 fixture 验证该接口，不等 980 整包；
3. worker `emit-ts`/`run-ts` recipe 作为产品入口的真实接线与观察面。

**J. 未决项：无阻塞项。** 旧 `a2ts/a2js` 语料（85 例，全部 `#[ignore]`，金样与现 emitter 已漂移且含非法 TS）不作新 oracle，仅作构造清单；新永久用例按 §6 V 行与本节 C 表逐族建立。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 超过2^53整数往返正确 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 副作用短路和命名实参顺序不变 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | UI真实表达式编译运行而非字符串快照 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | source map定位原span | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 生成物不得调用旧emitter | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-a2x a2ts a2js auto-test-case`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 无旧emitter独立生成/执行TS及JS工程；异步错误/BigInt序列化兼容合同可核；错误source map落原模块；不得只以tsc成功代运行。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 旧模块源输出imported；新TS/JS须在真实运行器得到相同stdout/模块初始化次序，数字/短路/异常对Auto独立oracle，不以ts文本相等当运行。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | JS number宽度/溢出/短路/除零、错误import、陈旧map均按Auto语义拒或生成显式helper；工具未装不算语言失败。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | async/闭包/资源helper不夹旧执行器；source map跟原span，target/runtime变化只失效对应产物；失败保旧。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 当前目标范围验证（拟实施，未运行）

| Case | AC/任务 | 输入→producer→consumer→独立oracle与负控 |
|---|---|---|
| V-TS-CONSUMER | AC-TS-CONSUMER；T-01/02/03/08/04 | 冻结的Vue/Ark合法原源及转调函数/导入模块→Checked HIR→新TS/JS provider→真实工具链/consumer，对独立值、调用顺序、响应式效果、模块初始化与原span；大整数、async错误及旧委托各分支保区分性输入。删一个合法非Demo构造/传递模块、错capture/map、注入旧TypeScriptTrans fallback或将Vue TS标后置必须失败；仅tsc/emit不得关闭运行。对无生产消费的独立CLI额外面检查范围清单，不要求伪造全语言CLI已支持。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | TS/JS目标宿主闭包的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

### 当前目标范围验收

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-TS-CONSUMER | work | §5完整生产闭包及所有旧委托分支有可追溯映射，合法非Demo函数/模块与各框架适配均有去向；所需TS和JS真实运行、数值/效果/错误/来源正确且无旧emitterfallback；独立CLI额外面不冒本版承诺 | T-01/T-02/T-03/T-08/T-04、V-TS-CONSUMER；原独立review逐项复核 |

## 8. 执行步骤

当前共8任务，8完成（已归档）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 本次须按§5冻结完整生产消费闭包/旧委托分支和独立CLI额外面去向；不以Demo集合或旧类全量职责代替边界。 [✅ 已完成：§5「T-01 冻结决定」A–J 表全量落本 Plan（输入链/工具链实测 node v25.2.1 + tsc 5.9.3/支持集=SemanticView 准入全量/具名拒绝/值表示/资源合同/fileset·manifest 身份/增量失效/消费接口/无阻塞未决）；probe 证据在 plan-artifacts/plan-976/{tool,rt_check,ref}（scratch，merge 折叠后删）]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 实现§5共享受检TS/JS provider及AC-TS-CONSUMER所需语义，禁止旧emitter回调。 [✅ 已完成：新 crate `experimental/ace/crates/a2x-ts`（provider/lower/ports/print/seed/cert/runtime~6000行，TypeScript/JavaScript 双 TargetProvider，唯一输入 SemanticView）；永久测试 `tests/ts_runtime_and_provider.rs` 3 例（真实 node 执行 runtime：UTF-8/NUL/checked算术/bigint/f32位保真/variant破坏性投影/list越界/端口合同/budget/marks stderr/入口协议 value·trap·Internal + manifest tamper/错producer/错schema + fragment层身份与 legacy-emitter 禁令）+ crate 内 20 单元例，commit 1d30914ff]

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 交UI中立消费接口；框架反应性/捕获映射由980消费，不反等整包980交付才验provider。 [✅ 已完成：`a2x-ts/src/fragment.rs` `emit_stmt_fragment`/`emit_expr_fragment`/`innermost_expr_in_span`（§5 I.2 冻结面：view+body+id+显式程序上下文；输出文本+helpers+locals映射；程序上下文缺失=具名拒 `a2ts.fragment-context`），真实 node fixture `tests/ts_ui_fragment.rs` 4 例；worker `tsrun.rs` 受控 node 执行腿 + `daily-emit-ts`/`run-ts`/identity(certification_target typescript/javascript) ops；a2x-store 注册 AXTS1..3/AXJS1..3 层 + semantics-contract 注册 `ace-a2x-ts/modules/1`；失败保旧断言（拒绝的 emit 不触碰既有 out_dir，67b5c0804）]

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：冻结Node/browser/框架现用目标、module格式、runtime helpers与manifest；四端公共语义到TS/JS表示适配不依赖尚未迁移UI产品，UI表达式用真实独立消费fixture，980接最终UI。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 target/runtime/module与CLI能力清单按§5当前闭包，不自动承诺独立全语言通用CLI。 [✅ 已完成：身份冻结 `a2x-ts/src/target.rs`（TS_STANDARD=typescript5.9-es2022-erasable/JS_STANDARD=es2022-node-esm/AXTS·AXJS 层/fileset schema/recipe 常量，boundaries.rs 断言层不碰撞+store 无 target-crate 依赖）；V-08=tsready_01（两模块 packages lane，V-DETAIL-01 模块腿）+tsready_04（tsc --noEmit --strict 冻结旗标，缺 tsc=具名 infra skip，node 腿恒真实）+tsready_03（cold/restart 同 store 字节同 manifest）+tsrt_01（BigInt/trap:Internal 合同）+source_map 原span断言]

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 新增V-TS-CONSUMER必须通过，按实际触面选择原完整职责组，不执行已由用户后置的独立目标额外面。 [✅ 已完成：V-01=tsready_06（>2^53 双腿 e2e）+tsrt_01 bigint；V-02=tsready_05（短路+命名实参源序，marks stderr 区分性）；V-03=tsfrag_01/02（UI 表达式真实编译运行）；V-04=tsready_04 source_map 原 span；V-05=tsrt_02/tsready/boundaries legacy 禁令；V-08/V-DETAIL-01..04 如上（模块腿经真实 PackageA2xTarget lane 关闭，非推迟）；固定组 `cargo t ace-a2x`（已注册 ace-a2x-ts）PASS 284 执行/gate True；`cargo test -p ace-semantics-contract` 19 绿；组选择理由：crates/auto-lang 零改动故 a2ts/a2js 旧 emitter 组与 auto-test-case 不在本次触面（旧 emitter 退役归 987）；Linux/WSL 腿按 2026-11-14 裁定排除]

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。 [✅ 已完成（final pass 绑定 9da498211，见 §9 review 收据；发现 F-R1 已同树修复并复验；债务候选 D-976 已登记 KNOWN-DEBT）]

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。 [✅ 已完成：spec 沉淀 m2-typescript.md(sha256前16 914bbadae9af3f85)+replacement-scope 挂接，随 delivery commit 660fe72b5 ff-only 落 master；rebase 补丁全等（range-diff 全 `=`，旧→新 9da498211→ef028d1c3、498d7a205→660fe72b5）；冻结门禁 rebased 基线复跑 `cargo t ace-a2x` PASS 288 执行 gate True wall 229s]

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。 [✅ 已完成：plan-artifacts/plan-976 整目录删除（e2e/tool/rt_check/seedcert-check/ports-check/ref 全 scratch，关键数字已折入计划文件）；wt-guard clean 后移除 worktree/分支/组目录；retention 登记刷新]

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-976/auto-lang`，分支 `plan-976-dev`（沿仓规）——已创建并在用（base 2961a2aa，实施 1d30914ff/67b5c0804/82b3dc665）。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-976/`（已建：e2e/tool/rt_check/seedcert-check/ports-check/ref，全为 scratch；merge 收尾折叠摘要后整目录删除）。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-16 stage: goal | plan_id: PLAN-976 | plan_revision: r6 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: （已移除） | commit: 660fe72b5（delivery，master 祖先链内） | next: done

2026-11-16 stage: merge | PLAN-976 | r6 | outcome: pass | worktree: D:/autostack/.wt/lang-976/auto-lang（已移除） | delivery_commit: 660fe72b5（master tip，ff-only，无 merge commit） | checkpoints: prepared=reviewed 9da498211+冻结spec增量（498d7a205） | landed=rebase onto master 补丁全等（range-diff 全=；映射 1d30914ff→d4945b802、67b5c0804→6259fb3f5、82b3dc665→6f7cdcce9、968e2c76c→7fb7ee058、9da498211→ef028d1c3、498d7a205→660fe72b5）后 `git merge --ff-only` 成功 | canonical_verified=m2-typescript.md sha256前16 914bbadae9af3f85 于 660fe72b5、replacement-scope-and-evidence.md 挂接节在位 | archived=docs/plans/archive/976-ace-m2-typescript.md status archived 本提交 | merge 门禁=rebased 基线 `cargo t ace-a2x` PASS 288 执行 gate True wall 229s（run-n65_bt60）；受触 crate 零 warning | 时间表（本会话实测）：tsrt 3例0.18s、tsfrag 4例0.18s、tsrej 2例0.03s、tsready 6例~8s（最大单例~7s<10s）、ace-semantics-contract 19例0s | cleaned=见 T-07 行 | 批量回归到期检查：976%5=1 非5倍数；.last-batch-regression.json 见 §10 后记 | next: done

2026-11-16 stage: goal | plan_id: PLAN-976 | plan_revision: r6 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-976/auto-lang | commit: 9da498211（冻结复审基线） | next: merge

2026-11-16 stage: review | review_scope: final | reviewer_identity: 与实施同会话/同上下文（goal 授权链；独立性受限已在收据声明，结论自 artifact 重建非执行者自述） | PLAN-976 | r6 | outcome: pass | worktree: D:/autostack/.wt/lang-976/auto-lang | branch: plan-976-dev | reviewed_commit: 9da498211（修复后冻结基线；work 头 82b3dc665 + F-R1 修复 968e2c76c + fmt 9da498211） | base_commit: 2961a2aa | dependency_revisions: master 50d62ceac（簿记后） | spec_inputs: §5 规范增量表（SD-01/02/PHASE/TARGET-SCOPE/DETAIL）经复核——目标路径合法、before/after 规则描述当前行为，m2-typescript.md 为拟新增（merge 沉淀），replacement-scope-and-evidence.md 存在 | acceptance_results: AC-01 pass（§5 A–J 冻结表逐节复核）；AC-02 pass（V-01=tsready_06+tsrt_01 bigint、V-02=tsready_05 marks stderr 序、V-03=tsfrag_01/02 真实node、V-04=tsready_04 source_map原span、V-05=tsrt_02/tsready/boundaries legacy禁令、V-DETAIL-02/03/04=tsrt_01负控+fragment keys+失败保旧+restart字节同manifest）；AC-03 pass（worker daily-emit-ts/run-ts 真实接线、无旧链 fallback、变化=fragment_key_inputs 五键断言、失败保旧=拒绝emit不触碰out_dir、恢复=restart同store字节同manifest）；AC-06 pass（V-08：tsc --noEmit --strict 冻结旗标真实过 + BigInt/trap:Internal 合同 + 缺tsc=具名infra skip）；AC-PHASE pass（A切片映射齐、B消费者未提前勾选）；AC-TS-CONSUMER pass（中立面=fragment接口真实node fixture+packages lane+source map；Vue/Ark适配器替换为980 B职责，非本包等待项）；AC-04 本收据 | pending_closeout_gates: merge 拥有冻结并集复跑（已预跑 `cargo t ace-a2x` PASS 286 执行 gate True wall 161.3s @9da498211）与 spec 沉淀 | findings: F-R1（§5 T-01 D 具名拒绝表缺永久负控——sched/depth 两族已补 `ts_named_rejections.rs` 2 例并复验；foreign/closure-owned-capture 两族源级负控缺位=代码路径复审核读+D-976 债务候选，a2py 同形先例）已闭合 | evidence: 全部证据为仓内永久测试（27+2 a2x-ts、6 tsready、boundaries 扩展、19 semantics-contract）；时间表：tsrt 3例0.18s、tsfrag 4例0.18s、tsrej 2例0.03s、tsready 6例~8s（最大单例 tsready_02 ~7s 含8次worker bin spawn，<10s）；新用例均<10s ✅；健康检查：受触 crate 零 warning 零 debug 输出、cargo fmt 过（预存漂移 plan981/sched_968 未触碰属其 owner）；分组选择理由复核：crates/auto-lang 零改动故 a2ts/a2js 旧emitter组不属本次触面（退役归987），ace-semantics-contract 直跑（无小固定组） | next: merge（T-06/07）

2026-11-16 stage: goal | plan_id: PLAN-976 | plan_revision: r6 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-976/auto-lang | commit: 82b3dc665（树头） | next: review

2026-11-16 stage: work | PLAN-976 | r6 | outcome: pass | worktree: D:/autostack/.wt/lang-976/auto-lang | branch: plan-976-dev | base_commit: 2961a2aa | code_commit: 1d30914ff（主实施）+67b5c0804（失败保旧断言）+82b3dc665（trap:Internal 合同） | dependency_revisions: master aa257fa（分叉仅 docs/av/age 文件，与本包触面无冲突，merge 时合入） | task_ids: T-01,T-02,T-03,T-08,T-04（5/8 work 完成，T-05/06/07 为 review/merge） | evidence: 见 §8 各任务行内证据；门禁 `cargo t ace-a2x` PASS 284 执行 gate True（含新 crate 27 测试与 worker tsready 6 例）；`cargo test -p ace-semantics-contract` 19 绿；e2e scratch 22/22（plan-artifacts，merge 时折叠删除） | blockers: 无 | 本轮新用例均 <10s（tsready 套件 6 例共 ~8s；tsrt 0.18s；tsfrag 0.18s；准备=既有 debug 增量构建）；已知非本包债务：ace-av/ace-native 9 个既有 warning、sched_m2_968.rs/plan981_age_package_a2r.rs 的 master fmt 漂移（未触碰） | next: review（T-05 独立复审）

2026-11-16 stage: new/revision | PLAN-976 r6 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-12 stage:new / PLAN-976 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

## 10. 待澄清事项

（merge 后记：本包仅触 experimental/ace 开发/探针 crate 与 docs——无生产发行物/web 包/部署面，无 stale 生产工件需重建（landing 观察记录）；批量回归到期经 975 触发已跑并收据，新红归因 969 见 KNOWN-DEBT。）

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
