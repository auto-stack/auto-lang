# ace-migration: replacement scope and evidence

> Status: proposed (PLAN-922 SD-01/SD-02)。本文件只规范 M2 替换盘点的**口径与证据边界**，
> 不把任何条目标为 implemented；实现状态以各模块 Specs 与已交付计划收据为准。
> 来源：[PLAN-922](../../plans/archive/922-ace-m2-replacement-inventory-and-critical-path.md)（merge 时按实际路径修正）、
> [M2 合同 r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)、
> [v0.6 发布门增补](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)。

## SD-01 全场景分母、分类与证据边界

### 分母构成（迁移台账的强制行集）

M2 替换分母 = 当前真实可达的：

1. **语言公共入口**：auto-lang facade 全部执行/转译 pub API、CLI 子命令与 runner 分派、
   REPL/持久会话/daemon、测试工具入口；
2. **领域消费者**：AutoUI（全部已支持模式/后端）、AutoMan（全部命令/求值/生成）、
   auto-web（服务宿主链）、AutoGen/SDK/模板、其余仓与安装物中真实调用旧引擎者；
3. **全部在用 A2X 目标**：Rust/C/Python/JavaScript/TypeScript/GDScript/Godot/TSCN 与
   r2a 逆向工具，按真实生产/测试/研究归属区分；
4. **发行/安装面**：默认与 all-features 构建、注册/feature/隐藏开关/fallback、
   包内二进制、安装/重定位/升级/回滚路径。

规则：以"入口 → producer → consumer"双向可追溯为准入；grep 零命中、目录名、
examples 计数均不构成覆盖证明。跨仓不可读/owner 未明/未发布安装物按具名 unknown
登记，不填"支持/无差额"。

### 状态分类（每行必选其一，禁混合百分比）

| 类别 | 判据 |
|---|---|
| implemented-verified | 新 ACE producer 存在且同一来源身份下有已执行独立 oracle 证据 |
| implemented-unwired | producer 已实现但该入口/consumer 仍走旧引擎（接线缺口） |
| evidence-stale | 声称已迁但证据非当前祖先/来源身份不适用/未重验 |
| capability-missing | 新引擎无该能力（须给最小真实工程切片与独立 oracle） |
| unknown | 未盘点/不可达/依赖未发布（须给 owner 与下一步只读核查） |

### 证据边界

- 运行证据必须绑定 repo/SHA/入口身份；历史 M1 收据对 M2 只作输入不作完成证明。
- "审计完整" ≠ "产品通过"：本口径下的任何登记都不替代 REP/V06 门的实际执行。
- 旧 required 静态格、M1 corpus（binding.tsv）、新 C/M2 门分账保持，不得互相冲销。

## SD-02 新旧义务、中立库与 disable/安装证据归属

- **义务迁移**：旧执行/转译义务经版本化 successor ledger/recipe/cert 迁移到新
  producer；原 source/expected/批准 NA/accepted 不改。旧 golden/证书不冒充新 producer
  身份；disable 后新版日常不为对拍重建旧引擎，追溯靠 Git 历史、独立已发布旧版本与
  行为 oracle。
- **中立库归属**：共享值类型/诊断/词法与文本工具/中立 runtime/native 静态库/渲染器/
  仍需的 Rust 库按真实依赖保留或迁中立 owner；判定依据是"是否夹带旧执行循环或旧
  AST 发射器"，不是目录名含 vm/trans。薄兼容 API 只能转发 ACE，不得包装旧算法。
- **disable/安装证据**：旧实现退出以 default+all-features 依赖图/构建单元/真实
  dispatch/发行包内容/日常测试选择为零为证；安装证据要求 clean 安装/仓外/重定位/
  升级/独立旧版回滚的实际收据，dev worktree、未发布绝对路径、本机旧缓存不算 fresh。
- **owner 交接**：跨仓迁移单写窗由对应仓 owner 以正式合同认领；盘点行只登记接口
  变动与需重核 consumer（带 SHA），快照不复制成新的永恒状态源，后续正式 SHA 到来
  只刷新受影响行。

## SD-03 分母台账与后继责任挂接（PLAN-959 SD-02）

- SD-01 的分母构成/五类状态/证据边界由 [m2-denominator.md](m2-denominator.md)
  落为**现行分母台账**：L（语言语法逐项）/S（标准库域）/K（发行安装冻结副本）新表 +
  922 A–G 行刷新引用；每行带来源身份（repo@rev + file:line）、四类语言分账
  （规范已定/真实支持/已知旧bug差异/未决，另设"无此形"防凭空新增）与行级后继归属。
  本节不改变 SD-01/SD-02 的任何分类与 REP/V06 门。
- **行级后继责任**：每行 required 与 unknown 必须指向正式后继 Plan/批次（945–948、
  960–987、auto-os/064）及其 T-01 决策门；"计划文本未逐名"的项（平台条件、TaskDef、
  Hash 各形态、vscode 模式等）由台账行显式补齐归属，缺行或无 owner 即盘点未完成，
  unknown 仍阻 M2 终签。
- **delta 规程**：台账冻结带源身份；新增/变更按其 §10 以 delta 行追加并使受影响旧
  证据陈旧（只重核受影响行），不重写已冻结行、不复制第二份永恒状态源；台账自身
  绝不声明实现，状态提升只引用后继计划的交付收据。

## SD-04 完整模块包 producer 挂接（PLAN-960 SD-02）

- 分母台账 L-37（use/dep）、L-38（别名/重导出解析）、L-41（全局 store/常量初始化次序/once/
  diamond/失败回滚）与 L-44（模块面）的 ACE 侧**已验证 producer** 为
  [m2-modules.md](m2-modules.md)（能力 `ac.source.modules.packages/v1`，TextRecord 域，
  四端冷运行+恢复收据见 `av::conformance::module_packages_960`）。本节只挂接来源，不改 SD-01
  的分类口径，也不把任何行标为 implemented；状态提升仍只引用 960 归档收据。
- **未闭缺口（具名，带 owner）**：934 事实页类型集不覆盖 text/record（D-960-1，query 层）；
  mn 端全局 cell（D-960-2，ace-native）；a2x 端全局 cell 发射与类型方法片段（A2X 后继）；
  L-42 跨模块泛型实例归 963；L-38 的 `alias`/type alias 语句形归 963；模块+CT 组合归 966。
  以上缺口在台账中保持 required/unknown 原状，不因 960 交付而消失。

## SD-05 数值面 producer 挂接（PLAN-961 SD-02）

- 分母台账 L-01/L-02/L-49（数值类型/字面量/运算转换）的 ACE 侧**已验证 producer** 为
  [m2-numbers.md](m2-numbers.md)：`SourceProfile::Numeric` 显式源码 profile（grammar numeric
  裁剪门控，旧 profile 字节恒等），同型运算纪律、checked 窄化/浮转整、IEEE 浮（÷±0 trap 沿
  legacy、% ±0=NaN 不 trap 的具名不对称）、移位界 trap 的 T-01 冻结矩阵在四端一致落地——AV
  （eval_num/eval_conv）、native（CLIF→COFF→PE）、a2x-rust（真实 rustc）、a2x-c（真实 MSVC
  /W4 /WX），全部真实 dispatch 无 legacy fallback；823 对 closures/collections 族的冻结排除由
  m2-numbers §MIR/wire 取代。本节只挂接来源，不改 SD-01 分类口径，也不把台账行标为
  implemented；状态提升仍只引用 961 归档收据。
- **具名差异（对 legacy）**：混合宽度隐式晋升矩阵不采纳（显式 as/to 替代）；legacy `as`
  截断/clamp 改 checked；位运算/移位为新增冻结面（legacy 无此面）。**未闭缺口（具名，带
  owner）**：semantics 查询通道数值注解具名拒绝 `numeric-annotation`（query 后继）；CTE 值面
  维持 I32/Bool、comptime admission 未扩（具名）；聚合/泛型/动态/控制效应组合面归 962–966。
  以上缺口在台账中保持 required/unknown 原状，不因 961 交付而消失。

## SD-06 组合数据结构面 producer 挂接（PLAN-962 SD-02）

- 分母台账 L-06..L-19/L-48（view/mut/move、Hold、Box、take、R 行、enum/union/tag、grid、tuple、
  array/List 字面量、Option/Result、模式、range、索引 place）的 ACE 侧**已验证 producer** 为
  [m2-aggregates.md](m2-aggregates.md)：`SourceProfile::Aggregates` 显式源码 profile（numeric 全表面
  超集，冻结面字节恒等），准入键=文档级 capability `av.aggregates/1`（mir_verify/native/a2x 同键放宽，
  旧面 822 一层元素域逐字节保持）；tuple=结构内联+匿名位置 record（零 schema/MIR 新指令）、Box=单元素
  List 胞+不透明边递归合法化、grid=列元组+静态维检查、整元素读=PLACE 视图（物化全拒）——四端一致落地：
  AV（真全链+资源账 8 fixture+21 探针腿）、native（真 PE 执行+collections COFF 发射）、a2x-rust（真实
  rustc 4 案例）、a2x-c（真实 MSVC cl 3 案例），全部真实 dispatch 无 legacy fallback；六职责组零代码红。
  本节只挂接来源，不改 SD-01 分类口径，也不把台账行标为 implemented；状态提升仍只引用 962 归档收据。
- **具名差异（对 legacy）**：位置式索引读以 `.get(i)` 为正式读形（`xs[i]=v` 写位已交付；裸读拼写=
  CST 机制待办 D-962-C1）；range 界需空格分隔（affix-num 词法）；union/grid 语义为新定（旧引擎
  parser-only/panic）。**未闭缺口（具名，带 owner）**：递归 Box 的 collections-unit 车道（D-962-C2）、
  C 侧 tuple 调用实参临时物化（D-962-C3）、List use-after-move 不拒（D-962-C4，冻结既有）、杂项诊断
  拼写（D-962-C5）；StructCover 嵌套模式/is 表达式→965；Map/Set 归 964/946-972（963 实例）。

## SD-07 泛型/约束见证/方法实例化 producer 挂接（PLAN-963 SD-02）

- 分母台账 L-24（默认/命名实参）、L-38（alias）、L-39（spec/约束/关联类型）、L-40（ext/
  方法接收者）、L-42（泛型 type/const 参数与跨模块实例）与 L-47（调用语义的函数值面）的
  ACE 侧**已验证 producer** 为 [m2-generics.md](m2-generics.md)：三个能力裁剪 profile
  （`scalar-instances` / `record-instances` / `text-record-instances`）+ `generic-call` 原子；
  935 单参身份字节不变、页 codec v4（值参数 `constc`），旧页按版本 MISS 重产；四端
  （AV / ac-native / a2x-rust / a2x-c）真实运行收据见该文件 V 行具名入口与
  `run-2v_8abue`（2453 Rust 用例 + 10 case check，七组 PASS）。本节只挂接来源，不改 SD-01
  的分类口径，也不把任何行标为 implemented；状态提升仍只引用 963 归档收据。
- **未闭缺口（具名，带 owner）**：管道算子 `|>`（D-963-C1，963 T-08 后续/965）、可变实参
  无用户声明位拼写（D-963-C2，962 collections 裁定）、`#[with(T)]` 属性形迁移拒
  （D-963-C3，language attribute grammar plan）、记录类型模块全局 `.move` 转移
  （D-963-C4，960 模块面）、默认实参求值作用域边界（D-963-C5，963 T-08 后续）、ACE
  workspace rustfmt 门禁（D-963-C6，ACE 治理）、泛型局部注解诊断质量（D-963-C7，ACE
  语法/投影诊断面）。以上缺口在台账中保持 required/unknown 原状，不因 963 交付而消失；
  963 自身覆盖行（L-38/39/40/42 全、L-24/47 部分）不因 B 未交付改 NA。

## SD-08 动态值对象反射 producer 挂接（PLAN-964 SD-02）

- 分母台账 L-16（array/List 字面量）、L-20（对象/map）、L-21（nil/null）、L-22（可调用值）、
  L-23（动态派发）的动态/对象/闭包切片与 FStr 插值的 ACE 侧**已验证 producer** 为
  [m2-dynamic.md](m2-dynamic.md)：`SourceProfile::Aggregates`/`Closures` 显式源码 profile
  （对象=Rc 共享可变单元、nil 统一、DynType::Callable、通用闭包捕获、FStr 编译前文本变换 +
  I32 显示）、`ac-av/aggregates-source-run-v1` worker recipe、INT-1026/IF-VAL 通用外部值交接
  （不持 PyO3/第二账本）。四端按原支持域矩阵（AV 全能力；native/a2r/a2c 仅 FStr/闭包族，
  对象/动态面具名拒）真实运行收据见该文件 V 行具名入口与 plan §9 并集记录。本节只挂接来源，
  不改 SD-01 的分类口径，也不把任何行标为 implemented；状态提升仍只引用 964 归档收据。
- **未闭缺口（具名，带 owner）**：`&&` 链 f-string 条件 lowering 边（D-964-C1，ACE
  lowering/verify 后续）、冻结措辞 `av.dyn.unsupported` 与实现 `portable.unsupported` 的
  命名空间差异（D-964-C2，文档级，语义一致）、grid 单行异构列语义澄清（D-964-C3，962
  语义裁定者）。以上缺口不因 964 交付而消失；964 自身覆盖行保持 required 原状。
