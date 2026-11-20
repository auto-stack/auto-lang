# AV 与 AutoVM 语言齐平：差距设计、AutoDown 验证与执行路线

2026-11-19，规划r3。用户决定先推进AV语言路线；依据当前工作区源码/Specs/原AutoDown作者源。本文与19个计划均**仅规划，没有修改实现、创建实施树或运行任何代码/用例**。原审计见[能力与示例审计](ace-capability-gap-audit-2026-11-19.md)；本文在语言侧继续下钻，不能把原审计中的最小缺口表当全分母。

## 1 目标、边界与“齐平”的含义

目标是**旧AutoVM真实可达且经规范认可的语言能力，在新AV普通程序/脚本/模块/会话中可用**：原合法源、结果/副作用/错误/生命周期与公开语言语义相符。不是仅让新AST存在、让opcode数相等、用多个不相容profile分别跑代表例；也不是复制旧VM错误。源码兼容优先，只有已批准、可追溯的规范差异允许显式兼容规范化，禁止换mini/手写loop/改expected后把原行标绿。

当前尚不能判齐平：管道、通用命名函数值、裸索引读、异构对象/键、跨函数catch、borrow/generator、TaskRef/async组合、属性、Script/Python协议及普通工程跨profile都有具体差额或缺证。另一面，数值、聚合、泛型、闭包、CTE、调度、FFI已有大量实现，不重建它们。

本路线负责语言的语法/绑定/类型/控制效果/资源语义及所需已有库/host接面；标准库全API、完整UI/2D/3D示例、AC/A2R/A2PY对齐、产品迁移/发行/旧实现退役与AAC独立留原路线。旧语言依赖的库或外部provider缺失逐case交真实owner，语言行保持具名依赖；不能借此把同一语法能力一笔排除。领域Node的普通表达式/业务CPU语言面纳入，renderer/GPU/游戏完整验收不在此包。

## 2 取证方法与当前差额

### 2.1 旧能力不能仅从AST或旧expected推断

959的L表49行覆盖语言族；本轮读取旧AST/parser/codegen、ACE resolve/MIR/AV与已有Spec。1066现行文档式语料必须是唯一作者源，历史.at/.expected仅溯源。磁盘静态扫描`test/vm`得到506份.case.ad，数量与1066历史发现数相同，但未执行生产发现/测试；约束角色、条件、内联Rust与外仓语言输入需AVP-00逐键确认，不能写“506已通过”。

需特别排除的误读：

- 旧函数默认参数仅parser-only，不继承任意默认作用域执行承诺；ACE963已有真实填充。
- 旧表达式级CT常在运行时求值；ACE936/958有真实AV CTE，966有装配折叠，应合流而不是退回伪CT。
- 旧MacroCall仅debug/info/warn/error映射Log；未知宏静默忽略。不是旧任意文本宏系统，不在本路线凭空设计宏定义/hygiene语法。
- 旧Box/Arc、union/grid、部分tag/Scene/NavCall、Lambda有parser-only/AST桩子项；新侧已交增强保回归，但不冒旧运行承诺。
- 旧`propagate_err`的expected为裸4000000、borrow_mut旧样例不写回、int宽度/混宽cast等已有差异或bug。用固定独立规范oracle，不能自动复制数值。

### 2.2 按族的差距与验收方向

| 能力族 | 实际差额/边界 | 生产批次 | 关键独立判据 |
|---|---|---|---|
| 标量/文本 | 已交961/972，完整普通入口组合、FStr类型/源序、CStr/NUL/Unicode与转换差异需补 | AVP-01 | >2^53宽整数精确、checked溢出、标量4/字节9、插值副作用一次 |
| 调用/管道 | D-963-C1；旧无类型函数与命名调用推导/绑定；不新增自由函数管道注入 | AVP-02 | 原管道3/18/18、实参源序且按名绑定、非法邻例 |
| 函数值/闭包 | source.fn-as-value；已有lambda/callback不替普通函数引用 | AVP-03 | double作为值/实参=42、局部栈与捕获、最后owner释放 |
| 集合/place | D-962-C1裸读、List.new旧构造、HOF真实API与nested写回 | AVP-04 | 每个base/index/RHS一次、原HOF值、完整结构未丢字段 |
| 类型/模式 | 962/965切片需nominal/enum各种payload与嵌套模式组合，旧parser-only增强另计 | AVP-05 | payload真实值/guards与绑定作用域/重复和穷尽 |
| 动态/null | 同型对象与动态源码域有限、旧int/bool/string键、异构对象与深层alias | AVP-06 | 异构读写正确、两实例隔离、缺键/类型/循环策略具名 |
| 模块/global | AV已有global，owned record全局.move与完整值域/初始化/会话仍有限 | AVP-07 | once/diamond/版本隔离、move/替换释放、失败恢复首次读取 |
| 借用/资源 | 1035已有资源record闭包；部分source clone、move后使用与复杂借用待补 | AVP-08 | 合法借用和同型非法逃逸区分，所有出口parent live=0 |
| 错误/控制 | 965跨调用fail不被调用方catch；普通null/index错误与host故障需分层 | AVP-09 | catch到原错误、跨帧finally顺序/一次、资源归零，host撤权不被吞 |
| 泛型/spec/ext | 已交963，普通工程局部注解/跨模块资源/函数值和失效组合 | AVP-10 | 实例实际执行/约束拒绝、同名包隔离、改body结果变化 |
| 生成器 | 已有惰性/close；借用参数一刀切拒绝、资源payload和close错误恢复需补 | AVP-11 | 不跨挂起借用合法、yield/drain/break/close状态与owner |
| async/task | 已有968/1067；main-only spawn、TaskRef存字段/参数、~{}/future与普通语言组合 | AVP-12 | 真park/唤醒/FIFO/背压、跨函数ask与value、late/关闭无复活 |
| CT/宏 | 966折叠/936真CT分离；旧日志宏与普通函数糖不等价 | AVP-13 | CT实际dispatch＋运行IR无残留、显式target、缓存失效与预算 |
| 标注/元数据 | #[with]共享语法缺，test/foreign/文件模式/target透传各consumer边界 | AVP-14 | 原属性保持到真实consumer、相同约束、错误位置、metadata变更失效 |
| FFI语言 | 967已交多面，普通fn/owned/async/回调的统一接缝与旧smoke弱oracle | AVP-15 | 真provider变更敏感、长度9、撤权dispatch=0、统一owner |
| Script/Python | 1026不等全.as/with/对象协议；有限文本、keyword与嵌套实参边界 | AVP-16 | 原标准操作/异常分类、Unicode/NUL、__exit__真实调用与原源模式 |
| 普通工程/领域 | 17个profile存在不是全工程组合证明；CPU域语言应复用统一checker | AVP-17 | 同包语言/库/CT/async/FFI组合，Node真实CPU值与恢复；禁旧fallback |

## 3 AutoDown测试设计

用户本轮明确：1078–1096只验AV，非AV检查先加**检查级#ignore**，后续开发AC/A2R等时再逐项打开。共用Auto源码/原预期/义务映射保持，非AV未实现、失败、缺工具、未执行不进入AV开发/复审/合入门；不计PASS，也不阻AV独立出口。原1078要求最小AC真执行取消。#ignore由1078接入现有CheckSpec.skip，当前尚未实现，实际测试本轮不改。


### 2026-11-19 r3 · 共享作者、当前AV-only

用户已明确要求按语言特性集中组织ACE语言AutoDown语料。具体合同见[共享语料与迁移设计](../testing/ace-feature-organized-autodown-corpus.md)，由1078建立`IF-ACE-FEATURE-CORPUS/1`（拟）和原AV基础入口；本轮只改规划，没有移动或改写实际测试。

唯一活跃作者根拟为`crates/auto-lang/test/cases/language/`，按basics/numbers/text/calls/functions/collections/types/patterns/dynamic/modules/ownership/errors/generics/generators/async/comptime/metadata/ffi/script/domain/composition命名；公共总选择拟`crates/auto-lang/test/cases/language.suite.ad`。不再新建`ace-m2/av-language-parity/`或每计划编号文件作为长期作者。AV/AC/A2R各视图只选择同一Case，追加本目标check、独立oracle和真实结果，不再复制Auto源。

每Plan §6保72个代码Case需求草案（整体数量不变），实施后canonical文档是唯一活跃作者，Plan只保Case ID/需求/验收与历史样例。已有稳定Case/check ID保留或在旧→新映射明确继承，不按旧ID中的av/计划号决定目录。1078–1096已逐包给语义目录和迁移职责。

AVP-00新增`SourceProfile::Auto`（拟名）、源码准入键`ac.source.language.av/v1`与`ac-av/language-source-run-v1`（拟注册recipe）仍为同一共享前门/worker薄接面；不因测试目录中立就新增解释器或要求AC/A2R同时实现全语言。首批复用已交基础，同一基础Case只运行AV，AC-native/旧VM检查先#ignore；A2R生成金样与编译/运行分开，未交run适配器保pending和具名owner，不能靠emit补绿。

`case-av-language-parity`改为公共语料AV视图，保旧检查ID；选择依据真实注册目标/动作/leg，不假设所有AV检查都叫av。现有`--check av`只是精确check选择。1078在原TestCase/固定组机制中核并按需补目标筛选与受控shared引用；拟议接口未交前不假称可运行。未知字段/recipe、重复ID、缺输入与零选中继续严格拒绝，不另造发现/注册体系。

### 旧用例逐批迁移和去重

旧ACE活跃case、VM/A2R及其它适用族先冻结原source/project、虚拟路径/模式、oracle、target动作、provider/授权/预算/平台/ignore/性能条件与所有读者。**完整输入与语义等价**才合并Case；同名、同输出、仅“都报错”都不够。不同输入/条件/错误或资源角色保变体/独立check。目标产物不同可共享Auto而保目标oracle；弱smoke与旧bug不借去重改expected。

原→新映射拟记录在`language/migration/legacy-map.tsv`（数据，不作运行注册器）；每条旧case/check/角色保可追溯关系。按“冻结分类→canonical唯一作者→规范化声明/输入/预期与旧新真实结果对照→切suite/fixed group/worker/工具消费者→撤重复活入口→逐原义务关账”执行。历史M1签章输入和归档收据不改，具名历史副本可以冻结留存但不能继续作活跃作者。原义务数与canonical唯一执行数分别计，不因去重文件变少减少验收义务。

每个特性随AV交付迁例/补AV反例；非AV定义可保留但本轮#ignore/未选，后续目标任务随自身进度逐项启用，不反卡AV；最终1096核AV语言与迁移完整性，1068核三端，985/B接最终来源。AAC的CL、UI/AGE的公共语言切片同源消费，CS主体/typed IR/正式压力/完整产品保原owner和原oracle。后端pending、未跑和不适用分列，不能隐去required或以另端通过冲销。

本系列只选AV及其共享正确性职责，混合组非AV执行腿按阶段过滤，不因共享变更加native/A2X完整功能门；FC控制按AV范围执行，跨端实际运行控制留后续目标任务，新增FC-08/09核#ignore隔离与逐项恢复。普通组排除performance，Windows不执行Linux/WSL；外部依赖缺失不是PASS，原条件不减。

## 4 计划、依赖与执行顺序

建议按下表AVP编号逐包交付，每包独立work→review→merge。表中依赖为真实必需来源；已交接口允许下游切片先行，不把1068/985/B/AAC/UI整包设为本路线前置。正式取号用仓库new-plan脚本，4位号沿仓库现行规则；不按旧技能三位限制截断。

| 批次 | 正式计划 | 主体 | 新路线硬依赖 | 代码case种子 | 正式输出 |
|---|---|---|---|---|---|
| AVP-00 | [PLAN-1078](../../plans/1078-av-parity-baseline-and-autodown.md) | ACE按语言特性共享AutoDown语料与AV普通源码基线 | 无 | 3＋全部所属旧行/补充反例 | `IF-AV-PARITY-BASE` |
| AVP-01 | [PLAN-1079](../../plans/1079-av-parity-scalars-and-text.md) | AV标量、转换与文本语言语义补齐 | AVP-00 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-SCALAR` |
| AVP-02 | [PLAN-1080](../../plans/1080-av-parity-calls-and-pipeline.md) | AV调用绑定、隐式函数签名与管道补齐 | AVP-00, AVP-01 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-CALL` |
| AVP-03 | [PLAN-1081](../../plans/1081-av-parity-function-values-and-closures.md) | AV命名函数值与闭包全链补齐 | AVP-00, AVP-02 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-FUNCTION` |
| AVP-04 | [PLAN-1082](../../plans/1082-av-parity-collections-and-places.md) | AV数组、集合语法与嵌套place补齐 | AVP-00, AVP-01, AVP-02, AVP-03 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-COLLECTION` |
| AVP-05 | [PLAN-1083](../../plans/1083-av-parity-records-variants-and-patterns.md) | AV名义类型、枚举与模式组合补齐 | AVP-00, AVP-01, AVP-02 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-TYPE` |
| AVP-06 | [PLAN-1084](../../plans/1084-av-parity-dynamic-objects-and-null.md) | AV动态值、对象键与null语义补齐 | AVP-00, AVP-01, AVP-04, AVP-05 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-DYNAMIC` |
| AVP-07 | [PLAN-1085](../../plans/archive/1085-av-parity-modules-and-global-state.md) | AV模块、全局状态与初始化补齐 | AVP-00, AVP-03, AVP-05 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-MODULE` |
| AVP-08 | [PLAN-1086](../../plans/1086-av-parity-borrows-moves-and-storage.md) | AV借用、move与资源值组合补齐 | AVP-00, AVP-03, AVP-04, AVP-05, AVP-07 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-OWNERSHIP` |
| AVP-09 | [PLAN-1087](../../plans/1087-av-parity-errors-and-unwinding.md) | AV跨调用语言错误与控制流展开补齐 | AVP-00, AVP-04, AVP-05, AVP-06, AVP-07, AVP-08 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-ERROR` |
| AVP-10 | [PLAN-1088](../../plans/1088-av-parity-generics-specs-and-methods.md) | AV泛型、spec与方法决议组合补齐 | AVP-00, AVP-02, AVP-03, AVP-05, AVP-07, AVP-08 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-GENERIC` |
| AVP-11 | [PLAN-1089](../../plans/1089-av-parity-generators-and-stream-lifetimes.md) | AV生成器与迭代生命周期补齐 | AVP-00, AVP-05, AVP-08, AVP-09 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-GENERATOR` |
| AVP-12 | [PLAN-1090](../../plans/1090-av-parity-tasks-async-and-actors.md) | AV任务、异步块与Actor语言补齐 | AVP-00, AVP-03, AVP-05, AVP-07, AVP-08, AVP-09, AVP-11 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-SCHEDULER` |
| AVP-13 | [PLAN-1091](../../plans/1091-av-parity-comptime-and-macros.md) | AV编译期执行与宏兼容补齐 | AVP-00, AVP-01, AVP-02, AVP-07, AVP-10 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-CT` |
| AVP-14 | [PLAN-1092](../../plans/1092-av-parity-attributes-and-metadata.md) | AV语言标注、元数据与反射接面补齐 | AVP-00, AVP-10, AVP-13 | 3＋全部所属旧行/补充反例 | `IF-AV-PARITY-METADATA` |
| AVP-15 | [PLAN-1093](../../plans/1093-av-parity-foreign-language-surface.md) | AV受检FFI语言接面与回调补齐 | AVP-00, AVP-01, AVP-03, AVP-08, AVP-09, AVP-12, AVP-14 | 3＋全部所属旧行/补充反例 | `IF-AV-PARITY-FOREIGN` |
| AVP-16 | [PLAN-1094](../../plans/1094-av-parity-script-and-python-protocols.md) | AV脚本模式与Python对象协议兼容补齐 | AVP-00, AVP-06, AVP-09, AVP-14, AVP-15 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-SCRIPT` |
| AVP-17 | [PLAN-1095](../../plans/1095-av-parity-ordinary-project-composition.md) | AV普通工程、会话与领域语言组合收口 | AVP-00, AVP-01, AVP-02, AVP-03, AVP-04, AVP-05, AVP-06, AVP-07, AVP-08, AVP-09, AVP-10, AVP-11, AVP-12, AVP-13, AVP-14, AVP-15, AVP-16 | 4＋全部所属旧行/补充反例 | `IF-AV-PARITY-PROJECT` |
| AVP-18 | [PLAN-1096](../../plans/1096-av-parity-full-language-certification.md) | AV与AutoVM语言齐平完整验收 | AVP-00, AVP-01, AVP-02, AVP-03, AVP-04, AVP-05, AVP-06, AVP-07, AVP-08, AVP-09, AVP-10, AVP-11, AVP-12, AVP-13, AVP-14, AVP-15, AVP-16, AVP-17 | 3＋全部所属旧行/补充反例 | `IF-AV-LANGUAGE-PARITY/1` |

阶段出口：

1. **P0基线**：AVP-00。按特性唯一作者/公共suite与原义务映射、普通源码薄接面及AV执行/#ignore隔离可用，交IF-ACE-FEATURE-CORPUS/1；后端未交行明确pending，尚不宣称完整语言。
2. **P1日常语言主体**：AVP-01..10。标量/调用/函数值/集合/名义类型/动态/模块/资源/错误/泛型在普通入口可用；已批准差异有逐项oracle。
3. **P2效果与生态语言**：AVP-11..16。生成器/async/CT/标注/FFI/script全部required能力与资源边界真实执行。
4. **P3完整语言齐平**：AVP-17..18。同一普通工程与领域CPU接缝组合、全义务真实验收，交`IF-AV-LANGUAGE-PARITY/1`；未跑/unknown/无owner任一存在则不能签。

每一步的新测试随本包交，不把所有测试拖到最后认证。最后不是另一个新框架，而是汇合既有逐项证据并重验失效/组合/遗漏行。

## 5 跨仓与其它路线接口

| 外部owner/计划 | 本路线消费/交付 | 依赖方向与验收边界 |
|---|---|---|
| auto-down / 1066测试协议 | 复用pinned parser、文档身份/比较与已有case suite | 当前格式已能表达大部分目标，优先语言仓薄adapter；如确需外仓协议修改先按其流程规划，不擅改vendor或建新解析器 |
| 960–968/1035/1064/1065/1026 | 各已交language/stdlib/FFI/py producer | 复用实际已交切片，不重开归档Plan，也不把旧窄面成功当全语言 |
| 967/971/978 | provider/shim/统一资源、test工具、SDK/构建接口 | 语言批生产一次共享前门，原owner继续生产外部provider/工具；本路线消费正式接口，不能API缺失默默SKIP |
| 982 CONFIG | IF-982-CONFIG-ACE及通用模块表达式 | CONFIG全链仍由982生产一次；AVP-07/17只补通用语言并做真实消费，不反置982整包前置 |
| 1068 | 本路线提供 AV 正式能力及按特性共源 AutoDown/迁移映射，AC/A2R 实施按具名生产计划，1068 收证并做三端汇总 | 1068非图形/已有接口可先行；最终全语言AV列须收IF-AV-LANGUAGE-PARITY/1，不由它再实现一份AV功能，不反等完整AAC |
| 979/981、1071/1075、auto-ui/age-examples | 普通语言/Node CPU描述与事件，后续UI/2D/3D爬梯消费 | AVP-17核CPU源/语义接缝，不代签窗口/GPU/游戏；UI完整逐例活动承接缺口仍归UI路线，本次不冒解决 |
| 985/986/987、AutoOS与应用 | 提供普通AV语言完整来源 | 985/B最终接收，A已签来源不重开；产品测试/安装/退役各原owner，不因语言证书自动关闭 |
| AAC1044..1056 | 可消费本路线已交语言/库切片 | AAC独立后续/与M2-B并行，非本路线硬门；AAC未用的语言特性仍须独立case覆盖 |

当前主检出有其它会话在途文档/未跟踪文件，只有本轮具名设计/计划/关联簿记改动属此scope；不批量格式化/提交/删除其它文件，不创建共享implementation树。

## 6 49个语言族的完整承接

本表挂接原L行，不新造语言分母；AVP-00最终逐case/check细化。一个L行可有旧真实/旧bug/AST-only子项混合，不能整行统一豁免。各分母状态提升必须有当前代码和原独立oracle，不由计划文字充实。

| 原行 | AV路线owner | 核查分类 |
|---|---|---|
| L-01 | AVP-01 | 已批准差异/旧实现与新规范双账 |
| L-02 | AVP-01 | 已批准差异/旧实现与新规范双账 |
| L-03 | AVP-01, AVP-15 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-04 | AVP-00 | 无此形，不凭空扩语言 |
| L-05 | AVP-01 | 已批准差异/旧实现与新规范双账 |
| L-06 | AVP-08 | 已批准差异/旧实现与新规范双账 |
| L-07 | AVP-08 | 已有/缺口与组合面按实际case核 |
| L-08 | AVP-08 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-09 | AVP-08 | 已有/缺口与组合面按实际case核 |
| L-10 | AVP-05 | 已有/缺口与组合面按实际case核 |
| L-11 | AVP-05 | 已有/缺口与组合面按实际case核 |
| L-12 | AVP-05 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-13 | AVP-05 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-14 | AVP-05 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-15 | AVP-05 | 已有/缺口与组合面按实际case核 |
| L-16 | AVP-04, AVP-06 | 已有/缺口与组合面按实际case核 |
| L-17 | AVP-05, AVP-09 | 已批准差异/旧实现与新规范双账 |
| L-18 | AVP-05 | 已批准差异/旧实现与新规范双账 |
| L-19 | AVP-04 | 已有/缺口与组合面按实际case核 |
| L-20 | AVP-06 | 已有/缺口与组合面按实际case核 |
| L-21 | AVP-06, AVP-09, AVP-16 | 已有/缺口与组合面按实际case核 |
| L-22 | AVP-03 | 已批准差异/旧实现与新规范双账 |
| L-23 | AVP-03 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-24 | AVP-02 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-25 | AVP-02 | 已批准差异/旧实现与新规范双账 |
| L-26 | AVP-09 | 已批准差异/旧实现与新规范双账 |
| L-27 | AVP-09 | 已有/缺口与组合面按实际case核 |
| L-28 | AVP-09 | 已有/缺口与组合面按实际case核 |
| L-29 | AVP-12 | 已有/缺口与组合面按实际case核 |
| L-30 | AVP-11 | 已有/缺口与组合面按实际case核 |
| L-31 | AVP-12 | 已有/缺口与组合面按实际case核 |
| L-32 | AVP-13 | 已有/缺口与组合面按实际case核 |
| L-33 | AVP-13 | 已批准差异/旧实现与新规范双账 |
| L-34 | AVP-13 | 已有/缺口与组合面按实际case核 |
| L-35 | AVP-14, AVP-15 | 已有/缺口与组合面按实际case核 |
| L-36 | AVP-13 | 已有/缺口与组合面按实际case核 |
| L-37 | AVP-07, AVP-15, AVP-16 | 已有/缺口与组合面按实际case核 |
| L-38 | AVP-07, AVP-10 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-39 | AVP-10 | 已有/缺口与组合面按实际case核 |
| L-40 | AVP-10 | 已有/缺口与组合面按实际case核 |
| L-41 | AVP-07 | 已有/缺口与组合面按实际case核 |
| L-42 | AVP-10 | 已有/缺口与组合面按实际case核 |
| L-43 | AVP-12 | 已有/缺口与组合面按实际case核 |
| L-44 | AVP-17 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-45 | AVP-00 | 旧parser-only/AST-only子项须单列，不冒legacy运行硬门 |
| L-46 | AVP-00 | 无此形，不凭空扩语言 |
| L-47 | AVP-02, AVP-03 | 已批准差异/旧实现与新规范双账 |
| L-48 | AVP-04, AVP-08 | 已批准差异/旧实现与新规范双账 |
| L-49 | AVP-01 | 已批准差异/旧实现与新规范双账 |

## 7 当前VM磁盘输入清单（静态，不是运行分母）

[506份旧文档的逐case/check/原oracle与主承接计划](av-autovm-legacy-case-map.md)已逐项列出；每个计划§6也链接自己的清单，原源码点击即可读取。这里是规划分派快照，实际语义、发现角色与运行支持仍须按AVP-00核验。

下表506份文档来自本轮磁盘盘点；生产discover与1066 manifest、约束角色和Rust内联/外部输入由AVP-00重核。aavm2保research/手动条件，不把研究程序转日常硬门；无文档.at须具名审查是否被实际消费，不能直接丢。cookbook目录另有133份文档，不是133个VM语言必需例，按实际adapter与语言/库角色拆分。

| VM族/目录 | case.ad份数 | 历史/未注册.at数 | 状态与后续 |
|---|---|---|---|
| [01_basics](../../../crates/auto-lang/test/vm/01_basics) | 11 | 11 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [02_bit_ops](../../../crates/auto-lang/test/vm/02_bit_ops) | 5 | 5 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [03_variables](../../../crates/auto-lang/test/vm/03_variables) | 8 | 8 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [04_control_flow](../../../crates/auto-lang/test/vm/04_control_flow) | 13 | 13 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [05_loops](../../../crates/auto-lang/test/vm/05_loops) | 5 | 5 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [06_arrays](../../../crates/auto-lang/test/vm/06_arrays) | 6 | 6 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [07_objects](../../../crates/auto-lang/test/vm/07_objects) | 11 | 11 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [08_strings](../../../crates/auto-lang/test/vm/08_strings) | 18 | 18 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [09_functions](../../../crates/auto-lang/test/vm/09_functions) | 21 | 21 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [10_types](../../../crates/auto-lang/test/vm/10_types) | 18 | 31 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [11_compound_ops](../../../crates/auto-lang/test/vm/11_compound_ops) | 6 | 6 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [12_type_coercion](../../../crates/auto-lang/test/vm/12_type_coercion) | 6 | 6 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [13_collections](../../../crates/auto-lang/test/vm/13_collections) | 35 | 35 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [14_borrow](../../../crates/auto-lang/test/vm/14_borrow) | 18 | 18 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [15_nested_mutation](../../../crates/auto-lang/test/vm/15_nested_mutation) | 15 | 15 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [16_option_result](../../../crates/auto-lang/test/vm/16_option_result) | 22 | 22 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [17_cffi](../../../crates/auto-lang/test/vm/17_cffi) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [17_modules](../../../crates/auto-lang/test/vm/17_modules) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [18_ffi](../../../crates/auto-lang/test/vm/18_ffi) | 54 | 53 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [19_rust_std](../../../crates/auto-lang/test/vm/19_rust_std) | 10 | 10 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [20_permission](../../../crates/auto-lang/test/vm/20_permission) | 2 | 2 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [20_rust_ffi](../../../crates/auto-lang/test/vm/20_rust_ffi) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [21_conv](../../../crates/auto-lang/test/vm/21_conv) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [22_generator](../../../crates/auto-lang/test/vm/22_generator) | 4 | 4 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [23_actor](../../../crates/auto-lang/test/vm/23_actor) | 13 | 13 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [24_generics](../../../crates/auto-lang/test/vm/24_generics) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [25_method_u64](../../../crates/auto-lang/test/vm/25_method_u64) | 12 | 12 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [26_str_method_on_heap](../../../crates/auto-lang/test/vm/26_str_method_on_heap) | 2 | 2 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [27_function_reference](../../../crates/auto-lang/test/vm/27_function_reference) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [28_enum_methods](../../../crates/auto-lang/test/vm/28_enum_methods) | 6 | 6 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [29_list_shims](../../../crates/auto-lang/test/vm/29_list_shims) | 2 | 2 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_bootstrap](../../../crates/auto-lang/test/vm/99_bootstrap) | 108 | 108 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_idiom2](../../../crates/auto-lang/test/vm/99_idiom2) | 13 | 13 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_idiom_probe](../../../crates/auto-lang/test/vm/99_idiom_probe) | 16 | 16 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_misc](../../../crates/auto-lang/test/vm/99_misc) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_p588debt](../../../crates/auto-lang/test/vm/99_p588debt) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_plan230](../../../crates/auto-lang/test/vm/99_plan230) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_plan231](../../../crates/auto-lang/test/vm/99_plan231) | 10 | 10 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_plan664](../../../crates/auto-lang/test/vm/99_plan664) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_py_dispatch](../../../crates/auto-lang/test/vm/99_py_dispatch) | 2 | 0 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_script_err](../../../crates/auto-lang/test/vm/99_script_err) | 5 | 0 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_short_circuit](../../../crates/auto-lang/test/vm/99_short_circuit) | 3 | 3 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_slice](../../../crates/auto-lang/test/vm/99_slice) | 1 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_spec_dispatch](../../../crates/auto-lang/test/vm/99_spec_dispatch) | 3 | 5 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [99_str_natives](../../../crates/auto-lang/test/vm/99_str_natives) | 0 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |
| [aavm2](../../../crates/auto-lang/test/vm/aavm2) | 2 | 165 | 研究/显式手动条件单列，不转为日常硬门；普通AutoVM语言角色仍由AVP-00核 |
| [for_outer_var](../../../crates/auto-lang/test/vm/for_outer_var) | 0 | 1 | 按生产发现/角色再核，原.at是冻结历史或未注册输入，不作为活跃作者源 |

## 8 本轮证据与非实施声明

| 来源 | SHA256（当前工作区内容） |
|---|---|
| [overview.md](../../specs/overview.md) | `bfd17555d7fa0f0c933785fbe505724b1c4a285df046717756eb2d5761f66e69` |
| [m2-denominator.md](../../specs/ace-migration/m2-denominator.md) | `d39f8420ce3800224b64bd51fea169e6dbfbcabc82de8bedb1fcb52ff88958a6` |
| [m2-generics.md](../../specs/ace-migration/m2-generics.md) | `91e902aa5bfabb0f06e7976da64e6f9e72ffc484c57515a8937e145a9085e235` |
| [m2-aggregates.md](../../specs/ace-migration/m2-aggregates.md) | `3ab46b8b004859d742265c5767a3c88a5354f5e241c56bfb271051e194dbd075` |
| [m2-control-effects.md](../../specs/ace-migration/m2-control-effects.md) | `d4cd66b3c6b41bd9b8d6127652999eb7689c0187faed30bb5ac3a7d3f460fb9d` |
| [m2-metaprogramming.md](../../specs/ace-migration/m2-metaprogramming.md) | `50dfe45b6a3ec7997ccf7bc5be02f1e853f4bdc7dee3bc23a3d401abb5c10423` |
| [overview.md](../../specs/auto-lang/comptime/overview.md) | `0a960892898484524c66896a8a8fbd781c65da4a5350c9fe03eb8a8d99327130` |
| [null-family.md](../../specs/auto-lang/vm/design/null-family.md) | `9c8b710832f5c569d197790ef57bb98dcb5cfddebf272d1a4264811bda1db0f8` |
| [document-cases.md](../../specs/auto-test-case/document-cases.md) | `71a8a1813f535f54cd565c5c215acad3f44c9c500db8d3f3a7008b914c0dedeb` |
| [author.rs](../../../crates/auto-test-case/src/author.rs) | `0eb0585ac9f5b9460b9a40035172c0050de91819fdf763c61ede0727e19fd48a` |
| [stmt_emit.rs](../../../crates/auto-lang-core/src/vm/codegen/stmt_emit.rs) | `c2779a620090f5423731282a0e546029b08939ddda5e2ac00dca7ac07d4d448d` |
| [resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | `54978de5495b2ada8f019a86e5b3623537f1b2d997d5ca274ce11cc20a88ea68` |

已有Specs存在快照滞后：document-cases.md“仅四recipe”段落已落后author.rs全部已注册family；旧MacroCall被泛称文本宏也不准确，本设计按实际Log路由纠正。指纹仅使本轮调查可追溯，开工须核漂移；不把source hash当语义验收。

本轮规划静态预检：19个新Plan号唯一、全部drafting 0/8；72个代码case ID唯一；506份旧文档有主承接；L-01..49共49行齐；依赖图无环；842个本地链接及1312个表格行结构检查通过，空输出围栏确为零字节而非多一个换行。这里只核规划结构、路径与声明，不证明Auto代码已经获编译器接受或实现正确。

没有执行compiler/test/demo/GPU/GUI，所有planned case都未验证。不新增全局性能/全仓回归门，也不改旧归档收据或已有语义expected。1068/985只增加AV正式来源的阶段接收关系，其它会话后续修订与编号不回退、不覆盖。

2026-11-19 r2规划修订：统一作者迁移到中立language根，72例原代码需求保留；旧506文档快照按语义归类迁移，AC/A2R/AAC/产品接收补同源规则。本轮未迁例、未改expected、未运行；旧静态预检数字为r1历史，r2按实际改动重新核链接/依赖。
