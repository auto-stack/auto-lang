# ACC 自举主体与 HIR 阶段契约（auto-acc-bootstrap-contract）

> 状态：📝 Draft（2026-10-04；PLAN-743 T-04 产出）。本稿是**契约文档**：定义阶段边界、
> pass 合同模板、非法变换反例与桥接候选矩阵，供后续实施计划引用和独立复审；
> 不声明任何新阶段已实现，不冻结字段编号，不实施桥接，不改写 741 交付的验收范围。
> 能力清单的取证与模块去留见 PLAN-743 报告
> `docs/reports/743-acc-hir-contract/inventory.md`（T-03）。
> 关联：[AutoHIR 语义](auto-hir-design.md)、[后端演进](auto-native-backend-evolution.md)、
> [AC 子集](auto-ac-subset-and-target.md)、[Native ABI RFC](auto-native-abi-rfc.md)、
> Specs：[auto-hir](../../specs/auto-hir/project.md)、[auto-ac](../../specs/auto-ac/project.md)。

## 0. 定位与不变式

ACC = Auto 实现的编译器主体。本契约给"源码 adapter、运行时、ABI、ACC 迁移"四类后续工作
提供共同的前置边界。四条不变式贯穿全文：

1. **Checked 门唯一**：后端与一切下游只消费 `verify::verify` 产出的 `CheckedModule`
   （experimental/ac-core/src/verify.rs:59）；输入中的任何 `checked` 类标记无授权作用
   （schema/core-i32.atom 头部约定）。改写后的 HIR 必须重新 verify，旧凭证作废。
2. **维度分开**：编译器源码用什么语言能力（compiler-source-demand）与编译器能编译什么
   （compiled-language-support）是两个清单，禁止互相推导（取证见 inventory.md §0）。
3. **公共/低层分界**：公共 HIR 不泄漏 CLIF、寄存器、桥接私有布局或 Rust 类型内存表示；
   低层 IR/SSA 属 v0.7 自主线（后端演进战略 §5），不在本契约发明实现。
4. **不冒充**：AAVM/AA2R 自举（auto/lib 七文件、a2r 自转译）不构成 AC/ACC 自举证据；
   旧 profile 的绿语料不自动成为新阶段的通过凭证。

## 1. 阶段契约（stage table）

~~~text
S0 源码工作态（前端私有）
S1 Schema-bound Atom 文本
S2 bound（类型化 Unchecked HIR Bundle）
S3 semantic verify
S4 Checked 公共 HIR
S5 profile 语义 lowering + 目标能力门
S6 后端发射/链接/运行（Cranelift adapter 消费）
~~~

不要求为每层新建一套 HIR：S2–S4 共用 `hir.rs` 的 Bundle 数据模型，S5/S6 共用 lowering
产物形态。分层约束的是**信息可见性与验证义务**，不是数据结构份数。

| 阶段 | 合法输入 → 输出 | 允许未决项 | 身份/profile | 验证入口 | 失败诊断 | 消费者 | 现状 |
|---|---|---|---|---|---|---|---|
| S0 源码工作态 | `.at` 源文本 → 词法/语法/未解析名字的私有工作结构 | 未解析引用、推导中类型、部分失败 | 无公共身份（前端私有） | 前端自检（语法层） | 前端私有 | S0→S2 adapter | **未实现**（旧 ast.rs/parser.at 均非本阶段形态； inventory.md MD-203） |
| S1 Atom 文本 | 符合 descriptor 的 Atom 文本 → SyntaxNode/Atom | 无（身份三重不齐即拒） | `auto.hir.core.draft`/rev 1/`core-i32-draft`（schema/core-i32.atom） | atom_text 解析+descriptor 校验 | `text/*` 阶段码，`file:line:col` | S2 | **已实现**（core-i32） |
| S2 bound | S1 → 类型化 Unchecked `hir::Bundle`（hir.rs：Module/Def/Body/Expr/Place/Stmt） | 语义合法性未验（引用已按类别+owner 绑定） | 同上；文本 ID 已转类别化 ID | binder（类别失配/悬空引用拒绝） | `bind.category-mismatch` / `bind.dangling-ref` | S3 | **已实现**（core-i32，单模块） |
| S3 semantic verify | S2 Bundle → `CheckedModule` | 无（全或无） | 同上 | `verify::verify`（verify.rs:59） | `verify/*` 阶段码 | S4 唯一入口 | **已实现**（core-i32） |
| S4 Checked 公共 HIR | `CheckedModule`（私有构造；bundle()/into_bundle()/required_capabilities()） | 无未决项；Error/Unknown/悬空引用**禁止**以成功态存在 | 版本化 descriptor 身份随行 | 再验=重走 S3 | 不适用（成功态即无未决项） | S5/S6、A2X/VM 探针（远期） | **已实现**（core-i32 profile 域） |
| S5 lowering + 能力门 | S4 → 目标形态（CLIF/对象）；`requires` 须由显式 `--capability` 供给 | 无 | target/profile 显式（x86_64-pc-windows-msvc） | `native::capabilities_check`（缺失=`capability.missing`） | capability/backend 阶段码 | S6 | **已实现**（core-i32/Windows 测试 profile）；通用 pass 框架**未实现** |
| S6 发射/链接/运行 | 对象 → COFF → PE → 执行 | 链接环境（SDK 路径）外部依赖 | 产物收据（`.ac-link.txt`） | `link_object`/`run_exe`（硬截止时间） | link/run 阶段码 | 用户/测试 | **已实现**（测试 profile） |

**阶段规则**：

- R1 每阶段输出携带产生它的阶段身份（schema/profile/revision + pass 版本链，见 §2）；
  消费方不得跨阶段取用（例：S5 不得接受 Unchecked Bundle）。
- R2 来源映射（SourceId/SpanId，auto-hir-design.md §9）自 S1 起随行；S0→S2 的 adapter
  **必须**为合成节点标注生成关系，否则 S4 的诊断义务不成立。
- R3 语义/效果/资源义务在 S3 定案（Effect 枚举，hir.rs:139）。**效果分析**与**效果
  语义保持**是两件事，不混用"收窄/放宽"方向词：
  (a) 静态效果*分析*从保守侧出发（未知 effect 一律按 observable 处理），
      向 pure 的精化必须携带证明——这是分析精度问题，不是程序改写；
  (b) 效果*语义保持*对任何 pass 都是双向禁止：既不得为程序引入原本不存在的
      可观察效果，也不得消除/重排已存在的可观察效果（含溢出 trap 与其前驱
      observable 的相对顺序）。
  S5 lowering 不得删除、合并或重排任何 S3 已定案的 observable 边界。
- R4 失败路径与成功路径同相等保真：任何阶段的拒绝都必须是显式诊断，禁止以空输出/
  未实现分支静默通过（741 CLI 已按 0/1/2 区分，main.rs）。
- R5 显式 `Dynamic` 若未来支持，必须有独立节点/类型身份与明确运行时操作语义；
  与"推导失败"永远不同态（auto-hir-design.md §3 继承并升格为本契约规则）。
- R6 多模块 Bundle 装配（跨模块 DefRef）在 S2/S3 之间的接入点预留；741 现状单模块，
  契约要求未来扩展**不得**改变 S3 的全或无语义。

## 2. Pass 合同模板

~~~yaml
pass:
  id: <域>.<名称>            # 如 lower.desugar-while
  version: <u32>             # 语义行为版本；行为变化必须升版
  input_stage: S4            # 只接受该阶段凭证 + 其身份链
  output_stage: S4 | S5      # S4→S4 变换必须重验后才能再称 Checked
  profiles: [core-i32-draft] # 支持的 profile 白名单；其余拒绝
  preconditions:             # 前置条件（结构/分析可用性）
  postconditions:            # 后置条件（机器可验断言清单）
  order:                     # 与其它 pass 的偏序约束
  semantics_preserved:       # 语义保持义务（见下）
  effects_traps:             # effect/trap 传播规则；未知 effect 按保守处理
  call_order:                # 调用求值顺序义务
  drops:                     # 释放/资源义务
  source_maps:               # 来源保持义务（span 去向）
  analysis_invalidation:     # 使哪些缓存/分析失效
  reverify: required|n/a     # S4→S4 一律 required
  diagnostics:               # 失败报告的阶段码/形态
~~~

语义保持义务基线（全部 pass 默认继承，除非显式声明更窄并给出证据）：

- 精确宽度与溢出语义（含 `overflow: trap` 的独立错误退出路径，ExitProcess(70) 测试约定）；
- 短路求值与条件/循环分支不提前求值；
- 调用实参按源码求值顺序（`eval_args` 左→右存临时，741 语义）、命名参数不按字段/形参
  声明序重排副作用；
- place/rvalue 区分、mutable/readonly（Access）、初始化谓词（741 块结构规则）；
- 未执行分支的 trap 不得提升为无条件错误；显式未定义行为不得被变换"顺手定义"；
- 未知 effect 一律按 observable 处理（保守缺省，禁止假设 pure）。

### 2.1 合同示例一：规范化 pass `norm.canonical-form`

~~~yaml
pass:
  id: norm.canonical-form
  version: 1
  input_stage: S2            # 作用于 Unchecked Bundle 的结构规范化
  output_stage: S2
  profiles: [core-i32-draft]
  preconditions: 输入通过 S1/S2 绑定；无悬空引用
  postconditions: 内建简写与内建引用归一（TypeUse 等价，verify.rs 现行规则）；
    表登记顺序保持 ID 分配不变；无节点增删
  order: 先于 S3；不得与 S5 组合跳过 S3
  semantics_preserved: 表达式图拓扑与 owner 关系逐点不变
  effects_traps: 无 effect 重写
  call_order: 不触碰 eval_args 顺序
  drops: 不引入释放
  source_maps: span 原样保留；禁止合并/去重 span
  analysis_invalidation: 无跨 pass 缓存（首个 pass）
  reverify: n/a（仍在 S2）
  diagnostics: 不产生失败（结构已验）；如遇违约=工具 bug，报 internal 阶段码
~~~

### 2.2 合同示例二：常量求值 pass `eval.const-fold`

~~~yaml
pass:
  id: eval.const-fold
  version: 2                 # r2：运行期 trap 边界消歧（QA-04）
  input_stage: S4            # 只对已验证模块操作
  output_stage: S4
  profiles: [core-i32-draft]
  preconditions: CheckedModule；折叠域仅限"普通运行期表达式"，即语言未规定
    必须编译期求值的表达式（显式 comptime/const 语境不在本 pass 域内，
    由该语境自己的独立契约治理）
  postconditions: 仅替换可证纯（Effect=pure）且类型精确的表达式；折叠在
    编译期算出的结果值本身不得触发溢出/trap——会溢出/出错的常量表达式
    保持运行期 trap 语义（原样保留或降级为等价 trap 节点），一律不在
    编译期拒绝、不产生编译期诊断、不改变错误阶段
  order: 在全部 S4→S4 变换之后、S5 之前
  semantics_preserved:
    - 求值位置唯一性不变（ExprId 单 owner，741 规则）
    - add_i32/mul_i32 的 overflow: trap 保持运行期语义：
      无论溢出常量表达式位于必经路径还是条件分支，本 pass 都不得将其
      提升为编译期错误。语义反例（QA-04）：
      mark_a(); return MAX_I32 + 1（overflow: trap）——约定语义是先观察
      a 再 ExitProcess(70)；若以"必经路径"为由编译期拒绝，a 消失且错误
      阶段改变，即违反 X9
    - 不可达分支中的常量溢出不提升（X1）；折叠后控制流结构不变
    - lt_i32 结果折叠为 bool 常量，不改变后续分支谓词结构
  effects_traps: observable/pure 之外（未知）一律不折叠；trap 是可观察
    行为的一部分，受 R3(b) 双向禁止约束
  call_order: 跨调用边界不折叠（函数无 purity 摘要前）
  drops: 不引入
  source_maps: 折叠结果 span 保留原表达式 span，另附合成标记
  analysis_invalidation: 失效常量表缓存与依赖它的可达性分析
  reverify: required（输出重走 S3 才能再入 S5）
  diagnostics: 本 pass 不引入编译期诊断（运行期 trap 语境无"不安全常量"
    概念）；显式 comptime 语境的编译期诊断属该语境契约，不归本 pass
~~~

**不实施声明**：以上两例是合同文本，不是已实现 pass；实现计划须按模板补全
pre/post 断言的可执行形态并重验。模板字段与两例为**合同要求**（目标态），
当前不存在任何已实现 pass——阶段表的"现状"列是实现状态的唯一权威。

## 3. 非法变换反例（复审用负面清单）

| # | 反例 | 违反的规则 |
|---|---|---|
| X1 | `if c { … overflow-trap … }` 被常量折叠为无条件编译错误 | 未执行分支 trap 提升（§2 义务基线；战略 §3） |
| X2 | 命名参数调用按 record 字段声明序/形参声明序重排实参求值 | 调用顺序义务（741 eval_args 左→右；auto-hir-design §6.2） |
| X3 | S4→S4 变换后沿用旧 CheckedModule 直接送 S5 | R1 + Checked 门唯一（§0.1；reverify required） |
| X4 | 输入文件带 `checked: true` 标记试图跳过 verify | §0.1（descriptor 未知字段拒绝已是 741 行为） |
| X5 | 未知 effect 的调用被当作 pure 参与重排/折叠 | 保守 unknown 策略（§2 基线） |
| X6 | 公共 HIR 节点新增指向 CLIF 值/寄存器的字段"便于后端" | §0.3 公共/低层分界 |
| X7 | 桥接层把 Auto 源码片段传给后端并让后端做名字解析/类型检查 | §4 后端职责边界 |
| X8 | 校验关口读输入里的 pass-version 链即认定"已验证" | 凭证只能来自本进程 verify 的成功返回（§0.1） |
| X9 | 必经路径上的常量溢出表达式被编译期拒绝（`mark_a(); return MAX_I32+1` 从"先观察 a 再 ExitProcess(70)"变成不带 a 的编译错误） | 运行期 trap 保持（§2.2；R3(b)——trap 与其前驱 observable 的顺序不可变） |

## 4. 后端桥候选矩阵（不实施，仅契约）

候选 A：**薄 Rust C ABI（进程内）**——ACC（Auto）经受限 C 函数面调用 Rust 侧
Cranelift 消费库；候选 B：**独立后端进程**——以批处理作业形态交换编译单元。

| 维度 | A 进程内 C ABI | B 独立后端进程 |
|---|---|---|
| 输入 | Checked HIR 的 ABI 化投影 + 显式 profile/target/能力集 | 同左，载体可选 Atom/Batom 编码（战略 §4） |
| 输出 | 对象码缓冲 + 产物收据 | 对象文件 + 收据文件 |
| 版本/身份交换 | 函数面版本号 + capability 字符串协商 | 协议版本 + 握手帧；不匹配即拒 |
| 能力拒绝 | 返回错误码（capability.missing 语义等价） | 协议级拒绝帧 |
| 所有权/错误 | 明确宽度的缓冲区/句柄 + 调用方释放契约；无 Rust Vec/String/enum 私有布局穿越 | 消息边界即所有权边界；错误带阶段码 |
| 批量开销 | 单元调用成本最低；大批量需分块约定 | 每进程启动成本固定；批量吞吐优 |
| 故障隔离 | 无（后端崩溃=编译器崩溃） | 进程级隔离，可重试 |
| 演进耦合 | 编译器与后端同生命周期，ABI 面小 | 版本独立演进；需维护协议兼容层 |
| 可测试性 | 进程内桩注入 | 黄金输入/输出文件对拍最直接 |

**选择条件（满足其一即重新评审）**：编译驱动需要 <ms 级高频小调用且批量化不可行 → A；
后端需独立版本/故障隔离/跨仓交付 → B；跨进程数据载体（Atom/Batom）未定型 → 先 A 后 B
演进；两者都不得复用 Rust 容器私有布局，字段编号一律不冻结。
**工具协议 vs 生产模块 ABI**：本节全部属于编译器工具内部桥接；生产 Auto 模块 ABI
（布局/调用/热重载）按 auto-native-abi-rfc 单独演进，两者禁止互相冒充。
**后端职责边界**：后端不做名字解析/typecheck/优化决策（X7）；它只消费合法投影并拒绝
能力外请求。

## 5. 自举代际判据（Gen1→Gen2→Gen3）

| 代际 | 定义 | 验收要点 |
|---|---|---|
| Gen1 | Rust AC 工具链编译 ACC 第一代（Auto 写的编译器主体） | ACC 主体清单（inventory.md §1/§2）全部经源码→native 管线编译；不要求 ACC 编译任何东西 |
| Gen2 | 第一代 ACC 编译 ACC 自身源码 → 第二代 | 同一 ACC 源码、同一语料：两代产物在约定语义/诊断/HIR 形态上等价 |
| Gen3 | 第二代再编译自身 + 全语料回归 | 代际不动点：Gen2=Gen3（语义域）；字节固定点仅在路径/版本/时间戳可复现约定后作为附加门 |

比较域是**约定语义、诊断与 HIR 形态**，不是任意构建的字节；A2R 转译中转（Rust 文本产物）
不算任何一代的 native 自举。保留依赖（Cranelift 桥、系统链接器、宿主内建）的来源与版本
清单随代际收据归档；自举不等同全部依赖改写为 Auto。

## 6. 与 741 及后续工作的关系

- 741 交付（S1–S6 的 core-i32 测试闭环）是本契约唯一"已实现"列的事实来源；本契约不扩充
  741 的任务/AC，也不要求其返工。
- S0→S2 adapter、多模块装配、pass 框架、桥实现均为**未实现**状态，由后续实施计划
  （NEXT-B/C 候选，见 next-work-packages.md）按本契约认领；认领时在计划文档引用本节编号。
- 本契约的修订走 auto-plan:new 契约修订流程；阶段表新增列、pass 模板字段增删、
  代际判据变化都属契约修订，不是执行期裁量。

## 7. 未决清单（显式不冻结）

桥编码与字段编号、HirTypeId 全集、多模块 Bundle 身份、Effect 全集（现有
observable/pure 两值之外的格）、所有权/释放义务的精确语义（probe-rt-own）、
int/char 宽度映射（probe-int-width/probe-char-int）、字节固定点的可复现条件。
以上在对应实施计划立项时裁定；本契约只保证"未决即拒绝"，不存在隐式缺省。
