---
plan_id: PLAN-1088
status: archived
completion_kind: delivered
feature_name: AV泛型、spec与方法决议组合补齐
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 4
current_step: 8
total_steps: 8
depends_on: [PLAN-1078, PLAN-1080, PLAN-1081, PLAN-1083, PLAN-1085, PLAN-1086]
supersedes_spec_components: [docs/specs/ace-migration/m2-generics.md, docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1088] AV泛型、spec与方法决议组合补齐

## 0. 变更摘要

### 2026-11-20 r4 · 普通类型解析驱动实例需求

r3 检查点 `fd8aa825a` 已验证标量/const/递归实例及部分语料。T-01 的四个组合反例表明 963 的有限求解适配器覆盖不了普通源的资源与函数值表达式。普通单文件和包闭包改以现有绑定器的实参类型产生需求，仍复用实例替换、键与受检管线。spec 类型值先验证现有名义枚举载荷能承接分派与所有权，再接入既有见证。保持全部 AC、任务 ID、旧用例、原 oracle、仓库与 AV 阶段边界，继承 §4 授权。新机制的观察与待验证范围见 §5–6。

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`generics/spec-methods.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM 语言齐平路线批次 **AVP-10**。现状和历史规划收据见 §9。总路线见[AV语言齐平设计与执行路线](../../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

把963已交模板/约束能力接入完整普通工程，关闭方法、局部注解和跨模块组合差额。

本包交付`IF-AV-PARITY-GENERIC`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一 `AutoDown → TestCase author/CaseSpec → 普通 Auto Schema/CST → 包/作用域绑定、实例需求与具体签名 → Checked HIR → 共享 MIR/ABC 验证 → AV → 父侧独立 oracle`。AVP-00 的 `ac-av/language-source-run-v1` 已落地；r3 检查点已经接通部分普通实例。r4 由普通源码解析产生需求，复用既有实例替换和验证。冻结窄 profile、旧实例页和其它后端消费保持原身份。

- 泛型函数/类型/方法、类型推导与显式实参、已定const参数域、递归/跨模块实例、资源payload和函数值参数。
- spec super/关联类型/默认体、多个候选优先级与歧义、ext可见性/receiver、泛型局部type annotation准确诊断。
- 实例身份/失效与同包同名不同版本、签名与body变更、恢复冷编一致；#[with(...)]由metadata批消费同一见证/约束实现。

## 3. 技术栈

AutoDown 文档式 `.case.ad`、既有 auto-test-case/auto-case/worker、ACE 共享 Rust 实现。实施树 `D:/autostack/.wt/lang-1088/auto-lang`，分支 `plan-1088-dev`，检查点 `fd8aa825a7e010cdabb7118586ed01455f646048`。组内 `auto-down@45b642d` 为 detached 只读依赖。材料统一登记在 `D:/autostack/plan-artifacts/plan-1088/`。

## 4. 需求分析与背景调查

2026-11-20 用户以 `/goal 完成计划1088` 授权合同内的实施、独立复审、修复、合入归档与清理。r4 沿用该授权。阶段选择只执行 AV 与必要共享正确性验证，保存非 AV 的历史条件、独立 oracle 与后续 owner。计划初稿的规划授权及 r2/r3 阶段选择历史保存在 §9 和 Git。用户没有指定额外预算或工期。

963已交多参/const模板、spec/关联类型/ext，不是重建泛型；局部泛型注解、属性遵从拼写与实例/元数据组合仍有限。属性语法由后续metadata批统一，避免本批再写parser。

既有来源与边界：

- [generics.document.case.ad](../../../crates/auto-lang/test/cases/ace-m2/generics/generics.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [ext_method.case.ad](../../../crates/auto-lang/test/vm/99_idiom2/m07_ext_method/ext_method.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：24_generics；28_enum_methods泛型/receiver；99_spec_dispatch；99_idiom2/m07/m08/m09；旧generic/spec Rust tests。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [project.rs](../../../experimental/ace/crates/core/src/source/cst/project.rs) | `2223bfbbf79279bd4fc88596febc6dd455a477bd793832084ab8ffcba2ff38cc` |
| [binding.rs](../../../experimental/ace/crates/core/src/source/packages/binding.rs) | `aa2cee1daf9ce0912f2de4952bd9618070c77807e0af8dcf9b5a2e36c0483f6d` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |
| [instances.rs](../../../experimental/ace/crates/source-query/src/instances.rs) | `88dab4f207038c80af4f9dade04c0b4f61445abd3886b30d4051d920c2ce09c3` |

硬依赖：PLAN-1078, PLAN-1080, PLAN-1081, PLAN-1083, PLAN-1085, PLAN-1086；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078 已将文档检查级 `#ignore` 接到现有 `CheckSpec.skip/SkipSpec` 并落主。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### r4 输入、真实代码路径与所有权

单文件入口为 `core::source::adapter::compile_language_source_with`；多文件入口为 `source_query::modules::packages::compile_package_product_auto`，后者先调用 `bind_packages_auto`。`resolve.rs` 的 `resolve_detailed_in_mode` 预注册具体声明与函数签名，实参进入 `resolve_call` 前已有 `ResolvedArg`，`VT` 和 `CollTables` 已区分借用/owned 字符串、列表元素、枚举及函数签名。`VT` 中的类型表下标只在一次绑定有效。上一轮在 `instantiate_program → discover_round → project_body_scoped_with_caps → InferSession` 中复现四项前置失败；其旧实例 schema 与路径继续服务冻结窄域。直接用已有 enum 含两个记录 payload 真实执行得 42，证明现有 AV 表示能区分两个实现；源端自动包装、super/关联类型和所有权仍须验证。

| r4 检查点来源 | 文件 SHA256 |
|---|---|
| [`project.rs`](../../../experimental/ace/crates/core/src/source/cst/project.rs) | `db009ead254cd3a8c1e708c92a4789cbcd1327a6b36c02462da4c7d200271d95` |
| [`resolve.rs`](../../../experimental/ace/crates/core/src/source/resolve.rs) | `42b3bd759d2c57a6b901b46122a5f50327a9ed5b2a5162d978247e7c64b0a4f4` |
| [`binding.rs`](../../../experimental/ace/crates/core/src/source/packages/binding.rs) | `4759ce9d86a8af273ab9adc7c6bd71785841af2ffe7f21c25903fa791ded013f` |
| [`instances.rs`](../../../experimental/ace/crates/source-query/src/instances.rs) | `ebdca1c78a18ac06c8bb9b5ea4351f6d70b4f7842691548c1f5d64c45f3b111c` |
| [`packages.rs`](../../../experimental/ace/crates/source-query/src/modules/packages.rs) | `998643cf9c8115252952479b5b020c23f49aaaa411b79fa6aebf45c556fe4b64` |

**T-01 → T-02，需求与替换。** 在普通绑定已有的显式/命名/位置实参规则旁产生实例需求。使用当前调用的真实已解析实参及已知的期望类型，对模板声明形状求替换并检查同一类型参数的多个约束。固定类型保持现有绑定和类型检查，类型实参与值实参按声明序配对，冲突、缺参、歧义与未确定类型保原调用跨度。需求载体只保源级规范类型与声明身份。由实际类型表还原 owned/borrow、列表、名义枚举与函数签名，禁止把 `VT` 的索引或哑值当作实例身份。替换/常量代入和装配键沿用 963 已有行为；普通扩展域若需持久化新类型，使用独立版本并对不兼容页 MISS。一次受限发现可以暂留未就绪实例，完整候选必须以具体签名回到原 resolve/typecheck 路径，通过 Checked HIR 和 MIR/ABC 验证后才进入 AV。无需求模板无可执行体，递归复用同一已登记签名。沿已存在的 frontend/worker 预算分类处理失败，保普通程序不触发发现和实例化的快路径。首个实施切口为 owned `Str` 和具名函数值两个最小树，工作轮先观察实际 `resolve_call`/表达式消费者的取型与实例回填，证明通过普通 recipe 再扩同一路径。若调用点无可用签名，比较延后单次调用绑定与按依赖重放的闭包，不增加第二套普通类型规则。

**T-02/T-04，包身份及恢复。** 由 `PackageModuleInput` 的锁定版本和 `BoundPackages` 的声明来源绑定 `(package, version, module, symbol)`，在包绑定之后提交实例请求，跨模块重名实例仍区分声明，两个调用点共享同一声明身份和替换时只生产一次。签名轴、体轴分别用实际源码事实。体变更保持逻辑实例身份、更新产物键。签名变化按实际依赖触发重算或具名拒绝；恢复拒绝错版本、错替换和缺见证，冷编作为独立对照。仅当前成功并验证的关闭内容能成为可恢复产物。取消和异常保住上一已提交产物，不留 live owner 或半页，同时保成功重试。无跨仓实现变更。

**T-03，spec、ext 与旧组合。** 沿 `project.rs` 的遵从检查保留已核验的声明和方法见证，由普通绑定决议调用点上的具体成员、绑定方法或 spec 类型值。静态/实例/扩展候选执行既定优先级及同层歧义，并保源码 span 和模块可见性。spec 类型值的方案在绑定前将合规的具体记录接到现有名义 variant payload 与方法，禁止把猜测的名称作为分派或自动认定遵从。实施先用两个不同实现的同一 spec 参数及一个无遵从反例验证包装注入、分派和所有权；借用、move、调用后释放沿已验证表示。要求三份 `99_spec_dispatch` 原 oracle，包含空记录与嵌套字符串的真实语义。28.005 由现有枚举 payload/方法表示消费泛型实例，先探测两臂各只能提供一个实参的旧推导依据，核模式、接收者和构造身份，保两条原 stdout。`List<Note>.to_array` 与 `filter` 使用普通 AV 现有 typed list 与闭包路径，补充可见的元素、顺序与独立结果断言，原 `ok` 检查继续保留。

**T-05，输入与读者。** canonical 文档已落第一段在 `crates/auto-lang/test/cases/language/generics/spec-methods.case.ad`，suite 已引用。旧 VM `load_vm_test_cache` 仍读取 `test/vm` 迁移文档。对本包迁移行切换活跃者到 canonical 共源，保旧 VM 平台/条件/预期和历史文档身份；既有历史 M1/M2 签章输入保留。非 AV 用检查级 `#ignore` 或相应视图，父侧选择按已注册目标/动作。无 AV required 时父侧 gate 拒绝。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-generics.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-FC | modify（本特性消费/来源） | `docs/specs/auto-test-case/feature-organized-corpus.md` | 按特性唯一作者、同源多端检查、旧义务映射和本包迁移/观察来源；实际交付后沉淀，当前不宣称已支持 | 公共机制由1078生产一次，本包保原语言/目标义务 | AC-FC、V-FC-SOURCE/MIGRATE/TARGET |

`language-parity.md` 由 AVP-00 创建；本包按最终独立复审结果增录实际支持与来源。


## 6. 测试设计

### V-AV-ONLY / V-IGNORE-LIFECYCLE（未执行）

原T-03..06落实、T-07复核：同一Case AV已选，AC/A2R检查加#ignore；制造该目标缺工具/运行失败/错误oracle时，目标prepare/dispatch计数为0，AV仍按独立预期通过，忽略行不充PASS。将AV oracle改错必须使AV门红，全部有效AV检查被ignore或零选中也必须红。移除某非AV阶段ignore并进入其后端选择后，该目标失败必须在该后端门显现，而AV视图仍不选择它；移除ignore不能同时删原平台/ignore理由。公共格式/重复ID/AV必需shared规则出错不被这一阶段策略豁免。

### FC 测试组织与迁移必验增量（未执行）

| V-ID | owner_stage / 原任务 | 必须观察的结果 |
|---|---|---|
| V-FC-SOURCE | work / T-02..06 | 本轮同一source/project仅选AV真实执行/按规定阶段拒绝；改公共body使相关旧证据陈旧，非AV行保持#ignore/未选与后续owner，不要求本轮运行。 |
| V-FC-MIGRATE | work / T-01/02/05/06 | 原case/check/条件/判决角色→canonical映射完整；非等价邻例、错误/资源/恢复角色不丢；suite、worker/内联/工具读者均切唯一活跃作者，旧副本仅具名历史。 |
| V-FC-TARGET | work / T-04..06 | AV oracle变异、重复活Case ID/AV旧活入口残留、零条有效AV/缺AV required检查必红；AC/A2X oracle不符或缺工具在#ignore/未选时不进入AV门。非AV完整目标oracle/emit-vs-run控制留给其启用阶段。 |

[本包旧VM文档逐项清单](../../design/strategy/av-autovm-legacy-case-map.md#plan-1088-原文档清单)静态主承接 **7份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

以下四项由 r3 检查点落入 canonical 作者并通过实际 AV 检查，当前结果与提交身份见 §9。新增组合检查仍待实施。保持各具名诊断和实际拒绝阶段。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1088.generic-two-types` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1088.generic-spec-default` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1088.ext-method` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1088.bound-missing` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | resolve: bind/source.instances-bound | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `INSTANCE-COMBO` | AC-03/AC-04 | work / T-04..05 | 同一模板分别实例化owned Str/List/variant/fn值，跨模块调用；未用模板体不生产，实际用body变化须改变值。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `TYPE-DIAG` | AC-03/AC-04 | work / T-04..05 | 合法Box<int>局部注解与未知/约束缺失/元数错各自点名，禁止一律source.unsupported；const超域遵已定u32边界。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### r4 补充判决（以下新增检查均待实现和执行）

| 入口与义务 | 对照与可区分判决 | 所有权和覆盖 |
|---|---|---|
| `INSTANCE-COMBO`，T-02/T-04，AC-01/03/04/05，普通 `ac-av/language-source-run-v1` 和包闭包 | 四条带独立实际值的检查。owned `Str` 传进模板并返回，`"test"` 长度 4；List 带两个元素 19、23，相加 42；`Found(42)` 保留枚举身份与原 match 载荷 42，错臂给 0；具名函数值 `twice` 经同一模板返回后间接调用 21 得 42。负控传签名不符函数或错元素；保诊断码及原调用 span。 | 检查 `Str`、list、variant payload 的 ownership 保真，入口所需义务使用现有账/trace 检查关闭与重试。一个源码包含多个类型时仍保证需求/调用点计数，未调用模板体零产物。 |
| `TYPE-DIAG`，T-02/T-03，AC-02/03/04，同上 | 合法 `Box<int>` 保 42。未知、缺约束、错误实参数量各自给具名码及源 span；分列大于 i32 实际体域和大于 u32 声明域的 const 负控。 | 区分已知形状、resolve 诊断与投影数值域。保 r3 检查点 valid/invalid 原源。 |
| `INSTANCE-IDENTITY`，T-04，AC-03/04，实际普通多文件 AV + 生成/恢复路径 | 两个调用点相同替换只生产一次；同名包模板换锁版本，两个版本结果分别 42/43 并核身份变化；body 变更值 42→44 且签名轴不动；签名变更拒绝过期使用；故意递交另一版本产物被拒，冷编及恢复后新执行值一致。 | 单/多文件相同替换规则；记录实际产物读取、拒绝、重试与释放，账不从 exit 推断。 |
| `SPEC-DISPATCH` 与旧义务，T-03/T-05，AC-01/02/03/04，普通 AV | 两合规记录的 `value` 分别得 5/35，合计加 2 得 42；同形无 claim 的第三记录必须在原实参拒绝。再原样对账三份 `99_spec_dispatch`、28.005 和 `24_generics` 全部原 oracle。 | 对照验证真实动态 fns 选择与 List 结果内容顺序；空记录、原 FStr 以及 enum 方法输入均随原文档追踪。 |

### AutoDown代码用例（历史草案，现行作者见 `language/generics/spec-methods.case.ad`）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: generic-two-types
id: "av.parity.1088.generic-two-types"
tags: ["av-language-parity", "plan-1088"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn id<T>(x T) T { return x }
fn main() int { if id(true) { return id(42) }; return 0 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: generic-spec-default
id: "av.parity.1088.generic-spec-default"
tags: ["av-language-parity", "plan-1088"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
spec Picker<T> { fn pick(a T, b T) T { return a } }
type AsInt as Picker<int> { v int }
fn main() int { let x AsInt = AsInt { v: 1 }; return x.pick(41, 7) + x.v }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: ext-method
id: "av.parity.1088.ext-method"
tags: ["av-language-parity", "plan-1088"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
type N { v int }
ext N { fn twice() int { return .v * 2 } }
fn main() int { return N(21).twice() }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: bound-missing
id: "av.parity.1088.bound-missing"
tags: ["av-language-parity", "plan-1088"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
spec S { fn value() int }
fn get<T has S>(x T) int { return x.value() }
type Plain { n int }
fn main() int { return get(Plain(42)) }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "resolve"
code: "bind/source.instances-bound"

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
诊断阶段/码是本路线目标合同；新码或现有family包装差异由本包T-01核对/冻结，修正必须保持同一错误类别、源位置与拒绝时机，不能降为any-Err。
`````

### 执行命令与范围

在专属实施树中执行以下冻结提交验证。前期仅执行 §9 明列的定向和语料检查：

```powershell
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-artifacts
```

固定组 `case-av-language-parity` 已接入生产 `--target av` 视图，按已注册目标和动作选择 AV run，零选中、缺文件或坏子项必须报错。选 `--check av` 仅对应 ID 为 `av` 的检查。旧 VM 参考保 `#ignore`，该视图不启动旧引擎。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [x] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [x] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [x] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [x] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [x] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [x] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [x] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [x] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [x] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | 泛型函数/类型/方法、类型推导与显式实参、已定const参数域、递归/跨模块实例、资源payload和函数值参数。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | spec super/关联类型/默认体、多个候选优先级与歧义、ext可见性/receiver、泛型局部type annotation准确诊断。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 实例身份/失效与同包同名不同版本、签名与body变更、恢复冷编一致；#[with(...)]由metadata批消费同一见证/约束实现。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-GENERIC。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

任务进度：

- [x] **T-01** work：§9 输入审计与旧义务对账覆盖七条主承接、额外泛型 enum 与 m08/m09 的既有 owner。
- [x] **T-02** work：普通绑定器产生实例需求，函数、记录、枚举、方法、别名与跨模块 AV 观察见检查点 2–10。
- [x] **T-03** work：分派、静态约束、继承/关联类型/默认体、扩展可见性和局部诊断见检查点 1、8–10。
- [x] **T-04** work：包实例身份、冷编恢复、过期产物拒绝、资源/取消/故障回放见检查点 5、8、10；属性源码由 PLAN-1092 消费见证。
- [x] **T-05** work：canonical 语料、八条原义务迁移、读者和元控制见检查点 6–10。
- [x] **T-06** work：冻结实现和适用组结果见检查点 10，语义组既有失败单列。
- [x] **T-07** review：§9 最终复审定位实际入口、反控、源身份、唯一作者及规范。
- [ ] **T-08** merge：见上表唯一任务合同。

已落主、验证规范并归档。T-08 的清理收据待核验。

## 9. 复审记录

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

- 2026-11-20 `stage: goal | plan_id: PLAN-1088 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: a3655eb2ff2638cdbecc002b7bfd74122d6ab43a | next: work`。继承 §4 的端到端授权。基线 `auto-down@45b642d`（组内 detached，只读消费）；六项硬依赖已在 master 归档，普通 AV recipe 与固定组已落地。开始 T-01，材料路径 `D:/autostack/plan-artifacts/plan-1088` 已登记。

### T-01 输入与机制探测（在途）

- 基线 `a3655eb2f`，专属树 `D:/autostack/.wt/lang-1088/auto-lang`，`auto-down@45b642d`。六项所需归档与 master 上的 recipe、suite 和固定组已核对，旧 PLAN-963 的窄域实例页和同源测试留原位。
- 在同源 debug `av-worker case-serve` 上逐一观察四个 §6 草案、七份主承接及 1083 转交的 28.005。worker SHA256 `87339ba4b488d6146d87b897d4027076b4fd9daa7b38ecb4456b30dc2383b749`；命令 `python probe.py baseline-av-worker.exe baseline-observations.json` 退出 0，12 个请求均收到原始结果，单请求约 0.009–0.070s。明细 `D:/autostack/plan-artifacts/plan-1088/baseline-observations.json` SHA256 `bcdcf86e84e7bfdb592d1e9b996cbcade0cdfbd2f0f895f02b53623be41dcddf`。前期探针误读了顶层 envelope，已以 `ok.steps` 并校验身份重放。本组为诊断基线，不作父侧验收。
- `generic-two-types`、`bound-missing` 初诊断 `text/source.syntax`；默认 spec 和简单 ext 分别成功 42。旧 `001_list_struct` 运行但 stdout 为 `0`，原预期 `2`；`002_to_array` 在 `bind/source.undefined` 拒，`003_filter` 在旧 `fn(...) bool` 表达式形拒。旧 m07 的 `double` 使用了普通面的类型词，改成中性名的定向探针区分此因，正式迁移保 `double`。28.005 暂卡泛型 enum 声明。三份 `99_spec_dispatch` 至少需要 spec 类型值、空记录和 FStr 内嵌字符串的组合，原 oracle 仍逐份保留。
- 源码追踪：普通单文件 `compile_language_source` 在 Checked HIR 前缺 963 的 `instantiate_program`；借助编译准备接面消费原求解器与实例装配。模板需求负控在原调用 span 给 `bind/source.instances-bound`，在 document 作者处通过 `stage: resolve` 和 `code: source.instances-bound` 表达。独立对照为 `Plain` 不声明 `as S`，其余源形与合法调用共享。泛型局部注解另观察有效的 `Box<int>` 与具名未知 `Missing<int>`；多个同层 spec 默认体的对照为有/无显式实现。`List<Note>.new([…])` 的原 stdout 是构造元素被丢弃的独立反控，由现有 typed list push 与所有权规则验证。
- Debug worker 构建第三次 5.352s，退出 0；直接探针合计运行 22 次请求（12 合同/旧义务及 10 补充观察）。四份 §6 草案以及递归、const 域、局部注解、默认值歧义与显式覆盖均取得预期观察。无额外非 AV 目标运行。初始化修正及正式父侧全域语料结果还在核验，T-01..06 仍未勾选。

### 普通 AV 实例化阶段检查点

提交 `fd8aa825a7e010cdabb7118586ed01455f646048`，专属树 clean。保留原四个草案 ID、12 条新语料检查和两个同源 VM 原 oracle。`001_list_struct` 保留旧拼写且 stdout=`2`，m07 保留 `double` 且 stdout=`get=21\ndouble=42\nbumped=6`；原 VM 检查各分列 SKIP。canonical 文档只登记已跑绿两行，迁移读者审计尚待 T-05。普通源码实例需求目前沿 963 求解器进入 Checked HIR；无模板程序从 CST 判断并绕过实例化，继续经过 FStr 预处理与原有工作计量。

| 实际范围 | 结果 | 准备 | 执行 | 总墙钟 |
|---|---|---:|---:|---:|
| `cargo t case-av-language-parity --run-prepared`，实例探测快路径优化前 | 259 AV PASS、110 SKIP、0 FAIL，父子退出 0 | 27.568s | 22.859s | 26.725s（执行命令） |
| `ace-source-query`，预建后重放 `instances_value_domain_963`、`instances_invalidation_963`、`instances_const_963` | 12、5、12 PASS；失败/忽略/过滤均 0 | 14.913s | 0.151s、0.139s、0.145s（各二进制含进程生命周期） | 各自同执行列 |
| `auto-case run --target av`，本包 canonical 的同字节子套件，最终提交输入 | 14 PASS、2 SKIP、0 FAIL，退出 0 | 9.004s debug worker；父侧工具按前次收据复用 | 1.230s | 1.230s（执行命令） |
| AV oracle 与选择反控，优化前检查点 | 合法对照 1 PASS/1 SKIP、变异 AV oracle 1 FAIL/1 SKIP、忽略全部 AV 阶段检查退出 2 | 复用已绑定工具 | 0.453s、0.402s、0.018s | 各自同执行列 |

发现、清理与跨会话资源等待未单独计时。初次父侧准入因把 worker 的 `bind` 当文档 stage 而退出 2，改用作者协议 `resolve` 后重准入通过。快路径第一轮绕过 FStr 预处理，原样 m07 定向 13 PASS/1 FAIL/2 SKIP，已复用原解析入口并在最终检查点复绿。冻结准备与运行仍待 T-06 依实际完整实施重跑，以上未执行 AC/A2R/A2C/A2PY 或旧 VM。

| 新 AV 检查（`av.parity.1088.* / av`，末两行为原 VM 项的新 AV 检查） | 判决 | debug 毫秒 |
|---|---|---:|
| generic-two-types | bool 与 int 两个实例得 42 | 596 |
| generic-spec-default | 默认体与具体类型得 42 | 523 |
| ext-method | 普通扩展方法得 42 | 458 |
| bound-missing | `resolve/source.instances-bound`，原 `get(Plain(42))` 调用 span 116..130 | 425 |
| local-record-annotation | 显式注解和独立构造的记录得 42 | 607 |
| local-record-missing | 具名未知 `Missing<int>` | 491 |
| spec-default-ambiguity | 两个同层默认体具名拒绝 | 589 |
| spec-explicit-priority | 同一对默认方法由显式实现决议，得 42 | 497 |
| const-parameter | 显式常量 2 参与函数体，得 42 | 575 |
| const-out-of-range | 4294967296 超出 u32，具名拒 | 466 |
| recursive-generic | 递归调用在同一具体实例返回 42 | 584 |
| list-initializer-type-mismatch | `List<int>` 初始化布尔值，类型具名拒 | 550 |
| legacy.99_idiom2.m07_ext_method | 原三行 stdout，扩展写回可见 | 607 |
| legacy.24_generics.001_list_struct | 原 stdout `2`，两元素未丢弃 | 570 |

证据路径相对于 `D:/autostack/plan-artifacts/plan-1088/`。全组 `checkpoint-run/results.json` SHA256 `2505622150bbe34ba09bdfeF31fce30a75de21a759878951981fdd81636f50b8`。当前提交输入定向 `checkpoint-canonical-replay/out/report.json` SHA256 `b8f66296c15d74930ea991e9de01ca7124e608f7184d8120704f9ff3aea57561`，当前 worker SHA256 `09364d0920e2b71736176b986fe3ac23c0955c27d48b8cc1ff5ade012cdf76c0`。

### 执行设计需修订

T-01 的组合探针证实，963 的求解适配器无法作为普通 AV 全体模板需求的唯一类型来源。它在 owned `Str.clone()`、`List<int>.new()`、具名函数值与枚举模式四组原样探针上，分别在非记录方法、泛型接收者、未知局部名、聚合控制投影处提前失败。对应生产调用路径是 `instantiate_program → discover_round → project_body_scoped_with_caps → InferSession`。`representation-observations.txt` SHA256 `a15a5eda96b101aa1bdb63681fecd73e057aa9aa961612aba42dc3b5f0709207`。同组两种记录转为受检枚举载荷再由 enum 方法分派的可行性探针（不同实现分别返回 5 和 35，加 2）真实 AV 得 42。它只是表示验证，三份原 spec 调用仍待接线和验收。

需要在现有 scope 内修订实例需求收集、替换值域与跨模块实例 owner 的设计，对原三份动态 spec、转交泛型 enum、`to_array/filter` 保留原始义务与独立 oracle。旧 VM `load_vm_test_cache` 仍读原文档，T-05 须处理读者单源。引用路径、案号、原检查、后端阶段条件均保留。没有验收项或任务标记为已完成，`IF-AV-PARITY-GENERIC` 尚未发布，独立复审与合入均待完整实现后执行。

- `stage: work | plan_id: PLAN-1088 | plan_revision: r3 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-1088/auto-lang | branch: plan-1088-dev | base_commit: a3655eb2ff2638cdbecc002b7bfd74122d6ab43a | code_commit: fd8aa825a7e010cdabb7118586ed01455f646048 | dependency_revisions: auto-down@45b642d | task_ids: T-01..T-05 partial, T-06..T-08 pending | evidence: §9 本检查点与组合探针 | affected: AC-01..05、AC-FC | next_responsible_role: new`。仅修订实施设计，既定目标、验收、后端和仓库边界不变。
- `stage: goal | plan_id: PLAN-1088 | plan_revision: r3 | round: 1 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: fd8aa825a7e010cdabb7118586ed01455f646048 | next: new`。

- 2026-11-20 `stage: new | plan_id: PLAN-1088 | plan_revision: r4 | outcome: pass | source_commit: fd8aa825a7e010cdabb7118586ed01455f646048 | tasks: T-01..T-06 | affected_acceptance: AC-01..06, AC-AV-ONLY, AC-FC | next: work`。本轮仅修订执行合同。`INSTANCE-COMBO`、`TYPE-DIAG`、跨包身份和旧义务行的实现及判决未由本次文档预检代替。
- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 2 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: fd8aa825a7e010cdabb7118586ed01455f646048 | next: work`。

### r4 普通绑定器实例化检查点

提交 `755d900bac10b9875c87c1f08ed7f51a57853818`，实施树 clean，`auto-down@45b642d` 未变。普通绑定器在真实实参解析后收集需求，保 `Str` 与借用 `str`、列表元素、名义枚举与函数签名。从一个 body 取得待回填的调用后，先按实际替换重绑并重放该 body，再采集依赖该结果的需求。发现阶段暂未完成的调用只停在诊断和工作列表中；最终从具体 AST 重走原 resolve/typecheck/HIR、MIR/ABC 和 AV。单文件与包使用同一实例生产者；包键含锁版本、包名、模块和声明，具体 def 保原声明来源。窄域旧 963 实例页仍由原路径处理。

| 实际范围 | 结果 | 准备 | 运行 | 总墙钟 |
|---|---|---:|---:|---:|
| `cargo t case-av-language-parity --run-prepared` | 266 AV PASS、110 SKIP、0 FAIL，父子退出 0 | 14.168s | 20.154s | 23.512s（执行命令） |
| `ace-av` 库的 `modules::packages::tests` | 4 PASS、0 FAIL、15 本次过滤 | 12.374s | 0.478s | 0.478s（执行进程） |
| `ace-source-query` 三个已准备目标 `instances_value_domain_963`、`instances_invalidation_963`、`instances_const_963` | 12、5、12 PASS，失败/忽略/过滤均 0 | 14.811s | 0.071s、0.048s、0.103s | 各自同左 |

本轮未单独计时发现、清理与等待资源。新包身份测试最初误用了保留词 `dep` 作包名，实际入口拒绝 `text/source.unsupported`。修正合法测试输入 `support` 后复绿。没有非 AV 执行。独立复审及 T-06 全套冻结证据尚待后续完成。

| 新检查 | 行为与固定 oracle | debug 毫秒 |
|---|---|---:|
| `av.parity.1088.owned-text-instance/av` | 移动 owned `Str` 穿过模板参数和返回，原内容长度为 4 | 558 |
| `av.parity.1088.typed-list-instance/av` | 具体 `List<int>` 跨实例传递，元素 19+23 得 42 | 500 |
| `av.parity.1088.function-value-instance/av` | `twice` 经实例返回后间接调用得 42 | 566 |
| `av.parity.1088.variant-instance/av` | 名义枚举载荷经实例和模式取出 42 | 571 |
| `av.parity.1088.nested-instance-demand/av` | 多个调用及同一表达式的嵌套需求得 42 | 519 |
| `av.parity.1088.conflicting-inferred-arguments/av` | 同一 T 接收 int 和 bool，`resolve/type-mismatch` | 548 |
| `av.parity.1088.cross-module-instance/av` | 包内两模块共用模板实例，19+23 得 42 | 639 |
| `modules::packages::tests::ordinary_packages_share_instances_and_bind_restore_to_version` | 跨包两调用只生成一 concrete def；改 body 值 42→44 且符号保持；换锁改变符号；旧/新恢复与各自冷编一致；跨版本收据拼接被拒 | 188（独立进程含准备已存在的运行场地与清理） |

材料作用域 `D:/autostack/plan-artifacts/plan-1088/`。`r4-prepared/prepared.json` SHA256 `4563adff51935bc49b0784ae766bfdeb4085adb44a92779765dc595b17b48511`；`r4-full-av/results.json` SHA256 `9367fdb36091a8abced1ed5b7eee67236ea5937ffa922f0fef4f1eb2bd699296`；`r4-package-neighbor.log` SHA256 `9112cfbcb1a72970df6b9a76f56edb23f2e6b217215fca4874866c31e9b9d99e`。七个新 AV 检查以及包恢复测试均小于 10s。

仍须落实显式类型实参、泛型方法和旧泛型 enum，补 spec 值分派及 `to_array/filter` 的原样检查。签名变化、所有权与取消的实例级组合证据继续归 T-04。T-05 的 VM 读者与所有旧义务映射尚未验收。保持 T-01..08 标记，保原验收门。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 3 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 755d900bac10b9875c87c1f08ed7f51a57853818 | next: work`。接着处理显式类型实参与模板方法接面，以及旧 spec 和枚举原用例。

### 2026-11-20 实施检查点 4：普通 AV 显式实参和包签名失效

- 工作树提交 `61385cfcaceb796ac60bcb4d9bcf69bec5740169`。普通入口以紧接函数名的 `<` 启动显式类型/常量实参，常量和类型按声明位置绑定。保留原 `twice <2>(21)` 常量调用兼容和带空格的比较解析。模板实例的类型从正常注解解析得到，具体调用复用普通 resolve/typecheck。旧常量调用结构键无字节变化，语法程序保留旧 atom tag，新增 tag 4 仅在启用相邻 token 的 profile 内生成。
- 新增 10 个普通 AV 检查（732 / 756 / 758 / 821 / 784 / 765 / 642 / 682 / 704 / 804ms，依次为仅返回类型、类型与常量混排、类型参数转发、List、Box、顺序错、元数错、实际类型不符、比较诊断、带空格常量）。正例固定值 42。直接 worker 复核顺序错定位到实参 `bool`（111..115），元数错到调用（62..82），实际类型错经 verify 到实参 `42`（74..76），`a < b > (c)` 的比较诊断仍为 verify/source.type-mismatch（52..57）。
- `cargo t case-av-language-parity ace-syntax --prepare-only` 29.324s / exit 0，prepared 首跑 37.267s / exit 1。语法 157 PASS、3 SKIP，nextest 子命令 6.245s、实际测试 2.707s；AV 275 PASS、1 FAIL、110 SKIP，FAIL 为通过值参数直接返回名义记录时的 `mir.return-type`。等价非泛型签名复现该诊断，显式 move 对照在泛型与非泛型均为 `mir.move-type`。同一显式 `Box<int>` 实参读字段可返回 42，现以该源码验收具体记录实参。记录返回组合仍待核，保留首红。
- `cargo t case-av-language-parity --prepare-only` 27.808s / exit 0；prepared 重跑 35.250s / exit 0，AV 子命令 28.645s / exit 0，386 项中实际执行 276 项全部 PASS，110 项 SKIP。从本计划材料目录启动父侧取样，报告 SHA-256 `66b19729c1c4cbafd5654aa943e4a3250ffaee411cb445e898b062993efe07c8`，worker `b507a2124ad6fcc6765a1444f1ae89518451115d960e4de7c1ad499902e8bc1b`。没有运行非 AV 目标或 Linux/WSL 用例。
- 两段独立构建准备 `cargo test --locked --offline -j 3 -p ace-av --lib --no-run` 21.792s、`-p ace-source-query --test instances_const_963 --test instances_value_domain_963 --test instances_invalidation_963 --no-run` 8.999s，均 exit 0。直接执行已编译二进制，AV 包模块 4 PASS / 1.342s（其余 15 filtered）；三个 963 冻结目标分别 12 / 12 / 5 PASS（0.071 / 0.086 / 0.109s），退出均为 0。包试验中显式和推断的跨模块调用共享一个实例。增加参数后的旧调用报 `source.argument`，修正两个调用后冷编和恢复均得 49。此前 body、版本和错误 receipt 控制同次复验。
- 探针已命名三处待实现组合。只有预期返回类型时 `answer()` 仍为 `unsolved-variable`；显式别名 `answer<Number>()` 为 `source.undefined`；模板内 `Box<U>` 在语法期展开过早，被拒为 `source.unsupported`。以上与泛型方法、旧泛型枚举、spec 值分派及原 `to_array/filter` 一并保留在当前验收。T-04 实例资源/取消组合和 T-05 原 VM 读者迁移仍待完成，正式复审与冻结合入门未触发。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 4 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 61385cfcaceb796ac60bcb4d9bcf69bec5740169 | next: work`。推进泛型方法、旧泛型 enum 和原 spec/List 用例，随后逐项补齐组合与迁移门。

### 2026-11-20 实施检查点 5：普通 AV 泛型方法

- 提交 `52ff4da5ae8934edbeefa7a38ffc0dfcec238c59`，实施树 clean。按实际接收者的记录或枚举身份挑选方法模板，实参与自由函数复用 binder 需求发现和 AST 替换。方法实例仍安装在原 owner 内，通过原静态/视图/可变方法的签名注册、resolve、verify 与 AV 执行。调用点重绑按 owner 和 source span，支持同名方法及递归重放。普通 `.method<T,2>(...)` 用与自由函数一致的紧接 `<`，旧冻结面仍拒模板方法。包编译保声明和具体成员来源，显式及推断的同一个替换组合共用一个实例。
- 新增 13 个 AV 检查。可变接收者、递归、两个同名 owner、常规 enum 泛型方法、泛型记录的方法、静态类型/常量混排、扩展转发、未使用模板和跨模块调用均为固定值 42；可变右值、静态方法接收值、显式调用非模板以及类型/值次序错分别命中 `source.mut-receiver-place`、`source.not-callable`、`source.instances-unsupported`、`source.instances-argument-kind`。首轮定向 43 PASS/1 FAIL/2 SKIP，失败项跨模块新见证用了包入口不支持的导入记录位置构造，报告 `source.module-symbol`；改新见证为该入口已支持的具名构造后定向 44 PASS/2 SKIP（3.559s，exit 0）。旧语料源码和 oracle 不变。
- `cargo t case-av-language-parity ace-syntax --prepare-only` 首次 7.917s/exit 2，格式整理移除了 `SourceProfile` 导入，修复后 43.607s/exit 0。相同固定选择 prepared 执行 72.245s/exit 0。公共 AV 399 项（289 PASS、110 SKIP），父侧 55.421s；语法固定组 157 PASS、3 SKIP，nextest 命令 4.307s，实际测试 2.522s。13 个新增用例单项 0.961–2.220s。报告 SHA-256 `1dd6485b9306c53217fcba0fef4ddf9be29722ca92d591015be96e6482406597`，worker SHA-256 `069e169dd774ff9e03a04d1a3b36dd6e2c49205dee4d0b20dd2bf60e0e5246c2`。本轮未运行非 AV、Linux 或 WSL 用例。
- `cargo test --locked --offline -j 3 -p ace-av --lib --no-run` 52.858s / exit 0。直接执行已准备的 `ace_av` 二进制，包模块 5 PASS、15 filtered，0.863s / exit 0。新增包验证在产物的 `Counter` 声明内实际找到唯一具体方法；绑定来源为 `support.math.Counter.add`，显式加推断调用执行得 42。日志 SHA-256 `036ac33a22d9c75d7e16710b55456dd8e9885847d35a5bd3cd914801b04674db`。
- 本轮以未改的六份原文再探测遗留差异。28.005 在泛型 enum 声明拒绝，仍须保 `Bag.Hit`、`Bag.Miss` 及两个 bool 输出原 oracle。24.002 无 `to_array`；24.003 在旧类型标注 lambda 拒绝，且目前非 int 元素的 HOF 仍走独立有限域诊断。99.000 在 spec 值注解 `Tool` 拒绝，99.020 在空记录拒绝，99.031 在 FStr 插值内带引号处拒绝。实际响应位于 `r5-legacy-source-observations.json`，SHA-256 `b9faedd11e70b6fcd93bf97ea67ea1369010b98916eddec3a780d15b5d7f5e98`。上一检查点的组合和资源控制、这些原文的验收、原 VM 读者切换及独立复审均保留。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 5 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 52ff4da5ae8934edbeefa7a38ffc0dfcec238c59 | next: work`。下一轮以 28.005 原源码推动泛型 enum 及其方法覆蓋。

### 2026-11-20 实施检查点 6：原泛型枚举源码和返回上下文

- 提交 `fc91134925290c9c56a7b53287ba591d90704684`，实施树 clean。普通端语法接入 `pub enum Bag<T,E>`、相邻的旧载荷写法和类型化构造。真实 binder 用选中臂的类型约束、显式实参或已知上下文发出实例需求。实例键按声明序记录具体类型和缺席位，原判别序保持。未定载荷的臂不能构造，需要未定类型的方法按原调用定位拒绝。具体实例沿现有名义枚举、模式、方法与 AV 执行路径；泛型名称仍可作为合规限定匹配，同形异名仍拒。枚举类型注解消费同一实例表，包含局部、参数、返回及可求解的模板形状。普通自由函数和方法也可从已知结果类型补全未由实际参数确定的替换。
- 28.005 原源码与原 VM stdout `1\n0` 逐字保留。已归档 PLAN-474 §四的第 3 条以及现有 `shim_print_unified` 将 bool 显示从数字改为 `true/false`，本例 AV 严格判断 `true\nfalse\n`。专项反控仅把 AV stdout 换回 `1\n0\n` 时实际 exit 1，精确定位 stdout byte 0，同时 VM check 仍 SKIP。正确 AV oracle exit 0。日志 SHA-256 `7ffd45c1e73c059ac9b290ad7413a5a79686222eadb9f55156b417c85b258f16`。
- 三份已承接原文（24.001、28.005、m07）通过既有 `CanonicalVmCases` 数据路径切到同一个 canonical 作者文件。原路径是引用，源代码、虚拟源路径、VM ID、检查和原 oracle 保留，VM 读取器原条件不变。静态对账取上一提交的旧源码和 VM oracle，三组 SHA256 前缀 `6adacccc13da0620` / `39161e08f33a1c8d` / `5398477de3dceb34`。历史 `.at` 和 expected sidecar 保留。未运行非 AV 后端。
- 定向首次 55 PASS / 1 FAIL / 3 SKIP，失败为显式 `str` 进入枚举元组载荷时未沿拥有型位置归一，MIR 报 `mir.field-borrow`。让容器载荷中的元组元素继承存储位置后，定向重跑 59 PASS / 3 SKIP（6.292s，exit 0）；检查显式 tuple 文本真实入值和读回。
- `cargo t case-av-language-parity ace-syntax --prepare-only` 39.965s/exit 0，prepared 执行 51.604s/exit 0。公共 AV 417 项（306 PASS、111 SKIP），父侧 41.557s/exit 0；语法 157 PASS、3 SKIP，命令 4.759s，实际测试 3.467s。新增 17 个 AV 检查（16 条新用例及 28.005 检查），单项 0.746–1.773s。报告 SHA-256 `3e66e2f9e88582d8416c91a7f9792fe1944a76da3f861f89872612d683e90d25`，worker `8fe397da56e316451c78acd7d31f62709f9b6eac5cc0d70747a31fb424acae05`。无 Linux/WSL 或非 AV 目标运行。
- 后续仍需执行原 `to_array/filter` 与三份 spec 分派、显式别名及模板内参数化记录组合、实例资源/取消/恢复及 `#[with]` 组合、剩余原义务迁移与控制、冻结验证和独立复审。本轮泛型枚举的证据是单模块普通入口，包入口目前仍保有 enum 的具名范围限制。计划验收标记维持。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 6 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: fc91134925290c9c56a7b53287ba591d90704684 | next: work`。下一轮推进 24.002/003 原源码及复制独立性和过滤结果顺序。

### 2026-11-20 实施检查点 7：泛型列表复制与过滤

- 提交 `2c22fb53cc97a3a2d864b5f11c6c0f6674402ce2`，实施树 clean。普通 AV 接通显式返回类型的块闭包、`List<T>.to_array()` 的深复制，以及非 int 列表过滤的类型检查和迭代。过滤对调用时的元素顺序快照执行，谓词逐元素调用一次，结果独立拥有保留的元素。借用型谓词保参数借用，拥有型谓词消费独立副本。受检 HIR 复用已有 MIR Clone、闭包调用和循环指令。int 过滤复用原标准库路径。
- 原 24.002、24.003 普通 AV 均输出 `ok\n`。原 Auto、虚拟源路径、VM 检查 ID 和原 stdout 逐字对账，通过既有迁移表切到 `language/generics/list-methods.case.ad`。旧 VM 检查留在共同作者并加 `#ignore`。源码 SHA256 前缀分别为 `e93454327dee43f7`、`ae058008d5937f5d`，加上上轮三份共核到五份。审计日志 `r7-source-audit.log` SHA256 `c9dee68171aa07476690047de3a30579a83a697b699ad560e006f1915dfd451a`。列表用例独立成文件以满足作者文件 64 用例的既有上限。
- 增加 11 个 AV 检查，覆盖记录及内嵌列表的复制独立性、过滤保留顺序与谓词调用次数、bool、借用与拥有文本、int、空结果、嵌套记录，以及错误谓词参数和错误闭包声明。首次运行原过滤见证因间接调用的记录借用签名报 `mir.closure.type`，修正签名和 AV 参数所有权来源后通过。定向父侧最终 74 PASS、5 SKIP，7.744s / exit 0。定向报告 SHA256 `f95aa2426a39b60d3980c75360683b51271e2ce6bc5deac493364c1f3a1d20f3`。
- `cargo t case-av-language-parity ace-syntax ace-source-syntax-927 --prepare-only` 准备成功，同一 receipt 的三个内部执行均 exit 0。公共 AV 432 项（319 PASS、113 SKIP），24.291s，新 13 个检查单项 0.508–0.688s。语法 157 PASS、3 SKIP，命令 3.350s；语法差分 13 PASS，833 项未选，命令 2.803s。准备 receipt `r7-list-prepared-2/prepared.json` SHA256 `e836d056171643f88ca30ecd3b3200bb43adc26aba8a0dc2e35def049a79162d`。结果 `r7-list-fixed/results.json` SHA256 `e782c8d70ec66ba1eaacfe9e58a070a1744dd1693f131e22b5ca6ee9c1654d9c`，AV 父报告 SHA256 `cf6810822cfae3b1ad821a67572f400fd80c6d8ee4d9b678e433b384ec137dd9`。另原生选择 AV/MIR 深复制、释放、失效闭包与错误记录类型 6 PASS，64 filtered，0.087s / exit 0，日志 SHA256 `7fe469ed4259343e9198f25238e6df6d09d864b9109680b2495e01b1755ebdab`。本轮无其他目标、Linux 或 WSL 执行。
- 待实施三份 spec 分派原文及继承、关联类型、扩展可见性组合，显式别名及模板内参数化记录，实例身份与资源/取消/恢复和 `#[with]`，以及剩余迁移控制、冻结验证和独立复审。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 7 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 2c22fb53cc97a3a2d864b5f11c6c0f6674402ce2 | next: work`。下一轮接入遵从见证和 spec 参数分派。

### 2026-11-20 实施检查点 8：spec 分派及三份原源码

- 提交 `be3a467869c332474d4a996edcde77b10b2c820a`，实施树 clean。普通单模块将已核验的具体遵从记录及其 super-spec 闭包装配成名义 variant 和常规方法，参数、返回和局部注解用现有受检构造，分派在真实 AV 执行。两个实现和继承默认体均得 42。无 claim 的同形记录在原 `Other(2)`（302..310）处给 `verify/source.type-mismatch`。额外覆盖同一资源记录重复传参、同一 spec 参数多次方法调用、扩展方法覆盖默认体、继承的关联类型绑定和默认体在具体记录上的消费。动态关联类型及跨模块 spec 值仍有边界。
- 原 99.000、99.020、99.031 的源码未改，AV stdout 与各自 VM 原 oracle 全等；所需普通面同时接通隐式记录字段、命名位置构造、空记录、`assert`、插值内引号和拥有文本的 spec 返回。Unicode `to_upper` 沿带 `av.text.case/1` 的受检 ABC 执行。独立 oracle 逐字验证 `Straße café ωﬃ` 切片转为 `STRASSE CAFÉ ΩFFI`，并核输入仍可读。动态 spec 返回保有序内容，借用文本实现的输出在分派边界具体化为拥有值；现存 1086 的借用正负控继续通过。
- 三份 99 文档以稳定 ID/check 迁到 `language/generics/spec-dispatch.case.ad`，旧 VM 小节为 `#ignore`，旧路径经同一迁移读者引用；原文和 oracle SHA256 前缀分别 `33f65b770780ae1a`、`75f52931259acfc9`、`43f5e78ecda06433`。累计 8 条原义务实查，审计 `r8-source-audit.log` SHA256 `c5ca7b549775f4a26a526b8e05ea168acd8ae398550c6ddeed5d27ecd6be7e9f`；只运行 AV。
- `cargo t case-av-language-parity ace-syntax ace-source-syntax-927 --prepare-only` 成功，准备 SHA256 `40e0b4b6e540151fc13248d9d6ff8d38bd167ffbde1f70b61c76bb0c1e372b97`。固定执行子项均 exit 0。公共 AV 452 项（336 PASS、116 SKIP），27.248s；新 17 个检查 0.520–0.623s。语法 157 PASS、3 SKIP，命令 3.286s；语法差分 13 PASS、833 未选，2.630s。父侧 AV 报告 SHA256 `a6a008b6995e72aa3b80388868569778248f726e6ac36567f6126746cd399d2e`，`r8-spec-fixed/results.json` SHA256 `0d1b63673f967b9552bb1ec5cccb37e868d8c8d479da8357d25e139e7e4a589c`。
- `cargo test --locked --offline -j 3 -p ace-av --test conformance --no-run` 39.13s / exit 0。直接选择本轮两项、原拥有型专项及指令契约，10 PASS、1356 未选，0.300s / exit 0，日志 SHA256 `ca4b7f2306395f8268eb81f82441508571b716fc7c73f65e565bf312c3c662c9`。逐 ID 核对分配/释放与 live/frame=0，具名测到删 uppercase capability 的 MIR 拒绝、低燃料时已有泛型拥有者释放且重试正确、列表过滤最终分配点故障时此前所有分配释放且重试返回 42。无 Linux/WSL 或其他目标运行。
- `r9-precheck.json` SHA256 `32e7c81c416e7431277ed637fd2eea6e799700b9b5282863a9c80f1c8e4e263d`。实测 `Box<T>` 模板形状在旧投影处被具名拒，显式别名在实参绑定处未替换。m08/m09 原 `.at` AV stdout 已实际通过，仍需对照文档、原义务与 canonical 位置。属性源 `#[with]` 在 `#` 处未准入，PLAN-1092 §5/任务 T-02 持有该源码语法；本计划需继续核共享见证和具体消费边界。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 8 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: be3a467869c332474d4a996edcde77b10b2c820a | next: work`。下一轮推进参数化记录、别名与剩余旧义务及迁移控制。

### 2026-11-20 实施检查点 9：依赖类型、别名与共享语料反控

- 提交 `1be4163dd769a43f78c994441a5593049d8943fd`，实施树 clean。原投影在 `Box<T>` 的参数、返回和局部注解均报 `source.unsupported`。普通源现在在签名或模板体中保留依赖类型，实际绑定从名义记录实例恢复有序类型形状，替换后复用原记录单态化，并回到真实类型与所有权验证。显式别名在现有类型表达式遍历中展开。标量记录的只读参数以现有 RecordClone 在值返回时复制。Spec 约束复用已验证的直接和继承成员身份；未声明遵从的同形记录仍拒。
- 新增 14 条 AV 检查涵盖隐式/显式 `Box<T>`，实例体构造、返回、嵌套、owned Str、显式标量及记录别名、两种跨模块构造与方法组合、继承与嵌套记录 claim，以及错记录实参和无 claim 反控。直接探针证明早先的四条依赖类型及两条别名红点转绿。定向父侧 104 PASS、8 SKIP，8.117s / exit 0，包含其中 13 条；第 14 条嵌套记录见证在固定组通过。定向报告 SHA256 `51fb0f965ec2dcc0a3d5991c64dc058238ba9fad5c3a8d8242d3e6573221d572`。
- m08/m09 原文、原 VM stdout 与 PLAN-1086 已存在的 canonical 条目逐字相同，保持该 owner，公共 AV 固定组两条实际 PASS，旧 VM 两条 SKIP。审计日志 `r9-receiver-owner.log` SHA256 `bab11af706af9c99e2742fe0527a5f8c13bc54350ee2b5b75348bd69e67b2805`。FC 反控使用真实 Python 固定选择逻辑与受检父入口：变更源重跑，source hash 变化且 oracle hash 保持；篡改 AV oracle 使 exit 1 且精确 mismatch；无 AV 与 AV 全 ignore 在 inventory 拒绝；重复 Case ID exit 2；错误 native oracle 与不存在的 worker 在 AV 视图中分别记录 SKIP 或 NOT_RUN，真实执行集合只有 AV。摘要 `r9-fc-controls-5/summary.json` SHA256 `ae2b563f0ee2586b1861cea7115cf3a654f4ee0bb1b2202a60468328221b088a`。
- 固定组 `case-av-language-parity ace-syntax ace-source-syntax-927` 准备成功，prepared 三条子命令 exit 0。公共 AV 466 项（350 PASS、116 SKIP），33.362s；新 14 个检查 0.623–0.728s。语法 157 PASS、3 SKIP，3.331s；语法差分 13 PASS、833 未选，3.098s。准备 SHA256 `7278e494cf3bfe739d3a829ab7a89a26933bdd555383593c0069ab3f67e8ba52`，AV 父报告 `c8676f4f27d4d2cbf9b4ca4045100e3266be93a510ca9d0527851090e9c7e847`，固定结果 `ccc60fa8528044ba64571d6fac6808e9a74bbe90b455a929ebc04b2e3829351e`。本轮无 Linux/WSL、旧 VM 或其他目标运行。
- 主分支已接收 PLAN-1087。纯合并模拟发现 6 个相交文件需在专属实施树内处理，含公共 suite/map、AV worker、指令契约、binder 和语法。本计划的 `STR_UPPER` 指令标记也须避开已用的 `TERM_INVOKE` 标记。下一轮把所需逻辑合到主分支新基面，验证活跃泛型拥有者的取消/重放与最终冻结候选，并完成独立复审和落主归档门。

- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 9 | from: work | to: work | status: executing | outcome: progress | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 1be4163dd769a43f78c994441a5593049d8943fd | next: work`。下一轮同步主分支、补取消控制并验证冻结候选。

### 2026-11-20 实施检查点 10：集成、生命周期与冻结交接

- 在 `a503d7ab23208a0496c35637303dd8cc8c4e4d51` 上冻结并实测 `27f45d4e3f1d69aa6e726c5eb45333f589443443`。先后同步 1087 与 1089，合并 suite/map，保 AAC value-only、条件列表与错误来源。新 `STR_UPPER` 标记移至 `0x53`，与 1087 终结指令分开。两条类型化条件列表控制分别返回 42 和拒绝错元素；导出记录扩展返回 42，私有导入给 `source.module-private`。嵌套记录从只读参数构造的首轮 `mir.aggregate-field-type` 经普通复制型参数在拥有值边界复用 RecordClone 修复，原输入得 42。确认资源仍执行原所有权检查。
- 主分支随后只新增 1089 归档收尾文档 `3b758c3e997de2772d87a95d94d48309f0398804`。最终候选 `50fa8e0180457068c798c247d015afb89a6c91e6`。十个补丁的 `git range-diff` 均 `=`；两冻点对 `crates experimental scripts .config .cargo Cargo.toml Cargo.lock` 的 diff 完全为空，唯一区别为已归档 1089 文档。range-diff SHA256 `7b977aed0aaaca758dcb0c39066e8c34436495ae8cfaac28945f0084ae908148`。共用原真实验证，实施树 clean，依赖 `auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。
- `cargo t case-av-language-parity ace-syntax ace-source-syntax-927 ace-semantics --prepare-only` 44.786s / exit 0，准备 `r10-delivery-prepared/prepared.json` SHA256 `4dcf940bcd0d160f3db644cf802b6cde1abb7ab8b634f6a679902808117d4eda`。同 receipt 执行合计 64.716s / exit 1。公共 AV 573 项，425 PASS、148 SKIP（exit 0，37.219s），其中新 102 项 0.492–0.941s。旧八项及 m08/m09 AV 全 PASS，旧 VM 各 SKIP。语法 159 PASS、3 SKIP（exit 0，5.898s），冻面差分 13 PASS、833 未选（exit 0，2.898s）。语义 294 PASS、1 FAIL、10 SKIP（exit 100，6.104s）。唯一 FAIL 为已知 D-1037-01 `sema_boundary_case_files_executed_status_honest`，报 `cache-roundtrip.document.case.ad` 缺 `status: executed`。master 与候选的断言 blob `0dbe00f1c81b30153fd06a869d262ad1efa1d8a8`、输入 blob `c4874997b7f528c6e6a1faf3efefb3d463a934b8` 各自相同。父报告 SHA256 `4c2eca6dcaefe13517608860b03047ee3fda62b359e4b47c3cb12d43aa7ea3d7`，固定结果 `1a932c354c86d86252e1b6e436edf15d98c668e5d38c08399956db5d91badd5a`。发现与执行日志独立，清理未单独计时。其他后端及 Linux/WSL 未执行。
- `cargo test --locked --offline -j 3 -p ace-av --test conformance --lib --no-run` 26.740s / exit 0。定向普通 AV 的本包三项、拥有型/异常回归和指令契约 20 PASS / 0.163s；包、版本与恢复 5 PASS / 0.577s。日志 SHA256 `fd5998c36c418539fea25b3cdbbfcba98f6215a7231ab71eac9d46c3d6b5b912`、`6b8b04787f4e91f8a79a6ce4ba770d2d98385876b0aba70d934790a63c09e276`。真实 host 对存活泛型拥有者取消，回调一次，frame/live 均零，分配/释放 ID 全等，重放返回 42；另核分配故障和燃料用尽后释放与恢复。首轮取消见证误用保留字 `hold`，改名后到达实际 host。包见证实际断言同替换只有一个实例、体变更 42→44、旧签名拒绝及新签名恢复 49、版本区分和错误根回执拒绝。
- 工作接口 `IF-AV-PARITY-GENERIC` 使用普通 `ac-av/language-source-run-v1` / `d_main`、受检 HIR/MIR/ABC 与 Win64 AV。四个 canonical 作者已加入公共 suite。规范提案及源身份见 `r11-spec-proposal/identities.json`。复审、落主、归档与清理待交接。

- `stage: work | plan_id: PLAN-1088 | plan_revision: r4 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1088/auto-lang | branch: plan-1088-dev | base_commit: 3b758c3e997de2772d87a95d94d48309f0398804 | code_commit: 50fa8e0180457068c798c247d015afb89a6c91e6 | dependency_revisions: auto-down@45b642d | task_ids: T-01..T-06 | evidence: §9 检查点 10 | next_responsible_role: review`。
- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 50fa8e0180457068c798c247d015afb89a6c91e6 | next: review`。

### 最终复审（第 11 轮）

同一会话在冻结提交后另行检查实际入口、差异、源身份并重放区分性反例。复审未修改实施代码，绑定 `50fa8e0180457068c798c247d015afb89a6c91e6`、基面 `3b758c3e997de2772d87a95d94d48309f0398804`、`auto-down@45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。生产命令父侧 SHA256 `56e9cf9257599d8634cb8ded7e5c1ea6b66bfd992c8e6b7d4b7f545386800b9f`，AV worker SHA256 `10eb8c7ba921da499b46107b21bbcd5776926db44fb8f597e2e0246e41052e4d`。

| 验收 | 复核与实际结果 |
|---|---|
| AC-01 / AC-02 | 顺着普通源码、binder、具体 AST、受检 HIR/MIR/ABC 和 Win64 AV 核对。实际 worker 重放嵌套泛型记录经别名、继承关联类型加扩展、引号和花括号插值、拥有文本过滤、只读记录转 spec、过滤谓词失败保原数据，分别得到 42 或原双流 oracle。同形未 claim 的 `Impostor(42)` 在 byte 223..235 给 `verify/source.type-mismatch`。移动型实参隐式装入拥有字段在 85..105 给 `mir.aggregate-field-type`。8 项重放记录 SHA256 `bfcafb3c33c747ec1f2ed610a8ed97208298a528aeb0bc9444e7892fa4ee4c58`。 |
| AC-03 / AC-04 | 复核 §6 需求、包与宿主断言。实例复用、签名失效、版本与重放身份由包测试实测，取消/分配故障与回收核实际 ID。八条原义务映射到唯一作者，三份 99 和 24 原 stdout 相同，28.005 保 VM `1/0` 与 AV `true/false` 的既定表现。原 m07 和 PLAN-1086 已迁 m08/m09 保逐字 oracle。动态 spec 的单模块完整具体接口边界及 PLAN-1092 的 `#[with]` 元数据责任已明确。 |
| AC-05 | 普通 binder 采集实际需求并回到相同检查路径。分派只消费已声明遵从，使用经检查的名义枚举与普通方法。`STR_UPPER` 标记 0x53 由 `av.text.case/1` 托管，查既有 1087 AAC 指令无冲突。旧 VM 读者沿 `legacy_doc_for_trans.rs` 的 canonical 引用恢复原 ID 与 check。 |
| AC-06 | 复核真实准备和子命令退出、425 AV PASS、148 SKIP、102 新 AV 项，最慢 0.941s。AV 原生 20 与包 5 均通过。963 def / spec / 窄箭头闭包独立重放 31 PASS、815 未选、0.18s，SHA256 `a90ae35c3cd2d0ff129f088e372d14e90494b714187e9fca3318f38cdfbc2152`。语义唯一红 D-1037-01 有原始断言/fixture blob 同一性。差异和目标格式检查干净，检查未见新增未处理警告和调试输出。 |
| AC-FC / AC-AV-ONLY | 两版源码改变了 source hash 且复用同一 oracle，两次通过。错 AV oracle 使 exit 1，缺 AV 或全 ignore 拒绝过门，重复 ID exit 2。故意错非 AV oracle、缺非 AV worker 时仅 AV 有实际执行，其他项 SKIP 或 NOT_RUN。真实父侧控制 SHA256 `a44ade03b16e44a833bb5abadab62d67c7255122522465011ac94fb0db733057`。Linux/WSL 未执行。 |
| AC-07 | 核对差异、实际消费者与约束。扫未批准延期、遗漏和兼容捷径后未发现本包需修复项。D-1037-01 已在债务表，按相同基线保留。三份规范仅增经验证的公共知识，源码差异和正式发布身份在合入时核。 |

- `stage: review | plan_id: PLAN-1088 | plan_revision: r4 | outcome: pass | reviewed_commit: 50fa8e0180457068c798c247d015afb89a6c91e6 | base_commit: 3b758c3e997de2772d87a95d94d48309f0398804 | dependency_revisions: auto-down@45b642d | task_ids: T-07 | evidence: §9 最终复审 | next: merge`。
- `stage: goal | plan_id: PLAN-1088 | plan_revision: r4 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1088/auto-lang | commit: 50fa8e0180457068c798c247d015afb89a6c91e6 | next: merge`。

### 落主、规范与归档（第 12 轮）

- `master` 从 `3b758c3e997de2772d87a95d94d48309f0398804` 快进到本轮复审的 `50fa8e0180457068c798c247d015afb89a6c91e6`。使用 §9 复审的父端与 worker 从主检出的公共 suite 实跑 `auto-case run --suite <master>/crates/auto-lang/test/cases/language.suite.ad --out <scope>/r12-main-av --target av`，exit 0，38.701s。结果 573 条，425 PASS、148 SKIP，其他为零，本包新增 102 条全部通过。报告 SHA256 `3557803496bf639060531247a7c3dd6ea17a6c6e9b727af69b3f0c13c59f4021`。汇总的零失败字段省略，按 573 条明细及父端退出码核验。
- 交付接口 `IF-AV-PARITY-GENERIC`。规范增量提交 `36cd653454093acac2f40ee7ca4db946856ca271`，三个提交 blob 的 SHA256 与已审提案逐字一致。

| 正式来源 | SHA256 |
|---|---|
| [AV 语言齐平](../../specs/auto-av/language-parity.md) | `03e6bc46c353c947f57e72c921d1c41023ac9fff42da148860fcc0a66f5172c9` |
| [ACE 泛型](../../specs/ace-migration/m2-generics.md) | `a0fb6ecbe0b4527cdf5369a1fbb18795e92206cae096914724ac1c95a3b25f3f` |
| [按特性组织的共享语料](../../specs/auto-test-case/feature-organized-corpus.md) | `3ead9c45be3be74e81c1ab19598fe156339f915f1ff4577fd2efff70d1f64d11` |

- [公开 suite](../../../crates/auto-lang/test/cases/language.suite.ad) 引用四个 [泛型 canonical 作者](../../../crates/auto-lang/test/cases/language/generics)。[旧义务映射](../../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv) 保留原路径、check、预期与阶段责任。路线和旧清单导航指向本归档。
- 输出对账：独立探针与日志、冻结测试、子进程临时目录及本次落主报告均在 `D:/autostack/plan-artifacts/plan-1088`，预清点 14726 文件、607311484 字节。开发和准备的 target 位于 `D:/autostack/.wt/lang-1088/auto-lang/{target,experimental/ace/target}`，连检出共 56235 文件、19226744693 字节；同组仅另有 detached 的 `auto-down` 检出，1318 文件、27021855 字节。逐轮命令与已登记路径对账未发现外置 target、安装/消费根或其他计划输出路径。清点未发现 reparse point 或该组活动构建/测试进程。
- `stage: merge | plan_id: PLAN-1088 | plan_revision: r4 | checkpoint: landed, specs_verified, archive pending commit | delivery_commit: 50fa8e0180457068c798c247d015afb89a6c91e6 | spec_commit: 36cd653454093acac2f40ee7ca4db946856ca271 | archive: docs/plans/archive/1088-av-parity-generics-specs-and-methods.md | cleanup: pending`。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
