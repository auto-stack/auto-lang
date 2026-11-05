---
status: proposed
describes: target-state
created_at: 2026-10-16
updated_at: 2026-11-04
implemented_by_plans: []
---

# Design 42：语义元信息、类型推导、反射与编译期执行

> **2026-10-28续接基线**：796已交付有限不可变meta/推导查询，817已交付真实AV标量CTE与A05冻结cache（见[auto-semantics](../specs/auto-semantics/project.md)、[auto-cte](../specs/auto-cte/project.md)）；完整泛型、类型/声明生成、双期反射尚需逐项扩面。helper body/类型查询/target/权限等完整输入不变才可跳过AV执行，不只比较CTE调用点源码。 统一进度快照与未完工作见[ACE续接基线](strategy/ace-implementation-handoff.md)，后续按current Spec/Plan及merge收据重核；本注不扩大原Plan验收。

用户要求：v0.6应规划完整meta信息及编译期/运行期反射；类型推导独立详细设计；元信息、类型、泛型实例化和AV编译期结果从第一天纳入增量与全局cache。本文确立架构边界及后续实施要求，具体语法、Schema字段/编号、首批反射API与支持矩阵仍需正式Plan冻结；不声明已实现、不修改787执行合同或现行Spec。

关联：[Design39](39-ac-incremental-syntax-and-language-platform.md)、[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[Design41](41-interface-metadata-and-foreign-abi.md)、[HIR语义](strategy/auto-hir-design.md)、[后端战略](strategy/auto-native-backend-evolution.md)、[ABI RFC](strategy/auto-native-abi-rfc.md)、[AutoDown测试](35-autodown-test-cases.md)。

2026-10-16后续确认见[Design43](43-ac-av-typed-mir-and-abc.md)：新引擎长期名为AV（先Rust、后Auto实现AAV）；CTE依赖checked HIR片段→最小Typed MIR→新ABC→AV。本文中目标态AVM称谓以AV为准，旧源码审计仍称旧AVM。最小MIR进入v0.6，性能优化另行分期。

## 1. 现状审计与重新划界

2026-10-16只读主检出4677d7361c0de7a7ad1d863c78f2d329c6ebb48e；以下是当前源码分布，不推断完整历史迁移已成功。

| 入口 | 观察 |
|---|---|
| crates/auto-lang/src/infer/context.rs | InferenceContext同时有局部type_env/constraints/scopes、Database和共享TypeStore；注释记录Universe→Database迁移 |
| crates/auto-lang/src/types.rs | TypeStore汇集类型/函数/spec/泛型模板；曾针对Parser/Codegen/infer多份注册表做统一 |
| crates/auto-val/src/types.rs | TypeInfoStore主要为按名字管理Value方法与运行时基础类型信息；不是完整语义库，内部含Rust函数指针 |
| crates/auto-lang/src/vm/codegen.rs | Codegen有构造用TypeInfo、参数信息、infer_ctx及type_store；后端仍承担推导/元信息职责 |
| crates/auto-lang/src/comptime/transformer.rs | 简单值直评；复杂表达式format回源码送VmInterpreter；存在整数截断/值回写不足，不能作为新语义合同 |
| docs/specs/auto-lang/comptime/design/{ctee-pipeline,comptime-eval}.md | 旧CTEE在index/infer前变换AST；hash节点被后续跳过。target注入、确定性/限额和来源存在明确缺口 |

因此不能简单说“Universe已经完全迁移到TypeInfoStore/Codegen”。新架构不继续共享一个可变Universe，不把Codegen升级为语义事实owner。旧实现可作语言案例/行为调查来源，旧bug和隐式降级不继承。

## 2. 三类数据、一个逻辑事实来源

完整meta表示：对已承诺语言形态和消费场景有完整、显式可追踪的描述，不由各后端自行补猜。不等于每个进程加载全部meta，也不等于所有meta随程序发布。完整Auto语言的覆盖仍要有支持矩阵，不能以基础类型demo冒称全语言反射完成。

| 类别 | 内容与owner | 存储/生命周期 |
|---|---|---|
| Semantic Facts | 声明/作用域/导入、类型/函数/字段/enum/方法、泛型约束、ownership/effect、属性、可见性、来源和查询结果；语义服务拥有 | 版本化不可变产物，按声明/函数体/递归组查询；共享store分页 |
| Inference Work State | 类型变量、约束/替换、候选集合、worklist、临时假设、诊断构造；单次查询/任务拥有 | 局部暂存，受预算/取消控制；不作为稳定全局事实或持久TypeId |
| Runtime Reflection Projection | 被选择的类型/字段/属性/成员描述，已授权的访问/调用adapter，必要target layout与模块代际 | 独立版本化工件；链接保留/剥离，运行时按需读；可调用代码有generation lease |

“单一事实来源”是共享语义查询合同和已提交不可变结果，不是唯一常驻HashMap或全局大锁。编译器、AV、native/A2X、FFI、LSP分别消费其投影，不维护互相改写的权威类型副本。codegen可有局部寄存器/栈槽/layout缓存，但不能发现新语义或悄悄推导为Any。

## 3. 身份、类型与物理布局

- DeclId含包/模块/名义声明身份，Revision/Fingerprint记录定义版本；内容相同不自动合并两个名义类型。文件移动/重命名是否保留逻辑ID由显式匹配合同决定，不承诺任意编辑永久稳定。
- SemanticTypeKey描述具体语义类型；类型参数/常量参数的绑定属于InstanceKey。进程内InternId/TypeId仅为短索引，持久产物使用版本化类型描述/引用；不存指针、Arc、共享锁、union-find父节点或函数地址。
- 类型的identity、接口指纹、实现body指纹、target layout指纹分别存在。字段/方法声明顺序在语言规定可观察时保留，不能为canonical hash排序而改变行为。
- 递归类型/相互引用声明按SCC组manifest+组内引用编码，不递归地要求每节点hash包含最终邻居hash；名义身份和递归组内容版本分开。
- TypeLayoutKey=(SemanticTypeKey, representation attributes, target, ABI/profile, layout algorithm version及实际依赖)。宿主VM对象布局不能作为交叉编译目标布局。
- CT类型值是对语义类型描述的受控引用，可参与泛型与反射；不是宿主内存地址，也不允许把错误/未解决类型当普通runtime值。

## 4. 查询式语义架构与管线

以下查询名为职责示意，不是已发布API：

```text
snapshot/CST → declarations/scope/import summaries
                         ↓
                signatures / generic templates
                         ↕
              infer/check body / instantiate
                         ↕
             compile-time evaluation request
                         ↓
             checked executable HIR fragment
                         ↓
                 Typed MIR → ABC → AV execution
                         ↓
             frozen CT value/type/diagnostic
                         ↕
            semantic queries continue / generated items
                         ↓
                 final Checked HIR
                    ↙          ↘
          Typed MIR → native   Typed MIR → ABC → AV
```

不能先要求整个程序final Checked，再执行决定类型的comptime；也不能把未检查工作态直接送VM。先检查所请求函数和可执行依赖，形成有明确profile的checked片段；结果回灌继续推导/实例化。普通运行时泛型替换未必调用VM，只有依赖编译期值/类型计算时才发起CT查询。

语义服务统一调度签名、body、实例与CT任务，记录依赖、等待/取消与发布状态。递归函数允许先收集显式签名再检查body；递归类型走组合同。comptime值/布局自依赖等无合法解的环报带查询链的错误；不能把所有环都当“再迭代直到好了”。有限单调分析才采用有终止条件的固定点，算法在专项Plan冻结。

CTE之前先设计新ABC及最小Typed MIR，实现Rust AV的必要执行子集；不等待完整runtime/插件/JIT，也不将旧AVM视为天然兼容。类型/meta投影、常量装载冻结和值/帧/调用模型分别审计，只有通过语义验证的旧组件才可复用。后端不回读旧AST或将表达式格式化为源码重推导；不另造用完即丢的CT解释器。v0.6最小MIR是正确性与多后端共用层，不以前置SSA/自有优化器为要求；完整旧后端迁移仍分阶段。

AC/AV/AJ共享语义服务与MIR，不再分别推导类型；详见[Design43 §8–10](43-ac-av-typed-mir-and-abc.md)。CTE先用AV解释即可，快速/优化JIT不是本版CTE前置；未来编译执行仍受同一语义、权限、预算及结果冻结合同约束。画像/优化不改名义类型身份。发布snapshot/profile选择的反射保留和实例集合是部署闭包输入，不按开发时实际运行过哪些类型裁剪。

## 5. 类型推导详细设计边界

### 5.1 算法与输入

推荐局部双向推导（synthesize/check）加显式约束worklist：输入是声明/导入签名、body、期望类型、generic substitution、语言/profile、target相关语义及内建能力；输出是表达式/绑定类型、已解析调用/重载、具体实例依赖、资源/effect义务及定位诊断。具体语言允许的隐式转换、字面量默认、泛化规则必须独立冻结，不把求解器的便利当语言规范。

先收集声明签名并建立作用域，之后按body/实例处理约束；变量遮蔽、同名导入和候选查找具有明确次序。参数/返回/字段预期类型向内传递，表达式类型向外合成。泛型模板可保留受约束参数，进入具体可执行profile的Checked结果不残留未解决变量。显式Dynamic/Any若语言支持，是合法类型，不是推导失败的兜底。

### 5.2 求解、错误与发布

类型变量/occurs检查、名义相等、结构匹配、常量参数、方法候选和约束满足各有诊断，不用格式化类型字符串判断相等。泛化/值限制、借用与effect如何相互制约在语言合同明示；不能无证据承诺完整HM、trait求解或依赖类型能力。

求解结果须确定：无序集合不决定重载选择；候选歧义受控拒绝。增加一个新方法/import可能改变原来的“唯一候选/不存在”，因此记录候选集合/作用域版本等**负依赖**，不能只记被选函数。

错误节点可支持IDE当前修订partial结果，但带状态/来源；成功的具体Checked门拒绝Unknown/Error/未解决约束。取消/资源超限/缺工具链是基础设施状态，不伪装类型错误；同版本真实类型诊断可按完整key缓存，取消/瞬时资源失败不永久记成程序非法。

查询粒度默认签名/声明/函数body/实例，跨边界递归按组；不为每个临时类型变量建立永久全局查询。求解器状态不能由不相干查询共享突变；发布结果绑定snapshot/配置，旧任务迟到不可覆盖当前诊断或根。

### 5.3 作为长期架构的约束

类型服务在所有后端之前；类型身份与表示分离；可持久输出与临时求解状态分离。后续加入泛型、反射、增量和并行时遵守同一边界，不继续将infer嵌入Codegen。性能迭代可以替换求解内部算法而不改变Schema/查询的可观察语义；算法版本变更进入失效规则，不能沿用旧成功凭证。

## 6. 编译期与运行期反射

### 6.1 完整编译meta与CT反射

编译期可查询语言允许的类型/字段/成员/属性、泛型参数/约束、声明/签名、可见性、来源、ownership/effect等事实，返回有Schema的描述。查询表达其真正读取面：列举全部字段依赖字段集合及顺序；仅取某字段类型依赖该字段；查询“不存在”也依赖查找域。源码正文/文档若可反射，正文/文档变化就是语义输入，不能统一把注释当无效改动。

是否允许构造类型、生成声明/表达式与访问私有meta，是分别需要冻结的语言能力，不因能读字段信息自动获得。生成项需有卫生/作用域/稳定身份、展开来源链及冲突诊断；不经字符串重parse掩盖来源。未实现反射操作明确unsupported，不返回空数组/Any伪成功。

### 6.2 runtime保留策略

建议按目标构建profile选择保留级别：none、identity/name、shape/attributes、经授权access/invoke adapters；语法/默认值待冻结。静态反射查询和显式保留声明形成链接根，动态“按任意字符串找类型”必须声明扫描/保留范围，不能自动保留整个依赖图。公开反射API的承诺在剥离时仍须满足；不能正常编译后运行时才悄悄缺meta。

描述数据与执行能力分离：看到字段名/offset不授予越过可见性/ownership的写权限，看到签名不证明FFI可调用。泛型运行时反射只覆盖实际实例化/明确生成的实例，不能默认VM/native都能运行时生成任意类型代码。runtime元信息从语义结果派生，不是另一个手工注册表。

定义指纹+ABI域+target/layout+实例+模块generation构成运行时描述的适用性。重载布局/方法改变时产生新版本；旧对象/回调/析构使用旧代描述/代码并保活，不能按同名全局TypeId替换。纯描述可持久/分页；执行地址在装载时绑定受控symbol/handle，cache不存原生地址。

## 7. 所有这些数据进入统一产物与cache

复用Design40的层级，不另建“meta数据库”绕过全局预算：

| 层级 | 实际应存的语义产物 | 不应作为稳定payload的东西 |
|---|---|---|
| A04 | 声明/作用域/导入摘要及来源映射 | 整个可变Parser/Universe |
| A05 | CT执行recipe/ABC及AV语义身份、冻结结果、类型/生成项描述、输入依赖manifest | VmInterpreter堆、任意宿主句柄/地址 |
| A06 | 类型/签名/字段/方法/约束、body推导结果、泛型实例及反射/FFI接口投影 | 未解决替换表、推导器内部指针 |
| A07 | final HIR与消费的语义/资源事实引用 | 输入trusted标志 |
| A08/A09 | 已实现的Typed MIR/lowering结果、ABC模块及版本/profile；共享语义/常量引用 | CLIF/解释器活对象、宿主调用地址或可反序列化trusted凭证 |
| A09/A10 | 描述数据/访问adapter及链接保留表、剥离策略指纹 | 把全部编译meta无条件塞进exe |
| A11 | 诊断、展开/推导/CT调用来源链、工具查询结果 | 迟到旧修订结果冒充当前 |
| A12 | 运行代际绑定/状态schema描述 | 运行中对象/可调用地址作为普通可淘汰cache |

Schema可映射Atom/Batom，热路径typed view、按声明/实例/组分块。大常量/类型图分块，热点解码缓存、索引、字符串池、活实例与求解暂存全部计入预算；一个语义服务不等于全仓全类型常驻。

### 7.1 key、依赖与命中

CT base key至少有callee实例/语义版本、常量与类型实参、profile/target/语言语义、HIR→MIR→ABC lowering及ABC/AV版本、允许能力和外部输入域。记录实际读取的函数实现、类型/字段/签名、解析候选域/反射集合、显式文件/环境值等完整依赖；上次依赖manifest先验证新鲜，验证本身经版本化查询进行，不能在命中后才发现漏读输入。动态控制流采用上次有效依赖集时，决定路径的条件及其输入也必须被覆盖。

“源码没变就命中”仅在所有依赖/配置也未变时成立。例：计算常量调用helper，helper body变就失效；计算sizeof(T)，字段或target/布局规则变就失效；列举字段生成代码，新增字段即使旧字段未变也失效；外部文件字节变而mtime不变仍失效。body-only修改是否传播看消费者是否读body，不用一刀切接口hash。

QueryKey是输入recipe，Artifact/内容身份是输出；重算结果未变可阻止无关下游继续失效。logical identity、输出内容、诊断snapshot各自处理；空白变更可复用不含来源的语义值，但来源映射/报错必须绑定当前修订，且反射读取source/doc时不能忽略相关变化。

### 7.2 确定性与结果物化

AV的CTE profile默认无任意文件/网络/时间/随机/FFI权限；有必要的宿主服务经能力broker提供快照输入及依赖记录。不可追踪/不可重放副作用明确拒绝可缓存语境，或标记为受限制的非缓存构建操作，不能取旧结果时悄悄省掉应发生的外部效果。

冻结结果只包含可移植标量/文本/聚合/类型引用及支持的常量图，保留别名/循环或明确拒绝不支持形态；不将指针/VM句柄直接写盘。原生lowering从冻结常量生成目标数据与重定位；宿主AV和目标机器的表示不是同一层。

成功命中跳过VM执行，但校验payload身份、版本/引用、依赖新鲜和本次消费准入；缓存值不是Checked HIR凭证。需要恢复HIR时仍经verify。坏块受控miss/重算，缺失再生输入则明确unavailable，不能把错误值变成0/Nil。

### 7.3 生命周期

依赖指纹用于失效，不默认强pin全部body/旧AST。若保留的常量结果消费时仍需其类型描述/数据页，这些是required-data；若编译进native后无需历史AV执行图，则只留recipe/溯源，不永久pin解释器工件。runtime活代际单独lease，不由普通LRU强制卸载。

相同完整key跨工程共享结果和不可变meta；overlay、配置、当前诊断和lease属于会话。两个producer同key需single-flight/冲突检测；结果与依赖manifest原子发布。增量元编程不能成为绕过Design40自动预算/回收的第二套缓存。

## 8. v0.6交付与实施拆分

用户确认v0.6要规划完整meta与编译期/runtime反射；这应改变长期架构而非仅加库函数。建议先冻结支持矩阵，要求每个已承诺语言形态有完整meta，分别列读meta、计算类型、生成声明、runtime保留/访问/调用的状态；不把“有TypeInfo结构”当作全部交付。

建议工作包（未分配Plan、非实施授权）：

1. 语义身份/Schema/查询、类型推导规则和代表性递归/泛型/反射探针；冻结完整覆盖矩阵与阶段准入。
2. 最小Typed MIR与新ABC/Rust AV执行合同及全路径探针；之后接有限实例化/CT查询回路。真实AV就绪后实现CTE，从首例持久结果/依赖、资源上限与AutoDown用例，不等语言完整后补cache。
3. 编译期反射及runtime描述投影/保留、链接/装载及代际兼容；与有限AutoABI/FFI合同共享类型身份，不让ABI或Codegen反过来拥有语义事实。
4. 逐项语言/元编程扩面与旧consumer迁移；完整旧AVM/A2X迁移及进阶优化分期；最小Typed MIR进入v0.6，详见Design43，不用本稿宣称全部已纳入某一在途Plan。

787仍只负责语法/分页，其partial CST不授类型或CT执行信任。AutoDown接线/ABI优先工作继续推进，同时上述语义合同须在泛型、元编程或新HIR→MIR→ABC/AV开始实施前形成正式可执行Plan。

## 9. 必须落地的区分性验证

| ID | 输入/变更 | 必须观察的结果 |
|---|---|---|
| M-01 | 同名不同模块类型、递归类型、重启后intern顺序不同 | 名义身份不混淆，持久ref重映射正确，递归图有界验证 |
| M-02 | 调用者不变，只改helper body/类型字段/target/VM版本 | 相关CT与实例化失效；未依赖查询仍可命中；结果与fresh一致 |
| M-03 | AST/依赖不变的第二构建及进程重启 | 真实AV执行计数0，冻结结果及native观察与fresh一致，不伪计数 |
| M-04 | 字段枚举新增成员、方法候选/导入新增、原不存在名字被定义 | 负依赖/集合依赖使旧结果失效，不误复用 |
| M-05 | 未解决类型、矛盾约束、歧义、递归CT/实例化环 | 定位/依赖链错误；无Any兜底、无无限求解；合法递归兄弟通过 |
| M-06 | type-dependent CT与跨宿主/target宽度 | 已检查片段可执行，目标类型/布局正确，无宿主地址/宽度泄漏 |
| M-07 | CT超步数/内存/取消、未授权IO、受控输入变更 | infra/权限/资源状态独立；不永久缓存非法结论；可追踪输入改变失效 |
| M-08 | 有效frame替换错误key、坏类型ref、CT结果到HIR | 真实消费入口拒绝；不能以frame自校验或持久trusted绕过 |
| M-09 | runtime none/name/shape/invoke策略与动态查找 | 保留承诺明确、无隐式全依赖保留；访问/调用能力有门控，构建策略入key |
| M-10 | 热重载布局变化，旧对象/callback/deleter仍活 | 旧描述/代码保活，新旧TypeId不串代，释放后可自动回收 |
| M-11 | 大类型/实例集合仅查一个签名，持续编辑/回收 | 页/索引/求解暂存有预算；不会每进程复制全类型库，空间长期收敛 |
| M-12 | 只改来源位置，body值相同；反射读取doc/source | 普通值可共享而来源更新；读取doc/source时相应失效，诊断链准确 |
| M-13 | meta查询/推导/实例化/CT各例及其端到端调用链，冷热/变更/压力分场景 | 每阶段从首例有真实span/计数/内存I/O，依赖不变CT命中时AV执行0且miss正控实际执行；代表负载预算与端到端亚秒目标关联，等待/嵌套查询不漏算或重算 |

各实施Plan须将适用项绑定实际入口/owner/正反例/资源负载和预期值；上表是设计验收要求，不是已执行测试。现有旧CTEE字符串往返/截断/数组退Nil不作为兼容正确性oracle。

## 10. 待冻结决定

首版反射语法/API与可见性、type-as-value支持、泛型/泛化规则、生成项卫生、递归求解允许域、target布局查询、冻结常量图类型、runtime默认保留/剥离和动态查找域、CT能力broker、资源预算、来源/隐私政策、Schema编码和版本。用户已确定方向与v0.6规划要求，以上技术细节仍需探针/Plan定界，不把草案编号当作实现或正式ABI发布。


## 10. 普通源码SQ、真实实例与AV CTE接线（2026-11-04，target-state）

用户授权本轮正式立项及后续规划复核，仅规划，不施工/建树/构建测试。934–936现为drafting r2，最新合同见[复核/写窗](../plans/reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)。正式拆为[SQ-A 934](../plans/934-source-semantic-query-and-invalidation.md)→[SQ-B 935](../plans/935-source-scalar-template-instances.md)与[SQ-C 936](../plans/936-source-av-cte-query-and-frozen-recovery.md)；C的私有执行服务提取可与B准备并行，最终含真实实例→CT→类型回灌联验，不能留模型接缝。调查/历史收据及输入hash见[基线与排期](../plans/reports/ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)。三包关闭ARCH-02/04相应有限机制职责，不替CA-D或最终ARCH总签章。

### 10.1 现有机制、缺口与为何三包

- 796真实有限i32/bool推导、FnSig/InferBody/RecordFields/ListCandidates、正负依赖/SCC/来源新鲜分类与S796持久页已交付；`reflect::InstanceKey`只含template/substitution，注释明确无代码生成。`adapter::project`是整Program投影，`InferBody`键有模块级src-fp；直接拿来暖调用会把未改body投影/位置漂移变成真实生产。
- 817不是假的CTE：真实checked callee→共享MIR/ABC→AV→executed envelope→796推导→HIR常量回填/verify→native观察已存在。普通源码还未自动产生CT请求；目前deck/anchor绑定与专用driver为主。其consumer会重新compile source，snapshot lane会以源码wrapper进行暖编辑，这些既有探针合法证据不能冒新普通源码P0接线。
- 依赖图实为`semantics[core-ast]→core`、`comptime→AV/native/core`、`native→AV`。让core/AV/A2X直接反依赖comptime会引环/耦合；把所有语义、模板、新CT语法、执行服务和四端恢复塞一包又无法独立验收。
- 因此A先交源码查询/依赖/事实消费与恢复；B独立交有限真实实例；C交从源码自动调度真实AV/冻结回灌。三包的唯一实施树/材料与独立review各自负责，不新增聊天或自动开工。

### 10.2 依赖方向与语义权威

拟新增`ace-source-query`职责包（路径`experimental/ace/crates/source-query/`，尚不存在）只依赖core、semantics/contract、artifacts以及必要可复用的sema持久库；**不依赖AV/native/目标后端**。它拥有SourceQuerySession/不可变事实、可恢复查询图、单体推导与实例任务；core提供消费事实的受检入口，不能反向依赖source-query。名字/资源规则仍由现core权威承担；有限i32/bool推导复用796算法，结果真实用于binding/expr结构ID上的类型/Checked HIR，不准跑完旧推导再只多写一份query日志。现core resolver本身产生VT，新facts端口需切断其旧类型综合而保留名字绑定；只屏蔽独立typecheck调用不能证明权威迁移。core verifier仍独立验事实/IR，不等于第二推导源。

新source-query需单体AST投影及纯声明输入，不能调用整module `adapter::project`/`current_fingerprints`遍历未改body再记M。928目录module字段由冷源FNV生成，只作语法来源/授权；新LogicalModule/Decl必须独立按package/module/owner绑定，不能用该源hash/Rc/绝对span作语义身份。796稀疏name/span facts亦不能直接回填，新增完整scope/local/node与body/context绑定；当前坐标经目录包装重锚，不原地改旧pin AST。允许遍历有限声明/候选目录，明确计字节/工作；改body只换该body版本，不换未读取者的语义键。796旧S796页/键、模型CT seam与旧探针保其兼容；新单位事实/来源刷新/实例页用独立版本化命名空间与producer，禁止同键改payload或假装旧全局src-fp已支持新P0。

CT用query层定义的中立executor端口，拟`CtExecutor`（名字待T-02冻结）只收已checked有限callee/实例闭包、参数/能力/预算/身份，返回typed executed/failure证据。拟从817提取最小`ace-source-cte`服务（路径`experimental/ace/crates/source-cte/`，尚不存在），依赖query端口、comptime-contract/core、shared lowering、av-core与artifacts；**不依赖ace-av前端会话或ace-native**。817旧probe再消费该服务，不能复制一套执行/cache；服务在worker/编译宿主的组合根注入。A2X HIR/provider/目标代码不依赖MIR、AV或Cranelift；其编译宿主可为CT显式组合AV服务，A2X打印仍只消费Checked HIR/资源事实。

旧SnapshotLane的前端FunctionIncrementalSession适配可保在probe侧，通过中立执行入口共享服务，不把该前端会话搬入最小服务引环。提取范围/依赖方向先由936 T-02整体可行性门证明，保持旧817closure/snapshot车道、producer/旧envelope及资源义务。新源producer必须区分来源：迁移817旧executed envelope只有完整身份/依赖/车道都满足才可复用，不把旧producer字面量冒源码新执行证明。

### 10.3 有限普通源码表面（新增目标，不是已有实现）

A无新语法：Core/Scalar现有i32/bool函数、局部/赋值/分支/循环、直接调用/命名绑定、前向显式签名与递归，从普通`.at`入口进入查询结果驱动的Checked HIR，并送AV/native/A2R/A2C。与796适用差额（如命名参数/算子/前向控制）由A枚举并补真实adapter/规则，不默认unsupported再走旧推导。其他资源/闭包profile保现机制与明确未迁范围，不以A冒全语言求解器重写。RecordFields/meta目录可只读消费源声明，不承诺record值级推导。

B复用Auto既有声明拼写`fn choose<T>(a T, b T, flag bool) T`，先限一个类型参数、通过实参/明确期望类型唯一确定的T=int/i32或bool（int/i32规范同型）；已有直接调用，不新增另一套`@[...]`语法。模板体可读T局部/参数/return，bool条件选择，标量常规子表达式按具体实例检查。例：

```auto
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
```

闭式oracle=42，至少两个实际checked具体实例，AV/native/Rust/C都真实执行。模板body改为交换分支后主程序=0，调用点不改亦应失效。类型参数必须可解，异质实参/无解/歧义具名拒，禁止Any。首期不含const泛型、约束/spec求解、模板方法、用户泛型record/owned资源类型参数、HOF或任意显式类型实参语法；这是覆盖边界，不豁免实例机制、依赖和恢复。新语法以显式版本化能力profile启用，原profile接受/拒绝与冻结差异不改；A与B的组合入口仍是普通源码，不是手写实例IR。

C复用Auto的`#{ expr }`拼写，限单表达式、i32/bool结果、纯scalar直接调用/有限递归与嵌套CT DAG；该表达式从普通`.at` AST自动产生query，不接收用户填的deck/placeholder/ct_ref→HIR位置绑定。例：

```auto
fn sum(a int, b int) int { return a + b }
fn main() int { let n = #{ sum(20, 22) }; return n }
```

冷/失效真AV结果42→冻结→局部推导事实→常量HIR重验→四端实际42；把sum改`a+b+1`而调用点不动，结果43。CT泛型组合`#{ choose(40, 42, false) }`须消费B真实实例，不将model substitution当实例执行。

有限source meta接缝采用**新增受能力保护的intrinsic声明**`ct_field_count(TypeName)`与`ct_has_field(TypeName,"field")`（最终符号身份在936 T-01/02冻结，不是已有stdlib API）：仅在CT表达式内、须`cte.meta.fields/v1`，TypeName绑定当前源中的名义type声明，field必须字面选择子。该meta源在显式Record声明能力的有限query profile中接纳type字段声明+i32/bool函数，不将record值/owned字段带入AV CT。返回i32/bool；TypeName/字段串只作为query selector，不进入AV运行值，不能伪称type-as-value或文本CT执行。以真实resolver绑定的intrinsic def身份识别，不按拼写绕过shadow；用户同名声明在该能力profile下具名冲突拒绝，缺cap/外部句法/未知类型按明确阶段拒绝。meta查询值作为实际AV scalar参数，如`#{ sum(ct_field_count(Pair), 40) }`；Pair两字段→42，加第三字段→43，HasField负依赖无→有使旧结果失效。字段shape/meta-only不是vm_execute，只有真实AV调用才计执行。

新CT/模板能力由统一语法规则/profile版本表达，普通旧profile既有拒绝不改；不改旧source/expected/accepted分母，新增文件化case独立登记。C不覆盖`#if/#for/#is`、任意多语句CT、外部mutable runtime局部/IO/env/FFI/type生成或完整反射；已识别但未支持的CT构造具名拒，不当注释剥掉继续编译。

### 10.4 查询身份与依赖矩阵

| 查询结果 | 必须读取并绑定 | 编辑后的正确动作 |
|---|---|---|
| 声明/签名/类型meta | package/module/logical owner、名义type、参数名/mode/result、字段有序shape、scope/候选集合、算法/profile/caps | body-only不得使纯签名查询失效；字段/候选/缺席名字变化只重验真实读取者；暂存overlay不能污染他工程 |
| 普通body推导/Checked单元 | 本体内容、期望类型、所有实际读callee签名、候选正负依赖、资源计划版本；语义与源坐标分开 | 未读callee body的普通caller不因body-only再推导；改签名/type/mode需重验读取者，不能因caller源未改就hit |
| 具体实例 | 模板logical id与签名/body版本、规范substitution/类型实际语义版本、约束/推导profile与caps、传递实例依赖 | 同实例共享checked实现；不同T隔离，owner/module不同不得合并；模板body变化使所有活实例及受影响产物正确换版，普通caller是否需重推导按它读的事实决定 |
| CT冻结/消费事实 | callee及helper真实body+签名传递闭包、实例、meta正负读取、实参、engine/MIR/ABC/codec/lane/target语义/caps/producer | helper/meta/type/caps/版本改而CT调用点不改也失效；非读者保持hit。CT值变更/类型依赖使owner推导/IR重验，不缓存旧Checked/PASS |

首次先检查CT实际请求callee/实例闭包，不能等全模块最终Checked后才启动决定类型的CT。合法普通递归先发布声明签名；实例检查InProgress可先用其已验证具体签名，CT依赖环则明确链（与AV递归fuel不同）。统一任务状态至少pending/in-progress/completed/failed与revision/cancel身份；失败/取消/资源/旧snapshot迟到不能发布为当前成功，single-flight也不能跨不相同完整key共用。CT miss必须反映真实运行时选择：普通`if`/`&&`/`||`的值计算由已checked AV程序决定，未执行分支trap潜伏；源码CT DAG只能对被语义请求的嵌套CT作调度，不能把不该执行的分支预先host求值。

CT执行语义key（ExecKey）与当前owner/source/node/期望类型的UseBinding分离；旧817 source_fp来源不能冒当前消费授权。ExecKey不能带整个模块source hash使非读者全miss，也不能漏实际callee展开后checked有效body及子CT/实例/meta传递版本。相同执行结果可复用，但各owner都要重新准入UseBinding，旧v1不足者换source lane schema而不改同key旧payload。

CT依赖可沿817 ClosureBodies保守颗粒：不承诺未执行callee分支里的helper body不失效，但不能把模块全部body或全module src哈希作为所有query语义键。meta候选集合的保守粒度必须声明，未读取该域的独立函数仍P0。来源坐标变化只刷新诊断/映射，不重新推导未改body；旧页若带span按新source身份重锚或受控拒，不能用旧spans。

### 10.5 函数级增量、恢复与发布

A交付SourceQuerySession与query页/root manifest；B将模板/具体实例/调用绑定接入同一事务；C将CT请求/envelope required边/owner事实接入。root原子发布包含source+CST/声明目录、完整依赖版本、语义单元/实例/CT与各目标关联。CAS中成功不可变页不等于current；失败事务留下的未root页须由旧租约/GC规则回收，不安装半候选或写PASS。RequiredData保确需载荷，Recipe/溯源边不强pin全部历史AST/AV工件；current/old/候选同时存活全计。

共享来源恢复消费929的已审稳定接口，**最终以D929或实际正式等价接缝闭合**。跨进程hit/restart先重验owner/source/grammar/decl/producer/schema/required闭包和嵌入ref，再恢复声明/事实与具体HIR；不序列化Checked位、union-find工作态、宿主地址/活AV。必要decode/deep-verify为R，重新parse/project/infer/instantiate/emit/lower是P，不得改记R/M。936若承诺恢复后CT实参编辑的旧helper单元生产/I/O0，session root需显式required已有checked/MIR/ABC页，恢复R窗口decode/deep-verify/有界复装（计真实读量，不是AV执行或source warmup）；缺页拒绝而非静默重编。新需求受影响的实例/执行单元单列真实生产，不能换名未改0；原固定负载0门不放宽。

新进程restore→立即有效body编辑与restore→首坏编辑→修复→第二有效编辑，无dummy/ensure_env/遍历body预热；未改函数/实例真实P=0及编辑时单元payload额外读写0，模块组装/link/manifest输出另列且进总预算。旧格式缺required不得静默cold，显式cold请求单列。

无变化/命中/历史回退需禁调真实producer反控；934普通caller未读helper body时保持infer0，935只改使用模板的一个call owner不会重新实例化别的T，936只改helper使相关CT重执行而无关函数全P0。失败attempt先计，不能早退默认0；恢复R不要求0 I/O，不能通过把真实compute藏在decode规避。

### 10.6 资源与正式时间

不增加未经批准的预算：语义query/实例共享796 hot8MiB+scratch8MiB、managed≤16MiB、单页512KiB与step/wall5s；CT沿817 hot8/staging16/AV8、managed≤32MiB、wire6MiB/input4MiB、query节点/引用深度128、单AV≤2s/整请求≤10s、worker private≤384MiB/tree≤512MiB。节点限制是CT调度原限额，不扩大为普通Auto语法新深度限制；新增实例按实际工作/内存限额受控资源拒绝，不加随意固定“最多N泛型”的语言上限。

CST/source/closures/各target仍沿929/原owner更严格的适用限额（语法64MiB档、A2X受管工作集≤48MiB等）；新query/实例/CT保持的页和解码瞬态须计入实际消费域，不能把16/32MiB当额外赠送或分域重复预留。先预留后clone/decode/encode/AV装载，所有错误出口/最后pin/close有界回收；多工程/并发相同key/取消旧代都测真实总峰值。整链可行性失败先needs_replan，不抬阈值。

时间沿正式release父连续窗口P90≤2000ms，原样本数量/规模/平台/ignore不减；796 gen796-1mi/v1的1MiB/30编辑、817 115fn≥256KiB/30 CT helper及meta编辑与≥32MiB页/512修订/GC原义务保持。新增实例与普通源组合选原规模的有限代表，逐状态明确生产/命中/恢复采样，不能用三份小例替正式样本。source/grammar/请求生成与序列化、实际等待/IPC、query/AV/冻结/回灌/父准入全入墙钟；冷准备单列，restore首操作不得从窗口外预热。真实工具hash/原始样本与分区/OS峰值可区分缺失、伪0、spike和放宽预算。

### 10.7 与928、929、931–933交接

| owner | 保留职责与本线边界 | 放行/最终条件 |
|---|---|---|
| 928 S | 九profile规则/AST/声明单元/sealed lambda body；本线不接管其在途树 | A可先读API/私有query，公共AST/grammar变更等待D928→929稳定释放；B/C新增能力不得并写auto-source.at/frontdoor/units |
| 929 S/H | 所有来源入口/首编辑恢复、重复parser退役与SYN父门；本线只增查询/实例/CT语义 | A全源编译接线、env/manifest恢复按已审符号SHA短窗；可提前交稳定query端口供929消费，最终A不能绕过D929真实来源恢复门；原929不被自动扩包或解除挂起 |
| 931 P/H | Rust临时资源/modes/缓存语义轴；本线只给checked单元/实例/CT常量 | 不写Rust lower/rt；若query输入影响A2X provider/证书，等稳定producer/key交窗重验，不复用旧错误片段 |
| 932 N/S/H | native合法暖准入/typed拒绝与914判面；本线不得恢复“一律ERROR”旧口径 | native/AV候选、source diagnostic、ac_mn/协议reader按其稳定SHA释放；本线新语言/资源结果维持原分类，必要consumer负控重验 |
| 933 P/H | sema跟踪来源与904独立manifest oracle；本线不能借新schema刷新原golden | 934原sema职责组需其跟踪fixture稳定提交消除隐形依赖；私有实现不等整包归档。原796schema值不因本线扩面改写，新增producer/page用新命名空间；最终原组红不可豁免 |

H含workspace Cargo/lock、driver/SDK、协议/父decoder、mod.rs/test-groups/来源登记，全文件单写。每窗owner/revision/base+handoff SHA、schema/所有producer-reader-construct-match、已验/STALE/pending/反控/释放逐项记录；不持写窗等慢认证，性能单实例错峰。当前各原计划进度、用户挂起及多会话WIP保持，本轮不启动任何实施。

本线触及共享AST/source会话的最终消费者验证还须保原866同PID安全点/在途旧链/最后lease及912/913闭包来源恢复原门；query/实例/CT root提交与现AV代际准入绑定，不能用restart/resume代同PID在线证据，不借新profile绕掉旧消费者。此处复用原机制，不新增完整AJ/OSR。


### 10.8 r2：权威事实、CT相位与恢复/交接的区分性出口

本次[934–936复核](../plans/reports/ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)把共同原则补成必需V例，保持原T/AC编号与0进度，不能用新文档代已验实现：

- **权威事实**：934实际worker四向对照为miss禁真实796 solver红、合法facts禁legacy类型综合仍绿、缺/错scope/callee/node事实由core实际拒且无fallback、hit双禁solver绿。完整fact-id消费/独立HIR断言与命名实参E01 value7/trace=b,a配对，单纯query日志或旁挂校验不够。
- **逻辑与有效实现**：935 InstanceId不因body/位置变，recipe/checked实现版本包含实际typed依赖；普通caller只读具体签名时不重推导/打印，target闭包仍切到新实现。首次需求、旧实例集合/last user/旧链lease、slot重映射与读集各自可观察，不预编全部T制造后续0。
- **跨层CT失效**：936强制`flag`内CT→`pick<T>`实现→外层CT例，flag true→false时模板/main源码不变，最终40→42；依赖是展开后checked实现和子CT版本，不只源码body hash。只有935页预留不能关闭这条链。
- **CT需求与运行分支**：被要求检查的owner/实例内显式HashExpr是编译期需求，外围runtime分支不跳过它；嵌套Hash先形成依赖。单个CT中普通pure函数/短路未执行的runtime trap潜伏，语法/类型错仍检查。CT闭包禁止能力在AV前审计，不能因普通宿主有trace/IO权限就透传。bool source lane只收Bool或已审严格0/1投影，任意非零转true不得照搬。多Hash/嵌套/等待共享一次累计deadline，不为子query重置预算。
- **安装与恢复角色**：backend-free query层先prepare语义candidate，消费宿主准备选中target、重验base/deps/caps/cancel再线性安装source/query/target当前记录；纯只读meta查询不构建target、不冒执行替换。929负责source/CST/decl，934 facts/deps，935实例，936 Exec/Use/执行页；这些required/R准入不能互相推给后继。恢复可R复装既有编码单元，不可新parse/infer/instantiate/lower/编码再标R；固定负载首CT实参编辑依旧旧helper/实例P与单元payload I/O0，缺页具名拒而非silent cold。
- **当前边界**：D928=6b429502f、D931=84c894b92已交付；931 print/2与D-931-1/3实际余账继承。929原恢复纪律不因D928满足自动解除，932已实施、933审计已有manifest独立来源；公共source-query/source/grammar/会话与H串行释放，935模板私有面与936 AV服务可分窗，不等于两个包可并写source-query/Cargo/decoder。原预算/P90/样本/分母保持。
