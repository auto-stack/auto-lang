---
plan_id: PLAN-964
status: archived
feature_name: 动态值对象反射与通用闭包
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-14
plan_revision: 6
current_step: 8
total_steps: 8
depends_on: [PLAN-962, PLAN-963]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-dynamic.md]
touched_goals: [GOAL-002]
---

# [PLAN-964] 动态值对象反射与通用闭包

## 0. 变更摘要

2026-11-13 r6：按用户对PLAN-1026的定向文档复查，细化本包通用外部值/可调用值的INT-1026接口交接及独立验证，明确不持PyO3、不自建宿主资源账本、不反等1026/Python provider。新增AC-INT-1026/V-INT-1026并分配给既有T-01/02/03/08/04，任务数8、状态/进度/原完整义务保持；仅改文档，不施工。


2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

明确Dynamic与推导失败分开，接实际对象/字典、nil/null、可调用值、通用闭包捕获、反射与配置值；运行时类型标签和共享/循环资源策略必须先由T-01冻结。四端允许使用中立boxed runtime，不得重新引入AutoVM执行循环。

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
- `crates/auto-val/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/core/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/av-core/src`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-core/src`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：动态对象/反射/可调用值与通用闭包。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs)：动态值/捕获绑定和源诊断；读取SHA256前16 `24441ead81f35f8f`

- [experimental/ace/crates/av-core/src/dynamic.rs](../../experimental/ace/crates/av-core/src/dynamic.rs)：复用DynType/DynValue等表示，不包装旧VM；读取SHA256前16 `b1081fd60d4c0a12`

- [experimental/ace/crates/av-core/src/closures.rs](../../experimental/ace/crates/av-core/src/closures.rs)：CaptureMode/EnvLimits与环境事务；读取SHA256前16 `0e3ddb3de244cb9d`

### 2026-11-13 goal 授权记录

用户指令"完成计划964"（/auto-plan-goal 目标）授权整链：work（含建专属树）、review、合同内有界修复、merge（规范沉淀、归档、清树、落地核验）。授权范围不改变目标/验收/预算/仓库集；不授权绕过worktree规约、证据政策或阶段停止条件。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


明确Dynamic与推导失败分开，接实际对象/字典、nil/null、可调用值、通用闭包捕获、反射与配置值；运行时类型标签和共享/循环资源策略必须先由T-01冻结。四端允许使用中立boxed runtime，不得重新引入AutoVM执行循环。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：共享动态资源与显示转换（r2）

明确Arc/共享可变/弱引用等实际已用形式及循环回收，不把Box递归与共享循环混为一谈；FStr的表达式求值和插值归本包，格式化算法归972。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at](../../crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at)（下列节选L4–L8；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
var obj = { x: 100 }
obj["y"] = 200
var k = "z"
obj[k] = 300
obj["y"] + obj[k] + obj["x"]
```

本包所需独立结果：旧对象写y=200/z=300/x=100后读和600；闭包factor=3映射[1,2,3]新例须核[3,6,9]而非只len3；动态类型标签与推导失败不同。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 对象/闭包普通源→共有类型/捕获事实→DynValue/环境所有权→共同MIR或A2X→独立字段/调用/资源观察。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
enum DynamicValue { Nil, Bool(bool), Scalar(ScalarValue), Text(TextRef), Object(ObjectRef), Callable(ClosureRef) }
struct CaptureBinding { source: DeclId, mode: CaptureMode, owner: OwnerId }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | 动态值/捕获绑定和源诊断 |
| [experimental/ace/crates/av-core/src/dynamic.rs](../../experimental/ace/crates/av-core/src/dynamic.rs) | 复用DynType/DynValue等表示，不包装旧VM |
| [experimental/ace/crates/av-core/src/closures.rs](../../experimental/ace/crates/av-core/src/closures.rs) | CaptureMode/EnvLimits与环境事务 |

**状态、错误与资源。** 缺字段按既有nil/默认值/拒绝合同冻结，不一律改成异常；不可调用值、错tag/反射越权与坏捕获生命周期正确拒，不能遇推导失败就Any。 共享可变/弱引用和循环策略具名；捕获环境失败保旧、最后owner释放；FStr求值原次序，闭包不夹旧解释循环。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at](../../crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at) | 对象/map写读600 | `map_bracket_write.expected.result`；SHA256前16 `9742083e77934ae0` | 拟ace.m2.p964.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at](../../crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at) | 捕获+HOF，旧len断言补完整值 | `closure_capture_hof.expected.out`；SHA256前16 `3b3eb1e9ecdbeb6a` | 拟ace.m2.p964.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/04_strings/001_fstring/fstring.at](../../crates/auto-lang/test/a2r/04_strings/001_fstring/fstring.at) | 插值表达式归此，格式算法归972 | `fstring.expected.rs`；SHA256前16 `5e2f064d4d5cf6c7` | 拟ace.m2.p964.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** Nil/null、object字典/反射域、共享循环策略与具体捕获borrow/move行为先按旧规范裁定；值类型集合扩展版本，不重释已交有限面。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### T-01 有界前探与冻结决定（2026-11-13 实施轮，goal 授权链内）

以下为 T-01 冻结：支持集、符号/入口、依赖、owner、资源/循环策略与真实命令。基于对
master @cb1a185ea 的实际源码静态核对（非运行证明）与四端测绘（对象/闭包/FStr/foreign
契约各一份探索报告）。**当前实现从零开始，本节是实施目标，不是已观察 PASS。**

**支持集与入口。** 本包 M2-A 切片挂 `aggregates`（对象/索引/nil）与 `closures`（闭包×
对象）profile；编译入口 `ace_core::source::compile_aggregates_source` /
`compile_closures_source`；四端固定组 `cargo t ace-source ace-semantics ace-av ace-native
ace-a2x ace-runtime auto-val`。FStr 由编译前门文本变换（`f"…"` → `__fstr(…)` 链）承载，
插值段由主解析器解析（无独立 f-string token），表达式段源序求值恰一次（TextConcat 链
左结合保证），显示转换 I32→十进制归本包，其余类型/格式算法归 972。

**对象（L-16/L-20）。** `Value::Object(Rc<RefCell<ObjectData>>)`（有序 key→Value 映射，
插入序）；`MirType::Object{ elem }`；MIR `ObjectNew`（keys/vals 操作数对）、`ObjectGet`
（缺失键运行期 trap `av.dyn.object-missing`——缺字段按"拒绝"合同冻结，V-04）、
`ObjectSet`（upsert，旧值 drop 恰一次）。源类型 `VT::Object(elem)`，字面量字段同型冻结，
异型 `source.object-elem` 具名拒。对象是共享可变单元：`let o2 = o1` 是 Rc 别名（共享），
写经任一名可见；捕获对象 = Move（Rc bump 即共享），两闭包共享更新可见；**循环策略：
自插/可达同 cell 的插入在 `ObjectSet` 以 `Rc::ptr_eq` 确定性拒（`av.dyn.object-cycle`），
不建 GC；共享 cell 最后 owner 释放（Rc），无环即无泄漏**（V-05/V-08）。四端：AV 执行；
native/a2r/a2c 按原支持域 = `mn.object-unsupported` 具名拒（负控，沿 native 拒 DYN 先例）。

**nil/null（L-21）。** `nil` 与 `null` 均为 Nil 字面量（PLAN-053 统一）；`Value::Nil`
（零宽，与 Unit 区分）；`MirType::Nil`；`VT::Nil`；`DynType::Nil`。let 可绑定；对象
elem=Nil 可存读；`==`/`!=` 仅 Nil 对（恒真/恒假，常数化降低）；其余算子
`source.nil-op` 具名拒。缺字段读按上条 trap（拒绝合同），不静默改异常。

**可调用值与动态派发（L-22/L-23）。** `DynType::Callable`：pack 收 `Value::Closure`
（仅声明 Callable 时放宽白名单）；对象 elem=Closure 可持闭包，
`obj["twice"](3)` = ObjectGet→闭包→CallClosure（callable/member/index 派发）。命名
顶层函数非一值对象（L-23 裁定：M2-A 不支持函数项为值，`source.fn-as-value` 具名拒，
延后续）。

**类型标签 vs 推导失败（V-01）。** DynType 标签 = 值实际类别（`describe()`），错 tag
take/is 为运行期 trap/Err 携 expected/actual 标签；源级推导失败 = 具名源诊断
（`source.object-elem`/`source.nil-op` 等），两者不同态、不互相冒充。

**INT-1026/IF-VAL（V-INT-1026）。** `DynType::External{ provider, kind }` +
`Value::External(Rc<RefCell<ExternalCell>>)`：外部值/可调用值中立载体（不持 PyO3、不建
第二资源账本、不依赖 1026/967 runtime）；retain=clone（Rc 增 owner）、drop=最后 owner
释放钩子恰一次（独立 provider 测试见证）；别名/参数返回/分支/循环传引用保身份；
错 kind 拒（DynTakeError 标签）；活外部值**不可** portable freeze（`av.dyn.unsupported`
具名拒）；原生近邻（普通 i32 相邻 pack/borrow 不受影响）。provider 为测试内独立
owned-resource/callable 提供者，无 Python/torch。

**通用闭包（source02）。** `closure_capture_hof.at` 按原支持域执行：factor=3 捕获 +
HOF，新例核 `[3,6,9]`（非只 len）；AV/native 双端（Move 快照捕获）；a2r/a2c 无闭包
emit = 不在域。List `map` HOF 不在 962 集合面——以显式循环+闭包调用替代（具名裁定，
不伪造旧覆盖）。

**依赖与 owner。** 962/963 已归档交付；967/1026 仅中立契约消费（`ace-foreign-contract`
`ResKind::Handle`/`ResourceSig` 事实形状），无反向运行依赖。owner：T-02 核心表示与
正反例（av-core/core/lowering）、T-03 四端接线（native/a2x/av session）、T-08 共享/
循环/FStr/外部见证、T-04 全量运行与组注册。材料目录 `D:/autostack/plan-artifacts/plan-964/`
（owner 964，本实施轮在途）。

### INT-1026：通用外部值与Python consumer的接口交接（2026-11-13，r6）

消费方：[PLAN-1026 r3 §5.4..6](1026-python-interop-standard-auto-syntax.md)。本节是**待实施的通用接口合同**，不声明Python支持已交。以IF-VAL为双方交接ID，本包T-01冻结真实符号/版本与kind/owner生命周期，T-02/03/08实施、T-04验证，不新开任务或缩减原动态/闭包/共享循环范围。

- 本包拥有通用值类别、callable/member/index派发、外部引用在别名/参数返回/分支/循环中的传递，以及显式Dynamic与推导失败区分。外部kind与引用cleanup钩子消费现有中立foreign-contract事实，不能反向依赖967 runtime/PyO3或1026适配器包。
- 外部资源的session/provider/entry generation、live/pin、授权/析构权威归967统一资源账本；本包仅用中立引用与retain/drop协议，不另建lease表，不把token解释成guest裸int或把Python数值对象自动拆掉。
- 反射派发只依赖实际值类别和受管操作；Python属性/callable检查由1026 adapter按需执行。本包不为torch/type_name/shape建立专用IR、镜像成员表或安全判据。类型与操作/效果事实为共享Meta，可变第三方属性不是静态事实。
- 外部引用包装进Dynamic不使之可portable freeze；不能将旧owned-tree白名单扩成含宿主活对象。冻结需具名Unsupported；普通Auto值原合同保持。
- 通用接口可用独立owned-resource/callable provider验证（单一owner下双别名共享修改、函数/分支后身份保留、最后drop、错kind拒绝、原生近邻）。provider不需要Python/torch或1026。967中立事实的实际版本缺失先登记差额，不为了“复用”加964→967运行实现→964的环。

交接收据须含正式源码SHA/接口版本、真实producer/consumer和独立值/资源trace；只有描述符或WIP不可交。1026仅消费正式切片，不以其Tensor训练通过替代本包完整共享循环/闭包/四端义务。此条不新增本包对1026的depends_on。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-dynamic.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-dynamic.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-INT-1026 | modify | `docs/specs/ace-migration/m2-dynamic.md`（拟交付后沉淀） | 通用值边界→实际已验外部kind/值流/cleanup钩子与不冻结规则；Python Meta不授信 | 单一共享值/资源权威 | AC-INT-1026 |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-dynamic.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 同内容不同类型标签不可混用 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 闭包越出定义作用域仍可调用，最后owner释放 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 捕获共享状态按原合同可见 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 缺字段/错误cast稳定报错 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 动态循环资源以确定owner策略回收 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-INT-1026 | AC-INT-1026 / IF-VAL | 独立非Python provider的外部值/可调用值经两别名、函数参数返回、分支/循环；一方修改另一方可见、最后drop一次；错kind/活对象freeze及普通Auto近邻 | T-01冻结实际入口，T-02/03/08接真实值流与资源钩子，T-04实际运行并核独立provider trace；不使用1026/PyO3作为前置 | work；共享类型/派发/所有权合同，不证明Python运行 |

完整职责组拟定：`cargo t ace-source ace-semantics ace-av ace-native ace-a2x ace-runtime auto-val`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 两闭包共享更新可见且最后owner回收；自环按冻结策略释放；FStr两个有副作用插值按源序各一次；未知类型不得静默Dynamic。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 旧对象写y=200/z=300/x=100后读和600；闭包factor=3映射[1,2,3]新例须核[3,6,9]而非只len3；动态类型标签与推导失败不同。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 缺字段按既有nil/默认值/拒绝合同冻结，不一律改成异常；不可调用值、错tag/反射越权与坏捕获生命周期正确拒，不能遇推导失败就Any。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 共享可变/弱引用和循环策略具名；捕获环境失败保旧、最后owner释放；FStr求值原次序，闭包不夹旧解释循环。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 共享动态资源与显示转换的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-INT-1026 | work | IF-VAL通用外部值/可调用值与retain/drop交接有真实正式版本和V-INT-1026；无PyO3/第二资源账本/依赖1026，无活对象portable误入；旧原生近邻保持 | T-01/02/03/08，T-04汇总，原T-05独立复审 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 已完成：962/963 已归档；master@cb1a185ea 四端测绘（对象/闭包/FStr/foreign 契约）；冻结决定摘要入 §5 T-01 节（支持集/入口/对象/nil/可调用/类型标签/INT-1026/循环策略/owner）；worktree `D:/autostack/.wt/lang-964/auto-lang`@plan-964-dev（基面 cb1a185ea，auto-down 兄弟 detached@58ea33f）；材料目录 `D:/autostack/plan-artifacts/plan-964/` 已登记。

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 已完成：av-core 动态值/对象/nil/可调用/外部单元、core 管道（grammar/CST/resolve/descriptor/lowering/fstr）、永久四端测试（av 10/10、av-core INT-1026 4/4、native 4/4、a2x 1/1）与语言侧 8 例语料全绿；wire 负控（tag 表扩展 11→13）与 grid 混列负控（按列内同型语义校正 + 具名 source.type-mismatch）已修。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 已完成：四端接线（native closures/fstr 车道 + CRT stub、a2x display/concat、av-worker 新增 aggregates-source-run-v1 recipe 并注册父侧 recipe 表/RecipeFamily/派发、closures-source 冻结表 D01/D02）、language-side case 组 8/8 绿。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：明确Arc/共享可变/弱引用等实际已用形式及循环回收，不把Box递归与共享循环混为一谈；FStr的表达式求值和插值归本包，格式化算法归972。 验证V-08及INT-1026外部引用清理见证；已有T-04（945–948为T-05）汇总此证据后才交独立review。 已完成：共享对象= Rc 别名（nil_and_alias 20 见证）、自环 ObjectSet 以 Rc::ptr_eq 具名拒、FStr 源序求值恰一次（fstring 42）、INT-1026 外部值 retain/drop 恰一次（av-core 4/4）、活对象 portable freeze 具名拒。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 已完成：`cargo t auto-val ace-source ace-semantics ace-av ace-native ace-a2x ace-runtime case-ace-m2-dynamic-964` 冻结提交并集两轮；最终轮 auto-val 150 / ace-semantics 269 / ace-a2x 230（230/230 ok，仅 meter "descendants live after workload exit" infra flake）/ case 组 8/8 全绿；ace-av 1098（红 2 = text.char-multi 预置词法）、ace-native 250（红 3 = text.char-multi）、ace-source 503（红 1 = text.char-multi）、ace-runtime 84（红 19 = __CxxFrameHandler3/_fltused 预置 lld 链接）。**无 PLAN-964 归因测试红**：本计划引入的 2 个真实回归（av-core wire 负控 tag 扩展、grid 混列负控按列内同型语义校正）已修复并隔离复验绿；首轮并发 Deadline（mn×10/a2x×2）隔离复跑全绿，为环境并发 flake。

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。 已完成：独立只读审计代理（general-5）AC-01..06/AC-INT-1026/AC-PHASE 逐项核验 + 四端定向复跑；needs_fix 3 处（F-1 Callable 臂序/F-2 自环双见证/F-4 probe/F-5 fmt/F-7 fn-as-value 码）已在 b03075d92 修复并经复核 **pass**；D-964-C1/C2/C3 记 KNOWN-DEBT；§9 复审记录见上。

- [x] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。 已完成：规范增量落盘（m2-dynamic.md SD-01/02/PHASE/INT-1026/DETAIL + replacement-scope SD-08），rebase master 后 ff-only 合回（master tip=9135c9ceb，线性历史，range-diff 等价证明），受影响面 rebase 态复验（av 12/12、av-core 4/4、native 3/3、a2x 1/1、case 8/8；rebase 暴露 TEXT_DISPLAY opcode 与 master PLATFORM_CALL 冲突，0x3C→0x3F 已修）；归档见下。

- [x] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。 已完成：证据摘要折入 §9；wt-guard clean（无 reparse point）；worktree/分支/组目录已删；plan-artifacts/plan-964 已删（无具名持有）；retention 已刷新。终态见 §9 merge 收据。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-964/auto-lang`，分支 `plan-964-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-964/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-964 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

### 2026-11-14 T-06/07 merge 收据

`stage: merge | plan_id: PLAN-964 | plan_revision: r6 | outcome: pass | delivery_commit: 9135c9ceb | branch: plan-964-dev（rebase 到 master@10a3dd46e，range-diff 1/3/4 恒等、2/5/6 冲突解析含 master packages-ct 合并与 conformance 去重、+spec +opcode 修复） | landed: master tip=9135c9ceb（--ff-only，线性） | canonical_verified: docs/specs/ace-migration/{m2-dynamic.md（新建）,replacement-scope-and-evidence.md（SD-08）} 在 master 在案 | archived: docs/plans/archive/964-ace-m2-dynamic.md，status: archived（见终态） | cleaned: wt-guard clean、worktree/分支/组目录删除、plan-artifacts/plan-964 删除、retention 刷新 | next: 无（整链完成）`

### 2026-11-14 T-05 独立复审结论（final pass）

`stage: review | review_scope: final | reviewer_identity: 实施会话内新上下文只读审计代理（general-5，局限性：同会话族，按工件重建判定） | plan_id: PLAN-964 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-964/auto-lang | branch: plan-964-dev | reviewed_commit: b03075d92 | base_commit: cb1a185ea | dependency_revisions: master@cb1a185ea（962/963 已归档交付） | spec_inputs: docs/specs/ace-migration/{m2-aggregates,replacement-scope-and-evidence}.md | acceptance_results: AC-01 partial→pass（F-1/F-7 修复后）、AC-02 pass、AC-03 pass、AC-06 partial→pass（F-2 双见证后）、AC-INT-1026 pass、AC-PHASE pass | pending_closeout_gates: T-06 merge（规范增量落盘 SD-01/02/PHASE/INT-1026/DETAIL、合回、归档、清树）；并集环境红具名（char-multi ×6、lld 链接 ×19、ace-a2x meter infra flake） | findings: F-1（dynamic.rs Callable 臂序，已修 b03075d92）、F-2（自环双见证，已修）、F-3（warning 清零，动态 2 条消）、F-4（probe 删除，已修）、F-5（fmt，已修）、F-7（fn-as-value 码对齐，已修）；F-6 为简报分类注记非缺陷；D-964-C1/C2/C3 登记 KNOWN-DEBT（fstr && 边/portable 码命名空间/grid 异构列语义） | evidence: 四端复跑 12/12+4/4+3/3+1/1、case 组 8/8、fstr 6/6、av-core warning 4 条全 base 既有、复核复验通过 | next: T-06/07 merge（goal 授权链内）`

### 2026-11-14 T-01..T-04/T-08 实施收据（goal 授权链内）

`stage: work | plan_id: PLAN-964 | plan_revision: r6 | round: 2 | from: work | to: review | status: drafting | outcome: pass（实施+验证） | worktree: D:/autostack/.wt/lang-964/auto-lang@plan-964-dev | commits: 6164713a1/973b08a93/0a58d62c3/7fab4b9b0/7a624c543 | next: T-05 独立复审`

- T-02/T-03/T-08/T-04 已完成（§8 勾选与证据摘要见上）；四端永久测试 av 10/10、av-core INT-1026 4/4、native 4/4、a2x 1/1；语言侧 case 组 8/8（map 600/HOF 369/fstr 42/callable 9/nil-alias 20/缺键 trap 70/两个 resolve 诊断）。新增 av-worker `ac-av/aggregates-source-run-v1` recipe（父侧 recipe 注册表/RecipeFamily/派发 + closures-source 冻结表 D01/D02）。
- 并集最终轮红集与预存基线一致（char-multi 词法 ×6、lld `__CxxFrameHandler3`/`_fltused` ×19、ace-a2x meter infra flake）；并发 Deadline 类隔离复跑全绿（环境归因）。本计划净改善：mn 车道 CRT stub 解除 K02/owned-text 既有链接红。

### 2026-11-13 r6接口规划复核（未实施）

本轮仅补INT-1026/IF-VAL、SD-INT-1026、V-INT-1026、AC-INT-1026，均由既有任务承接；原8任务/0进度、依赖与完整验收不变。消除consumer对外部值/Meta的单方假定，交付仍待原owner正式实现/验证。不要求1026运行才交通用切片，不宣称独立实现review通过；只做文档静态检查。

### 2026-11-13 goal 轮次收据

`stage: goal | plan_id: PLAN-964 | plan_revision: r6 | round: 1 | from: goal | to: work | status: drafting | outcome: pass | worktree: （待建） | commit: - | next: work`

首轮：goal 授权已记录于§4；主检出仅docs/plans与docs/design簿记WIP（无代码路径WIP）；无964专属树，状态drafting与§9最新规划收据一致，进入work阶段。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
