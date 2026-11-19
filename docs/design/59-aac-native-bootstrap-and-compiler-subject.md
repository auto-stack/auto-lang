# Design 59：AAC 编译器主体、需求审计与原生自举

G-AAC 已于 2026-11-17 取得 985/T-A05 正式 M2A-08（95c7fbad8），AAC 下一入口为 1044/T-01。

> 2026-11-16 · proposed r4（规划复核）。用户授权本轮正式设计、需求审计与拆计划；**仅规划，未施工**。AAC 在 M2-A 稳定出口后可与 M2-B 并行实施，独立验收，不作为 M2-A、M2-B 或 v0.6 发布硬门。交付仍属独立后续路线（沿用 v0.7 归属），不必等 M2-B 全部产品迁完才开工。[执行路线](strategy/aac-delivery-roadmap.md)。

本条是新的排程决定，替代旧 [ACC 契约](strategy/auto-acc-bootstrap-contract.md)及 [2026-10-24 分版记录](../plans/reports/ace-milestones-version-boundary-2026-10-24.md)中“必须全 M2 产品迁移完成才启动 AAC”的时序；历史已交能力、阶段安全约束不变。当前只批准规划，不代表未来代码施工授权。

## 0. M2-B期间AAC实证与AC渐进补齐（当前）

### M2-A基础可用、M2-B三端对齐与AAC实证（2026-11-16当前优先）

用户最新裁定：**M2最终AV/AC/A2R公共能力一致；A2C完整对齐为增强项。** M2-A仍须完整AV/A2R语言/库及原required真实验证，AC至少有能承载AAC启动的真实多模块/类型与owned数据、泛型/Option/Result基本链、所需CTE/FFI、文本/分配/同步文件等基础。AC其余具名差额允许移M2-B：原行ID/输入/oracle/错误及资源义务、实际来源、B-owner/关闭条件必须保留，不能记PASS/NA或从分母删除。仅“原native不支持/旧profile外”不再是M2最终豁免。

[1067](../plans/archive/1067-ace-ac-native-async-and-task-parity.md)补原生异步/Future/Task；[1068](../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)收全部三端语言/库/FFI/领域差额，并以AAC实际模块持续实证。985/A不等待两包，985/B及最终986/987必须收三端完整闭合；A2C现有代码/用例/支持记录保留、缺完整对齐不阻M2。A2PY既有标准库义务与AV三provider、Linux1063延期规则不变。

AAC在A出口后、M2-B期间推进，以真实Auto编译器模块检验AC语言/库；某任务所需能力未交只阻该任务/切片，缺口回语言/库真实owner补齐。AAC未使用的语言/领域/异步特性仍补独立三端用例。完整Gen1→Gen2→Gen3自举签章保持独立，不强加为M2通过前置；其所暴露的M2三端能力缺口必须在M2最终前关闭。此规则优先于旧“AC所有义务必须A期全齐”“M2-B与AAC完全无能力交接”“原A2C全量同等硬门”等表述。

## 1. 名称、交付范围与成功条件

AAC = 用 Auto 实现 ACE 的 **AC 编译器主体**，旧称 ACC。AACE 是更大的 Auto 版 ACE 范围，AAV 是解释执行器子线；本路线不承担 AAV、A2R/A2C/A2PY emitter 全面重写，不接管 AutoOS/App 迁移，不要求自研机器码后端。

编译器主体必须由 Auto 完成：源码/语法处理与模块装配、名字与类型语义、实例化及元编程编排、受检 HIR 构造/校验、资源与效果分析、共享 Typed MIR 生成、增量依赖与缓存策略、编译驱动。不能由 Auto CLI 调 Rust 的源代码编译入口后声称主体完成。最终支持面取开工时冻结的 **M2-A 中适用于 AC 的完整 Auto 语言、标准库调用及编译期能力**；语法完整含 Schema/AutoUI/AGE，渲染和领域运行时仍由原 owner 提供，必须通过真实受检宿主接缝验证，不能用忽略语法或无效果 stub 消除义务。

两个轴分别计量，不能互相替代：

| 轴 | 冻结内容 | 可声明的结论 |
|---|---|---|
| CS：compiler-source-demand | AAC 自身源代码实际使用的语言、stdlib 操作和资源 profile | Rust AC 能编译这一份 AAC 源码；不证明 AAC 支持完整语言 |
| CL：compiled-language-support | AAC 对 M2-A AC 适用语言/库/领域接缝的逐项支持矩阵 | AAC 可编译对应用户程序；不能仅凭自编译或原 Rust 测试通过认领 |

最终 AAC 需同时关闭主体迁移、CL 完整性及 Gen1→Gen2→Gen3 代际验收。中间 scalar/前端/Gen1 切片有各自计划验收，但不得签整个 AAC 完成。对 M2-A 正式确定为非 AC 的能力保留具名适用性及规范理由；不得自行把难项改为 NA。

## 2. 现状取证与范围去留

本轮为只读静态审计；观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`，含工作区当时的未提交规划。实际实现以 `experimental/ace/Cargo.toml` 独立 workspace 为准。`docs/specs/overview.md` 总览较早，相关 [AC Spec](../specs/auto-ac/project.md)、[native 退役合同](../specs/auto-ac/native-backend-and-retirement.md)、[HIR 阶段合同](../specs/auto-hir/stage-contract.md)与源码共同取证；模块首部的早期 scalar 注释不能覆盖后续已扩面的真实接口。PLAN-743 历史库存和旧 `auto/` AAVM/AA2R 不当作当前 AAC 可用实现。

| 当前实际所有权/证据路径（均相对仓根） | AAC 主体范围 | 可留在 Rust/外部的边界 |
|---|---|---|
| `experimental/ace/crates/syntax-runtime/src/{lexer,parse,cst,edit}/`、`syntax-grammar/src/lib.rs`，core 的 `source/cst/` 与 `source/parser.rs` | Auto 语法执行、CST→AST 投影、span/edit 与模块语法处理；复用同一 grammar 定义与身份，不另发明一套语言 | 冻结 grammar 数据生成工具可留 Rust；最终不得委托 Rust parser 完成用户源码解析 |
| `core/src/source/{ast,resolve,typecheck,adapter,modules,packages,decl_env}.rs`、`descriptor`、`hir.rs`、`verify.rs` | 名字绑定、类型/受检 HIR、模块/包与公共语义准入规则 | 原 Rust 实现留作 reference；生产 AAC 路径不得静默回落 |
| `semantics/src/{infer,input,query,reflect,budget}.rs`、`source-query/src/{produce,apply,facts,instances,resources}.rs` | 推导算法、实例化、反射投影、语义事实及资源事实生成 | 统一 schema/算法版本合同共享；不能把 Rust solver 作为隐藏服务保留主体 |
| `source-cte/src/{schedule,service,executor,envelope,store}.rs`、`comptime/src/{driver,av_driver}.rs` | CT 需求发现、依赖/预算/实例编排、冻结结果和失效策略 | AV 执行器及 ABC/MIR 执行准入可留 Rust；服务仅消费已定型执行单元，不收源码代做语义 |
| `lowering/src/hir_adapter.rs`；`av-core/src/{mir,mir_verify,ownership,place}.rs` | HIR→Typed MIR、控制流/布局语义、资源 drop/borrow/effect 保持；Auto 本地准入 | Rust MIR verifier 可作为机器码/AV 服务的二次安全边界，不代替 Auto 源码语义权威；ABI/layout 的机器相关映射可留后端 |
| `source-query/src/{host,session,store,restore,ident,keys}.rs`、`artifacts/src/identity.rs`、`artifacts/src/codec.rs`与`artifacts/src/store/` | 查询、正负依赖、缓存键/恢复准入、事务发布和驱动策略 | 底层文件/进程/原子 OS 原语由新版标准库提供，不用 Rust ManagedStore 隐藏整个增量主体 |
| `native/src/{backend,lower,emit,layout,foreign_lower,typed_pipeline,typed_session,driver,link_driver}.rs`、`bin/ac-native.rs`、`core/src/link.rs` | Auto 编译驱动、对象/链接编排和身份；审计拆出其中仍含源码语义/资源规则的部分，不能整 crate 豁免 | Cranelift、对象格式 writer、机器后端、平台 linker 及显式 runtime/FFI helper 可保留 |

量级取样（`src/**/*.rs`，含文件中的测试，不是纯生产 LOC 或工期估计）：core 37 文件/68,977 行，source-query 28/16,965，semantics 7/5,002，lowering 2/6,354，syntax-runtime 14/6,547，source-cte 9/5,446，native 45/26,849。这个体量要求分层迁移和独立批次，不能把“整个 core 移植”写成单个任务。

本次内容指纹：`core/src/hir.rs` SHA256 `0176789ebd78eb7a559bd8b5a00c6d58dc40c4a8f5130be391bdf29084467b5e`；`core/src/verify.rs` `cba93ead10ab8ab01ddbdd91e615453870d2fae7b851666c110fc5348baf3632`；`source-cte/src/lib.rs` `60f92ecf48b6d79dbdc0efd6410076545b46aa89ce8b10d7be5652091bc5ab85`。开工需重新绑定冻结 M2-A，而非沿用此观察快照冒稳定证书。

## 3. 语言与标准库需求审计

下表的 CS 为本轮基于真实 Rust 数据流推导的 **AAC 源码设计需求**，不是已经存在的 Auto 源码扫描结果。1044 在 M2-A 后把它收敛为实际 `module → language feature → stdlib operation → owner → receipt`，不机械逐 API 翻译 Rust。只要求实现算法所需能力，不要求复制 Arc/Mutex、Rust trait 或 serde 宏设计。

| 需求 ID | CS 所需能力与依据 | M2 供给 owner / 具体缺口与验法 |
|---|---|---|
| D-L01 | record/enum/递归节点、match、函数/方法、局部可变、精确整数/位运算；AST/HIR/MIR 图与 ID 表 | 962/965/972；962 已归档只是其有限合同，CS 源码普通 AC 编译及非法类型拒绝仍需逐行证据 |
| D-L02 | 泛型容器/函数与跨模块实例化、可观测错误传播；infer 的工作表、事实表/集合 | 963/965/946；963 已归档不等于所有 AAC 容器组合通过。1047/1048 用相同参数顺序/实例闭包反例验 |
| D-L03 | 模块/namespace/use/pub、编译期平台选择与类型/语法元数据；syntax-grammar、packages、CT schedule | 960/966/1043；1043 已归档的新装配是基础，不证明完整 CT/元编程或自举源码可编译 |
| D-L04 | 所有权/借用/资源终结与 FFI 句柄；失败/早退的清理 | 962/965/967/972；错误/取消不双 drop、不漏最后 owner。Rust RAII 不能直接按语法翻译成未审计 Auto 生命周期 |
| D-L05 | CS 默认单线程确定性编译；受限递归/遍历预算、显式错误和取消 | 965/968/972；不强迫 CS 用 async/线程/全 runtime 反射。CL 中 dynamic/闭包/async/调度/反射仍由1048/1049完整支持，不据 CS 未使用豁免 |
| D-S01 | 长度字符串/bytes/UTF-8、切片/span、文本构造、intern、诊断格式 | 945/972/1041；945 已归档有限数学/文本，仍需真实 Unicode 文件名/非法编码/嵌零及 span 边界验证 |
| D-S02 | Vec/Map/Set、arena/索引图、确定性排序、精确数值与溢出 | 946/972；BTreeMap/工作队列等需自有算法或等价结构，稳定顺序不能依随机 hash 偶然成立 |
| D-S03 | allocator、arena 释放、预算统计、深图遍历；可分配资源 profile | 972/1039；固定 arena vs 通用 allocator由1044有界评估。AAC 是 hosted+alloc 工具，不据此强迫 MCU profile 携带这些模块 |
| D-S04 | File/Path/目录、环境/目标信息、完整读写、原子发布、进程/管道/退出码/取消回收与链接器发现 | 973/1039/1040/1041/978；Windows 与 Linux 的实际 owner 供给，超时和重命名失败保旧。先验 Windows x64 host，Linux 单列待所需已交底座和真实 runner |
| D-S05 | 精确BLAKE3/实际所需SHA-256、域/长度帧、稳定codec/manifest及恢复校验 | 972的hash操作、975的通用数据API与978不能自动证明精确工件协议齐；新增1056补所需字节算法/应用codec并复用统一Auto库单源。1044/1052另冻物理store格式，保既有逻辑算法/版本，变更须升版和旧缓存拒绝 |
| D-S06 | Typed MIR/ABC/backend 请求编码、外部对象/链接器、CTE 调用、FFI 错误/句柄协议 | 967/978/ABI owner，1044/1051只补 AAC 服务接缝；不重复建设通用 Rust/C 自动 FFI 或另造标准库 |

**r2补齐的源码需求**：D-L06为高级函数值/闭包或等价显式回调接口、递归图/索引/别名资源语义及错误传播，用于LowerCt工厂和统一语义工作表，由1044核实际CS、1047/1048及1055提供；D-L07为所有实际数值/字面量的位精度、parse/format和locale无关性，D-S07为CLI argv/env/cwd、诊断bytes→UTF-8/UTF-16位置及CRLF/emoji规则，D-S08为typed foreign规范化描述/句柄/错误/异步callback与binding/shim构建闭包；D-S09为现有AC canonical Atom/HIR文本数据reader/writer及公开输入/产物合同，1056处理纯codec、1047语义准入、1052驱动、1053实际兼容。已有语言级规范化/常量处理/pass随主体迁移，版本/效果/资源重验完整；机器优化可留Cranelift，不强加新优化器。1045/1056验库，1046验来源，1047/1049验声明及资源，1051/1052验执行与构建。无需模仿Rust trait/宏/Arc，CS若选择替代算法需用真实依赖扫描说明能力增减；CL不能据此删行。

供给审计快照（2026-11-14）：945/947/962/963/1042/1043 已归档；946/948/964–970/972–975/978/985 与 Linux1040等仍有未交义务。**标准库已设计≠已实现，单项 archived≠M2-A稳定**。本轮没有编译 AAC、运行能力探针或签稳定出口。

CS 的标准库场景分层为 `core`（纯算法/文本）+`alloc`（容器/arena）+`hosted compiler`（文件/进程/平台）+可选 backend/AV adapter。默认走 Auto 自有 ABI/标准库与 OS 适配，不以 libc 为默认实现，不把 Auto 字符串变成 NUL 终止字符串；进入外部接口的布局、调用约定、长度/编码/终止符由显式 adapter 处理。后端/链接工具或 Rust AV 自身的 libc/CRT 依赖单列，不借此声称整工具链无 libc。

完整 CL 分母从最终M2 AV/AC/A2R required矩阵与A后延增量并集导入，含标准库逐操作、泛型/CT、闭包/dynamic/async/资源、FFI/模块/领域语法接缝；CS 表只是其实际源码需求子集。新增 M2-A 之后的语言版本不自动扩成无限承诺，按基线升级和独立后继管理。

### CL语言族完整性与实际补齐分工（2026-11-16检查后当前）

AAC计划同时承担两件不同的事：用Auto实现编译器主体；使该编译器能识别、检查、编译本版完整Auto语言。旧“从M2-A AC适用子集导入”不得成为最终支持上限。CL从最终M2的AV/AC/A2R required语言/库/领域矩阵及A后延增量并集取分母；CS只是AAC源码实际使用子集。下表是1044原CL矩阵的族级映射，不是新registry、另一个测试框架或扩大到已后置Godot/JET/ARK等目标。

| CL能力族 | AAC语义生产与执行任务 | 现AC/runtime或库缺口归属 | 必须证明 |
|---|---|---|---|
| 模块/名称/数值/聚合/模式 | 1046/T-02..05；1047/T-02..04/T-09；1050/T-03/04 | 1046/1047/1050/1051 对应任务 | 普通多模块源、宽整数/位精度、递归数据与路径/span，合法执行和错误阶段/资源各有独立oracle |
| ?T、!T与组合 | 1046投影；1047类型；1049/T-02..05；1050/T-03/04 | 1049/1050/1051 对应任务 | Option/Result不可擦成T，缺失和失败分开；默认/传播/匹配/泛型返回/集合/跨模块及清理 |
| ~T、await/go、Task、~Iter | 1046投影；1049/T-03/04/T-09；1050/T-03/09 | 1067 已交原生调度；1051 补执行余量；1068 联合回验 | 被编译程序真AC运行挂起/恢复/取消/背压/close；AAC自身单线程不是排除理由 |
| 泛型/spec/ext/实例化 | 1048/T-02；1047签名/接口；1050布局与调用 | 1048/T-02 共享接面、1050/1051 按实际降级与执行 | 类型/值参数、约束、方法、跨模块与嵌套owned容器；不能只编AAC源码出现的几个具体实例 |
| CTE/宏/#in/metadata | 1048/T-03..05/T-09，1055基础LowerCt→1050完整LowerCt，1051执行尾 | 1048/T-03/04/09、1050/T-09、1051/T-04 | 真实求值→生成声明→重绑定/verify，target条件、hygiene、读集/预算/权限与失败恢复 |
| 标注及其语义消费者 | 1046保源和声明/字段绑定；1047/T-09、1048/T-03/T-09、1049/T-09 | 1048 共享语义及 971/967/980/982 等实际消费者 | 本版required的test/api/FFI等标注元数据有实际效果，参数/作用域/阶段按规范；仅parse保字符串不算支持 |
| dynamic/反射/闭包/控制/资源 | 1049/T-02..05/T-09；1050/T-03/04/09 | 1049 源语义、1050 MIR、1051 机器执行 | 捕获/函数值/分支与循环合流、失败/回调/挂起的borrow/move/drop规则，非法近邻运行前拒绝 |
| 普通库/FFI/领域Node | 1045/1056供CS客户端，1047/1049核语义，1050映射，1051接机器/AV服务 | 1045、1051、1052、标准库/ABI 原生产者及 1097 图形 AC | CL全部公共操作仍逐行验证，CS只使用一部分不删行；真实OS/协议/ABI/Node宿主回调，不借隐含libc/旧VM |
| 完整编译运行与各代复验 | 1053/T-03..05/T-09；1054/T-03/T-09 | 1068向985/B交三端，AAC完整签章独立 | Gen1编译CL到实际AC产物并运行；Gen1/2/3各验全CL，不以自编译成功、代表例或Rust reference同错代替 |

标注只要求本版已确认required的机制和消费者，未裁定的未来标注不自动加入。AAC优先完成AC输出及显式AV CT服务；不要求它重写所有A2X emitter。AV/A2R的正式producer可作三端对照，每端仍对独立expected。

每个CL行除原字段外明确“语法投影→Auto类型/语义事实→Auto MIR→机器/runtime→实际consumer”及相应任务/源码/独立oracle。阶段driver必须由该包首次使用的work任务建立，在1053前就验证Auto算法；Gen0编译driver合法，driver处理被测源码时不得再调用Gen0源码编译器来冒AAC。早期1055有限LowerCt用于打通依赖环，完整CL由1050/1053最终关闭，不把早期子集扩大为假全量。

修复必须回实际生产者：AAC前端算法差额由1046–1050等原任务实现；现 Rust AC 机器/运行时、OS/库和产品接面按下表归属，由 1068 核正式证据。只写“依赖其他owner”不能关闭CL行，必须有具名source/版本/结果；只有A/早期阶段允许有明确后续去向，完整Gen1/代际签章不允许required缺项。

### 现有 AC 差额的实施归属

2026-11-19。AAC 阶段任务同时负责其共享接面上的现有 Gen0 AC 修复。每个生产任务管理实现、独立正反例、复审和正式交付。1068 冻结最终验收分母，按 Case/source/实际产物身份收正式证据、跑最终 AV/AC/A2R 和 AAC 实证。失败退回原生产任务。AAC 的 CS 需求可以限定早期交付，M2 最终 CL 保全部 required。

| 现有 AC 差额 | 实施和正式证据来源 |
|---|---|
| 框架及原后延账 | [1044](../plans/1044-aac-baseline-and-stage-contracts.md) T-01 起固定正式 M2A-08、CS/CL、任务和逐行独立预期。 |
| 语法、模块、共享声明、全局初值 | [1046](../plans/1046-aac-syntax-source-and-modules.md) T-02/04；[1047](../plans/1047-aac-binding-types-and-checked-hir.md) T-02/04/09 交共享绑定/HIR/标注签名；否则正确的 global storage 仍由 1051 验。 |
| 泛型/spec/ext、CTE 和 required 标注 | [1048](../plans/1048-aac-generics-meta-and-cte.md) T-02/03/04/09 负责共享 query/实例化、meta 语义、CT 执行编排、生成与重绑定。1055 先交有界 LowerCt，1050 接高级 LowerCt，1051/967 和实际标注消费者各交执行证据。 |
| ~T/?T/!T、动态/对象、闭包、控制和资源 | [1049](../plans/1049-aac-control-dynamic-and-resource-semantics.md) T-02/03/04/09 修共享语义与各类合法/非法组合。 |
| 共享 HIR→MIR | [1050](../plans/1050-aac-shared-typed-mir-lowering.md) T-03/04/05/09 交 global/dynamic/object、调用/调度/控制流、资源清理、错误阶段和源位置，补现 AC 所经共享构建层并复跑同源机器程序。 |
| MIR→现 AC 机器码、runtime、网络/FFI | [1051](../plans/1051-aac-native-backend-and-cte-bridges.md) T-03/04/05/09 交真正的 native lower、值/全局/对象、调度、平台与 ABI，复用 1067/967 等已交接面。补 owned-text、外部取消、ask/reply 和 platform+sched；设备和库的公共协议/实现归其原生产者。1051 用现 Gen0 MIR 先独立交机器服务，1050 不反卡 1051。 |
| AAC 启动库、SDK、真实工程和配置 | [1045](../plans/1045-aac-hosted-stdlib-and-sdk.md) T-02–05/T-09 与 1056 交启动平台/库/SDK；[1052](../plans/1052-aac-incremental-driver-and-artifacts.md) T-02–05/T-09 交实际驱动、缓存、恢复和包闭包，按需接 982/B-CONFIG 正式配置。启动必需薄层在 1045 或实际先行库交，不能等 1051/1052。 |
| AutoUI/AGE 的 AC 领域实现 | [1097](../plans/1097-ace-ac-autoui-w2-and-age-host.md) 独立交 AC 宿主、W2 和 AGE 的正式 AC 域证据，1074 用 W2 继续 A2R。 |
| 联合验收 | [1068](../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md) 收上述生产方已交证据及其阶段 AAC 模块，最终三端按独立预期验证并交 985/B。1053 全 CL 与 1054 代际签章沿 AAC 路线独立完成。 |



### AC 复用 AV 的 AutoDown 语言语料

2026-11-19阶段补充：AV 1078–1096仅验AV，AC/A2R/AAC等共享检查可先#ignore；这些检查不齐或失败不得回卡AV计划。对应AAC/AC任务开发时逐check移除本阶段标记、保原source/oracle/条件并真实运行。AAC自己的CL/代际完整门、1068/985最终三端门仍不接受忽略充PASS。

公共作者权威按[按特性共享语料合同](testing/ace-feature-organized-autodown-corpus.md)统一为`crates/auto-lang/test/cases/language/`。同一Case维护一份Auto/工程闭包，AV、AC、A2R与AAC各代独立绑定实际编译器/目标检查、oracle与结果；不再按AV、native或AAC代际复制CL源码。源位置/错误/资源观察分别验，正例AC/AAC真编译并运行，Rust emit/reference不能代run。

本规则约束1046–1052等阶段语言CL生产与1053/1054汇总；AAC自身CS主体/中间IR专项、宿主/代际构建链保原owner，不为目录统一重写编译器或改AAC独立验收地位。已交语义Case随对应任务立即复用；未建立的语言行保pending与实际producer，按特性交付逐步迁入/追加checks，不人为等1078–1096整条路线结束。最终1096证书和1068/985的全量对账保持。

原case/check/角色在1078建立的迁移数据中映射到canonical特性Case，旧bug/条件/原oracle保可追溯；源/模式/依赖不等价不合并。各代保source/grammar/工具/SDK/provider/产物身份，不能用前代结果充本代或回填expected；缺required行、只emit或尚未运行仍阻对应完整签章。1078拟交IF-ACE-FEATURE-CORPUS/1，接口或目标adapter未交时只阻需要它的切片，不反卡其它已可行能力。

## 4. 架构与可信边界

拟新增 `experimental/ace/aac/` 为 Auto 主体根，按 `syntax/source/semantics/hir/cte/mir/store/driver` 分模块；服务适配器拟置 `experimental/ace/crates/aac-bridge/`。这是未来路径，当前不存在也不在本轮建立。正式模块接口、grammar/schema 复用点由1044冻结；最终不在 `auto/` 旧实验里堆第二套编译器。

生产链：普通 `.at` → Auto lexer/parser/CST → Auto AST/模块 → Auto resolve/infer/实例/CT编排 → Auto checked HIR → Auto Typed MIR lowering+准入 → 显式 backend 服务的 MIR 复验 → Cranelift/对象 → Auto 驱动链接 → native 程序。CT 执行从 Auto 已定型请求进入 Rust AV，执行结果/读集/资源凭证经 Auto 验证再进入语义事实。

| 接缝 | 所有权与非法输入处理 | 关闭条件 |
|---|---|---|
| 语法/语义/HIR | Auto 为 AAC 本链唯一源码语义权威；Rust 参考独立跑，不做 fallback | 拔掉 Rust parse/resolve/infer/源码 compile 服务后，AAC 正例仍成功；故意非法源仍在对应前端阶段拒绝 |
| Typed MIR → 后端 | 序列化对象不是可信 Checked；Rust 先验版本/形状/类型/布局/资源及目标 ABI，重新产生其进程内 capability，禁止反序列化 checked 标志绕验 | 篡改 ownership/DefRef/布局/版本失败且不调用 codegen；合法近邻产实际对象，不能因安全复验隐藏整套源语义 |
| CT → AV | Auto 决定 CT/实例/读集；Rust 仅运行受检可执行体及明确 host imports | 禁止传入源文本/AST让服务重绑定；CT timeout/禁用 IO/取消不能冒普通语言错误或发布冻结值 |
| 增量/恢复 | Auto 持有语义依赖和发布决策；hash/schema/target/stdlib/grammar/profile/pass/桥身份分轴 | 冷/热/恢复同 oracle；失败 edit 不发布半新集合，坏缓存不升级为 Checked |
| FFI/OS/link | 句柄归属、调用约定、字符串编码/长度、进程树/超时由显式层负责 | 受检 ABI 真调用与清理证明；不可用 dependency 明确缺失，不退回旧 AutoVM/A2X |

桥接采用 **优先评估有界进程协议，C ABI 薄 shim 为备选** 的决策次序，当前不把现有 `mn-worker` 当作可直接接 AAC 的完整协议。1044/T-03静态冻结候选请求/信任模型；1051/T-02仅在开工门之后做有限可行性验证：同一标量与 owned MIR 经两种候选的适用入口核对身份/错误/清理，以是否满足安全边界及实测开销选择并记录一个生产 transport。失败要回到该计划设计，不让调用方自行复制 lowering/solver。传输选择未定不影响源码需求审计；影响1051后续实现和1053集成，必须先决策。

不要求同时写两套所有机器后端或重建 AV；复用 grammar 数据、IR schema、独立 oracle、runtime 并不意味着放弃 Auto 主体。共享合同变化仍由原 owner 单写、版本化和补其必要消费者。

## 5. 开工门、并行与跨仓依赖

所有实施包共有门 **G-AAC**：

1. Design55 M2A-01..08 关闭，PLAN-985 的 T-A04 独立复核与 T-A05 已合入来源收据成立。只消费 **985/A**，不依赖整个985/B、984产品全迁、986安装或987退役。
2. 1044绑定A实际基线和完整三端CL目标，并将CS所需能力分为已交可用与待补具名差额；不得因AC尚未最终齐平阻整个AAC开工。任务只消费其实际所需的已交能力，依赖缺口仅阻对应切片，由下列实施任务或实际库 owner 交付；CL完整验收仍需所有相关后延行真实关闭。
3. 执行时有实施授权；本轮“先不施工”持续有效，当前无 probe/编译/新测试实现/工作树。

“稳定”不额外发明等 N 天或 N 次全量回归：以原 M2-A 正式出口、已合入身份及缺陷/known-debt 准入为准。若先前 A 收据受共享修改影响，具名失效/重验后才能认领对应 AAC 证据。

| 仓/owner | 输入→AAC | 任务/交接与并行纪律 |
|---|---|---|
| auto-lang M2能力/stdlib/构建 owner | 语言/库/ABI/grammar/source SDK/测试规范与985/A收据 | 1044登记版本和具名生产者；1045消费，缺口回原 owner 活动计划或新后继，不重开归档945/947/963/1043，不让 AAC 偷建第二stdlib |
| auto-lang backend/AV/artifacts | shared MIR、安全 verifier、codegen/link/runtime及协议 | 1044冻结合同，1051实现薄adapter；1050/1052自有主体。共享目录单写，M2-B真实修复优先按owner交接；AAC私有新路径可并行 |
| auto-down/AutoDown/TestCase owner（现有语言/外仓入口） | 复用版本化 TestCase/AutoDown 与报告格式 | 1053/1054消费固定版本；若真缺表达力，记录具体字段/原oracle及 owner 后继再接入，不默认改协议、另建全仓测试框架或新增外仓未授权代码任务 |
| auto-man/auto-ui/auto-web/auto-os/应用 | M2-A具名真实验证及 M2-B兼容约束 | AAC仅消费已交稳定SDK/域接口和最小领域执行 oracle；不要求各仓改默认/迁AAC。M2-B不等完整AAC代际签章，但必须收1068三端闭合及其AAC实证；AAC不反等完整产品验收 |

同仓各Plan一棵专用树；将来跨仓只有确需修改的仓加入同组 sibling树，不设链接。读源解析按 env→同组 sibling→主检出。冻结基线是可复现输入而不是禁止共享owner继续修复；更新走影响集重验，不能拿旧SHA证明新源。

## 6. 分批路线与出口

| 批次 | Plan | 出口/可独立声明 |
|---|---|---|
| R0 基线与契约 | 1044 | G-AAC核对、CS/CL全量有owner、数据/语法/桥契约及预算冻结；不声明编译器移植 |
| R1a 摘要/codec | 1056 | 精确摘要/分帧/codec和统一库来源，公开新增操作保现覆盖合同；不依赖AAC主体/后端服务 |
| R1b hosted支撑 | 1045 | CS最小库/SDK真实AC使用，OS/工具链/运行依赖账明确；不以外部helper覆盖标准库缺口 |
| R2 前端 | 1046 | Auto解析与模块/CST/span，独立语法正反例；不依Rust parser |
| R3 核心语义 | 1047 | Auto绑定/推导/受检HIR，资源基础规则和非法输入拒绝 |
| R3b 基础MIR/CT | 1055 | 1047→Auto基础MIR/verify→1051执行，先交闭合基础函数/CT工厂，解除1048启动环；完整扩面仍1050 |
| R4 语义扩面 | 1048、1049 | 泛型/meta/CT编排与高级语义各自完整实现/阶段验证；高阶CT组合交1050/1053，不以基础CT冒完整CL；可在稳定接口处分工并行 |
| R5 MIR | 1050 | 扩同一1055实现至完整lowering、类型/资源/effect，并关闭高阶CT生成资源/域组合；不绕回Rust HIR lowering |
| R6 服务 | 1051 | 明确 transport 的backend/AV服务、信任边界/资源清理；可与R2–R5并行 |
| R7 驱动 | 1052 | Auto增量依赖、持久缓存恢复、模块编译/链接发布；稳定接口后可与R4–R6并行 |
| R8 Gen1 | 1053 | Gen0编出真实native AAC；Gen1完整CL执行、无Rust源语义回落、真实标准库/领域接缝 |
| R9 独立收口 | 1054 | 同源Gen1→2→3、独立oracle、包/宿主依赖与复现说明；AAC独立签章 |

详细任务、最小前置、各计划AC和验证入口见[路线](strategy/aac-delivery-roadmap.md)与1044–1056。任何子包完成不修改M2完成分母。

## 7. 验证设计与独立验收

语言/库验证复用 M2-A 义务矩阵与既有 TestCase/ACE worker，**不是拷贝所有旧目录就称全覆盖，也不是只重新写自举 happy-path**。保留每条 AC 适用旧义务的去向和独立 expected，补足 Auto主体特有的 CT/依赖/坏缓存/生命周期/协议边界反例。Rust reference是一个对照，被测 AAC 结果还要对独立规范/金样/事件轨迹，不能双方同错仍PASS。

未来拟永久测试资产 `crates/auto-lang/test/cases/aac/`、主体模块测试及 `aac-bridge` 测试；本轮不创建。优先复用现有固定职责组 `ace-syntax`、`ace-source`、`ace-semantics`、`ace-comptime`、`ace-lowering`、`ace-native`、`ace-abi`、`ace-artifacts`，按实际触面完整选择。新增 AAC 命名入口由1053在现固定组机制注册，具体名字未实现前不写成可运行命令。纯语法/语义层用阶段产物与 independent expected独立验收，不强制等待末端native，也不能以其验收代实际执行。

代际链明确为 `Rust AC(Gen0) + S → native AAC(Gen1)`；`Gen1 + 同一 S → Gen2`；`Gen2 + 同一 S → Gen3`。每代绑定源码树/stdlib/schema/backend/runtime/link/target与实际可执行文件hash，记录谁编谁，至少检查同源自编译、全部CL适用测试、CS、编译失败诊断阶段、资源/失败清理、冷/热/恢复。Gen1/2/3不能调用Rust源码入口，也不能只是A2R/A2C生成壳再调用原编译器。

不无条件要求 PE/ELF 逐字节相同：先控制时间戳、路径、编译随机种子、链接metadata/版本等；可复现范围内核字节，否则记录具体非确定字段并核语义IR/对象/程序输出。仅“差不多同大小/exit0”不是固定点。该限制不放宽语义/诊断/资源的确定性要求，也不自动认领消除bootstrap信任攻击。

资源 profile 由1044按M2-A现有合同冻结单请求源字节/节点/深度、CT fuel/内存、消息/输出/并发和父子期限；默认单编译请求/单CT服务串行作为首版CS执行模式，不承诺无限资源。现有更严上限保留，拒绝需区分源码错误、资源拒绝、外部工具/OS失败，不能硬截断为成功。预算变动不得为了通过而临时放宽；超出已授权支持面走具名需求决策。正式规模/性能若需新增，单列专项并沿 `cargo t ace-perf` 显式串行；不作为每Plan日常/收尾门。

1054输出 AAC 独立验收结论、支持面/平台/外部依赖/未覆盖边界；不签 M2、不切所有产品默认、不退役Rust参考。实现review独立于自报完成，review/merge保留原范式、专树及材料清理规则。

## 8. 本轮交付与未决项

本轮交付：本设计、AAC路线、1044–1054 r2与新增1055/1056草案和现有入口的当前排程指针。静态审计发现：完整AAC主体未实现，需求供给未齐，无M2-A稳定证明；旧743库存不够用，现native crate混有前端/资源职责需按符号拆分。对应缺口已由具名计划承接，不把未交能力算PASS。

实施前有限决策：1044冻结具体数据schema/CS语法及预算，1051/T-02选择生产transport；Linux宿主分支依1040等真实交付，不用交叉emit冒运行。首个独立AAC基线是Windows x64 host/native target；Linux及其他host扩面作为1054显式pending/后继，不影响声明的Windows AAC独立验收，不改变M2原平台义务。全CL以该宿主目标下最终M2三端完整语言/库目标验收，不能冻结为A期AC有限子集。以上均不是本轮施工许可。


## 9. r2再次审计：发现、补齐与可执行顺序（2026-11-14）

本轮授权是再次验证与必要规划细化，继续不施工。逐包追真实生产/消费数据流，不只看frontmatter是否无环。观察auto-lang HEAD `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各Plan r1代码指纹保历史取材；没有重新测试实现，也没有M2-A稳定证明。1044–1054修为r2，新增1055/1056；主体范围、CS/CL、M2独立边界保持，新增任务不删除原义务。

| Finding | 实际问题/证据 | 当前修订与负责关闭的任务 |
|---|---|---|
| F01 CT启动数据流缺前置 | `source-cte/src/executor.rs` 实际为resolve/verify→`lower_checked`→MIR/ABC→AV；原1048需要MIR却把1050排其后 | 新1055从1047生产基础Auto MIR/CT工厂，1048取1055；1050继承同实现扩全域。高级CT源到执行组合明确1050/T-09、1053/T-04，不准借Rust lowering |
| F02 parser/装配/CT边界不清 | 1046原称#in装配完成，但选择/生成/重绑定涉及1047/1048；函数互递归不能当非法模块循环 | 1046/T-04只保条件AST/候选目录；1047基础绑定；1048/T-09固定点；1046/T-09及1047/1050保合法近邻与最终准入 |
| F03 hash/codec供应泛化 | `artifacts/src/identity.rs`和`source-query/src/ident.rs`用BLAKE3及明确域/长度帧；旧`stdlib/auto/hash.vm.at`仅legacy声明，不证明新库Auto实现；975有JSON不代表这些协议齐 | 新1056精确补字节算法/codec和库单源，1045/1051/1052消费；若新增公共库能力保Design56后端/provider覆盖，不改M2-A分母 |
| F04 物理持久层与信任混淆 | `artifacts/Cargo.toml`实际含auto-cache/rusqlite/bundled SQLite；不能把原Rust ManagedStore整个委托出去或把SQLite页面当Checked | 1044/T-03/09冻logical identity与physical namespace/format及存储原语边界，1052/T-03/09实现Auto策略与恢复重验；文件/SQLite实现选择不扩大成产品数据库迁移 |
| F05 驱动缺MIR正式输入 | 1052要真build/check，却原depends_on没有1050，文字也未列lowering生产者 | 1052显式依1050，并取1056；基础查询/事务可先准备，完整验收须已交Auto MIR/实际backend及链接发布 |
| F06 语言/库/FFI需求有空白 | CS库表未展开argv/env/cwd、精确数值/浮点位、format/parse、UTF-16位置、逻辑来源与SnapshotId、canonical Atom/HIR和现AC公开输入/产物、语言级pass、规范化foreign/自动binding、生成源码闭包；CL不能仅统称完整 | 1044/T-09逐行完整矩阵，1045/T-09、1046/T-09、1047/T-09、1049/T-09与1052/T-09具名承担，不扩成第二stdlib/包管理器或Rust/C parser |
| F07 平台/权限轴不完整 | 默认host=target测试容易把CT主机布局当目标布局，把compiler文件权限继承给CT | 1044冻build/host/target/CT域；1051/T-09核ABI/layout、capability/预算；`sys.current`沿语言target合同，host另名；只验证首版Windows不冒Linux运行 |
| F08 测试owner和启动时机含糊 | 实际协议在`crates/auto-test-case`，adapter在`crates/auto-lang-core/src/test_cases`，解析器是`vendor/autodown-test-parser`；auto-down不是TestCase执行owner | 1044/T-10冻结所有权，早期每包首次driver任务注册/实际执行；1053汇总整机，不以旧Rust组绿证明Auto阶段已测 |
| F09 禁Rust入口观察窗口易误杀Gen0 | Gen0编译S本就用Rust源码前端；Gen1处理被测源时才必须禁Rust源码语义 | 1053/T-02/09明确两窗与真正源码入口阻断/计数；1051安全MIR复验保留，不误计成回落。外部Rust/C源码工具不准解析Auto源 |
| F10 代际/证据/确定性验收不够具体 | 不能拿Gen2绿报告充Gen3、用“二进制可不同”掩盖指令/常量错误；1054原通用AC不能妨碍其签独立AAC | 1054/T-09核各代exe/input identity，metadata例外事前具名且不屏蔽语义差异；work只产unsigned候选，真实独立review和落主身份核准后才签Windows AAC，仍不签M2/AAV/AACE |

### 9.1 无环的数据流与阶段出口

单执行者建议：`1044 → 1056 → 1045 → 1046 → 1047 → 1051 → 1055 → 1048 → 1049 → 1050 → 1052 → 1053 → 1054`。1051可在1045后与1046/1047并行。1048算法通过1044定义的LowerCt接口，先注入1055基础实现；1050/T-09再注入完整实现核高级CT。**1048不反import1050，1055不依1048/1049**。每包只在其profile内签阶段结果；最终Gen1仍核全部CL适用义务，无任何CT/资源/域功能因拆包删除。

标准库1056是AAC专需操作/消费差额的后继，不反置为985/A新前置。已由M2-A交的等价操作原样复用；确新增公共API按原库版本/装配和覆盖合同，AAC自身依赖闭包绑定新版本。M2-B可继续冻结旧兼容输入，不因AAC支撑包未交而等待。

### 9.2 主体、平台与跨仓输入的明确边界

编译器source闭包包括Auto主体、生成源码、标准库/CT库、静态资源和声明的FFI/runtime；本链语法投影/类型/资源/CT/增量策略不得藏在Rust helper。完整CL的语言族与操作须逐行映射到1047/1048/1049/1050生产点和1053执行oracle，不能只拿“支持完整Auto”一句话代分母。

| 实际owner/已核来源 | 输入身份/用途 | AAC任务与可执行关闭条件 |
|---|---|---|
| auto-lang `crates/auto-test-case/Cargo.toml`、`vendor/autodown-test-parser/PROVENANCE.md`、`crates/auto-lang-core/src/test_cases/{mod,ac_av}.rs`、`experimental/ace/crates/case-worker/` | 协议core/厂商解析器/语言adapter与ACE worker；输入/oracle按现协议保字节及来源身份 | 1044/T-10、各包driver任务、1053/T-03；真跑Auto阶段/整机并使漏行/假PASS/错工具/emit冒run反控红，不要求auto-down代码改变 |
| auto-man `crates/auto-package-lock/src/{model,codec,verify}.rs`与`docs/specs/modules/generated-cargo-context.md`（观察54fe4070bdbd） | 声明包闭包/锁身份与构建装配，非AAC编译策略 | 1052/T-09消费冻结manifest/锁包，幽灵依赖拒绝；未供给消费接口则登记原owner的具体缺口/关闭结果，不自建联网包管理器 |
| auto-ui `docs/specs/auto-graphics/ace-migration-feasibility.md`、`docs/specs/runtime/rq-execution-input-and-host-services.md`（观察93e6b924749a） | UI/AGE域接缝与真实source/host oracle；旧067 no-go只是历史研究，不可当新域已支持证据 | 1044锁M2-A真正已交替代接口，1049语义/1050 MIR/1053实际域执行；不等整个UI产品迁完，不能把域前端留Rust然后叫runtime |
| auto-web `docs/specs/project.md`、`docs/specs/testing.md`（观察8b1742a0f84c） | Web语言owner边界和原实际执行oracle | 1053选M2-A相应具名消费者，沿owner入口核包/资源，不把完整Web App迁移加AAC门 |
| auto-os `docs/plans/064-ace-desktop-and-app-migration-batches.md`（观察868c80d3e697） | 原A/B批次边界及已交最小host/应用oracle | 1044只消费985/A要求的实际已交切片；1053无需等064整包，064/M2-B不反等AAC |
| auto-down `docs/specs/00-overview.md`（观察58ea33f699ef） | 文档格式owner；实际vendored parser版本以语言仓PROVENANCE/源码为准 | 1044/T-10分清来源；仅协议真缺表达力才具名分owner后继，不能预设全测试框架重写 |

本轮外仓全部只读，以上commit只是观察，不是稳定证明；源码/SDK/接口/域签章在G-AAC后重新绑定。未来确需外仓代码修改须其真实owner Plan与单仓单树交接，本轮不建外仓任务或树。host/target/CT/type-layout/ABI及各provider版本都入身份；只支持Windows首版仍保全CL在该目标下的完整适用面。

### 9.3 验收与资源补强

所有Plan新增AC-07（1044另有AC-08）及对应V补充，将额外work任务置于本包T-06之前。标准是本阶段required输出/用例全关闭；未测、未知、已记pending均不等于PASS。source/grammar/span只是语法事实，未决CT/泛型/资源不可产全模块Checked。类型化中间数据经wire/磁盘后须按消费方安全规则重验，不共享进程内capability。

阶段、整机、代际的观察窗口/可接受保留Rust项分别记录。Gen1/2/3实际exe哈希不同就不能复用跨代执行结论；可复现metadata例外事前定义，不能用它掩盖错误的MIR、常量、机器指令或重定位语义。边界样例、合法近邻、内存/句柄/进程回收及坏输入前置拒绝是正确性验证，不因此每包附加正式ace-perf套件。

本轮仅文档审计：修订1044–1054并新增1055/1056，所有work/review/merge任务未执行。静态核验最终收据见AAC路线§6；这不是独立实现review或自举验收。
