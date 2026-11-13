---
plan_id: PLAN-963
status: reviewed
feature_name: 通用泛型spec与方法实例化
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-13
plan_revision: 5
current_step: 6
total_steps: 8
depends_on: [PLAN-960, PLAN-962, PLAN-935]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-generics.md]
touched_goals: [GOAL-002]
---

# [PLAN-963] 通用泛型spec与方法实例化

## 0. 变更摘要

2026-11-12 r5：补齐原稿缺少的具体Auto语法/正反例、类型与模块设计、旧VM/A2R用例到新四端检查的映射。保原8任务/全部原AC与V/依赖及drafting零进度。T-01冻结下列具名未决项，不再把全部设计推给实施者临场发明。本轮只改计划，不实现/建树/生成fixture/运行测试。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

从935有限标量模板扩到真实类型/值参数、约束spec、ext、alias、方法接收者、跨模块实例与重载规则；共享实例图承载语义身份，目标缓存独立，递归实例检测与预算不冒语法限制。

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
- `experimental/ace/crates/source-query`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/core/src/source`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/trait_checker.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 复核：语言来源、旧测试与ACE现状

用户指出963缺少具体语法和类型/代码设计，询问与AutoVM及旧用例的关系；本轮延续规划授权，只修订合同。主线读取`f9fb3854beb3886338156cfa61d16f7f84893cc9`，962已8/8归档并落地主线；935/960亦已归档。前置齐不等于r4正文细度已充分。

依据为当前已裁定语言/模块Spec、[语言说明Generics](../language/specification.md#generics)、959分母L-39/40/42与既有公开行为。旧VM/旧A2R提供兼容源码、断言与实现调研，不是新ACE的最终语义裁判或fallback。旧bug、规范冲突、仅parse/emit成功与真正运行需分开登记，不随意重写expected。旧[VM泛型/堆设计](../specs/auto-lang/vm/design/generics-heap.md)的早期限制须结合417等后续测试核，不能照旧快照缩范围。

已核实际来源：

- 旧AST `crates/auto-ast/src/ast/types.rs::{GenericParam,TypeParam,ConstParam,GenericInstance}`；parser在`crates/auto-lang-core/src/parser/{type_syntax,decls,expressions}.rs`，约束检查在`trait_checker.rs`。参考语法/行为，不接旧Parser或VM执行结构。
- [935当前实例合同](../specs/auto-semantics/source-scalar-instances.md)：单T、i32/bool，复用共享求解器，需求实例化为具体程序，再经唯一verify/四端；不具备通用spec/const/nominal参数。
- ACE `experimental/ace/crates/core/src/source/ast.rs`：FnItem.type_params仍为Vec<Ident>，TypeItem无通用泛型参数，Item无通用spec/ext/alias；`source-query/src/instances.rs::InstanceRecord`只有单ty_param/PortableTy，`instance_host.rs::InstancePage`类型码亦有限。放宽parser或拼函数名不能完成963。
- [960模块](../specs/ace-migration/m2-modules.md)与[962聚合](../specs/ace-migration/m2-aggregates.md)是正式底座；D-962-C1..C5尤其List move、递归Box和C tuple实参必须核对，不因归档假定所有组合无缺口。

源码读取SHA256前16：`experimental/ace/crates/core/src/source/ast.rs` `69b38593cf3c72a3`；`experimental/ace/crates/source-query/src/instances.rs` `d2d4c532065622cf`；`experimental/ace/crates/source-query/src/instance_host.rs` `860b295538178504`；`crates/auto-lang/src/tests/trait_vm_tests.rs` `a1f3924261667a1c`。本轮只读，没有重跑旧VM或ACE；后文均为待执行验证合同。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


从935有限标量模板扩到真实类型/值参数、约束spec、ext、alias、方法接收者、跨模块实例与重载规则；共享实例图承载语义身份，目标缓存独立，递归实例检测与预算不冒语法限制。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：完整函数调用与实例边界（r2）

补默认/命名/可变实参、方法与扩展优先级、函数值签名、返回推导及现用运算符/约束；实例生成须接962布局与资源事实。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 5.1 要实施的具体源码与语义

963沿用既有Auto语言，不新造一套Rust/C++泛型语法。以下是既有拼写或其最小验证输入，尚未在新963运行；代表例不代完整支持集。

**泛型函数与用户泛型类型**：

```auto
fn identity<T>(x T) T { return x }
fn main() int { return identity(42) }
```

结果应为42；另验bool、文本、聚合/owned实参、多类型参数与跨模块方法实例，不能只做935已有两标量。

```auto
type Point<T> { x T; y T }
var p = Point<int> { x: 100, y: 200 }
p.x + p.y
```

源自旧VM `10_types/003_generic_instantiation`，M2须有实际执行300及字段类型/布局的独立正例。近邻`Point<int> { x: true, y: 200 }`应在类型检查拒绝、业务dispatch=0。旧目录只附空`generic_instantiation.expected.error`，不是旧VM运行成功证明；T-01先核原loader判据，新M2正例另有来源与expected，不篡改旧错误期望。

**spec约束与实现**（现有417测试形态）：

```auto
spec Comparable { fn compare(other int) int }
fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}
type Score as Comparable {
    val int
    fn compare(other int) int { return self.val - other }
}
fn main() {
    var x Score = Score { val: 3 }
    var y Score = Score { val: 9 }
    print(max_of(x, y).val)
}
```

stdout应为9；另验缺抽象方法/约束不满足的拒绝、wrapper向下传约束、多实现者与默认方法继承/显式覆盖。参考`trait_vm_tests.rs`公开断言，ACE不必复制旧CALL_SPEC/堆tag实现；统一实例/见证选择可产生相同公开行为。完整dyn对象归964，不反置为本包静态泛型的前置。

**值参数、别名与注解形**：

```auto
// 语言说明中的类型/值参数形；完整运行构造/初始化由T-01先证冻结。
type Inline<T, N u32> { data [N]T }

// 现有旧A2R用例只声明，未实际调用，不能作为值参数执行证明。
fn test_fn_with_const_param<T, N uint>(capacity N) int { return 0 }

// 现有旧别名与with约束形，也需纳入差额表。
type List<T> = List<T, Heap>;
#[with(T as Clone)]
fn duplicate(x T) T { return x }
```

T-01逐形核当前规范/parser/原检查，值参数必须补实际影响类型、布局或结果的输入：N=2/N=3的不同实例、同值规范化复用、错类型/错值/错arity拒绝。不能以未调用声明或生成Rust文本通过值泛型。

泛型spec、关联类型、super-spec、ext/方法接收者与显式类型实参沿现有AST/parser和旧用例逐项纳入；不从别的语言猜拼写。T-08原默认/命名/可变实参、函数值签名和返回推导义务保留，实参按源码序求值、默认值每次调用一次，方法/扩展候选在共享绑定层决议；不能为避未来组合依赖删掉963负责的部分。

### 5.2 ACE类型、实例与模块设计（拟实现）

普通源→泛型声明与参数环境→共享绑定/约束候选→规范替换/实例需求→具体类型布局/函数体→唯一verify→AV/AC/A2R/A2C。沿935/960/962机制扩展，四端消费同一已解语义，不把泛型原样交Rust后端替ACE求解。

下列为语义字段草案，名字不是现存Rust API、没有编译；T-01核复用/等价实现并冻结准确类型与接口：

```rust
// Proposed shapes, not existing API.
enum GenericArg { Type(CanonicalTypeRef), Const(TypedConstValue) }
struct LogicalInstanceKey {
    owner: DeclIdentity,       // package + module + declaration owner
    args: Vec<GenericArg>,     // declaration order, canonical identities
}
// Implementation recipe additionally binds effective signature/body,
// selected impl/constraint dependencies, algorithm/profile and their versions.
```

| 对象 | 实际owner/拟改面 | 必须关闭的设计不变量 |
|---|---|---|
| 声明/参数环境 | ACE core `source/ast.rs`、`source/cst/project.rs`与`grammars/auto-source.at` | 类型/值参数、约束、泛型type/spec/ext/alias带原span；T按声明owner/位置绑定，不当全局名字；同步所有构造器/reader，旧capability版本隔离 |
| 替换/布局 | core source resolve/type事实＋960模块声明身份＋962聚合/容器 | nominal与结构类型区分，值参数有声明类型和值，alias正规化保依赖；替换后仍核borrow/move/有限大小，不Any擦除 |
| 约束与见证 | 共享语义/绑定层，扩spec/关联类型/impl/默认体/ext候选 | 缺实现/同优先级歧义拒绝；候选正负读取入依赖图；不复制trait_checker或VM成第二套语义 |
| 实例发现/身份 | `source-query/src/instances.rs`扩单T与PortableTy，沿原共享求解与需求不动点 | logical key不含span/callsite/body版本；implementation key含有效body/约束/类型依赖；同实例去重，不同包/类型/值不混，装配名非身份 |
| 页/增量/恢复 | `instance_host.rs`及既有query/module/store | 新schema/producer或明确升级；不重释旧S935页；错误/取消不半发布；恢复重验而非持久化Checked；body改不让无关caller重推导 |
| 四端消费 | Checked/MIR、现有AV/native/A2X适用入口 | 值/效果/资源分别有独立oracle；新A2R/A2C真实编译运行，无legacy fallback，emit-only不完成 |

通用实例不能把935有限的“每模板两种标量”当新上限。沿现行solver步数/截止、受管预算与发布事务，T-01给合法规模/超界分类和预算账；不能临场加任意泛型深度/实例数语言限制或提高预算。合法同实例递归与无限增长需求分别核准入/资源拒绝，不无限展开字符串类型。

### 5.3 旧用例复用与新验证映射

直接复用旧Auto源码和公开行为断言；保旧expected/原检查身份。旧VM opcode/对象注册表、旧A2R生成Rust全文金样属旧实现检查，不能直接当新四端语义oracle；已知旧差异具名裁定，不把所有旧行为自动升级为新规范。

| 已核旧来源 | 原义务与新侧必须补足的证据 |
|---|---|
| VM `10_types/003_generic_instantiation,004_generic_field_x,005_generic_field_y,010_ext_method`＋`vm_file_tests.rs` | 用户泛型type/字段/方法；分清错误期望/编译检查与执行，补字段读写、参数返回与错误类型近邻 |
| VM `24_generics`（3个.at） | List用户类型/to_array/filter；内建List<T>不证明用户泛型，打印ok不证明过滤算法；963实例义务与库/闭包组合owner分别登记 |
| `crates/auto-lang/src/tests/trait_vm_tests.rs` | 默认方法、抽象方法拒绝、bound/unbounded方法调用、多实现者、违例与wrapper；逐test名保原source/断言，补ACE运行与约束拒绝 |
| A2R `08_generics`（8个.at） | alias/const/字段/指针字段/with/Map/显式形/has；const未调用、字段例未构造，要补真实调用/构造，不以.expected.rs替执行；保ACE已定i32，不照抄旧生成i64 |
| A2R `11_methods`（10个.at）、`12_specs`（13个.at） | ext/static/receiver/默认体/参数/关联类型/generic spec/super-spec；逐条登记963静态语义、964动态/闭包及组合consumer，不能整目录绿充完整覆盖 |
| 935 `core/tests/cases/source_instances_935.rs`与source-query实例测试/既有TestCase | 原capability、实例身份、单飞/增量/恢复不变量继续；新通用参数/资源/值/约束候选另有新case |

除标明的Rust路径，旧目录前缀为`crates/auto-lang/test/`；ACE短路径前缀`experimental/ace/crates/`。.at库存不是测试数/通过数，T-01继续扫描相关inline/parser/调用用例，不以种子表封顶或重复计original.at。

T-01在本Plan内冻结“旧case/test/check ID、source路径/hash、原断言/expected种类→规范结论→新case/后端/独立oracle→能力/组合owner”。未知项具名阻相关验收，不默认NA；963负责泛型部分，不能把自身义务都丢给985，也不要求在963迁完整旧产品。

新永久源拟`crates/auto-lang/test/cases/ace-m2/generics/`（T-02/T-08未来创建），沿现TestCase/ACE worker；core/source-query命名case沿原owner补充。inline Rust源可按明确provenance形成新consumer输入，旧测试不改；复用1002试点的已交能力，不因格式试点未交新增硬依赖，不另造runner/checker。

### 5.4 T-01具名未决与进入T-02的条件

T-01先交语法/旧新义务表、类型/实例/约束模块图和三个全链小样：泛型type字段真执行、spec有约束调用、值参数实际影响；各配同类拒绝和修复恢复。未来探针只在专属树/scope建立，本轮不生成。

须关闭：①值参数完整源形/调用/初始化与合法域；②关联类型/约束/ext选择优先级及规范冲突；③类型/实例/页升级方案不破935；④D-962-C1..C5对新泛型/四端的实际影响，尤其move后使用/递归Box/C tuple实参；⑤四端真实target/命令、资源与独立oracle。影响公共语义/预算的变化具名修订，不由实现者猜或删原义务。未决只阻对应依赖实现，不可用“以后T-01再定”把全部设计留空。

### 5.5 T-01 冻结记录（2026-11-12，基面 990d27f，只读核对）

现状闸门（实读，`E/`=`experimental/ace/`）：

| 事实 | 证据 |
|---|---|
| 语法已能识别 `<T>`/`<T,U>`/`<T has B>`/`<const N>`，但仅 fn 尾部 | `E/grammars/auto-source.at:304-332` |
| `type` 声明无类型参数位 | `auto-source.at:498-504`；投影 `E/crates/core/src/source/cst/project.rs:4015-4090` |
| `spec/ext/impl/alias/with` 只有保留字，零规则零投影 | `auto-source.at:102,109,113,131,145`；`ast.rs:178-192` |
| 调用点无显式类型/值实参位 | `auto-source.at:1005-1008`；`ast.rs:378-381` |
| `TypeRef` 无 const 引用、无定长数组 | `ast.rs:67-96`；`auto-source.at:369-390` |
| 用户泛型类型名在 resolve 全拒（仅内建集合按 arity 匹配） | `E/crates/core/src/source/resolve.rs:3819-3826,3970+` |
| 实例层硬门=单 T + I32/Bool | `project.rs:3546-3555`；`E/crates/source-query/src/instances.rs:87-101,537` |
| 语义侧已多参数就绪（`Ann::TyParam(u16)`、每参数 mint var、demand 返 Vec） | `E/crates/semantics/src/input.rs:42`；`infer.rs:170-185,574-578` |
| 候选选择「1 个才绑，>1 即歧义拒」 | `infer.rs:557-561,669-677` |
| 935 常量被测试逐字钉死；页 magic `IN935P`/codec v1 | `E/crates/source-query/tests/instance_host_935.rs:94-96`；`instance_host.rs:30-37` |
| 重绑遍历白名单只递归 Call/Hash/Binary/Unary | `instances.rs:790-839` |

五项具名未决的冻结结论：

1. **值参数**：唯一接受形 `<const N>` / `<const N u32>`（缺省 u32）；计划 §5.1 的 `<N u32>`、`<N uint>` 现状**不可解析**，按此形修订。调用点拟 `rep<2>(21)`，需新 `generic-call`（提交条件照 `generic-atom` `auto-source.at:872-880`）。**布局效应无载体**：ACE 无定长数组类型，`[N]T` 不可表达，故 V-G04 只兑现「值/身份/规范化」半边（原文即"改变布局或值"，不构成范围缩减）；布局形另立 owner。合法域=声明整数型范围；同值规范化=（声明类型 tag, 宽度内 bits）→ 小写 hex 入键。
2. **spec/ext/优先级**：ACE 无 spec/conform/ext 机制（零先例）。表面冻结为 `Item::Spec{name,methods}`、`TypeItem.conforms: Vec<Ident>`、`Item::Ext{target,methods}`；候选层级 `自有方法 > bound spec > ext`，层内 >1 即复用 `ambiguous-candidates` 拒；`ext` 与自有方法同名拒。决策点：`resolve.rs` 方法/字段查找点（现 `:5157` 附近）建 spec/conform/ext 表 → `adapter.rs:202-228` 投影（`input.rs:82-86` 扩 conform）→ `infer.rs:557-561` 负依赖/歧义。诊断码 `source.spec-{unimplemented,unsatisfied,ambiguous}`，全部 dispatch 前。关联类型零先例，仅列最小新缝，不承诺本切片。
3. **类型/实例/页升级**：单参身份**保持字节不变**（logical/key 生成顺序对 N=1 等价），故 `INSTANCES_*` 常量不动、935 键/页语义不动；页 codec 升 v2 表示 `argc + 每 arg(name,ty)`，旧 v1 页按既有「新版本必须 MISS，绝不重释」语义自然 MISS 重产。结构改 `InstanceRecord.ty_param+ty` → `args: Vec<InstanceArg>`、`InstancePage` 同步；`PortableTy`/934 事实页不扩。
4. **D-962-C1..C5**：C1 真风险=重绑遍历白名单漏 `Index/TupleLit/Method`（模板调用藏在其中不重绑）；C2 决定探针不得用递归 Box（native 车道未闭）；C3 决定探针不用 tuple 实参（与 C 端临时取址冲突）；C4 泛型放大 use-after-move（`concretize` 只换注解不算所有权）；C5 新诊断不得沿用错层拼写。跨模块模板由 `E/crates/core/src/source/packages/binding.rs:645-648,673-676` 以 `source.module-templates`（"close in PLAN-963"）拒，关它须在 SD-02 具名。
5. **四端与 oracle**：组 `cargo t ace-source ace-semantics ace-av ace-native ace-a2x`；P1 泛型 type 字段执行 oracle=300、P2 spec 约束调用 oracle=9、P3 值参数 oracle=42/63（21×2/21×3 手算闭式）；四端各自独立断言，emit-only/零选择/工具缺失/超时不计 PASS。

仍属 T-01 未完部分（须在实现可用后补）：三个全链小样未建立（依赖 T-02/T-03 的 generic type、spec、const 参数落地）；`ProduceLimits` 步数/截止数值与新增域预算账未读定。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-generics.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-generics.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-GENERICS | add | `docs/specs/ace-migration/m2-generics.md`（沿本包owner） | 仅范围→实际已验语法/类型值参数/约束见证/实例身份/旧新case映射及限制 | 可按源码/独立oracle复核 | AC-01/AC-02/AC-03/AC-06 |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 同名不同包实例不串 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 改变被读取的约束/实现使实例失效 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 未读方法body改动不重产 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 歧义/不满足约束在dispatch前拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 恢复后首实例编辑与冷编结果一致 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-native ace-a2x`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 默认实参只求值一次，命名实参源序求值；同名ext歧义拒绝；资源泛型跨模块参数返回正确；MIX-01实例腿交给985。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 963具体补充验证（未来建立，当前未执行）

| Case ID | AC / work owner | 普通源输入与独立关闭oracle | 边界与负控 |
|---|---|---|---|
| V-G01 | AC-02/03；T-02/03/04 | identity/choose多次同类型调用及int/bool/文本/聚合，42/true/文本/字段值准确；同实例共享、不同替换不串 | 不限935两标量；错误类型dispatch前拒 |
| V-G02 | AC-02/03；T-02/03/04 | §5.1 Point实际结果300；用户generic字段/嵌套/方法参数返回正确 | Point<int>填bool拒；List例不替用户泛型type |
| V-G03 | AC-02/06；T-02/08/04 | max_of Score(3/9)得9，两个Labeler产生各自文本，wrapper保约束，默认继承/覆盖各命中 | 缺抽象方法/不满足bound/歧义拒，不测旧堆布局代语义 |
| V-G04 | AC-01/02；T-01/02/03/04 | 冻结合法值参数源实际调用/构造，N=2/3改变布局或值并有不同身份，同值规范化复用 | 错类型/值域/arity拒；未调用声明/emit不关行 |
| V-G05 | AC-02/06；T-02/08/03/04 | generic spec/关联类型、with/alias/ext/static/receiver逐源有独立值/选择断言；默认/命名/可变参数精确trace和一次求值 | 候选负依赖变化重选；同优先级歧义/alias递归拒，多算漏算反控 |
| V-G06 | AC-02/03；T-02/03/04 | 两包同名模板/类型与跨模块实例值、类型身份各对expected；owned跨参数/返回保借用/转移/释放账 | owner/值替换不误hit；move后使用/视图逃逸/非法递归拒，962缺口不冻结成新M2合法行为 |
| V-G07 | AC-03；T-03/04 | 模板body/签名/所读impl/关联类型/值参数变化后与冷编oracle一致；无关body未读实例生产0 | 恢复不信旧Checked；错页/身份拒，失败保旧、精确修复恢复 |
| V-G08 | AC-02/03；T-02/03/04 | 同源AV/native/A2R/A2C各对独立值/效果/诊断/资源，合法递归/超界分态，取消/late无半发布 | 禁legacy provider仍执行；emit-only/零选择/工具故障不能冒运行或语言负例PASS |

原V-01..05/V-08/V-PHASE全保留。新行细化其源与断言，不替旧门；T-01冻结实际永久case/check名字、入口/target/命令。新fixture尚未建立，拟路径不写成已运行。职责并集仍为`cargo t ace-source ace-semantics ace-artifacts ace-av ace-native ace-a2x`；实际改legacy/parser/根接线才补相应完整组，不因调查旧用例无条件全跑旧VM。最终核真实终态、原条件排除、首次红/重放与缺证，不能以四端互等作唯一oracle。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | §5.1源码/旧新义务、§5.2类型/模块图、§5.4具名决定与真实命令/资源冻结，unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 完整函数调用与实例边界的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

## 8. 执行步骤

当前 6/8 完成（T-01..T-05、T-08 已闭；T-06/T-07 按用户指令本轮不执行）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01/AC-PHASE**：核935/960/962正式来源，按§5.1..5.4交逐语法正反源/旧新case-check表、类型与实例模块图、三个全链小样及资源/真实四端命令，闭具名未决再进入对应实现；空expected/旧VM或emit不当新侧通过。

- [x] **T-02｜M2-A｜work｜AC-02**：按§5.2实际owner扩声明/替换/约束见证与实例页，创建§5.3永久源及V-G01..06/08正反例，保935能力面；先共同事实/类型/资源，不复制旧VM语义引擎。

- [x] **T-03｜M2-A｜work｜AC-02/03**：将同一已解实例接四端真实consumer，完成装配/资源/增量/恢复；补V-G06..08及旧义务组合，C/Rust真实编译运行，各对独立oracle，不以生成全文代运行。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：补默认/命名/可变实参、方法与扩展优先级、函数值签名、返回推导及现用运算符/约束；实例生成须接962布局与资源事实。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 同时逐项运行V-G01..08与§5.3旧新义务映射，列真实发现/执行/原条件排除和缺项。

- [x] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-963/auto-lang`，分支 `plan-963-dev`（已创建，终树提交 `f1d63cdf0`；早期证据绑定 `03b5a30f8` 已被其后的证据补强提交覆盖），按仓规。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-963/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。

## 9. 复审记录

2026-11-13 stage:review·T-05 独立复审（同树自审，未派 agent；结论绑定终树提交 `f1d63cdf0` + 终树并集 `run-2v_8abue`；早期 `03b5a30f8`/`run-ghf902pr` 的证据面已被其后的补强提交覆盖，按"过程观察不计"处理）/ PLAN-963 r5：

**AC 逐项（证据 → 结论）**

- **AC-01 pass**：§5.1/5.2/5.4 的 path:line 冻结在 T-01 记录（2026-11-12 条目，含 `crates/auto-lang-core/src/parser/expressions.rs`、`crates/auto-lang/src/tests/vm_types_tests.rs` 的 `spec`/`with` 源、`semantics-contract/src/metadata.rs` 的属性面）。unknown 项均在 §9 具名登记，无漏计。
- **AC-02 pass**：§6 每行 V 都有终树具名入口与独立 oracle（映射见下条 T-04），正反例各含负控；oracle 为原源期望值/诊断码或独立 expected，不以四端互等作唯一 oracle。
- **AC-03 pass**：AV（`ace-av-core` 执行）、ac-native（`MnDriver` exit code）、A2R（真实 rustc）、A2C（真实 cl）四条生产 consumer 全部走 ACE MIR 产物；case 语料值行同时带 `ac-av/core-run-v1` 与 `ac/mir-native-scalar-v1` 双腿，无 legacy fallback、无 emit-only 冒充运行。
- **AC-04 pass（in-scope）**：本条目即 T-05。规范增量按 §5「规范增量」表逐行核过「after 规则 ↔ 实际证据」一致性（SD-01/SD-GENERICS 的语法、类型/值参数、约束见证、实例身份、旧新映射与限制均有具名证据；SD-02 的新 producer 身份由四端真实运行支撑；SD-PHASE 的 B 仍 pending 不被写成已交）。遗漏/延后/workaround 扫描见下；7 条**超范围/边界/质量**项已具名登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md` §「PLAN-963 复审登记的候选债」（D-963-C1..C7），不属 §6 任何 V 行的未闭项。
- **AC-06 pass**：V-08 四项逐条有具名证据——默认实参一次求值（`av_default_argument_evaluates_exactly_once_963`：MIR 里默认值 callee 恰 1 次 + 值 42）、命名实参源序（`av_named_arguments_evaluate_in_source_order_963`：MIR 指令序 `d_five` 先于 `d_twenty`）、同优先级 ext 歧义拒（`two_ext_blocks_with_the_same_method_are_rejected`）、资源泛型跨模块（`packages_963_owned_generic_parameters_and_returns_cross_modules`）；MIX-01 实例腿按 §6 该行既有裁定交 985。V-G05 的「可变参数」以负控 `variadic_parameter_form_is_rejected_963`（`source.syntax`）落地边界——AutoLang 无用户声明位可变参数拼写（legacy 亦无，唯一「varargs」是 builtin `List.new(a,b,c)`）；V-G02 的「List 例不替用户泛型 type」以 `builtin_list_does_not_replace_user_generic_type_963` 钉住（用户模板集是唯一泛型面，`List` 不产实例页、其拼写在 instances lane 语法层拒）。
- **AC-PHASE pass**：§5 阶段表 → §8 任务 → §6 V 行逐行可追溯；A 组 required 全闭，无 scope 缩减；B 全产品替换仍 pending，未改 NA。
- **AC-05 未执行**：按用户指令本轮禁止 T-06/T-07（不 merge、不归档、不清树），规范增量 `docs/specs/ace-migration/m2-generics.md` 仍未落盘。

**遗漏/延后/workaround 扫描（全部具名，无静默）**

- 管道算子 `|>`：ACE 语法/AST 零命中（`grammars/auto-source.at`、全 crate grep），legacy 脱糖语义在 `crates/auto-lang-core/src/parser/expressions.rs:562-590` + `docs/design/pipe-operator.md`；不属 §6 任何 V 行 → D-963-C1（owner 963 T-08 后续/965）。
- `#[with(T)]` 属性形：963 以 `source.unsupported` 具名迁移拒（规范形 `type N as Spec`），owner 为 language attribute grammar plan → D-963-C3。
- 记录类型模块全局 `.move`：`module.global.set:<sym>` 伪签名只有借用形，属 960 模块面 → D-963-C4。
- 默认实参求值作用域（空作用域、`moved:false`）是 963 自定边界且 legacy 无 oracle → D-963-C5（须在 Spec 明写）。
- 格式：ACE workspace 无 fmt 门禁，master 基线即非 clean；本计划 20 个触碰文件同状，未统一格式化（避免无门禁面的纯格式翻动） → D-963-C6。
- workaround 检查：改动源零 `dbg!/todo!/unimplemented!/FIXME`、零新增 `#[allow]`/`#[ignore]`；新增 `unwrap()` 仅出现在测试夹具的 `create_dir_all`/`write`。`fill_default_args` 对混合 named/positional 提前返回是交 `check_args` 的既有分工，不是绕过。

**未闭但不属本包 AC 的具名残余**（供 merge 决策）：T-06/T-07（merge/归档/清树）、规范增量**落盘**（表内容已复审，仅未写入 `docs/specs`）、D-963-C1..C7 七条。

2026-11-13 stage:work·T-04 证据闭合（终树 `f1d63cdf0`，工作树 clean）/ PLAN-963 r5：

固定组并集在**终树**上单实例跑通，父 gate 自证：`$env:CARGO_TERM_COLOR='never'; cargo t ace-source ace-syntax ace-semantics ace-av ace-native ace-a2x case-ace-m2-generics-963`
→ `target/test-groups/run-2v_8abue`，`gate_passed=true`、`error=空`、`source_state=a46c4996369d133c42d6e9d834d8461a09fbce3aee9b88a4f8a5c5869ebf85b5`、`config_sha256=e8b55f0eeecc01bb4eb2c5d672a11c2113ee00a39aae8842d81b3f650f6c7f2f`、wall=**608.3s**、退出码 0。
各组 PASS 与执行数：ace-source **497**、ace-syntax **132**、ace-semantics **269**、ace-av **1080**、ace-native **236**、ace-a2x **229**、case-ace-m2-generics-963 **10 checks**（含 2 负控）——合计 **2453** 个 Rust 用例 + 10 个 case check，`errors=0`、`not_run=0`。
- **条件排除（如实列出，不充 PASS）**：`excluded` 共 3 条，均为**既有 `#[ignore]`**、非本计划引入（`git diff master..HEAD` 对 `core/src/link.rs` 与 `a2x-worker/tests/conformance.rs` 的既有忽略行零改动）：`ace-core link::tests::run_exe_60s_deadline_bounds_descendant_pipe_hold`（`#[ignore = "60s acceptance evidence for AC-17; run explicitly"]`）、`ace-a2x-worker inventory_941::read_scalar_contract_inventory_941`、`inventory_941::read_composite_contract_inventory_941`（均 `ignored: true`）。本计划新增用例零 `#[ignore]`、零平台/feature 条件排除；`discovery_complete=False` 是发现面标记，未影响七组执行终态。
- 期间另两次并集：`run-ghf902pr`（`03b5a30f8`，七组 PASS 但随后我又加证据，作过程观察不计）；`run-7yo3z078` 七组亦 PASS，因我在该运行期间编辑测试，父 gate 判 `error: "code/config/inputs changed during execution"` → 明确**作废不计**。三次 a2x 全绿（229）说明上一轮 `run-1w1lnge5` 的唯一红 `worker_ct_frontdoor_958::worker_ct_941_ordinary_same_names_four_operations`（`CToolError("cl -? spawn: Deadline")`）是同机工具时限的环境归因，非 lane 缺陷。
- V 行 → 终树具名入口：V-01/V-05 `source-query/tests/instances_multi_963.rs` + `instance_host_935.rs`（跨包同名模板身份、恢复后首实例）；V-02/V-03/V-G07 `instances_invalidation_963.rs`（约束/实现变化失效、未读 body 零生产、`av_generic_record_body_change_executes_301`）；V-04/V-G03 core `source_generics_963.rs` + `source_specs_generic_963.rs` + `source_spec_super_assoc_963.rs`（bound 满足/不满足具名拒、缺抽象方法拒、默认继承/覆盖、`max_of` 函数级 bound 真实族）；V-G01/V-G04 `instances_const_963.rs`、`instances_value_domain_963.rs`（含新增 `builtin_list_does_not_replace_user_generic_type_963`）；V-G02 av/native/a2x `generic_records_963.rs`（300）；V-G05 core `source_alias_963.rs`/`source_ext_963.rs`/`source_default_args_963.rs`（含 `variadic_parameter_form_is_rejected_963`）与 native `alias_963.rs`/`ext_963.rs`/`spec_assoc_963.rs`/`spec_generic_963.rs`；V-G06 `module_packages_960.rs`（新增 owned 记录跨模块方案例）；V-G08 av/native/a2x 三份 `const_instances_963.rs`+`default_args_963.rs`+`generic_records_963.rs`（真实 rustc/cl、exit code 42/70/42 与 AV 值域），另 `av_default_argument_evaluates_exactly_once_963` 提供 V-08「只求值一次」的 MIR 见证（默认值 callee 恰 1 次）；V-PHASE 即本次并集；§5.3 永久语料 `crates/auto-lang/test/cases/ace-m2/generics/generics.{case,suite}.ad` 经 `cargo t case-ace-m2-generics-963` 实跑 10/10。
- 发现/执行/条件排除：7 组 `outcome=PASS`、`errors=0`、`not_run=0`；3 条排除均为既有 `#[ignore]`（上条具名，非本计划引入），本计划新增用例零忽略、零平台/feature 排除；准备（编译/装配）与运行、清理分列在该 run 的 invocations 中；无 timeout、无缺工具。

2026-11-13 stage:work（V-08 三项已闭；T-04/T-05 未做）/ PLAN-963 r5：
提交 `47cb29273`（5 文件 +129/−1）补齐 V-08 余下三项，均有具名证据：

- **命名实参源序求值**：绑定层 `eval` 按源序保留、`bindings` 再 permute 到参数位
  （adapter 的 Call 构建），故逆序命名调用也按源序求值。新证 `ace-av`
  `named_args_963::O-963-N1`：`sub(b: five(), a: twenty())` 的 Typed MIR 里
  `d_five` 调用严格先于 `d_twenty`（block-major 指令序），且 AV 执行 = 20−5 = **15**
  （按名字绑定，非按位置）。
- **同名 ext 歧义拒绝**：新增 `source_ext_963::O-963-E6`——**两个不同 ext 块**对同一
  类型声明同名方法 → `source.duplicate`（不静默取其一），6/6 全绿。
- **资源泛型跨模块参数/返回**：把 `type_spelling` 从 int/bool/record 扩到
  `str`/`Str`（`str` 借用视图与 `Str` owned 拼写不同即不同实例身份），诊断文案同步；
  新证 `ace-av` `module_packages_960::packages_963_owned_generic_parameters_and_returns_cross_modules`：
  lib 声明 `Holder<T>`、app 以 **owned `Str`** 实例化，`wrap(move v Str)` 跨模块接收
  owned 资源并 `v.move` 转移进字段、返回 `Holder<Str>`，`use_it()` = len("m2") = **2**。
  途中三条既有 profile 规则（记录字段必须 owned 文本、构造器隐式转移拒、`move` 参数
  必须配 `.move` 实参）按原样满足，未放宽。
- 回归：`cargo test -p ace-core -p ace-source-query` 全绿（lib 38 + 各 conformance 套件）。
- **仍具名未闭**：记录类型模块全局的 `.move` 转移（前条 §9）；T-04 全 V 行证据与
  六固定组并集、T-05 独立复审未做。

2026-11-13 stage:work（T-08 默认实参片已提交；仍无终裁）/ PLAN-963 r5：
提交 `80bdaced4`（15 文件）实现 §5.1/T-08 的**声明位默认实参** `fn f(n int = 42)`，
四端真实运行全绿，非 emit 快照。

- 语义（V-08「默认实参只求值一次」）：默认值只存 AST 形状（`ast::Param.default`，
  `FnSig.defaults` 与 `param_names` 逐位对齐），**不进任何语义身份/缓存键**；
  仅在调用位实参缺席时补一位，按**调用者**作用域解析一次（`fill_default_args`
  以空局部作用域解析，任何参数/局部名 fail-closed 为 `source.undefined`），
  补位后即普通实参 → verify/MIR/四后端零改动。位置式只补末尾缺席位、中段缺席
  仍由 `check_args` 报 `source.argument`；命名式可补任意缺席位并以参数名标注。
- 语法/投影：`auto-source.at` 新增 `param-default`（`param` 末尾 `opt`），
  CST 投影读 `default`；手工 parser 与 `syntax-core` 通用投影器恒 `None`
  （后者当前无消费者，已注明正式入口在 ace-core `cst/project.rs`）；
  `source-query` 单态化替换显式携带 `default`（模板方法默认值不丢）。
- 四端证据（全部手算闭式 oracle）：
  core `source_default_args_963` 6/6（补位、显式覆盖、命名补洞、命名参数默认值具名拒、
  缺无默认参数仍 arity 拒、超元数仍拒）；
  AV `default_args_963` 3/3 → `42 / 70 / 42`；ac-native 3/3 → `exit 42 / 70 / 42`；
  A2R(真实 rustc) + A2C(真实 MSVC cl) 3/3 → `value:42 / 70 / 42`。
- 回归：`cargo test -p ace-core`（lib 38 + conformance 457）全绿；
  `cargo test -p ace-semantics -p ace-source-query -p ace-syntax-core` 全绿。
- **V-08 仍余三项未施工**：命名实参求值序、同名 ext 歧义拒、资源泛型跨模块参数/返回。

2026-11-13 stage:work（T-03 永久语料目录已提交；仍无终裁）/ PLAN-963 r5：
提交 `8c9fb58e5`（9 文件 +约 190 行）在唯一树 `D:/autostack/.wt/lang-963/auto-lang`
（`plan-963-dev`）建成 §5.3 的永久 TestCase 语料 `crates/auto-lang/test/cases/ace-m2/generics/`
并在 `scripts/test-groups.toml` 注册固定组 `case-ace-m2-generics-963`（`runner = "case"`）。

- 语料（6 case / 10 check，全部真实运行，非 emit 快照）：
  `g01.choose42`（`<T>` 标量模板 int+bool 经实例车道 → 42）、
  `g02.point300`（`type Point<T>` 显式类型实参单态化 + 模板方法体执行 → 300）、
  `g03.bounded5`（`<T has Summable>` 遵从记录 → 5）、
  `g05.picker42`（`spec Picker<T>` 在 int 实例化 + 记录显式遵从 → 41+1=42）四行各带
  AV（`ac-av/core-run-v1` + `ac.source.instances.scalar/v1`；记录面 `ac-av/data-run-v1`）
  与 ac-native（`ac/mir-native-scalar-v1`）两腿；两条负控行
  `g02.point-negative`（`Point<int>{ x: true }` → `source.type-mismatch`，stage parse）、
  `g03.bound-missing`（无遵从记录 → `source.unsupported`，stage parse）为 AV 腿具名拒。
- A2R/A2C 不走 emit-only 快照充运行：其真实编译+运行义务由 `cargo t ace-a2x` 的
  `generic_records_963` / `const_instances_963` 承载（本轮已单例重放全绿），语料注释内写明该分账。
- 验证：`cargo t case-ace-m2-generics-963` → **PASS**（run-ff01yaf4；10/10 check PASS）。
  首次跑 `run-ol2w8tdg` 仅 `point-negative` 期望码写窄（写 `type-mismatch`，实为
  `source.type-mismatch`），按**实际诊断码**校正期望后复跑全绿——未放宽为"任意诊断通过"。
  退出码/身份证据：AV 腿 `exit 42/300/5/42`，mn 腿 `exit 42/300/5/42`，
  suite 身份 `83638d62…`，各 case 源 SHA 见 results.json。
- 同片复核（本窗口全绿）：`cargo test -p ace-source-query --test instances_const_963`（12/12）、
  `cargo test -p ace-av --test conformance -- const_instances_963`、
  `cargo test -p ace-native --test conformance -- const_instances_963`、
  `cargo test -p ace-a2x-worker --test conformance -- const_instances_963` ——
  V-G04 值参数（`rep<2>(21)`=42 / `rep<3>(21)`=63、身份/规范化去重/拒）四端真实执行已闭。
- **仍具名未闭**：记录类型模块全局的 `.move` 转移（见上一条 §9）；
  T-08 的默认/命名/可变实参、方法·扩展优先级、函数值签名、返回推导尚未施工。

2026-11-13 stage:work（T-03 关联类型 / super-spec 分片已提交；仍无终裁）/ PLAN-963 r5：
提交 `c8bbeace1`（8 文件 +736/−70）落唯一树 `D:/autostack/.wt/lang-963/auto-lang`
（`plan-963-dev`）；闭合上一片具名未闭的第 2、3 项（关联类型、super-spec），
`#[with(T)]` 按具名迁移拒落地负控。

- 语法（record / record-instances / aggregates 面）：`spec-item` 增 super-spec 列表
  （`spec S: A + B`；新 `super-specs` / `super-spec-tail`）；`spec-member` 两 alt
  （`fn` 方法 / `type ident` 关联类型）；`spec-args` 改 `spec-arg-list`——位置实参 +
  `Item=int` 命名绑定靠 `=` 后缀区分，不靠首 token 分派（同名首 token 变体不能并存的引擎约束）。
- AST / 投影：`ConformRef.assoc: Vec<(Ident, TypeAnn)>`（书写序，投影期消化，不进 AST 之外）；
  `SpecDecl { type_params, supers, assoc, methods }`；`apply_spec_conformance` 改 claim 模型
  （每条遵从 → spec 方法义务闭包 + 具体化替换表），命名绑定左值必须裸名、重复名
  `source.duplicate`、未绑定关联类型 / 未知绑定名 / 未知 super-spec / super-spec 成环 /
  通用 super-spec 一律具名拒（`source.unsupported`，不静默当无约束）；`binding_ok` 保证
  绑定错误时不再报"签名不符"掩盖真错（`first_diag()` 只回 span 最小一条）。
- 测试：core `source_spec_super_assoc_963.rs`（SA1..SA11：实例化替换、未绑定/未知/重复绑定、
  通用+命名混用、super-spec 义务传递、缺方法、未知、成环、通用 super-spec 拒、
  `#[with(T)]` 具名迁移拒）；native `spec_assoc_963.rs`（关联类型方法 exit=42、
  super-spec 默认体 exit=42，mn 真实构建+运行）。
- 验证：`cargo test -p ace-core`（lib 38 + conformance 451）全绿；`cargo test -p ace-semantics
  -p ace-source-query` 全绿。固定组 run-djwtkgdd（10:05 起，wall 1336.5s）五组 PASS，
  `ace-a2x` 单例 `worker_ct_941_ordinary_same_names_four_operations` 红
  （`CToolError("cl -? spawn: Deadline")` 56.7s），隔离重放 ok 19.41s → 环境归因
  （该组并行 C 工具链负载），本片不改该面代码。

2026-11-13 stage:work（V-G06 跨包记录模板片已提交；仍无终裁）/ PLAN-963 r5：
提交 `4f0967829`（6 文件 +380/−33）在包模块面闭合 V-G06 的「两包同名模板/类型 +
跨模块实例身份」。

- 语法（packages-modules 面）：`type-item` 的 `type-params` 与 `ty-word` / `ann-ty` 的
  `generic-ty` 追加 packages-modules 门——此前 `pub type Point<T>` 声明与 `Point<int>` 注解
  在该面均报拼写不符；补门后其余 profile 的过滤后程序不变。
- 绑定（`packages/binding.rs`）：`Walker.tyvars`（当前记录模板的类型参数名）——模板内
  `TyVar` 放行、模板外仍具名拒；新增 `TypeRef::Generic` 分支（基名按符号 mangle、实参递归
  mangle）；`CtorNamed` 的显式类型实参一并 walk（此前 `..` 丢弃）；模板方法/模板函数仍具名拒。
- 单态化（`cst/project.rs` / `frontdoor.rs`）：新增 `project_program_deferring_templates`
  （包模块投影跳过模块内单态化）与 `monomorphize_package_program`（仅程序级）；
  `bind_packages` 在扁平程序拼装完成后统一实例化——实例符号由已 mangle 的模板符号派生
  （`t3_libm3_geo_Point_int`），两包同名模板天然不串、跨模块同一实参共享同一实例；
  模板项随单态化移除，故实例方法 `owner` 重指实例、`type_origins` / `origins` 补登记
  （否则 owner 悬空）；诊断经 `locate_flat` 归回模块源坐标；`initializers` 改存符号、
  值体从扁平程序取，避免克隆漏改与二次平移。
- 测试：`packages::binding` 三例（跨模块模板 + 实例身份/成员归属/源位置、两包同名模板不串、
  错 arity 归模块具名拒 `source.unsupported`）+ 一例非模板 `T` 具名拒
  （`source.module-symbol`：unknown type `T`）；`source-query` 一例端到端
  `package_generic_record_templates_reach_checked_hir_963`（atom text 只见实例名、模板不抵达
  checked HIR）。
- 验证：`cargo test -p ace-core`（lib 38 + conformance 451）全绿；`cargo test -p ace-semantics
  -p ace-source-query` 全绿；固定组 run-1w1lnge5（wall 1310.7s）五组 PASS（executed
  489 / 132 / 268 / 1074 / 233），`ace-a2x` 仍是同一单例
  `worker_ct_941_ordinary_same_names_four_operations` 红
  （`CToolError("cl -? spawn: Deadline")` 106.2s；两次连红同一例），本轮隔离重放
  ok 10.20s → 与上一片同因环境归因（并行 C 工具链负载），非本片触面。
- **本片具名未闭**：记录类型的模块全局（`pub const ORIGIN Point<int> = Point { x: 40 }`）
  在 verify 被 `record param 0 requires a borrow argument` 拒——`module.global.set:` 伪签名
  只有借用形，owned 跨参转移（`.move` 实参 + MoveOwn 形参）属 V-G06 的 owned 账，未在本片
  实现；现为具名拒（不静默通过），下一轮连同 `.move` 转移补齐。

2026-11-13 stage:work（T-03 别名/ext/通用 spec 分片已提交；仍无终裁）/ PLAN-963 r5：
本片落唯一树 `D:/autostack/.wt/lang-963/auto-lang`（`plan-963-dev`）提交 `443a4e9f9`
（13 文件 +1033/−29），固定组六组一次全绿（run-76q2bqzs，见下）；V-G05 的 alias / ext /
通用 spec 子项闭合，`with` / 关联类型 / super-spec 留具名未闭（拼写已从旧用例取得）。

- 语法（`experimental/ace/grammars/auto-source.at`，record / record-instances / aggregates 面）：
  `type-item` 末尾改由独立 `type-tail` 按首 token 分派——`=` 别名形 `type N = T` vs `as`/`{`
  记录体形（本 CST 引擎的 `alt` 只作用于整条规则、不能中途分支）；`spec-item` 增
  `type-params`（`spec Storage<T>`）与 `spec-args`（`as Storage<int>`）；新 `ext-item`
  （`ext Type { members }`，首 token `ext` 独立分派）。
- AST：`ConformRef { name, args }`（`TypeItem.conforms: Vec<ConformRef>`，书写序；
  `parser.rs` 旧面 `conforms` 恒空）、`SpecDecl.type_params`；`kinds.rs` 增 `ext_item` /
  `spec_args`。`instances.rs:272` 同步取 `c.name.name`。
- 投影（`cst/project.rs`）：`run_spec_item` 对 `const` 参数具名拒（spec 只收类型参数）；
  `project_conform_ref`；`instantiate_spec_method`（签名与默认体一起替换）；
  `apply_spec_conformance` 增 arity 校验 / 替换 / 注入；`record_specs` 改
  `BTreeMap<String, Vec<ConformRef>>`。遍序：别名展开 → ext 合入 → 泛型记录单态化 →
  遵从检查与注入 → **再展开一次别名**（spec 默认体注入的别名首遍覆盖不到）。
- 测试：core `source_alias_963.rs`(5)、`source_ext_963.rs`(5)、
  `source_specs_generic_963.rs`(6：O-963-S1 int/bool 两实例各自具体化、S2 少给实参拒、
  S3 多给实参拒、S4 非通用 spec 给实参拒、S5 `<const N int>` 具名拒、S6 默认体别名注入后展开)；
  native `alias_963.rs`(1，exit=102)、`ext_963.rs`(1，exit=42)、`spec_generic_963.rs`(1，exit=42)，
  均经 mn 管线真实构建+运行、oracle 闭式手算。分片验证：`cargo test -p ace-core --test
  conformance` 440 passed、`cargo test -p ace-source-query` 全绿、`cargo test -p ace-native
  --test conformance -- alias_963 ext_963 spec_generic_963` 3 passed；触碰文件 rustfmt `--check` clean。
- 固定组：`cargo t ace-source ace-syntax ace-semantics ace-av ace-native ace-a2x`
  → run-76q2bqzs（09:32 起跑，wall 1353.7s）六组逐项 PASS：executed 474 / 132 / 267 / 1074 /
  231 / 226；`gate_passed=True`、`discovery_complete=False`（发现面标记，未影响六组执行终态；
  与 T-04 记录中 `ace-a2x` 的 process-tree 环境残留不同，本轮 a2x 亦 PASS）。
- **本片具名未闭（下一轮实施，拼写不猜）**：
  1. `with` 旧拼写 `#[with(T)]` / `#[with(T as Clone)]`（Plan 061 注解形，见
     `crates/auto-lang/test/a2c/08_generics/004_with_constraint`、`a2r/16_interop/019_multi_bound`）——
     ACE 无语言属性语法：`semantics-contract/src/metadata.rs` 的 `visibility.attributes` 仍是
     `MetaSupport::SchemaOnly`、`future_owner: "language attribute grammar plan"`，属性语法属他计划
     owner，本包不新造；规范形为 `fn f<T>` / `fn f<T has S>`，故按**具名迁移拒**处理（不静默忽略）。
  2. 关联类型 `spec Container { type Item }` + `as Container<Item=int>`（旧 case
     `a2r/12_specs/011_associated_types`、`012_generic_spec_assoc`）。
  3. super-spec `spec Tool: Send + Sync { .. }`（旧 case `a2r/12_specs/010_spec_supertrait`）。

2026-11-13 stage:work（T-04 const 值参数分片已提交，V-G04 闭合；仍无终裁）/ PLAN-963 r5：
本片两提交落唯一树 `D:/autostack/.wt/lang-963/auto-lang`（`plan-963-dev`）：
`d761ad7c5`（const 值参数全链路：语法→投影→语义输入→求解需求→身份/去重→体代入→脱糖重绑
→页 codec v4；28 文件）与 `55492a50c`（V-01 跨包同名模板身份 + late 拒测试）。

- 语法（`experimental/ace/grammars/auto-source.at`，仅实例面 `scalar-instances` /
  `record-instances` / `text-record-instances`）：`type-param` 第三 alt `const N`（可选声明类型经
  新 `const-param-ty`，仅引用该 profile 内既有词 `int-kw`/`i32`/`bool`/`ident`）；新
  `generic-call`（**括号属原子自身**：`ident < const-args > ( args )`）、`const-args` /
  `const-arg-tail` / `const-arg` 列表；`condition-atom` 与 `atom-expr` 在 `ident()` 前插
  `alt(profiles:[...]) { generic-call() }`。plain profile 过滤后程序不变。
- AST/投影：`TypeParamDecl.kind` + `TypeParamKind::{Type,Const}`、`ConstArgUse{value,span}`、
  `ExprKind::ConstCall{callee,consts,args}`；`project_type_param_decls` 合法化 const 声明，
  非实例面/非 `u32` 声明类型具名拒（`source.instances-unsupported`）；`enter_atom` 读
  `const-arg` 的 `lit` 拼写，`> i32::MAX` → `source.literal-range`（i32 体域），`> u32::MAX`
  → `source.literal-range`（u32 域）。`closure_capture`/`shift`/`packages::binding` 穷尽性臂；
  `resolve` 具名拒"must be instantiated before resolve"。
- 语义/实例：`InputFn.const_params`、`SignatureView.const_params`、`InputExprKind::ConstCall`；
  `TplDemand.consts: Vec<u32>` 与 Call/ConstCall 共用路径，值实参 arity 准入
  `value-arity-mismatch`/`value-argument-required`；`InstanceRecord.consts`、身份 `instance_key`
  按**声明序交错** `(type|value)` 项，装配名 `_u32_<十进制>` 只作拼写；`subst_block_consts`
  把模板体 `Var(值参数名)` 代成 `IntLit`；`rebind_callsites_owned` ConstCall→Call；
  新增 `program_has_value_call`：模板为空但存在值实参调用时不再恒等返回，使 `id<2>(1)`
  得具名 arity 诊断而非笼统 resolve 拒。`signature_canonical` 写 `\0value-param\0`+name
  （无值参数签名逐字节不变），表达式编码新增标签 `G`。
- 页（`instance_host.rs`）：`INSTANCE_PAGE_CODEC_VERSION = 4`，`InstancePage.consts:
  Vec<(String,u32)>`，编解码新增 `constc` 字段；v1..v3 页按版本不符 MISS 重产，信封不变。
- 测试：`source-query/tests/instances_const_963.rs`（新建 12 例）正例
  `const_param_instance_is_produced`/`distinct_values_are_distinct_instances`（logical 相同、
  key 不同）/`same_value_is_normalized_to_one_instance`/`explicit_u32_declaration_is_accepted`/
  `mixed_value_and_type_params`（`wrap_u32_7_i32`）/`value_is_substituted_in_the_template_body`；
  负例 `missing_value_argument_is_rejected`/`extra_value_arguments_are_rejected`/
  `value_argument_to_plain_fn_is_rejected`/`out_of_domain_value_is_rejected`（含 i32 体域）/
  `non_u32_declared_type_is_rejected`/`without_capability_value_params_are_rejected`。
  四端真实运行 oracle：`rep<2>(21)`→42、`rep<3>(21)`→63、两者相加→105（2 实例），
  分别在 `crates/av/tests/cases/const_instances_963.rs`（`Value::I32`）、
  `crates/native/tests/cases/const_instances_963.rs`（exit code = main 结果）、
  `crates/a2x-worker/tests/cases/const_instances_963.rs`（A2R rustc `assemble_file_set` +
  A2C cl `cold_assemble`/`compile_and_run_c`，断言装配名 `rep_u32_2`/`rep_u32_3` 且无
  `rep<const` 残留、stdout `value:42/63/105`）。页 v4 往返与跨包身份：
  `source-query/tests/instance_host_935.rs::page_codec_v4_carries_value_arguments`、
  `same_name_templates_in_two_packages_stay_distinct`（跨包 key/logical 互异；第二次 commit 按
  `LateResult` 拒，produced=2，不混装）。935 面用例
  `core/tests/cases/source_instances_935.rs::constraint_and_const_shapes_named_rejections` 迁为
  "`<T has S>`/`<const N>` 合法 + kind 断言；`<const N bool>` 具名拒；无能力仍 parse 拒"；
  `syntax-grammar/tests/cases/profiles_927.rs` 的 `DECLARED` 补 `record-instances` /
  `text-record-instances`（与 `auto-source.at` 声明序对齐）。
- 验证命令与结果（`experimental/ace` 下，`CARGO_TERM_COLOR=never`）：
  `cargo t ace-source ace-semantics ace-av ace-native` → run-vmk8psje 458/262/1074/228 全 PASS，
  `gate_passed=True`，wall 1102s；`cargo t ace-syntax` → run-xt7yfrow 132 PASS；
  `cargo t ace-a2x` → run-_h_zh1sd `executed=226 outcome=PASS`（日志 `PASS: ace-a2x`），
  `gate_passed=False` 仅因 `[process-tree infrastructure] descendants live after workload exit`
  （`vctip.exe` 残留）与 `discovery_complete=False`，按环境归因，非代码失败。
- **本轮具名未闭**：V-G05（泛型 spec `spec S<T>` / 关联类型 / super-spec / ext / alias / `with` /
  默认或命名实参——grammar 中完全不存在，属大特性）、V-G06（跨模块/两包同名模板调用受
  `packages/binding.rs` 的 `source.module-templates` 与单模块 `instantiate_program` 限制；
  owned 跨参借用未验）、V-02/V-G07（约束或实现变化使实例失效、冷编一致、未读 body 零生产）、
  §5.3 永久 fixture 目录 `crates/auto-lang/test/cases/ace-m2/generics/`（`.case.ad`/`.suite.ad`/
  `src`/oracle/`binding.tsv` 与 certify 接线）、T-08 剩余（默认/可变实参、ext 优先级、函数值
  签名、返回推导）。按用户裁定，本轮不做 T-06/T-07（merge/归档/清树）。

2026-11-12 stage:work（T-03 两分片已提交，仍无终裁）/ PLAN-963 r5：第五片 `c44d80ad6`
（spec 声明 + `as` 遵从，投影期擦除；测试补片 `9bfad9593`）与第六片 `2ef9f3874`
（`<T has S>` 类型参数约束）落唯一树 `D:/autostack/.wt/lang-963/auto-lang`，两片均先跑
固定组再提交。

- 语法（`experimental/ace/grammars/auto-source.at`，均为 record/aggregates 面新增）：
  `spec-item` + `spec-member`（`spec Name { fn m(..) R [body] }`，复用既有保留字 `r-spec`）；
  `type-item` 在 `type-params` 后新增 `opt(as-spec-list)`（`as A[, B]*`）与 `as-spec-tail`；
  `type-param` 的 `has S` 约束从 scalar/packages 面扩到 record/aggregates（`type-param-constraint`
  同步），`const N` 形状保持可解析、投影具名拒。
- AST：`TypeItem.conforms: Vec<Ident>`（书写序）、`TypeItem.type_params: Vec<TypeParamDecl>`
  （`TypeParamDecl { name, bound: Option<Ident> }`）；`parser.rs` 旧面 `conforms` 恒空。
- 投影（`cst/project.rs`）：`SpecDecl`/`SpecMethodDecl` 侧表，**spec 不产生 `Item` 变体**、不进
  resolve/四端；`apply_spec_conformance` 在 `expand_user_generic_types` 之后运行，未知 spec /
  缺抽象方法 / 签名不符（参数位与返回位按 `TypeRef` kind 比较）/ 重复声明 / 遵从列表重复
  均为 `source.unsupported`/`source.duplicate` 具名拒；spec 默认体作为 View 方法注入实现者
  （显式实现优先）→ 方法调用仍是既有静态解析，零后端改动。
- `<T has S>`：`GenericExpand` 持 `spec_names` + `record_specs`（记录名→遵从列表，含已合成实例），
  合成前按实参记录核验 bound；未知 bound 名/标量实参/未遵从记录均具名拒，不合成“无约束”实例。
- 证据：core `source_generics_963.rs` O-963-10..24（含 spec 擦除断言、`Pair<T> as Summable`
  实例级核验、`Box<Pair<int>>` 嵌套传递）；av/native/a2x-worker `generic_records_963.rs`
  追加 spec 显式 37 / 默认体 7 / 覆盖 9 与 `Box<Score>.total()`=5（A2R rustc + A2C cl 双端）。
  命令与结果：`cargo t ace-source ace-syntax ace-semantics ace-av ace-native ace-a2x` → 前五组 PASS
  （run-ruayk7_a 之后 23:20 轮 run-ar94p7vb：ace-source 458/ace-syntax 132/ace-semantics 237/
  ace-av 1071/ace-native 225 全绿）；同轮 `ace-a2x` 单项
  `a2c_rt_view_method_body_warm_edit_phases` 红，隔离单测重放绿、单独 `cargo t ace-a2x` 整组
  PASS（run-ddwo256h）——按环境归因（该组并行 rustc/cl 重负载时瞬时红），不作为代码失败。
  分片测试 `cargo test -p ace-core --test conformance source_generics_963` 24 passed、
  av/native/a2x `generic_records_963` 各 7 passed。
- **T-03 具名未闭**：泛型 spec（`spec S<T>`）、关联类型、super-spec、ext/alias/`with` 未实现；
  fn 级 bound（`fn max_of<T has Comparable>`）仍在 `crates/source-query/src/instances.rs` 被
  `source.instances-unsupported` 拒——其放行需先放宽实例车道类型域（本轮复核：`crates/semantics/
  src/infer.rs` 的 `Ty` 仅 I32/Bool/Var，`input.rs` 把记录构造/字段/方法表达式投影为 `Unsupported`，
  故放宽面含求解器 Ty/输入投影/统一、`TypeDesc` 映射、`PortableTy`（`facts.rs`）与
  `instance_host` 页 schema/codec，属 T-02 剩余核心，未在本片范围）。

2026-11-12 stage:work（T-02 分片进行中，仍无终裁）/ PLAN-963 r5：本会话在唯一树
`D:/autostack/.wt/lang-963/auto-lang`（`plan-963-dev`，基面 `990d27fcc`）落三片，均先跑固定组再提交：
`ea0d6a82`（935 实例车道多类型参数：`InstanceArg`/`InstanceRecord.args`、页 codec v2、四端多参）、
`b447112bc`（用户泛型记录语法期单态化：`type Point<T>`→`Point_int`/`Point_bool`，四端 300/301/双实例，
`core/tests/cases/source_generics_963.rs` O-963-1..9）、`6b06003f7`（显式类型实参构造
`Point<int> { .. }` 独立原子规则 + 全表达式构造点换名 + 无模板时仍具名拒，四端同三例）。
证据目录 `D:/autostack/plan-artifacts/plan-963/`（不入库）。验证：`cargo t ace-source ace-syntax
ace-semantics ace-av ace-native ace-a2x` 六组绿；`differential_corpus_all_profiles` 因 record 面新开
泛型类型应用，V02/V03 在 Record 面两侧仍拒绝（旧=语法错、新=profile 具名拒），已加具名白名单
`PROFILE_EXTENSION_DIFFS` 并在注释登记为本包既有口径变更；`sema_boundary_core_tree_has_no_product_backends`
首次红系本机 `CARGO_TERM_COLOR=always` 污染 `cargo tree` 解析，`CARGO_TERM_COLOR=never` 重放绿（环境归因）。

2026-11-12 stage:work（T-02 值域放宽片已提交，仍无终裁）/ PLAN-963 r5：第七片 `33ebeafcc`
（实例车道值域扩到文本/记录 + 约束见证，26 文件）。要点：

- 语法（`grammars/auto-source.at`）：新增两个**能力裁剪面** `record-instances` /
  `text-record-instances` —— record / text-record 全表面 + fn 类型参数；门控按 profile 追加，
  既有 profile 的过滤后程序不变（grammar identity 不动）。
- 声明形状：`FnItem.type_params` 由 `Vec<Ident>` 统一为 `Vec<TypeParamDecl>`，fn 模板与 type
  模板共用 `<T has S>`；`source_duplicate` 统一重名诊断（原 fn 面为 `source.instances-unsupported`）。
- 推理（`semantics`）：`Ty` 加 `Text`/`Record(name)`；`Ann` 加 `Text`；`InputExprKind` 加
  `StrLit`/`Ctor`/`Field`/`MethodCall`（原四类 `Unsupported` 拒绝删除）；字段/方法检查改为
  **延迟约束**（`FieldRead`/`MethodCallCheck`）在 `finish` 前有界不动点解出，覆盖
  `pick(a,b).val` 这类接收者类型晚于使用点可知的后序场景。
- 实例（`source-query/instances.rs`）：`InstanceTy{I32,Bool,Text,Record(String)}`（`PortableTy`
  保持 934 冻结面不扩）；模板 `bounds` 按实参记录的 `conforms` 核验，不满足 → 新诊断码
  `source.instances-bound`（点名模板与 spec）；`INSTANCES_ALGO` → `ac935/instances-discovery/3`。
- 页（`instance_host.rs`）：codec v3（类型码 + 记录名）；v1/v2 页按版本 MISS 自然重产，信封不变。
- 产品面：新增 `compile_record_product`（record profile 自有车道，无 foreign 环境），
  `walk.rs` 把文本/记录表达式改为结构化节点（原先按表达式种类整体拒）。
- 证据：`source-query/tests/instances_value_domain_963.rs`（10 例，V-G01 标量/文本/记录替换
  身份、同替换共享、异替换不串、错类型 dispatch 前拒；V-G03 满足生成/不满足具名拒/缺成员拒/
  字段与方法绑定/构造字段负控）；`instances_multi_963` 迁 v3；935 面用例按统一 `TypeParamDecl`
  与约束合法化更新。命令（`experimental/ace` 下，`CARGO_TERM_COLOR=never`）：
  `cargo test -p ace-core -p ace-semantics -p ace-semantics-store -p ace-source-query` 全绿
  （core 含 conformance 15 个 test-result 块，source-query 91+1+13+5+3+10+4+2+2）。
  两例 `::performance::` 内存预算用例在并行构建负载下红（`infer_body_memory_stays_bounded`
  等），按 PLAN-1016 不属日常组，未纳入本轮判定，留 T-04 单实例复核。
- **T-02 具名未闭**：V-G02（用户泛型记录字段/嵌套/方法参数面）、V-G04（const 值参数）、
  V-G05（generic spec/关联类型/ext/alias/`with`/默认或命名实参）、V-G06（两包同名模板与
  owned 跨参借用）、V-G08（四端真实运行 oracle）与 §5.3 永久源目录
  `crates/auto-lang/test/cases/ace-m2/generics/` 尚未创建；均在本包 T-02/T-03/T-08 范围。

**具名未闭缺口（T-03 前置）**：`source-query` 的泛型函数实例车道仍以 `PortableTy`（Unit/I32/Bool）
为解空间，`spec/conform/ext` 见证与 V-G03（`max_of` over 用户记录）需先把实例实参放宽到
记录/文本（`TypeDesc`→`PortableTy`→页 schema 同步），该放宽是 T-02 剩余核心，尚未开工；
`spec`/`as` 关键字与 `Item` 变体在 ACE 语法/AST 中目前完全不存在。

2026-11-12 stage:work（进行中，无终裁）/ PLAN-963 r5：goal 授权整链（work→review→bounded repair→merge）已记录；实现树 `D:/autostack/.wt/lang-963/auto-lang`、分支 `plan-963-dev`、基面 `990d27f`；材料 scope `D:/autostack/plan-artifacts/plan-963/`。主检出开工前核：代码路径零 WIP（仅他会话 docs WIP，未触碰）。本轮只交 T-01 冻结（§5.5，全部结论带 path:line），未建 fixture、未跑 ACE/旧 VM、未改代码；T-01 尚未闭合（三探针待实现）。

2026-11-12 stage:new / PLAN-963 r5：按用户指出的语法/设计/测试缺失补订。保8任务/原AC与V/依赖、drafting 0/8；新增既有Auto具体源/独立预期、旧实现证据差异、ACE拟类型/模块/实例设计、旧新义务映射及V-G01..08。只读源码与原用例，没有实现、fixture生成、旧VM/ACE运行、独立产品复审、建树、派agent或提交；本次修正文档缺口，不将依赖就绪包装为产品支持。

2026-11-12 r5文档静态预检：0–10章节完整；原8任务/7AC/全部旧V及SD保留，现15验证行/4条SD，阶段/任务与AC引用一致；8处本地链接/锚点、6个拟职责组名、9个具名源码SHA均核过；counter与本轮起点相同。未运行产品或旧VM/新ACE用例，此记录只证明规划结构与来源核对，非实现/独立产品review PASS。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
