---
status: proposed
describes: target-state
created_at: 2026-10-16
updated_at: 2026-10-16
implemented_by_plans: []
---

# Design 42：语义元信息、类型推导、反射与编译期执行

用户要求：v0.6应规划完整meta信息及编译期/运行期反射；类型推导独立详细设计；元信息、类型、泛型实例化和AVM编译期结果从第一天纳入增量与全局cache。本文确立架构边界及后续实施要求，具体语法、Schema字段/编号、首批反射API与支持矩阵仍需正式Plan冻结；不声明已实现、不修改787执行合同或现行Spec。

关联：[Design39](39-ac-incremental-syntax-and-language-platform.md)、[Design40](40-compiler-artifacts-and-cache-lifecycle.md)、[Design41](41-interface-metadata-and-foreign-abi.md)、[HIR语义](strategy/auto-hir-design.md)、[后端战略](strategy/auto-native-backend-evolution.md)、[ABI RFC](strategy/auto-native-abi-rfc.md)、[AutoDown测试](35-autodown-test-cases.md)。

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

“单一事实来源”是共享语义查询合同和已提交不可变结果，不是唯一常驻HashMap或全局大锁。编译器、AVM、native/A2X、FFI、LSP分别消费其投影，不维护互相改写的权威类型副本。codegen可有局部寄存器/栈槽/layout缓存，但不能发现新语义或悄悄推导为Any。

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
                 ABC → AVM execution
                         ↓
             frozen CT value/type/diagnostic
                         ↕
            semantic queries continue / generated items
                         ↓
                 final Checked HIR
                    ↙          ↘
          native lowering     ABC backend
```

不能先要求整个程序final Checked，再执行决定类型的comptime；也不能把未检查工作态直接送VM。先检查所请求函数和可执行依赖，形成有明确profile的checked片段；结果回灌继续推导/实例化。普通运行时泛型替换未必调用VM，只有依赖编译期值/类型计算时才发起CT查询。

语义服务统一调度签名、body、实例与CT任务，记录依赖、等待/取消与发布状态。递归函数允许先收集显式签名再检查body；递归类型走组合同。comptime值/布局自依赖等无合法解的环报带查询链的错误；不能把所有环都当“再迭代直到好了”。有限单调分析才采用有终止条件的固定点，算法在专项Plan冻结。

不要求全量重写AVM后才实现comptime：先建立受限HIR→ABC，复用兼容解释器时重新验证数值/调用/资源语义；后端不再回读旧AST重推导。MIR可选，v0.6不依赖自有优化器。全部旧AVM/A2X迁移仍可分阶段，但支撑本版元编程的HIR→ABC子集不能一并延期。

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
| A05 | CT执行recipe/ABC身份、冻结结果、类型/生成项描述、输入依赖manifest | VmInterpreter堆、任意宿主句柄/地址 |
| A06 | 类型/签名/字段/方法/约束、body推导结果、泛型实例及反射/FFI接口投影 | 未解决替换表、推导器内部指针 |
| A07 | final HIR与消费的语义/资源事实引用 | 输入trusted标志 |
| A09/A10 | 描述数据/访问adapter及链接保留表、剥离策略指纹 | 把全部编译meta无条件塞进exe |
| A11 | 诊断、展开/推导/CT调用来源链、工具查询结果 | 迟到旧修订结果冒充当前 |
| A12 | 运行代际绑定/状态schema描述 | 运行中对象/可调用地址作为普通可淘汰cache |

Schema可映射Atom/Batom，热路径typed view、按声明/实例/组分块。大常量/类型图分块，热点解码缓存、索引、字符串池、活实例与求解暂存全部计入预算；一个语义服务不等于全仓全类型常驻。

### 7.1 key、依赖与命中

CT base key至少有callee实例/语义版本、常量与类型实参、profile/target/语言语义、HIR→ABC及AVM版本、允许能力和外部输入域。记录实际读取的函数实现、类型/字段/签名、解析候选域/反射集合、显式文件/环境值等完整依赖；上次依赖manifest先验证新鲜，验证本身经版本化查询进行，不能在命中后才发现漏读输入。动态控制流采用上次有效依赖集时，决定路径的条件及其输入也必须被覆盖。

“源码没变就命中”仅在所有依赖/配置也未变时成立。例：计算常量调用helper，helper body变就失效；计算sizeof(T)，字段或target/布局规则变就失效；列举字段生成代码，新增字段即使旧字段未变也失效；外部文件字节变而mtime不变仍失效。body-only修改是否传播看消费者是否读body，不用一刀切接口hash。

QueryKey是输入recipe，Artifact/内容身份是输出；重算结果未变可阻止无关下游继续失效。logical identity、输出内容、诊断snapshot各自处理；空白变更可复用不含来源的语义值，但来源映射/报错必须绑定当前修订，且反射读取source/doc时不能忽略相关变化。

### 7.2 确定性与结果物化

AVM comptime默认无任意文件/网络/时间/随机/FFI权限；有必要的宿主服务经能力broker提供快照输入及依赖记录。不可追踪/不可重放副作用明确拒绝可缓存语境，或标记为受限制的非缓存构建操作，不能取旧结果时悄悄省掉应发生的外部效果。

冻结结果只包含可移植标量/文本/聚合/类型引用及支持的常量图，保留别名/循环或明确拒绝不支持形态；不将指针/VM句柄直接写盘。原生lowering从冻结常量生成目标数据与重定位；宿主AVM和目标机器的表示不是同一层。

成功命中跳过VM执行，但校验payload身份、版本/引用、依赖新鲜和本次消费准入；缓存值不是Checked HIR凭证。需要恢复HIR时仍经verify。坏块受控miss/重算，缺失再生输入则明确unavailable，不能把错误值变成0/Nil。

### 7.3 生命周期

依赖指纹用于失效，不默认强pin全部body/旧AST。若保留的常量结果消费时仍需其类型描述/数据页，这些是required-data；若编译进native后无需历史AVM执行图，则只留recipe/溯源，不永久pin解释器工件。runtime活代际单独lease，不由普通LRU强制卸载。

相同完整key跨工程共享结果和不可变meta；overlay、配置、当前诊断和lease属于会话。两个producer同key需single-flight/冲突检测；结果与依赖manifest原子发布。增量元编程不能成为绕过Design40自动预算/回收的第二套缓存。

## 8. v0.6交付与实施拆分

用户确认v0.6要规划完整meta与编译期/runtime反射；这应改变长期架构而非仅加库函数。建议先冻结支持矩阵，要求每个已承诺语言形态有完整meta，分别列读meta、计算类型、生成声明、runtime保留/访问/调用的状态；不把“有TypeInfo结构”当作全部交付。

建议工作包（未分配Plan、非实施授权）：

1. 语义身份/Schema/查询、类型推导规则和代表性递归/泛型/反射探针；冻结完整覆盖矩阵与阶段准入。
2. HIR→ABC与受控AVM、有限实例化/CT查询回路；从首例持久结果/依赖、资源上限与AutoDown用例，不等语言完整后补cache。
3. 编译期反射及runtime描述投影/保留、链接/装载及代际兼容；与有限AutoABI/FFI合同共享类型身份，不让ABI或Codegen反过来拥有语义事实。
4. 逐项语言/元编程扩面与旧consumer迁移；完整旧AVM/A2X迁移和Typed MIR/优化按既定分期，不用本稿宣称全部已纳入某一在途Plan。

787仍只负责语法/分页，其partial CST不授类型或CT执行信任。AutoDown接线/ABI优先工作继续推进，同时上述语义合同须在泛型、元编程或新HIR→ABC开始实施前形成正式可执行Plan。

## 9. 必须落地的区分性验证

| ID | 输入/变更 | 必须观察的结果 |
|---|---|---|
| M-01 | 同名不同模块类型、递归类型、重启后intern顺序不同 | 名义身份不混淆，持久ref重映射正确，递归图有界验证 |
| M-02 | 调用者不变，只改helper body/类型字段/target/VM版本 | 相关CT与实例化失效；未依赖查询仍可命中；结果与fresh一致 |
| M-03 | AST/依赖不变的第二构建及进程重启 | 真AVM执行计数0，冻结结果及native观察与fresh一致，不伪计数 |
| M-04 | 字段枚举新增成员、方法候选/导入新增、原不存在名字被定义 | 负依赖/集合依赖使旧结果失效，不误复用 |
| M-05 | 未解决类型、矛盾约束、歧义、递归CT/实例化环 | 定位/依赖链错误；无Any兜底、无无限求解；合法递归兄弟通过 |
| M-06 | type-dependent CT与跨宿主/target宽度 | 已检查片段可执行，目标类型/布局正确，无宿主地址/宽度泄漏 |
| M-07 | CT超步数/内存/取消、未授权IO、受控输入变更 | infra/权限/资源状态独立；不永久缓存非法结论；可追踪输入改变失效 |
| M-08 | 有效frame替换错误key、坏类型ref、CT结果到HIR | 真实消费入口拒绝；不能以frame自校验或持久trusted绕过 |
| M-09 | runtime none/name/shape/invoke策略与动态查找 | 保留承诺明确、无隐式全依赖保留；访问/调用能力有门控，构建策略入key |
| M-10 | 热重载布局变化，旧对象/callback/deleter仍活 | 旧描述/代码保活，新旧TypeId不串代，释放后可自动回收 |
| M-11 | 大类型/实例集合仅查一个签名，持续编辑/回收 | 页/索引/求解暂存有预算；不会每进程复制全类型库，空间长期收敛 |
| M-12 | 只改来源位置，body值相同；反射读取doc/source | 普通值可共享而来源更新；读取doc/source时相应失效，诊断链准确 |
| M-13 | meta查询/推导/实例化/CT各例及其端到端调用链，冷热/变更/压力分场景 | 每阶段从首例有真实span/计数/内存I/O，依赖不变CT命中时AVM执行0且miss正控实际执行；代表负载预算与端到端亚秒目标关联，等待/嵌套查询不漏算或重算 |

各实施Plan须将适用项绑定实际入口/owner/正反例/资源负载和预期值；上表是设计验收要求，不是已执行测试。现有旧CTEE字符串往返/截断/数组退Nil不作为兼容正确性oracle。

## 10. 待冻结决定

首版反射语法/API与可见性、type-as-value支持、泛型/泛化规则、生成项卫生、递归求解允许域、target布局查询、冻结常量图类型、runtime默认保留/剥离和动态查找域、CT能力broker、资源预算、来源/隐私政策、Schema编码和版本。用户已确定方向与v0.6规划要求，以上技术细节仍需探针/Plan定界，不把草案编号当作实现或正式ABI发布。
